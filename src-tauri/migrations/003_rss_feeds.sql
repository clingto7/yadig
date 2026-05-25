-- RSS feed sources
CREATE TABLE IF NOT EXISTS rss_feeds (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    name            TEXT NOT NULL,
    url             TEXT NOT NULL UNIQUE,
    site_url        TEXT,
    description     TEXT,
    is_active       INTEGER NOT NULL DEFAULT 1,
    refresh_interval_minutes INTEGER NOT NULL DEFAULT 60,
    last_fetched_at TEXT,
    last_error      TEXT,
    created_at      TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Cached articles from feeds
CREATE TABLE IF NOT EXISTS articles (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    feed_id      INTEGER NOT NULL REFERENCES rss_feeds(id) ON DELETE CASCADE,
    title        TEXT NOT NULL,
    url          TEXT NOT NULL UNIQUE,
    author       TEXT,
    summary      TEXT,
    llm_summary  TEXT,
    content_hash TEXT,
    image_url    TEXT,
    published_at TEXT,
    fetched_at   TEXT NOT NULL DEFAULT (datetime('now')),
    is_read      INTEGER NOT NULL DEFAULT 0,
    is_starred   INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX IF NOT EXISTS idx_articles_feed_id ON articles(feed_id);
CREATE INDEX IF NOT EXISTS idx_articles_published ON articles(published_at DESC);
CREATE INDEX IF NOT EXISTS idx_articles_content_hash ON articles(content_hash);
CREATE INDEX IF NOT EXISTS idx_articles_is_read ON articles(is_read);
