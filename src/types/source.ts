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
  audioUrl?: string;
  downloadUrl?: string;
  duration?: number;
  license?: string;
  extra?: Record<string, unknown>;
  relevanceScore?: number;
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

// YouTube extraction types
export interface YoutubeAudioSegment {
  title: string;
  filePath: string;
  duration: number;
  audioUrl: string;
  ext: string;
}

export interface YoutubeExtractionResult {
  videoTitle: string;
  videoUrl: string;
  thumbnailUrl: string | null;
  duration: number;
  segments: YoutubeAudioSegment[];
  hasChapters: boolean;
}
