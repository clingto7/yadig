import type { LibraryItem } from "@/lib/tauri";
import {
  LLM_CLASSIFICATION_CHUNK_SIZE,
  buildClassificationProgress,
  chunkClassificationItems,
  formatClassificationProgress,
} from "@/lib/llm-classification-progress";

const items = Array.from({ length: 17 }, (_, index): LibraryItem => ({
  source: "bilibili",
  externalId: `BV${index}`,
  itemType: "bili_favorite_video",
  title: `Contract video ${index}`,
  author: null,
  url: null,
  imageUrl: null,
  rawMetadata: {},
}));

const chunks = chunkClassificationItems(items);
const progress = buildClassificationProgress({
  mode: "llm",
  stage: "requesting",
  currentChunk: 2,
  totalChunks: chunks.length,
  processedItems: chunks[0]?.length ?? 0,
  totalItems: items.length,
  savedItems: chunks[0]?.length ?? 0,
  failedChunks: 0,
});

export const llmClassificationProgressContract = {
  chunkSize: LLM_CLASSIFICATION_CHUNK_SIZE,
  chunkCount: chunks.length,
  lastChunkSize: chunks[chunks.length - 1]?.length ?? 0,
  progress,
  label: formatClassificationProgress(progress),
};
