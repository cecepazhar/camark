//! Encrypted SQLite storage engine for CAMark. Every record persisted lives inside a
//! single SQLCipher-encrypted file at `<data_dir>/camark.db` — see [`crate::vault`].

use crate::error::{CafError, DbError};
use rusqlite::Connection;
use std::path::Path;
use zeroize::Zeroize;

/// Opens (creating if absent) the SQLCipher-encrypted database at `<data_dir>/camark.db`,
/// keys it with `passphrase`, and ensures the schema exists.
pub fn open_encrypted(data_dir: &Path, passphrase: &str) -> Result<Connection, CafError> {
    std::fs::create_dir_all(data_dir)
        .map_err(|e| CafError::Db(DbError::Generic(format!("failed to create data dir: {e}"))))?;
    let db_path = crate::paths::db_path(data_dir);

    let mut raw_key = raw_key_literal(passphrase);
    let try_open = |path: &Path, raw_key: &Option<String>| -> Result<Connection, CafError> {
        let mut conn = if let Some(raw) = raw_key.as_deref() {
            match open_keyed(path, raw) {
                Ok(conn) => conn,
                Err(_) => {
                    let mut passphrase_literal = passphrase_literal(passphrase);
                    let conn = open_keyed(path, &passphrase_literal)?;
                    conn.execute_batch(&format!("PRAGMA rekey = {raw};"))
                        .map_err(|e| {
                            CafError::Db(DbError::Generic(format!(
                                "failed to migrate database to raw-key mode: {e}"
                            )))
                        })?;
                    passphrase_literal.zeroize();
                    conn
                }
            }
        } else {
            let mut literal = passphrase_literal(passphrase);
            let conn = open_keyed(path, &literal)?;
            literal.zeroize();
            conn
        };

        init_schema(&mut conn)?;
        Ok(conn)
    };

    // H-02: no silent recovery. A wrong key / corrupted DB is reported as an error;
    // we never rename the database, delete WAL/SHM, or create a fresh empty DB.
    let opened = try_open(&db_path, &raw_key);
    if let Some(raw) = raw_key.as_mut() {
        raw.zeroize();
    }
    opened
}

fn passphrase_literal(passphrase: &str) -> String {
    format!("'{}'", passphrase.replace('\'', "''"))
}

fn raw_key_literal(passphrase: &str) -> Option<String> {
    if passphrase.len() == 64 && passphrase.bytes().all(|b| b.is_ascii_hexdigit()) {
        Some(format!("\"x'{passphrase}'\""))
    } else {
        None
    }
}

/// Opens `db_path` with a SQLCipher key literal and verifies the key decrypts
/// the file. Also used by the one-time legacy `vault.key` migration in
/// [`crate::vault`].
pub(crate) fn open_keyed(db_path: &Path, key_literal: &str) -> Result<Connection, CafError> {
    let conn = Connection::open(db_path)
        .map_err(|e| CafError::Db(DbError::Generic(format!("failed to open database: {e}"))))?;

    conn.execute_batch(&format!(
        "PRAGMA key = {key_literal};\nPRAGMA journal_mode = WAL;\nPRAGMA synchronous = NORMAL;\nPRAGMA foreign_keys = ON;"
    ))
    .map_err(|e| CafError::Db(DbError::Generic(format!("failed to set vault key: {e}"))))?;

    conn.query_row("SELECT count(*) FROM sqlite_master", [], |_| Ok(()))
        .map_err(|_| {
            CafError::Db(DbError::Generic(
                "invalid vault key or corrupted database".into(),
            ))
        })?;

    Ok(conn)
}

pub fn init_schema(conn: &mut Connection) -> Result<(), CafError> {
    crate::migrations::run_migrations(conn)
}

/// Convenience helper to open the encrypted DB with the current vault key.
pub fn open() -> Result<Connection, CafError> {
    let data_dir = crate::paths::data_dir()?;
    let key = crate::vault::ensure_unlocked_key()?;
    open_encrypted(&data_dir, &key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_open_encrypted_roundtrip() {
        let temp_dir = std::env::temp_dir().join(format!("caf_db_test_{}", uuid::Uuid::new_v4()));
        let key = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
        let conn = open_encrypted(&temp_dir, key).expect("open encrypted db");

        conn.execute(
            "INSERT INTO app_kv (key, value) VALUES (?1, ?2)",
            ["test_k", "test_v"],
        )
        .unwrap();
        let val: String = conn
            .query_row("SELECT value FROM app_kv WHERE key = ?1", ["test_k"], |r| {
                r.get(0)
            })
            .unwrap();
        assert_eq!(val, "test_v");

        drop(conn);
        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn h02_wrong_key_errors_without_touching_files() {
        let dir = std::env::temp_dir().join(format!("caf_db_h02_{}", uuid::Uuid::new_v4()));
        let key_a = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
        let key_b = "fedcba9876543210fedcba9876543210fedcba9876543210fedcba9876543210";
        let conn = open_encrypted(&dir, key_a).expect("create");
        conn.execute("INSERT INTO app_kv (key, value) VALUES ('k', 'v')", [])
            .expect("insert");
        drop(conn);

        let db_file = crate::paths::db_path(&dir);
        let before = std::fs::read(&db_file).expect("read db");

        assert!(open_encrypted(&dir, key_b).is_err(), "wrong key must error");

        let after = std::fs::read(&db_file).expect("db still there");
        assert_eq!(before, after, "database bytes must be unchanged");
        let baks = std::fs::read_dir(&dir)
            .expect("read dir")
            .filter_map(|e| e.ok())
            .filter(|e| e.file_name().to_string_lossy().contains(".bak"))
            .count();
        assert_eq!(baks, 0, "no .bak file may be created");

        // Original key still works and data is intact.
        let conn = open_encrypted(&dir, key_a).expect("reopen with right key");
        let v: String = conn
            .query_row("SELECT value FROM app_kv WHERE key='k'", [], |r| r.get(0))
            .expect("row");
        assert_eq!(v, "v");
        drop(conn);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
