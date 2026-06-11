import type { ExtractionResult } from "@/lib/tauri";

export interface BiliBatchDownloadProgress {
  completed: number;
  total: number;
}

export interface BiliBatchDownloadState {
  show: boolean;
  disabled: boolean;
  label: string;
}

export function shouldShowBiliBatchDownload(result: ExtractionResult): boolean {
  return result.segments.length > 1;
}

export function buildBiliBatchDownloadState(
  result: ExtractionResult,
  progress: BiliBatchDownloadProgress | null
): BiliBatchDownloadState {
  const show = shouldShowBiliBatchDownload(result);
  const idleLabel = getBiliSavedFolderPath(result) ? "Open Folder" : "Download All";
  if (!show) {
    return {
      show,
      disabled: true,
      label: idleLabel,
    };
  }

  if (!progress) {
    return {
      show,
      disabled: false,
      label: idleLabel,
    };
  }

  return {
    show,
    disabled: true,
    label: `Opening ${progress.completed}/${progress.total}`,
  };
}

export function getBiliSavedFolderPath(result: ExtractionResult): string | null {
  const filePath = result.segments.find((segment) => segment.filePath.trim().length > 0)?.filePath;
  if (!filePath) return null;

  const lastUnixSeparator = filePath.lastIndexOf("/");
  const lastWindowsSeparator = filePath.lastIndexOf("\\");
  const separatorIndex = Math.max(lastUnixSeparator, lastWindowsSeparator);
  if (separatorIndex <= 0) return null;
  return filePath.slice(0, separatorIndex);
}
