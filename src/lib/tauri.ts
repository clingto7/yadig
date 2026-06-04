import { invoke } from "@tauri-apps/api/core";
import type { Source, ContentItem, SearchResult } from "@/types/source";

export interface BiliSession {
  sessdata: string;
  biliJct: string;
  dedeUserId: string;
  vipStatus: number;
}

export interface AudioSegment {
  title: string;
  filePath: string;
  duration: number;
  quality: number;
  audioUrl: string;
}

export interface ExtractionResult {
  videoTitle: string;
  segments: AudioSegment[];
  extractionType: "Single" | "MultiPart" | "Chapters" | "Collection";
}

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

  downloadAudio: (params: { url: string; filename: string }): Promise<string> =>
    invoke("download_audio", params),

  openUrl: (params: { url: string }): Promise<void> =>
    invoke("open_url", params),

  // Bilibili auth
  biliQrLoginStart: (): Promise<{ url: string; qrcodeKey: string }> =>
    invoke("bili_qr_login_start"),

  biliQrLoginPoll: (params: { qrcodeKey: string }): Promise<{ code: number; message: string; session: BiliSession | null }> =>
    invoke("bili_qr_login_poll", params),

  biliCookieLogin: (params: { sessdata: string }): Promise<void> =>
    invoke("bili_cookie_login", params),

  biliLogout: (): Promise<void> => invoke("bili_logout"),

  biliSessionStatus: (): Promise<{ loggedIn: boolean; username: string | null; isPremium: boolean }> =>
    invoke("bili_session_status"),

  biliExtractAudio: (params: { url: string }): Promise<ExtractionResult> =>
    invoke("bili_extract_audio", params),

  biliGetPlayurl: (params: { bvid: string; cid: number }): Promise<{ audioUrl: string; quality: number; bandwidth: number }> =>
    invoke("bili_get_playurl", params),
};
