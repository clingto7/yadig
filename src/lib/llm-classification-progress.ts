import type { LibraryItem, LlmClassificationMode } from "@/lib/tauri";

export const LLM_CLASSIFICATION_CHUNK_SIZE = 8;

export type ClassificationProgressStage =
  | "preparing"
  | "requesting"
  | "saving"
  | "completed"
  | "failed";

export interface ClassificationProgress {
  mode: LlmClassificationMode;
  stage: ClassificationProgressStage;
  currentChunk: number;
  totalChunks: number;
  processedItems: number;
  totalItems: number;
  savedItems: number;
  failedChunks: number;
  currentChunkItemCount?: number;
  latestError?: string | null;
}

export function chunkClassificationItems<T extends LibraryItem>(
  items: T[],
  chunkSize = LLM_CLASSIFICATION_CHUNK_SIZE
): T[][] {
  if (chunkSize <= 0) {
    throw new Error("Classification chunk size must be greater than zero.");
  }

  const chunks: T[][] = [];
  for (let index = 0; index < items.length; index += chunkSize) {
    chunks.push(items.slice(index, index + chunkSize));
  }
  return chunks;
}

export function buildClassificationProgress(
  progress: ClassificationProgress
): ClassificationProgress {
  return progress;
}

export function formatClassificationProgress(progress: ClassificationProgress): string {
  const source = progress.mode === "llm" ? "LLM" : "Local metadata";
  const chunkLabel = progress.totalChunks > 0
    ? `chunk ${progress.currentChunk}/${progress.totalChunks}`
    : "chunk 0/0";
  const itemLabel = `${progress.processedItems}/${progress.totalItems} processed`;
  const savedLabel = `${progress.savedItems} saved`;
  const failureLabel = progress.failedChunks > 0
    ? `, ${progress.failedChunks} failed chunk${progress.failedChunks === 1 ? "" : "s"}`
    : "";

  switch (progress.stage) {
    case "preparing":
      return `${source} classification preparing ${progress.totalItems} item${progress.totalItems === 1 ? "" : "s"}.`;
    case "requesting":
      return `${source} classification requesting ${chunkLabel} (${progress.currentChunkItemCount ?? 0} items); ${itemLabel}, ${savedLabel}${failureLabel}.`;
    case "saving":
      return `${source} classification saving ${chunkLabel}; ${itemLabel}, ${savedLabel}${failureLabel}.`;
    case "completed":
      return `${source} classification completed; ${itemLabel}, ${savedLabel}${failureLabel}.`;
    case "failed":
      return `${source} classification failed at ${chunkLabel}; ${itemLabel}, ${savedLabel}${failureLabel}${
        progress.latestError ? `: ${progress.latestError}` : "."
      }`;
  }
}
