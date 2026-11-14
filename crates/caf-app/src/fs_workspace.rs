//! Native Filesystem Workspace operations for CAMark (P1).

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::sync::Mutex;

pub const SUPPORTED_TEXT_EXTENSIONS: &[&str] = &[
    // Markdown & Documentation
    "md",
    "markdown",
    "mdown",
    "mkd",
    "txt",
    "text",
    "org",
    "rst",
    "adoc",
    "asciidoc",
    // Binary / Office / Web Documents (renderable in preview viewer)
    "pdf",
    "docx",
    "doc",
    "pptx",
    "xlsx",
    "csv",
    "tsv",
    "html",
    "htm",
    // Config & Data
    "json",
    "jsonc",
    "json5",
    "yaml",
    "yml",
    "toml",
    "xml",
    "ini",
    "conf",
    "cfg",
    "env",
    "properties",
    // Web
    "html",
    "htm",
    "css",
    "scss",
    "sass",
    "less",
    "js",
    "mjs",
    "cjs",
    "jsx",
    "ts",
    "mts",
    "cts",
    "tsx",
    "svelte",
    "vue",
    "svg",
    // Programming Languages
    "rs",
    "py",
    "pyw",
    "go",
    "sh",
    "bash",
    "zsh",
    "fish",
    "sql",
    "c",
    "cpp",
    "cc",
    "cxx",
    "h",
    "hpp",
    "hxx",
    "java",
    "php",
    "rb",
    "lua",
    "r",
    "swift",
    "kt",
    "kts",
    "dart",
    "zig",
    "ex",
    "exs",
    "erl",
    "clj",
    // Scripts & Build
    "dockerfile",
    "makefile",
    "cmake",
    "diff",
    "patch",
    "log",
];

pub fn is_supported_file(name: &str) -> bool {
    let lower = name.to_lowercase();
    if lower.starts_with('.') {
        let stem = lower.trim_start_matches('.');
        if matches!(
            stem,
            "env"
                | "gitignore"
                | "gitattributes"
                | "editorconfig"
                | "dockerignore"
                | "bashrc"
                | "zshrc"
                | "profile"
                | "npmrc"
                | "prettierrc"
                | "eslintrc"
        ) || stem.starts_with("env.")
        {
            return true;
        }
    }
    if matches!(
        lower.as_str(),
        "dockerfile" | "makefile" | "license" | "licence" | "readme" | "changelog"
    ) {
        return true;
    }
    if let Some(ext) = lower.rsplit('.').next() {
        if ext != lower && SUPPORTED_TEXT_EXTENSIONS.contains(&ext) {
            return true;
        }
    }
    false
}

static INITIAL_FILE: Mutex<Option<String>> = Mutex::new(None);

pub fn set_initial_file(path: String) {
    if let Ok(mut lock) = INITIAL_FILE.lock() {
        *lock = Some(path);
    }
}

