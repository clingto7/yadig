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

export interface BiliCollectionExtractionProgress {
  jobId: string;
  completed: number;
  total: number;
  currentTitle: string | null;
  cancelled: boolean;
}

export interface BiliExtractionProgressStateInput {
  extracting: boolean;
  progress: BiliCollectionExtractionProgress | null;
  cancelRequested: boolean;
}

export interface BiliExtractionProgressState {
  show: boolean;
  label: string;
  percent: number;
  canCancel: boolean;
}

export function shouldShowBiliBatchDownload(result: ExtractionResult): boolean {
  return result.segments.length > 1;
}

export function buildBiliExtractionProgressState({
  extracting,
  progress,
  cancelRequested,
}: BiliExtractionProgressStateInput): BiliExtractionProgressState {
  if (!extracting) {
    return {
      show: false,
      label: "",
      percent: 0,
      canCancel: false,
    };
  }

  if (!progress) {
    return {
      show: true,
      label: "Preparing audio extraction...",
      percent: 0,
      canCancel: false,
    };
  }

  const total = Math.max(0, progress.total);
  const completed = Math.min(Math.max(0, progress.completed), total);
  const percent = total > 0 ? Math.round((completed / total) * 100) : 0;
  const position = total > 0 ? `${completed}/${total}` : "0/0";
  const cancelling = cancelRequested || progress.cancelled;
  const label = cancelling
    ? `Cancelling after ${position}...`
    : progress.currentTitle
      ? `Extracting ${position}: ${progress.currentTitle}`
      : `Extracting ${position}`;

  return {
    show: true,
    label,
    percent,
    canCancel: !cancelling && total > 0 && completed < total,
  };
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
