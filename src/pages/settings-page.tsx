import { useState } from "react";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { Key, Eye, EyeOff } from "lucide-react";
import { Store } from "@tauri-apps/plugin-store";
import { tauri } from "@/lib/tauri";

export function SettingsPage() {
  const { data: sources } = useQuery({
    queryKey: ["sources"],
    queryFn: () => tauri.listSources().catch(() => []),
  });

  return (
    <div className="flex h-full flex-col">
      <header className="border-b border-border p-6">
        <h2 className="text-2xl font-bold">Settings</h2>
        <p className="mt-1 text-sm text-muted-foreground">
          Configure sources and API keys
        </p>
      </header>

      <div className="flex-1 overflow-y-auto p-6 space-y-8">
        <SourcesSection sources={sources} />
        <ApiKeysSection />
        <AboutSection />
      </div>
    </div>
  );
}

function SourcesSection({ sources }: { sources?: { id: string; name: string; kind: string; baseUrl: string; isActive: boolean }[] }) {
  const queryClient = useQueryClient();

  const toggleMutation = useMutation({
    mutationFn: async ({ id, enabled }: { id: string; enabled: boolean }) => {
      await tauri.setSourceEnabled({ id, enabled });
      const store = await Store.load("settings.json");
      const states = await store.get<Record<string, boolean>>("source_states") || {};
      states[id] = enabled;
      await store.set("source_states", states);
      await store.save();
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["sources"] });
    },
  });

  return (
    <section>
      <h3 className="text-lg font-semibold">Sources</h3>
      <p className="mt-1 text-sm text-muted-foreground">
        Music information sources currently available.
      </p>

      {sources && sources.length > 0 && (
        <div className="mt-4 grid gap-3">
          {sources.map((s) => (
            <div
              key={s.id}
              className="flex items-center justify-between rounded-lg border border-border bg-card p-4"
            >
              <div className="flex items-center gap-3">
                <div className="flex h-9 w-9 items-center justify-center rounded-lg bg-primary/15">
                  <span className="text-xs font-bold text-primary uppercase">
                    {s.name.slice(0, 2)}
                  </span>
                </div>
                <div>
                  <span className="font-medium">{s.name}</span>
                  <p className="text-xs text-muted-foreground">{s.baseUrl}</p>
                </div>
              </div>
              <div className="flex items-center gap-3">
                <span className={`inline-flex items-center rounded-full px-2 py-0.5 text-xs font-medium ${
                  s.kind === "rss" ? "bg-primary/15 text-primary" :
                  s.kind === "api" ? "bg-destructive/15 text-destructive" :
                  "bg-secondary text-secondary-foreground"
                }`}>
                  {s.kind}
                </span>
                <button
                  onClick={() => toggleMutation.mutate({ id: s.id, enabled: !s.isActive })}
                  disabled={toggleMutation.isPending}
                  className={`inline-flex items-center gap-1.5 rounded-full px-3 py-1 text-xs font-medium transition-colors ${
                    s.isActive
                      ? "bg-primary/15 text-primary hover:bg-primary/25"
                      : "bg-secondary text-muted-foreground hover:bg-secondary/80"
                  }`}
                >
                  <span className={`h-1.5 w-1.5 rounded-full ${
                    s.isActive ? "bg-primary" : "bg-muted-foreground"
                  }`} />
                  {s.isActive ? "Active" : "Inactive"}
                </button>
              </div>
            </div>
          ))}
        </div>
      )}

      {!sources && (
        <p className="mt-4 text-sm text-muted-foreground">Loading sources...</p>
      )}
    </section>
  );
}

