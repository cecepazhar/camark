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
    if let Some(ext) = lower.rsplit('.').next()
        && ext != lower
        && SUPPORTED_TEXT_EXTENSIONS.contains(&ext)
    {
        return true;
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceFileContent {
    pub content: String,
    pub version_stamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind")]
pub enum WorkspaceSaveError {
    Conflict {
        current_stamp: String,
        message: String,
    },
    NotFound {
        message: String,
    },
    Io {
        message: String,
    },
}

impl std::fmt::Display for WorkspaceSaveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Conflict {
                current_stamp,
                message,
            } => {
                write!(f, "Conflict: current stamp {current_stamp} ({message})")
            }
            Self::NotFound { message } => write!(f, "File not found: {message}"),
            Self::Io { message } => write!(f, "IO error: {message}"),
        }
    }
}

impl std::error::Error for WorkspaceSaveError {}

pub fn get_file_version_stamp<P: AsRef<Path>>(path: P) -> std::io::Result<String> {
    let p = path.as_ref();
    let meta = fs::metadata(p)?;
    let mtime = meta.modified()?;
    let duration = mtime
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    let nanos = duration.as_nanos();
    let len = meta.len();
    Ok(format!("{nanos}:{len}"))
}

pub async fn read_file(file_path: String) -> Result<WorkspaceFileContent, String> {
    tokio::task::spawn_blocking(move || {
        let path = Path::new(&file_path);
        let content = fs::read_to_string(path).map_err(|e| format!("Failed to read file: {e}"))?;
        let version_stamp = get_file_version_stamp(path)
            .map_err(|e| format!("Failed to compute version stamp: {e}"))?;
        Ok(WorkspaceFileContent {
            content,
            version_stamp,
        })
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

/// Atomically writes `content` to `target_path` (R-01).
///
/// 1. If `target_path` is a symlink, writes through to the canonical target without replacing the link.
/// 2. Creates a temporary file in the same parent directory as the target.
/// 3. Preserves existing file permissions on Unix if the target exists.
/// 4. Flushes and syncs (`fsync`) data to disk.
/// 5. Atomically replaces the target file via `tempfile::NamedTempFile::persist`.
/// 6. Cleans up temporary file automatically if any error occurs.
pub fn write_file_atomic<P: AsRef<Path>, C: AsRef<[u8]>>(
    target_path: P,
    content: C,
) -> std::io::Result<()> {
    let raw_path = target_path.as_ref();

    // 1. If target is a symlink, write through to link destination (R-01)
    let target = if fs::symlink_metadata(raw_path)
        .map(|m| m.is_symlink())
        .unwrap_or(false)
    {
        fs::canonicalize(raw_path)?
    } else {
        raw_path.to_path_buf()
    };

    let parent = target.parent().unwrap_or_else(|| Path::new("."));
    if !parent.as_os_str().is_empty() && !parent.exists() {
        fs::create_dir_all(parent)?;
    }

    // 2. Preserve existing permissions on Unix if target exists
    #[cfg(unix)]
    let existing_perms = fs::metadata(&target).ok().map(|m| m.permissions());

    // 3. Create temp file in the same directory
    let mut temp_file = tempfile::Builder::new()
        .prefix(".camark_tmp_")
        .tempfile_in(parent)?;

    #[cfg(unix)]
    if let Some(ref perms) = existing_perms {
        let _ = fs::set_permissions(temp_file.path(), perms.clone());
    }

    // 4. Write data and fsync
    use std::io::Write;
    temp_file.write_all(content.as_ref())?;
    temp_file.as_file().sync_all()?;

    // 5. Atomic persist / rename over target
    temp_file.persist(&target).map_err(|e| e.error)?;

    // Sync directory on Unix to guarantee directory entry persistence
    #[cfg(unix)]
    if let Ok(dir_file) = fs::File::open(parent) {
        let _ = dir_file.sync_all();
    }

    crate::fs_watcher::record_app_write(target);

    Ok(())
}

pub async fn write_file(
    file_path: String,
    content: String,
    expected_stamp: String,
) -> Result<String, WorkspaceSaveError> {
    tokio::task::spawn_blocking(move || {
        let path = Path::new(&file_path);

        if path.exists() {
            let current_stamp = get_file_version_stamp(path).map_err(|e| WorkspaceSaveError::Io {
                message: format!("Failed to read metadata: {e}"),
            })?;

            if expected_stamp != "*" && expected_stamp != current_stamp {
                let msg = format!(
                    "File modified on disk (current stamp: {current_stamp}, expected: {expected_stamp})"
                );
                return Err(WorkspaceSaveError::Conflict {
                    current_stamp,
                    message: msg,
                });
            }
        } else if !expected_stamp.is_empty() && expected_stamp != "*" && expected_stamp != "new" {
            return Err(WorkspaceSaveError::NotFound {
                message: "File does not exist on disk".to_string(),
            });
        }

        write_file_atomic(path, content.as_bytes()).map_err(|e| WorkspaceSaveError::Io {
            message: format!("Failed to write atomic: {e}"),
        })?;

        let new_stamp = get_file_version_stamp(path).map_err(|e| WorkspaceSaveError::Io {
            message: format!("Failed to read new metadata: {e}"),
        })?;

        Ok(new_stamp)
    })
    .await
    .map_err(|e| WorkspaceSaveError::Io {
        message: e.to_string(),
    })?
}

pub async fn create_file(file_path: String) -> Result<(), String> {
    tokio::task::spawn_blocking(move || {
        let path = Path::new(&file_path);
        if path.exists() {
            return Err("File already exists".to_string());
        }
        write_file_atomic(path, b"").map_err(|e| format!("Failed to create file: {e}"))
    })
    .await
    .map_err(|e| e.to_string())?
}

pub async fn rename_file(old_path: String, new_path: String) -> Result<(), String> {
    tokio::task::spawn_blocking(move || {
        crate::fs_watcher::record_app_write(&old_path);
        crate::fs_watcher::record_app_write(&new_path);
        fs::rename(&old_path, &new_path).map_err(|e| format!("Failed to rename file: {e}"))
    })
    .await
    .map_err(|e| e.to_string())?
}

pub async fn create_directory(dir_path: String) -> Result<(), String> {
    tokio::task::spawn_blocking(move || {
        let path = Path::new(&dir_path);
        crate::fs_watcher::record_app_write(path);
        fs::create_dir_all(path).map_err(|e| format!("Failed to create directory: {e}"))
    })
    .await
    .map_err(|e| e.to_string())?
}

pub async fn delete_file(file_path: String) -> Result<(), String> {
    tokio::task::spawn_blocking(move || {
        let path = Path::new(&file_path);
        crate::fs_watcher::record_app_write(path);
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
                let wait_result;
                loop {
                    match child.try_wait() {
                        Ok(Some(status)) => {
                            wait_result = Ok(status);
                            break;
                        }
                        Ok(None) => {
                            // Still running — check for stderr so far
                            if let Some(mut err_pipe) = child.stderr.take() {
                                let mut chunk = [0u8; 1024];
                                if let Ok(n) = err_pipe.read(&mut chunk)
                                    && n > 0
                                        && let Some(slice) = chunk.get(..n) {
                                            stderr_buf.extend_from_slice(slice);
                                        }
                            }
                            if std::time::Instant::now() >= deadline {
                                // Give up waiting — process is alive, treat as success
                                return Ok(());
                            }
                            std::thread::sleep(std::time::Duration::from_millis(200));
                        }
                        Err(e) => {
                            wait_result = Err(e);
                            break;
                        }
                    }
                }
                let status = match wait_result {
                    Ok(s) => s,
                    Err(e) => return Err(format!("try_wait failed for {cmd}: {e}")),
                };
                if status.success() {
                    return Ok(());
                }
                // Drain any remaining stderr
                if let Some(mut err_pipe) = child.stderr.take() {
                    let mut chunk = [0u8; 4096];
                    while let Ok(n) = err_pipe.read(&mut chunk) {
                        if n == 0 { break; }
                        if let Some(slice) = chunk.get(..n) {
                            stderr_buf.extend_from_slice(slice);
                        }
                    }
                }
                let stderr_str = String::from_utf8_lossy(&stderr_buf);
                Err(format!(
                    "{cmd} exited with {status}. stderr: {}",
                    stderr_str.trim()
                ))
            }

            let path_str = canonical.to_string_lossy().into_owned();
            let path_arg = std::ffi::OsStr::new(&path_str);

            // 1st: xdg-open (default handler registry)
            match try_launch("xdg-open", &[path_arg]) {
                Ok(()) => Ok(()),
                Err(primary_err) => {
                    eprintln!("[open_with_default_app] xdg-open failed: {primary_err}");
                    // 2nd: kde-open (KDE-specific, often more reliable for Office/PDF on KDE)
                    match try_launch("kde-open", &[path_arg]) {
                        Ok(()) => Ok(()),
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
                            if let Some((app, args)) = direct_app
                                && let Ok(()) = try_launch(app, args) {
                                    return Ok(());
                                }
                            Err(format!(
                                "All openers failed for {path_str}.\n  xdg-open: {primary_err}\n  kde-open: {e2}"
                            ))
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
            crate::fs_watcher::record_app_write(dest);
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
        if chars.peek().is_some_and(|c| c.is_ascii_digit()) {
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
            if let Some(c) = chars.next() {
                token.push(c);
            }
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

// ─── I-01: Image paste / drag-and-drop asset saving ──────────────────────────
/// Maximum image size accepted from the clipboard or drop target.
pub const MAX_IMAGE_BYTES: u64 = 25 * 1024 * 1024; // 25 MB

/// Allowed image MIME types and their canonical extensions.
pub const ALLOWED_IMAGE_MIMES: &[(&str, &str)] = &[
    ("image/png", "png"),
    ("image/jpeg", "jpg"),
    ("image/jpg", "jpg"),
    ("image/webp", "webp"),
    ("image/gif", "gif"),
    ("image/bmp", "bmp"),
    ("image/avif", "avif"),
    ("image/svg+xml", "svg"),
    ("image/tiff", "tif"),
];

/// Resolve a MIME type to its stored-file extension. Rejects anything not explicitly
/// whitelisted so unknown binary blobs never land in the assets folder.
fn mime_to_extension(mime: &str) -> Result<&'static str, String> {
    let normalized = mime
        .split(';')
        .next()
        .unwrap_or(mime)
        .trim()
        .to_ascii_lowercase();
    ALLOWED_IMAGE_MIMES
        .iter()
        .find(|(m, _)| *m == normalized)
        .map(|(_, ext)| *ext)
        .ok_or_else(|| format!("Unsupported image type: {mime}"))
}

/// Result of saving an image into the workspace assets folder.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WorkspaceImageResult {
    /// Markdown-relative link path to embed in the active document, e.g. `assets/foo.png`.
    pub link: String,
    /// Absolute (or cwd-relative) stored path, useful for the UI / watcher.
    pub stored_path: String,
    /// Byte size of the stored image.
    pub bytes: u64,
}

/// Error variants for image saving, surfaced to the frontend as a `String` by the
/// Tauri command wrapper.
#[derive(Debug)]
pub enum ImageSaveError {
    /// MIME type not in the allow-list.
    UnsupportedMime(String),
    /// Decoded payload exceeds [`MAX_IMAGE_BYTES`].
    TooLarge(u64),
    /// Resolved asset path would escape the workspace root (path traversal).
    PathTraversal(String),
    /// Underlying filesystem error.
    Io(std::io::Error),
}

impl std::fmt::Display for ImageSaveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnsupportedMime(m) => write!(f, "Unsupported image type: {m}"),
            Self::TooLarge(n) => write!(f, "Image too large ({n} bytes); limit {MAX_IMAGE_BYTES}"),
            Self::PathTraversal(p) => write!(f, "Refusing to write outside workspace: {p}"),
            Self::Io(e) => write!(f, "IO error saving image: {e}"),
        }
    }
}

/// Synchronous, testable core of image saving. Runs on a blocking thread via
/// [`save_image`].
///
/// Layout: for a document at `…/notes/sub/file.md` the image is written to
/// `…/notes/sub/assets/<name>` so the generated markdown link is always a short
/// relative path (`assets/<name>`) that keeps working if the whole workspace is
/// moved. The filename is collision-safe: a local-timestamp + short SHA-256 of
/// the payload + an optional `-N` suffix when an identical name already exists.
fn save_image_inner(
    doc_path: &str,
    data: &[u8],
    mime: &str,
    asset_folder: &str,
) -> Result<WorkspaceImageResult, ImageSaveError> {
    // 1. Type validation — reject before touching the filesystem.
    let ext = mime_to_extension(mime).map_err(ImageSaveError::UnsupportedMime)?;

    // 2. Size validation.
    let len = data.len() as u64;
    if len > MAX_IMAGE_BYTES {
        return Err(ImageSaveError::TooLarge(len));
    }
    if len == 0 {
        return Err(ImageSaveError::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "empty image payload",
        )));
    }

    // 3. Resolve the document to a canonical path. A relative `doc_path` is
    //    interpreted against the process CWD (the frontend always passes a path
    //    that is valid in the app's working directory).
    let doc = std::path::Path::new(doc_path);
    let doc_abs = if doc.is_absolute() {
        doc.to_path_buf()
    } else {
        std::env::current_dir()
            .map_err(ImageSaveError::Io)?
            .join(doc)
    };

    // 4. Containment guard. Reject any `..` (ParentDir) component in the
    //    caller-supplied document path first — that is the only untrusted input,
    //    and a parent-directory segment is how an attacker would walk outside
    //    the workspace root. This must run before `canonicalize` so a malicious
    //    path is refused rather than surfacing as a filesystem error.
    for component in doc.components() {
        if matches!(component, std::path::Component::ParentDir) {
            return Err(ImageSaveError::PathTraversal(doc_path.to_string()));
        }
    }
    // The document's parent is the "containing folder"; the assets dir sits next
    // to the document itself so the markdown link stays `assets/…` regardless of
    // how deep the document is nested.
    let doc_parent = doc_abs.parent().ok_or_else(|| {
        ImageSaveError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "document has no parent directory",
        ))
    })?;
    let doc_parent_canon = std::fs::canonicalize(doc_parent).map_err(|e| {
        ImageSaveError::Io(std::io::Error::new(
            e.kind(),
            format!("document folder not found: {e}"),
        ))
    })?;
    // Sanitise the configurable asset folder: a single relative path component only.
    // Reject absolute paths (leading / or \\) AND any `..` / nested-segment input,
    // falling back to `assets`. Check the *raw* input for absoluteness before
    // trimming, otherwise `/abs/` would be trimmed to the valid-looking `abs`.
    let raw_folder = asset_folder.trim();
    let is_absolute = raw_folder.starts_with('/') || raw_folder.starts_with('\\');
    let folder = raw_folder
        .trim_matches(|c| c == '/' || c == '\\')
        .to_string();
    let folder = if is_absolute
        || folder.is_empty()
        || folder.contains("..")
        || folder.contains('/')
        || folder.contains('\\')
    {
        "assets"
    } else {
        &folder
    };
    let assets_abs = doc_parent_canon.join(folder);

    // 5. Collision-safe name: `YYYYMMDD-HHMMSS-{hash8}.{ext}` with `-N` suffix
    //    when the exact name already exists.
    use std::time::{SystemTime, UNIX_EPOCH};
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    // Cheap UTC-ish local timestamp from seconds (sufficient for naming; the
    // hash below is the real uniqueness source).
    let ts_tag = ts.to_string();
    let hash: String = {
        use sha2::{Digest, Sha256};
        let mut h = Sha256::new();
        h.update(data);
        let digest = Sha256::finalize(h);
        hex::encode(digest.iter().take(4).copied().collect::<Vec<u8>>())
    };
    let stem = format!("{ts_tag}-{hash}");
    let mut file_name = format!("{stem}.{ext}");

    fs::create_dir_all(&assets_abs).map_err(ImageSaveError::Io)?;

    // Find a free filename.
    // The counter goes before the extension (`name-1.png`, not `name.png-1`), so a
    // de-duplicated file is still recognised and rendered as an image.
    let mut candidate = assets_abs.join(&file_name);
    let mut suffix = 1u32;
    while candidate.exists() {
        file_name = format!("{stem}-{suffix}.{ext}");
        candidate = assets_abs.join(&file_name);
        suffix += 1;
    }

    // 6. Atomic write (reuses R-01 helper so permissions are preserved and the
    //    file watcher learns it was an app write via `record_app_write`).
    write_file_atomic(&candidate, data).map_err(ImageSaveError::Io)?;

    // 7. Compute the relative markdown link from the document to the stored
    //    asset. Since both live under `doc_parent`, the link is simply
    //    `<asset_folder>/<candidate_file_name>`.
    let link = format!("{}/{}", folder, file_name);

    Ok(WorkspaceImageResult {
        link,
        stored_path: candidate.to_string_lossy().to_string(),
        bytes: len,
    })
}

