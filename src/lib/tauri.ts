import { invoke } from "@tauri-apps/api/core";
import type { Source, ContentItem, SearchResult } from "@/types/source";

export const tauri = {
  searchSources: (params: {
    query: string;
    sourceIds?: string[];
    limit?: number;
  }): Promise<SearchResult> => invoke("search_sources", params),

  fetchLatest: (params: {
    sourceIds?: string[];
    limit?: number;
  }): Promise<ContentItem[]> => invoke("fetch_latest", params),

  listSources: (): Promise<Source[]> => invoke("list_sources"),

  addFeed: (params: { name: string; url: string }): Promise<void> =>
    invoke("add_feed", params),

  removeFeed: (params: { id: number }): Promise<void> =>
    invoke("remove_feed", params),

  listFeeds: (): Promise<unknown[]> => invoke("list_feeds"),

  refreshFeeds: (): Promise<void> => invoke("refresh_feeds"),

  getArticles: (params: {
    feedId?: number;
    limit?: number;
    offset?: number;
  }): Promise<unknown[]> => invoke("get_articles", params),

  saveSearch: (params: {
    query: string;
    resultCount: number;
    sources: string;
  }): Promise<void> => invoke("save_search", params),

  listSearches: (params?: { limit?: number }): Promise<unknown[]> =>
    invoke("list_searches", params),

  clearHistory: (): Promise<void> => invoke("clear_history"),
};