function ApiKeysSection() {
  const [discogsKey, setDiscogsKey] = useState("");
  const [discogsSecret, setDiscogsSecret] = useState("");
  const [showKey, setShowKey] = useState(false);
  const [showSecret, setShowSecret] = useState(false);
  const [saved, setSaved] = useState(false);

  async function handleSave() {
    try {
      const store = await Store.load("settings.json");
      await store.set("discogs_key", discogsKey);
      await store.set("discogs_secret", discogsSecret);
      await store.save();
      await tauri.updateDiscogsKeys({ key: discogsKey, secret: discogsSecret });
      setSaved(true);
      setTimeout(() => setSaved(false), 2000);
    } catch (err) {
      console.error("Failed to save Discogs keys:", err);
    }
  }

  return (
    <section>
      <h3 className="text-lg font-semibold">API Keys</h3>
      <p className="mt-1 text-sm text-muted-foreground">
        Configure API keys for sources that require authentication.
      </p>

      <div className="mt-4 space-y-4">
        {/* Discogs */}
        <div className="rounded-lg border border-border bg-card p-4">
          <div className="flex items-center gap-2 mb-3">
            <Key className="h-4 w-4 text-muted-foreground" />
            <h4 className="font-medium">Discogs</h4>
            <span className="text-xs text-muted-foreground">(optional — higher rate limits)</span>
          </div>
          <div className="space-y-2">
            <div className="relative">
              <input
                type={showKey ? "text" : "password"}
                value={discogsKey}
                onChange={(e) => setDiscogsKey(e.target.value)}
                placeholder="Consumer Key"
                className="h-9 w-full rounded-md border border-input bg-background px-3 pr-9 text-sm placeholder:text-muted-foreground focus:border-ring focus:outline-none focus:ring-1 focus:ring-ring"
              />
              <button
                onClick={() => setShowKey(!showKey)}
                className="absolute right-2 top-1/2 -translate-y-1/2 text-muted-foreground hover:text-foreground"
              >
                {showKey ? <EyeOff className="h-4 w-4" /> : <Eye className="h-4 w-4" />}
              </button>
            </div>
            <div className="relative">
              <input
                type={showSecret ? "text" : "password"}
                value={discogsSecret}
                onChange={(e) => setDiscogsSecret(e.target.value)}
                placeholder="Consumer Secret"
                className="h-9 w-full rounded-md border border-input bg-background px-3 pr-9 text-sm placeholder:text-muted-foreground focus:border-ring focus:outline-none focus:ring-1 focus:ring-ring"
              />
              <button
                onClick={() => setShowSecret(!showSecret)}
                className="absolute right-2 top-1/2 -translate-y-1/2 text-muted-foreground hover:text-foreground"
              >
                {showSecret ? <EyeOff className="h-4 w-4" /> : <Eye className="h-4 w-4" />}
              </button>
            </div>
            <button
              onClick={handleSave}
              disabled={!discogsKey.trim() || !discogsSecret.trim()}
              className="h-9 rounded-md bg-primary px-4 text-sm font-medium text-primary-foreground hover:bg-primary/90 disabled:opacity-50"
            >
              {saved ? "Saved" : "Save"}
            </button>
          </div>
        </div>

        <div className="rounded-lg border border-border bg-card p-4">
          <div className="flex items-center gap-2 mb-2">
            <Key className="h-4 w-4 text-muted-foreground" />
            <h4 className="font-medium">LLM Integration</h4>
            <span className="text-xs text-muted-foreground">(Phase 2)</span>
          </div>
          <p className="text-sm text-muted-foreground">
            OpenAI / Anthropic / OpenRouter API keys for intelligent search and summarization.
          </p>
        </div>
      </div>
    </section>
  );
}

function AboutSection() {
  return (
    <section>
      <h3 className="text-lg font-semibold">About</h3>
      <div className="mt-4 rounded-lg border border-border bg-card p-4">
        <div className="flex items-center gap-3">
          <div className="flex h-10 w-10 items-center justify-center rounded-lg bg-primary/15">
            <span className="font-bold text-primary">Y</span>
          </div>
          <div>
            <p className="font-medium">yadig</p>
            <p className="text-xs text-muted-foreground">v0.1.0 — Music Discovery</p>
          </div>
        </div>
      </div>
    </section>
  );
}
