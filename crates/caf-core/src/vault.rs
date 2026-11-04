//! Zero-Knowledge Argon2id Vault Manager.

use crate::error::{CatermError, VaultError};
use argon2::{
    Argon2, Params, Version,
    password_hash::rand_core::{OsRng, RngCore},
};
use parking_lot::RwLock;
use std::sync::LazyLock;
use zeroize::Zeroize;

pub const MIN_PASSWORD_LEN: usize = 8;

const CANARY_FILE: &str = "vault_canary.bin";
const CANARY_PLAINTEXT: &[u8] = b"CMRKRAMEWORK_VAULT_CANARY_V2";
const ARGON2_M_COST: u32 = 64 * 1024; // 64 MB
const ARGON2_T_COST: u32 = 3;
const ARGON2_P_COST: u32 = 4;

const SALT_FILE: &str = "vault_salt.bin";
const SALT_LEN: usize = 16;

static ACTIVE_VAULT_KEY: LazyLock<RwLock<Option<[u8; 32]>>> = LazyLock::new(|| RwLock::new(None));

fn vault_err(msg: impl Into<String>) -> CatermError {
    CatermError::Vault(VaultError::Generic(msg.into()))
}

pub fn is_unlocked() -> Result<bool, CatermError> {
    let guard = ACTIVE_VAULT_KEY.read();
    Ok(guard.is_some())
}

pub fn ensure_unlocked_key() -> Result<String, CatermError> {
    let data_dir = crate::paths::resolve_data_dir()?.path;
    load_or_create_local_key(&data_dir)
}

pub fn is_vault_initialized() -> Result<bool, CatermError> {
    let data_dir = crate::paths::resolve_data_dir()?.path;
    Ok(data_dir.join(CANARY_FILE).exists())
}

pub fn lock() {
    let mut guard = ACTIVE_VAULT_KEY.write();
    if let Some(mut key) = guard.take() {
        key.zeroize();
    }
}

pub fn reset_vault() -> Result<(), CatermError> {
    let data_dir = crate::paths::resolve_data_dir()?.path;

    {
        let mut guard = ACTIVE_VAULT_KEY.write();
        if let Some(mut key) = guard.take() {
            key.zeroize();
        }
    }

    let files = [
        CANARY_FILE,
        SALT_FILE,
        "vault.key",
        "local.key",
        "camark.db",
        "camark.db-wal",
        "camark.db-shm",
        "camark.db-journal",
    ];
    for f in &files {
        let p = data_dir.join(f);
        if p.exists() {
            let _ = std::fs::remove_file(&p);
        }
    }

    Ok(())
}

fn check_password_len(password: &str) -> Result<(), CatermError> {
    if password.len() < MIN_PASSWORD_LEN {
        return Err(vault_err(format!(
            "Master password minimal {MIN_PASSWORD_LEN} karakter."
        )));
    }
    Ok(())
}

fn derive_key(master_password: &str, salt: &[u8]) -> Result<[u8; 32], CatermError> {
    check_password_len(master_password)?;

    let mut derived_key = [0u8; 32];
    let params = Params::new(ARGON2_M_COST, ARGON2_T_COST, ARGON2_P_COST, Some(32))
        .map_err(|e| vault_err(e.to_string()))?;
    let argon2 = Argon2::new(argon2::Algorithm::Argon2id, Version::V0x13, params);

    argon2
        .hash_password_into(master_password.as_bytes(), salt, &mut derived_key)
        .map_err(|e| vault_err(e.to_string()))?;
    Ok(derived_key)
}

fn load_or_create_salt(data_dir: &std::path::Path) -> Result<[u8; SALT_LEN], CatermError> {
    let path = data_dir.join(SALT_FILE);
    if let Ok(bytes) = std::fs::read(&path)
        && let Ok(salt) = <[u8; SALT_LEN]>::try_from(bytes.as_slice())
    {
        return Ok(salt);
    }

    std::fs::create_dir_all(data_dir).map_err(|e| vault_err(e.to_string()))?;
    let mut salt = [0u8; SALT_LEN];
    OsRng.fill_bytes(&mut salt);
    std::fs::write(&path, salt).map_err(|e| vault_err(e.to_string()))?;
    Ok(salt)
}

