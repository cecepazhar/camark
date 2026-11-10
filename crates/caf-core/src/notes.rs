//! Notes module — sample vertical slice demonstrating sync-ready conventions.

use crate::db;
use crate::error::{CafError, DbError};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteRecord {
    pub id: String,
    pub title: String,
    pub content: String,
    pub tags: Vec<String>,
    pub rev: i64,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
    pub origin_device_id: String,
    pub owner_profile_id: String,
    pub visibility: String, // 'shared' | 'private_summary' | 'private'
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteInput {
    pub id: Option<String>,
    pub title: String,
    pub content: String,
    pub tags: Vec<String>,
    pub visibility: Option<String>,
    pub owner_profile_id: String,
}

pub fn list_notes(caller_profile_id: &str, is_super: bool) -> Result<Vec<NoteRecord>, CafError> {
    let conn = db::open()?;
    let scope_clause = crate::visibility::sql_scope(is_super);
    let sql = format!(
        "SELECT id, title, content, tags, rev, created_at, updated_at, deleted_at, origin_device_id, owner_profile_id, visibility
         FROM notes
         WHERE {}
         ORDER BY updated_at DESC",
        scope_clause
    );

    let mut stmt = conn
        .prepare(&sql)
        .map_err(|e| CafError::Db(DbError::Generic(e.to_string())))?;

    let map_fn = |row: &rusqlite::Row| -> rusqlite::Result<NoteRecord> {
        let tags_str: String = row.get(3)?;
        let tags: Vec<String> = serde_json::from_str(&tags_str).unwrap_or_default();
        Ok(NoteRecord {
            id: row.get(0)?,
            title: row.get(1)?,
            content: row.get(2)?,
            tags,
            rev: row.get(4)?,
            created_at: row.get(5)?,
            updated_at: row.get(6)?,
            deleted_at: row.get(7)?,
            origin_device_id: row.get(8)?,
            owner_profile_id: row.get(9)?,
            visibility: row.get(10)?,
        })
    };

    let mut notes = Vec::new();
    if is_super {
        let rows = stmt
            .query_map([], map_fn)
            .map_err(|e| CafError::Db(DbError::Generic(e.to_string())))?;
        for row in rows {
            notes.push(row.map_err(|e| CafError::Db(DbError::Generic(e.to_string())))?);
        }
    } else {
        let rows = stmt
            .query_map([caller_profile_id], map_fn)
            .map_err(|e| CafError::Db(DbError::Generic(e.to_string())))?;
        for row in rows {
            notes.push(row.map_err(|e| CafError::Db(DbError::Generic(e.to_string())))?);
        }
    };

    Ok(notes)
}

/// Full-text search over notes using SQLite FTS5 index.
pub fn search_notes(
    query: &str,
    caller_profile_id: &str,
    is_super: bool,
) -> Result<Vec<NoteRecord>, CafError> {
    let clean_query = query.trim();
    if clean_query.is_empty() {
        return list_notes(caller_profile_id, is_super);
    }

    let conn = db::open()?;

    let terms: Vec<String> = clean_query
        .split_whitespace()
        .map(|term| {
            let sanitized = term.replace('"', "\"\"");
            format!("\"{sanitized}\"*")
        })
        .collect();

    if terms.is_empty() {
        return list_notes(caller_profile_id, is_super);
    }
    let fts_query = terms.join(" ");

    let map_fn = |row: &rusqlite::Row| -> rusqlite::Result<NoteRecord> {
        let tags_str: String = row.get(3)?;
        let tags: Vec<String> = serde_json::from_str(&tags_str).unwrap_or_default();
        Ok(NoteRecord {
            id: row.get(0)?,
            title: row.get(1)?,
            content: row.get(2)?,
            tags,
            rev: row.get(4)?,
            created_at: row.get(5)?,
            updated_at: row.get(6)?,
            deleted_at: row.get(7)?,
            origin_device_id: row.get(8)?,
            owner_profile_id: row.get(9)?,
            visibility: row.get(10)?,
        })
    };

    let mut notes = Vec::new();
    if is_super {
        let sql = "SELECT n.id, n.title, n.content, n.tags, n.rev, n.created_at, n.updated_at, n.deleted_at, n.origin_device_id, n.owner_profile_id, n.visibility
                   FROM notes n
                   JOIN notes_fts ON notes_fts.note_id = n.id
                   WHERE notes_fts MATCH ?1 AND n.deleted_at IS NULL
                   ORDER BY bm25(notes_fts) ASC";
        let mut stmt = conn
            .prepare(sql)
            .map_err(|e| CafError::Db(DbError::Generic(e.to_string())))?;
        let rows = stmt
            .query_map([&fts_query], map_fn)
            .map_err(|e| CafError::Db(DbError::Generic(e.to_string())))?;
        for row in rows {
            notes.push(row.map_err(|e| CafError::Db(DbError::Generic(e.to_string())))?);
        }
    } else {
        let sql = "SELECT n.id, n.title, n.content, n.tags, n.rev, n.created_at, n.updated_at, n.deleted_at, n.origin_device_id, n.owner_profile_id, n.visibility
                   FROM notes n
                   JOIN notes_fts ON notes_fts.note_id = n.id
                   WHERE notes_fts MATCH ?1 AND n.deleted_at IS NULL AND (n.owner_profile_id = ?2 OR n.visibility = 'shared')
                   ORDER BY bm25(notes_fts) ASC";
        let mut stmt = conn
            .prepare(sql)
            .map_err(|e| CafError::Db(DbError::Generic(e.to_string())))?;
        let rows = stmt
            .query_map([&fts_query, caller_profile_id], map_fn)
            .map_err(|e| CafError::Db(DbError::Generic(e.to_string())))?;
        for row in rows {
            notes.push(row.map_err(|e| CafError::Db(DbError::Generic(e.to_string())))?);
        }
    }

    Ok(notes)
}

pub fn save_note(input: NoteInput, caller_profile_id: &str) -> Result<NoteRecord, CafError> {
    let mut conn = db::open()?;
    let tx = conn
        .transaction()
        .map_err(|e| CafError::Db(DbError::Generic(e.to_string())))?;

    let now = Utc::now().to_rfc3339();
    let device_id = crate::paths::device_id().unwrap_or_else(|_| "device-local".into());
    let visibility = input.visibility.unwrap_or_else(|| "shared".into());
    let tags_json = serde_json::to_string(&input.tags).unwrap_or_else(|_| "[]".into());

    let (id, rev, created_at, action) = if let Some(existing_id) = input.id {
        // Update existing note
        let existing: (i64, String, String, String) = tx
            .query_row(
                "SELECT rev, created_at, owner_profile_id, visibility FROM notes WHERE id = ?1 AND deleted_at IS NULL",
                [&existing_id],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
            )
            .map_err(|e| CafError::Db(DbError::Generic(format!("Note not found: {e}"))))?;

        let new_rev = existing.0 + 1;
        tx.execute(
            "UPDATE notes SET title = ?1, content = ?2, tags = ?3, rev = ?4, updated_at = ?5, visibility = ?6 WHERE id = ?7",
            rusqlite::params![input.title, input.content, tags_json, new_rev, now, visibility, existing_id],
        )
        .map_err(|e| CafError::Db(DbError::Generic(e.to_string())))?;

        (existing_id, new_rev, existing.1, "update")
    } else {
        // Create new note with UUIDv7
        let new_id = Uuid::now_v7().to_string();
        let rev = 1;
        tx.execute(
            "INSERT INTO notes (id, title, content, tags, rev, created_at, updated_at, origin_device_id, owner_profile_id, visibility)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            rusqlite::params![
                new_id,
                input.title,
                input.content,
                tags_json,
                rev,
                now,
                now,
                device_id,
                caller_profile_id,
                visibility
            ],
        )
        .map_err(|e| CafError::Db(DbError::Generic(e.to_string())))?;

        (new_id, rev, now.clone(), "create")
    };

    // Append to change_log in the exact same transaction
    let payload = serde_json::json!({
        "id": id,
        "title": input.title,
        "content": input.content,
        "tags": input.tags,
        "visibility": visibility,
    })
    .to_string();

    tx.execute(
        "INSERT INTO change_log (entity_type, entity_id, rev, action, payload, origin_device_id, owner_profile_id, visibility, timestamp)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        rusqlite::params![
            "note",
            id,
            rev,
            action,
            payload,
            device_id,
            caller_profile_id,
            visibility,
            now
        ],
    )
    .map_err(|e| CafError::Db(DbError::Generic(e.to_string())))?;

    tx.commit()
        .map_err(|e| CafError::Db(DbError::Generic(e.to_string())))?;

    Ok(NoteRecord {
        id,
        title: input.title,
        content: input.content,
        tags: input.tags,
        rev,
        created_at,
        updated_at: now,
        deleted_at: None,
        origin_device_id: device_id,
        owner_profile_id: caller_profile_id.to_string(),
        visibility,
    })
}

