import { invoke } from "@tauri-apps/api/core";
import type { Source, ContentItem, SearchResult, YoutubeExtractionResult } from "@/types/source";

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

export type LibraryItemType =
  | "bili_favorite_video"
  | "bili_watch_later_video"
  | "bili_followed_up";

export type LibraryCollectionType = "bili_favorite_folder";

export interface LibraryItem {
  source: string;
  externalId: string;
  itemType: LibraryItemType;
  title: string;
  author: string | null;
  url: string | null;
  imageUrl: string | null;
  rawMetadata: Record<string, unknown>;
}

export interface LibraryCollection {
  source: string;
  externalId: string;
  collectionType: LibraryCollectionType;
  title: string;
  rawMetadata: Record<string, unknown>;
}

export interface LibraryItemCollection {
  source: string;
  itemExternalId: string;
  itemType: LibraryItemType;
  collectionExternalId: string;
  collectionType: LibraryCollectionType;
  rawMetadata: Record<string, unknown>;
}

export interface BiliSyncScope {
  favorites: boolean;
  follows: boolean;
  watchLater: boolean;
}

export interface BiliSyncResult {
  items: LibraryItem[];
  collections: LibraryCollection[];
  itemCollections: LibraryItemCollection[];
  syncedFavorites: boolean;
  syncedFollows: boolean;
  syncedWatchLater: boolean;
}

export interface LlmProviderConfig {
  provider: string;
  baseUrl: string | null;
  apiKey: string | null;
  model: string;
}

export interface LlmSuggestedAction {
  kind: string;
  target: string | null;
}

export type LlmClassificationProvenance = "llm" | "local_metadata";

export type LlmClassificationMode = "llm" | "local_metadata";

export interface LlmItemAnalysis {
  externalId: string;
  category?: string | null;
  suggestedTags: string[];
  reason: string;
  confidence: number;
  suggestedAction: LlmSuggestedAction | null;
}

export interface LlmAnalysisResponse {
  items: LlmItemAnalysis[];
  source: "llm" | "metadata_fallback";
  warning: string | null;
}

export interface LlmAnalyzeItemsRequest {
  instruction: string;
  items: LibraryItem[];
  provider: LlmProviderConfig | null;
}

export interface LlmClassifyItemsRequest {
  instruction: string;
  items: LibraryItem[];
  provider: LlmProviderConfig | null;
  mode: LlmClassificationMode;
}

export interface LlmClassificationItem {
  externalId: string;
  category: string;
  suggestedTags: string[];
  reason: string;
  confidence: number;
  suggestedAction: LlmSuggestedAction | null;
  provenance: LlmClassificationProvenance;
  provider: string;
  model: string;
  analysisAt: string;
}

export interface LlmClassificationChunkFailure {
  chunkIndex: number;
  itemExternalIds: string[];
  error: string;
}

export interface LlmClassificationResponse {
  items: LlmClassificationItem[];
  chunkFailures: LlmClassificationChunkFailure[];
}

export type LlmProviderTestErrorKind =
  | "missing_config"
  | "auth"
  | "network"
  | "incompatible_response"
  | "invalid_json";

export interface LlmProviderTestError {
  kind: LlmProviderTestErrorKind;
  message: string;
}

export interface LlmProviderTestResult {
  ok: boolean;
  provider: string;
  model: string;
  usedResponseFormat: boolean;
}

export interface AudioExtractionCandidate {
  bvid: string;
  title: string;
  isMusic: boolean;
}

export type FavoriteOperationAction = "copy" | "move" | "delete";

export type OperationPlanItemStatus =
  | "pending"
  | "running"
  | "success"
  | "skipped"
  | "failed"
  | "blocked";

export interface FavoriteOperationCandidate {
  externalId: string;
  title: string;
  sourceCollectionExternalId: string | null;
  sourceCollectionTitle: string | null;
  collectionExternalIds: string[];
  resourceId: string | null;
  resourceType: string | null;
}

export interface FavoriteOperationPlanRequest {
  action: FavoriteOperationAction;
  targetCollectionExternalId: string | null;
  targetCollectionTitle: string | null;
  items: FavoriteOperationCandidate[];
}

export type FavoriteFolderPrivacy = "public" | "private";

export interface FavoriteFolderCreatePlanRequest {
  title: string;
  intro: string;
  privacy: FavoriteFolderPrivacy;
}

export type OperationPlanKind =
  | "bili_batch_audio_extraction"
  | "bili_batch_copy"
  | "bili_batch_move"
  | "bili_batch_delete"
  | "bili_favorite_folder_create";

export interface OperationPlanItem {
  externalId: string;
  title: string;
  action: string;
  target: string | null;
  status: OperationPlanItemStatus;
  error: string | null;
  sourceCollectionExternalId: string | null;
  sourceCollectionTitle: string | null;
  targetCollectionExternalId: string | null;
  targetCollectionTitle: string | null;
  resourceId: string | null;
  resourceType: string | null;
}

