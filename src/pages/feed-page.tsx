import { useState } from "react";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { Plus, Trash2, Rss, Loader2, ExternalLink, Copy, Check } from "lucide-react";
import { listFeeds, addFeed, removeFeed, type RssFeed } from "@/lib/db";
import { tauri } from "@/lib/tauri";
import type { ContentItem } from "@/types/source";

export function FeedPage() {
  const [newName, setNewName] = useState("");
  const [newUrl, setNewUrl] = useState("");
  const [showAdd, setShowAdd] = useState(false);
  const queryClient = useQueryClient();

  const { data: feeds, isLoading: feedsLoading } = useQuery({
    queryKey: ["feeds"],
    queryFn: () => listFeeds().catch(() => []),
  });

  const { data: latest, isLoading: latestLoading } = useQuery({
    queryKey: ["latest"],
    queryFn: () => tauri.fetchLatest({ limit: 20 }).catch(() => []),
  });

  const addMutation = useMutation({
    mutationFn: () => addFeed(newName.trim(), newUrl.trim()),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["feeds"] });
      setNewName("");
      setNewUrl("");
      setShowAdd(false);
    },
  });

  const removeMutation = useMutation({
    mutationFn: (id: number) => removeFeed(id),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ["feeds"] }),
  });

  return (
    <div className="flex h-full flex-col">
      <header className="flex items-center justify-between border-b border-border p-6">
        <div>
          <h2 className="text-2xl font-bold">Feed</h2>
          <p className="mt-1 text-sm text-muted-foreground">
            Latest from your music sources
          </p>
        </div>
        <div className="flex gap-2">
          <button
            onClick={() => setShowAdd(!showAdd)}
            className="flex items-center gap-2 rounded-lg bg-primary px-3 py-2 text-sm font-medium text-primary-foreground hover:bg-primary/90"
          >
            <Plus className="h-4 w-4" />
            Add RSS
          </button>
        </div>
      </header>

      {showAdd && (
        <div className="border-b border-border p-4">
          <div className="flex gap-2">
            <input
              type="text"
              value={newName}
              onChange={(e) => setNewName(e.target.value)}
              placeholder="Feed name"
              className="h-9 flex-1 rounded-md border border-input bg-background px-3 text-sm placeholder:text-muted-foreground focus:border-ring focus:outline-none focus:ring-1 focus:ring-ring"
            />
            <input
              type="url"
              value={newUrl}
              onChange={(e) => setNewUrl(e.target.value)}
              placeholder="RSS feed URL"
              className="h-9 flex-[2] rounded-md border border-input bg-background px-3 text-sm placeholder:text-muted-foreground focus:border-ring focus:outline-none focus:ring-1 focus:ring-ring"
            />
            <button
              onClick={() => addMutation.mutate()}
              disabled={!newName.trim() || !newUrl.trim()}
              className="h-9 rounded-md bg-primary px-4 text-sm font-medium text-primary-foreground hover:bg-primary/90 disabled:opacity-50"
            >
              Add
            </button>
          </div>
        </div>
      )}

      <div className="flex-1 overflow-y-auto p-6">
        {latestLoading && (
          <div className="flex items-center gap-2 text-muted-foreground">
            <Loader2 className="h-4 w-4 animate-spin" />
            Loading latest...
          </div>
        )}

        {latest && latest.length > 0 && (
          <div className="grid gap-3">
            {latest.map((item, i) => (
              <FeedCard key={`${item.sourceId}-${i}`} item={item} />
            ))}
          </div>
        )}

        {latest && latest.length === 0 && !latestLoading && (
          <p className="text-muted-foreground">No recent content from your sources.</p>
        )}

        {/* Custom RSS feeds management */}
        {feeds && feeds.length > 0 && (
          <div className="mt-8">
            <h3 className="mb-4 text-lg font-semibold">Custom RSS Feeds</h3>
            <div className="grid gap-2">
              {feeds.map((feed: RssFeed) => (
                <div
                  key={feed.id}
                  className="flex items-center justify-between rounded-lg border border-border bg-card px-4 py-3"
                >
                  <div className="flex items-center gap-3">
                    <Rss className="h-4 w-4 text-primary" />
                    <div>
                      <span className="text-sm font-medium">{feed.name}</span>
                      <p className="text-xs text-muted-foreground">{feed.url}</p>
                    </div>
                  </div>
                  <div className="flex items-center gap-3">
                    {feed.is_active ? (
                      <span className="flex items-center gap-1 text-xs text-primary">
                        <span className="h-1.5 w-1.5 rounded-full bg-primary" />
                        Active
                      </span>
                    ) : (
                      <span className="text-xs text-muted-foreground">Paused</span>
                    )}
                    <button
                      onClick={() => removeMutation.mutate(feed.id)}
                      className="p-1 text-muted-foreground hover:text-destructive"
                      title="Remove feed"
                    >
                      <Trash2 className="h-4 w-4" />
                    </button>
                  </div>
                </div>
              ))}
            </div>
          </div>
        )}

        {feedsLoading && (
          <div className="mt-8 flex items-center gap-2 text-muted-foreground">
            <Loader2 className="h-4 w-4 animate-spin" />
            Loading feeds...
          </div>
        )}
      </div>
    </div>
  );
}

function FeedCard({ item }: { item: ContentItem }) {
  const [copied, setCopied] = useState(false);

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

  return (
    <div className="group flex gap-4 rounded-lg border border-border bg-card p-4 transition-colors hover:bg-accent">
      {item.imageUrl ? (
        <a href={item.url} target="_blank" rel="noopener noreferrer">
          <img
            src={item.imageUrl}
            alt={item.title}
            className="h-20 w-20 flex-shrink-0 rounded object-cover bg-secondary"
          />
        </a>
      ) : (
        <div className="flex h-20 w-20 flex-shrink-0 items-center justify-center rounded bg-secondary text-xs text-muted-foreground">
          <Rss className="h-5 w-5" />
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
          <span className="inline-flex items-center rounded-full bg-primary/15 px-2 py-0.5 text-xs font-medium text-primary ring-1 ring-primary/30">
            {item.sourceId}
          </span>
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
          {item.author && (
            <span className="text-xs text-muted-foreground">{item.author}</span>
          )}
          {item.publishedAt && (
            <span className="text-xs text-muted-foreground">
              {formatDate(item.publishedAt)}
            </span>
          )}
        </div>
      </div>
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
