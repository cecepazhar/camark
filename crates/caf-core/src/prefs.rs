//! Device-level preferences that must be readable *before* the vault is unlocked, so they
//! cannot live in the encrypted database. Today that is only how the WebView is created
//! (GPU on/off), which the shell has to know before the first window exists.
//!
//! Plain JSON next to the database in the data directory. Nothing sensitive may go here.

use crate::error::{CafError, IoError};
use crate::paths::resolve_data_dir;
use serde::{Deserialize, Serialize};
use std::path::Path;

const FILE_NAME: &str = "performance.json";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct PerformancePrefs {
    /// GPU-accelerated rendering for the WebView. On by default: hardware compositing
    /// keeps long preview scrolls at 60fps. The low-memory background trims still apply.
    pub gpu_acceleration: bool,
    /// Automatic trimming of memory working set when app is idle or minimized.
    pub background_memory_saving: bool,
    /// Inherited from CATerm; CAMark has no terminal sessions. Kept so existing
    /// `performance.json` files still load and save unchanged. Not shown in Settings.
    pub scrollback_lines: u32,
    /// Inherited from CATerm; unused by CAMark. Kept for file compatibility.
    pub inactive_session_sleep: bool,
    /// Low power mode: no UI animations or ambient glow, and a slower preview refresh.
    pub low_power_mode: bool,
    /// Documents with more words than this re-render the preview only after a typing
    /// pause instead of while typing. `0` turns large-document mode off.
    pub large_doc_words: u32,
}

/// Default for [`PerformancePrefs::large_doc_words`].
pub const DEFAULT_LARGE_DOC_WORDS: u32 = 20_000;

impl Default for PerformancePrefs {
    fn default() -> Self {
        Self {
            gpu_acceleration: true,
            background_memory_saving: true,
            scrollback_lines: 5000,
            inactive_session_sleep: true,
            low_power_mode: false,
            large_doc_words: DEFAULT_LARGE_DOC_WORDS,
        }
    }
}

/// If anything fails: missing, unreadable or corrupt file falls back to defaults, because a bad
/// preferences file must not be able to stop the app from opening its window.
pub fn load_performance_prefs() -> Result<PerformancePrefs, CafError> {
    let info = resolve_data_dir()?;
    Ok(load_from(&info.path))
}

pub fn save_performance_prefs(prefs: &PerformancePrefs) -> Result<(), CafError> {
    save_to(&resolve_data_dir()?.path, prefs)
}

fn load_from(dir: &Path) -> PerformancePrefs {
    std::fs::read(dir.join(FILE_NAME))
        .ok()
        .and_then(|bytes| serde_json::from_slice(&bytes).ok())
        .unwrap_or_default()
}

fn save_to(dir: &Path, prefs: &PerformancePrefs) -> Result<(), CafError> {
    std::fs::create_dir_all(dir)
        .map_err(|e| CafError::Io(IoError::Generic(format!("create data dir: {e}"))))?;
    let json = serde_json::to_vec_pretty(prefs)
        .map_err(|e| CafError::Io(IoError::Generic(format!("serialize prefs: {e}"))))?;
    std::fs::write(dir.join(FILE_NAME), json)
        .map_err(|e| CafError::Io(IoError::Generic(format!("write {FILE_NAME}: {e}"))))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch_dir(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("camark-prefs-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        dir
    }

    #[test]
    fn missing_file_yields_defaults() {
        let dir = scratch_dir("missing");
        assert_eq!(load_from(&dir), PerformancePrefs::default());
    }

    #[test]
    fn corrupt_file_yields_defaults() {
        let dir = scratch_dir("corrupt");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join(FILE_NAME), b"{not json").unwrap();
        assert_eq!(load_from(&dir), PerformancePrefs::default());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn round_trips() {
        let dir = scratch_dir("roundtrip");
        let prefs = PerformancePrefs {
            gpu_acceleration: true,
            background_memory_saving: false,
            scrollback_lines: 10000,
            inactive_session_sleep: false,
            low_power_mode: true,
            large_doc_words: 5_000,
        };
        save_to(&dir, &prefs).unwrap();
        assert_eq!(load_from(&dir), prefs);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn file_written_before_large_doc_words_still_loads() {
        let dir = scratch_dir("legacy");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join(FILE_NAME),
            br#"{"gpuAcceleration":false,"backgroundMemorySaving":true,"scrollbackLines":5000,"inactiveSessionSleep":true,"lowPowerMode":true}"#,
        )
        .unwrap();
        let loaded = load_from(&dir);
        assert!(!loaded.gpu_acceleration);
        assert!(loaded.low_power_mode);
        assert_eq!(loaded.large_doc_words, DEFAULT_LARGE_DOC_WORDS);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn unknown_or_missing_fields_use_defaults() {
        let dir = scratch_dir("partial");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join(FILE_NAME), br#"{"somethingElse": 1}"#).unwrap();
        assert_eq!(load_from(&dir), PerformancePrefs::default());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
