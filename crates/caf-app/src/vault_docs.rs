//! SQLCipher Encrypted Vault Documents storage for CAMark (P4).

use caf_core::db;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultDocument {
    pub id: String,
    pub title: String,
    pub content: String,
    pub tags: Vec<String>,
    pub rev: i64,
    pub created_at: String,
    pub updated_at: String,
    pub owner_profile_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultDocInput {
    #[serde(default)]
    pub id: Option<String>,
    pub title: String,
    pub content: String,
    #[serde(default)]
    pub tags: Vec<String>,
}

pub async fn list_documents(caller_profile_id: String) -> Result<Vec<VaultDocument>, String> {
    tokio::task::spawn_blocking(move || list_documents_sync(&caller_profile_id))
        .await
        .map_err(|e| e.to_string())?
}

fn list_documents_sync(caller_profile_id: &str) -> Result<Vec<VaultDocument>, String> {
    let conn = db::open().map_err(|e| e.to_string())?;

    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS vault_documents (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            content TEXT NOT NULL,
            tags TEXT NOT NULL DEFAULT '[]',
            rev INTEGER NOT NULL DEFAULT 1,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now')),
            owner_profile_id TEXT NOT NULL DEFAULT 'default'
        );",
    )
    .map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare("SELECT id, title, content, tags, rev, created_at, updated_at, owner_profile_id FROM vault_documents WHERE owner_profile_id = ?1 ORDER BY updated_at DESC")
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map([caller_profile_id], |row| {
            let tags_str: String = row.get(3)?;
            let tags: Vec<String> = serde_json::from_str(&tags_str).unwrap_or_default();
            Ok(VaultDocument {
                id: row.get(0)?,
                title: row.get(1)?,
                content: row.get(2)?,
                tags,
                rev: row.get(4)?,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
                owner_profile_id: row.get(7)?,
            })
        })
        .map_err(|e| e.to_string())?;

    let mut docs = Vec::new();
    for r in rows {
        docs.push(r.map_err(|e| e.to_string())?);
    }
    Ok(docs)
}

pub async fn save_document(
    input: VaultDocInput,
    caller_profile_id: String,
) -> Result<VaultDocument, String> {
    tokio::task::spawn_blocking(move || save_document_sync(input, &caller_profile_id))
        .await
        .map_err(|e| e.to_string())?
}

fn save_document_sync(
    input: VaultDocInput,
    caller_profile_id: &str,
) -> Result<VaultDocument, String> {
    let conn = db::open().map_err(|e| e.to_string())?;
    let doc_id = input.id.unwrap_or_else(|| {
        format!(
            "doc_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis()
        )
    });
    let tags_json = serde_json::to_string(&input.tags).unwrap_or_else(|_| "[]".to_string());

    conn.execute(
        "INSERT INTO vault_documents (id, title, content, tags, rev, owner_profile_id, updated_at) 
         VALUES (?1, ?2, ?3, ?4, 1, ?5, datetime('now'))
         ON CONFLICT(id) DO UPDATE SET 
            title = excluded.title,
            content = excluded.content,
            tags = excluded.tags,
            rev = vault_documents.rev + 1,
            updated_at = datetime('now');",
        [
            &doc_id,
            &input.title,
            &input.content,
            &tags_json,
            caller_profile_id,
        ],
    )
    .map_err(|e| e.to_string())?;

    Ok(VaultDocument {
        id: doc_id,
        title: input.title,
        content: input.content,
        tags: input.tags,
        rev: 1,
        created_at: "now".to_string(),
        updated_at: "now".to_string(),
        owner_profile_id: caller_profile_id.to_string(),
    })
}

pub async fn delete_document(id: String) -> Result<(), String> {
    tokio::task::spawn_blocking(move || {
        let conn = db::open().map_err(|e| e.to_string())?;
        conn.execute("DELETE FROM vault_documents WHERE id = ?1", [id])
            .map_err(|e| e.to_string())?;
        Ok(())
    })
    .await
    .map_err(|e| e.to_string())?
}
