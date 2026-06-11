import { useEffect, useState } from "react";
import { Routes, Route, Navigate } from "react-router-dom";
import { Store } from "@tauri-apps/plugin-store";
import { invoke } from "@tauri-apps/api/core";
import { AppLayout } from "@/components/layout/app-layout";
import { SearchPage } from "@/pages/search-page";
import { FeedPage } from "@/pages/feed-page";
import { ChatPage } from "@/pages/chat-page";
import { SettingsPage } from "@/pages/settings-page";
import { WorkstationPage } from "@/pages/workstation-page";
import { ErrorBoundary } from "@/components/error-boundary";
import { clearPersistedBiliSession, loadPersistedBiliSession } from "@/lib/bili-session-store";

export default function App() {
  const [settingsLoaded, setSettingsLoaded] = useState(false);

  useEffect(() => {
    (async () => {
      try {
        const store = await Store.load("settings.json");

        // Restore Discogs keys
        const key = await store.get<string>("discogs_key");
        const secret = await store.get<string>("discogs_secret");
        if (key || secret) {
          await invoke("update_discogs_keys", {
            key: key || "",
            secret: secret || "",
          });
        }

        // Restore source enable states
        const sourceStates = await store.get<Record<string, boolean>>("source_states");
        if (sourceStates) {
          await Promise.all(
            Object.entries(sourceStates).map(([id, enabled]) =>
              invoke("set_source_enabled", { id, enabled })
            )
          );
        }

        // Restore Bilibili session into the Rust auth state before pages query status.
        const biliSession = await loadPersistedBiliSession();
        if (biliSession) {
          try {
            await invoke("bili_restore_session", { session: biliSession });
            try {
              const status = await invoke<{ loggedIn: boolean }>("bili_session_status");
              if (!status.loggedIn) {
                await clearPersistedBiliSession();
              }
            } catch (err) {
              console.error("Failed to verify restored Bilibili session:", err);
            }
          } catch (err) {
            await clearPersistedBiliSession();
            console.error("Failed to restore Bilibili session:", err);
          }
        }
      } catch (err) {
        console.error("Failed to load settings from store:", err);
      } finally {
        setSettingsLoaded(true);
      }
    })();
  }, []);

  if (!settingsLoaded) {
    return (
      <div className="flex h-screen items-center justify-center text-sm text-muted-foreground">
        Loading settings...
      </div>
    );
  }

  return (
    <ErrorBoundary>
      <Routes>
        <Route element={<AppLayout />}>
          <Route index element={<Navigate to="/search" replace />} />
          <Route
            path="/search"
            element={
              <ErrorBoundary>
                <SearchPage />
              </ErrorBoundary>
            }
          />
          <Route
            path="/chat"
            element={
              <ErrorBoundary>
                <ChatPage />
              </ErrorBoundary>
            }
          />
          <Route
            path="/feed"
            element={
              <ErrorBoundary>
                <FeedPage />
              </ErrorBoundary>
            }
          />
          <Route
            path="/workstation"
            element={
              <ErrorBoundary>
                <WorkstationPage />
              </ErrorBoundary>
            }
          />
          <Route
            path="/settings"
            element={
              <ErrorBoundary>
                <SettingsPage />
              </ErrorBoundary>
            }
          />
        </Route>
      </Routes>
    </ErrorBoundary>
  );
}