pub fn delete_note(id: &str, caller_profile_id: &str) -> Result<(), CafError> {
    let mut conn = db::open()?;
    let tx = conn
        .transaction()
        .map_err(|e| CafError::Db(DbError::Generic(e.to_string())))?;

    let now = Utc::now().to_rfc3339();
    let device_id = crate::paths::device_id().unwrap_or_else(|_| "device-local".into());

    let (rev, visibility): (i64, String) = tx
        .query_row(
            "SELECT rev, visibility FROM notes WHERE id = ?1 AND deleted_at IS NULL",
            [id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .map_err(|e| CafError::Db(DbError::Generic(format!("Note not found: {e}"))))?;

    let new_rev = rev + 1;
    tx.execute(
        "UPDATE notes SET deleted_at = ?1, rev = ?2, updated_at = ?3 WHERE id = ?4",
        rusqlite::params![now, new_rev, now, id],
    )
    .map_err(|e| CafError::Db(DbError::Generic(e.to_string())))?;

    tx.execute(
        "INSERT INTO change_log (entity_type, entity_id, rev, action, payload, origin_device_id, owner_profile_id, visibility, timestamp)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        rusqlite::params![
            "note",
            id,
            new_rev,
            "delete",
            "{}",
            device_id,
            caller_profile_id,
            visibility,
            now
        ],
    )
    .map_err(|e| CafError::Db(DbError::Generic(e.to_string())))?;

    tx.commit()
        .map_err(|e| CafError::Db(DbError::Generic(e.to_string())))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_note_lifecycle_and_changelog() {
        let _guard = crate::test_support::isolated_data_dir("notes_lifecycle_test");
        crate::vault::validate_password("test-password-p0-3").expect("unlock vault");

        // 1. Create note
        let created = save_note(
            NoteInput {
                id: None,
                title: "Belajar Rust CAMark".into(),
                content: "Starter template berbasis CATerm v2".into(),
                tags: vec!["rust".into(), "svelte".into()],
                visibility: Some("shared".into()),
                owner_profile_id: "user-1".into(),
            },
            "user-1",
        )
        .expect("save note");

        assert_eq!(created.rev, 1);
        assert_eq!(created.title, "Belajar Rust CAMark");

        // 2. List note
        let list = list_notes("user-1", false).expect("list notes");
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].id, created.id);

        // 3. Update note
        let updated = save_note(
            NoteInput {
                id: Some(created.id.clone()),
                title: "Belajar Rust CAMark - Updated".into(),
                content: "Starter template modern".into(),
                tags: vec!["rust".into(), "tauri".into()],
                visibility: Some("shared".into()),
                owner_profile_id: "user-1".into(),
            },
            "user-1",
        )
        .expect("update note");

        assert_eq!(updated.rev, 2);
        assert_eq!(updated.title, "Belajar Rust CAMark - Updated");

        // 4. Delete note
        delete_note(&created.id, "user-1").expect("delete note");
        let list_after = list_notes("user-1", false).expect("list after delete");
        assert!(list_after.is_empty());
    }

    #[test]
    fn test_notes_fts5_search_and_triggers() {
        let _guard = crate::test_support::isolated_data_dir("notes_fts5_test");
        crate::vault::validate_password("test-password-p0-3").expect("unlock vault");

        // 1. Insert notes with different contents
        let note1 = save_note(
            NoteInput {
                id: None,
                title: "Rust Systems Architecture".into(),
                content: "High performance database engine with SQLite FTS5 index".into(),
                tags: vec!["rust".into(), "database".into()],
                visibility: Some("shared".into()),
                owner_profile_id: "user-fts".into(),
            },
            "user-fts",
        )
        .expect("save note 1");

        let _note2 = save_note(
            NoteInput {
                id: None,
                title: "Frontend UI Svelte".into(),
                content: "Tailwind CSS and reactive components for desktop shell".into(),
                tags: vec!["svelte".into(), "ui".into()],
                visibility: Some("shared".into()),
                owner_profile_id: "user-fts".into(),
            },
            "user-fts",
        )
        .expect("save note 2");

        // 2. FTS5 search match title & content
        let results = search_notes("SQLite FTS5", "user-fts", false).expect("search notes");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, note1.id);

        let results_arch = search_notes("Architecture", "user-fts", false).expect("search notes");
        assert_eq!(results_arch.len(), 1);
        assert_eq!(results_arch[0].id, note1.id);

        // 3. FTS5 search non-matching term
        let no_results =
            search_notes("Kubernetes", "user-fts", false).expect("search non-matching");
        assert!(no_results.is_empty());

        // 4. Update note and verify FTS5 reflects change
        save_note(
            NoteInput {
                id: Some(note1.id.clone()),
                title: "Rust Systems Architecture - Distributed".into(),
                content: "Replaced content with Raft consensus protocol".into(),
                tags: vec!["rust".into(), "consensus".into()],
                visibility: Some("shared".into()),
                owner_profile_id: "user-fts".into(),
            },
            "user-fts",
        )
        .expect("update note 1");

        let old_search = search_notes("SQLite", "user-fts", false).expect("search old keyword");
        assert!(
            old_search.is_empty(),
            "Old content should be deleted from FTS5 index"
        );

        let new_search = search_notes("Raft", "user-fts", false).expect("search new keyword");
        assert_eq!(new_search.len(), 1);
        assert_eq!(new_search[0].id, note1.id);

        // 5. Delete note and verify FTS5 removed
        delete_note(&note1.id, "user-fts").expect("delete note");
        let search_after_del = search_notes("Raft", "user-fts", false).expect("search deleted");
        assert!(
            search_after_del.is_empty(),
            "Deleted note must not appear in FTS5 search"
        );
    }
}
