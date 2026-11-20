//! External change watcher for CAMark workspace (R-03).
//!
//! Watches the open workspace root with `notify`, debounced (~300ms),
//! and emits events with changed paths and kind (`created`, `modified`, `removed`, `renamed`).
//! Automatically ignores:
//! 1. App's own writes (tracked in the last 1.5 seconds)
//! 2. `.git/` directory
//! 3. `node_modules/` directory
//! 4. Temporary atomic save files (`.camark_tmp_*`)

use notify::Watcher;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FileChangeKind {
    Created,
    Modified,
    Removed,
    Renamed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceFileChange {
    pub path: String,
    pub kind: FileChangeKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceChangesPayload {
    pub changes: Vec<WorkspaceFileChange>,
}

pub type EventCallback = Arc<dyn Fn(WorkspaceChangesPayload) + Send + Sync + 'static>;

pub const WORKSPACE_CHANGE_EVENT: &str = "workspace://file-changed";

static RECENT_APP_WRITES: parking_lot::Mutex<Option<HashMap<PathBuf, Instant>>> =
    parking_lot::Mutex::new(None);

/// Record an app-originated write so subsequent watcher events for this path
/// are suppressed within the 1.5 second window.
pub fn record_app_write<P: AsRef<Path>>(path: P) {
    let now = Instant::now();
    let p = path.as_ref();
    let mut guard = RECENT_APP_WRITES.lock();
    let map = guard.get_or_insert_with(HashMap::new);
    map.retain(|_, time| now.duration_since(*time) < Duration::from_secs(5));
    map.insert(p.to_path_buf(), now);
    if let Ok(canonical) = p.canonicalize() {
        map.insert(canonical, now);
    }
}

/// Check if a path was modified by CAMark itself within the last 1.5 seconds.
pub fn is_recent_app_write<P: AsRef<Path>>(path: P) -> bool {
    let now = Instant::now();
    let p = path.as_ref();
    let mut guard = RECENT_APP_WRITES.lock();
    let map = match guard.as_mut() {
        Some(m) => m,
        None => return false,
    };
    map.retain(|_, time| now.duration_since(*time) < Duration::from_secs(5));

    if let Some(t) = map.get(p)
        && now.duration_since(*t) < Duration::from_millis(1500)
    {
        return true;
    }
    if let Ok(canonical) = p.canonicalize()
        && let Some(t) = map.get(&canonical)
        && now.duration_since(*t) < Duration::from_millis(1500)
    {
        return true;
    }
    false
}

/// Determine whether a path should be ignored by the watcher.
pub fn should_ignore_path<P: AsRef<Path>>(path: P) -> bool {
    let p = path.as_ref();
    for comp in p.components() {
        let s = comp.as_os_str().to_string_lossy();
        if s == ".git" || s == "node_modules" {
            return true;
        }
    }
    if let Some(file_name) = p.file_name() {
        let name_str = file_name.to_string_lossy();
        if name_str.starts_with(".camark_tmp_") {
            return true;
        }
    }
    is_recent_app_write(p)
}

struct ActiveWatcher {
    stop_tx: tokio::sync::watch::Sender<bool>,
    _watcher: notify::RecommendedWatcher,
}

static ACTIVE_WATCHER: parking_lot::Mutex<Option<ActiveWatcher>> = parking_lot::Mutex::new(None);

/// Stop the currently active workspace watcher, if any.
pub fn stop_watching() {
    let mut guard = ACTIVE_WATCHER.lock();
    if let Some(active) = guard.take() {
        let _ = active.stop_tx.send(true);
    }
}

/// Start watching a directory with a custom event callback (used by tests and Tauri bridge).
pub fn start_watching_with_callback<P: AsRef<Path>>(
    dir_path: P,
    callback: EventCallback,
) -> Result<(), String> {
    let raw_dir = dir_path.as_ref();
    if !raw_dir.exists() || !raw_dir.is_dir() {
        return Err(format!(
            "Directory does not exist or is not a directory: {:?}",
            raw_dir
        ));
    }
    let dir = match raw_dir.canonicalize() {
        Ok(c) => c,
        Err(_) => raw_dir.to_path_buf(),
    };

    // Stop any existing watcher
    stop_watching();

    let (event_tx, mut event_rx) = tokio::sync::mpsc::unbounded_channel::<notify::Event>();
    let (stop_tx, mut stop_rx) = tokio::sync::watch::channel(false);

    let event_tx_clone = event_tx.clone();
    let mut watcher =
        notify::recommended_watcher(move |res: Result<notify::Event, notify::Error>| {
            if let Ok(event) = res {
                let _ = event_tx_clone.send(event);
            }
        })
        .map_err(|e| format!("Failed to create notify watcher: {e}"))?;

    watcher
        .watch(&dir, notify::RecursiveMode::Recursive)
        .map_err(|e| format!("Failed to watch directory {:?}: {e}", dir))?;

    // Spawn debouncer task
    tokio::spawn(async move {
        let mut pending: HashMap<PathBuf, (FileChangeKind, Instant)> = HashMap::new();
        let debounce_duration = Duration::from_millis(300);

        loop {
            tokio::select! {
                _ = stop_rx.changed() => {
                    if *stop_rx.borrow() {
                        break;
                    }
                }
                maybe_event = event_rx.recv() => {
                    match maybe_event {
                        Some(event) => {
                            let kind = match event.kind {
                                notify::EventKind::Create(_) => FileChangeKind::Created,
                                notify::EventKind::Modify(notify::event::ModifyKind::Name(_)) => FileChangeKind::Renamed,
                                notify::EventKind::Modify(_) => FileChangeKind::Modified,
                                notify::EventKind::Remove(_) => FileChangeKind::Removed,
                                _ => FileChangeKind::Modified,
                            };
                            let now = Instant::now();
                            for path in event.paths {
                                if !should_ignore_path(&path) {
                                    let effective_kind = match pending.get(&path) {
                                        Some((FileChangeKind::Created, _)) if kind == FileChangeKind::Modified => FileChangeKind::Created,
                                        _ => kind,
                                    };
                                    pending.insert(path, (effective_kind, now));
                                }
                            }
                        }
                        None => break,
                    }
                }
                _ = tokio::time::sleep(Duration::from_millis(50)), if !pending.is_empty() => {
                    let now = Instant::now();
                    let mut ready = Vec::new();
                    pending.retain(|path, (kind, timestamp)| {
                        if now.duration_since(*timestamp) >= debounce_duration {
                            if !should_ignore_path(path) {
                                ready.push(WorkspaceFileChange {
                                    path: path.to_string_lossy().to_string(),
                                    kind: *kind,
                                });
                            }
                            false
                        } else {
                            true
                        }
                    });

                    if !ready.is_empty() {
                        callback(WorkspaceChangesPayload { changes: ready });
                    }
                }
            }
        }
    });

    let active = ActiveWatcher {
        stop_tx,
        _watcher: watcher,
    };
    *ACTIVE_WATCHER.lock() = Some(active);

    Ok(())
}

/// Start watching a directory and emit Tauri events to the frontend.
pub fn start_watching<P: AsRef<Path>>(app: tauri::AppHandle, dir_path: P) -> Result<(), String> {
    use tauri::Emitter;
    let app_clone = app.clone();
    let callback = Arc::new(move |payload: WorkspaceChangesPayload| {
        let _ = app_clone.emit(WORKSPACE_CHANGE_EVENT, &payload);
    });
    start_watching_with_callback(dir_path, callback)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[tokio::test]
    async fn test_external_change_watcher_debouncing_and_ignore_rules() {
        let temp = tempfile::tempdir().expect("create tempdir");
        let dir = temp.path().to_path_buf();

        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<WorkspaceChangesPayload>();
        let callback = Arc::new(move |payload: WorkspaceChangesPayload| {
            let _ = tx.send(payload);
        });

        start_watching_with_callback(&dir, callback).expect("start watching");

        // 1. External write: directly using std::fs (not via CAMark app write)
        let file1 = dir.join("external_note.md");
        fs::write(&file1, "hello from outside").expect("write external");

        // Wait for debounced event (~300ms + margin)
        let event = tokio::time::timeout(Duration::from_millis(2000), rx.recv())
            .await
            .expect("timeout waiting for external write event")
            .expect("channel closed");

        assert!(!event.changes.is_empty());
        assert!(
            event
                .changes
                .iter()
                .any(|c| c.path.contains("external_note.md"))
        );

        // 2. App-originated write: record_app_write followed by write
        let file2 = dir.join("app_note.md");
        record_app_write(&file2);
        fs::write(&file2, "written by app").expect("write internal");

        // Wait 600ms and verify NO event is emitted
        let unexpected = tokio::time::timeout(Duration::from_millis(600), rx.recv()).await;
        assert!(
            unexpected.is_err(),
            "app-originated write should not produce an event"
        );

        // 3. Ignored paths: .git/, node_modules/, .camark_tmp_*
        let git_dir = dir.join(".git");
        let _ = fs::create_dir_all(&git_dir);
        fs::write(git_dir.join("config"), "git config").expect("write git");

        let node_dir = dir.join("node_modules").join("pkg");
        let _ = fs::create_dir_all(&node_dir);
        fs::write(node_dir.join("index.js"), "module").expect("write node_modules");

        let tmp_file = dir.join(".camark_tmp_12345");
        fs::write(&tmp_file, "temp content").expect("write tempfile");

        let unexpected_ignored = tokio::time::timeout(Duration::from_millis(600), rx.recv()).await;
        assert!(
            unexpected_ignored.is_err(),
            "ignored paths should not produce an event"
        );

        // 4. Stop watching and verify no more events
        stop_watching();

        let file3 = dir.join("another_external.md");
        fs::write(&file3, "after stop").expect("write after stop");

        let unexpected_stopped = tokio::time::timeout(Duration::from_millis(600), rx.recv()).await;
        // Either timeout or channel closed (None) means no event was produced
        let received_event = match unexpected_stopped {
            Ok(Some(evt)) => Some(evt),
            _ => None,
        };
        assert!(
            received_event.is_none(),
            "stopped watcher should not produce an event, got: {:?}",
            received_event
        );
    }
}
