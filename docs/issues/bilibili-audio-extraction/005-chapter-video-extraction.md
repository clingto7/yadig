# Issue 5: Chapter (view_points) video extraction with FFmpeg splitting

## What to build

Handle long videos that have chapter markers (view_points) in the progress bar. Each chapter represents a song/section. Extract the full audio, then use FFmpeg to split it into individual files based on chapter timestamps.

1. **Chapter detection** — When `bili_extractor` detects `view_points` from the player API (`/x/player/wbi/v2`), classify as `ExtractionType::Chapters`. Each `ViewPoint` has `content` (title), `from` (start seconds), `to` (end seconds).

2. **FFmpeg integration** — Add a `split_audio` utility that takes an input audio file and a list of `(start, end, output_path)` segments, runs FFmpeg to split:
   ```
   ffmpeg -i input.m4a -ss {start} -to {end} -c copy output.m4a
   ```
   Uses `-c copy` for lossless splitting (no re-encoding). Falls back to re-encoding if copy fails (some AAC streams have keyframe issues).

3. **Extraction flow** — For chapter-based videos:
   - Download full audio stream to temp file
   - Call `split_audio` with chapter timestamps
   - Save segments as `{video_title} - {chapter_title}.m4a`
   - Clean up temp file

4. **FFmpeg dependency check** — On app startup or before extraction, check if FFmpeg is available in PATH. If not, show a user-friendly message explaining how to install it. Chapter splitting is optional — full audio extraction works without FFmpeg.

## Acceptance criteria

- [ ] Video with 8 chapters returns 8 AudioSegments with correct titles and timestamps
- [x] FFmpeg splits audio at chapter boundaries without quality loss (-c copy)
- [x] Split files are named `{video_title} - {chapter_title}.m4a`
- [x] If FFmpeg is not installed, full audio extraction still works; chapter splitting shows install instructions
- [x] Fallback to re-encoding works if -c copy fails
- [x] Temp files are cleaned up after splitting
- [x] `cargo check` passes

## Implementation notes

- Chapter detection uses `/x/player/wbi/v2` `view_points` for the selected page.
- When FFmpeg is available, extraction downloads the full audio to a temp file, splits by chapter timestamps, removes the temp file, and returns `ExtractionType::Chapters`.
- When FFmpeg is unavailable, extraction returns the full audio as a single segment with `ExtractionType::Chapters` plus a warning explaining that FFmpeg is needed for chapter splitting.
- Search result warnings are shown inline, so users can see why a chapter video produced only one saved file.
- A live 8-chapter Bilibili sample has not been manually smoke-tested in this slice, so that criterion remains open.

## Blocked by

- Issue 3 (single video extraction must work first)

## PRD reference

- User stories #6, #19
- Module 5: `bili_extractor` (Chapters path)
- Out of Scope note: FFmpeg chapter splitting was listed as Phase 2, but we're pulling it forward
