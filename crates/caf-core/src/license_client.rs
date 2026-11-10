//! Official GCC Backend License Client for CAMark (GCC /api/v1/licenses/*).
//!
//! Pro status comes from one place only: the local license file written by a successful
//! activation against GCC. A key must have the `CA[PROD][TIER]-[RAND6]-[CK]` shape with a
//! correct checksum and belong to CAMark or All-Access. The app re-checks the stored key with
//! GCC once per start; a key GCC rejects stops granting Pro until GCC accepts it again.

use crate::error::{CafError, IoError, ProError};
use crate::license_local::{self, Expiry, KeyTier, LocalLicense};
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

pub const DEFAULT_GCC_BASE_URL: &str = "https://gcc.fathforce.com";
pub const PRODUCT_CODE: &str = "CAMARK";
/// Two-letter product codes whose keys unlock CAMark Pro: CAMark itself and All-Access.
pub const ACCEPTED_KEY_PRODUCTS: &[&str] = &["MK", "AA"];

const LICENSE_APP: &str = "camark";
const HTTP_TIMEOUT: Duration = Duration::from_secs(10);

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

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LicenseStatusInfo {
    pub active: bool,
    pub valid: bool,
    pub license_key: Option<String>,
    pub product_sku: Option<String>,
    pub plan_tier: Option<String>,
    pub device_id: Option<String>,
    /// RFC 3339 timestamp, or `"Lifetime"`.
    pub expires_at: Option<String>,
    /// Unix timestamp of the expiry; `None` for lifetime or when there is no license.
    #[serde(default)]
    pub expires_at_unix: Option<i64>,
    pub offline_capable: bool,
}

/// What GCC said about a key on this device.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServerVerdict {
    /// GCC confirmed the key for this device.
    Valid,
    /// GCC answered and refused the key (unknown, revoked, expired, or not this device).
    Rejected,
    /// No usable answer: offline, timeout, server error. Local state is left alone.
    Unknown,
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

/// A timestamp GCC may send either as a unix number or as text (RFC 3339 or digits).
#[derive(Deserialize)]
#[serde(untagged)]
enum Timestamp {
    Unix(i64),
    Text(String),
}

impl Timestamp {
    fn to_unix(&self) -> Option<i64> {
        match self {
            Self::Unix(n) => Some(*n),
            Self::Text(s) => {
                let s = s.trim();
                chrono::DateTime::parse_from_rfc3339(s)
                    .map(|dt| dt.timestamp())
                    .ok()
                    .or_else(|| s.parse::<i64>().ok())
            }
        }
    }
}

fn unix_to_rfc3339(ts: i64) -> Option<String> {
    chrono::DateTime::from_timestamp(ts, 0).map(|dt| dt.to_rfc3339())
}

#[derive(Deserialize)]
struct ActivateData {
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
    #[serde(default)]
    expires_at: Option<Timestamp>,
    #[serde(default)]
    activated_at: Option<Timestamp>,
    #[serde(default)]
    activation_token: String,
}

#[derive(Deserialize)]
struct VerifyData {
    valid: bool,
}

fn agent() -> ureq::Agent {
    let config = ureq::Agent::config_builder()
        .timeout_global(Some(HTTP_TIMEOUT))
        .build();
    ureq::Agent::new_with_config(config)
}

fn pro_err(msg: impl Into<String>) -> CafError {
    CafError::Pro(ProError::Generic(msg.into()))
}

/// Base URL of the license server. An explicit `custom` URL wins; the `GCC_BASE_URL`
/// environment variable is honoured in debug builds only, so a release build cannot be
/// pointed at another server through the environment.
pub fn resolve_gcc_url(custom: Option<&str>) -> String {
    if let Some(c) = custom {
        let trimmed = c.trim();
        if !trimmed.is_empty() {
            return trimmed.trim_end_matches('/').to_string();
        }
    }
    #[cfg(debug_assertions)]
    if let Ok(env_url) = std::env::var("GCC_BASE_URL") {
        let trimmed = env_url.trim();
        if !trimmed.is_empty() {
            return trimmed.trim_end_matches('/').to_string();
        }
    }
    DEFAULT_GCC_BASE_URL.to_string()
}

