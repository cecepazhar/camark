-- 0001_baseline_schema.sql: True baseline schema for CAMark
CREATE TABLE IF NOT EXISTS app_kv (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS profiles (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    role TEXT NOT NULL DEFAULT 'owner',
    avatar TEXT,
    pin_hash TEXT,
    rev INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    deleted_at TEXT,
    origin_device_id TEXT NOT NULL,
    owner_profile_id TEXT NOT NULL,
    visibility TEXT NOT NULL DEFAULT 'shared'
);

CREATE TABLE IF NOT EXISTS notes (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    content TEXT NOT NULL DEFAULT '',
    tags TEXT NOT NULL DEFAULT '[]',
    rev INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    deleted_at TEXT,
    origin_device_id TEXT NOT NULL,
    owner_profile_id TEXT NOT NULL,
    visibility TEXT NOT NULL DEFAULT 'shared'
);

CREATE TABLE IF NOT EXISTS change_log (
    seq INTEGER PRIMARY KEY AUTOINCREMENT,
    entity_type TEXT NOT NULL,
    entity_id TEXT NOT NULL,
    rev INTEGER NOT NULL,
    action TEXT NOT NULL, -- 'create' | 'update' | 'delete'
    payload TEXT,
    synced_at TEXT,
    timestamp TEXT NOT NULL DEFAULT (datetime('now')),
    origin_device_id TEXT NOT NULL,
    owner_profile_id TEXT NOT NULL,
    visibility TEXT NOT NULL DEFAULT 'shared'
);
