-- Favorites: user-saved music entities
CREATE TABLE IF NOT EXISTS favorites (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    entity_type TEXT NOT NULL CHECK (entity_type IN ('artist', 'album', 'track', 'label')),
    entity_id   TEXT NOT NULL,
    source      TEXT NOT NULL CHECK (source IN ('spotify', 'lastfm', 'discogs', 'musicbrainz')),
    name        TEXT NOT NULL,
    image_url   TEXT,
    metadata    TEXT,
    created_at  TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(entity_type, entity_id, source)
);

CREATE INDEX IF NOT EXISTS idx_favorites_entity_type ON favorites(entity_type);
CREATE INDEX IF NOT EXISTS idx_favorites_source ON favorites(source);
