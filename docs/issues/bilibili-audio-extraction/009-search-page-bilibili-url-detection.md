# Issue 9: Search page — Bilibili URL detection + extraction UI

## What to build

When the user pastes a Bilibili URL into yadig's search bar, detect it and switch to an extraction-focused UI instead of normal search.

1. **URL detection** — In the search input `onChange`/`onPaste` handler, detect Bilibili URLs via regex:
   - `bilibili.com/video/BV`
   - `b23.tv/`
   - `space.bilibili.com/.../collectiondetail`

2. **Extraction mode UI** — When a Bilibili URL is detected:
   - Show "Extract Audio" button instead of "Search"
   - On click, call `bili_extract_audio(url)`
   - Show loading state with progress indicator

3. **Result display** — Based on extraction type:
   - **Single**: Show audio card with play/download buttons
   - **MultiPart/Chapters**: Show list of segments, each with play/download, plus "Download All"
   - **Collection**: Show progress bar + completed file list

4. **Audio playback** — Integrate with existing `PlayerContext`. Clicking play on a segment calls `bili_get_playurl(bvid, cid)` to get a fresh stream URL (they expire in 120min), then passes to `player.play()`.

5. **Download** — Clicking download calls `bili_extract_segment` or the batch equivalent. Show toast notification on completion with file path.

## Acceptance criteria

- [x] Pasting `bilibili.com/video/BV1xxx` triggers extraction mode
- [x] Pasting `b23.tv/xxx` triggers extraction mode (short link resolved)
- [x] Extraction results display correctly for all types (single/multipart/chapters/collection)
- [x] Play button streams audio through the global player
- [x] Download button saves file and shows notification
- [x] "Download All" for multi-part/chapters extracts all segments
- [x] Error states (invalid URL, network error, auth required) show clear messages
- [x] Normal (non-Bilibili) search still works as before
- [x] `pnpm build` passes

## Implementation notes

- Search page URL detection covers `bilibili.com/video/BV`, `b23.tv/`, and collection-detail URLs through `src/lib/search-url-detection.ts` contract coverage.
- Extraction results render Bilibili single/multi-part/chapter/collection results through the shared result panel.
- Multi-segment results expose a batch action; when extraction already saved local files, the action opens the saved folder instead of re-downloading remote audio URLs.
- Chapter fallback warnings are displayed inline for FFmpeg-unavailable environments.
- Download buttons now call `notifyDownloadSaved` after saving or opening an already-saved extraction file, using `@tauri-apps/plugin-notification` when permission is granted.
- The `b23.tv` short-link backend path is implemented and routed through `bili_extract_audio`; a live short-link manual smoke test has not been run in this slice.

## Blocked by

- Issue 3 (single extraction backend)
- Issue 8 (settings UI — user needs to be logged in for best quality)

## PRD reference

- User stories #1, #5, #6, #7, #8, #9
- Module 7: Frontend Changes (Search page section)
