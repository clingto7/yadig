import { useQuery } from "@tanstack/react-query";
import { tauri } from "@/lib/tauri";

export function SettingsPage() {
  const { data: sources } = useQuery({
    queryKey: ["sources"],
    queryFn: () => tauri.listSources(),
  });

  return (
    <div className="p-6">
      <h2 className="text-2xl font-bold">Settings</h2>
      <p className="mt-1 text-sm text-muted-foreground">
        Configure API keys, sources, and preferences
      </p>

      <div className="mt-8">
        <h3 className="text-lg font-semibold">Active Sources</h3>
        <p className="mt-1 text-sm text-muted-foreground">
          Music information sources currently configured. More sources can be added as plugins.
        </p>

        {sources && sources.length > 0 && (
          <div className="mt-4 grid gap-3">
            {sources.map((s) => (
              <div
                key={s.id}
                className="flex items-center justify-between rounded-lg border border-border bg-card p-4"
              >
                <div>
                  <span className="font-medium">{s.name}</span>
                  <span className="ml-2 inline-flex items-center rounded-full bg-secondary px-2 py-0.5 text-xs text-secondary-foreground">
                    {s.kind}
                  </span>
                </div>
                <div className="flex items-center gap-2">
                  <span className="h-2 w-2 rounded-full bg-green-500" />
                  <span className="text-xs text-muted-foreground">Active</span>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>

      <div className="mt-8">
        <h3 className="text-lg font-semibold">API Keys</h3>
        <p className="mt-1 text-sm text-muted-foreground">
          Configure API keys for sources that require authentication (e.g., Discogs consumer key/secret).
        </p>
        <p className="mt-2 text-sm text-muted-foreground">
          API key management UI coming soon.
        </p>
      </div>

      <div className="mt-8">
        <h3 className="text-lg font-semibold">LLM Integration</h3>
        <p className="mt-1 text-sm text-muted-foreground">
          Configure LLM API keys for intelligent search and summarization (OpenAI / Anthropic / OpenRouter).
        </p>
        <p className="mt-2 text-sm text-muted-foreground">
          LLM configuration UI coming in Phase 2.
        </p>
      </div>
    </div>
  );
}
