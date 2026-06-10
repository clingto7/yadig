CREATE TABLE IF NOT EXISTS library_items (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    source          TEXT NOT NULL,
    external_id     TEXT NOT NULL,
    item_type       TEXT NOT NULL,
    title           TEXT NOT NULL,
    author          TEXT,
    url             TEXT,
    image_url       TEXT,
    raw_metadata    TEXT NOT NULL DEFAULT '{}',
    last_synced_at  TEXT NOT NULL DEFAULT (datetime('now')),
    created_at      TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at      TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(source, external_id, item_type)
);

CREATE INDEX IF NOT EXISTS idx_library_items_source ON library_items(source);
CREATE INDEX IF NOT EXISTS idx_library_items_item_type ON library_items(item_type);
CREATE INDEX IF NOT EXISTS idx_library_items_title ON library_items(title);

CREATE TABLE IF NOT EXISTS library_collections (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    source          TEXT NOT NULL,
    external_id     TEXT NOT NULL,
    collection_type TEXT NOT NULL,
    title           TEXT NOT NULL,
    raw_metadata    TEXT NOT NULL DEFAULT '{}',
    last_synced_at  TEXT NOT NULL DEFAULT (datetime('now')),
    created_at      TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at      TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(source, external_id, collection_type)
);

CREATE INDEX IF NOT EXISTS idx_library_collections_source ON library_collections(source);
CREATE INDEX IF NOT EXISTS idx_library_collections_type ON library_collections(collection_type);

CREATE TABLE IF NOT EXISTS library_item_collections (
    item_id       INTEGER NOT NULL REFERENCES library_items(id) ON DELETE CASCADE,
    collection_id INTEGER NOT NULL REFERENCES library_collections(id) ON DELETE CASCADE,
    created_at    TEXT NOT NULL DEFAULT (datetime('now')),
    PRIMARY KEY (item_id, collection_id)
);

CREATE TABLE IF NOT EXISTS library_tags (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    name       TEXT NOT NULL UNIQUE,
    source     TEXT NOT NULL DEFAULT 'user',
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS library_item_tags (
    item_id    INTEGER NOT NULL REFERENCES library_items(id) ON DELETE CASCADE,
    tag_id     INTEGER NOT NULL REFERENCES library_tags(id) ON DELETE CASCADE,
    confidence REAL,
    reason     TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    PRIMARY KEY (item_id, tag_id)
);

CREATE TABLE IF NOT EXISTS llm_analyses (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    item_id       INTEGER REFERENCES library_items(id) ON DELETE CASCADE,
    provider      TEXT NOT NULL,
    model         TEXT NOT NULL,
    instruction   TEXT NOT NULL,
    response_json TEXT NOT NULL,
    created_at    TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS operation_plans (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    kind       TEXT NOT NULL,
    status     TEXT NOT NULL DEFAULT 'draft',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS operation_plan_items (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    plan_id       INTEGER NOT NULL REFERENCES operation_plans(id) ON DELETE CASCADE,
    external_id   TEXT NOT NULL,
    title         TEXT NOT NULL,
    action        TEXT NOT NULL,
    target        TEXT,
    status        TEXT NOT NULL DEFAULT 'pending',
    error         TEXT,
    created_at    TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at    TEXT NOT NULL DEFAULT (datetime('now'))
);
