//! Official GCC Backend License Client for CAMark (GCC /api/v1/licenses/*).

use crate::error::{CafError, IoError, ProError};
use crate::license_local::{self, LocalLicense};
use serde::{Deserialize, Serialize};

pub const DEFAULT_GCC_BASE_URL: &str = "https://gcc.fathforce.com";
pub const PRODUCT_CODE: &str = "CAMARK";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivationResult {
    pub success: bool,
    pub license_key: String,
    pub product_sku: String,
    pub product_name: String,
    pub plan_tier: String,
    pub device_id: String,
    pub max_devices: u32,
    pub expires_at: Option<String>,
    pub activated_at: Option<String>,
    pub activation_token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseStatusInfo {
    pub active: bool,
    pub valid: bool,
    pub license_key: Option<String>,
    pub product_sku: Option<String>,
    pub plan_tier: Option<String>,
    pub device_id: Option<String>,
    pub expires_at: Option<String>,
    pub offline_capable: bool,
}

#[derive(Deserialize)]
struct GccEnvelope<T> {
    success: bool,
    data: Option<T>,
    error: Option<String>,
}

#[derive(Deserialize)]
struct ValidateData {
    valid: bool,
}

#[derive(Deserialize)]
struct ActivateData {
    #[serde(default)]
    license_key: String,
    #[serde(default)]
    product_sku: String,
    #[serde(default)]
    product_name: String,
    #[serde(default)]
    plan_tier: String,
    #[serde(default)]
    device_id: String,
    #[serde(default)]
    max_devices: u32,
    expires_at: Option<String>,
    activated_at: Option<String>,
    #[serde(default)]
    activation_token: String,
}

#[derive(Deserialize)]
struct VerifyData {
    valid: bool,
}

pub fn resolve_gcc_url(custom: Option<&str>) -> String {
    if let Some(c) = custom {
        let trimmed = c.trim();
        if !trimmed.is_empty() {
            return trimmed.trim_end_matches('/').to_string();
        }
    }
    if let Ok(env_url) = std::env::var("GCC_BASE_URL") {
        let trimmed = env_url.trim();
        if !trimmed.is_empty() {
            return trimmed.trim_end_matches('/').to_string();
        }
    }
    DEFAULT_GCC_BASE_URL.to_string()
}

/// GET /api/v1/licenses/validate/:key
pub fn validate_license_key(gcc_url: Option<&str>, key: &str) -> Result<bool, CafError> {
    let clean_key = key.trim();
    if clean_key.is_empty() {
        return Ok(false);
    }
    let base = resolve_gcc_url(gcc_url);
    let url = format!("{base}/api/v1/licenses/validate/{clean_key}");

    let resp = ureq::get(&url)
        .call()
        .map_err(|e| CafError::Io(IoError::Generic(format!("GCC validate network error: {e}"))))?;

    let envelope: GccEnvelope<ValidateData> = resp.into_body().read_json().map_err(|e| {
        CafError::Io(IoError::Generic(format!(
            "Failed to parse GCC response: {e}"
        )))
    })?;

    if let Some(data) = envelope.data {
        Ok(data.valid)
    } else {
        Ok(false)
    }
}

/// POST /api/v1/licenses/activate
pub fn activate_license(
    gcc_url: Option<&str>,
    key: &str,
    device_name: Option<&str>,
) -> Result<ActivationResult, CafError> {
    let clean_key = key.trim();
    if clean_key.len() < 8 {
        return Err(CafError::Pro(ProError::Generic(
            "INVALID_LICENSE_KEY: Minimal 8 karakter".into(),
        )));
    }

    let base = resolve_gcc_url(gcc_url);
    let url = format!("{base}/api/v1/licenses/activate");
    let device_id = crate::paths::device_id()?;
    let dev_name = device_name
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| {
            std::env::var("HOSTNAME")
                .or_else(|_| std::env::var("COMPUTERNAME"))
                .unwrap_or_else(|_| "Desktop".into())
        });

    let payload = serde_json::json!({
        "license_key": clean_key,
        "device_id": device_id,
        "device_name": dev_name,
        "platform": std::env::consts::OS
    });

    let resp = ureq::post(&url).send_json(&payload).map_err(|e| {
        CafError::Io(IoError::Generic(format!(
            "GCC activation network error: {e}"
        )))
    })?;

    let envelope: GccEnvelope<ActivateData> = resp.into_body().read_json().map_err(|e| {
        CafError::Io(IoError::Generic(format!(
            "Failed to parse activation JSON: {e}"
        )))
    })?;

    if !envelope.success {
        let err_msg = envelope
            .error
            .unwrap_or_else(|| "Activation rejected by GCC".into());
        return Err(CafError::Pro(ProError::Generic(err_msg)));
    }

    let data = envelope.data.ok_or_else(|| {
        CafError::Pro(ProError::Generic(
            "Empty activation payload from GCC".into(),
        ))
    })?;

    // Parse expires_at unix timestamp if present
    let expires_ts = if let Some(ref exp_str) = data.expires_at {
        chrono::DateTime::parse_from_rfc3339(exp_str)
            .map(|dt| dt.timestamp())
            .unwrap_or(0)
    } else {
        0
    };

    // Save to local license file
    let local = LocalLicense {
        key: clean_key.to_string(),
        product: if data.product_sku.is_empty() {
            PRODUCT_CODE.to_string()
        } else {
            data.product_sku.clone()
        },
        tier: if data.plan_tier.is_empty() {
            "pro".to_string()
        } else {
            data.plan_tier.clone()
        },
        expires_at: expires_ts,
        activation_token: data.activation_token.clone(),
        activated_at: data.activated_at.clone().unwrap_or_default(),
        device_id: data.device_id.clone(),
    };
    let _ = license_local::save_license("camark", &local);

    // Sync state into app_kv if vault database is reachable
    if let Ok(conn) = crate::db::open() {
        let _ = conn.execute(
            "INSERT OR REPLACE INTO app_kv (key, value) VALUES ('billing.license_key', ?1), ('billing.plan', 'pro')",
            [clean_key],
        );
    }

    Ok(ActivationResult {
        success: true,
        license_key: clean_key.to_string(),
        product_sku: data.product_sku,
        product_name: data.product_name,
        plan_tier: data.plan_tier,
        device_id: data.device_id,
        max_devices: data.max_devices,
        expires_at: data.expires_at,
        activated_at: data.activated_at,
        activation_token: data.activation_token,
    })
}

