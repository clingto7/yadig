import { useState, useEffect, useCallback } from "react";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { Key, Eye, EyeOff, LogIn, LogOut, QrCode, Cookie } from "lucide-react";
import { Store } from "@tauri-apps/plugin-store";
import { tauri } from "@/lib/tauri";
import type { LlmProviderTestError, LlmProviderTestErrorKind } from "@/lib/tauri";
import { clearPersistedBiliSession, savePersistedBiliSession } from "@/lib/bili-session-store";
import { biliAccountTierLabel, qrLoginUiState } from "@/lib/bili-login-ui";

const DEFAULT_LLM_PROVIDER = "openai-compatible";
const DEFAULT_LLM_BASE_URL = "https://token-plan-cn.xiaomimimo.com/v1";
const DEFAULT_LLM_MODEL = "mimo-v2.5-pro";
const LLM_TEST_ERROR_LABELS: Record<LlmProviderTestErrorKind, string> = {
  missing_config: "Missing config",
  auth: "Authentication failed",
  network: "Network error",
  incompatible_response: "Incompatible response",
  invalid_json: "Invalid JSON",
};

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
        <BiliLoginSection />
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
  const [llmProvider, setLlmProvider] = useState(DEFAULT_LLM_PROVIDER);
  const [llmBaseUrl, setLlmBaseUrl] = useState(DEFAULT_LLM_BASE_URL);
  const [llmModel, setLlmModel] = useState(DEFAULT_LLM_MODEL);
  const [llmApiKey, setLlmApiKey] = useState("");
  const [showKey, setShowKey] = useState(false);
  const [showSecret, setShowSecret] = useState(false);
  const [showLlmKey, setShowLlmKey] = useState(false);
  const [savedDiscogs, setSavedDiscogs] = useState(false);
  const [savedLlm, setSavedLlm] = useState(false);
  const [llmTestStatus, setLlmTestStatus] = useState<{
    state: "idle" | "testing" | "success" | "error";
    message: string;
  }>({ state: "idle", message: "" });

  useEffect(() => {
    (async () => {
      const store = await Store.load("settings.json");
      setDiscogsKey((await store.get<string>("discogs_key")) ?? "");
      setDiscogsSecret((await store.get<string>("discogs_secret")) ?? "");
      setLlmProvider((await store.get<string>("llm_provider")) ?? DEFAULT_LLM_PROVIDER);
      setLlmBaseUrl((await store.get<string>("llm_base_url")) ?? DEFAULT_LLM_BASE_URL);
      setLlmModel((await store.get<string>("llm_model")) ?? DEFAULT_LLM_MODEL);
      setLlmApiKey((await store.get<string>("llm_api_key")) ?? "");
    })().catch((err) => console.error("Failed to load LLM settings:", err));
  }, []);

  async function handleSaveDiscogs() {
    try {
      const store = await Store.load("settings.json");
      await store.set("discogs_key", discogsKey);
      await store.set("discogs_secret", discogsSecret);
      await store.save();
      await tauri.updateDiscogsKeys({ key: discogsKey, secret: discogsSecret });
      setSavedDiscogs(true);
      setTimeout(() => setSavedDiscogs(false), 2000);
    } catch (err) {
      console.error("Failed to save Discogs keys:", err);
    }
  }

  async function handleSaveLlm() {
    try {
      const store = await Store.load("settings.json");
      await store.set("llm_provider", llmProvider);
      await store.set("llm_base_url", llmBaseUrl);
      await store.set("llm_model", llmModel);
      await store.set("llm_api_key", llmApiKey);
      await store.save();
      setSavedLlm(true);
      setTimeout(() => setSavedLlm(false), 2000);
    } catch (err) {
      console.error("Failed to save LLM settings:", err);
    }
  }

  async function handleTestLlm() {
    setLlmTestStatus({ state: "testing", message: "Testing LLM provider..." });
    try {
      const result = await tauri.llmTestProvider({
        provider: llmProvider.trim(),
        baseUrl: llmBaseUrl.trim(),
        apiKey: llmApiKey,
        model: llmModel.trim(),
      });
      setLlmTestStatus({
        state: "success",
        message: result.usedResponseFormat
          ? `Connected to ${result.provider} with ${result.model}.`
          : `Connected to ${result.provider} with ${result.model}. JSON mode is not supported, so prompt-only JSON will be used.`,
      });
    } catch (err) {
      const error = err as Partial<LlmProviderTestError>;
      const prefix = error.kind ? `${LLM_TEST_ERROR_LABELS[error.kind]}: ` : "";
      setLlmTestStatus({
        state: "error",
        message: `${prefix}${error.message ?? String(err)}`,
      });
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
              onClick={handleSaveDiscogs}
              className="h-9 rounded-md bg-primary px-4 text-sm font-medium text-primary-foreground hover:bg-primary/90 disabled:opacity-50"
            >
              {savedDiscogs ? "Saved" : "Save Discogs"}
            </button>
          </div>
        </div>

        <div className="rounded-lg border border-border bg-card p-4">
          <div className="flex items-center gap-2 mb-3">
            <Key className="h-4 w-4 text-muted-foreground" />
            <h4 className="font-medium">LLM Integration</h4>
          </div>
          <div className="grid gap-2 md:grid-cols-2">
            <input
              type="text"
              value={llmProvider}
              onChange={(e) => setLlmProvider(e.target.value)}
              placeholder="Provider"
              className="h-9 rounded-md border border-input bg-background px-3 text-sm placeholder:text-muted-foreground focus:border-ring focus:outline-none focus:ring-1 focus:ring-ring"
            />
            <input
              type="text"
              value={llmModel}
              onChange={(e) => setLlmModel(e.target.value)}
              placeholder="Model"
              className="h-9 rounded-md border border-input bg-background px-3 text-sm placeholder:text-muted-foreground focus:border-ring focus:outline-none focus:ring-1 focus:ring-ring"
            />
            <input
              type="text"
              value={llmBaseUrl}
              onChange={(e) => setLlmBaseUrl(e.target.value)}
              placeholder="Base URL"
              className="h-9 rounded-md border border-input bg-background px-3 text-sm placeholder:text-muted-foreground focus:border-ring focus:outline-none focus:ring-1 focus:ring-ring md:col-span-2"
            />
            <div className="relative md:col-span-2">
              <input
                type={showLlmKey ? "text" : "password"}
                value={llmApiKey}
                onChange={(e) => setLlmApiKey(e.target.value)}
                placeholder="API Key"
                className="h-9 w-full rounded-md border border-input bg-background px-3 pr-9 text-sm placeholder:text-muted-foreground focus:border-ring focus:outline-none focus:ring-1 focus:ring-ring"
              />
              <button
                onClick={() => setShowLlmKey(!showLlmKey)}
                className="absolute right-2 top-1/2 -translate-y-1/2 text-muted-foreground hover:text-foreground"
              >
                {showLlmKey ? <EyeOff className="h-4 w-4" /> : <Eye className="h-4 w-4" />}
              </button>
            </div>
          </div>
          <p className="mt-2 text-xs text-muted-foreground">
            Used by the media workstation for metadata classification and batch-operation suggestions.
          </p>
          <div className="mt-3 flex flex-wrap gap-2">
            <button
              onClick={handleSaveLlm}
              disabled={!llmProvider.trim() || !llmBaseUrl.trim() || !llmModel.trim()}
              className="h-9 rounded-md bg-primary px-4 text-sm font-medium text-primary-foreground hover:bg-primary/90 disabled:opacity-50"
            >
              {savedLlm ? "Saved" : "Save LLM"}
            </button>
            <button
              onClick={handleTestLlm}
              disabled={
                llmTestStatus.state === "testing"
                || !llmProvider.trim()
                || !llmBaseUrl.trim()
                || !llmModel.trim()
                || !llmApiKey.trim()
              }
              className="h-9 rounded-md border border-border px-4 text-sm font-medium hover:bg-secondary disabled:opacity-50"
            >
              {llmTestStatus.state === "testing" ? "Testing LLM" : "Test LLM"}
            </button>
          </div>
          {llmTestStatus.state !== "idle" && (
            <p className={`mt-2 text-xs ${
              llmTestStatus.state === "error" ? "text-destructive" : "text-muted-foreground"
            }`}>
              {llmTestStatus.message}
            </p>
          )}
        </div>
      </div>
    </section>
  );
}

