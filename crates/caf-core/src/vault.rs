//! Zero-Knowledge Argon2id Vault Manager.

use crate::error::{CafError, DbError, VaultError};
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

fn vault_err(msg: impl Into<String>) -> CafError {
    CafError::Vault(VaultError::Generic(msg.into()))
}

pub fn is_unlocked() -> Result<bool, CafError> {
    let guard = ACTIVE_VAULT_KEY.read();
    Ok(guard.is_some())
}

/// Returns the hex-encoded master-password-derived key from the in-memory vault
/// state. When the vault is locked there is no key: callers get `CMRK-VAULT-001`
/// instead of a plaintext key read back from disk (P0.3/T-1).
pub fn ensure_unlocked_key() -> Result<String, CafError> {
    let guard = ACTIVE_VAULT_KEY.read();
    guard
        .as_ref()
        .map(hex::encode)
        .ok_or(CafError::Vault(VaultError::Locked))
}

pub fn is_vault_initialized() -> Result<bool, CafError> {
    let data_dir = crate::paths::resolve_data_dir()?.path;
    Ok(data_dir.join(CANARY_FILE).exists())
}

pub fn lock() {
    let mut guard = ACTIVE_VAULT_KEY.write();
    if let Some(mut key) = guard.take() {
        key.zeroize();
    }
}

