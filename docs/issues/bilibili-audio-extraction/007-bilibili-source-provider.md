# Issue 7: Bilibili as SourceProvider in unified search

## What to build

Implement `SourceProvider` for Bilibili so it appears in yadig's unified search alongside Discogs, Pitchfork, Bandcamp, etc.

1. **`BiliSource` struct** — Implements `SourceProvider` trait:
   - `id()`: `"bilibili"`
   - `name()`: `"Bilibili"`
   - `kind()`: `SourceKind::Api`
   - `base_url()`: `"https://www.bilibili.com"`
   - `search()`: Uses Bilibili's `/x/web-interface/wbi/search/type` with `search_type=video`. Maps results to `ContentItem` with `source_id: "bilibili"`, `title`, `url`, `image_url` (thumbnail), `author` (UP主 name), `duration`.
   - `fetch_latest()`: Uses Bilibili music zone ranking API or returns empty initially.

2. **Lazy audio stream** — Search results do NOT pre-fetch audio URLs (too slow). Instead, `ContentItem.audio_url` is left `None` during search. Audio URLs are fetched on-demand when the user clicks play/download via a separate `bili_get_playurl(bvid, cid)` command.

3. **Registration** — Register `BiliSource` in `lib.rs` alongside other sources. Bilibili source is enabled by default but can be toggled in settings.

4. **Music relevance** — Optionally add music-related keyword boosting (e.g., "MV", "live", "cover", "翻唱", "现场") in the search query or result filtering.

## Acceptance criteria

- [x] Bilibili appears in `list_sources` output
- [x] `search_sources` with Bilibili enabled returns Bilibili results alongside other sources
- [x] Each result has correct title, URL, thumbnail, author, duration
- [x] Toggling Bilibili off in settings excludes it from search
- [ ] Search completes within reasonable time (< 3s)
- [x] `cargo check` passes

## Implementation notes

- `BiliSource` is registered in `src-tauri/src/lib.rs` with id `bilibili`, name `Bilibili`, kind `api`, and base URL `https://www.bilibili.com`.
- Search uses Bilibili's video search endpoint and maps title, BVID URL, thumbnail, author, duration, `bvid`, and `cid` into `ContentItem`.
- Search results intentionally leave `audio_url` and `download_url` empty so playback/extraction can fetch stream URLs lazily.
- Tests cover Bilibili result mapping and registry exclusion when a source is disabled.
- A live network timing smoke test against Bilibili has not been run in this slice, so the `<3s` criterion remains open.

## Blocked by

- Issue 3 (bili_client must exist for API calls)

## PRD reference

- User stories #13, #14
- Module 4: `source/bilibili`
