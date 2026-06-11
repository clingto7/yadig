# Issue 6: Collection (合集) batch extraction

## What to build

Handle Bilibili collection URLs (合集/ugc_season). When the user pastes a collection URL or a video that belongs to a collection, enumerate all videos in the collection and allow batch audio extraction.

1. **Collection URL parsing** — Extend `parse_bilibili_url` to handle:
   - `https://space.bilibili.com/{mid}/channel/collectiondetail?sid={season_id}` → `BiliUrl::Collection { mid, season_id }`
   - Video URLs that have `ugc_season` in their info → prompt user to extract full collection

2. **Collection enumeration** — Use `/x/polymer/web-space/seasons_archives_list` API to paginate through all videos in a collection. Returns list of `(aid, bvid, title, duration)` for each video.

3. **Batch extraction** — Command `bili_extract_collection(bvid)` that:
   - Fetches video info to get `ugc_season`
   - Enumerates all episodes in the collection
   - For each episode, fetches playurl and downloads audio
   - Saves as `{collection_title}/{episode_title}.m4a`
   - Returns progress (current/total) via Tauri events

4. **Frontend** — When extraction detects a collection, show a progress bar with current/total count. Allow cancel. Show completed files in a list.

## Acceptance criteria

- [x] Collection URL is correctly parsed and recognized
- [x] All videos in a collection are enumerated (handles pagination)
- [x] Each video's audio is extracted to a separate file in a collection-named subfolder
- [x] Progress is reported to frontend (current/total)
- [x] Cancel button stops extraction mid-way
- [x] Files named: `Downloads/yadig/{collection_title}/{episode_title}.m4a`
- [x] `cargo check` passes

## Notes

- Collection URL parsing is covered by `src-tauri/src/bili/url.rs` tests and extraction routing calls `extract_collection` for `BiliUrl::Collection`.
- Collection archive enumeration now paginates `/x/polymer/web-space/seasons_archives_list` using `page_num`/`page_size`, merging page archives until `meta.total` is reached or an empty page is returned.
- Pagination URL construction, stop conditions, and page merging are covered by `src-tauri/src/bili/client.rs` unit tests.
- Collection extraction now passes archive episode titles into the downloader, so files are created under the sanitized collection-title directory with sanitized episode-title filenames.
- Collection output path construction is covered by `collection_episode_output_path_uses_collection_subfolder`, which verifies sanitized `{collection_title}/{episode_title}.m4a` paths.
- Collection extraction emits `bili://collection-progress` events with `jobId`, `completed`, `total`, `currentTitle`, and `cancelled`; the Search page filters events by the active job and shows a progress bar.
- The Search page can request cancellation via `bili_cancel_extraction`. Cancellation is cooperative: the current episode download is allowed to finish, and remaining collection episodes are skipped.

## Blocked by

- Issue 3 (single video extraction must work first)

## PRD reference

- User stories #7, #18
- Module 5: `bili_extractor` (Collection path)
- Module 4: `bili_client` (season_archives API)
