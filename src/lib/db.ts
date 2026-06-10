import Database from "@tauri-apps/plugin-sql";
import type {
  BiliSyncResult,
  BiliSyncScope,
  FavoriteOperationCandidate,
  LibraryCollection,
  LibraryCollectionType,
  LibraryItem,
  LlmItemAnalysis,
  LlmProviderConfig,
  OperationPlan,
} from "@/lib/tauri";

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

// --- Personal Media Library ---

type LibraryItemRow = {
  id?: number;
  source: string;
  external_id: string;
  item_type: LibraryItem["itemType"];
  title: string;
  author: string | null;
  url: string | null;
  image_url: string | null;
  raw_metadata: string;
};

type LibraryCollectionRow = {
  source: string;
  external_id: string;
  collection_type: LibraryCollectionType;
  title: string;
  raw_metadata: string;
};

type FavoriteOperationCandidateRow = {
  external_id: string;
  title: string;
  source_collection_external_id: string;
  source_collection_title: string;
  raw_metadata: string;
};

function parseRawMetadata(rawMetadata: string): Record<string, unknown> {
  try {
    const parsed = JSON.parse(rawMetadata);
    return parsed && typeof parsed === "object" && !Array.isArray(parsed)
      ? parsed as Record<string, unknown>
      : {};
  } catch {
    return {};
  }
}

function mapLibraryItem(row: LibraryItemRow): LibraryItem {
  return {
    source: row.source,
    externalId: row.external_id,
    itemType: row.item_type,
    title: row.title,
    author: row.author,
    url: row.url,
    imageUrl: row.image_url,
    rawMetadata: parseRawMetadata(row.raw_metadata),
  };
}

function mapLibraryCollection(row: LibraryCollectionRow): LibraryCollection {
  return {
    source: row.source,
    externalId: row.external_id,
    collectionType: row.collection_type,
    title: row.title,
    rawMetadata: parseRawMetadata(row.raw_metadata),
  };
}

function rawMetadataString(metadata: Record<string, unknown>, key: string): string | null {
  const value = metadata[key];
  if (typeof value === "string") return value;
  if (typeof value === "number") return String(value);
  return null;
}

export type LibraryItemWithCollections = LibraryItem & {
  collections: LibraryCollection[];
};

export async function upsertLibraryItems(items: LibraryItem[], syncedScope?: BiliSyncScope): Promise<void> {
  const d = await getDb();
  for (const item of items) {
    await d.execute(
      `INSERT INTO library_items
        (source, external_id, item_type, title, author, url, image_url, raw_metadata, last_synced_at, updated_at)
       VALUES ($1, $2, $3, $4, $5, $6, $7, $8, datetime('now'), datetime('now'))
       ON CONFLICT(source, external_id, item_type) DO UPDATE SET
        title = excluded.title,
        author = excluded.author,
        url = excluded.url,
        image_url = excluded.image_url,
        raw_metadata = excluded.raw_metadata,
        last_synced_at = datetime('now'),
        updated_at = datetime('now')`,
      [
        item.source,
        item.externalId,
        item.itemType,
        item.title,
        item.author,
        item.url,
        item.imageUrl,
        JSON.stringify(item.rawMetadata ?? {}),
      ]
    );
  }

  if (!syncedScope) return;

  const syncedTypes: LibraryItem["itemType"][] = [];
  if (syncedScope.favorites) syncedTypes.push("bili_favorite_video");
  if (syncedScope.watchLater) syncedTypes.push("bili_watch_later_video");
  if (syncedScope.follows) syncedTypes.push("bili_followed_up");

  for (const itemType of syncedTypes) {
    const currentIds = items
      .filter((item) => item.source === "bilibili" && item.itemType === itemType)
      .map((item) => item.externalId);

    if (currentIds.length === 0) {
      await d.execute(
        "DELETE FROM library_items WHERE source = 'bilibili' AND item_type = $1",
        [itemType]
      );
      continue;
    }

    const placeholders = currentIds.map((_, index) => `$${index + 2}`).join(", ");
    await d.execute(
      `DELETE FROM library_items
       WHERE source = 'bilibili'
         AND item_type = $1
         AND external_id NOT IN (${placeholders})`,
      [itemType, ...currentIds]
    );
  }
}

