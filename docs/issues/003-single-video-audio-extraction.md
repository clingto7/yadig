# Issue 3: Single video audio extraction to local file

## What to build

The core extraction path: given a Bilibili BV号, fetch the video info, get the DASH audio stream URL, download it, and save as a local `.m4a` file. This is the simplest extraction case (single video, single part, no chapters).

1. **`bili_client` module** — Implements the subset of Bilibili API calls needed:
   - `video_info(bvid)` → fetch from `/x/web-interface/view`
   - `playurl(aid, cid, fnval=16)` → fetch DASH streams from `/x/player/wbi/playurl`
   - Uses `http_client::build_client()` for proxy support
   - Attaches cookies from `BiliAuth` when available
   - Sets `Referer: https://www.bilibili.com` header

2. **Audio download** — Tauri command `bili_extract_audio(url: String)` that:
   - Parses the BV号 from URL
   - Calls video_info → gets cid from first page
   - Calls playurl → gets best audio stream URL (using quality selection from Issue 1)
   - Downloads the audio stream to `{Downloads}/yadig/{video_title}.m4a`
   - Returns the file path and metadata

3. **Frontend integration** — Wire `bili_extract_audio` into `tauri.ts`. On the search page, when a Bilibili URL is detected in the search input, show an "Extract Audio" button that calls this command and shows progress/result.

## Acceptance criteria

- [ ] Given a BV号, the backend fetches video info and audio stream URL
- [ ] Audio stream is downloaded to user's Downloads folder as `.m4a`
- [ ] Best available quality is selected based on login state (64K anonymous, 192K logged in)
- [ ] Downloaded file plays correctly in standard audio players
- [ ] Proxy support works (HTTPS_PROXY env var respected)
- [ ] Error handling: invalid BV号, network failure, expired stream URL all return clear errors
- [ ] Frontend shows extraction progress and success/failure state
- [ ] `cargo check` and `pnpm build` pass

## Blocked by

- Issue 1 (bili_types, quality selection)
- Issue 2 (bili_auth for authenticated requests)

## PRD reference

- Module 1: `bili_client`
- Module 5: `bili_extractor` (single video path)
- Module 6: `commands/bilibili` (bili_extract_audio, bili_get_playurl)
- Audio Extraction to Local Files section
