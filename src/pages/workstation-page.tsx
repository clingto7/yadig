import { useEffect, useMemo, useState } from "react";
import { Store } from "@tauri-apps/plugin-store";
import { Brain, Clock3, Copy as CopyIcon, Download, FolderInput, FolderPlus, RefreshCw, Tags, Trash2 } from "lucide-react";
import {
  buildClassificationProgress,
  chunkClassificationItems,
  formatClassificationProgress,
  type ClassificationProgress,
} from "@/lib/llm-classification-progress";
import {
  DEFAULT_CLASSIFICATION_REVIEW_FILTERS,
  filterClassificationReviewItems,
  selectFilteredFavoriteIds,
  uniqueBiliCategories,
  uniqueClassificationCategories,
  uniqueClassificationTags,
  uniqueSuggestedTargets,
  type ClassificationReviewFilters,
} from "@/lib/classification-review";
import { CHECKBOX_CLASS_NAME } from "@/lib/ui-style";
import {
  tauri,
  type FavoriteFolderPrivacy,
  type FavoriteOperationAction,
  type LibraryCollection,
  type LibraryItem,
  type LlmClassificationItem,
  type LlmClassificationMode,
  type OperationPlan,
  type OperationPlanItemStatus,
} from "@/lib/tauri";
import {
  listFavoriteOperationCandidates,
  listBiliFavoriteOperationPlanHistory,
  listLatestLlmClassifications,
  listLibraryCollections,
  listLibraryItemsWithCollections,
  saveLlmClassifications,
  saveOperationPlan,
  updateBiliFavoriteCopyMemberships,
  updateBiliFavoriteDeleteMemberships,
  updateBiliFavoriteMoveMemberships,
  upsertBiliFavoriteFolderFromCreatePlan,
  upsertBiliSyncResult,
  type LibraryItemWithCollections,
  type OperationPlanHistoryEntry,
  type OperationPlanHistoryItem,
} from "@/lib/db";
import {
  classifyOperationIssue,
  operationIssueLabel,
  operationPlanHistoryStatusLabel,
  operationPlanItemStatusLabel,
  sanitizeOperationError,
} from "@/lib/operation-plan-history";

const DEFAULT_LLM_PROVIDER = "openai-compatible";
const DEFAULT_LLM_BASE_URL = "https://token-plan-cn.xiaomimimo.com/v1";
const DEFAULT_LLM_MODEL = "mimo-v2.5-pro";

type ResourceFilter = "all" | LibraryItem["itemType"];
const OPERATION_ITEM_STATUSES: OperationPlanItemStatus[] = [
  "pending",
  "running",
  "success",
  "skipped",
  "failed",
  "blocked",
];

function itemTypeLabel(type: LibraryItem["itemType"]) {
  switch (type) {
    case "bili_favorite_video":
      return "Favorite";
    case "bili_watch_later_video":
      return "Watch Later";
    case "bili_followed_up":
      return "Following";
  }
}

function isMusicSuggestion(analysis?: LlmClassificationItem) {
  return Boolean(
    analysis?.suggestedTags.some((tag) => tag.includes("音乐"))
    || analysis?.category.toLowerCase().includes("music")
  );
}

function favoritePlanKindLabel(kind: OperationPlanHistoryEntry["kind"]) {
  switch (kind) {
    case "bili_batch_copy":
      return "Copy";
    case "bili_batch_move":
      return "Move";
    case "bili_batch_delete":
      return "Delete";
    case "bili_favorite_folder_create":
      return "Create Folder";
  }
}

function operationActionLabel(action: string) {
  switch (action) {
    case "copy":
    case "copy_favorite":
      return "Copy";
    case "move":
    case "move_favorite":
      return "Move";
    case "delete":
    case "delete_favorite":
      return "Delete";
    case "create_folder":
      return "Create Folder";
    default:
      return action;
  }
}

function folderPrivacyLabel(value: string | null) {
  return value === "1" ? "Private" : "Public";
}

function formatHistoryTime(value: string) {
  const date = new Date(value.replace(" ", "T"));
  if (Number.isNaN(date.getTime())) return value;
  return date.toLocaleString();
}

function sanitizeLlmError(error: string | null | undefined): string | null {
  const sanitized = sanitizeOperationError(error)
    ?.replace(/Bearer\s+[A-Za-z0-9._~+/=-]+/gi, "Bearer [redacted]")
    .replace(/(api[_-]?key=)[^&\s]+/gi, "$1[redacted]")
    .replace(/(token=)[^&\s]+/gi, "$1[redacted]")
    .replace(/\btp-[A-Za-z0-9_-]+/g, "[redacted]")
    .replace(/\bsk-[A-Za-z0-9_-]+/g, "[redacted]")
    .trim();

  return sanitized || null;
}

function safeErrorMessage(prefix: string, err: unknown) {
  return `${prefix}: ${sanitizeLlmError(String(err)) ?? "Unknown error"}`;
}

function confidencePercentToRatio(value: string): number {
  const parsed = Number(value);
  if (!Number.isFinite(parsed)) return 0;
  return Math.max(0, Math.min(100, parsed)) / 100;
}