/// POST /api/v1/licenses/verify
pub fn verify_license(gcc_url: Option<&str>, key: Option<&str>) -> Result<bool, CafError> {
    let resolved_key = match key {
        Some(k) if !k.trim().is_empty() => k.trim().to_string(),
        _ => {
            if let Ok(Some(lic)) = license_local::load_license("camark") {
                lic.key
            } else {
                return Ok(false);
            }
        }
    };

    let base = resolve_gcc_url(gcc_url);
    let url = format!("{base}/api/v1/licenses/verify");
    let device_id = crate::paths::device_id()?;

    let payload = serde_json::json!({
        "license_key": resolved_key,
        "device_id": device_id
    });

    let resp = ureq::post(&url)
        .send_json(&payload)
        .map_err(|e| CafError::Io(IoError::Generic(format!("GCC verify network error: {e}"))))?;

    let envelope: GccEnvelope<VerifyData> = resp.into_body().read_json().map_err(|e| {
        CafError::Io(IoError::Generic(format!(
            "Failed to parse verify JSON: {e}"
        )))
    })?;

    if let Some(data) = envelope.data {
        Ok(data.valid)
    } else {
        Ok(false)
    }
}

/// POST /api/v1/licenses/deactivate
pub fn deactivate_license(gcc_url: Option<&str>, key: Option<&str>) -> Result<bool, CafError> {
    let resolved_key = match key {
        Some(k) if !k.trim().is_empty() => k.trim().to_string(),
        _ => {
            if let Ok(Some(lic)) = license_local::load_license("camark") {
                lic.key
            } else {
                return Ok(true);
            }
        }
    };

    let base = resolve_gcc_url(gcc_url);
    let url = format!("{base}/api/v1/licenses/deactivate");
    let device_id = crate::paths::device_id()?;

    let payload = serde_json::json!({
        "license_key": resolved_key,
        "device_id": device_id
    });

    let _ = ureq::post(&url).send_json(&payload);

    // Clear local storage regardless of network result
    let _ = license_local::remove_license("camark");
    if let Ok(conn) = crate::db::open() {
        let _ = conn.execute(
            "DELETE FROM app_kv WHERE key IN ('billing.license_key', 'billing.plan')",
            [],
        );
    }

    Ok(true)
}

