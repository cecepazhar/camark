//! Guard: every place that states CAMark's version agrees with the workspace `Cargo.toml`.
//!
//! The workspace `[workspace.package].version` is the single source. The updater compares
//! against the version Tauri embeds, so a stale copy anywhere means users stop receiving
//! updates or see the wrong version. This check fails on any mismatch.
//!
//! Checked:
//! - `crates/caf-app/tauri.conf.json` `version`: must be absent (Tauri then uses the crate
//!   version) or equal.
//! - `frontend/package.json` `version` (the About screen reads it).
//! - `app.toml` `[app].version` and its generated copy `frontend/src/lib/generated/app.ts`.
//! - `README.md` release badge (shields.io escapes `-` as `--`).
//! - Every `caf-core` path dependency that pins a version.

use std::fs;
use std::path::Path;

/// Crates whose `caf-core` dependency may carry a version requirement.
const CRATES_DEPENDING_ON_CORE: &[&str] = &["caf-app", "caf-cli", "caf-migrator"];

pub fn check_version_sync(root: &Path, failures: &mut Vec<String>) {
    let version = match workspace_version(root) {
        Ok(v) => v,
        Err(e) => {
            failures.push(format!("version_sync: {e}"));
            return;
        }
    };
    let mut fail = |msg: String| failures.push(format!("version_sync: {msg}"));

    match read_json(&root.join("crates/caf-app/tauri.conf.json")) {
        Ok(conf) => match conf.get("version").and_then(|v| v.as_str()) {
            None => {}
            Some(v) if v == version => {}
            Some(v) => fail(format!(
                "crates/caf-app/tauri.conf.json version is {v}, expected {version} (or remove it)"
            )),
        },
        Err(e) => fail(e),
    }

    match read_json(&root.join("frontend/package.json")) {
        Ok(pkg) => {
            let v = pkg.get("version").and_then(|v| v.as_str()).unwrap_or("");
            if v != version {
                fail(format!(
                    "frontend/package.json version is {v:?}, expected {version}"
                ));
            }
        }
        Err(e) => fail(e),
    }

    match read_toml(&root.join("app.toml")) {
        Ok(app) => {
            let v = app
                .get("app")
                .and_then(|a| a.get("version"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if v != version {
                fail(format!(
                    "app.toml [app].version is {v:?}, expected {version}"
                ));
            }
        }
        Err(e) => fail(e),
    }

    let generated = root.join("frontend/src/lib/generated/app.ts");
    if let Ok(text) = fs::read_to_string(&generated)
        && !text.contains(&format!("version: \"{version}\""))
    {
        fail(format!(
            "frontend/src/lib/generated/app.ts does not contain version: \"{version}\""
        ));
    }

    if let Ok(readme) = fs::read_to_string(root.join("README.md")) {
        let badge = format!("badge/release-v{}-", version.replace('-', "--"));
        if readme.contains("img.shields.io/badge/release-v") && !readme.contains(&badge) {
            fail(format!("README.md release badge does not show v{version}"));
        }
    }

    for krate in CRATES_DEPENDING_ON_CORE {
        let path = root.join(format!("crates/{krate}/Cargo.toml"));
        let manifest = match read_toml(&path) {
            Ok(m) => m,
            Err(e) => {
                fail(e);
                continue;
            }
        };
        let req = manifest
            .get("dependencies")
            .and_then(|d| d.get("caf-core"))
            .and_then(|d| d.get("version"))
            .and_then(|v| v.as_str());
        if let Some(req) = req {
            let bare = req.trim_start_matches(['=', '^']);
            if bare != version {
                fail(format!(
                    "crates/{krate}/Cargo.toml requires caf-core {req:?}, expected \"={version}\""
                ));
            }
        }
    }
}

fn workspace_version(root: &Path) -> Result<String, String> {
    let manifest = read_toml(&root.join("Cargo.toml"))?;
    manifest
        .get("workspace")
        .and_then(|w| w.get("package"))
        .and_then(|p| p.get("version"))
        .and_then(|v| v.as_str())
        .map(str::to_string)
        .ok_or_else(|| "Cargo.toml has no [workspace.package].version".to_string())
}

fn read_toml(path: &Path) -> Result<toml::Value, String> {
    let text = fs::read_to_string(path).map_err(|e| format!("read {}: {e}", path.display()))?;
    toml::from_str(&text).map_err(|e| format!("parse {}: {e}", path.display()))
}

fn read_json(path: &Path) -> Result<serde_json::Value, String> {
    let text = fs::read_to_string(path).map_err(|e| format!("read {}: {e}", path.display()))?;
    serde_json::from_str(&text).map_err(|e| format!("parse {}: {e}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    struct Fixture(PathBuf);

    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn write(root: &Path, rel: &str, text: &str) {
        let p = root.join(rel);
        if let Some(parent) = p.parent() {
            fs::create_dir_all(parent).expect("mkdir");
        }
        fs::write(p, text).expect("write");
    }

    /// A tree where every location says `v`.
    fn synced(name: &str, v: &str) -> Fixture {
        let root =
            std::env::temp_dir().join(format!("camark-version-sync-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        write(
            &root,
            "Cargo.toml",
            &format!("[workspace.package]\nversion = \"{v}\"\n"),
        );
        write(
            &root,
            "crates/caf-app/tauri.conf.json",
            r#"{"productName":"CAMark"}"#,
        );
        write(
            &root,
            "frontend/package.json",
            &format!(r#"{{"version":"{v}"}}"#),
        );
        write(&root, "app.toml", &format!("[app]\nversion = \"{v}\"\n"));
        write(
            &root,
            "frontend/src/lib/generated/app.ts",
            &format!("export const app = {{\n  version: \"{v}\",\n}};\n"),
        );
        write(
            &root,
            "README.md",
            &format!(
                "[![Release](https://img.shields.io/badge/release-v{}-06b6d4.svg)](x)\n",
                v.replace('-', "--")
            ),
        );
        for krate in CRATES_DEPENDING_ON_CORE {
            write(
                &root,
                &format!("crates/{krate}/Cargo.toml"),
                &format!(
                    "[dependencies]\ncaf-core = {{ path = \"../caf-core\", version = \"={v}\" }}\n"
                ),
            );
        }
        Fixture(root)
    }

    fn run(root: &Path) -> Vec<String> {
        let mut failures = Vec::new();
        check_version_sync(root, &mut failures);
        failures
    }

    #[test]
    fn synced_prerelease_tree_passes() {
        let f = synced("ok", "1.1.0-beta.1");
        assert_eq!(run(&f.0), Vec::<String>::new());
    }

    #[test]
    fn stale_tauri_conf_version_fails() {
        let f = synced("tauri", "1.1.0-beta.1");
        write(
            &f.0,
            "crates/caf-app/tauri.conf.json",
            r#"{"version":"1.0.25"}"#,
        );
        let failures = run(&f.0);
        assert_eq!(failures.len(), 1, "{failures:?}");
        assert!(failures[0].contains("tauri.conf.json"));
    }

    #[test]
    fn every_location_is_checked() {
        let f = synced("all", "1.1.0-beta.1");
        write(&f.0, "frontend/package.json", r#"{"version":"1.0.50"}"#);
        write(&f.0, "app.toml", "[app]\nversion = \"1.0.25\"\n");
        write(
            &f.0,
            "frontend/src/lib/generated/app.ts",
            "version: \"1.0.25\"",
        );
        write(
            &f.0,
            "README.md",
            "https://img.shields.io/badge/release-v1.0.17-06b6d4.svg",
        );
        write(
            &f.0,
            "crates/caf-cli/Cargo.toml",
            "[dependencies]\ncaf-core = { path = \"../caf-core\", version = \"=1.0.50\" }\n",
        );
        let failures = run(&f.0);
        assert_eq!(failures.len(), 5, "{failures:?}");
    }

    #[test]
    fn readme_badge_needs_escaped_hyphen() {
        let f = synced("badge", "1.1.0-beta.1");
        write(
            &f.0,
            "README.md",
            "https://img.shields.io/badge/release-v1.1.0-beta.1-06b6d4.svg",
        );
        assert_eq!(run(&f.0).len(), 1);
    }
}
