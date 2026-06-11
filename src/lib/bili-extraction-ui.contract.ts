import type { ExtractionResult } from "@/lib/tauri";
import {
  buildBiliBatchDownloadState,
  buildBiliExtractionProgressState,
  getBiliSavedFolderPath,
  shouldShowBiliBatchDownload,
} from "@/lib/bili-extraction-ui";

const singleResult: ExtractionResult = {
  videoTitle: "Single song",
  extractionType: "Single",
  warnings: [],
  segments: [
    {
      title: "Full",
      filePath: "/tmp/full.m4a",
      duration: 180,
      quality: 64,
      audioUrl: "https://example.com/full.m4a",
    },
  ],
};

const multipartResult: ExtractionResult = {
  videoTitle: "Multipart set",
  extractionType: "MultiPart",
  warnings: [],
  segments: [
    {
      title: "Part 1",
      filePath: "/tmp/part-1.m4a",
      duration: 120,
      quality: 192,
      audioUrl: "https://example.com/part-1.m4a",
    },
    {
      title: "Part 2",
      filePath: "/tmp/part-2.m4a",
      duration: 150,
      quality: 192,
      audioUrl: "https://example.com/part-2.m4a",
    },
  ],
};

const chapterFallbackResult: ExtractionResult = {
  videoTitle: "Chapter set",
  extractionType: "Chapters",
  warnings: ["Detected 2 chapters, but FFmpeg is not installed."],
  segments: [
    {
      title: "Full",
      filePath: "/tmp/chapter-full.m4a",
      duration: 300,
      quality: 192,
      audioUrl: "https://example.com/chapter-full.m4a",
    },
  ],
};

export const biliExtractionUiContract = {
  hidesBatchForSingle: shouldShowBiliBatchDownload(singleResult) === false,
  showsBatchForMultipart: shouldShowBiliBatchDownload(multipartResult) === true,
  hidesBatchForChapterFallback: shouldShowBiliBatchDownload(chapterFallbackResult) === false,
  preservesWarningContract: chapterFallbackResult.warnings[0],
  savedFolderPath: getBiliSavedFolderPath(multipartResult),
  idle: buildBiliBatchDownloadState(multipartResult, null),
  idleLabelUsesSavedFolder: buildBiliBatchDownloadState(multipartResult, null).label === "Open Folder",
  active: buildBiliBatchDownloadState(multipartResult, {
    completed: 1,
    total: 2,
  }),
  preparingExtraction: buildBiliExtractionProgressState({
    extracting: true,
    progress: null,
    cancelRequested: false,
  }),
  runningCollectionExtraction: buildBiliExtractionProgressState({
    extracting: true,
    progress: {
      jobId: "bili-job-contract",
      completed: 3,
      total: 10,
      currentTitle: "Track 3",
      cancelled: false,
    },
    cancelRequested: false,
  }),
  cancellingCollectionExtraction: buildBiliExtractionProgressState({
    extracting: true,
    progress: {
      jobId: "bili-job-contract",
      completed: 3,
      total: 10,
      currentTitle: "Track 3",
      cancelled: false,
    },
    cancelRequested: true,
  }),
};