function BiliLoginSection() {
  const [status, setStatus] = useState<{ loggedIn: boolean; username: string | null; isPremium: boolean } | null>(null);
  const [loading, setLoading] = useState(true);
  const [qrUrl, setQrUrl] = useState<string | null>(null);
  const [qrKey, setQrKey] = useState<string | null>(null);
  const [qrStatus, setQrStatus] = useState<string>("");
  const [qrImg, setQrImg] = useState<string | null>(null);
  const [qrExpired, setQrExpired] = useState(false);
  const [showCookieInput, setShowCookieInput] = useState(false);
  const [showPasswordInput, setShowPasswordInput] = useState(false);
  const [sessdata, setSessdata] = useState("");
  const [biliUsername, setBiliUsername] = useState("");
  const [biliPassword, setBiliPassword] = useState("");
  const [error, setError] = useState<string | null>(null);

  const checkStatus = useCallback(async () => {
    try {
      const s = await tauri.biliSessionStatus();
      setStatus(s);
      if (!s.loggedIn) {
        await clearPersistedBiliSession();
      }
    } catch {
      setStatus({ loggedIn: false, username: null, isPremium: false });
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => { checkStatus(); }, [checkStatus]);

  // Poll QR login status
  useEffect(() => {
    if (!qrKey) return;
    const interval = setInterval(async () => {
      try {
        const resp = await tauri.biliQrLoginPoll({ qrcodeKey: qrKey });
        if (resp.code === 0) {
          const ui = qrLoginUiState(resp.code);
          if (resp.session) {
            await savePersistedBiliSession(resp.session);
          }
          setQrStatus(ui.message);
          setQrExpired(ui.expired);
          setQrUrl(null);
          setQrKey(null);
          setQrImg(null);
          await checkStatus();
          clearInterval(interval);
        } else if (resp.code === 86090) {
          const ui = qrLoginUiState(resp.code);
          setQrStatus(ui.message);
          setQrExpired(ui.expired);
        } else if (resp.code === 86038) {
          const ui = qrLoginUiState(resp.code);
          setQrStatus(ui.message);
          setQrExpired(ui.expired);
          clearInterval(interval);
        } else {
          const ui = qrLoginUiState(resp.code);
          setQrStatus(ui.message);
          setQrExpired(ui.expired);
        }
      } catch (e) {
        setQrStatus(`Error: ${e}`);
        setQrExpired(false);
        clearInterval(interval);
      }
    }, 2000);
    return () => clearInterval(interval);
  }, [qrKey, checkStatus]);

  async function startQrLogin() {
    setError(null);
    setQrExpired(false);
    setQrStatus("Generating QR code...");
    setQrImg(null);
    try {
      const resp = await tauri.biliQrLoginStart();
      setQrUrl(resp.url);
      setQrKey(resp.qrcodeKey);
      setQrStatus("Scan with Bilibili app");

      // Generate QR code locally using qrcode library
      const QRCode = (await import("qrcode"));
      const dataUrl = await QRCode.toDataURL(resp.url, {
        width: 200,
        margin: 2,
        color: { dark: "#000", light: "#fff" },
      });
      setQrImg(dataUrl);
    } catch (e) {
      setError(`Failed to start QR login: ${e}`);
    }
  }

  async function handleCookieLogin() {
    setError(null);
    try {
      const session = await tauri.biliCookieLogin({ sessdata: sessdata.trim() });
      await savePersistedBiliSession(session);
      setSessdata("");
      setShowCookieInput(false);
      await checkStatus();
    } catch (e) {
      setError(`Cookie login failed: ${e}`);
    }
  }

  async function handlePasswordLogin() {
    setError(null);
    try {
      const session = await tauri.biliPasswordLogin({ username: biliUsername.trim(), password: biliPassword });
      await savePersistedBiliSession(session);
      setBiliUsername("");
      setBiliPassword("");
      setShowPasswordInput(false);
      await checkStatus();
    } catch (e) {
      setError(`Password login failed: ${e}`);
    }
  }

  async function handleLogout() {
    try {
      await tauri.biliLogout();
      await clearPersistedBiliSession();
      await checkStatus();
    } catch (e) {
      setError(`Logout failed: ${e}`);
    }
  }

  if (loading) {
    return (
      <section>
        <h3 className="text-lg font-semibold">Bilibili</h3>
        <p className="mt-1 text-sm text-muted-foreground">Loading...</p>
      </section>
    );
  }

  return (
    <section>
      <h3 className="text-lg font-semibold">Bilibili</h3>
      <p className="mt-1 text-sm text-muted-foreground">
        Login to access higher quality audio streams (192K+ requires login).
      </p>

      <div className="mt-4 rounded-lg border border-border bg-card p-4">
        {status?.loggedIn ? (
          <div className="space-y-3">
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-2">
                <LogIn className="h-4 w-4 text-primary" />
                <span className="font-medium">{status.username ?? "Logged in"}</span>
                <span className={`rounded-full px-2 py-0.5 text-xs font-medium ${
                  status.isPremium
                    ? "bg-primary/15 text-primary"
                    : "bg-secondary text-secondary-foreground"
                }`}>
                  {biliAccountTierLabel(status.isPremium)}
                </span>
              </div>
              <button
                onClick={handleLogout}
                className="inline-flex items-center gap-1.5 rounded-md border border-border px-3 py-1.5 text-sm text-muted-foreground hover:bg-secondary"
              >
                <LogOut className="h-3.5 w-3.5" />
                Logout
              </button>
            </div>
            <p className="text-xs text-muted-foreground">
              {status.isPremium
                ? "Max quality: Hi-Res / Dolby Atmos"
                : "Max quality: 192K. Upgrade to Premium for Hi-Res."}
            </p>
          </div>
        ) : (
          <div className="space-y-3">
            <div className="flex items-center gap-2">
              <LogIn className="h-4 w-4 text-muted-foreground" />
              <span className="text-muted-foreground">Not logged in</span>
              <span className="text-xs text-muted-foreground">(max 64K audio)</span>
            </div>

            <div className="flex flex-wrap gap-2">
              <button
                onClick={startQrLogin}
                className="inline-flex items-center gap-1.5 rounded-md bg-primary px-3 py-1.5 text-sm font-medium text-primary-foreground hover:bg-primary/90"
              >
                <QrCode className="h-3.5 w-3.5" />
                Login with QR Code
              </button>
              <button
                onClick={() => setShowPasswordInput(!showPasswordInput)}
                className="inline-flex items-center gap-1.5 rounded-md border border-border px-3 py-1.5 text-sm text-muted-foreground hover:bg-secondary"
              >
                <LogIn className="h-3.5 w-3.5" />
                Account & Password
              </button>
              <button
                onClick={() => setShowCookieInput(!showCookieInput)}
                className="inline-flex items-center gap-1.5 rounded-md border border-border px-3 py-1.5 text-sm text-muted-foreground hover:bg-secondary"
              >
                <Cookie className="h-3.5 w-3.5" />
                Cookie Login
              </button>
            </div>

            {qrImg && (
              <div className="rounded-md border border-border bg-background p-4 text-center">
                <img
                  src={qrImg}
                  alt="Bilibili Login QR Code"
                  className="mx-auto h-48 w-48"
                />
                <p className="mt-2 text-sm text-muted-foreground">{qrStatus}</p>
                {qrExpired && (
                  <button
                    onClick={startQrLogin}
                    className="mt-3 inline-flex items-center gap-1.5 rounded-md border border-border px-3 py-1.5 text-sm text-muted-foreground hover:bg-secondary"
                  >
                    <QrCode className="h-3.5 w-3.5" />
                    Refresh QR Code
                  </button>
                )}
                <p className="mt-1 text-xs text-muted-foreground break-all">
                  <a href={qrUrl!} target="_blank" rel="noopener noreferrer"
                     className="hover:text-primary underline">
                    Open link in browser
                  </a> if QR code doesn't work
                </p>
              </div>
            )}

            {showPasswordInput && (
              <div className="space-y-2">
                <input
                  type="text"
                  value={biliUsername}
                  onChange={(e) => setBiliUsername(e.target.value)}
                  placeholder="Bilibili username / phone"
                  className="h-9 w-full rounded-md border border-input bg-background px-3 text-sm placeholder:text-muted-foreground focus:border-ring focus:outline-none focus:ring-1 focus:ring-ring"
                />
                <input
                  type="password"
                  value={biliPassword}
                  onChange={(e) => setBiliPassword(e.target.value)}
                  placeholder="Password"
                  className="h-9 w-full rounded-md border border-input bg-background px-3 text-sm placeholder:text-muted-foreground focus:border-ring focus:outline-none focus:ring-1 focus:ring-ring"
                />
                <button
                  onClick={handlePasswordLogin}
                  disabled={!biliUsername.trim() || !biliPassword.trim()}
                  className="h-9 rounded-md bg-primary px-4 text-sm font-medium text-primary-foreground hover:bg-primary/90 disabled:opacity-50"
                >
                  Login
                </button>
              </div>
            )}

            {showCookieInput && (
              <div className="space-y-2">
                <input
                  type="text"
                  value={sessdata}
                  onChange={(e) => setSessdata(e.target.value)}
                  placeholder="Paste full Cookie header or SESSDATA value"
                  className="h-9 w-full rounded-md border border-input bg-background px-3 text-sm placeholder:text-muted-foreground focus:border-ring focus:outline-none focus:ring-1 focus:ring-ring"
                />
                <button
                  onClick={handleCookieLogin}
                  disabled={!sessdata.trim()}
                  className="h-9 rounded-md bg-primary px-4 text-sm font-medium text-primary-foreground hover:bg-primary/90 disabled:opacity-50"
                >
                  Login
                </button>
              </div>
            )}
          </div>
        )}

        {error && (
          <p className="mt-2 text-sm text-destructive">{error}</p>
        )}
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
            <p className="text-xs text-muted-foreground">v0.1.0 — Personal Media Workstation</p>
          </div>
        </div>
      </div>
    </section>
  );
}
