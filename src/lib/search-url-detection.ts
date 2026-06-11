const BILI_URL_RE = /(?:bilibili\.com\/video\/BV|b23\.tv\/|space\.bilibili\.com\/.*collectiondetail)/i;
const YOUTUBE_URL_RE = /(?:youtube\.com\/watch\?v=|youtu\.be\/|youtube\.com\/embed\/)/i;

export function isBiliExtractableUrl(value: string): boolean {
  return BILI_URL_RE.test(value.trim());
}

export function isYoutubeExtractableUrl(value: string): boolean {
  return YOUTUBE_URL_RE.test(value.trim());
}