export interface OperationPlan {
  kind: OperationPlanKind;
  items: OperationPlanItem[];
}

export interface BiliAudioExtractionExecutionResult {
  results: {
    externalId: string;
    title: string;
    status: string;
    outputPaths: string[];
    error: string | null;
  }[];
}

export interface BiliFavoriteMoveExecutionResult {
  plan: OperationPlan;
  stopped: boolean;
}

export interface BiliFavoriteCopyExecutionResult {
  plan: OperationPlan;
  stopped: boolean;
}

export interface BiliFavoriteDeleteExecutionResult {
  plan: OperationPlan;
  stopped: boolean;
}

export interface BiliFavoriteFolderCreateExecutionResult {
  plan: OperationPlan;
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

  biliCookieLogin: (params: { sessdata: string }): Promise<BiliSession> =>
    invoke("bili_cookie_login", params),

  biliRestoreSession: (params: { session: BiliSession }): Promise<void> =>
    invoke("bili_restore_session", params),

  biliPasswordLogin: (params: { username: string; password: string }): Promise<BiliSession> =>
    invoke("bili_password_login", params),

  biliLogout: (): Promise<void> => invoke("bili_logout"),

  biliSessionStatus: (): Promise<{ loggedIn: boolean; username: string | null; isPremium: boolean }> =>
    invoke("bili_session_status"),

  biliExtractAudio: (params: { url: string }): Promise<ExtractionResult> =>
    invoke("bili_extract_audio", params),

  biliExtractSegment: (params: { bvid: string; cid: number; title: string }): Promise<ExtractionResult> =>
    invoke("bili_extract_segment", params),

  biliExtractCollection: (params: { mid: number; seasonId: number }): Promise<ExtractionResult> =>
    invoke("bili_extract_collection", params),

  biliCheckFfmpeg: (): Promise<boolean> => invoke("bili_check_ffmpeg"),

  biliGetPlayurl: (params: { bvid: string; cid: number }): Promise<{ audioUrl: string; quality: number; bandwidth: number }> =>
    invoke("bili_get_playurl", params),

  // Personal media workstation
  biliSyncLibrary: (params: { scope: BiliSyncScope }): Promise<BiliSyncResult> =>
    invoke("bili_sync_library", params),

  llmAnalyzeItems: (request: LlmAnalyzeItemsRequest): Promise<LlmAnalysisResponse> =>
    invoke("llm_analyze_items", { request }),

  llmClassifyItems: (request: LlmClassifyItemsRequest): Promise<LlmClassificationResponse> =>
    invoke("llm_classify_items", { request }),

  llmTestProvider: (provider: LlmProviderConfig): Promise<LlmProviderTestResult> =>
    invoke("llm_test_provider", { provider }),

  createBiliAudioExtractionPlan: (params: { candidates: AudioExtractionCandidate[] }): Promise<OperationPlan> =>
    invoke("create_bili_audio_extraction_plan", params),

  createBiliFavoriteOperationPlan: (request: FavoriteOperationPlanRequest): Promise<OperationPlan> =>
    invoke("create_bili_favorite_operation_plan", { request }),

  createBiliFavoriteFolderCreatePlan: (request: FavoriteFolderCreatePlanRequest): Promise<OperationPlan> =>
    invoke("create_bili_favorite_folder_create_plan", { request }),

  executeBiliAudioExtractionPlan: (params: { plan: OperationPlan }): Promise<BiliAudioExtractionExecutionResult> =>
    invoke("execute_bili_audio_extraction_plan", params),

  executeBiliFavoriteMovePlan: (params: { plan: OperationPlan; confirmed: boolean }): Promise<BiliFavoriteMoveExecutionResult> =>
    invoke("execute_bili_favorite_move_plan", params),

  executeBiliFavoriteCopyPlan: (params: { plan: OperationPlan; confirmed: boolean }): Promise<BiliFavoriteCopyExecutionResult> =>
    invoke("execute_bili_favorite_copy_plan", params),

  executeBiliFavoriteDeletePlan: (params: { plan: OperationPlan; confirmationText: string }): Promise<BiliFavoriteDeleteExecutionResult> =>
    invoke("execute_bili_favorite_delete_plan", params),

  executeBiliFavoriteFolderCreatePlan: (params: { plan: OperationPlan; confirmed: boolean }): Promise<BiliFavoriteFolderCreateExecutionResult> =>
    invoke("execute_bili_favorite_folder_create_plan", params),

  // YouTube
  youtubeExtractAudio: (params: { url: string }): Promise<YoutubeExtractionResult> =>
    invoke("youtube_extract_audio", params),

  youtubeSearch: (params: { query: string; limit?: number }): Promise<ContentItem[]> =>
    invoke("youtube_search", params),

  youtubeCheckReady: (): Promise<boolean> =>
    invoke("youtube_check_ready"),
};
