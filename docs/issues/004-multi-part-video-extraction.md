# Issue 4: Multi-part (分P) video extraction

## What to build

Extend the extraction path to handle videos with multiple parts (分P). When a video has `pages.len() > 1`, list all parts and allow the user to extract audio from specific parts or all parts.

1. **Extraction result enhancement** — `bili_extract_audio` returns `ExtractionResult` with `extraction_type: MultiPart` and a list of `AudioSegment`s, one per 分P. Each segment includes: `title` (from `page.part`), `cid`, `duration`, `audio_url`.

2. **Selective extraction** — New command `bili_extract_segment(bvid, cid, title)` that extracts audio for a single 分P by its cid. This allows the user to pick which parts to download.

3. **Batch extraction** — `bili_extract_audio` with a `MultiPart` result triggers download of all parts. Files are named `{video_title} - {part_title}.m4a`.

4. **Frontend** — When extraction returns `MultiPart` type, show a list of segments with individual play/download buttons and a "Download All" button.

## Acceptance criteria

- [ ] Video with 5 分P returns 5 AudioSegments with correct titles and durations
- [ ] Each 分P can be extracted independently via `bili_extract_segment`
- [ ] "Download All" extracts all parts to individual files
- [ ] File naming: `{video_title} - {part_title}.m4a`
- [ ] Frontend shows segment list with play/download controls
- [ ] `cargo check` passes

## Blocked by

- Issue 3 (single video extraction must work first)

## PRD reference

- User stories #5, #20
- Module 5: `bili_extractor` (MultiPart path)
