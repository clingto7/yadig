export type SourceKind = "rss" | "api" | "scraper";

export interface Source {
  id: string;
  name: string;
  kind: SourceKind;
  baseUrl: string;
  isActive: boolean;
}

export interface ContentItem {
  sourceId: string;
  title: string;
  url: string;
  summary?: string;
  author?: string;
  publishedAt?: string;
  imageUrl?: string;
  extra?: Record<string, unknown>;
}

export interface SearchPage {
  page: number;
  hasMore: boolean;
}

export interface SearchResult {
  query: string;
  items: ContentItem[];
  totalResults: number;
  elapsedMs: number;
  page: SearchPage;
}
