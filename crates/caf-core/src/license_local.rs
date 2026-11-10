//! Local license storage — offline-first entitlement.
//!
//! After activation the server returns an `expires_at` unix timestamp.
//! This module persists it locally so every subsequent `is_pro()` check
//! works without network.

use crate::error::{CafError, IoError};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LocalLicense {
    pub key: String,
    pub product: String,
    pub tier: String,
    pub expires_at: i64, // 0 means lifetime
    #[serde(default)]
    pub activation_token: String,
    #[serde(default)]
    pub activated_at: String,
    #[serde(default)]
    pub device_id: String,
}

fn license_path(app: &str) -> Result<PathBuf, CafError> {
    if let Ok(custom_dir) = std::env::var("CMRK_DATA_DIR") {
        return Ok(PathBuf::from(custom_dir).join(format!("{app}_license.json")));
    }
    let base = directories::BaseDirs::new()
        .ok_or_else(|| CafError::Io(IoError::Generic("no home directory".to_string())))?;
    Ok(base.data_local_dir().join(format!("{app}/license.json")))
}

pub fn save_license(app: &str, lic: &LocalLicense) -> Result<(), CafError> {
    let p = license_path(app)?;
    if let Some(parent) = p.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| CafError::Io(IoError::Generic(e.to_string())))?;
    }
    let data = serde_json::to_string_pretty(lic)
        .map_err(|e| CafError::Io(IoError::Generic(e.to_string())))?;
    std::fs::write(&p, data).map_err(|e| CafError::Io(IoError::Generic(e.to_string())))?;
    Ok(())
}

pub fn load_license(app: &str) -> Result<Option<LocalLicense>, CafError> {
    let p = license_path(app)?;
    if !p.exists() {
        return Ok(None);
    }
    let txt =
        std::fs::read_to_string(p).map_err(|e| CafError::Io(IoError::Generic(e.to_string())))?;
    Ok(Some(serde_json::from_str(&txt).map_err(|e| {
        CafError::Io(IoError::Generic(e.to_string()))
    })?))
}

pub fn remove_license(app: &str) -> Result<(), CafError> {
    let p = license_path(app)?;
    if p.exists() {
        std::fs::remove_file(p).map_err(|e| CafError::Io(IoError::Generic(e.to_string())))?;
    }
    Ok(())
}

/// Returns true when the license is valid right now.
/// Lifetime tier (tier="99" or "lifetime" or expires_at=0) never expires; others are checked against `now`.
pub fn is_license_valid(lic: &LocalLicense) -> bool {
    if lic.tier == "99" || lic.tier.eq_ignore_ascii_case("lifetime") || lic.expires_at == 0 {
        return true;
    }
    let now = chrono::Utc::now().timestamp();
    now <= lic.expires_at
}
