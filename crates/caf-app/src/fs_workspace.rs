//! Native Filesystem Workspace operations for CAMark (P1).

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::sync::Mutex;

pub const SUPPORTED_TEXT_EXTENSIONS: &[&str] = &[
    // Markdown & Documentation
    "md", "markdown", "mdown", "mkd", "txt", "text", "org", "rst", "adoc", "asciidoc",
    // Config & Data
    "json", "jsonc", "json5", "yaml", "yml", "toml", "xml", "csv", "tsv", "ini", "conf", "cfg", "env", "properties",
    // Web
    "html", "htm", "css", "scss", "sass", "less", "js", "mjs", "cjs", "jsx", "ts", "mts", "cts", "tsx", "svelte", "vue", "svg",
    // Programming Languages
    "rs", "py", "pyw", "go", "sh", "bash", "zsh", "fish", "sql", "c", "cpp", "cc", "cxx", "h", "hpp", "hxx",
    "java", "php", "rb", "lua", "r", "swift", "kt", "kts", "dart", "zig", "ex", "exs", "erl", "clj",
    // Scripts & Build
    "dockerfile", "makefile", "cmake", "diff", "patch", "log",
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
        read_dir_recursive(path, 3)
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
            a.name.to_lowercase().cmp(&b.name.to_lowercase())
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
}
