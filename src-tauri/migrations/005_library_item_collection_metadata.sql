ALTER TABLE library_item_collections
ADD COLUMN raw_metadata TEXT NOT NULL DEFAULT '{}';

ALTER TABLE library_item_collections
ADD COLUMN last_synced_at TEXT;

ALTER TABLE library_item_collections
ADD COLUMN updated_at TEXT;

UPDATE library_item_collections
SET last_synced_at = datetime('now'),
    updated_at = datetime('now')
WHERE last_synced_at IS NULL OR updated_at IS NULL;
