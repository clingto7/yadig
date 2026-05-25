import { invoke } from "@tauri-apps/api/core";
import type { Source, ContentItem, SearchResult } from "@/types/source";

export const tauri = {
  searchSources: (params: {
    query: string;
    sourceIds?: string[];
    limit?: number;
    page?: number;
  }): Promise<SearchResult> => invoke("search_sources", params),

  fetchLatest: (params: {
    sourceIds?: string[];
    limit?: number;
  }): Promise<ContentItem[]> => invoke("fetch_latest", params),

  listSources: (): Promise<Source[]> => invoke("list_sources"),

  updateDiscogsKeys: (params: { key: string; secret: string }): Promise<void> =>
    invoke("update_discogs_keys", params),

  setSourceEnabled: (params: { id: string; enabled: boolean }): Promise<void> =>
    invoke("set_source_enabled", params),
};
