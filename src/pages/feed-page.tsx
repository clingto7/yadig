import { useState } from "react";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { Plus, Trash2, Rss } from "lucide-react";
import { listFeeds, addFeed, removeFeed, type RssFeed } from "@/lib/db";

export function FeedPage() {
  const [newName, setNewName] = useState("");
  const [newUrl, setNewUrl] = useState("");
  const [showAdd, setShowAdd] = useState(false);
  const queryClient = useQueryClient();

  const { data: feeds, isLoading } = useQuery({
    queryKey: ["feeds"],
    queryFn: listFeeds,
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
    <div className="p-6">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-bold">RSS Feeds</h2>
          <p className="mt-1 text-sm text-muted-foreground">
            Add and manage your custom RSS feed sources
          </p>
        </div>
        <button
          onClick={() => setShowAdd(!showAdd)}
          className="flex items-center gap-2 rounded-lg bg-primary px-3 py-2 text-sm font-medium text-primary-foreground hover:bg-primary/90"
        >
          <Plus className="h-4 w-4" />
          Add Feed
        </button>
      </div>

      {showAdd && (
        <div className="mt-4 rounded-lg border border-border bg-card p-4">
          <h3 className="mb-3 text-sm font-medium">Add new RSS feed</h3>
          <div className="flex gap-2">
            <input
              type="text"
              value={newName}
              onChange={(e) => setNewName(e.target.value)}
              placeholder="Feed name (e.g., Pitchfork News)"
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

      {isLoading && <p className="mt-4 text-muted-foreground">Loading feeds...</p>}

      {feeds && feeds.length > 0 && (
        <div className="mt-4 grid gap-3">
          {feeds.map((feed: RssFeed) => (
            <div
              key={feed.id}
              className="flex items-center justify-between rounded-lg border border-border bg-card p-4"
            >
              <div className="flex items-center gap-3">
                <Rss className="h-4 w-4 text-primary" />
                <div>
                  <span className="font-medium">{feed.name}</span>
                  <p className="text-xs text-muted-foreground">{feed.url}</p>
                </div>
              </div>
              <div className="flex items-center gap-3">
                {feed.is_active ? (
                  <span className="flex items-center gap-1 text-xs text-green-500">
                    <span className="h-1.5 w-1.5 rounded-full bg-green-500" />
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
      )}

      {feeds && feeds.length === 0 && (
        <div className="mt-8 text-center">
          <Rss className="mx-auto h-12 w-12 text-muted-foreground/50" />
          <p className="mt-2 text-muted-foreground">
            No RSS feeds added yet. Click "Add Feed" to get started.
          </p>
        </div>
      )}
    </div>
  );
}