fn verify_canary(key: &[u8; 32], canary_path: &std::path::Path) -> Result<(), CatermError> {
    let wrong_password = || vault_err("Master password salah. Silakan coba lagi.");
    let encrypted_canary =
        std::fs::read_to_string(canary_path).map_err(|e| vault_err(e.to_string()))?;
    let decrypted =
        crate::secret::decrypt_bytes(key, &encrypted_canary).map_err(|_| wrong_password())?;
    if decrypted != CANARY_PLAINTEXT {
        return Err(wrong_password());
    }
    Ok(())
}

pub fn validate_password(password: &str) -> Result<bool, CatermError> {
    let data_dir = crate::paths::resolve_data_dir()?.path;
    let canary_path = data_dir.join(CANARY_FILE);

    if !canary_path.exists() {
        // First run — initialize vault
        let salt = load_or_create_salt(&data_dir)?;
        let key = derive_key(password, &salt)?;
        let encrypted = crate::secret::encrypt_bytes(&key, CANARY_PLAINTEXT)?;
        std::fs::write(&canary_path, encrypted).map_err(|e| vault_err(e.to_string()))?;

        {
            let mut guard = ACTIVE_VAULT_KEY.write();
            *guard = Some(key);
        }

        let _ = crate::db::open()?;
        let _ = crate::audit::log_event("VAULT_INIT", None, "Vault initialized");
        return Ok(true);
    }

    let salt = load_or_create_salt(&data_dir)?;
    let key = derive_key(password, &salt)?;
    if verify_canary(&key, &canary_path).is_ok() {
        {
            let mut guard = ACTIVE_VAULT_KEY.write();
            *guard = Some(key);
        }
        let _ = crate::db::open()?;
        let _ = crate::audit::log_event("VAULT_UNLOCK", None, "Vault unlocked");
        Ok(true)
    } else {
        Err(vault_err("Master password salah."))
    }
}

pub fn change_master_password(old_pass: &str, new_pass: &str) -> Result<(), CatermError> {
    if old_pass == new_pass {
        return Err(vault_err("Master password baru harus berbeda."));
    }
    let data_dir = crate::paths::resolve_data_dir()?.path;
    let canary_path = data_dir.join(CANARY_FILE);
    if !canary_path.exists() {
        return Err(vault_err("Vault belum dibuat."));
    }

    let salt = load_or_create_salt(&data_dir)?;
    let old_key = derive_key(old_pass, &salt)?;
    verify_canary(&old_key, &canary_path)?;

    let new_key = derive_key(new_pass, &salt)?;

    // NOTE (H-02): the SQLCipher database is keyed by `vault.key`, NOT by the
    // master-password-derived key (see H-01 for binding them together). Rekeying
    // the DB with `new_key` here made every later `db::open()` fail and used to
    // trigger a silent "rename to .bak + create empty DB". We therefore only
    // replace the canary, and do it atomically: write a temp file, verify it
    // decrypts with `new_key`, then rename over the old canary. Any failure
    // leaves the old canary (and old password) fully intact.
    let encrypted = crate::secret::encrypt_bytes(&new_key, CANARY_PLAINTEXT)?;
    let tmp_path = data_dir.join(format!("{CANARY_FILE}.tmp"));
    std::fs::write(&tmp_path, &encrypted).map_err(|e| vault_err(e.to_string()))?;
    if let Err(e) = verify_canary(&new_key, &tmp_path) {
        let _ = std::fs::remove_file(&tmp_path);
        return Err(e);
    }
    if let Err(e) = std::fs::rename(&tmp_path, &canary_path) {
        let _ = std::fs::remove_file(&tmp_path);
        return Err(vault_err(e.to_string()));
    }

    {
        let mut guard = ACTIVE_VAULT_KEY.write();
        *guard = Some(new_key);
    }

    let _ = crate::audit::log_event("VAULT_REKEY", None, "Master password changed");
    Ok(())
}

