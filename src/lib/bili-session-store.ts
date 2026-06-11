import { Store } from "@tauri-apps/plugin-store";
import type { BiliSession } from "@/lib/tauri";

export const BILI_SESSION_KEY = "bilibili_session";
export const LEGACY_BILI_SESSION_KEY = "bili_session";

async function settingsStore() {
  return Store.load("settings.json");
}

function isBiliSession(value: unknown): value is BiliSession {
  if (!value || typeof value !== "object") return false;
  const candidate = value as Partial<Record<keyof BiliSession, unknown>>;
  return typeof candidate.sessdata === "string"
    && typeof candidate.biliJct === "string"
    && typeof candidate.dedeUserId === "string"
    && typeof candidate.vipStatus === "number";
}

export async function loadPersistedBiliSession(): Promise<BiliSession | null> {
  const store = await settingsStore();
  const session = await store.get<unknown>(BILI_SESSION_KEY);
  if (isBiliSession(session) && session.sessdata.trim()) {
    return session;
  }

  const legacySession = await store.get<unknown>(LEGACY_BILI_SESSION_KEY);
  if (isBiliSession(legacySession) && legacySession.sessdata.trim()) {
    await store.set(BILI_SESSION_KEY, legacySession);
    await store.delete(LEGACY_BILI_SESSION_KEY);
    await store.save();
    return legacySession;
  }

  return null;
}

export async function savePersistedBiliSession(session: BiliSession): Promise<void> {
  const store = await settingsStore();
  await store.set(BILI_SESSION_KEY, {
    sessdata: session.sessdata,
    biliJct: session.biliJct,
    dedeUserId: session.dedeUserId,
    vipStatus: session.vipStatus,
  });
  await store.delete(LEGACY_BILI_SESSION_KEY);
  await store.save();
}

export async function clearPersistedBiliSession(): Promise<void> {
  const store = await settingsStore();
  await store.delete(BILI_SESSION_KEY);
  await store.delete(LEGACY_BILI_SESSION_KEY);
  await store.save();
}