async function upsertLibraryCollections(collections: LibraryCollection[]): Promise<void> {
  const d = await getDb();
  for (const collection of collections) {
    await d.execute(
      `INSERT INTO library_collections
        (source, external_id, collection_type, title, raw_metadata, last_synced_at, updated_at)
       VALUES ($1, $2, $3, $4, $5, datetime('now'), datetime('now'))
       ON CONFLICT(source, external_id, collection_type) DO UPDATE SET
        title = excluded.title,
        raw_metadata = excluded.raw_metadata,
        last_synced_at = datetime('now'),
        updated_at = datetime('now')`,
      [
        collection.source,
        collection.externalId,
        collection.collectionType,
        collection.title,
        JSON.stringify(collection.rawMetadata ?? {}),
      ]
    );
  }
}

async function upsertLibraryItemCollections(syncResult: BiliSyncResult): Promise<void> {
  const d = await getDb();
  for (const membership of syncResult.itemCollections) {
    const itemRows = await d.select<{ id: number }[]>(
      `SELECT id FROM library_items
       WHERE source = $1 AND external_id = $2 AND item_type = $3
       LIMIT 1`,
      [membership.source, membership.itemExternalId, membership.itemType]
    );
    const collectionRows = await d.select<{ id: number }[]>(
      `SELECT id FROM library_collections
       WHERE source = $1 AND external_id = $2 AND collection_type = $3
       LIMIT 1`,
      [membership.source, membership.collectionExternalId, membership.collectionType]
    );
    const itemId = itemRows[0]?.id;
    const collectionId = collectionRows[0]?.id;
    if (!itemId || !collectionId) continue;

    await d.execute(
      `INSERT INTO library_item_collections
        (item_id, collection_id, raw_metadata, last_synced_at, updated_at)
       VALUES ($1, $2, $3, datetime('now'), datetime('now'))
       ON CONFLICT(item_id, collection_id) DO UPDATE SET
        raw_metadata = excluded.raw_metadata,
        last_synced_at = datetime('now'),
        updated_at = datetime('now')`,
      [itemId, collectionId, JSON.stringify(membership.rawMetadata ?? {})]
    );
  }
}

async function pruneBiliFavoriteCollections(syncResult: BiliSyncResult): Promise<void> {
  if (!syncResult.syncedFavorites) return;

  const d = await getDb();
  const collectionIds = syncResult.collections
    .filter((collection) => collection.source === "bilibili" && collection.collectionType === "bili_favorite_folder")
    .map((collection) => collection.externalId);
  const membershipKeys = new Set(
    syncResult.itemCollections.map(
      (membership) => `${membership.itemExternalId}\u0000${membership.collectionExternalId}`
    )
  );

  const existingMemberships = await d.select<{
    item_id: number;
    collection_id: number;
    item_external_id: string;
    collection_external_id: string;
  }[]>(
    `SELECT lic.item_id,
            lic.collection_id,
            li.external_id AS item_external_id,
            lc.external_id AS collection_external_id
     FROM library_item_collections lic
     INNER JOIN library_items li ON li.id = lic.item_id
     INNER JOIN library_collections lc ON lc.id = lic.collection_id
     WHERE li.source = 'bilibili'
       AND li.item_type = 'bili_favorite_video'
       AND lc.source = 'bilibili'
       AND lc.collection_type = 'bili_favorite_folder'`,
    []
  );

  for (const row of existingMemberships) {
    if (!membershipKeys.has(`${row.item_external_id}\u0000${row.collection_external_id}`)) {
      await d.execute(
        `DELETE FROM library_item_collections
         WHERE item_id = $1 AND collection_id = $2`,
        [row.item_id, row.collection_id]
      );
    }
  }

  if (collectionIds.length === 0) {
    await d.execute(
      "DELETE FROM library_collections WHERE source = 'bilibili' AND collection_type = 'bili_favorite_folder'",
      []
    );
    return;
  }

  const placeholders = collectionIds.map((_, index) => `$${index + 1}`).join(", ");
  await d.execute(
    `DELETE FROM library_collections
     WHERE source = 'bilibili'
       AND collection_type = 'bili_favorite_folder'
       AND external_id NOT IN (${placeholders})`,
    collectionIds
  );
}

