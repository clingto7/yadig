import type {
  FavoriteOperationAction,
  FavoriteOperationCandidate,
  LibraryCollection,
  LlmClassificationItem,
} from "@/lib/tauri";
import type { ReviewableLibraryItem } from "@/lib/classification-review";

export type ClassificationDraftMode = "suggested" | FavoriteOperationAction;

export interface ClassificationDraftSelection {
  mode: ClassificationDraftMode;
  suggestedAction: FavoriteOperationAction;
}

export interface ClassificationDraftRow {
  externalId: string;
  title: string;
  category: string;
  confidence: number;
  provenance: LlmClassificationItem["provenance"];
  suggestedAction: FavoriteOperationAction | null;
  suggestedTarget: string | null;
}

export interface ClassificationDraftGroup {
  action: FavoriteOperationAction | "none";
  count: number;
  targetNames: string[];
}

export interface ClassificationDraftMetadata {
  classificationDraft: {
    category: string;
    confidence: number;
    provenance: LlmClassificationItem["provenance"];
    suggestedAction: FavoriteOperationAction | null;
    suggestedTarget: string | null;
    selectedAction: FavoriteOperationAction;
  };
}

const FAVORITE_ACTIONS: FavoriteOperationAction[] = ["copy", "move", "delete"];

function normalized(value: string | null | undefined): string {
  return value?.trim().toLocaleLowerCase() ?? "";
}

function normalizeFavoriteAction(value: string | null | undefined): FavoriteOperationAction | null {
  const normalizedValue = normalized(value);
  return FAVORITE_ACTIONS.includes(normalizedValue as FavoriteOperationAction)
    ? normalizedValue as FavoriteOperationAction
    : null;
}

export function buildClassificationDraftRows(
  items: ReviewableLibraryItem[],
  classifications: LlmClassificationItem[],
  selectedIds: Set<string>
): ClassificationDraftRow[] {
  const classificationById = new Map(
    classifications.map((classification) => [classification.externalId, classification])
  );

  return items
    .filter((item) => item.itemType === "bili_favorite_video" && selectedIds.has(item.externalId))
    .flatMap((item) => {
      const classification = classificationById.get(item.externalId);
      if (!classification) return [];

      return [{
        externalId: item.externalId,
        title: item.title,
        category: classification.category,
        confidence: classification.confidence,
        provenance: classification.provenance,
        suggestedAction: normalizeFavoriteAction(classification.suggestedAction?.kind),
        suggestedTarget: classification.suggestedAction?.target?.trim() || null,
      }];
    });
}

export function groupClassificationDraftRows(rows: ClassificationDraftRow[]): ClassificationDraftGroup[] {
  const groups = new Map<FavoriteOperationAction | "none", ClassificationDraftRow[]>();
  for (const row of rows) {
    const action = row.suggestedAction ?? "none";
    const group = groups.get(action) ?? [];
    group.push(row);
    groups.set(action, group);
  }

  return Array.from(groups.entries())
    .map(([action, groupRows]) => ({
      action,
      count: groupRows.length,
      targetNames: Array.from(
        new Set(groupRows.map((row) => row.suggestedTarget ?? "").filter(Boolean))
      ).sort((left, right) => left.localeCompare(right)),
    }))
    .sort((left, right) => left.action.localeCompare(right.action));
}

export function selectClassificationDraftRows(
  rows: ClassificationDraftRow[],
  selection: ClassificationDraftSelection
): ClassificationDraftRow[] {
  if (selection.mode === "suggested") {
    return rows.filter((row) => row.suggestedAction === selection.suggestedAction);
  }
  return rows;
}

export function matchFavoriteFolderBySuggestion(
  folders: LibraryCollection[],
  suggestedTarget: string | null | undefined
): LibraryCollection | null {
  const target = normalized(suggestedTarget);
  if (!target) return null;
  return folders.find((folder) => normalized(folder.title) === target) ?? null;
}

export function attachClassificationDraftMetadata(
  candidates: FavoriteOperationCandidate[],
  rows: ClassificationDraftRow[],
  selectedAction: FavoriteOperationAction
): FavoriteOperationCandidate[] {
  const rowsById = new Map(rows.map((row) => [row.externalId, row]));
  return candidates.map((candidate) => {
    const row = rowsById.get(candidate.externalId);
    if (!row) return candidate;

    const metadata: ClassificationDraftMetadata = {
      classificationDraft: {
        category: row.category,
        confidence: row.confidence,
        provenance: row.provenance,
        suggestedAction: row.suggestedAction,
        suggestedTarget: row.suggestedTarget,
        selectedAction,
      },
    };

    return {
      ...candidate,
      metadata: {
        ...(candidate.metadata ?? {}),
        ...metadata,
      },
    };
  });
}
