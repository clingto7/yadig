import { useState, useEffect, useMemo, useRef } from "react";
import { useQuery, useInfiniteQuery } from "@tanstack/react-query";
import { Search as SearchIcon, Loader2, ExternalLink, Star, Clock, ChevronDown, Copy, Check, Play, Pause, Download } from "lucide-react";
import { tauri } from "@/lib/tauri";
import { saveSearch, listSearches, addFavorite, isFavorite } from "@/lib/db";
import { usePlayer } from "@/lib/player-context";
import type { ContentItem, SearchResult } from "@/types/source";

const SOURCE_TAB_ALL = "__all__";

export function SearchPage() {
  const [query, setQuery] = useState("");
  const [searchTerm, setSearchTerm] = useState("");
  const [showHistory, setShowHistory] = useState(false);
  const [activeTab, setActiveTab] = useState(SOURCE_TAB_ALL);
  const savedSearch = useRef(false);

  const { data: sources } = useQuery({
    queryKey: ["sources"],
    queryFn: () => tauri.listSources().catch(() => []),
  });

  const {
    data: results,
    isLoading,
    isFetchingNextPage,
    hasNextPage,
    fetchNextPage,
  } = useInfiniteQuery({
    queryKey: ["search", searchTerm],
    queryFn: ({ pageParam }) => tauri.searchSources({
      query: searchTerm,
      page: pageParam,
    }),
    initialPageParam: 1,
    getNextPageParam: (lastPage: SearchResult) =>
      lastPage.page.hasMore ? lastPage.page.page + 1 : undefined,
    enabled: searchTerm.length > 0,
  });

  const { data: latest } = useQuery({
    queryKey: ["latest"],
    queryFn: () => tauri.fetchLatest({ limit: 10 }).catch(() => []),
  });

  const { data: history } = useQuery({
    queryKey: ["searchHistory"],
    queryFn: () => listSearches(10).catch(() => []),
  });

  // Build a source name lookup for displaying names instead of IDs
  const sourceNameMap = useMemo(() => {
    const map = new Map<string, string>();
    sources?.forEach((s) => map.set(s.id, s.name));
    return map;
  }, [sources]);

  // Deduplicate all fetched pages by URL
  const deduplicatedItems = useMemo(() => {
    if (!results || !searchTerm) return null;
    const seen = new Map<string, ContentItem>();
    for (const page of results.pages) {
      for (const item of page.items) {
        const key = item.url || `${item.sourceId}-${item.title}`;
        if (!seen.has(key)) {
          seen.set(key, item);
        }
      }
    }
    return Array.from(seen.values());
  }, [results, searchTerm]);

  function handleSearch(e: React.FormEvent) {
    e.preventDefault();
    if (query.trim()) {
      setSearchTerm(query.trim());
      setShowHistory(false);
    }
  }

  function handleHistoryClick(historicalQuery: string) {
    setQuery(historicalQuery);
    setSearchTerm(historicalQuery);
    setShowHistory(false);
  }

  // Reset savedSearch ref when search term changes
  useEffect(() => {
    savedSearch.current = false;
  }, [searchTerm]);

  // Save search to history only once (first page)
  useEffect(() => {
    if (results && searchTerm && !savedSearch.current) {
      savedSearch.current = true;
      saveSearch(
        searchTerm,
        results.pages[0]?.totalResults ?? 0,
        sources?.map((s) => s.id).join(",") ?? ""
      ).catch(() => {});
    }
  }, [results, searchTerm, sources]);

  const allItems = searchTerm ? deduplicatedItems : latest;

  // Group items by source for tab display
  const itemsBySource = useMemo(() => {
    if (!deduplicatedItems) return new Map<string, ContentItem[]>();
    const map = new Map<string, ContentItem[]>();
    for (const item of deduplicatedItems) {
      const list = map.get(item.sourceId) || [];
      list.push(item);
      map.set(item.sourceId, list);
    }
    return map;
  }, [deduplicatedItems]);

  // Active sources that have results
  const activeSourceTabs = useMemo(() => {
    const tabs: { id: string; name: string; count: number }[] = [];
    for (const [sourceId, items] of itemsBySource) {
      tabs.push({
        id: sourceId,
        name: sourceNameMap.get(sourceId) ?? sourceId,
        count: items.length,
      });
    }
    return tabs;
  }, [itemsBySource, sourceNameMap]);

  // Items for the currently active tab
  const tabItems = useMemo(() => {
    if (!searchTerm) return allItems;
    if (activeTab === SOURCE_TAB_ALL) return deduplicatedItems;
    return itemsBySource.get(activeTab) ?? [];
  }, [searchTerm, activeTab, allItems, deduplicatedItems, itemsBySource]);

  return (
    <div className="flex h-full flex-col">
      <header className="border-b border-border p-6">
        <h2 className="text-2xl font-bold">Discover</h2>
        <p className="mt-1 text-sm text-muted-foreground">
          Search across music sources
        </p>

        <form onSubmit={handleSearch} className="mt-4 flex gap-2">
          <div className="relative flex-1">
            <SearchIcon className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
            <input
              type="text"
              value={query}
              onChange={(e) => setQuery(e.target.value)}
              onFocus={() => setShowHistory(true)}
              onBlur={() => setTimeout(() => setShowHistory(false), 200)}
              placeholder="Search artists, albums, labels..."
              className="h-10 w-full rounded-lg border border-input bg-background pl-10 pr-4 text-sm placeholder:text-muted-foreground focus:border-ring focus:outline-none focus:ring-1 focus:ring-ring"
            />

            {showHistory && history && history.length > 0 && !searchTerm && (
              <div className="absolute left-0 top-full z-10 mt-1 w-full rounded-lg border border-border bg-card shadow-lg">
                <div className="flex items-center gap-1 px-3 py-2 text-xs text-muted-foreground">
                  <Clock className="h-3 w-3" />
                  Recent searches
                </div>
                {history.map((h) => (
                  <button
                    key={h.id}
                    className="flex w-full items-center gap-2 px-3 py-2 text-sm hover:bg-accent text-left"
                    onMouseDown={() => handleHistoryClick(h.query)}
                  >
                    <Clock className="h-3 w-3 text-muted-foreground" />
                    {h.query}
                    {h.result_count != null && (
                      <span className="ml-auto text-xs text-muted-foreground">
                        {h.result_count} results
                      </span>
                    )}
                  </button>
                ))}
              </div>
            )}
          </div>
          <button
            type="submit"
            className="h-10 rounded-lg bg-primary px-4 text-sm font-medium text-primary-foreground hover:bg-primary/90"
          >
            Search
          </button>
        </form>
      </header>

      {/* Source tabs — only shown when searching */}
      {searchTerm && deduplicatedItems && deduplicatedItems.length > 0 && (
        <div className="border-b border-border px-6 pt-2">
          <div className="flex gap-1 overflow-x-auto">
            <button
              onClick={() => setActiveTab(SOURCE_TAB_ALL)}
              className={`whitespace-nowrap rounded-t-md px-3 py-1.5 text-xs font-medium transition-colors ${
                activeTab === SOURCE_TAB_ALL
                  ? "bg-primary/15 text-primary border-b-2 border-primary"
                  : "text-muted-foreground hover:text-foreground hover:bg-accent"
              }`}
            >
              All ({deduplicatedItems.length})
            </button>
            {activeSourceTabs.map((tab) => (
              <button
                key={tab.id}
                onClick={() => setActiveTab(tab.id)}
                className={`whitespace-nowrap rounded-t-md px-3 py-1.5 text-xs font-medium transition-colors ${
                  activeTab === tab.id
                    ? "bg-primary/15 text-primary border-b-2 border-primary"
                    : "text-muted-foreground hover:text-foreground hover:bg-accent"
                }`}
              >
                {tab.name} ({tab.count})
              </button>
            ))}
          </div>
        </div>
      )}

      <div className="flex-1 overflow-y-auto p-6">
        {isLoading && (
          <div className="flex items-center gap-2 text-muted-foreground">
            <Loader2 className="h-4 w-4 animate-spin" />
            Searching...
          </div>
        )}

        {tabItems && tabItems.length > 0 && (
          <div className="grid gap-3">
            {tabItems.map((item, i) => (
              <ContentCard
                key={`${item.url}-${i}`}
                item={item}
                sourceName={sourceNameMap.get(item.sourceId) ?? item.sourceId}
              />
            ))}
          </div>
        )}

        {searchTerm && activeTab === SOURCE_TAB_ALL && hasNextPage && (
          <div className="mt-4 flex justify-center">
            <button
              onClick={() => fetchNextPage()}
              disabled={isFetchingNextPage}
              className="inline-flex items-center gap-2 rounded-lg border border-input bg-background px-6 py-2 text-sm font-medium hover:bg-accent disabled:opacity-50"
            >
              {isFetchingNextPage ? (
                <Loader2 className="h-4 w-4 animate-spin" />
              ) : (
                <ChevronDown className="h-4 w-4" />
              )}
              {isFetchingNextPage ? "Loading..." : "Load More"}
            </button>
          </div>
        )}

        {searchTerm && deduplicatedItems && deduplicatedItems.length === 0 && !isLoading && (
          <p className="text-muted-foreground">No results found for &ldquo;{searchTerm}&rdquo;</p>
        )}

        {!searchTerm && (!latest || latest.length === 0) && (
          <p className="text-muted-foreground">
            Enter a search term to discover music across your sources.
          </p>
        )}
      </div>
    </div>
  );
}