pub fn get_initial_file_path() -> Option<String> {
    INITIAL_FILE.lock().ok().and_then(|mut lock| lock.take())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileNode {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub children: Option<Vec<FileNode>>,
    pub size_bytes: Option<u64>,
    pub modified_at: Option<u64>,
}

pub async fn list_dir(dir_path: String) -> Result<Vec<FileNode>, String> {
    tokio::task::spawn_blocking(move || {
        let path = Path::new(&dir_path);
        if !path.exists() || !path.is_dir() {
            return Err("Directory does not exist".to_string());
        }
        read_dir_recursive(path, 8)
    })
    .await
    .map_err(|e| e.to_string())?
}

pub async fn read_file(file_path: String) -> Result<String, String> {
    tokio::task::spawn_blocking(move || {
        fs::read_to_string(&file_path).map_err(|e| format!("Failed to read file: {e}"))
    })
    .await
    .map_err(|e| e.to_string())?
}

pub async fn read_file_base64(file_path: String) -> Result<String, String> {
    tokio::task::spawn_blocking(move || {
        let bytes = fs::read(&file_path).map_err(|e| format!("Failed to read binary file: {e}"))?;
        use base64::Engine;
        Ok(base64::engine::general_purpose::STANDARD.encode(bytes))
    })
    .await
    .map_err(|e| e.to_string())?
}

pub async fn write_file(file_path: String, content: String) -> Result<(), String> {
    tokio::task::spawn_blocking(move || {
        let path = Path::new(&file_path);
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        fs::write(path, content).map_err(|e| format!("Failed to write file: {e}"))
    })
    .await
    .map_err(|e| e.to_string())?
}

pub async fn create_file(file_path: String) -> Result<(), String> {
    tokio::task::spawn_blocking(move || {
        let path = Path::new(&file_path);
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if path.exists() {
            return Err("File already exists".to_string());
        }
        fs::write(path, "").map_err(|e| format!("Failed to create file: {e}"))
    })
    .await
    .map_err(|e| e.to_string())?
}

pub async fn rename_file(old_path: String, new_path: String) -> Result<(), String> {
    tokio::task::spawn_blocking(move || {
        fs::rename(&old_path, &new_path).map_err(|e| format!("Failed to rename file: {e}"))
    })
    .await
    .map_err(|e| e.to_string())?
}

pub async fn create_directory(dir_path: String) -> Result<(), String> {
    tokio::task::spawn_blocking(move || {
        let path = Path::new(&dir_path);
        fs::create_dir_all(path).map_err(|e| format!("Failed to create directory: {e}"))
    })
    .await
    .map_err(|e| e.to_string())?
}

pub async fn delete_file(file_path: String) -> Result<(), String> {
    tokio::task::spawn_blocking(move || {
        let path = Path::new(&file_path);
        if path.is_dir() {
            fs::remove_dir_all(path).map_err(|e| format!("Failed to delete directory: {e}"))
        } else {
            fs::remove_file(path).map_err(|e| format!("Failed to delete file: {e}"))
        }
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Opens a file directly with its default OS application (e.g. Evince for PDF, LibreOffice for DOCX).
/// This is ideal for binary files that cannot be rendered inside the WebView.
pub async fn open_with_default_app(file_path: String) -> Result<(), String> {
    tokio::task::spawn_blocking(move || {
        let path = Path::new(&file_path);
        if !path.exists() {
            return Err(format!("File not found: {file_path}"));
        }
        let canonical = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());

        #[cfg(target_os = "linux")]
        {
            // Try a chain of launchers: xdg-open (default handler) -> kde-open -> direct app.
            // `xdg-open` can silently fail when no default app is registered for the
            // file's MIME type, or when the GUI app itself crashes (e.g. Qt/Wayland
            // init errors) — so we wait briefly for each spawned child and check the
            // exit status before falling through to the next option.
            fn try_launch(cmd: &str, args: &[&std::ffi::OsStr]) -> Result<(), String> {
                use std::io::Read;
                let mut child = std::process::Command::new(cmd)
                    .args(args)
                    .stdout(std::process::Stdio::piped())
                    .stderr(std::process::Stdio::piped())
                    .spawn()
                    .map_err(|e| format!("Failed to launch {cmd}: {e}"))?;
                // Wait up to 5s: if the app is still running after that, it took over
                // successfully. If it exits fast, inspect the status + stderr.
                let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
                let mut stderr_buf: Vec<u8> = Vec::new();
                let mut wait_result: Option<std::io::Result<std::process::ExitStatus>> = None;
                loop {
                    match child.try_wait() {
                        Ok(Some(status)) => {
                            wait_result = Some(Ok(status));
                            break;
                        }
                        Ok(None) => {
                            // Still running — check for stderr so far
                            if let Some(mut err_pipe) = child.stderr.take() {
                                let mut chunk = [0u8; 1024];
                                if let Ok(n) = err_pipe.read(&mut chunk) {
                                    if n > 0 {
                                        stderr_buf.extend_from_slice(&chunk[..n]);
                                    }
                                }
                            }
                            if std::time::Instant::now() >= deadline {
                                // Give up waiting — process is alive, treat as success
                                return Ok(());
                            }
                            std::thread::sleep(std::time::Duration::from_millis(200));
                        }
                        Err(e) => {
                            wait_result = Some(Err(e));
                            break;
                        }
                    }
                }
                let status = match wait_result {
                    Some(Ok(s)) => s,
                    Some(Err(e)) => return Err(format!("try_wait failed for {cmd}: {e}")),
                    None => unreachable!(),
                };
                if status.success() {
                    return Ok(());
                }
                // Drain any remaining stderr
                if let Some(mut err_pipe) = child.stderr.take() {
                    let mut chunk = [0u8; 4096];
                    while let Ok(n) = err_pipe.read(&mut chunk) {
                        if n == 0 { break; }
                        stderr_buf.extend_from_slice(&chunk[..n]);
                    }
                }
                let stderr_str = String::from_utf8_lossy(&stderr_buf);
                return Err(format!(
                    "{cmd} exited with {status}. stderr: {}",
                    stderr_str.trim()
                ));
            }

            let path_str = canonical.to_string_lossy().into_owned();
            let path_arg = std::ffi::OsStr::new(&path_str);

            // 1st: xdg-open (default handler registry)
            match try_launch("xdg-open", &[path_arg]) {
                Ok(()) => return Ok(()),
                Err(primary_err) => {
                    eprintln!("[open_with_default_app] xdg-open failed: {primary_err}");
                    // 2nd: kde-open (KDE-specific, often more reliable for Office/PDF on KDE)
                    match try_launch("kde-open", &[path_arg]) {
                        Ok(()) => return Ok(()),
                        Err(e2) => {
                            eprintln!("[open_with_default_app] kde-open failed: {e2}");
                            // 3rd: direct fallback based on extension
                            let ext = std::path::Path::new(&path_str)
                                .extension()
                                .and_then(|e| e.to_str())
                                .unwrap_or("")
                                .to_ascii_lowercase();
                            let direct_app: Option<(&str, &[&std::ffi::OsStr])> = match ext.as_str() {
                                "pdf" => Some(("okular", &[path_arg])),
                                _ => None,
                            };
                            if let Some((app, args)) = direct_app {
                                if let Ok(()) = try_launch(app, args) {
                                    return Ok(());
                                }
                            }
                            return Err(format!(
                                "All openers failed for {path_str}.\n  xdg-open: {primary_err}\n  kde-open: {e2}"
                            ));
                        }
                    }
                }
            }
        }

        #[cfg(target_os = "macos")]
        {
            std::process::Command::new("open")
                .arg(&canonical)
                .spawn()
                .map_err(|e| format!("Failed to launch open: {e}"))?;
            return Ok(());
        }

        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            use std::process::Command;
            let _ = Command::new("explorer")
                .arg(format!("/select,{}", canonical.display()))
                .creation_flags(0x0800_0000) // CREATE_NO_WINDOW
                .spawn();
            return Ok(());
        }

        #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
        {
            return Err("open_with_default_app not supported on this OS".to_string());
        }
    })
    .await
    .map_err(|e| e.to_string())?
}

pub async fn reveal_file(file_path: String) -> Result<(), String> {
    tokio::task::spawn_blocking(move || {
        let path = Path::new(&file_path);
        let canonical = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
        #[cfg(target_os = "linux")]
        {
            if std::process::Command::new("nautilus")
                .arg("-s")
                .arg(&canonical)
                .spawn()
                .is_ok()
            {
                return Ok(());
            }

            let uri = format!("['file://{}']", canonical.to_string_lossy());
            if std::process::Command::new("gdbus")
                .args([
                    "call",
                    "--session",
                    "--dest",
                    "org.freedesktop.FileManager1",
                    "--object-path",
                    "/org/freedesktop/FileManager1",
                    "--method",
                    "org.freedesktop.FileManager1.ShowItems",
                    &uri,
                    "",
                ])
                .spawn()
                .is_ok()
            {
                return Ok(());
            }

            std::process::Command::new("xdg-open")
                .arg(canonical.parent().unwrap_or(&canonical))
                .spawn()
                .map_err(|e| format!("Failed to open file manager: {e}"))?;
            Ok(())
        }
        #[cfg(windows)]
        {
            let target = if canonical.is_file() {
                canonical.parent().unwrap_or(&canonical)
            } else {
                &canonical
            };
            let _ = std::process::Command::new("explorer")
                .arg(target)
                .spawn()
                .map_err(|e| format!("Failed to open file explorer: {e}"))?;
            Ok(())
        }
        #[cfg(target_os = "macos")]
        {
            let target = if canonical.is_file() {
                canonical.parent().unwrap_or(&canonical)
            } else {
                &canonical
            };
            let _ = std::process::Command::new("open")
                .arg(target)
                .spawn()
                .map_err(|e| format!("Failed to open in Finder: {e}"))?;
            Ok(())
        }
        #[cfg(not(any(target_os = "linux", windows, target_os = "macos")))]
        Err("Unsupported OS".to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

pub async fn copy_file(src_path: String, dest_path: String) -> Result<(), String> {
    tokio::task::spawn_blocking(move || {
        let src = Path::new(&src_path);
        let dest = Path::new(&dest_path);
        if !src.exists() {
            return Err("Source file or folder does not exist".to_string());
        }
        if dest.exists() {
            return Err("Destination already exists".to_string());
        }
        if src.is_dir() {
            copy_dir_recursive(src, dest)
        } else {
            if let Some(parent) = dest.parent() {
                let _ = fs::create_dir_all(parent);
            }
            fs::copy(src, dest)
                .map(|_| ())
                .map_err(|e| format!("Failed to copy file: {e}"))
        }
    })
    .await
    .map_err(|e| e.to_string())?
}

fn copy_dir_recursive(src: &Path, dest: &Path) -> Result<(), String> {
    fs::create_dir_all(dest).map_err(|e| format!("Failed to create destination directory: {e}"))?;
    for entry in fs::read_dir(src).map_err(|e| format!("Failed to read source directory: {e}"))? {
        let entry = entry.map_err(|e| format!("Failed to read directory entry: {e}"))?;
        let entry_path = entry.path();
        let file_name = entry.file_name();
        let target_path = dest.join(file_name);
        if entry_path.is_dir() {
            copy_dir_recursive(&entry_path, &target_path)?;
        } else {
            fs::copy(&entry_path, &target_path)
                .map_err(|e| format!("Failed to copy file {:?}: {e}", entry_path))?;
        }
    }
    Ok(())
}

const IGNORED_DIRS: &[&str] = &[
    "node_modules",
    "target",
    "venv",
    ".venv",
    "env",
    ".git",
    "dist",
    "build",
    ".cache",
    "__pycache__",
];

/// Natural-sort key: split a name into alternating (is_number, token) chunks so that
/// "2" sorts before "10" (human/numeric ordering) instead of the lexicographic
/// "10" < "2" string comparison.
fn natural_key(name: &str) -> Vec<(bool, Vec<u8>)> {
    let mut out: Vec<(bool, Vec<u8>)> = Vec::new();
    let mut chars = name.chars().peekable();
    while chars.peek().is_some() {
        if chars.peek().unwrap().is_ascii_digit() {
            let mut digits = String::new();
            while let Some(&c) = chars.peek() {
                if c.is_ascii_digit() {
                    digits.push(c);
                    chars.next();
                } else {
                    break;
                }
            }
            // Strip leading zeros so 002 == 2 numerically; use raw bytes as key.
            let trimmed = digits.trim_start_matches('0');
            out.push((true, trimmed.as_bytes().to_vec()));
        } else {
            let mut token = String::new();
            token.push(chars.next().unwrap());
            while let Some(&c) = chars.peek() {
                if !c.is_ascii_digit() {
                    token.push(c);
                    chars.next();
                } else {
                    break;
                }
            }
            out.push((false, token.to_lowercase().into_bytes()));
        }
    }
    out
}

fn natural_cmp(a: &str, b: &str) -> std::cmp::Ordering {
    let ka = natural_key(a);
    let kb = natural_key(b);
    for (i, (x, y)) in ka.iter().zip(kb.iter()).enumerate() {
        match (x.0, y.0) {
            (true, true) => {
                let xa: u64 =
                    x.1.iter()
                        .map(|b| u64::from(*b))
                        .fold(0, |acc, d| acc * 10 + d);
                let ya: u64 =
                    y.1.iter()
                        .map(|b| u64::from(*b))
                        .fold(0, |acc, d| acc * 10 + d);
                if xa != ya {
                    return xa.cmp(&ya);
                }
            }
            (false, false) => {
                if x.1 != y.1 {
                    return x.1.cmp(&y.1);
                }
            }
            _ => {
                // One side has a number, the other a non-number token at this position
                return x.1.cmp(&y.1);
            }
        }
        if i + 1 == ka.len() && i + 1 == kb.len() {
            break;
        }
    }
    ka.len().cmp(&kb.len())
}

fn read_dir_recursive(path: &Path, depth: u8) -> Result<Vec<FileNode>, String> {
    if depth == 0 {
        return Ok(vec![]);
    }
    let mut entries = Vec::new();
    let read = fs::read_dir(path).map_err(|e| e.to_string())?;

    for entry in read.flatten() {
        let p = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();

        if name.starts_with('.') {
            continue;
        }

        let is_dir = p.is_dir();
        if is_dir && IGNORED_DIRS.contains(&name.as_str()) {
            continue;
        }
        let (size_bytes, modified_at) = if let Ok(meta) = p.metadata() {
            let size = if is_dir { None } else { Some(meta.len()) };
            let mod_time = meta
                .modified()
                .ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_secs());
            (size, mod_time)
        } else {
            (None, None)
        };

        if is_dir || is_supported_file(&name) {
            entries.push(FileNode {
                name,
                path: p.to_string_lossy().to_string(),
                is_dir,
                children: if is_dir {
                    read_dir_recursive(&p, depth - 1).ok()
                } else {
                    None
                },
                size_bytes,
                modified_at,
            });
        }
    }

    entries.sort_by(|a, b| {
        if a.is_dir == b.is_dir {
            natural_cmp(&a.name, &b.name)
        } else if a.is_dir {
            std::cmp::Ordering::Less
        } else {
            std::cmp::Ordering::Greater
        }
    });

    Ok(entries)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_directory_and_filter_ignored() {
        let temp = tempfile::tempdir().expect("tempdir");
        let temp_path = temp.path();

        let sub_folder = temp_path.join("docs");
        create_directory(sub_folder.to_string_lossy().to_string())
            .await
            .expect("create docs dir");

        let node_modules = temp_path.join("node_modules");
        create_directory(node_modules.to_string_lossy().to_string())
            .await
            .expect("create node_modules dir");

        let target_dir = temp_path.join("target");
        create_directory(target_dir.to_string_lossy().to_string())
            .await
            .expect("create target dir");

        let git_dir = temp_path.join(".git");
        create_directory(git_dir.to_string_lossy().to_string())
            .await
            .expect("create .git dir");

        let sample_file = sub_folder.join("test.md");
        fs::write(&sample_file, "# Hello").expect("write md");

        let entries = list_dir(temp_path.to_string_lossy().to_string())
            .await
            .expect("list dir");

        // Verify node_modules, target, .git are excluded
        assert!(entries.iter().all(|e| e.name != "node_modules"));
        assert!(entries.iter().all(|e| e.name != "target"));
        assert!(entries.iter().all(|e| e.name != ".git"));

        // Verify docs folder is included and has children
        let docs = entries
            .iter()
            .find(|e| e.name == "docs")
            .expect("find docs");
        assert!(docs.is_dir);
        let children = docs.children.as_ref().expect("docs children");
        assert_eq!(children.len(), 1);
        assert_eq!(children[0].name, "test.md");
    }

    #[tokio::test]
    async fn test_copy_file_and_directory() {
        let temp = tempfile::tempdir().expect("tempdir");
        let temp_path = temp.path();

        let src_file = temp_path.join("file1.md");
        fs::write(&src_file, "hello").expect("write file");
        let dest_file = temp_path.join("file2.md");

        copy_file(
            src_file.to_string_lossy().to_string(),
            dest_file.to_string_lossy().to_string(),
        )
        .await
        .expect("copy file");
        assert_eq!(fs::read_to_string(&dest_file).expect("read"), "hello");

        let src_dir = temp_path.join("folder1");
        fs::create_dir_all(&src_dir).expect("mkdir");
        fs::write(src_dir.join("sub.md"), "nested").expect("write sub");
        let dest_dir = temp_path.join("folder2");

        copy_file(
            src_dir.to_string_lossy().to_string(),
            dest_dir.to_string_lossy().to_string(),
        )
        .await
        .expect("copy dir");
        assert_eq!(
            fs::read_to_string(dest_dir.join("sub.md")).expect("read"),
            "nested"
        );
    }
}
