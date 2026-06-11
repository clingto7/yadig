import type {
  FavoriteOperationCandidate,
  LibraryCollection,
  LibraryItem,
  LlmClassificationItem,
} from "@/lib/tauri";
import {
  attachClassificationDraftMetadata,
  buildClassificationDraftRows,
  groupClassificationDraftRows,
  matchFavoriteFolderBySuggestion,
  selectClassificationDraftRows,
  type ClassificationDraftMetadata,
} from "@/lib/classification-draft-prefill";

type ReviewItem = LibraryItem & { collections: LibraryCollection[] };

const inboxFolder: LibraryCollection = {
  source: "bilibili",
  externalId: "folder-inbox",
  collectionType: "bili_favorite_folder",
  title: "Inbox",
  rawMetadata: {},
};

const archiveFolder: LibraryCollection = {
  source: "bilibili",
  externalId: "folder-archive",
  collectionType: "bili_favorite_folder",
  title: "Music Archive",
  rawMetadata: {},
};

const reviewItems: ReviewItem[] = [
  {
    source: "bilibili",
    externalId: "BVcopy",
    itemType: "bili_favorite_video",
    title: "Copy candidate",
    author: null,
    url: null,
    imageUrl: null,
    rawMetadata: { resourceId: "1001", resourceType: "2" },
    collections: [inboxFolder],
  },
  {
    source: "bilibili",
    externalId: "BVmove",
    itemType: "bili_favorite_video",
    title: "Move candidate",
    author: null,
    url: null,
    imageUrl: null,
    rawMetadata: { resourceId: "1002", resourceType: "2" },
    collections: [inboxFolder],
  },
  {
    source: "bilibili",
    externalId: "BVnone",
    itemType: "bili_favorite_video",
    title: "Force action candidate",
    author: null,
    url: null,
    imageUrl: null,
    rawMetadata: { resourceId: "1003", resourceType: "2" },
    collections: [inboxFolder],
  },
];

const classifications: LlmClassificationItem[] = [
  {
    externalId: "BVcopy",
    category: "music",
    suggestedTags: ["live"],
    reason: "Archive this music item.",
    confidence: 0.92,
    suggestedAction: { kind: "copy", target: "music archive" },
    provenance: "llm",
    provider: "openai-compatible",
    model: "mimo-v2.5-pro",
    analysisAt: "2026-06-11T00:00:00Z",
  },
  {
    externalId: "BVmove",
    category: "programming",
    suggestedTags: ["Rust"],
    reason: "Move this programming item.",
    confidence: 0.81,
    suggestedAction: { kind: "move", target: "Tech" },
    provenance: "llm",
    provider: "openai-compatible",
    model: "mimo-v2.5-pro",
    analysisAt: "2026-06-11T00:00:00Z",
  },
  {
    externalId: "BVnone",
    category: "misc",
    suggestedTags: [],
    reason: "No remote operation suggested.",
    confidence: 0.44,
    suggestedAction: null,
    provenance: "llm",
    provider: "openai-compatible",
    model: "mimo-v2.5-pro",
    analysisAt: "2026-06-11T00:00:00Z",
  },
];

const selectedIds = new Set(["BVcopy", "BVmove", "BVnone"]);
const rows = buildClassificationDraftRows(reviewItems, classifications, selectedIds);
const groups = groupClassificationDraftRows(rows);
const suggestedCopyRows = selectClassificationDraftRows(rows, {
  mode: "suggested",
  suggestedAction: "copy",
});
const forcedDeleteRows = selectClassificationDraftRows(rows, {
  mode: "delete",
  suggestedAction: "copy",
});

const candidates: FavoriteOperationCandidate[] = [
  {
    externalId: "BVcopy",
    title: "Copy candidate",
    sourceCollectionExternalId: "folder-inbox",
    sourceCollectionTitle: "Inbox",
    collectionExternalIds: ["folder-inbox"],
    resourceId: "1001",
    resourceType: "2",
  },
];

const classifiedCandidates = attachClassificationDraftMetadata(candidates, suggestedCopyRows, "copy");
const matchedFolder = matchFavoriteFolderBySuggestion([archiveFolder], "music archive");
const firstMetadata = classifiedCandidates[0]?.metadata as ClassificationDraftMetadata | undefined;

export const classificationDraftPrefillContract = {
  groupCounts: groups.map((group) => `${group.action}:${group.count}`),
  suggestedCopyIds: suggestedCopyRows.map((row) => row.externalId),
  forcedDeleteIds: forcedDeleteRows.map((row) => row.externalId),
  matchedFolderId: matchedFolder?.externalId ?? null,
  metadataCategory: firstMetadata?.classificationDraft.category,
  metadataSelectedAction: firstMetadata?.classificationDraft.selectedAction,
  createDraftOnly: true,
};