export async function upsertBiliSyncResult(
  syncResult: BiliSyncResult,
  syncedScope?: BiliSyncScope
): Promise<void> {
  await upsertLibraryItems(syncResult.items, syncedScope);
  await upsertLibraryCollections(syncResult.collections);
  await upsertLibraryItemCollections(syncResult);
  await pruneBiliFavoriteCollections(syncResult);
}

export async function listLibraryItems(): Promise<LibraryItem[]> {
  const d = await getDb();
  const rows = await d.select<LibraryItemRow[]>(
    `SELECT source, external_id, item_type, title, author, url, image_url, raw_metadata
     FROM library_items
     ORDER BY last_synced_at DESC, updated_at DESC`,
    []
  );
  return rows.map(mapLibraryItem);
}

export async function listLibraryCollections(
  collectionType?: LibraryCollectionType
): Promise<LibraryCollection[]> {
  const d = await getDb();
  const rows = collectionType
    ? await d.select<LibraryCollectionRow[]>(
        `SELECT source, external_id, collection_type, title, raw_metadata
         FROM library_collections
         WHERE collection_type = $1
         ORDER BY title COLLATE NOCASE ASC`,
        [collectionType]
      )
    : await d.select<LibraryCollectionRow[]>(
        `SELECT source, external_id, collection_type, title, raw_metadata
         FROM library_collections
         ORDER BY collection_type ASC, title COLLATE NOCASE ASC`,
        []
      );
  return rows.map(mapLibraryCollection);
}

export async function listLibraryItemsWithCollections(): Promise<LibraryItemWithCollections[]> {
  const d = await getDb();
  const itemRows = await d.select<(LibraryItemRow & { id: number })[]>(
    `SELECT id, source, external_id, item_type, title, author, url, image_url, raw_metadata
     FROM library_items
     ORDER BY last_synced_at DESC, updated_at DESC`,
    []
  );

  const items = itemRows.map((row) => ({
    ...mapLibraryItem(row),
    collections: [] as LibraryCollection[],
  }));
  const itemById = new Map(itemRows.map((row, index) => [row.id, items[index]]));

  const collectionRows = await d.select<(LibraryCollectionRow & { item_id: number })[]>(
    `SELECT lic.item_id,
            lc.source,
            lc.external_id,
            lc.collection_type,
            lc.title,
            lc.raw_metadata
     FROM library_item_collections lic
     INNER JOIN library_collections lc ON lc.id = lic.collection_id
     INNER JOIN library_items li ON li.id = lic.item_id
     WHERE li.source = 'bilibili'
       AND li.item_type = 'bili_favorite_video'
       AND lc.source = 'bilibili'
       AND lc.collection_type = 'bili_favorite_folder'
     ORDER BY lc.title COLLATE NOCASE ASC`,
    []
  );

  for (const row of collectionRows) {
    itemById.get(row.item_id)?.collections.push(mapLibraryCollection(row));
  }

  return items;
}

export async function listFavoriteOperationCandidates(
  sourceCollectionExternalId: string
): Promise<FavoriteOperationCandidate[]> {
  const d = await getDb();
  const rows = await d.select<FavoriteOperationCandidateRow[]>(
    `SELECT li.external_id,
            li.title,
            lc.external_id AS source_collection_external_id,
            lc.title AS source_collection_title,
            lic.raw_metadata
     FROM library_item_collections lic
     INNER JOIN library_items li ON li.id = lic.item_id
     INNER JOIN library_collections lc ON lc.id = lic.collection_id
     WHERE li.source = 'bilibili'
       AND li.item_type = 'bili_favorite_video'
       AND lc.source = 'bilibili'
       AND lc.collection_type = 'bili_favorite_folder'
       AND lc.external_id = $1
     ORDER BY li.title COLLATE NOCASE ASC`,
    [sourceCollectionExternalId]
  );

  return rows.map((row) => {
    const rawMetadata = parseRawMetadata(row.raw_metadata);
    return {
      externalId: row.external_id,
      title: row.title,
      sourceCollectionExternalId: row.source_collection_external_id,
      sourceCollectionTitle: row.source_collection_title,
      resourceId: rawMetadataString(rawMetadata, "resourceId"),
      resourceType: rawMetadataString(rawMetadata, "resourceType"),
    };
  });
}

