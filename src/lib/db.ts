import Database from "@tauri-apps/plugin-sql";

let db: Database | null = null;

async function getDb(): Promise<Database> {
  if (!db) {
    db = await Database.load("sqlite:yadig.db");
  }
  return db;
}

// --- RSS Feeds ---

export interface RssFeed {
  id: number;
  name: string;
  url: string;
  is_active: number;
  refresh_interval_minutes: number;
  last_fetched_at: string | null;
  created_at: string;
}

export async function addFeed(name: string, url: string): Promise<void> {
  const d = await getDb();
  await d.execute("INSERT INTO rss_feeds (name, url) VALUES ($1, $2)", [name, url]);
}

export async function removeFeed(id: number): Promise<void> {
  const d = await getDb();
  await d.execute("DELETE FROM rss_feeds WHERE id = $1", [id]);
}

export async function listFeeds(): Promise<RssFeed[]> {
  const d = await getDb();
  return d.select<RssFeed[]>(
    "SELECT id, name, url, is_active, refresh_interval_minutes, last_fetched_at, created_at FROM rss_feeds ORDER BY created_at DESC",
    []
  );
}

export async function toggleFeed(id: number, isActive: boolean): Promise<void> {
  const d = await getDb();
  await d.execute("UPDATE rss_feeds SET is_active = $1 WHERE id = $2", [
    isActive ? 1 : 0,
    id,
  ]);
}

// --- Favorites ---

export interface Favorite {
  id: number;
  entity_type: string;
  entity_id: string;
  source: string;
  name: string;
  image_url: string | null;
  metadata: string | null;
  created_at: string;
}

export async function addFavorite(
  entityType: string,
  entityId: string,
  source: string,
  name: string,
  imageUrl?: string,
  metadata?: string
): Promise<void> {
  const d = await getDb();
  await d.execute(
    "INSERT INTO favorites (entity_type, entity_id, source, name, image_url, metadata) VALUES ($1, $2, $3, $4, $5, $6)",
    [entityType, entityId, source, name, imageUrl ?? null, metadata ?? null]
  );
}

export async function removeFavorite(id: number): Promise<void> {
  const d = await getDb();
  await d.execute("DELETE FROM favorites WHERE id = $1", [id]);
}

export async function listFavorites(entityType?: string): Promise<Favorite[]> {
  const d = await getDb();
  if (entityType) {
    return d.select<Favorite[]>(
      "SELECT id, entity_type, entity_id, source, name, image_url, metadata, created_at FROM favorites WHERE entity_type = $1 ORDER BY created_at DESC",
      [entityType]
    );
  }
  return d.select<Favorite[]>(
    "SELECT id, entity_type, entity_id, source, name, image_url, metadata, created_at FROM favorites ORDER BY created_at DESC",
    []
  );
}

export async function isFavorite(entityId: string, source: string): Promise<boolean> {
  const d = await getDb();
  const rows = await d.select<{ cnt: number }[]>(
    "SELECT COUNT(*) as cnt FROM favorites WHERE entity_id = $1 AND source = $2",
    [entityId, source]
  );
  return rows.length > 0 && rows[0].cnt > 0;
}

// --- Search History ---

export interface SearchHistoryEntry {
  id: number;
  query: string;
  result_count: number | null;
  sources: string | null;
  created_at: string;
}

export async function saveSearch(
  query: string,
  resultCount: number,
  sources: string
): Promise<void> {
  const d = await getDb();
  await d.execute(
    "INSERT INTO search_history (query, result_count, sources) VALUES ($1, $2, $3)",
    [query, resultCount, sources]
  );
}

export async function listSearches(limit = 20): Promise<SearchHistoryEntry[]> {
  const d = await getDb();
  return d.select<SearchHistoryEntry[]>(
    "SELECT id, query, result_count, sources, created_at FROM search_history ORDER BY created_at DESC LIMIT $1",
    [limit]
  );
}

export async function clearHistory(): Promise<void> {
  const d = await getDb();
  await d.execute("DELETE FROM search_history", []);
}
