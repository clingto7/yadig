import { BILI_SESSION_KEY, LEGACY_BILI_SESSION_KEY } from "@/lib/bili-session-store";
import type { BiliSession } from "@/lib/tauri";

if (BILI_SESSION_KEY !== "bilibili_session") {
  throw new Error("Bilibili session persistence must use the documented store key.");
}

if (LEGACY_BILI_SESSION_KEY !== "bili_session") {
  throw new Error("Legacy Bilibili session key must stay available for migration.");
}

const persistedSession: BiliSession = {
  sessdata: "sess",
  biliJct: "csrf",
  dedeUserId: "42",
  vipStatus: 1,
};

if (!persistedSession.sessdata || !persistedSession.biliJct || !persistedSession.dedeUserId) {
  throw new Error("Persisted Bilibili sessions must preserve write-capable cookie fields.");
}