/// Checks the key's shape, checksum and product. Returns the normalized key and its parse.
fn checked_key(key: &str) -> Result<(String, license_local::ParsedKey), CafError> {
    let clean = license_local::normalize_key(key);
    let parsed = license_local::parse_license_key(&clean).ok_or_else(|| {
        pro_err("INVALID_LICENSE_KEY: Format kunci harus CAMK99-XXXXXX-X dan checksum harus cocok")
    })?;
    if !ACCEPTED_KEY_PRODUCTS.contains(&parsed.product.as_str()) {
        return Err(pro_err(
            "LICENSE_WRONG_PRODUCT: Kunci ini bukan untuk CAMark",
        ));
    }
    Ok((clean, parsed))
}

/// GET /api/v1/licenses/validate/:key
pub fn validate_license_key(gcc_url: Option<&str>, key: &str) -> Result<bool, CafError> {
    let Ok((clean_key, _)) = checked_key(key) else {
        return Ok(false);
    };
    let base = resolve_gcc_url(gcc_url);
    let url = format!("{base}/api/v1/licenses/validate/{clean_key}");

    let resp = agent()
        .get(&url)
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
    let (clean_key, parsed) = checked_key(key)?;

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

    let resp = agent()
        .post(&url)
        .send_json(&payload)
        .map_err(|e| match e {
            ureq::Error::StatusCode(404) => {
                pro_err("LICENSE_NOT_FOUND: Kunci tidak terdaftar di GCC")
            }
            ureq::Error::StatusCode(409) => {
                pro_err("DEVICE_LIMIT_EXCEEDED: Jumlah device melebihi limit lisensi")
            }
            ureq::Error::StatusCode(403 | 410) => {
                pro_err("LICENSE_EXPIRED_OR_INACTIVE: Lisensi kedaluwarsa atau nonaktif")
            }
            other => CafError::Io(IoError::Generic(format!(
                "GCC activation network error: {other}"
            ))),
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
        return Err(pro_err(err_msg));
    }

    let data = envelope
        .data
        .ok_or_else(|| pro_err("Empty activation payload from GCC"))?;

    // A missing or unreadable activation date becomes "now", so a timed key always has a
    // date to count its months from and can never fall through to lifetime.
    let activated_at = data
        .activated_at
        .as_ref()
        .and_then(Timestamp::to_unix)
        .and_then(unix_to_rfc3339)
        .unwrap_or_else(|| chrono::Utc::now().to_rfc3339());
    let server_expiry = data
        .expires_at
        .as_ref()
        .and_then(Timestamp::to_unix)
        .filter(|ts| *ts > 0);

    let local = LocalLicense {
        key: clean_key.clone(),
        product: parsed.product.clone(),
        tier: parsed.tier_code.clone(),
        expires_at: server_expiry.unwrap_or(0),
        activation_token: data.activation_token.clone(),
        activated_at: activated_at.clone(),
        device_id: if data.device_id.is_empty() {
            device_id
        } else {
            data.device_id.clone()
        },
        server_rejected: false,
    };
    let expiry = license_local::effective_expiry(&local);
    if expiry == Expiry::Undetermined
        || (parsed.tier != KeyTier::Lifetime && expiry == Expiry::Never)
    {
        return Err(pro_err(
            "INVALID_LICENSE_KEY: Masa berlaku lisensi tidak dapat ditentukan",
        ));
    }
    license_local::save_license(LICENSE_APP, &local)?;

    Ok(ActivationResult {
        success: true,
        license_key: clean_key,
        product_sku: if data.product_sku.is_empty() {
            PRODUCT_CODE.to_string()
        } else {
            data.product_sku
        },
        product_name: data.product_name,
        plan_tier: if data.plan_tier.is_empty() {
            "pro".to_string()
        } else {
            data.plan_tier
        },
        device_id: local.device_id,
        max_devices: data.max_devices,
        expires_at: expiry.unix().and_then(unix_to_rfc3339),
        activated_at: Some(activated_at),
        activation_token: data.activation_token,
    })
}

/// Asks GCC whether `key` is valid on this device (POST /api/v1/licenses/verify).
///
/// Only a clear answer counts as [`ServerVerdict::Rejected`]: a response body saying so, or
/// HTTP 403/404/410. Timeouts, other statuses and unreadable bodies are `Unknown`, so an
/// outage on the server never takes Pro away from a paying customer.
pub fn check_license_online(gcc_url: Option<&str>, key: &str) -> ServerVerdict {
    let Ok(device_id) = crate::paths::device_id() else {
        return ServerVerdict::Unknown;
    };
    let base = resolve_gcc_url(gcc_url);
    let url = format!("{base}/api/v1/licenses/verify");
    let payload = serde_json::json!({
        "license_key": license_local::normalize_key(key),
        "device_id": device_id
    });

    match agent().post(&url).send_json(&payload) {
        Ok(resp) => match resp.into_body().read_json::<GccEnvelope<VerifyData>>() {
            Ok(envelope) => match envelope.data {
                Some(data) if data.valid => ServerVerdict::Valid,
                Some(_) => ServerVerdict::Rejected,
                None if !envelope.success => ServerVerdict::Rejected,
                None => ServerVerdict::Unknown,
            },
            Err(_) => ServerVerdict::Unknown,
        },
        Err(ureq::Error::StatusCode(403 | 404 | 410)) => ServerVerdict::Rejected,
        Err(_) => ServerVerdict::Unknown,
    }
}

/// POST /api/v1/licenses/verify
pub fn verify_license(gcc_url: Option<&str>, key: Option<&str>) -> Result<bool, CafError> {
    let resolved_key = match key {
        Some(k) if !k.trim().is_empty() => k.trim().to_string(),
        _ => {
            if let Ok(Some(lic)) = license_local::load_license(LICENSE_APP) {
                lic.key
            } else {
                return Ok(false);
            }
        }
    };

    match check_license_online(gcc_url, &resolved_key) {
        ServerVerdict::Valid => Ok(true),
        ServerVerdict::Rejected => Ok(false),
        ServerVerdict::Unknown => Err(CafError::Io(IoError::Generic(
            "GCC verify network error: no usable answer from the license server".into(),
        ))),
    }
}

/// Re-checks the stored license with GCC and records the answer in the local file.
///
/// A rejected key stops granting Pro (the file is kept, not deleted); a key GCC accepts
/// again is restored. With no answer the local file is left as it is, which is the offline
/// allowance: Pro keeps working until the license's own expiry.
pub fn revalidate_stored_license(gcc_url: Option<&str>) -> ServerVerdict {
    let Ok(Some(mut lic)) = license_local::load_license(LICENSE_APP) else {
        return ServerVerdict::Unknown;
    };
    if license_local::parse_license_key(&lic.key).is_none() {
        // Not a key GCC could know; it is already invalid locally and is not sent anywhere.
        return ServerVerdict::Unknown;
    }
    let verdict = check_license_online(gcc_url, &lic.key);
    let rejected = match verdict {
        ServerVerdict::Valid => false,
        ServerVerdict::Rejected => true,
        ServerVerdict::Unknown => return verdict,
    };
    if lic.server_rejected != rejected {
        lic.server_rejected = rejected;
        let _ = license_local::save_license(LICENSE_APP, &lic);
    }
    verdict
}

/// Runs [`revalidate_stored_license`] against the default server the first time it is called
/// in this process and does nothing afterwards. Call it before reading Pro status at start.
pub fn revalidate_once() {
    static DONE: AtomicBool = AtomicBool::new(false);
    if DONE.swap(true, Ordering::SeqCst) {
        return;
    }
    let _ = revalidate_stored_license(None);
}

/// POST /api/v1/licenses/deactivate
pub fn deactivate_license(gcc_url: Option<&str>, key: Option<&str>) -> Result<bool, CafError> {
    let resolved_key = match key {
        Some(k) if !k.trim().is_empty() => Some(k.trim().to_string()),
        _ => license_local::load_license(LICENSE_APP)
            .ok()
            .flatten()
            .map(|lic| lic.key),
    };

    if let Some(resolved_key) = resolved_key {
        let base = resolve_gcc_url(gcc_url);
        let url = format!("{base}/api/v1/licenses/deactivate");
        let device_id = crate::paths::device_id()?;

        let payload = serde_json::json!({
            "license_key": resolved_key,
            "device_id": device_id
        });

        let _ = agent().post(&url).send_json(&payload);
    }

    // Clear local storage regardless of network result
    let _ = license_local::remove_license(LICENSE_APP);
    clear_legacy_billing_rows();

    Ok(true)
}

/// Removes the `billing.*` rows older builds wrote into the vault. They no longer grant
/// anything; deleting them keeps stale data from lingering. Skipped while the vault is locked.
pub(crate) fn clear_legacy_billing_rows() {
    if let Ok(conn) = crate::db::open() {
        let _ = conn.execute(
            "DELETE FROM app_kv WHERE key IN ('billing.email', 'billing.name', 'billing.license_key', 'billing.plan')",
            [],
        );
    }
}

/// Retrieves the current license status from the local license file. No network.
pub fn get_license_status() -> Result<LicenseStatusInfo, CafError> {
    if let Ok(Some(lic)) = license_local::load_license(LICENSE_APP) {
        let valid = license_local::is_license_valid(&lic, ACCEPTED_KEY_PRODUCTS);
        let expiry = license_local::effective_expiry(&lic).unix();
        let expires_str = match expiry {
            Some(ts) => unix_to_rfc3339(ts),
            None if valid => Some("Lifetime".to_string()),
            None => None,
        };

        return Ok(LicenseStatusInfo {
            active: valid,
            valid,
            license_key: Some(lic.key),
            product_sku: Some(PRODUCT_CODE.to_string()),
            plan_tier: valid.then(|| "pro".to_string()),
            device_id: if lic.device_id.is_empty() {
                crate::paths::device_id().ok()
            } else {
                Some(lic.device_id)
            },
            expires_at: expires_str,
            expires_at_unix: expiry,
            offline_capable: valid,
        });
    }

    Ok(LicenseStatusInfo {
        device_id: crate::paths::device_id().ok(),
        ..LicenseStatusInfo::default()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const LIFETIME: &str = "CAMK99-TEST01-7";
    const MONTHLY: &str = "CAMK01-TEST03-2";
    /// Nothing listens here, so calls fail at once instead of reaching the real server.
    const DEAD_SERVER: &str = "http://127.0.0.1:9";

    fn local(key: &str, expires_at: i64, activated_at: &str) -> LocalLicense {
        LocalLicense {
            key: key.to_string(),
            product: "MK".to_string(),
            tier: key[4..6].to_string(),
            expires_at,
            activation_token: String::new(),
            activated_at: activated_at.to_string(),
            device_id: "test-device-uuid".to_string(),
            server_rejected: false,
        }
    }

    #[test]
    fn test_resolve_gcc_url() {
        assert_eq!(
            resolve_gcc_url(Some("https://custom.gcc.com/")),
            "https://custom.gcc.com"
        );
        assert_eq!(resolve_gcc_url(None), DEFAULT_GCC_BASE_URL);
    }

    #[test]
    fn timestamp_accepts_number_and_text() {
        let n: Timestamp = serde_json::from_str("1800000000").expect("number");
        assert_eq!(n.to_unix(), Some(1_800_000_000));
        let s: Timestamp = serde_json::from_str("\"2027-01-15T08:00:00Z\"").expect("text");
        assert_eq!(s.to_unix(), Some(1_800_000_000));
        let digits: Timestamp = serde_json::from_str("\"1800000000\"").expect("digits");
        assert_eq!(digits.to_unix(), Some(1_800_000_000));
        let junk: Timestamp = serde_json::from_str("\"soon\"").expect("junk");
        assert_eq!(junk.to_unix(), None);
    }

    #[test]
    fn activation_refuses_malformed_or_foreign_keys_before_any_network() {
        let _guard = crate::test_support::isolated_data_dir("lic_activate_reject");
        for bad in ["12345678", "sk-pro-anything", "CAMK99-TEST01-8"] {
            let err = activate_license(Some(DEAD_SERVER), bad, None).expect_err("must fail");
            assert!(err.to_string().contains("INVALID_LICENSE_KEY"), "{err}");
        }
        let err = activate_license(Some(DEAD_SERVER), "CATM99-TEST04-5", None)
            .expect_err("other product");
        assert!(err.to_string().contains("LICENSE_WRONG_PRODUCT"), "{err}");
        assert!(!get_license_status().expect("status").valid);
    }

    #[test]
    fn failed_activation_grants_nothing() {
        let _guard = crate::test_support::isolated_data_dir("lic_activate_offline");
        assert!(activate_license(Some(DEAD_SERVER), LIFETIME, None).is_err());
        let status = get_license_status().expect("status");
        assert!(!status.valid);
        assert!(status.license_key.is_none());
    }

    #[test]
    fn test_offline_license_status_roundtrip() {
        let _guard = crate::test_support::isolated_data_dir("lic_status_test");

        // Initial: no license
        let initial = get_license_status().expect("get status");
        assert!(!initial.active);
        assert!(!initial.valid);

        license_local::save_license(LICENSE_APP, &local(LIFETIME, 0, "2026-11-10T00:00:00Z"))
            .expect("save license");

        // Verify status reflects active offline license
        let active_status = get_license_status().expect("get active status");
        assert!(active_status.active);
        assert!(active_status.valid);
        assert_eq!(active_status.license_key.as_deref(), Some(LIFETIME));
        assert_eq!(active_status.plan_tier.as_deref(), Some("pro"));
        assert_eq!(active_status.expires_at.as_deref(), Some("Lifetime"));
        assert!(active_status.offline_capable);

        // No answer from the server leaves the license alone.
        assert_eq!(
            revalidate_stored_license(Some(DEAD_SERVER)),
            ServerVerdict::Unknown
        );
        assert!(get_license_status().expect("status").valid);

        // Deactivate removes local license
        deactivate_license(Some(DEAD_SERVER), None).expect("deactivate");
        let after_deact = get_license_status().expect("get status after deact");
        assert!(!after_deact.active);
    }

    #[test]
    fn handwritten_license_file_does_not_grant_pro() {
        let _guard = crate::test_support::isolated_data_dir("lic_forged");
        license_local::save_license(LICENSE_APP, &local("GCC-CAMARK-TEST-KEY-1234", 0, ""))
            .expect("save");
        assert!(!get_license_status().expect("status").valid);
    }

    #[test]
    fn timed_license_without_expiry_is_not_lifetime() {
        let _guard = crate::test_support::isolated_data_dir("lic_timed");
        license_local::save_license(LICENSE_APP, &local(MONTHLY, 0, "")).expect("save");
        let status = get_license_status().expect("status");
        assert!(!status.valid);
        assert_ne!(status.expires_at.as_deref(), Some("Lifetime"));

        license_local::save_license(LICENSE_APP, &local(MONTHLY, 0, "2020-01-01T00:00:00Z"))
            .expect("save");
        assert!(!get_license_status().expect("status").valid, "long expired");
    }

    #[test]
    fn server_rejected_license_is_not_valid() {
        let _guard = crate::test_support::isolated_data_dir("lic_rejected");
        let mut lic = local(LIFETIME, 0, "");
        lic.server_rejected = true;
        license_local::save_license(LICENSE_APP, &lic).expect("save");
        assert!(!get_license_status().expect("status").valid);
    }
}
