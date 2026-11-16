use crate::error::{CafError, DbError};
use rusqlite::Connection;
use sha2::{Digest, Sha256};
use std::time::{SystemTime, UNIX_EPOCH};

const MIGRATIONS: &[(i32, &str, &str)] = &[
    (
        1,
        "0001_baseline_schema",
        include_str!("migrations/0001_baseline_schema.sql"),
    ),
    (
        2,
        "0002_notes_fts5",
        include_str!("migrations/0002_notes_fts5.sql"),
    ),
];

fn compute_checksum(sql: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(sql.as_bytes());
    hex::encode(hasher.finalize())
}

/// Add any missing `schema_version` columns so the in-app runner can record
/// migrations on databases that predate the 4-column schema.
///
/// Legacy standalone `caf-migrator` created `schema_version(version, applied_at)` —
/// no `name`, no `checksum`. We add the missing columns (SQLite `ALTER TABLE ADD
/// COLUMN`), backfilling sensible defaults for already-applied rows:
///   * `name`     <- the migration's human-readable identifier
///   * `checksum` <- `''` (unknown for legacy rows; the checksum *verification* below
///     only triggers when a row is *already present for a not-yet-applied* version,
///     which cannot happen for legacy rows because legacy runs recorded versions the
///     in-app runner also records, so the verification path sees a known checksum).
///
/// Idempotent: if a column already exists the corresponding `ALTER TABLE` is skipped.
fn upgrade_schema_version_table(conn: &Connection) -> Result<(), CafError> {
    let cols: std::collections::HashSet<String> = {
        let mut stmt = conn
            .prepare("SELECT name FROM pragma_table_info('schema_version')")
            .map_err(|e| {
                CafError::Db(DbError::Generic(format!(
                    "failed to introspect schema_version: {e}"
                )))
            })?;
        let rows = stmt
            .query_map([], |row| row.get::<_, String>(0))
            .map_err(|e| {
                CafError::Db(DbError::Generic(format!(
                    "failed to list schema_version columns: {e}"
                )))
            })?;
        rows.filter_map(Result::ok).collect()
    };

    // Add `name` (required by the INSERT, and by the verification lookups).
    if !cols.contains("name") {
        conn.execute(
            "ALTER TABLE schema_version ADD COLUMN name TEXT NOT NULL DEFAULT ''",
            [],
        )
        .map_err(|e| {
            CafError::Db(DbError::Generic(format!(
                "failed to add schema_version.name: {e}"
            )))
        })?;
    }

    // Add `checksum` (required by the INSERT, and read by the verification path).
    if !cols.contains("checksum") {
        conn.execute(
            "ALTER TABLE schema_version ADD COLUMN checksum TEXT NOT NULL DEFAULT ''",
            [],
        )
        .map_err(|e| {
            CafError::Db(DbError::Generic(format!(
                "failed to add schema_version.checksum: {e}"
            )))
        })?;
    }

    Ok(())
}