export async function listLatestLlmAnalyses(): Promise<LlmItemAnalysis[]> {
  const d = await getDb();
  const rows = await d.select<{ response_json: string }[]>(
    `SELECT la.response_json
     FROM llm_analyses la
     INNER JOIN (
       SELECT item_id, MAX(id) AS latest_id
       FROM llm_analyses
       GROUP BY item_id
     ) latest ON latest.latest_id = la.id
     ORDER BY la.created_at DESC`,
    []
  );

  return rows.flatMap((row) => {
    try {
      const parsed = JSON.parse(row.response_json) as LlmItemAnalysis;
      return parsed.externalId ? [parsed] : [];
    } catch {
      return [];
    }
  });
}

export async function saveLlmAnalysis(
  instruction: string,
  provider: LlmProviderConfig | null,
  items: LibraryItem[],
  analyses: LlmItemAnalysis[]
): Promise<void> {
  if (analyses.length === 0) return;

  const d = await getDb();
  for (const analysis of analyses) {
    const matchingItems = items.filter((candidate) => candidate.externalId === analysis.externalId);
    if (matchingItems.length === 0) {
      continue;
    }

    for (const item of matchingItems) {
      const rows = await d.select<{ id: number }[]>(
        `SELECT id FROM library_items
         WHERE source = $1 AND external_id = $2 AND item_type = $3
         LIMIT 1`,
        [item.source, item.externalId, item.itemType]
      );
      const itemId = rows[0]?.id;
      if (!itemId) continue;

      await d.execute(
        `INSERT INTO llm_analyses (item_id, provider, model, instruction, response_json)
         VALUES ($1, $2, $3, $4, $5)`,
        [
          itemId,
          provider?.provider ?? "metadata-fallback",
          provider?.model ?? "metadata-fallback",
          instruction,
          JSON.stringify(analysis),
        ]
      );

      for (const tag of analysis.suggestedTags) {
        const trimmed = tag.trim();
        if (!trimmed) continue;

        await d.execute(
          "INSERT OR IGNORE INTO library_tags (name, source) VALUES ($1, 'llm')",
          [trimmed]
        );
        const tagRows = await d.select<{ id: number }[]>(
          "SELECT id FROM library_tags WHERE name = $1 LIMIT 1",
          [trimmed]
        );
        const tagId = tagRows[0]?.id;
        if (!tagId) continue;

        await d.execute(
          `INSERT INTO library_item_tags (item_id, tag_id, confidence, reason)
           VALUES ($1, $2, $3, $4)
           ON CONFLICT(item_id, tag_id) DO UPDATE SET
            confidence = excluded.confidence,
            reason = excluded.reason`,
          [itemId, tagId, analysis.confidence, analysis.reason]
        );
      }
    }
  }
}

export async function saveOperationPlan(plan: OperationPlan): Promise<void> {
  const d = await getDb();
  const result = await d.execute("INSERT INTO operation_plans (kind, status) VALUES ($1, 'draft')", [
    plan.kind,
  ]);
  const planId = result.lastInsertId;
  if (!planId) return;

  for (const item of plan.items) {
    await d.execute(
      `INSERT INTO operation_plan_items
        (plan_id, external_id, title, action, target, status, error,
         source_collection_external_id, source_collection_title,
         target_collection_external_id, target_collection_title,
         resource_id, resource_type)
       VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)`,
      [
        planId,
        item.externalId,
        item.title,
        item.action,
        item.target,
        item.status ?? "pending",
        item.error ?? null,
        item.sourceCollectionExternalId ?? null,
        item.sourceCollectionTitle ?? null,
        item.targetCollectionExternalId ?? null,
        item.targetCollectionTitle ?? null,
        item.resourceId ?? null,
        item.resourceType ?? null,
      ]
    );
  }
}
