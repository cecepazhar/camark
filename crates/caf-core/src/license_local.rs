//! Local license storage — offline-first entitlement.
//!
//! After activation the server returns an `expires_at` unix timestamp.
//! This module persists it locally so every subsequent `is_pro()` check
//! works without network.
//!
//! The file is plain JSON and is not signed, so it cannot prove on its own that a key was
//! bought. What this module does guarantee: the key has the `CA[PROD][TIER]-[RAND6]-[CK]`
//! shape with a correct checksum, it belongs to an accepted product, a timed key never turns
//! into a lifetime one, and a key the server has rejected stays rejected.

use crate::error::{CafError, IoError};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LocalLicense {
    pub key: String,
    pub product: String,
    pub tier: String,
    pub expires_at: i64, // 0 means "not given"; only a lifetime key may leave it at 0
    #[serde(default)]
    pub activation_token: String,
    #[serde(default)]
    pub activated_at: String,
    #[serde(default)]
    pub device_id: String,
    /// Set when the license server answered that this key is not valid for this device.
    /// Cleared again by a later successful check or a new activation.
    #[serde(default)]
    pub server_rejected: bool,
}

/// How long a key lasts, read from its two-digit tier code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyTier {
    /// Tier `99`.
    Lifetime,
    /// Tier `24`, `12` or `01`: valid for this many months from activation.
    Months(u32),
}

/// A license key that passed the format and checksum rules.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedKey {
    /// Two-letter product code, e.g. `MK`.
    pub product: String,
    /// Two-digit tier code as written in the key, e.g. `99`.
    pub tier_code: String,
    pub tier: KeyTier,
}

/// Trims and upper-cases a key the way it is stored and sent to the server.
pub fn normalize_key(key: &str) -> String {
    key.trim().to_ascii_uppercase()
}

/// The mod-10 check digit over every character before the last hyphen: digits count as
/// themselves, `A`..`Z` as 10..35, hyphens are skipped.
fn check_digit(body: &str) -> Option<u32> {
    let mut sum = 0u32;
    for c in body.chars() {
        match c {
            '-' => {}
            '0'..='9' => sum += c as u32 - '0' as u32,
            'A'..='Z' => sum += c as u32 - 'A' as u32 + 10,
            _ => return None,
        }
    }
    Some((10 - (sum % 10)) % 10)
}

/// Parses `CA[PROD][TIER]-[RAND6]-[CK]`. Returns `None` for any other shape, an unknown tier,
/// or a wrong check digit.
pub fn parse_license_key(key: &str) -> Option<ParsedKey> {
    let key = normalize_key(key);
    if key.len() != 15 || !key.is_ascii() {
        return None;
    }
    let (body, check) = key.rsplit_once('-')?;
    let (head, rand) = body.split_once('-')?;
    if head.len() != 6 || rand.len() != 6 || check.len() != 1 {
        return None;
    }
    if !head.starts_with("CA") {
        return None;
    }
    let product = &head[2..4];
    let tier_code = &head[4..6];
    if !product.chars().all(|c| c.is_ascii_uppercase()) {
        return None;
    }
    if !rand
        .chars()
        .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
    {
        return None;
    }
    let tier = match tier_code {
        "99" => KeyTier::Lifetime,
        "24" => KeyTier::Months(24),
        "12" => KeyTier::Months(12),
        "01" => KeyTier::Months(1),
        _ => return None,
    };
    let given = check.chars().next()?.to_digit(10)?;
    if check_digit(body)? != given {
        return None;
    }
    Some(ParsedKey {
        product: product.to_string(),
        tier_code: tier_code.to_string(),
        tier,
    })
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

/// When a license stops being valid.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Expiry {
    /// A lifetime key with no expiry.
    Never,
    /// Unix timestamp: the stored `expires_at`, or for a timed key without one, the
    /// activation date plus the months its tier grants.
    At(i64),
    /// The key is malformed, or it is a timed key with neither an expiry nor a readable
    /// activation date. Such a license is never treated as lifetime.
    Undetermined,
}

impl Expiry {
    /// The expiry as a unix timestamp, if there is one.
    pub fn unix(self) -> Option<i64> {
        match self {
            Self::At(ts) => Some(ts),
            Self::Never | Self::Undetermined => None,
        }
    }
}

/// Works out when `lic` stops being valid. See [`Expiry`].
pub fn effective_expiry(lic: &LocalLicense) -> Expiry {
    let Some(parsed) = parse_license_key(&lic.key) else {
        return Expiry::Undetermined;
    };
    if lic.expires_at > 0 {
        return Expiry::At(lic.expires_at);
    }
    match parsed.tier {
        KeyTier::Lifetime => Expiry::Never,
        KeyTier::Months(months) => chrono::DateTime::parse_from_rfc3339(lic.activated_at.trim())
            .ok()
            .map(|dt| dt.with_timezone(&chrono::Utc))
            .and_then(|dt| dt.checked_add_months(chrono::Months::new(months)))
            .map_or(Expiry::Undetermined, |dt| Expiry::At(dt.timestamp())),
    }
}

/// Returns true when the license is valid right now for one of `products` (two-letter codes).
pub fn is_license_valid(lic: &LocalLicense, products: &[&str]) -> bool {
    is_license_valid_at(lic, products, chrono::Utc::now().timestamp())
}