/// Retrieves the current license status, first checking local offline cache.
pub fn get_license_status() -> Result<LicenseStatusInfo, CafError> {
    if let Ok(Some(lic)) = license_local::load_license("camark") {
        let valid = license_local::is_license_valid(&lic);
        let expires_str = if lic.expires_at == 0 {
            Some("Lifetime".to_string())
        } else {
            chrono::DateTime::from_timestamp(lic.expires_at, 0).map(|dt| dt.to_rfc3339())
        };

        return Ok(LicenseStatusInfo {
            active: valid,
            valid,
            license_key: Some(lic.key),
            product_sku: Some(lic.product),
            plan_tier: Some(lic.tier),
            device_id: if lic.device_id.is_empty() {
                crate::paths::device_id().ok()
            } else {
                Some(lic.device_id)
            },
            expires_at: expires_str,
            offline_capable: !lic.activation_token.is_empty(),
        });
    }

    // Check DB fallback
    if let Ok(conn) = crate::db::open() {
        let row: Result<(String, String), _> = conn.query_row(
            "SELECT
                (SELECT value FROM app_kv WHERE key = 'billing.license_key'),
                (SELECT value FROM app_kv WHERE key = 'billing.plan')",
            [],
            |r| Ok((r.get(0).unwrap_or_default(), r.get(1).unwrap_or_default())),
        );

        if let Ok((key, plan)) = row {
            if !key.is_empty() && plan == "pro" {
                return Ok(LicenseStatusInfo {
                    active: true,
                    valid: true,
                    license_key: Some(key),
                    product_sku: Some(PRODUCT_CODE.to_string()),
                    plan_tier: Some("pro".to_string()),
                    device_id: crate::paths::device_id().ok(),
                    expires_at: Some("Lifetime".to_string()),
                    offline_capable: false,
                });
            }
        }
    }

    Ok(LicenseStatusInfo {
        active: false,
        valid: false,
        license_key: None,
        product_sku: None,
        plan_tier: None,
        device_id: crate::paths::device_id().ok(),
        expires_at: None,
        offline_capable: false,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_gcc_url() {
        assert_eq!(
            resolve_gcc_url(Some("https://custom.gcc.com/")),
            "https://custom.gcc.com"
        );
        assert_eq!(resolve_gcc_url(None), DEFAULT_GCC_BASE_URL);
    }

    #[test]
    fn test_offline_license_status_roundtrip() {
        let _guard = crate::test_support::isolated_data_dir("lic_status_test");
        crate::vault::validate_password("test-password-p0-3").expect("unlock vault");

        // Initial: no license
        let initial = get_license_status().expect("get status");
        assert!(!initial.active);
        assert!(!initial.valid);

        // Save a mock local license
        let lic = LocalLicense {
            key: "GCC-CAMARK-TEST-KEY-1234".to_string(),
            product: "CAMARK".to_string(),
            tier: "pro".to_string(),
            expires_at: 0, // Lifetime
            activation_token: "mock.token.signature".to_string(),
            activated_at: "2026-11-10T00:00:00Z".to_string(),
            device_id: "test-device-uuid".to_string(),
        };
        license_local::save_license("camark", &lic).expect("save license");

        // Verify status reflects active offline license
        let active_status = get_license_status().expect("get active status");
        assert!(active_status.active);
        assert!(active_status.valid);
        assert_eq!(
            active_status.license_key.as_deref(),
            Some("GCC-CAMARK-TEST-KEY-1234")
        );
        assert_eq!(active_status.plan_tier.as_deref(), Some("pro"));
        assert!(active_status.offline_capable);

        // Deactivate removes local license
        deactivate_license(None, None).expect("deactivate");
        let after_deact = get_license_status().expect("get status after deact");
        assert!(!after_deact.active);
    }
}
