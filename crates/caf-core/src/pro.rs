//! Generic Pro Licensing & Account client for CAMark.

use crate::error::{CafError, ProError};
use ed25519_dalek::VerifyingKey;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProAccount {
    pub id: String,
    pub email: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProLicense {
    pub status: String,
    pub tier: String,
    pub trial_ends_at: Option<i64>,
    pub current_period_end: Option<i64>,
    pub entitled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Entitlement {
    pub state: String,
    pub tier: String,
    pub features: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProStatus {
    pub signed_in: bool,
    pub pending: bool,
    pub account: Option<ProAccount>,
    pub license: Option<ProLicense>,
    pub entitlement: Entitlement,
    pub last_sync_at: Option<i64>,
    pub key_configured: bool,
    pub is_pro: bool,
    pub plan: String,
    pub email: Option<String>,
    pub expires_at: Option<String>,
}

/// Aligned Ed25519 GCC Pro license public key
pub fn public_key(version: u32) -> Option<VerifyingKey> {
    let compiled = match version {
        1 => option_env!("CMRK_PRO_LICENSE_PUBKEY_V1").or(Some(
            "4aed277ac7b92ee58778d3c5233ebda741c652c753e3dae7127f367dbb58d92f",
        )),
        2 => option_env!("CMRK_PRO_LICENSE_PUBKEY_V2"),
        _ => None,
    };
    #[cfg(debug_assertions)]
    let runtime = std::env::var("CMRK_PRO_LICENSE_PUBKEY").ok();
    #[cfg(not(debug_assertions))]
    let runtime: Option<String> = None;

    let hex_key = compiled.map(str::to_string).or(runtime)?;
    let bytes: [u8; 32] = hex::decode(hex_key.trim()).ok()?.try_into().ok()?;
    VerifyingKey::from_bytes(&bytes).ok()
}

/// Pro status, read from the local license file only. No network and no vault access, so it
/// also works on the lock screen.
///
/// The single source of truth is [`crate::license_client::get_license_status`]: a license
/// activated against GCC. Nothing stored in the vault's `app_kv` table grants Pro.
pub fn get_pro_status() -> Result<ProStatus, CafError> {
    let lic_status = crate::license_client::get_license_status().unwrap_or_default();
    let is_pro = lic_status.valid;

    let license = is_pro.then(|| ProLicense {
        status: "active".to_string(),
        tier: "pro".to_string(),
        trial_ends_at: None,
        current_period_end: lic_status.expires_at_unix,
        entitled: true,
    });

    let entitlement = if is_pro {
        Entitlement {
            state: "valid".to_string(),
            tier: "pro".to_string(),
            features: vec![
                "cloud_sync".to_string(),
                "pdf_export".to_string(),
                "ai_copilot".to_string(),
                "table_tools".to_string(),
            ],
        }
    } else {
        Entitlement {
            state: "none".to_string(),
            tier: "free".to_string(),
            features: vec![],
        }
    };

    Ok(ProStatus {
        signed_in: is_pro,
        pending: false,
        account: None,
        license,
        entitlement,
        last_sync_at: None,
        key_configured: true,
        is_pro,
        plan: if is_pro {
            "pro".to_string()
        } else {
            "free".to_string()
        },
        email: None,
        expires_at: if is_pro { lic_status.expires_at } else { None },
    })
}

pub fn server_available() -> bool {
    true
}

/// GCC account sign-in is not built yet. This used to mark any email as Pro without checking
/// anything; it now always fails and grants nothing. Pro is unlocked with a license key.
pub fn login(_email: &str, _password: &str) -> Result<ProAccount, CafError> {
    Err(CafError::Pro(ProError::Generic(
        "ACCOUNT_LOGIN_UNAVAILABLE: Login akun belum tersedia. Aktifkan Pro dengan kunci lisensi."
            .into(),
    )))
}

/// See [`login`]: account registration is not built yet and grants nothing.
pub fn register(_email: &str, _password: &str, _name: &str, _locale: &str) -> Result<(), CafError> {
    Err(CafError::Pro(ProError::Generic(
        "ACCOUNT_LOGIN_UNAVAILABLE: Pendaftaran akun belum tersedia. Aktifkan Pro dengan kunci lisensi."
            .into(),
    )))
}

/// Activates a license key against GCC and returns the resulting status. The key's format,
/// checksum and product are checked before anything is sent.
pub fn activate_license(key: &str) -> Result<ProStatus, CafError> {
    crate::license_client::activate_license(None, key, None)?;
    get_pro_status()
}

pub fn logout() -> Result<(), CafError> {
    crate::license_client::deactivate_license(None, None)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn login_and_register_grant_nothing() {
        let _guard = crate::test_support::isolated_data_dir("pro_login_gate");
        assert!(login("anyone@example.com", "whatever").is_err());
        assert!(register("anyone@example.com", "whatever", "Anyone", "id").is_err());
        let status = get_pro_status().expect("status");
        assert!(!status.is_pro);
        assert!(!status.signed_in);
        assert_eq!(status.plan, "free");
        assert!(status.entitlement.features.is_empty());
    }

    #[test]
    fn legacy_billing_rows_grant_nothing() {
        let _guard = crate::test_support::isolated_data_dir("pro_legacy_rows");
        crate::vault::validate_password("test-password-p0-3").expect("unlock vault");
        let conn = crate::db::open().expect("db");
        conn.execute(
            "INSERT OR REPLACE INTO app_kv (key, value) VALUES ('billing.plan', 'pro'), ('billing.license_key', '12345678'), ('billing.email', 'a@b.c')",
            [],
        )
        .expect("seed legacy rows");
        let status = get_pro_status().expect("status");
        assert!(!status.is_pro);
        assert!(status.license.is_none());
    }

    #[test]
    fn activation_rejects_short_arbitrary_key() {
        let _guard = crate::test_support::isolated_data_dir("pro_activate_gate");
        assert!(activate_license("12345678").is_err());
        assert!(!get_pro_status().expect("status").is_pro);
    }
}