/// [`is_license_valid`] against an explicit clock.
pub fn is_license_valid_at(lic: &LocalLicense, products: &[&str], now: i64) -> bool {
    if lic.server_rejected {
        return false;
    }
    let Some(parsed) = parse_license_key(&lic.key) else {
        return false;
    };
    if !products.contains(&parsed.product.as_str()) {
        return false;
    }
    match effective_expiry(lic) {
        Expiry::Never => true,
        Expiry::At(expires_at) => now <= expires_at,
        Expiry::Undetermined => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const LIFETIME: &str = "CAMK99-TEST01-7";
    const YEARLY: &str = "CAMK12-TEST02-1";
    const MONTHLY: &str = "CAMK01-TEST03-2";
    const OTHER_PRODUCT: &str = "CATM99-TEST04-5";
    const ALL_ACCESS: &str = "CAAA99-TEST05-5";
    const PRODUCTS: &[&str] = &["MK", "AA"];
    const NOW: i64 = 1_800_000_000;

    fn lic(key: &str, expires_at: i64, activated_at: &str) -> LocalLicense {
        LocalLicense {
            key: key.to_string(),
            product: "MK".to_string(),
            tier: "99".to_string(),
            expires_at,
            activation_token: String::new(),
            activated_at: activated_at.to_string(),
            device_id: String::new(),
            server_rejected: false,
        }
    }

    #[test]
    fn parses_well_formed_keys() {
        let k = parse_license_key(LIFETIME).expect("lifetime key");
        assert_eq!(k.product, "MK");
        assert_eq!(k.tier, KeyTier::Lifetime);
        assert_eq!(
            parse_license_key(YEARLY).map(|k| k.tier),
            Some(KeyTier::Months(12))
        );
        assert_eq!(
            parse_license_key(MONTHLY).map(|k| k.tier),
            Some(KeyTier::Months(1))
        );
        assert_eq!(
            parse_license_key("  camk99-test01-7 ").map(|k| k.product),
            Some("MK".to_string())
        );
    }

    #[test]
    fn rejects_malformed_keys() {
        for bad in [
            "",
            "12345678",
            "sk-pro-anything",
            "CAMK99-TEST01-8",  // wrong check digit
            "CAMK50-TEST01-7",  // unknown tier
            "XXMK99-TEST01-7",  // wrong prefix
            "CAMK99-TEST0-7",   // short random part
            "CAMK99-TEST01-77", // long check
            "CAMK99-TÉST1-7",   // non-ASCII
        ] {
            assert!(parse_license_key(bad).is_none(), "accepted {bad:?}");
        }
    }

    #[test]
    fn lifetime_key_without_expiry_is_valid() {
        assert!(is_license_valid_at(&lic(LIFETIME, 0, ""), PRODUCTS, NOW));
        assert!(is_license_valid_at(&lic(ALL_ACCESS, 0, ""), PRODUCTS, NOW));
    }

    #[test]
    fn other_product_key_is_not_valid() {
        assert!(!is_license_valid_at(
            &lic(OTHER_PRODUCT, 0, ""),
            PRODUCTS,
            NOW
        ));
    }

    #[test]
    fn timed_key_without_expiry_is_never_lifetime() {
        // No expiry and no activation date: invalid, not lifetime.
        assert!(!is_license_valid_at(&lic(MONTHLY, 0, ""), PRODUCTS, NOW));
        assert!(!is_license_valid_at(
            &lic(YEARLY, 0, "garbage"),
            PRODUCTS,
            NOW
        ));
    }

    #[test]
    fn timed_key_falls_back_to_activation_date_plus_tier() {
        let monthly = lic(MONTHLY, 0, "2026-11-10T00:00:00Z");
        let activated = chrono::DateTime::parse_from_rfc3339("2026-11-10T00:00:00Z")
            .expect("date")
            .timestamp();
        assert!(is_license_valid_at(&monthly, PRODUCTS, activated + 86_400));
        assert!(!is_license_valid_at(
            &monthly,
            PRODUCTS,
            activated + 32 * 86_400
        ));
    }

    #[test]
    fn stored_expiry_is_enforced() {
        assert!(is_license_valid_at(
            &lic(YEARLY, NOW + 10, ""),
            PRODUCTS,
            NOW
        ));
        assert!(!is_license_valid_at(
            &lic(YEARLY, NOW - 10, ""),
            PRODUCTS,
            NOW
        ));
        // An expiry on a lifetime key is honoured too.
        assert!(!is_license_valid_at(
            &lic(LIFETIME, NOW - 10, ""),
            PRODUCTS,
            NOW
        ));
    }

    #[test]
    fn server_rejection_overrides_everything() {
        let mut l = lic(LIFETIME, 0, "");
        l.server_rejected = true;
        assert!(!is_license_valid_at(&l, PRODUCTS, NOW));
    }

    #[test]
    fn handwritten_file_with_arbitrary_key_is_not_valid() {
        assert!(!is_license_valid_at(
            &lic("GCC-CAMARK-TEST-KEY-1234", 0, ""),
            PRODUCTS,
            NOW
        ));
    }
}
