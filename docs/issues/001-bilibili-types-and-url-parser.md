# Issue 1: Bilibili types and URL parser

## What to build

Create the foundational types and URL parsing logic for Bilibili integration. This includes:

1. **`bili_types` module** — Rust structs that map to Bilibili API JSON responses: `VideoInfo`, `Page`, `UgcSeason`, `Section`, `Episode`, `PlayerInfo`, `ViewPoint`, `PlayUrlResponse`, `DashInfo`, `DashAudio`, `SearchResponse`. These are pure data types with `Deserialize` derives.

2. **URL parser** — A pure function `parse_bilibili_url(url: &str) -> Result<BiliUrl>` that extracts identifiers from Bilibili URLs. Supports:
   - `https://www.bilibili.com/video/BV1xxxxx` → `BiliUrl::Video { bvid }`
   - `https://www.bilibili.com/video/BV1xxxxx?p=2` → `BiliUrl::Video { bvid, page: Some(2) }`
   - `https://b23.tv/xxxxx` → `BiliUrl::ShortLink { url }` (resolved later via HTTP redirect)
   - `https://space.bilibili.com/xxx/channel/collectiondetail?sid=123` → `BiliUrl::Collection { mid, season_id }`

3. **Structure detection** — A pure function `detect_structure(info: &VideoInfo) -> ExtractionType` that classifies a video as `Single`, `MultiPart`, `Chapters`, or `Collection` based on its `pages` and `ugc_season` fields.

4. **Quality selection** — A pure function `select_best_audio(streams: &[DashAudio], has_session: bool, is_premium: bool) -> Option<&DashAudio>` that picks the highest quality audio stream the user's account tier allows.

All four components are pure logic with no network or Tauri dependencies — fully unit-testable.

## Acceptance criteria

- [ ] `bili_types` module compiles and deserializes sample Bilibili JSON responses (provide test fixtures)
- [ ] `parse_bilibili_url` correctly extracts bvid, page number, mid, season_id from all supported URL formats
- [ ] `parse_bilibili_url` rejects non-Bilibili URLs with a clear error
- [ ] `detect_structure` correctly classifies: single video (1 page, no ugc_season), multi-part (N pages), chapters (1 page + view_points), collection (has ugc_season)
- [ ] `select_best_audio` picks 192K for logged-in users, 64K for anonymous, Dolby/Hi-Res for premium
- [ ] All pure functions have unit tests with fixture data
- [ ] `cargo test` passes

## Blocked by

None — can start immediately.

## PRD reference

- Module 3: `bili_types`
- Module 5: `bili_extractor` (URL parsing, structure detection, quality selection parts only)
