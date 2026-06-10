import { useEffect, useMemo, useState } from "react";
import { Store } from "@tauri-apps/plugin-store";
import { Brain, Download, RefreshCw, Tags } from "lucide-react";
import { tauri, type LibraryItem, type LlmItemAnalysis, type OperationPlan } from "@/lib/tauri";
import { listLatestLlmAnalyses, listLibraryItems, saveLlmAnalysis, saveOperationPlan, upsertLibraryItems } from "@/lib/db";

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

function isMusicSuggestion(analysis?: LlmItemAnalysis) {
  return Boolean(
    analysis?.suggestedTags.some((tag) => tag.includes("音乐"))
    || analysis?.suggestedAction?.kind === "extract_audio"
  );
}

export function WorkstationPage() {
  const [items, setItems] = useState<LibraryItem[]>([]);
  const [analyses, setAnalyses] = useState<LlmItemAnalysis[]>([]);
  const [plan, setPlan] = useState<OperationPlan | null>(null);
  const [instruction, setInstruction] = useState("请按领域给这些 B 站资源分类，并标出适合批量提取音频的音乐类视频。");
  const [busy, setBusy] = useState<string | null>(null);
  const [message, setMessage] = useState<string | null>(null);

  const analysisById = useMemo(() => {
    return new Map(analyses.map((analysis) => [analysis.externalId, analysis]));
  }, [analyses]);

  useEffect(() => {
    let cancelled = false;

    Promise.all([listLibraryItems(), listLatestLlmAnalyses()])
      .then(([storedItems, storedAnalyses]) => {
        if (!cancelled) {
          setItems(storedItems);
          setAnalyses(storedAnalyses);
        }
      })
      .catch((err) => {
        if (!cancelled) {
          setMessage(`Could not load local library: ${err}`);
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
      await upsertLibraryItems(result.items, syncedScope);
      setItems(await listLibraryItems());
      setAnalyses([]);
      setMessage(`Synced and saved ${result.items.length} Bilibili resources.`);
    } catch (err) {
      setMessage(`Sync failed: ${err}`);
    } finally {
      setBusy(null);
    }
  }

  async function analyzeMetadata() {
    setBusy("analyze");
    setMessage(null);
    try {
      const store = await Store.load("settings.json");
      const provider = {
        provider: (await store.get<string>("llm_provider")) ?? "openai-compatible",
        baseUrl: (await store.get<string>("llm_base_url")) ?? "https://api.openai.com/v1",
        apiKey: (await store.get<string>("llm_api_key")) ?? null,
        model: (await store.get<string>("llm_model")) ?? "gpt-4o-mini",
      };
      const result = await tauri.llmAnalyzeItems({
        instruction,
        items,
        provider,
      });
      await saveLlmAnalysis(instruction, provider, items, result.items);
      setAnalyses(result.items);
      setPlan(null);
      setMessage(
        result.warning
          ? `Analyzed ${result.items.length} resources with local metadata fallback. ${result.warning}`
          : `Analyzed ${result.items.length} resources with LLM and saved suggested tags.`
      );
    } catch (err) {
      setMessage(`Analysis failed: ${err}`);
    } finally {
      setBusy(null);
    }
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
      setMessage(`Plan creation failed: ${err}`);
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
      setMessage(`Audio extraction failed: ${err}`);
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
              <h3 className="font-semibold">AI Metadata Task</h3>
              <textarea
                value={instruction}
                onChange={(event) => setInstruction(event.target.value)}
                className="mt-3 min-h-28 w-full rounded-md border border-input bg-background p-3 text-sm focus:border-ring focus:outline-none focus:ring-1 focus:ring-ring"
              />
              <button
                onClick={analyzeMetadata}
                disabled={busy !== null || items.length === 0}
                className="mt-3 inline-flex h-9 items-center gap-2 rounded-md bg-primary px-4 text-sm font-medium text-primary-foreground hover:bg-primary/90 disabled:opacity-50"
              >
                <Brain className="h-4 w-4" />
                {busy === "analyze" ? "Analyzing" : "Analyze Metadata"}
              </button>
            </div>

            <div className="rounded-lg border border-border bg-card p-4">
              <h3 className="font-semibold">Music Audio Batch</h3>
              <p className="mt-1 text-sm text-muted-foreground">
                Build a download plan from videos tagged as music. Execution reuses the existing Bilibili audio extractor.
              </p>
              <div className="mt-4 flex flex-wrap gap-2">
                <button
                  onClick={createAudioPlan}
                  disabled={busy !== null || analyses.length === 0}
                  className="inline-flex h-9 items-center gap-2 rounded-md border border-border px-3 text-sm hover:bg-secondary disabled:opacity-50"
                >
                  <Tags className="h-4 w-4" />
                  Create Plan
                </button>
                <button
                  onClick={executeAudioPlan}
                  disabled={busy !== null || !plan || plan.items.length === 0}
                  className="inline-flex h-9 items-center gap-2 rounded-md bg-primary px-3 text-sm font-medium text-primary-foreground hover:bg-primary/90 disabled:opacity-50"
                >
                  <Download className="h-4 w-4" />
                  Extract Audio
                </button>
              </div>
              {plan && (
                <p className="mt-3 text-sm text-muted-foreground">
                  Plan ready: {plan.items.length} video{plan.items.length === 1 ? "" : "s"}.
                </p>
              )}
            </div>

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
            </div>
            <div className="divide-y divide-border">
              {items.length === 0 && (
                <div className="p-6 text-sm text-muted-foreground">
                  Sync Bilibili to start building your personal resource library.
                </div>
              )}
              {items.map((item) => {
                const analysis = analysisById.get(item.externalId);
                return (
                  <div key={`${item.itemType}:${item.externalId}`} className="grid gap-3 p-4 md:grid-cols-[1fr_260px]">
                    <div>
                      <div className="flex flex-wrap items-center gap-2">
                        <span className="rounded-full bg-secondary px-2 py-0.5 text-xs text-muted-foreground">
                          {itemTypeLabel(item.itemType)}
                        </span>
                        {isMusicSuggestion(analysis) && (
                          <span className="rounded-full bg-primary/15 px-2 py-0.5 text-xs text-primary">
                            Music candidate
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
                          <div className="flex flex-wrap gap-1">
                            {analysis.suggestedTags.map((tag) => (
                              <span key={tag} className="rounded bg-secondary px-2 py-0.5 text-xs">
                                {tag}
                              </span>
                            ))}
                          </div>
                          <p className="mt-2">{analysis.reason}</p>
                          <p className="mt-1 text-xs">Confidence {(analysis.confidence * 100).toFixed(0)}%</p>
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
