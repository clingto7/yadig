import { isBiliExtractableUrl, isYoutubeExtractableUrl } from "@/lib/search-url-detection";

if (!isBiliExtractableUrl("https://www.bilibili.com/video/BV1GJ411x7h7")) {
  throw new Error("Bilibili video URLs should trigger extraction mode.");
}

if (!isBiliExtractableUrl("https://b23.tv/abcdef")) {
  throw new Error("Bilibili short links should trigger extraction mode.");
}

if (!isBiliExtractableUrl("https://space.bilibili.com/37737161/channel/collectiondetail?sid=1227671")) {
  throw new Error("Bilibili collection URLs should trigger extraction mode.");
}

if (isBiliExtractableUrl("https://example.com/video/BV1GJ411x7h7")) {
  throw new Error("Non-Bilibili URLs should not trigger Bilibili extraction mode.");
}

if (!isYoutubeExtractableUrl("https://youtu.be/dQw4w9WgXcQ")) {
  throw new Error("YouTube short links should still trigger YouTube extraction mode.");
}