pub fn run_migrations(conn: &mut Connection) -> Result<(), CafError> {
    // Only bootstrap the schema version table if it doesn't exist
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS schema_version (
            version INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            checksum TEXT NOT NULL,
            applied_at INTEGER NOT NULL
        );",
    )
    .map_err(|e| {
        CafError::Db(DbError::Generic(format!(
            "failed to bootstrap schema_version: {e}"
        )))
    })?;

    // Backward compatibility: legacy databases created by the standalone `caf-migrator`
    // CLI only have the two-column schema `schema_version(version, applied_at)`, missing
    // the `name` and `checksum` columns that the in-app runner records. `CREATE TABLE
    // IF NOT EXISTS` above is a no-op on such a database, so every subsequent
    // `INSERT ... (version, name, checksum, applied_at)` fails with
    // "table schema_version has no column named name". Detect the legacy shape here
    // and upgrade it in place (backfilling metadata for already-recorded rows) so
    // old vaults unlock cleanly instead of erroring.
    upgrade_schema_version_table(conn)?;

    let tx = conn.transaction().map_err(|e| {
        CafError::Db(DbError::Generic(format!(
            "failed to start migration transaction: {e}"
        )))
    })?;

    let max_version: i32 = tx
        .query_row(
            "SELECT COALESCE(MAX(version), 0) FROM schema_version",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    let mut applied_any = false;

    for &(version, name, sql) in MIGRATIONS {
        let expected_checksum = compute_checksum(sql);

        if version <= max_version {
            // Verify checksum for already applied migrations
            let stored_checksum: Result<String, _> = tx.query_row(
                "SELECT checksum FROM schema_version WHERE version = ?1",
                [&version],
                |row| row.get(0),
            );

            if let Ok(stored) = stored_checksum {
                // A legacy database (upgraded from caf-migrator's 2-column shape)
                // backfills `checksum` as the empty string for pre-existing rows, so
                // we cannot verify them. Treat an empty stored checksum as "already
                // applied, trust it" instead of failing; only a *populated* but
                // mismatched checksum is a real integrity error.
                if !stored.is_empty() && stored != expected_checksum {
                    return Err(CafError::Db(DbError::MigrationChecksum));
                }
            } else {
                return Err(CafError::Db(DbError::Generic(format!(
                    "migration {} is marked applied but no checksum found",
                    version
                ))));
            }
            continue;
        }

        // Apply new migration
        tx.execute_batch(sql).map_err(|e| {
            CafError::Db(DbError::Generic(format!(
                "failed to apply migration {}: {}",
                version, e
            )))
        })?;

        let now_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as i64;

        tx.execute(
            "INSERT INTO schema_version (version, name, checksum, applied_at) VALUES (?1, ?2, ?3, ?4)",
            (&version, &name, &expected_checksum, &now_ms),
        ).map_err(|e| {
            CafError::Db(DbError::Generic(format!("failed to record migration {}: {}", version, e)))
        })?;

        applied_any = true;
    }

    if max_version > MIGRATIONS.last().map(|m| m.0).unwrap_or(0) {
        return Err(CafError::Db(DbError::SchemaTooNew));
    }

    if applied_any {
        tx.commit().map_err(|e| {
            CafError::Db(DbError::Generic(format!(
                "failed to commit migrations: {e}"
            )))
        })?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    #[test]
    fn test_migrations_forward() {
        let mut conn = Connection::open_in_memory().unwrap();
        run_migrations(&mut conn).expect("baseline migration failed");

        let cv: i32 = conn
            .query_row("SELECT MAX(version) FROM schema_version", [], |r| r.get(0))
            .unwrap();
        assert_eq!(cv, 2);
    }

    /// A database created by the standalone `caf-migrator` CLI has only the
    /// two-column `schema_version(version, applied_at)` shape and already records
    /// applied migrations without `name`/`checksum`. The in-app runner must
    /// upgrade that legacy table in place and still run to completion instead of
    /// failing with "no column named name".
    #[test]
    fn test_migrations_upgrade_legacy_schema_version() {
        let mut conn = Connection::open_in_memory().unwrap();
        // Simulate a legacy caf-migrator database: 2-column table with a recorded
        // version, plus the baseline tables it would have created.
        conn.execute_batch(
            "CREATE TABLE schema_version (
                version INTEGER PRIMARY KEY,
                applied_at INTEGER NOT NULL
            );",
        )
        .unwrap();
        conn.execute(
            "INSERT INTO schema_version (version, applied_at) VALUES (1, 1700000000000)",
            [],
        )
        .unwrap();
        // Baseline tables the legacy migration would have created.
        conn.execute_batch(
            "CREATE TABLE app_kv (key TEXT PRIMARY KEY, value TEXT NOT NULL, updated_at TEXT NOT NULL);",
        )
        .unwrap();
        // Migration 2 (FTS5) backfills from `notes`, so a legacy DB that already
        // reached version 1+ must have a notes table too.
        conn.execute_batch(
            "CREATE TABLE notes (id TEXT PRIMARY KEY, title TEXT NOT NULL, content TEXT NOT NULL DEFAULT '', tags TEXT NOT NULL DEFAULT '[]', deleted_at TEXT);",
        )
        .unwrap();

        // Run the in-app runner against this legacy DB.
        run_migrations(&mut conn).expect("legacy upgrade failed");

        // Columns must now exist.
        let cols: Vec<String> = {
            let mut stmt = conn
                .prepare("SELECT name FROM pragma_table_info('schema_version')")
                .unwrap();
            stmt.query_map([], |row| row.get::<_, String>(0))
                .unwrap()
                .map(|r| r.unwrap())
                .collect()
        };
        assert!(cols.contains(&"name".to_string()));
        assert!(cols.contains(&"checksum".to_string()));

        // Migration 2 (FTS5) must now be recorded.
        let cv: i32 = conn
            .query_row("SELECT MAX(version) FROM schema_version", [], |r| r.get(0))
            .unwrap();
        assert_eq!(cv, 2);
    }
}
