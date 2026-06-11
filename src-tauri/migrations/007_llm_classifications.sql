CREATE TABLE IF NOT EXISTS llm_classifications (
    id                       INTEGER PRIMARY KEY AUTOINCREMENT,
    item_id                  INTEGER NOT NULL REFERENCES library_items(id) ON DELETE CASCADE,
    category                 TEXT NOT NULL,
    suggested_tags_json      TEXT NOT NULL DEFAULT '[]',
    reason                   TEXT NOT NULL,
    confidence               REAL NOT NULL,
    suggested_action_kind    TEXT,
    suggested_action_target  TEXT,
    provenance               TEXT NOT NULL,
    provider                 TEXT NOT NULL,
    model                    TEXT NOT NULL,
    analysis_at              TEXT NOT NULL,
    created_at               TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_llm_classifications_item_id
ON llm_classifications(item_id);

CREATE INDEX IF NOT EXISTS idx_llm_classifications_category
ON llm_classifications(category);

CREATE INDEX IF NOT EXISTS idx_llm_classifications_suggested_action
ON llm_classifications(suggested_action_kind);

CREATE INDEX IF NOT EXISTS idx_llm_classifications_provenance
ON llm_classifications(provenance);