pub fn reset_vault() -> Result<(), CafError> {
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

fn check_password_len(password: &str) -> Result<(), CafError> {
    if password.len() < MIN_PASSWORD_LEN {
        return Err(vault_err(format!(
            "Master password minimal {MIN_PASSWORD_LEN} karakter."
        )));
    }
    Ok(())
}

fn derive_key(master_password: &str, salt: &[u8]) -> Result<[u8; 32], CafError> {
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

fn load_or_create_salt(data_dir: &std::path::Path) -> Result<[u8; SALT_LEN], CafError> {
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

fn verify_canary(key: &[u8; 32], canary_path: &std::path::Path) -> Result<(), CafError> {
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

pub fn validate_password(password: &str) -> Result<bool, CafError> {
    let data_dir = crate::paths::resolve_data_dir()?.path;
    let canary_path = data_dir.join(CANARY_FILE);

    if !canary_path.exists() {
        // First run — initialize vault
        let salt = load_or_create_salt(&data_dir)?;
        let mut key = derive_key(password, &salt)?;
        let encrypted = crate::secret::encrypt_bytes(&key, CANARY_PLAINTEXT)?;
        std::fs::write(&canary_path, encrypted).map_err(|e| vault_err(e.to_string()))?;

        {
            let mut guard = ACTIVE_VAULT_KEY.write();
            *guard = Some(key);
        }

        migrate_legacy_vault_key(&key)?;

        let _ = crate::db::open()?;
        let _ = crate::audit::log_event("VAULT_INIT", None, "Vault initialized");
        return Ok(true);
    }

    let salt = load_or_create_salt(&data_dir)?;
    let mut key = derive_key(password, &salt)?;
    if verify_canary(&key, &canary_path).is_ok() {
        {
            let mut guard = ACTIVE_VAULT_KEY.write();
            *guard = Some(key);
        }
        migrate_legacy_vault_key(&key)?;

        let _ = crate::db::open()?;
        let _ = crate::audit::log_event("VAULT_UNLOCK", None, "Vault unlocked");
        Ok(true)
    } else {
        Err(vault_err("Master password salah."))
    }
}

pub fn change_master_password(old_pass: &str, new_pass: &str) -> Result<(), CafError> {
    if old_pass == new_pass {
        return Err(vault_err("Master password baru harus berbeda."));
    }
    let data_dir = crate::paths::resolve_data_dir()?.path;
    let canary_path = data_dir.join(CANARY_FILE);
    if !canary_path.exists() {
        return Err(vault_err("Vault belum dibuat."));
    }

    let salt = load_or_create_salt(&data_dir)?;
    let mut old_key = derive_key(old_pass, &salt)?;
    verify_canary(&old_key, &canary_path)?;

    let mut new_key = derive_key(new_pass, &salt)?;

    // NOTE (H-02 + P0.3): the SQLCipher database is keyed by the
    // master-password-derived key (after the one-time P0.3 migration from the
    // legacy plaintext `vault.key`). Changing the password therefore must
    // (a) atomically swap the canary so the new password unwraps it, and
    // (b) rekey the database to the newly derived key. If the rekey fails the
    // canary is reverted, the database keeps its old key and the old password
    // still unlocks everything — no state where the two disagree.

    let migration_was_legacy =
        migrate_legacy_vault_key(&old_key).map_err(|e| {
            VaultError::Generic(format!("legacy vault.key migration failed: {e}"))
        })?;

    let new_encrypted = crate::secret::encrypt_bytes(&new_key, CANARY_PLAINTEXT)?;
    let tmp_path = data_dir.join(format!("{CANARY_FILE}.tmp"));
    std::fs::write(&tmp_path, &new_encrypted).map_err(|e| vault_err(e.to_string()))?;
    if let Err(e) = verify_canary(&new_key, &tmp_path) {
        let _ = std::fs::remove_file(&tmp_path);
        return Err(e);
    }
    if let Err(e) = std::fs::rename(&tmp_path, &canary_path) {
        let _ = std::fs::remove_file(&tmp_path);
        return Err(vault_err(e.to_string()));
    }

    if !migration_was_legacy {
        // Database is keyed by the old derived key — rekey it in place.
        if let Err(rekey_err) = rekey_db(&data_dir, &old_key, &new_key) {
            // Roll the canary back so the effective password stays `old_pass`.
            let old_encrypted = crate::secret::encrypt_bytes(&old_key, CANARY_PLAINTEXT)
                .map_err(|e| vault_err(e.to_string()))?;
            let revert_path = data_dir.join(format!("{CANARY_FILE}.tmp"));
            if std::fs::write(&revert_path, &old_encrypted).is_ok() {
                let _ = std::fs::rename(&revert_path, &canary_path);
            }
            let _ = crate::audit::log_event("VAULT_REKEY_FAILED", None, &rekey_err.to_string());
            return Err(rekey_err);
        }
    }

    {
        let mut guard = ACTIVE_VAULT_KEY.write();
        *guard = Some(new_key);
    }

    old_key.zeroize();

    let _ = crate::audit::log_event("VAULT_REKEY", None, "Master password changed");
    Ok(())
}

/// Rekeys the encrypted database from `old_key` to `new_key` in place. Fails
/// without side effects if the database cannot be opened with `old_key`.
fn rekey_db(
    data_dir: &std::path::Path,
    old_key: &[u8; 32],
    new_key: &[u8; 32],
) -> Result<(), CafError> {
    let old_hex = hex::encode(old_key);
    let new_hex = hex::encode(new_key);

    let conn = crate::db::open_encrypted(data_dir, &old_hex)?;
    let mut rekey_stmt = format!("PRAGMA rekey = \"x'{new_hex}'\";");
    conn.execute_batch(&rekey_stmt)
        .map_err(|e| CafError::Db(DbError::Generic(format!("failed to rekey database: {e}"))))?;
    conn.query_row("SELECT count(*) FROM sqlite_master", [], |_| Ok(()))
        .map_err(|e| {
            CafError::Db(DbError::Generic(format!(
                "rekey verification failed: {e}"
            )))
        })?;
    rekey_stmt.zeroize();
    Ok(())
}

/// One-time migration away from the legacy plaintext `vault.key` (T-1/P0.3).
///
/// Databases created before P0.3 were keyed by a random 64-hex key stored in
/// plaintext in `data_dir/vault.key`. On the first unlock with the master
/// password the database is rekeyed to the master-password-derived key and the
/// plaintext file is deleted. If anything fails the database is left untouched
/// and the error is surfaced — there is never a silent recovery (H-02) and the
/// file is only removed once the database demonstrably works with the new key.
/// Returns `true` if a legacy database was actually migrated.
fn migrate_legacy_vault_key(derived_key: &[u8; 32]) -> Result<bool, CafError> {
    let data_dir = crate::paths::data_dir()?;
    let legacy_path = data_dir.join("vault.key");
    if !legacy_path.exists() {
        return Ok(false);
    }

    let db_path = crate::paths::db_path(&data_dir);
    if !db_path.exists() {
        // Stale key file without a database: nothing to migrate, remove the leak.
        let _ = std::fs::remove_file(&legacy_path);
        return Ok(false);
    }

    let legacy_hex = std::fs::read_to_string(&legacy_path)
        .map_err(|e| vault_err(e.to_string()))?;
    let legacy_hex = legacy_hex.trim().to_string();

    let is_valid_legacy_hex = legacy_hex.len() == 64 && hex::decode(&legacy_hex).is_ok();
    if !is_valid_legacy_hex {
        // Never a valid key format — it cannot have keyed a database. Remove it.
        let _ = std::fs::remove_file(&legacy_path);
        return Ok(false);
    }

    let mut derived_hex = hex::encode(derived_key);
    let mut rekey_stmt = format!("PRAGMA rekey = \"x'{derived_hex}'\";");

    let conn = crate::db::open_keyed(&db_path, &format!("\"x'{legacy_hex}'\""))?;
    {
        let mut legacy_hex_copy = legacy_hex.clone();
        legacy_hex_copy.zeroize();
    }
    derived_hex.zeroize();

    let conn = crate::db::open_keyed(&db_path, &format!("\"x'{legacy_hex}'\""))?;
    conn.execute_batch(&rekey_stmt)
        .map_err(|e| {
            CafError::Db(DbError::Generic(format!(
                "failed to rekey legacy database: {e}"
            )))
        })?;

    // Only after the rekeyed database demonstrably works do we drop the file.
    conn.query_row("SELECT count(*) FROM sqlite_master", [], |_| Ok(()))
        .map_err(|e| {
            CafError::Db(DbError::Generic(format!(
                "rekey verification failed: {e}"
            )))
        })?;
    rekey_stmt.zeroize();

    let _ = std::fs::remove_file(&legacy_path);
    let _ = crate::audit::log_event(
        "VAULT_MIGRATE_KEY",
        None,
        "legacy vault.key removed after successful rekey",
    );
    Ok(true)
}

// The legacy `load_or_create_local_key` (plaintext `vault.key`) was removed
// as part of P0.3/T-1. Existing installs are migrated by
// `migrate_legacy_vault_key` on the first master-password unlock.

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

    #[test]
    fn p03_locked_vault_returns_cmrk_vault_001_and_never_creates_vault_key() {
        let dir = crate::test_support::isolated_data_dir("p03_locked_guard");

        assert!(!is_unlocked().expect("vault locked, so is_unlocked() == false"));

        let err = crate::db::open().expect_err("locked vault must fail");
        assert_eq!(err.code(), "CMRK-VAULT-001", "locked must map to VaultError::Locked");

        // Neither the plaintext key file nor the database may be created while locked.
        assert!(!dir.path.join("vault.key").exists(), "vault.key must never be created");
        assert!(!dir.path.join("camark.db").exists(), "db must not be created while locked");
    }

    #[test]
    fn p03_unlocked_vault_never_writes_plaintext_key_file() {
        let dir = crate::test_support::isolated_data_dir("p03_unlocked_guard");
        assert!(validate_password(OLD_PASS).expect("init + unlock"));
        let _ = crate::db::open().expect("db open while unlocked");

        assert!(!dir.path.join("vault.key").exists(), "vault.key must never be created");
    }

    #[test]
    fn p03_legacy_vault_key_migrated_on_first_unlock_with_data_intact() {
        let dir = crate::test_support::isolated_data_dir("p03_legacy_migration");

        // Simulate a pre-P0.3 install: DB keyed by the plaintext legacy key.
        let legacy_hex = "00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff";
        std::fs::write(dir.path.join("vault.key"), legacy_hex).expect("write legacy key");
        {
            let conn = crate::db::open_encrypted(&dir.path, legacy_hex).expect("legacy db");
            conn.execute("INSERT INTO app_kv (key, value) VALUES ('legacy', 'kept')", [])
                .expect("insert");
        }

        assert!(!crate::vault::is_vault_initialized().expect("no canary yet"));
        assert!(validate_password(OLD_PASS).expect("unlock migrates"));

        // Legacy file gone, data readable, and a lock/unlock cycle still works.
        assert!(!dir.path.join("vault.key").exists(), "vault.key must be removed after migration");

        assert_eq!(
            crate::db::open()
                .and_then(|c| {
                    c.query_row("SELECT value FROM app_kv WHERE key='legacy'", [], |r| r.get::<_, String>(0))
                        .map_err(|e| CafError::Db(crate::error::DbError::Generic(e.to_string())))
                })
                .expect("legible data after migration"),
            "kept"
        );

        lock();
        assert!(crate::db::open().expect_err("locked again").code() == "CMRK-VAULT-001");
        assert!(validate_password(OLD_PASS).expect("re-unlock"));
        assert!(crate::db::open().is_ok(), "db opens with master-derived key after migration");
    }

    #[test]
    fn p03_change_password_rekeys_master_derived_database() {
        let dir = crate::test_support::isolated_data_dir("p03_pw_change_rekey");
        assert!(validate_password(OLD_PASS).expect("init + unlock"));
        seed_rows();

        change_master_password(OLD_PASS, NEW_PASS).expect("change password");
        lock();

        assert!(validate_password(NEW_PASS).expect("new password unlocks rekeyed db"));
        assert_eq!(count_rows(), 3, "data survives rekey");
        assert!(!dir.path.join("vault.key").exists(), "no legacy key file involved");
    }
}