function ContentCard({ item, sourceName }: { item: ContentItem; sourceName: string }) {
  const [isFav, setIsFav] = useState(false);
  const [copied, setCopied] = useState(false);
  const [downloading, setDownloading] = useState(false);
  const player = usePlayer();

  useEffect(() => {
    if (item.url) {
      isFavorite(item.url, item.sourceId)
        .then(setIsFav)
        .catch(() => {});
    }
  }, [item.url, item.sourceId]);

  async function handleFavorite() {
    if (isFav) {
      // TODO: full remove needs listFavorites + filter by entity_id
    } else {
      await addFavorite("content", item.url, item.sourceId, item.title, item.imageUrl ?? undefined);
      setIsFav(true);
    }
  }

  async function handleDownload() {
    if (!item.downloadUrl) return;
    setDownloading(true);
    try {
      // Extract filename from title or URL
      const ext = "mp3";
      const safeName = item.title.replace(/[^a-zA-Z0-9\u4e00-\u9fff _-]/g, "").trim() || "track";
      const filename = `${safeName}.${ext}`;
      const path = await tauri.downloadAudio({ url: item.downloadUrl, filename });
      console.log("Downloaded to:", path);
    } catch (err) {
      console.error("Download failed:", err);
      // Fallback: open in browser
      await tauri.openUrl({ url: item.downloadUrl }).catch(() => {});
    } finally {
      setDownloading(false);
    }
  }

  async function handleCopyLink() {
    if (item.url) {
      try {
        await navigator.clipboard.writeText(item.url);
        setCopied(true);
        setTimeout(() => setCopied(false), 2000);
      } catch (err) {
        console.error("Failed to copy:", err);
      }
    }
  }

  const isCurrentTrack = player.current?.audioUrl === item.audioUrl;
  const isPlaying = isCurrentTrack && player.isPlaying;

  function formatDuration(seconds?: number): string {
    if (!seconds) return "";
    const m = Math.floor(seconds / 60);
    const s = seconds % 60;
    return `${m}:${s.toString().padStart(2, "0")}`;
  }

  const kindColor = item.sourceId === "discogs"
    ? "bg-destructive/15 text-destructive ring-1 ring-destructive/30"
    : item.sourceId === "jamendo"
    ? "bg-blue-500/15 text-blue-400 ring-1 ring-blue-500/30"
    : "bg-primary/15 text-primary ring-1 ring-primary/30";

  return (
    <div className="group flex gap-4 rounded-lg border border-border bg-card p-4 transition-colors hover:bg-accent">
      {item.imageUrl ? (
        <a href={item.url} target="_blank" rel="noopener noreferrer" className="relative flex-shrink-0">
          <img
            src={item.imageUrl}
            alt={item.title}
            className="h-24 w-24 rounded object-cover bg-secondary"
            loading="lazy"
          />
          {item.audioUrl && (
            <button
              onClick={(e) => {
                e.preventDefault();
                player.play(item);
              }}
              className="absolute inset-0 flex items-center justify-center rounded bg-black/40 opacity-0 transition-opacity group-hover:opacity-100"
            >
              {isPlaying ? (
                <Pause className="h-8 w-8 text-white" />
              ) : (
                <Play className="h-8 w-8 text-white ml-0.5" />
              )}
            </button>
          )}
        </a>
      ) : (
        <div className="flex h-24 w-24 flex-shrink-0 items-center justify-center rounded bg-secondary text-xs text-muted-foreground">
          {item.audioUrl ? (
            <button
              onClick={() => player.play(item)}
              className="flex h-full w-full items-center justify-center"
            >
              {isPlaying ? (
                <Pause className="h-8 w-8 text-primary" />
              ) : (
                <Play className="h-8 w-8 text-primary ml-0.5" />
              )}
            </button>
          ) : (
            "No image"
          )}
        </div>
      )}
      <div className="flex-1 min-w-0">
        <a
          href={item.url}
          target="_blank"
          rel="noopener noreferrer"
          className="flex items-start gap-2 group/title"
        >
          <h3 className="font-medium leading-tight group-hover/title:text-primary line-clamp-2">
            {item.title}
          </h3>
          <ExternalLink className="mt-0.5 h-3 w-3 flex-shrink-0 text-muted-foreground opacity-0 group-hover/title:opacity-100" />
        </a>
        {item.summary && (
          <p className="mt-1 line-clamp-2 text-sm text-muted-foreground">
            {item.summary}
          </p>
        )}
        <div className="mt-2 flex flex-wrap items-center gap-2">
          <span className={`inline-flex items-center rounded-full px-2 py-0.5 text-xs font-medium ${kindColor}`}>
            {sourceName}
          </span>
          {item.duration && (
            <span className="text-xs text-muted-foreground tabular-nums">
              {formatDuration(item.duration)}
            </span>
          )}
          {item.url && (
            <div className="inline-flex items-center gap-1">
              <a
                href={item.url}
                target="_blank"
                rel="noopener noreferrer"
                className="inline-flex items-center gap-1 text-xs text-primary/70 hover:text-primary hover:underline truncate max-w-[200px]"
                title={item.url}
              >
                <ExternalLink className="h-3 w-3 flex-shrink-0" />
                {new URL(item.url).hostname}
              </a>
              <button
                onClick={handleCopyLink}
                className="inline-flex items-center gap-1 text-xs text-muted-foreground hover:text-primary transition-colors"
                title="Copy link"
              >
                {copied ? (
                  <Check className="h-3 w-3 text-primary" />
                ) : (
                  <Copy className="h-3 w-3" />
                )}
              </button>
            </div>
          )}
          {item.downloadUrl && (
            <button
              onClick={handleDownload}
              disabled={downloading}
              className="inline-flex items-center gap-1 text-xs text-muted-foreground hover:text-primary transition-colors disabled:opacity-50"
              title="Download"
            >
              <Download className="h-3 w-3" />
              {downloading ? "Downloading..." : "Download"}
            </button>
          )}
          {item.author && (
            <span className="text-xs text-muted-foreground">{item.author}</span>
          )}
          {item.publishedAt && (
            <span className="text-xs text-muted-foreground">
              {formatDate(item.publishedAt)}
            </span>
          )}
          {item.license && (
            <a
              href={item.license}
              target="_blank"
              rel="noopener noreferrer"
              className="text-xs text-muted-foreground hover:text-primary hover:underline"
              title={item.license}
            >
              License
            </a>
          )}
          {item.extra && "rating" in item.extra && (
            <span className="text-xs font-medium text-destructive">
              {String(item.extra.rating)}
            </span>
          )}
          {item.extra && "year" in item.extra && (
            <span className="text-xs text-muted-foreground">
              {String(item.extra.year)}
            </span>
          )}
          {item.extra && "genres" in item.extra && Array.isArray(item.extra.genres) && (
            <span className="text-xs text-muted-foreground">
              {(item.extra.genres as string[]).join(", ")}
            </span>
          )}
        </div>
      </div>
      <button
        onClick={handleFavorite}
        className="flex-shrink-0 p-1 text-muted-foreground hover:text-primary"
        title={isFav ? "Remove from favorites" : "Add to favorites"}
      >
        <Star className={`h-4 w-4 ${isFav ? "fill-primary text-primary" : ""}`} />
      </button>
    </div>
  );
}

function formatDate(dateStr: string): string {
  try {
    const d = new Date(dateStr);
    if (isNaN(d.getTime())) return dateStr;
    return d.toLocaleDateString("en-US", { month: "short", day: "numeric", year: "numeric" });
  } catch {
    return dateStr;
  }
}
