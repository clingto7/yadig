import { useState } from "react";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { Search as SearchIcon, Loader2, ExternalLink, Star, Clock } from "lucide-react";
import { tauri } from "@/lib/tauri";
import { saveSearch, listSearches, addFavorite, isFavorite } from "@/lib/db";
import type { ContentItem } from "@/types/source";

export function SearchPage() {
  const [query, setQuery] = useState("");
  const [searchTerm, setSearchTerm] = useState("");
  const [showHistory, setShowHistory] = useState(false);
  const queryClient = useQueryClient();

  const { data: sources } = useQuery({
    queryKey: ["sources"],
    queryFn: () => tauri.listSources(),
  });

  const { data: results, isLoading } = useQuery({
    queryKey: ["search", searchTerm],
    queryFn: () => tauri.searchSources({ query: searchTerm }),
    enabled: searchTerm.length > 0,
  });

  const { data: latest } = useQuery({
    queryKey: ["latest"],
    queryFn: () => tauri.fetchLatest({ limit: 10 }),
  });

  const { data: history } = useQuery({
    queryKey: ["searchHistory"],
    queryFn: () => listSearches(10),
  });

  const saveSearchMutation = useMutation({
    mutationFn: () =>
      saveSearch(
        searchTerm,
        results?.totalResults ?? 0,
        sources?.map((s) => s.id).join(",") ?? ""
      ),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ["searchHistory"] }),
  });

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

  // Save search when results come in
  const hasSaved = useState(false);
  if (results && searchTerm && !hasSaved[0]) {
    hasSaved[1](true);
    saveSearchMutation.mutate();
  }
  if (!searchTerm) hasSaved[1](false);

  return (
    <div className="flex h-full flex-col">
      <header className="border-b border-border p-6">
        <h2 className="text-2xl font-bold">Discover</h2>
        <p className="mt-1 text-sm text-muted-foreground">
          Search across music sources — Pitchfork, Discogs, Bandcamp, Album of the Year
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

        {sources && sources.length > 0 && (
          <div className="mt-3 flex flex-wrap gap-2">
            {sources.map((s) => (
              <span
                key={s.id}
                className="inline-flex items-center rounded-full bg-secondary px-2.5 py-0.5 text-xs font-medium text-secondary-foreground"
              >
                {s.name}
                <span className="ml-1 text-muted-foreground">({s.kind})</span>
              </span>
            ))}
          </div>
        )}
      </header>

      <div className="flex-1 overflow-y-auto p-6">
        {isLoading && (
          <div className="flex items-center gap-2 text-muted-foreground">
            <Loader2 className="h-4 w-4 animate-spin" />
            Searching...
          </div>
        )}

        {results && results.items.length > 0 && (
          <div>
            <p className="mb-4 text-sm text-muted-foreground">
              {results.totalResults} results in {results.elapsedMs}ms
            </p>
            <div className="grid gap-3">
              {results.items.map((item, i) => (
                <ContentCard key={`${item.sourceId}-${i}`} item={item} />
              ))}
            </div>
          </div>
        )}

        {results && results.items.length === 0 && searchTerm && (
          <p className="text-muted-foreground">No results found for "{searchTerm}"</p>
        )}

        {!searchTerm && latest && latest.length > 0 && (
          <div>
            <h3 className="mb-4 text-lg font-semibold">Latest</h3>
            <div className="grid gap-3">
              {latest.map((item, i) => (
                <ContentCard key={`${item.sourceId}-${i}`} item={item} />
              ))}
            </div>
          </div>
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

function ContentCard({ item }: { item: ContentItem }) {
  const [favChecked, setFavChecked] = useState(false);
  const [isFav, setIsFav] = useState(false);

  // Check if this item is favorited (lazy, once)
  if (!favChecked && item.url) {
    setFavChecked(true);
    isFavorite(item.url, item.sourceId).then(setIsFav);
  }

  async function handleFavorite() {
    if (isFav) {
      // For simplicity, we'd need the favorite ID to remove
      // This is a placeholder — full remove needs listFavorites + filter
    } else {
      await addFavorite("content", item.url, item.sourceId, item.title, item.imageUrl ?? undefined);
      setIsFav(true);
    }
  }

  return (
    <div className="group flex gap-4 rounded-lg border border-border bg-card p-4 transition-colors hover:bg-accent">
      {item.imageUrl && (
        <img
          src={item.imageUrl}
          alt={item.title}
          className="h-16 w-16 rounded object-cover"
        />
      )}
      <div className="flex-1 min-w-0">
        <a
          href={item.url}
          target="_blank"
          rel="noopener noreferrer"
          className="flex items-start gap-2"
        >
          <h3 className="font-medium leading-tight group-hover:text-primary">
            {item.title}
          </h3>
          <ExternalLink className="mt-0.5 h-3 w-3 flex-shrink-0 text-muted-foreground opacity-0 group-hover:opacity-100" />
        </a>
        {item.summary && (
          <p className="mt-1 line-clamp-2 text-sm text-muted-foreground">
            {item.summary}
          </p>
        )}
        <div className="mt-2 flex items-center gap-2">
          <span className="inline-flex items-center rounded-full bg-primary/10 px-2 py-0.5 text-xs font-medium text-primary">
            {item.sourceId}
          </span>
          {item.author && (
            <span className="text-xs text-muted-foreground">{item.author}</span>
          )}
          {item.publishedAt && (
            <span className="text-xs text-muted-foreground">{item.publishedAt}</span>
          )}
          {item.extra && "rating" in item.extra && (
            <span className="text-xs font-medium text-primary">
              {String(item.extra.rating)}
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