/// Tauri-facing async wrapper: decodes the base64 payload and hands off to the
/// blocking [`save_image_inner`]. The Tauri command signature in `commands.rs`
/// keeps the base64 encoding so the JS side can `File`/`Blob` → `ArrayBuffer` →
/// base64 without serializing raw `Vec<u8>` through the FFI boundary.
pub async fn save_image(
    doc_path: String,
    data_b64: String,
    mime: String,
    asset_folder: Option<String>,
) -> Result<WorkspaceImageResult, String> {
    use base64::Engine;
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(data_b64)
        .map_err(|e| format!("Invalid base64 image payload: {e}"))?
        .to_vec();
    let folder = asset_folder.unwrap_or_else(|| "assets".to_string());
    tokio::task::spawn_blocking(move || save_image_inner(&doc_path, &bytes, &mime, &folder))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}

/// I-02: when a note file is renamed / moved to a *different* folder, carry its
/// sibling asset folder (`<old_dir>/<asset_folder>` → `<new_dir>/<asset_folder>`) so
/// the relative `assets/<name>` image links inside the note keep working.
/// No-op when the note stays in the same folder or has no asset folder.
pub fn move_note_assets(
    old_doc_path: &str,
    new_doc_path: &str,
    asset_folder: &str,
) -> Result<(), String> {
    let folder = if asset_folder.trim().is_empty() {
        "assets"
    } else {
        asset_folder
    };
    let old_dir = Path::new(old_doc_path)
        .parent()
        .map(|p| p.to_path_buf())
        .ok_or_else(|| "old doc has no parent".to_string())?;
    let new_dir = Path::new(new_doc_path)
        .parent()
        .map(|p| p.to_path_buf())
        .ok_or_else(|| "new doc has no parent".to_string())?;
    if old_dir == new_dir {
        return Ok(()); // same folder: links already relative, nothing to move
    }
    let src = old_dir.join(folder);
    if !src.exists() {
        return Ok(()); // this note has no asset folder
    }
    let dest = new_dir.join(folder);
    fs::create_dir_all(&dest).map_err(|e| format!("failed to create asset dir: {e}"))?;
    // Move each asset file; if a same-named file already exists at the
    // destination, rename the incoming one with a numeric suffix to avoid
    // clobbering (keeps both copies so no link silently breaks).
    for entry in fs::read_dir(&src).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let file_name = entry.file_name();
        let from = entry.path();
        if !from.is_file() {
            continue;
        }
        let mut to = dest.join(&file_name);
        if to.exists() {
            let mut n = 1u32;
            let stem = file_name.to_string_lossy();
            while to.exists() {
                to = dest.join(format!("{stem}-{n}"));
                n += 1;
            }
        }
        fs::rename(&from, &to).map_err(|e| format!("failed to move asset: {e}"))?;
    }
    // Remove the now-empty old asset folder.
    let _ = fs::remove_dir(&src);
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchMatch {
    pub file_path: String,
    pub relative_path: String,
    pub line_number: usize,
    pub line_content: String,
    pub match_start: usize,
    pub match_end: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchWorkspaceResult {
    pub matches: Vec<SearchMatch>,
    pub files_scanned: usize,
    pub duration_ms: u64,
}

pub async fn search_workspace(
    workspace_root: String,
    query: String,
    case_sensitive: bool,
    whole_word: bool,
    max_results: Option<usize>,
) -> Result<SearchWorkspaceResult, String> {
    tokio::task::spawn_blocking(move || {
        search_workspace_sync(
            &workspace_root,
            &query,
            case_sensitive,
            whole_word,
            max_results,
        )
    })
    .await
    .map_err(|e| e.to_string())?
}

fn search_workspace_sync(
    workspace_root: &str,
    query: &str,
    case_sensitive: bool,
    whole_word: bool,
    max_results: Option<usize>,
) -> Result<SearchWorkspaceResult, String> {
    let start_time = std::time::Instant::now();
    let root = Path::new(workspace_root);
    if !root.exists() || !root.is_dir() {
        return Err(format!("Workspace root does not exist: {workspace_root}"));
    }

    if query.trim().is_empty() {
        return Ok(SearchWorkspaceResult {
            matches: Vec::new(),
            files_scanned: 0,
            duration_ms: start_time.elapsed().as_millis() as u64,
        });
    }

    let limit = max_results.unwrap_or(500);
    let mut matches = Vec::new();
    let mut files_scanned = 0;

    let mut stack = vec![root.to_path_buf()];
    let query_lower = if case_sensitive {
        String::new()
    } else {
        query.to_lowercase()
    };

    while let Some(current_dir) = stack.pop() {
        let entries = match fs::read_dir(&current_dir) {
            Ok(e) => e,
            Err(_) => continue,
        };

        for entry in entries.flatten() {
            let path = entry.path();
            let file_name = entry.file_name();
            let name_str = file_name.to_string_lossy();

            if name_str.starts_with('.') {
                continue;
            }

            let is_dir = path.is_dir();
            if is_dir && IGNORED_DIRS.contains(&name_str.as_ref()) {
                continue;
            }

            if is_dir {
                stack.push(path);
            } else if path.is_file() && is_supported_file(&name_str) {
                files_scanned += 1;
                let rel_path = path
                    .strip_prefix(root)
                    .unwrap_or(&path)
                    .to_string_lossy()
                    .to_string();

                if let Ok(content) = fs::read_to_string(&path) {
                    for (line_idx, line) in content.lines().enumerate() {
                        let line_to_search = if case_sensitive {
                            line.to_string()
                        } else {
                            line.to_lowercase()
                        };

                        let search_term = if case_sensitive { query } else { &query_lower };

                        let mut search_from = 0;
                        while let Some(pos) = line_to_search[search_from..].find(search_term) {
                            let match_start = search_from + pos;
                            let match_end = match_start + search_term.len();

                            let is_match = if whole_word {
                                let before_ok = match_start == 0
                                    || !line_to_search
                                        .as_bytes()
                                        .get(match_start - 1)
                                        .is_some_and(|b| b.is_ascii_alphanumeric() || *b == b'_');
                                let after_ok = match_end >= line_to_search.len()
                                    || !line_to_search
                                        .as_bytes()
                                        .get(match_end)
                                        .is_some_and(|b| b.is_ascii_alphanumeric() || *b == b'_');
                                before_ok && after_ok
                            } else {
                                true
                            };

                            if is_match {
                                matches.push(SearchMatch {
                                    file_path: path.to_string_lossy().to_string(),
                                    relative_path: rel_path.clone(),
                                    line_number: line_idx + 1,
                                    line_content: line.to_string(),
                                    match_start,
                                    match_end,
                                });

                                if matches.len() >= limit {
                                    return Ok(SearchWorkspaceResult {
                                        matches,
                                        files_scanned,
                                        duration_ms: start_time.elapsed().as_millis() as u64,
                                    });
                                }
                            }

                            search_from = match_end;
                            if search_from >= line_to_search.len() {
                                break;
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(SearchWorkspaceResult {
        matches,
        files_scanned,
        duration_ms: start_time.elapsed().as_millis() as u64,
    })
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_search_workspace_finds_matches_and_options() {
        let temp = tempfile::tempdir().expect("tempdir");
        let root = temp.path();

        let doc1 = root.join("note1.md");
        fs::write(&doc1, "# Arsitektur CAMark\nRust dan Tauri v2.\n").expect("write doc1");

        let sub = root.join("sub");
        fs::create_dir(&sub).expect("create sub");
        let doc2 = sub.join("note2.md");
        fs::write(&doc2, "camark adalah editor cepat.\nBukan camark_app.\n").expect("write doc2");

        // Case insensitive match
        let res = search_workspace_sync(&root.to_string_lossy(), "camark", false, false, None)
            .expect("search");
        assert_eq!(res.matches.len(), 3);
        assert_eq!(res.files_scanned, 2);

        // Case sensitive match
        let res_case = search_workspace_sync(&root.to_string_lossy(), "CAMark", true, false, None)
            .expect("search case");
        assert_eq!(res_case.matches.len(), 1);

        // Whole word match
        let res_word = search_workspace_sync(&root.to_string_lossy(), "camark", false, true, None)
            .expect("search word");
        assert_eq!(res_word.matches.len(), 2);
    }

    #[test]
    fn test_search_workspace_5000_notes_performance() {
        let temp = tempfile::tempdir().expect("tempdir");
        let root = temp.path();

        for i in 0..5000 {
            let p = root.join(format!("note_{i}.md"));
            let text = if i == 2500 {
                "Target kata kunci unik F3-06 di catatan tengah."
            } else {
                "Catatan biasa untuk pengujian performa pencarian workspace."
            };
            fs::write(&p, text).expect("write batch");
        }

        let res = search_workspace_sync(&root.to_string_lossy(), "F3-06", false, false, None)
            .expect("search 5k");
        assert_eq!(res.matches.len(), 1);
        assert_eq!(res.files_scanned, 5000);
        assert!(
            res.duration_ms < 5000,
            "5,000 files scan took {} ms (expected < 5000ms)",
            res.duration_ms
        );
    }

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

    #[tokio::test]
    async fn test_atomic_save_normal_and_keeps_permissions() {
        let temp = tempfile::tempdir().expect("tempdir");
        let target = temp.path().join("atomic_doc.md");

        write_file(
            target.to_string_lossy().to_string(),
            "v1".to_string(),
            "*".to_string(),
        )
        .await
        .expect("initial write");
        assert_eq!(fs::read_to_string(&target).expect("read"), "v1");

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let custom_perms = std::fs::Permissions::from_mode(0o640);
            fs::set_permissions(&target, custom_perms).expect("set perms");
        }

        write_file(
            target.to_string_lossy().to_string(),
            "v2 updated".to_string(),
            "*".to_string(),
        )
        .await
        .expect("overwrite");
        assert_eq!(fs::read_to_string(&target).expect("read"), "v2 updated");

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let final_perms = fs::metadata(&target)
                .expect("metadata")
                .permissions()
                .mode()
                & 0o777;
            assert_eq!(
                final_perms, 0o640,
                "permissions must be preserved across atomic saves"
            );
        }
    }

    #[tokio::test]
    async fn test_atomic_save_symlink_writes_through() {
        let temp = tempfile::tempdir().expect("tempdir");
        let real_file = temp.path().join("real_target.md");
        fs::write(&real_file, "original target content").expect("write real");

        let link_file = temp.path().join("symlink_to_real.md");
        #[cfg(unix)]
        std::os::unix::fs::symlink(&real_file, &link_file).expect("create symlink");

        #[cfg(unix)]
        {
            assert!(
                fs::symlink_metadata(&link_file)
                    .expect("symlink meta")
                    .is_symlink()
            );

            write_file(
                link_file.to_string_lossy().to_string(),
                "written through link".to_string(),
                "*".to_string(),
            )
            .await
            .expect("write through symlink");

            // Symlink must remain a symlink, NOT replaced by regular file
            assert!(
                fs::symlink_metadata(&link_file)
                    .expect("symlink meta")
                    .is_symlink()
            );
            assert_eq!(
                fs::read_to_string(&real_file).expect("read real"),
                "written through link"
            );
            assert_eq!(
                fs::read_to_string(&link_file).expect("read link"),
                "written through link"
            );
        }
    }

    #[tokio::test]
    async fn test_atomic_save_failure_keeps_original_and_cleans_temp() {
        let temp = tempfile::tempdir().expect("tempdir");
        let target = temp.path().join("important.md");
        fs::write(&target, "pristine content").expect("write original");

        // Attempting to write atomic to a directory path instead of file path will fail
        let invalid_target = temp.path().join("important.md").join("sub_invalid.md");
        let res = write_file_atomic(&invalid_target, b"fail content");
        assert!(res.is_err());

        // Assert original file content is pristine
        assert_eq!(
            fs::read_to_string(&target).expect("read"),
            "pristine content"
        );

        // Assert no temp files left behind
        let temp_files: Vec<_> = fs::read_dir(temp.path())
            .expect("read dir")
            .filter_map(|e| e.ok())
            .filter(|e| e.file_name().to_string_lossy().starts_with(".camark_tmp_"))
            .collect();
        assert!(
            temp_files.is_empty(),
            "no temp files should remain after failure: {temp_files:?}"
        );
    }
    #[tokio::test]
    async fn test_conflict_detection_stale_stamp_rejects_and_fresh_stamp_succeeds() {
        let temp = tempfile::tempdir().expect("tempdir");
        let target = temp.path().join("conflict_test.md");
        let target_str = target.to_string_lossy().to_string();

        // 1. Initial write
        write_file(
            target_str.clone(),
            "initial content".to_string(),
            "*".to_string(),
        )
        .await
        .expect("initial write");

        // 2. Read file to get initial content + stamp
        let read_res = read_file(target_str.clone()).await.expect("read initial");
        assert_eq!(read_res.content, "initial content");
        let initial_stamp = read_res.version_stamp;

        // 3. Simulate external modification on disk
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
        fs::write(&target, "externally modified content").expect("external write");

        // 4. Attempt to write with stale stamp -> must return Conflict error and leave disk untouched
        let save_res = write_file(
            target_str.clone(),
            "my stale editor content".to_string(),
            initial_stamp,
        )
        .await;

        match save_res {
            Err(WorkspaceSaveError::Conflict { current_stamp, .. }) => {
                let actual_disk_stamp = get_file_version_stamp(&target).expect("get stamp");
                assert_eq!(
                    current_stamp, actual_disk_stamp,
                    "returned current stamp must match disk"
                );
            }
            other => panic!("expected Conflict error, got {other:?}"),
        }

        // Verify disk content was untouched!
        assert_eq!(
            fs::read_to_string(&target).expect("read"),
            "externally modified content"
        );

        // 5. Write with fresh current stamp -> succeeds!
        let fresh_stamp = get_file_version_stamp(&target).expect("fresh stamp");
        let new_stamp = write_file(
            target_str.clone(),
            "my fresh editor content".to_string(),
            fresh_stamp,
        )
        .await
        .expect("write with fresh stamp must succeed");

        assert_eq!(
            fs::read_to_string(&target).expect("read"),
            "my fresh editor content"
        );
        let final_disk_stamp = get_file_version_stamp(&target).expect("final stamp");
        assert_eq!(new_stamp, final_disk_stamp);
    }

    #[tokio::test]
    async fn test_conflict_detection_force_overwrite_with_wildcard() {
        let temp = tempfile::tempdir().expect("tempdir");
        let target = temp.path().join("wildcard_test.md");
        let target_str = target.to_string_lossy().to_string();

        fs::write(&target, "external disk content").expect("external write");

        // Wildcard "*" bypasses conflict check for explicit overwrite
        let new_stamp = write_file(
            target_str.clone(),
            "forced content".to_string(),
            "*".to_string(),
        )
        .await
        .expect("wildcard write succeeds");

        assert_eq!(fs::read_to_string(&target).expect("read"), "forced content");
        let final_disk_stamp = get_file_version_stamp(&target).expect("final stamp");
        assert_eq!(new_stamp, final_disk_stamp);
    }

    // ─── I-01 image-save tests ──────────────────────────────────────
    use base64::Engine as _;

    fn png_bytes(seed: u8) -> Vec<u8> {
        // Minimal valid-looking payload; we only exercise naming / guards, not
        // real PNG decoding. Content uniqueness comes from `seed`.
        let mut v = vec![0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a];
        v.extend_from_slice(&[seed, seed.wrapping_add(1), 0, 1, 2, 3, 4, 5]);
        v
    }

    #[tokio::test]
    async fn image_save_writes_asset_and_returns_relative_link() {
        let temp = tempfile::tempdir().expect("tempdir");
        let doc_dir = temp.path().join("notes");
        fs::create_dir_all(&doc_dir).unwrap();
        let doc = doc_dir.join("file.md");
        fs::write(&doc, "# hi").unwrap();

        let res = save_image(
            doc.to_string_lossy().to_string(),
            base64::engine::general_purpose::STANDARD.encode(png_bytes(1)),
            "image/png".into(),
            None,
        )
        .await
        .expect("image save");

        // Link is a short relative path rooted at the document's folder.
        assert!(res.link.starts_with("assets/"), "link={}", res.link);
        assert!(res.link.ends_with(".png"), "link={}", res.link);
        assert_eq!(res.bytes, png_bytes(1).len() as u64);

        // The stored file actually exists next to the document under assets/.
        let expected = doc_dir
            .join("assets")
            .join(res.link.strip_prefix("assets/").unwrap());
        assert!(expected.exists(), "missing {expected:?}");
        assert_eq!(fs::read(&expected).unwrap(), png_bytes(1));
    }

    #[tokio::test]
    async fn image_save_collision_safety_appends_suffix() {
        let temp = tempfile::tempdir().expect("tempdir");
        let doc_dir = temp.path().join("d");
        fs::create_dir_all(&doc_dir).unwrap();
        let doc = doc_dir.join("file.md");
        fs::write(&doc, "").unwrap();

        let same = png_bytes(7);
        let b64 = base64::engine::general_purpose::STANDARD.encode(&same);
        // Two saves of identical bytes must produce two distinct files.
        let r1 = save_image(
            doc.to_string_lossy().into(),
            b64.clone(),
            "image/png".into(),
            None,
        )
        .await
        .expect("first");
        let r2 = save_image(doc.to_string_lossy().into(), b64, "image/png".into(), None)
            .await
            .expect("second");

        assert_ne!(r1.link, r2.link, "collision-safe names must differ");
        // The de-duplicated name keeps its extension (`x-1.png`, never `x.png-1`), or the
        // preview would not render it as an image.
        assert!(r1.link.ends_with(".png"), "{}", r1.link);
        assert!(r2.link.ends_with(".png"), "{}", r2.link);
        assert!(
            doc_dir
                .join("assets")
                .join(r1.link.strip_prefix("assets/").unwrap())
                .exists()
        );
        assert!(
            doc_dir
                .join("assets")
                .join(r2.link.strip_prefix("assets/").unwrap())
                .exists()
        );
    }

    #[tokio::test]
    async fn image_save_rejects_unsupported_mime() {
        let temp = tempfile::tempdir().expect("tempdir");
        let doc = temp.path().join("f.md");
        fs::write(&doc, "").unwrap();
        let res = save_image(
            doc.to_string_lossy().into(),
            base64::engine::general_purpose::STANDARD.encode(b"exec"),
            "application/octet-stream".into(),
            None,
        )
        .await;
        assert!(res.is_err(), "octet-stream must be rejected");
        let msg = res.unwrap_err();
        assert!(msg.contains("Unsupported image type"), "got: {msg}");
    }

    #[tokio::test]
    async fn image_save_rejects_oversized_payload() {
        let temp = tempfile::tempdir().expect("tempdir");
        let doc = temp.path().join("f.md");
        fs::write(&doc, "").unwrap();

        let big: Vec<u8> = vec![0u8; (MAX_IMAGE_BYTES as usize) + 1];
        let res = save_image(
            doc.to_string_lossy().into(),
            base64::engine::general_purpose::STANDARD.encode(&big),
            "image/png".into(),
            None,
        )
        .await;
        assert!(res.is_err(), "oversized payload must be rejected");
        let msg = res.unwrap_err();
        assert!(msg.contains("too large"), "got: {msg}");
    }

    #[tokio::test]
    async fn image_save_rejects_path_traversal() {
        let temp = tempfile::tempdir().expect("tempdir");
        let doc = temp.path().join("f.md");
        fs::write(&doc, "").unwrap();

        // `..` in the document path is refused outright.
        let malicious = format!("{}///../escape.png", doc.to_string_lossy());
        let res = save_image(
            malicious,
            base64::engine::general_purpose::STANDARD.encode(png_bytes(9)),
            "image/png".into(),
            None,
        )
        .await;
        assert!(res.is_err(), "path traversal must be rejected");
        let msg = res.unwrap_err();
        assert!(msg.contains("outside workspace"), "got: {msg}");
    }

    #[tokio::test]
    async fn image_save_custom_asset_folder() {
        let temp = tempfile::tempdir().expect("tempdir");
        let doc_dir = temp.path().join("d");
        fs::create_dir_all(&doc_dir).unwrap();
        let doc = doc_dir.join("file.md");
        fs::write(&doc, "").unwrap();

        let b64 = base64::engine::general_purpose::STANDARD.encode(png_bytes(3));
        let res = save_image(
            doc.to_string_lossy().into(),
            b64,
            "image/png".into(),
            Some("media".into()),
        )
        .await
        .expect("custom folder save");

        assert!(res.link.starts_with("media/"), "link={}", res.link);
        assert!(
            doc_dir
                .join("media")
                .join(res.link.strip_prefix("media/").unwrap())
                .exists()
        );
    }

    #[tokio::test]
    async fn image_save_rejects_asset_folder_traversal() {
        let temp = tempfile::tempdir().expect("tempdir");
        let doc_dir = temp.path().join("d");
        fs::create_dir_all(&doc_dir).unwrap();
        let doc = doc_dir.join("file.md");
        fs::write(&doc, "").unwrap();

        // `..` / absolute / nested folders are coerced back to `assets`.
        for bad in ["../evil", "/abs/", "a/b", ""] {
            let b64 = base64::engine::general_purpose::STANDARD.encode(png_bytes(5));
            let res = save_image(
                doc.to_string_lossy().into(),
                b64,
                "image/png".into(),
                Some(bad.to_string()),
            )
            .await
            .expect("fallback to assets");
            assert!(res.link.starts_with("assets/"), "link={}", res.link);
        }
    }

    #[test]
    fn move_note_assets_carries_folder_across_move() {
        let temp = tempfile::tempdir().expect("tempdir");
        let old_dir = temp.path().join("old");
        let new_dir = temp.path().join("new");
        fs::create_dir_all(old_dir.join("assets")).unwrap();
        fs::create_dir_all(&new_dir).unwrap();
        fs::write(old_dir.join("assets".to_string() + "/a.png"), b"img").unwrap();

        let old_doc = old_dir.join("note.md");
        let new_doc = new_dir.join("note.md");
        move_note_assets(
            old_doc.to_string_lossy().as_ref(),
            new_doc.to_string_lossy().as_ref(),
            "assets",
        )
        .expect("move assets");

        assert!(
            new_dir.join("assets/a.png").exists(),
            "asset must be carried to the new folder"
        );
        assert!(!old_dir.join("assets/a.png").exists());
    }

    #[test]
    fn move_note_assets_same_folder_is_noop() {
        let temp = tempfile::tempdir().expect("tempdir");
        let dir = temp.path().join("d");
        fs::create_dir_all(dir.join("assets")).unwrap();
        fs::write(dir.join("assets/a.png"), b"img").unwrap();

        move_note_assets(
            dir.join("a.md").to_string_lossy().as_ref(),
            dir.join("b.md").to_string_lossy().as_ref(),
            "assets",
        )
        .expect("same-folder move");

        // Same folder: the asset dir is untouched.
        assert!(dir.join("assets/a.png").exists());
    }
}
