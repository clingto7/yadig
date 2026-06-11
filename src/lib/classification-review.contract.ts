import type { LibraryCollection, LibraryItem, LlmClassificationItem } from "@/lib/tauri";
import {
  DEFAULT_CLASSIFICATION_REVIEW_FILTERS,
  filterClassificationReviewItems,
  selectFilteredFavoriteIds,
} from "@/lib/classification-review";

type ReviewItem = LibraryItem & { collections: LibraryCollection[] };

const musicFolder: LibraryCollection = {
  source: "bilibili",
  externalId: "folder-music",
  collectionType: "bili_favorite_folder",
  title: "Music",
  rawMetadata: {},
};

const techFolder: LibraryCollection = {
  source: "bilibili",
  externalId: "folder-tech",
  collectionType: "bili_favorite_folder",
  title: "Tech",
  rawMetadata: {},
};

const favoriteItems: ReviewItem[] = [
  {
    source: "bilibili",
    externalId: "BVmusic",
    itemType: "bili_favorite_video",
    title: "Live music session",
    author: "Singer UP",
    url: null,
    imageUrl: null,
    rawMetadata: { tname: "音乐" },
    collections: [musicFolder],
  },
  {
    source: "bilibili",
    externalId: "BVtech",
    itemType: "bili_favorite_video",
    title: "Rust ownership",
    author: "Code UP",
    url: null,
    imageUrl: null,
    rawMetadata: { tname: "知识" },
    collections: [techFolder],
  },
  {
    source: "bilibili",
    externalId: "BVlater",
    itemType: "bili_watch_later_video",
    title: "Music watch later",
    author: "Singer UP",
    url: null,
    imageUrl: null,
    rawMetadata: { tname: "音乐" },
    collections: [],
  },
];

const classifications: LlmClassificationItem[] = [
  {
    externalId: "BVmusic",
    category: "music",
    suggestedTags: ["音乐", "live"],
    reason: "Music category and title.",
    confidence: 0.92,
    suggestedAction: { kind: "copy", target: "Music Archive" },
    provenance: "llm",
    provider: "openai-compatible",
    model: "mimo-v2.5-pro",
    analysisAt: "2026-06-11T00:00:00Z",
  },
  {
    externalId: "BVtech",
    category: "programming",
    suggestedTags: ["Rust"],
    reason: "Programming topic.",
    confidence: 0.81,
    suggestedAction: { kind: "move", target: "Tech" },
    provenance: "llm",
    provider: "openai-compatible",
    model: "mimo-v2.5-pro",
    analysisAt: "2026-06-11T00:00:00Z",
  },
];

const filtered = filterClassificationReviewItems(favoriteItems, classifications, {
  ...DEFAULT_CLASSIFICATION_REVIEW_FILTERS,
  sourceFolderId: "folder-music",
  category: "music",
  tag: "live",
  minConfidence: 0.9,
  suggestedAction: "copy",
  suggestedTarget: "Music Archive",
  textQuery: "singer",
  biliCategory: "音乐",
});

export const classificationReviewContract = {
  filteredIds: filtered.map((item) => item.externalId),
  selectedFilteredFavoriteIds: Array.from(selectFilteredFavoriteIds(filtered)),
};