export function WorkstationPage() {
  const [items, setItems] = useState<LibraryItemWithCollections[]>([]);
  const [favoriteFolders, setFavoriteFolders] = useState<LibraryCollection[]>([]);
  const [resourceFilter, setResourceFilter] = useState<ResourceFilter>("all");
  const [selectedFolderId, setSelectedFolderId] = useState<string>("all");
  const [selectedFavoriteIds, setSelectedFavoriteIds] = useState<Set<string>>(() => new Set());
  const [targetFolderId, setTargetFolderId] = useState<string>("");
  const [folderCreateTitle, setFolderCreateTitle] = useState("");
  const [folderCreateIntro, setFolderCreateIntro] = useState("");
  const [folderCreatePrivacy, setFolderCreatePrivacy] = useState<FavoriteFolderPrivacy>("private");
  const [classifications, setClassifications] = useState<LlmClassificationItem[]>([]);
  const [plan, setPlan] = useState<OperationPlan | null>(null);
  const [operationHistory, setOperationHistory] = useState<OperationPlanHistoryEntry[]>([]);
  const [selectedHistoryPlanId, setSelectedHistoryPlanId] = useState<number | null>(null);
  const [instruction, setInstruction] = useState("请按领域给这些 B 站资源分类，并标出适合批量提取音频的音乐类视频。");
  const [reviewFilters, setReviewFilters] = useState<ClassificationReviewFilters>(
    DEFAULT_CLASSIFICATION_REVIEW_FILTERS
  );
  const [busy, setBusy] = useState<string | null>(null);
  const [message, setMessage] = useState<string | null>(null);
  const [classificationProgress, setClassificationProgress] = useState<ClassificationProgress | null>(null);

  const analysisById = useMemo(() => {
    return new Map(classifications.map((analysis) => [analysis.externalId, analysis]));
  }, [classifications]);

  const visibleItems = useMemo(() => {
    const typeFilteredItems =
      resourceFilter === "all"
        ? items
        : items.filter((item) => item.itemType === resourceFilter);

    if (resourceFilter !== "bili_favorite_video") {
      return typeFilteredItems;
    }

    return filterClassificationReviewItems(typeFilteredItems, classifications, {
      ...reviewFilters,
      sourceFolderId: selectedFolderId,
    });
  }, [classifications, items, resourceFilter, reviewFilters, selectedFolderId]);

  const favoritePlanContextReady =
    resourceFilter === "bili_favorite_video" && selectedFolderId !== "all";

  const selectedFolder = useMemo(
    () => favoriteFolders.find((folder) => folder.externalId === selectedFolderId) ?? null,
    [favoriteFolders, selectedFolderId]
  );

  const targetFolder = useMemo(
    () => favoriteFolders.find((folder) => folder.externalId === targetFolderId) ?? null,
    [favoriteFolders, targetFolderId]
  );

  const favoriteVisibleIds = useMemo(() => {
    return new Set(
      visibleItems
        .filter((item) => item.itemType === "bili_favorite_video")
        .map((item) => item.externalId)
    );
  }, [visibleItems]);

  const visibleFavoriteCount = favoriteVisibleIds.size;

  const classificationCategories = useMemo(
    () => uniqueClassificationCategories(classifications),
    [classifications]
  );

  const classificationTags = useMemo(
    () => uniqueClassificationTags(classifications),
    [classifications]
  );

  const suggestedTargets = useMemo(
    () => uniqueSuggestedTargets(classifications),
    [classifications]
  );

  const biliCategories = useMemo(
    () => uniqueBiliCategories(items.filter((item) => item.itemType === "bili_favorite_video")),
    [items]
  );

  const planStatusCounts = useMemo(() => {
    const counts = new Map<string, number>();
    for (const item of plan?.items ?? []) {
      counts.set(item.status, (counts.get(item.status) ?? 0) + 1);
    }
    return counts;
  }, [plan]);

  const selectedHistoryPlan = useMemo(() => {
    if (selectedHistoryPlanId === null) return operationHistory[0] ?? null;
    return operationHistory.find((entry) => entry.id === selectedHistoryPlanId) ?? operationHistory[0] ?? null;
  }, [operationHistory, selectedHistoryPlanId]);

  async function refreshOperationHistory(preferredPlanId?: number | null) {
    const history = await listBiliFavoriteOperationPlanHistory(20);
    setOperationHistory(history);
    setSelectedHistoryPlanId((current) => {
      if (history.length === 0) return null;
      if (preferredPlanId && history.some((entry) => entry.id === preferredPlanId)) return preferredPlanId;
      if (current && history.some((entry) => entry.id === current)) return current;
      return history[0].id;
    });
  }

  useEffect(() => {
    if (resourceFilter !== "bili_favorite_video" && selectedFolderId !== "all") {
      setSelectedFolderId("all");
    }
  }, [resourceFilter, selectedFolderId]);

  useEffect(() => {
    setSelectedFavoriteIds(new Set());
    setTargetFolderId((current) => {
      if (current && current !== selectedFolderId) return current;
      return favoriteFolders.find((folder) => folder.externalId !== selectedFolderId)?.externalId ?? "";
    });
  }, [favoriteFolders, resourceFilter, selectedFolderId]);

  useEffect(() => {
    let cancelled = false;

    Promise.all([
      listLibraryItemsWithCollections(),
      listLibraryCollections("bili_favorite_folder"),
      listLatestLlmClassifications(),
      listBiliFavoriteOperationPlanHistory(20),
    ])
      .then(([storedItems, storedFolders, storedAnalyses, storedHistory]) => {
        if (!cancelled) {
          setItems(storedItems);
          setFavoriteFolders(storedFolders);
          setClassifications(storedAnalyses);
          setOperationHistory(storedHistory);
          setSelectedHistoryPlanId(storedHistory[0]?.id ?? null);
        }
      })
      .catch((err) => {
        if (!cancelled) {
          setMessage(safeErrorMessage("Could not load local library", err));
        }
      });

    return () => {
      cancelled = true;
    };
  }, []);

  async function syncBili() {
    setBusy("sync");
    setMessage(null);
    setPlan(null);
    try {
      const result = await tauri.biliSyncLibrary({
        scope: { favorites: true, follows: true, watchLater: true },
      });
      const syncedScope = {
        favorites: result.syncedFavorites,
        follows: result.syncedFollows,
        watchLater: result.syncedWatchLater,
      };
      await upsertBiliSyncResult(result, syncedScope);
      const [storedItems, storedFolders] = await Promise.all([
        listLibraryItemsWithCollections(),
        listLibraryCollections("bili_favorite_folder"),
      ]);
      setItems(storedItems);
      setFavoriteFolders(storedFolders);
      setResourceFilter("all");
      setSelectedFolderId("all");
      setSelectedFavoriteIds(new Set());
      setTargetFolderId("");
      setClassifications([]);
      setMessage(`Synced and saved ${result.items.length} Bilibili resources.`);
    } catch (err) {
      setMessage(safeErrorMessage("Sync failed", err));
    } finally {
      setBusy(null);
    }
  }

  function classificationInputItems() {
    const favoriteItems = visibleItems.filter((item) => item.itemType === "bili_favorite_video");
    if (selectedFavoriteIds.size === 0) return favoriteItems;
    return favoriteItems.filter((item) => selectedFavoriteIds.has(item.externalId));
  }

  async function classifyFavorites(mode: LlmClassificationMode) {
    const classificationItems = classificationInputItems();
    if (classificationItems.length === 0) {
      setMessage("Choose favorite items before classification.");
      return;
    }

    const chunks = chunkClassificationItems(classificationItems);
    setBusy(mode === "llm" ? "classify-llm" : "classify-local");
    setMessage(null);
    setClassificationProgress(buildClassificationProgress({
      mode,
      stage: "preparing",
      currentChunk: 0,
      totalChunks: chunks.length,
      processedItems: 0,
      totalItems: classificationItems.length,
      savedItems: 0,
      failedChunks: 0,
      latestError: null,
    }));

    try {
      const provider = mode === "llm"
        ? await loadLlmProvider()
        : null;

      let processedItems = 0;
      let savedItems = 0;
      let failedChunks = 0;
      let firstFailure: string | null = null;

      for (const [chunkIndex, chunkItems] of chunks.entries()) {
        const currentChunk = chunkIndex + 1;
        setClassificationProgress(buildClassificationProgress({
          mode,
          stage: "requesting",
          currentChunk,
          totalChunks: chunks.length,
          processedItems,
          totalItems: classificationItems.length,
          savedItems,
          failedChunks,
          currentChunkItemCount: chunkItems.length,
          latestError: null,
        }));

        const result = await tauri.llmClassifyItems({
          instruction,
          items: chunkItems,
          provider,
          mode,
        });

        failedChunks += result.chunkFailures.length;
        firstFailure ??= sanitizeLlmError(result.chunkFailures[0]?.error);

        setClassificationProgress(buildClassificationProgress({
          mode,
          stage: "saving",
          currentChunk,
          totalChunks: chunks.length,
          processedItems,
          totalItems: classificationItems.length,
          savedItems,
          failedChunks,
          currentChunkItemCount: chunkItems.length,
          latestError: firstFailure,
        }));

        await saveLlmClassifications(chunkItems, result.items);
        processedItems += chunkItems.length;
        savedItems += result.items.length;

        const storedChunkClassifications = await listLatestLlmClassifications();
        setClassifications(storedChunkClassifications);
      }

      const storedClassifications = await listLatestLlmClassifications();
      setClassifications(storedClassifications);
      setPlan(null);
      const sourceLabel = mode === "llm" ? "LLM" : "local metadata";
      setClassificationProgress(buildClassificationProgress({
        mode,
        stage: "completed",
        currentChunk: chunks.length,
        totalChunks: chunks.length,
        processedItems,
        totalItems: classificationItems.length,
        savedItems,
        failedChunks,
        latestError: firstFailure,
      }));
      setMessage(
        `${sourceLabel} classified and saved ${savedItems}/${classificationItems.length} favorite items${
          firstFailure
            ? `; ${failedChunks} chunk${failedChunks === 1 ? "" : "s"} failed: ${firstFailure}`
            : "."
        }`
      );
    } catch (err) {
      const error = sanitizeLlmError(String(err));
      setClassificationProgress((current) => current
        ? buildClassificationProgress({
            ...current,
            stage: "failed",
            latestError: error,
          })
        : null
      );
      setMessage(`Classification failed: ${error ?? "Unknown error"}`);
    } finally {
      setBusy(null);
    }
  }

  async function loadLlmProvider() {
    const store = await Store.load("settings.json");
    return {
      provider: (await store.get<string>("llm_provider")) ?? DEFAULT_LLM_PROVIDER,
      baseUrl: (await store.get<string>("llm_base_url")) ?? DEFAULT_LLM_BASE_URL,
      apiKey: (await store.get<string>("llm_api_key")) ?? null,
      model: (await store.get<string>("llm_model")) ?? DEFAULT_LLM_MODEL,
    };
  }

  async function createAudioPlan() {
    setBusy("plan");
    setMessage(null);
    try {
      const candidates = items
        .filter((item) => item.itemType === "bili_favorite_video" || item.itemType === "bili_watch_later_video")
        .map((item) => ({
          bvid: item.externalId,
          title: item.title,
          isMusic: isMusicSuggestion(analysisById.get(item.externalId)),
      }));
      const nextPlan = await tauri.createBiliAudioExtractionPlan({ candidates });
      await saveOperationPlan(nextPlan);
      setPlan(nextPlan);
      setMessage(`Created audio extraction plan with ${nextPlan.items.length} music videos.`);
    } catch (err) {
      setMessage(safeErrorMessage("Plan creation failed", err));
    } finally {
      setBusy(null);
    }
  }

  function toggleFavoriteSelection(externalId: string) {
    setSelectedFavoriteIds((current) => {
      const next = new Set(current);
      if (next.has(externalId)) {
        next.delete(externalId);
      } else {
        next.add(externalId);
      }
      return next;
    });
  }

  function selectVisibleFavorites() {
    const reviewItems = filterClassificationReviewItems(items, classifications, {
      ...reviewFilters,
      sourceFolderId: selectedFolderId,
    });
    setSelectedFavoriteIds(selectFilteredFavoriteIds(reviewItems));
  }

  function clearFavoriteSelection() {
    setSelectedFavoriteIds(new Set());
  }

  function updateReviewFilter<K extends keyof ClassificationReviewFilters>(
    key: K,
    value: ClassificationReviewFilters[K]
  ) {
    setReviewFilters((current) => ({
      ...current,
      [key]: value,
    }));
  }

  function clearReviewFilters() {
    setReviewFilters(DEFAULT_CLASSIFICATION_REVIEW_FILTERS);
  }

  async function createFavoritePlan(action: FavoriteOperationAction) {
    if (!favoritePlanContextReady) {
      setMessage("Choose a specific favorite folder before creating a remote operation plan.");
      return;
    }
    if (selectedFavoriteIds.size === 0) {
      setMessage("Select at least one favorite video before creating a plan.");
      return;
    }
    if ((action === "copy" || action === "move") && !targetFolder) {
      setMessage(`Choose a target favorite folder before creating a ${action} plan.`);
      return;
    }

    setBusy("favorite-plan");
    setMessage(null);
    try {
      const candidates = (await listFavoriteOperationCandidates(selectedFolderId))
        .filter((candidate) => selectedFavoriteIds.has(candidate.externalId));
      const nextPlan = await tauri.createBiliFavoriteOperationPlan({
        action,
        targetCollectionExternalId: action === "copy" || action === "move" ? targetFolder?.externalId ?? null : null,
        targetCollectionTitle: action === "copy" || action === "move" ? targetFolder?.title ?? null : null,
        items: candidates,
      });
      const savedPlanId = await saveOperationPlan(nextPlan);
      await refreshOperationHistory(savedPlanId);
      setPlan(nextPlan);
      const pendingCount = nextPlan.items.filter((item) => item.status === "pending").length;
      setMessage(`Created ${action} draft plan with ${pendingCount}/${nextPlan.items.length} executable items.`);
    } catch (err) {
      setMessage(safeErrorMessage("Favorite plan creation failed", err));
    } finally {
      setBusy(null);
    }
  }

  async function createFavoriteFolderCreatePlan() {
    const title = folderCreateTitle.trim();
    if (!title) {
      setMessage("Favorite folder title is required.");
      return;
    }
    if (title.length > 40) {
      setMessage("Favorite folder title must be 40 characters or fewer.");
      return;
    }

    setBusy("favorite-folder-create-plan");
    setMessage(null);
    try {
      const nextPlan = await tauri.createBiliFavoriteFolderCreatePlan({
        title,
        intro: folderCreateIntro.trim(),
        privacy: folderCreatePrivacy,
      });
      const savedPlanId = await saveOperationPlan(nextPlan);
      await refreshOperationHistory(savedPlanId);
      setPlan(nextPlan);
      const pendingCount = nextPlan.items.filter((item) => item.status === "pending").length;
      setMessage(`Created folder draft plan with ${pendingCount}/${nextPlan.items.length} executable items.`);
    } catch (err) {
      setMessage(safeErrorMessage("Favorite folder draft creation failed", err));
    } finally {
      setBusy(null);
    }
  }

  async function executeAudioPlan() {
    if (!plan) return;
    const confirmed = window.confirm(
      `Extract audio for ${plan.items.length} Bilibili video${plan.items.length === 1 ? "" : "s"}?`
    );
    if (!confirmed) return;

    setBusy("execute");
    setMessage(null);
    try {
      const result = await tauri.executeBiliAudioExtractionPlan({ plan });
      const successCount = result.results.filter((item) => item.status === "success").length;
      setMessage(`Extracted audio for ${successCount}/${result.results.length} videos.`);
    } catch (err) {
      setMessage(safeErrorMessage("Audio extraction failed", err));
    } finally {
      setBusy(null);
    }
  }

  async function executeFavoriteMovePlan() {
    if (!plan || plan.kind !== "bili_batch_move") return;
    const pendingCount = plan.items.filter((item) => item.status === "pending").length;
    if (pendingCount === 0) {
      setMessage("This move plan has no pending items to execute.");
      return;
    }
    const confirmed = window.confirm(
      `Move ${pendingCount} favorite video${pendingCount === 1 ? "" : "s"} on Bilibili? This changes your remote account.`
    );
    if (!confirmed) return;

    setBusy("favorite-move-execute");
    setMessage(null);
    try {
      const result = await tauri.executeBiliFavoriteMovePlan({ plan, confirmed: true });
      await updateBiliFavoriteMoveMemberships(result.plan);
      const savedPlanId = await saveOperationPlan(result.plan);
      await refreshOperationHistory(savedPlanId);
      const [storedItems, storedFolders] = await Promise.all([
        listLibraryItemsWithCollections(),
        listLibraryCollections("bili_favorite_folder"),
      ]);
      setItems(storedItems);
      setFavoriteFolders(storedFolders);
      setPlan(result.plan);
      setSelectedFavoriteIds(new Set());
      const successCount = result.plan.items.filter((item) => item.status === "success").length;
      setMessage(
        result.stopped
          ? `Move stopped after ${successCount}/${result.plan.items.length} successful items.`
          : `Moved ${successCount}/${result.plan.items.length} favorite items.`
      );
    } catch (err) {
      setMessage(safeErrorMessage("Favorite move execution failed", err));
    } finally {
      setBusy(null);
    }
  }

  async function executeFavoriteCopyPlan() {
    if (!plan || plan.kind !== "bili_batch_copy") return;
    const pendingCount = plan.items.filter((item) => item.status === "pending").length;
    if (pendingCount === 0) {
      setMessage("This copy plan has no pending items to execute.");
      return;
    }
    const confirmed = window.confirm(
      `Copy ${pendingCount} favorite video${pendingCount === 1 ? "" : "s"} on Bilibili? This preserves the source folder membership.`
    );
    if (!confirmed) return;

    setBusy("favorite-copy-execute");
    setMessage(null);
    try {
      const result = await tauri.executeBiliFavoriteCopyPlan({ plan, confirmed: true });
      await updateBiliFavoriteCopyMemberships(result.plan);
      const savedPlanId = await saveOperationPlan(result.plan);
      await refreshOperationHistory(savedPlanId);
      const [storedItems, storedFolders] = await Promise.all([
        listLibraryItemsWithCollections(),
        listLibraryCollections("bili_favorite_folder"),
      ]);
      setItems(storedItems);
      setFavoriteFolders(storedFolders);
      setPlan(result.plan);
      setSelectedFavoriteIds(new Set());
      const successCount = result.plan.items.filter((item) => item.status === "success").length;
      setMessage(
        result.stopped
          ? `Copy stopped after ${successCount}/${result.plan.items.length} successful items.`
          : `Copied ${successCount}/${result.plan.items.length} favorite items.`
      );
    } catch (err) {
      setMessage(safeErrorMessage("Favorite copy execution failed", err));
    } finally {
      setBusy(null);
    }
  }

  async function executeFavoriteDeletePlan() {
    if (!plan || plan.kind !== "bili_batch_delete") return;
    const pendingCount = plan.items.filter((item) => item.status === "pending").length;
    if (pendingCount === 0) {
      setMessage("This delete plan has no pending items to execute.");
      return;
    }
    const confirmationText = window.prompt(
      `Delete ${pendingCount} favorite video${pendingCount === 1 ? "" : "s"} from the selected Bilibili favorite folder? Type DELETE to confirm.`
    );
    if (confirmationText !== "DELETE") {
      setMessage("Delete execution cancelled.");
      return;
    }

    setBusy("favorite-delete-execute");
    setMessage(null);
    try {
      const result = await tauri.executeBiliFavoriteDeletePlan({ plan, confirmationText });
      await updateBiliFavoriteDeleteMemberships(result.plan);
      const savedPlanId = await saveOperationPlan(result.plan);
      await refreshOperationHistory(savedPlanId);
      const [storedItems, storedFolders] = await Promise.all([
        listLibraryItemsWithCollections(),
        listLibraryCollections("bili_favorite_folder"),
      ]);
      setItems(storedItems);
      setFavoriteFolders(storedFolders);
      setPlan(result.plan);
      setSelectedFavoriteIds(new Set());
      const successCount = result.plan.items.filter((item) => item.status === "success").length;
      setMessage(
        result.stopped
          ? `Delete stopped after ${successCount}/${result.plan.items.length} successful items.`
          : `Deleted ${successCount}/${result.plan.items.length} favorite items.`
      );
    } catch (err) {
      setMessage(safeErrorMessage("Favorite delete execution failed", err));
    } finally {
      setBusy(null);
    }
  }

  async function executeFavoriteFolderCreatePlan() {
    if (!plan || plan.kind !== "bili_favorite_folder_create") return;
    const pendingCount = plan.items.filter((item) => item.status === "pending").length;
    if (pendingCount === 0) {
      setMessage("This folder create plan has no pending items to execute.");
      return;
    }
    const folderTitle = plan.items[0]?.title ?? "favorite folder";
    const confirmed = window.confirm(`Create Bilibili favorite folder "${folderTitle}"?`);
    if (!confirmed) return;

    setBusy("favorite-folder-create-execute");
    setMessage(null);
    try {
      const result = await tauri.executeBiliFavoriteFolderCreatePlan({ plan, confirmed: true });
      await upsertBiliFavoriteFolderFromCreatePlan(result.plan);
      const savedPlanId = await saveOperationPlan(result.plan);
      await refreshOperationHistory(savedPlanId);
      const storedFolders = await listLibraryCollections("bili_favorite_folder");
      setFavoriteFolders(storedFolders);
      setPlan(result.plan);
      const createdItem = result.plan.items.find((item) => item.status === "success");
      if (createdItem?.targetCollectionExternalId) {
        setTargetFolderId(createdItem.targetCollectionExternalId);
      }
      setFolderCreateTitle("");
      setFolderCreateIntro("");
      const successCount = result.plan.items.filter((item) => item.status === "success").length;
      setMessage(`Created ${successCount}/${result.plan.items.length} favorite folders.`);
    } catch (err) {
      setMessage(safeErrorMessage("Favorite folder creation failed", err));
    } finally {
      setBusy(null);
    }
  }

  return (
    <div className="flex h-full flex-col">
      <header className="border-b border-border p-6">
        <h2 className="text-2xl font-bold">Media Workstation</h2>
        <p className="mt-1 text-sm text-muted-foreground">
          Organize Bilibili favorites, follows, and watch-later items with metadata-aware AI suggestions.
        </p>
      </header>

      <div className="flex-1 overflow-y-auto p-6">
        <section className="grid gap-4 lg:grid-cols-[360px_1fr]">
          <div className="space-y-4">
            <div className="rounded-lg border border-border bg-card p-4">
              <h3 className="font-semibold">Bilibili Scope</h3>
              <p className="mt-1 text-sm text-muted-foreground">
                Syncs favorites, followed UPs, and watch later. Full Cookie or QR login is recommended.
              </p>
              <button
                onClick={syncBili}
                disabled={busy !== null}
                className="mt-4 inline-flex h-9 items-center gap-2 rounded-md bg-primary px-4 text-sm font-medium text-primary-foreground hover:bg-primary/90 disabled:opacity-50"
              >
                <RefreshCw className="h-4 w-4" />
                {busy === "sync" ? "Syncing" : "Sync Bilibili"}
              </button>
            </div>

            <div className="rounded-lg border border-border bg-card p-4">
              <h3 className="font-semibold">Classification Task</h3>
              <textarea
                value={instruction}
                onChange={(event) => setInstruction(event.target.value)}
                className="mt-3 min-h-28 w-full rounded-md border border-input bg-background p-3 text-sm focus:border-ring focus:outline-none focus:ring-1 focus:ring-ring"
              />
              <div className="mt-3 flex flex-wrap gap-2">
                <button
                  onClick={() => void classifyFavorites("llm")}
                  disabled={busy !== null || visibleFavoriteCount === 0}
                  className="inline-flex h-9 items-center gap-2 rounded-md bg-primary px-4 text-sm font-medium text-primary-foreground hover:bg-primary/90 disabled:opacity-50"
                >
                  <Brain className="h-4 w-4" />
                  {busy === "classify-llm" ? "Classifying" : "Classify with LLM"}
                </button>
                <button
                  onClick={() => void classifyFavorites("local_metadata")}
                  disabled={busy !== null || visibleFavoriteCount === 0}
                  className="inline-flex h-9 items-center gap-2 rounded-md border border-border px-4 text-sm hover:bg-secondary disabled:opacity-50"
                >
                  <Tags className="h-4 w-4" />
                  {busy === "classify-local" ? "Classifying" : "Local Metadata"}
                </button>
              </div>
              {classificationProgress && (
                <div className="mt-3 rounded-md border border-border bg-secondary/40 p-3 text-sm text-muted-foreground">
                  <div className="flex items-start justify-between gap-3">
                    <span>{formatClassificationProgress(classificationProgress)}</span>
                    <span className="shrink-0 text-xs">
                      {classificationProgress.savedItems}/{classificationProgress.totalItems}
                    </span>
                  </div>
                  <div className="mt-2 h-2 overflow-hidden rounded-full bg-background">
                    <div
                      className="h-full rounded-full bg-primary transition-all"
                      style={{
                        width: `${
                          classificationProgress.totalItems > 0
                            ? Math.round(
                                (classificationProgress.processedItems / classificationProgress.totalItems) * 100
                              )
                            : 0
                        }%`,
                      }}
                    />
                  </div>
                </div>
              )}
            </div>

            <div className="rounded-lg border border-border bg-card p-4">
              <h3 className="font-semibold">Music Audio Batch</h3>
              <p className="mt-1 text-sm text-muted-foreground">
                Build a download plan from videos tagged as music. Execution reuses the existing Bilibili audio extractor.
              </p>
              <div className="mt-4 flex flex-wrap gap-2">
                <button
                  onClick={createAudioPlan}
                  disabled={busy !== null || classifications.length === 0}
                  className="inline-flex h-9 items-center gap-2 rounded-md border border-border px-3 text-sm hover:bg-secondary disabled:opacity-50"
                >
                  <Tags className="h-4 w-4" />
                  Create Plan
                </button>
                <button
                  onClick={executeAudioPlan}
                  disabled={busy !== null || !plan || plan.kind !== "bili_batch_audio_extraction" || plan.items.length === 0}
                  className="inline-flex h-9 items-center gap-2 rounded-md bg-primary px-3 text-sm font-medium text-primary-foreground hover:bg-primary/90 disabled:opacity-50"
                >
                  <Download className="h-4 w-4" />
                  Extract Audio
                </button>
              </div>
              {plan && (
                <p className="mt-3 text-sm text-muted-foreground">
                  Current plan: {plan.items.length} item{plan.items.length === 1 ? "" : "s"}.
                </p>
              )}
            </div>

            <div className="rounded-lg border border-border bg-card p-4">
              <h3 className="font-semibold">Favorite Remote Draft</h3>
              <div className="mt-3 space-y-3">
                <div className="grid gap-2">
                  <label className="block text-sm text-muted-foreground">
                    New folder title
                    <input
                      value={folderCreateTitle}
                      onChange={(event) => setFolderCreateTitle(event.target.value)}
                      className="mt-1 h-9 w-full rounded-md border border-input bg-background px-3 text-sm text-foreground focus:border-ring focus:outline-none focus:ring-1 focus:ring-ring"
                    />
                  </label>
                  <label className="block text-sm text-muted-foreground">
                    Introduction
                    <input
                      value={folderCreateIntro}
                      onChange={(event) => setFolderCreateIntro(event.target.value)}
                      className="mt-1 h-9 w-full rounded-md border border-input bg-background px-3 text-sm text-foreground focus:border-ring focus:outline-none focus:ring-1 focus:ring-ring"
                    />
                  </label>
                  <label className="block text-sm text-muted-foreground">
                    Privacy
                    <select
                      value={folderCreatePrivacy}
                      onChange={(event) => setFolderCreatePrivacy(event.target.value as FavoriteFolderPrivacy)}
                      className="mt-1 h-9 w-full rounded-md border border-input bg-background px-3 text-sm text-foreground focus:border-ring focus:outline-none focus:ring-1 focus:ring-ring"
                    >
                      <option value="private">Private</option>
                      <option value="public">Public</option>
                    </select>
                  </label>
                  <button
                    onClick={() => void createFavoriteFolderCreatePlan()}
                    disabled={busy !== null}
                    className="inline-flex h-9 items-center justify-center gap-2 rounded-md border border-border px-3 text-sm hover:bg-secondary disabled:opacity-50"
                  >
                    <FolderPlus className="h-4 w-4" />
                    Create Folder Draft
                  </button>
                </div>
                <label className="block text-sm text-muted-foreground">
                  Copy or move target
                  <select
                    value={targetFolderId}
                    onChange={(event) => setTargetFolderId(event.target.value)}
                    disabled={!favoritePlanContextReady}
                    className="mt-1 h-9 w-full rounded-md border border-input bg-background px-3 text-sm text-foreground focus:border-ring focus:outline-none focus:ring-1 focus:ring-ring disabled:opacity-50"
                  >
                    <option value="">Select folder</option>
                    {favoriteFolders.map((folder) => (
                      <option key={folder.externalId} value={folder.externalId}>
                        {folder.title}
                      </option>
                    ))}
                  </select>
                </label>
                <div className="flex flex-wrap gap-2 text-sm">
                  <button
                    onClick={() => void createFavoritePlan("copy")}
                    disabled={!favoritePlanContextReady || selectedFavoriteIds.size === 0 || !targetFolder || busy !== null}
                    className="inline-flex h-9 items-center gap-2 rounded-md border border-border px-3 hover:bg-secondary disabled:opacity-50"
                  >
                    <CopyIcon className="h-4 w-4" />
                    Copy Draft
                  </button>
                  <button
                    onClick={() => void createFavoritePlan("move")}
                    disabled={!favoritePlanContextReady || selectedFavoriteIds.size === 0 || !targetFolder || busy !== null}
                    className="inline-flex h-9 items-center gap-2 rounded-md border border-border px-3 hover:bg-secondary disabled:opacity-50"
                  >
                    <FolderInput className="h-4 w-4" />
                    Move Draft
                  </button>
                  <button
                    onClick={() => void createFavoritePlan("delete")}
                    disabled={!favoritePlanContextReady || selectedFavoriteIds.size === 0 || busy !== null}
                    className="inline-flex h-9 items-center gap-2 rounded-md border border-border px-3 hover:bg-secondary disabled:opacity-50"
                  >
                    <Trash2 className="h-4 w-4" />
                    Delete Draft
                  </button>
                </div>
                {favoritePlanContextReady && (
                  <p className="text-sm text-muted-foreground">
                    {selectedFavoriteIds.size} selected in {selectedFolder?.title ?? "favorite folder"}.
                  </p>
                )}
              </div>
            </div>

            <div className="rounded-lg border border-border bg-card p-4">
              <div className="flex items-center justify-between gap-3">
                <h3 className="font-semibold">Favorite Operation History</h3>
                <button
                  onClick={() => void refreshOperationHistory()}
                  disabled={busy !== null}
                  className="inline-flex h-8 items-center gap-2 rounded-md border border-border px-2 text-xs hover:bg-secondary disabled:opacity-50"
                >
                  <RefreshCw className="h-3.5 w-3.5" />
                  Refresh
                </button>
              </div>

              {operationHistory.length === 0 ? (
                <div className="mt-3 rounded border border-dashed border-border p-3 text-sm text-muted-foreground">
                  No favorite operation plans yet.
                </div>
              ) : (
                <div className="mt-3 space-y-3">
                  <div className="space-y-2">
                    {operationHistory.map((entry) => {
                      const selected = selectedHistoryPlan?.id === entry.id;
                      return (
                        <button
                          key={entry.id}
                          onClick={() => setSelectedHistoryPlanId(entry.id)}
                          className={`w-full rounded border p-3 text-left text-sm transition-colors hover:bg-secondary ${
                            selected ? "border-primary bg-primary/5" : "border-border"
                          }`}
                        >
                          <div className="flex items-center justify-between gap-3">
                            <span className="font-medium">{favoritePlanKindLabel(entry.kind)}</span>
                            <span className="rounded bg-secondary px-2 py-0.5 text-xs text-muted-foreground">
                              {operationPlanHistoryStatusLabel(entry.status)}
                            </span>
                          </div>
                          <div className="mt-1 flex flex-wrap items-center gap-x-2 gap-y-1 text-xs text-muted-foreground">
                            <span className="inline-flex items-center gap-1">
                              <Clock3 className="h-3 w-3" />
                              {formatHistoryTime(entry.createdAt)}
                            </span>
                            <span>{entry.itemCount} item{entry.itemCount === 1 ? "" : "s"}</span>
                          </div>
                          <div className="mt-2 flex flex-wrap gap-1 text-xs text-muted-foreground">
                            {OPERATION_ITEM_STATUSES.filter((status) => entry.statusCounts[status] > 0).map((status) => (
                              <span key={status} className="rounded bg-secondary px-2 py-0.5">
                                {operationPlanItemStatusLabel(status)} {entry.statusCounts[status]}
                              </span>
                            ))}
                          </div>
                        </button>
                      );
                    })}
                  </div>

                  {selectedHistoryPlan && (
                    <div className="max-h-80 space-y-2 overflow-y-auto rounded border border-border p-2">
                      {selectedHistoryPlan.items.map((item: OperationPlanHistoryItem) => {
                        const issueKind = classifyOperationIssue(item);
                        const safeError = sanitizeOperationError(item.error);
                        return (
                          <div key={item.id} className="rounded border border-border p-2 text-sm">
                            <div className="font-medium">{item.title}</div>
                            <div className="mt-1 text-xs text-muted-foreground">
                              {operationActionLabel(item.action)}
                              {" · "}
                              {item.sourceCollectionTitle ?? "Unknown source"}
                              {item.targetCollectionTitle ? ` -> ${item.targetCollectionTitle}` : ""}
                              {" · "}
                              {operationPlanItemStatusLabel(item.status)}
                            </div>
                            {issueKind !== "none" && (
                              <div className="mt-1 text-xs text-muted-foreground">
                                {operationIssueLabel(issueKind)}
                              </div>
                            )}
                            {safeError && (
                              <div className="mt-1 text-xs text-destructive">{safeError}</div>
                            )}
                          </div>
                        );
                      })}
                    </div>
                  )}
                </div>
              )}
            </div>

            {plan && plan.kind !== "bili_batch_audio_extraction" && (
              <div className="rounded-lg border border-border bg-card p-4">
                <h3 className="font-semibold">Plan Preview</h3>
                <div className="mt-2 flex flex-wrap gap-2 text-xs text-muted-foreground">
                  <span className="rounded bg-secondary px-2 py-0.5">
                    {favoritePlanKindLabel(plan.kind)}
                  </span>
                  {OPERATION_ITEM_STATUSES.map((status) => (
                    <span key={status} className="rounded bg-secondary px-2 py-0.5">
                      {operationPlanItemStatusLabel(status)} {planStatusCounts.get(status) ?? 0}
                    </span>
                  ))}
                </div>
                <div className="mt-3 max-h-64 space-y-2 overflow-y-auto text-sm">
                  {plan.items.map((item) => (
                    <div key={`${item.externalId}:${item.sourceCollectionExternalId ?? ""}`} className="rounded border border-border p-2">
                      <div className="font-medium">{item.title}</div>
                      <div className="mt-1 text-xs text-muted-foreground">
                        {operationActionLabel(item.action)}
                        {" · "}
                        {item.action === "create_folder" ? folderPrivacyLabel(item.target) : item.sourceCollectionTitle ?? "Unknown source"}
                        {item.action === "create_folder"
                          ? item.targetCollectionTitle ? ` · ${item.targetCollectionTitle}` : ""
                          : item.targetCollectionTitle ? ` -> ${item.targetCollectionTitle}` : ""}
                        {" · "}
                        {operationPlanItemStatusLabel(item.status)}
                      </div>
                      {item.error && (
                        <div className="mt-1 text-xs text-destructive">{sanitizeOperationError(item.error)}</div>
                      )}
                    </div>
                  ))}
                </div>
                {plan.kind === "bili_batch_move" && (
                  <button
                    onClick={() => void executeFavoriteMovePlan()}
                    disabled={busy !== null || (planStatusCounts.get("pending") ?? 0) === 0}
                    className="mt-3 inline-flex h-9 items-center gap-2 rounded-md bg-primary px-3 text-sm font-medium text-primary-foreground hover:bg-primary/90 disabled:opacity-50"
                  >
                    <FolderInput className="h-4 w-4" />
                    Execute Move
                  </button>
                )}
                {plan.kind === "bili_batch_copy" && (
                  <button
                    onClick={() => void executeFavoriteCopyPlan()}
                    disabled={busy !== null || (planStatusCounts.get("pending") ?? 0) === 0}
                    className="mt-3 inline-flex h-9 items-center gap-2 rounded-md bg-primary px-3 text-sm font-medium text-primary-foreground hover:bg-primary/90 disabled:opacity-50"
                  >
                    <CopyIcon className="h-4 w-4" />
                    Execute Copy
                  </button>
                )}
                {plan.kind === "bili_batch_delete" && (
                  <button
                    onClick={() => void executeFavoriteDeletePlan()}
                    disabled={busy !== null || (planStatusCounts.get("pending") ?? 0) === 0}
                    className="mt-3 inline-flex h-9 items-center gap-2 rounded-md bg-destructive px-3 text-sm font-medium text-destructive-foreground hover:bg-destructive/90 disabled:opacity-50"
                  >
                    <Trash2 className="h-4 w-4" />
                    Execute Delete
                  </button>
                )}
                {plan.kind === "bili_favorite_folder_create" && (
                  <button
                    onClick={() => void executeFavoriteFolderCreatePlan()}
                    disabled={busy !== null || (planStatusCounts.get("pending") ?? 0) === 0}
                    className="mt-3 inline-flex h-9 items-center gap-2 rounded-md bg-primary px-3 text-sm font-medium text-primary-foreground hover:bg-primary/90 disabled:opacity-50"
                  >
                    <FolderPlus className="h-4 w-4" />
                    Execute Create Folder
                  </button>
                )}
              </div>
            )}

            {message && (
              <div className="rounded-lg border border-border bg-card p-4 text-sm text-muted-foreground">
                {message}
              </div>
            )}
          </div>

          <div className="rounded-lg border border-border bg-card">
            <div className="border-b border-border p-4">
              <h3 className="font-semibold">Resource Review</h3>
              <p className="text-sm text-muted-foreground">
                {items.length} resources in the local library. AI suggestions stay local until you choose an action.
              </p>
              <label className="mt-3 block text-sm text-muted-foreground">
                Resource type
                <select
                  value={resourceFilter}
                  onChange={(event) => setResourceFilter(event.target.value as ResourceFilter)}
                  className="mt-1 h-9 w-full rounded-md border border-input bg-background px-3 text-sm text-foreground focus:border-ring focus:outline-none focus:ring-1 focus:ring-ring"
                >
                  <option value="all">All resources</option>
                  <option value="bili_favorite_video">Favorites</option>
                  <option value="bili_watch_later_video">Watch Later</option>
                  <option value="bili_followed_up">Following</option>
                </select>
              </label>
              {resourceFilter === "bili_favorite_video" && favoriteFolders.length > 0 && (
                <label className="mt-3 block text-sm text-muted-foreground">
                  Favorite folder
                  <select
                    value={selectedFolderId}
                    onChange={(event) => setSelectedFolderId(event.target.value)}
                    className="mt-1 h-9 w-full rounded-md border border-input bg-background px-3 text-sm text-foreground focus:border-ring focus:outline-none focus:ring-1 focus:ring-ring"
                  >
                    <option value="all">All favorite folders</option>
                    {favoriteFolders.map((folder) => (
                      <option key={folder.externalId} value={folder.externalId}>
                        {folder.title}
                      </option>
                    ))}
                  </select>
                </label>
              )}
              {resourceFilter === "bili_favorite_video" && (
                <div className="mt-4 grid gap-3 md:grid-cols-2 xl:grid-cols-4">
                  <label className="block text-sm text-muted-foreground">
                    Category
                    <select
                      value={reviewFilters.category}
                      onChange={(event) => updateReviewFilter("category", event.target.value)}
                      className="mt-1 h-9 w-full rounded-md border border-input bg-background px-3 text-sm text-foreground focus:border-ring focus:outline-none focus:ring-1 focus:ring-ring"
                    >
                      <option value="all">All categories</option>
                      {classificationCategories.map((category) => (
                        <option key={category} value={category}>
                          {category}
                        </option>
                      ))}
                    </select>
                  </label>
                  <label className="block text-sm text-muted-foreground">
                    Tag
                    <select
                      value={reviewFilters.tag}
                      onChange={(event) => updateReviewFilter("tag", event.target.value)}
                      className="mt-1 h-9 w-full rounded-md border border-input bg-background px-3 text-sm text-foreground focus:border-ring focus:outline-none focus:ring-1 focus:ring-ring"
                    >
                      <option value="all">All tags</option>
                      {classificationTags.map((tag) => (
                        <option key={tag} value={tag}>
                          {tag}
                        </option>
                      ))}
                    </select>
                  </label>
                  <label className="block text-sm text-muted-foreground">
                    Suggested action
                    <select
                      value={reviewFilters.suggestedAction}
                      onChange={(event) => updateReviewFilter("suggestedAction", event.target.value)}
                      className="mt-1 h-9 w-full rounded-md border border-input bg-background px-3 text-sm text-foreground focus:border-ring focus:outline-none focus:ring-1 focus:ring-ring"
                    >
                      <option value="all">All actions</option>
                      <option value="copy">Copy</option>
                      <option value="move">Move</option>
                      <option value="delete">Delete</option>
                      <option value="none">None</option>
                    </select>
                  </label>
                  <label className="block text-sm text-muted-foreground">
                    Suggested target
                    <select
                      value={reviewFilters.suggestedTarget}
                      onChange={(event) => updateReviewFilter("suggestedTarget", event.target.value)}
                      className="mt-1 h-9 w-full rounded-md border border-input bg-background px-3 text-sm text-foreground focus:border-ring focus:outline-none focus:ring-1 focus:ring-ring"
                    >
                      <option value="all">All targets</option>
                      {suggestedTargets.map((target) => (
                        <option key={target} value={target}>
                          {target}
                        </option>
                      ))}
                    </select>
                  </label>
                  <label className="block text-sm text-muted-foreground">
                    Min confidence
                    <input
                      type="number"
                      min="0"
                      max="100"
                      step="5"
                      value={Math.round(reviewFilters.minConfidence * 100)}
                      onChange={(event) =>
                        updateReviewFilter("minConfidence", confidencePercentToRatio(event.target.value))
                      }
                      className="mt-1 h-9 w-full rounded-md border border-input bg-background px-3 text-sm text-foreground focus:border-ring focus:outline-none focus:ring-1 focus:ring-ring"
                    />
                  </label>
                  <label className="block text-sm text-muted-foreground">
                    Bilibili category
                    <select
                      value={reviewFilters.biliCategory}
                      onChange={(event) => updateReviewFilter("biliCategory", event.target.value)}
                      className="mt-1 h-9 w-full rounded-md border border-input bg-background px-3 text-sm text-foreground focus:border-ring focus:outline-none focus:ring-1 focus:ring-ring"
                    >
                      <option value="all">All Bilibili categories</option>
                      {biliCategories.map((category) => (
                        <option key={category} value={category}>
                          {category}
                        </option>
                      ))}
                    </select>
                  </label>
                  <label className="block text-sm text-muted-foreground">
                    Provenance
                    <select
                      value={reviewFilters.provenance}
                      onChange={(event) => updateReviewFilter("provenance", event.target.value)}
                      className="mt-1 h-9 w-full rounded-md border border-input bg-background px-3 text-sm text-foreground focus:border-ring focus:outline-none focus:ring-1 focus:ring-ring"
                    >
                      <option value="all">All sources</option>
                      <option value="llm">LLM</option>
                      <option value="local_metadata">Local metadata</option>
                    </select>
                  </label>
                  <label className="block text-sm text-muted-foreground">
                    Title or author
                    <input
                      value={reviewFilters.textQuery}
                      onChange={(event) => updateReviewFilter("textQuery", event.target.value)}
                      className="mt-1 h-9 w-full rounded-md border border-input bg-background px-3 text-sm text-foreground focus:border-ring focus:outline-none focus:ring-1 focus:ring-ring"
                    />
                  </label>
                </div>
              )}
              {resourceFilter === "bili_favorite_video" && (
                <div className="mt-3 flex flex-wrap gap-2 text-sm">
                  <button
                    onClick={selectVisibleFavorites}
                    className="inline-flex h-8 items-center rounded-md border border-border px-3 hover:bg-secondary"
                  >
                    Select Filtered
                  </button>
                  <button
                    onClick={clearFavoriteSelection}
                    className="inline-flex h-8 items-center rounded-md border border-border px-3 hover:bg-secondary"
                  >
                    Clear
                  </button>
                  <button
                    onClick={clearReviewFilters}
                    className="inline-flex h-8 items-center rounded-md border border-border px-3 hover:bg-secondary"
                  >
                    Reset Filters
                  </button>
                  <span className="inline-flex h-8 items-center text-muted-foreground">
                    {selectedFavoriteIds.size} selected · {visibleFavoriteCount} filtered favorites
                  </span>
                </div>
              )}
            </div>
            <div className="divide-y divide-border">
              {visibleItems.length === 0 && (
                <div className="p-6 text-sm text-muted-foreground">
                  Sync Bilibili to start building your personal resource library.
                </div>
              )}
              {visibleItems.map((item) => {
                const analysis = analysisById.get(item.externalId);
                return (
                  <div
                    key={`${item.itemType}:${item.externalId}:${item.collections.map((collection) => collection.externalId).join(",")}`}
                    className="grid gap-3 p-4 md:grid-cols-[1fr_260px]"
                  >
                    <div>
                      <div className="flex flex-wrap items-center gap-2">
                        {resourceFilter === "bili_favorite_video" && item.itemType === "bili_favorite_video" && (
                          <input
                            type="checkbox"
                            checked={selectedFavoriteIds.has(item.externalId)}
                            onChange={() => toggleFavoriteSelection(item.externalId)}
                            className={CHECKBOX_CLASS_NAME}
                            aria-label={`Select ${item.title}`}
                          />
                        )}
                        <span className="rounded-full bg-secondary px-2 py-0.5 text-xs text-muted-foreground">
                          {itemTypeLabel(item.itemType)}
                        </span>
                        {item.collections.map((collection) => (
                          <span
                            key={collection.externalId}
                            className="rounded-full bg-secondary px-2 py-0.5 text-xs text-muted-foreground"
                          >
                            {collection.title}
                          </span>
                        ))}
                        {isMusicSuggestion(analysis) && (
                          <span className="rounded-full bg-primary/15 px-2 py-0.5 text-xs text-primary">
                            Music candidate
                          </span>
                        )}
                        {analysis && (
                          <span className="rounded-full bg-secondary px-2 py-0.5 text-xs text-muted-foreground">
                            {analysis.provenance === "llm" ? "LLM" : "Local metadata"}
                          </span>
                        )}
                      </div>
                      <h4 className="mt-2 font-medium">{item.title}</h4>
                      <p className="mt-1 text-sm text-muted-foreground">
                        {item.author ?? item.externalId}
                      </p>
                    </div>
                    <div className="text-sm text-muted-foreground">
                      {analysis ? (
                        <>
                          <div className="mb-2 flex flex-wrap gap-1">
                            <span className="rounded bg-secondary px-2 py-0.5 text-xs">
                              {analysis.category}
                            </span>
                            {analysis.suggestedAction && (
                              <span className="rounded bg-secondary px-2 py-0.5 text-xs">
                                {analysis.suggestedAction.kind}
                                {analysis.suggestedAction.target ? ` -> ${analysis.suggestedAction.target}` : ""}
                              </span>
                            )}
                          </div>
                          <div className="flex flex-wrap gap-1">
                            {analysis.suggestedTags.map((tag) => (
                              <span key={tag} className="rounded bg-secondary px-2 py-0.5 text-xs">
                                {tag}
                              </span>
                            ))}
                          </div>
                          <p className="mt-2">{analysis.reason}</p>
                          <p className="mt-1 text-xs">Confidence {(analysis.confidence * 100).toFixed(0)}%</p>
                          <p className="mt-1 text-xs">
                            {analysis.provenance === "llm" ? `${analysis.provider} · ${analysis.model}` : "Local metadata"}
                          </p>
                        </>
                      ) : (
                        "No AI suggestion yet."
                      )}
                    </div>
                  </div>
                );
              })}
            </div>
          </div>
        </section>
      </div>
    </div>
  );
}