pub fn load_or_create_local_key(data_dir: &std::path::Path) -> Result<String, CatermError> {
    let path = data_dir.join("vault.key");
    if let Ok(content) = std::fs::read_to_string(&path) {
        let trimmed = content.trim();
        if trimmed.len() == 64 && hex::decode(trimmed).is_ok() {
            return Ok(trimmed.to_string());
        }
    }

    std::fs::create_dir_all(data_dir).map_err(|e| vault_err(e.to_string()))?;
    let mut bytes = [0u8; 32];
    OsRng.fill_bytes(&mut bytes);
    let hex_key = hex::encode(bytes);
    std::fs::write(&path, &hex_key).map_err(|e| vault_err(e.to_string()))?;
    Ok(hex_key)
}

#[cfg(test)]
mod tests {
    use super::*;

    const OLD_PASS: &str = "password-lama-123";
    const NEW_PASS: &str = "password-baru-456";

    fn seed_rows() {
        let conn = crate::db::open().expect("open db");
        for i in 0..3 {
            conn.execute(
                "INSERT INTO app_kv (key, value) VALUES (?1, ?2)",
                [format!("h02_k{i}"), format!("v{i}")],
            )
            .expect("insert");
        }
    }

    fn count_rows() -> i64 {
        let conn = crate::db::open().expect("open db");
        conn.query_row(
            "SELECT count(*) FROM app_kv WHERE key LIKE 'h02_k%'",
            [],
            |r| r.get(0),
        )
        .expect("count")
    }

    #[test]
    fn h02_change_password_keeps_data_and_swaps_password() {
        let dir = crate::test_support::isolated_data_dir("h02_change_pw");
        assert!(validate_password(OLD_PASS).expect("init"));
        seed_rows();
        assert_eq!(count_rows(), 3);

        change_master_password(OLD_PASS, NEW_PASS).expect("change password");
        lock();

        assert!(validate_password(OLD_PASS).is_err(), "old password must be rejected");
        assert!(validate_password(NEW_PASS).expect("unlock new"));
        assert_eq!(count_rows(), 3, "rows must survive password change");

        let leftovers: Vec<_> = std::fs::read_dir(&dir.path)
            .expect("read dir")
            .filter_map(|e| e.ok())
            .map(|e| e.file_name().to_string_lossy().to_string())
            .filter(|n| n.contains(".bak") || n.ends_with(".tmp"))
            .collect();
        assert!(leftovers.is_empty(), "no .bak/.tmp files expected: {leftovers:?}");
    }

    #[test]
    fn h02_wrong_old_password_changes_nothing() {
        let _dir = crate::test_support::isolated_data_dir("h02_wrong_old");
        assert!(validate_password(OLD_PASS).expect("init"));
        seed_rows();

        assert!(change_master_password("salah-total-999", NEW_PASS).is_err());
        lock();
        assert!(validate_password(OLD_PASS).expect("old still works"));
        assert_eq!(count_rows(), 3);
    }

    #[test]
    fn h02_failed_canary_swap_keeps_old_password() {
        let dir = crate::test_support::isolated_data_dir("h02_swap_fail");
        assert!(validate_password(OLD_PASS).expect("init"));
        seed_rows();

        // Force the temp-canary write to fail by occupying its path with a directory.
        let tmp = dir.path.join(format!("{CANARY_FILE}.tmp"));
        std::fs::create_dir(&tmp).expect("mkdir at tmp path");

        assert!(change_master_password(OLD_PASS, NEW_PASS).is_err());
        std::fs::remove_dir(&tmp).expect("cleanup");

        lock();
        assert!(validate_password(OLD_PASS).expect("old password still valid"));
        assert_eq!(count_rows(), 3);
    }
}
