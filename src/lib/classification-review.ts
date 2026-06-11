import type { LibraryCollection, LibraryItem, LlmClassificationItem } from "@/lib/tauri";

export type ReviewableLibraryItem = LibraryItem & {
  collections: LibraryCollection[];
};

export interface ClassificationReviewItem extends ReviewableLibraryItem {
  classification: LlmClassificationItem | null;
}

export interface ClassificationReviewFilters {
  sourceFolderId: string;
  category: string;
  tag: string;
  minConfidence: number;
  suggestedAction: string;
  suggestedTarget: string;
  textQuery: string;
  biliCategory: string;
  provenance: string;
}

export const DEFAULT_CLASSIFICATION_REVIEW_FILTERS: ClassificationReviewFilters = {
  sourceFolderId: "all",
  category: "all",
  tag: "all",
  minConfidence: 0,
  suggestedAction: "all",
  suggestedTarget: "all",
  textQuery: "",
  biliCategory: "all",
  provenance: "all",
};

function normalized(value: string | null | undefined): string {
  return value?.trim().toLocaleLowerCase() ?? "";
}

function metadataString(item: LibraryItem, key: string): string {
  const value = item.rawMetadata[key];
  if (typeof value === "string") return value;
  if (typeof value === "number") return String(value);
  return "";
}

export function buildClassificationReviewItems(
  items: ReviewableLibraryItem[],
  classifications: LlmClassificationItem[]
): ClassificationReviewItem[] {
  const classificationById = new Map(
    classifications.map((classification) => [classification.externalId, classification])
  );
  return items.map((item) => ({
    ...item,
    classification: classificationById.get(item.externalId) ?? null,
  }));
}

export function filterClassificationReviewItems(
  items: ReviewableLibraryItem[],
  classifications: LlmClassificationItem[],
  filters: ClassificationReviewFilters
): ClassificationReviewItem[] {
  return buildClassificationReviewItems(items, classifications).filter((item) => {
    if (item.itemType !== "bili_favorite_video") return false;

    const classification = item.classification;
    if (filters.sourceFolderId !== "all") {
      const inFolder = item.collections.some(
        (collection) => collection.externalId === filters.sourceFolderId
      );
      if (!inFolder) return false;
    }

    if (filters.category !== "all" && normalized(classification?.category) !== normalized(filters.category)) {
      return false;
    }

    if (
      filters.tag !== "all"
      && !classification?.suggestedTags.some((tag) => normalized(tag) === normalized(filters.tag))
    ) {
      return false;
    }

    if (filters.minConfidence > 0 && (classification?.confidence ?? -1) < filters.minConfidence) {
      return false;
    }

    const action = classification?.suggestedAction?.kind ?? "none";
    if (filters.suggestedAction !== "all" && normalized(action) !== normalized(filters.suggestedAction)) {
      return false;
    }

    const target = classification?.suggestedAction?.target ?? "";
    if (filters.suggestedTarget !== "all" && normalized(target) !== normalized(filters.suggestedTarget)) {
      return false;
    }

    if (filters.provenance !== "all" && normalized(classification?.provenance) !== normalized(filters.provenance)) {
      return false;
    }

    const query = normalized(filters.textQuery);
    if (query) {
      const haystack = [
        item.title,
        item.author ?? "",
        item.externalId,
      ].map(normalized).join(" ");
      if (!haystack.includes(query)) return false;
    }

    if (filters.biliCategory !== "all" && normalized(metadataString(item, "tname")) !== normalized(filters.biliCategory)) {
      return false;
    }

    return true;
  });
}

export function selectFilteredFavoriteIds(items: ClassificationReviewItem[]): Set<string> {
  return new Set(
    items
      .filter((item) => item.itemType === "bili_favorite_video")
      .map((item) => item.externalId)
  );
}

export function uniqueClassificationCategories(classifications: LlmClassificationItem[]): string[] {
  return Array.from(
    new Set(classifications.map((classification) => classification.category).filter(Boolean))
  ).sort((a, b) => a.localeCompare(b));
}

export function uniqueClassificationTags(classifications: LlmClassificationItem[]): string[] {
  return Array.from(
    new Set(classifications.flatMap((classification) => classification.suggestedTags).filter(Boolean))
  ).sort((a, b) => a.localeCompare(b));
}

export function uniqueSuggestedTargets(classifications: LlmClassificationItem[]): string[] {
  return Array.from(
    new Set(
      classifications
        .map((classification) => classification.suggestedAction?.target?.trim() ?? "")
        .filter(Boolean)
    )
  ).sort((a, b) => a.localeCompare(b));
}

export function uniqueBiliCategories(items: LibraryItem[]): string[] {
  return Array.from(
    new Set(items.map((item) => metadataString(item, "tname").trim()).filter(Boolean))
  ).sort((a, b) => a.localeCompare(b));
}
