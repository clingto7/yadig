import type { LibraryItem, LlmClassificationItem } from "@/lib/tauri";
import {
  listLatestLlmClassifications,
  saveLlmClassifications,
} from "@/lib/db";

const item: LibraryItem = {
  source: "bilibili",
  externalId: "BV1contract",
  itemType: "bili_favorite_video",
  title: "Contract video",
  author: "Contract UP",
  url: "https://www.bilibili.com/video/BV1contract",
  imageUrl: null,
  rawMetadata: {
    tname: "音乐",
    resourceId: "resource-id",
    resourceType: "2",
  },
};

const classification: LlmClassificationItem = {
  externalId: "BV1contract",
  category: "music",
  suggestedTags: ["音乐"],
  reason: "The title and Bilibili category indicate music content.",
  confidence: 0.91,
  suggestedAction: {
    kind: "copy",
    target: "Music",
  },
  provenance: "llm",
  provider: "openai-compatible",
  model: "mimo-v2.5-pro",
  analysisAt: "2026-06-11T00:00:00Z",
};

export const llmClassificationContract = {
  itemId: classification.externalId,
  category: classification.category,
  provenance: classification.provenance,
  provider: classification.provider,
  save: () => saveLlmClassifications([item], [classification]),
  list: () => listLatestLlmClassifications(),
};
