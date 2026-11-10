-- 0002_notes_fts5.sql: SQLite FTS5 Full-Text Search index and synchronization triggers
CREATE VIRTUAL TABLE IF NOT EXISTS notes_fts USING fts5(
    note_id UNINDEXED,
    title,
    content,
    tags,
    tokenize = 'porter unicode61'
);

-- Backfill existing active notes into FTS5 index
INSERT INTO notes_fts (note_id, title, content, tags)
SELECT id, title, COALESCE(content, ''), COALESCE(tags, '')
FROM notes
WHERE deleted_at IS NULL;

-- Triggers to synchronize notes_fts with notes table
CREATE TRIGGER IF NOT EXISTS trg_notes_fts_insert AFTER INSERT ON notes
WHEN new.deleted_at IS NULL
BEGIN
    INSERT INTO notes_fts (note_id, title, content, tags)
    VALUES (new.id, new.title, COALESCE(new.content, ''), COALESCE(new.tags, ''));
END;

CREATE TRIGGER IF NOT EXISTS trg_notes_fts_update AFTER UPDATE ON notes
BEGIN
    DELETE FROM notes_fts WHERE note_id = old.id;
    INSERT INTO notes_fts (note_id, title, content, tags)
    SELECT new.id, new.title, COALESCE(new.content, ''), COALESCE(new.tags, '')
    WHERE new.deleted_at IS NULL;
END;

CREATE TRIGGER IF NOT EXISTS trg_notes_fts_delete AFTER DELETE ON notes
BEGIN
    DELETE FROM notes_fts WHERE note_id = old.id;
END;
