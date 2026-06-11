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

- [ ] Pasting `bilibili.com/video/BV1xxx` triggers extraction mode
- [ ] Pasting `b23.tv/xxx` triggers extraction mode (short link resolved)
- [ ] Extraction results display correctly for all types (single/multipart/chapters/collection)
- [ ] Play button streams audio through the global player
- [ ] Download button saves file and shows notification
- [ ] "Download All" for multi-part/chapters extracts all segments
- [ ] Error states (invalid URL, network error, auth required) show clear messages
- [ ] Normal (non-Bilibili) search still works as before
- [ ] `pnpm build` passes

## Blocked by

- Issue 3 (single extraction backend)
- Issue 8 (settings UI — user needs to be logged in for best quality)

## PRD reference

- User stories #1, #5, #6, #7, #8, #9
- Module 7: Frontend Changes (Search page section)
