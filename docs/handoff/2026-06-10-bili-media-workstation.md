# Handoff: Bilibili Media Workstation

Date: 2026-06-10
Repo: `/home/m1zu/ws/yadig`
Branch: `main`
Current HEAD: `708d1a4 docs: add media workstation report`

## Current State

The Bilibili media workstation work has been merged into `main`.

Relevant commits:

- `de6f8aa feat: add Bilibili media workstation`
- `708d1a4 docs: add media workstation report`

The local `main` branch is ahead of `origin/main`; it has not been pushed in this session.

The feature worktree still exists at:

- `/tmp/yadig-bili-media-workstation`

It is clean and points at `feature/bili-media-workstation`. It was left in place because it is a `/tmp` worktree, not an owned project-local worktree.

## Primary Artifacts

Use these instead of re-deriving the same context:

- Implementation report: `docs/research/yadig-media-workstation-implementation-report.html`
- Bilibili batch operations research report: `docs/research/bilibili-batch-ops-report.html`
- Bilibili audio extraction PRD: `docs/prd/PRD-bilibili-audio-extraction.md`
- New media workstation page: `src/pages/workstation-page.tsx`
- Library domain model: `src-tauri/src/library.rs`
- LLM integration: `src-tauri/src/llm.rs`
- Bilibili sync/audio commands: `src-tauri/src/commands/library.rs`
- Library DB migration: `src-tauri/migrations/004_library_tables.sql`

## What Was Completed

This round changed yadig from a music discovery app toward a personal media/resource workstation.

Implemented Bilibili workstation scope:

- Sync Bilibili favorites, followed UPs, and watch-later items.
- Persist synced items into a local SQLite media library.
- Add LLM-assisted metadata classification through OpenAI-compatible `/chat/completions`.
- Add local metadata fallback when LLM config is missing or the request fails.
- Generate batch audio extraction plans for music-classified favorite/watch-later videos.
- Execute batch audio extraction by reusing the existing Bilibili audio extractor.
- Add a Workstation route and sidebar entry.
- Add LLM settings to the Settings page.
- Improve Bilibili cookie/QR login handling for full session data.

Review fixes already applied:

- Removed raw QR poll response logging that could leak `SESSDATA`, `bili_jct`, and `DedeUserID`.
- Preserved old snake_case `BiliSession` JSON compatibility with serde aliases.
- Added LLM response `source` and `warning` fields so the UI can distinguish real LLM analysis from fallback.
- Added stale item cleanup for Bilibili sync by synced scope.

## Verification Already Run

Run on final `main` after merge and report commit:

```bash
cargo test --manifest-path src-tauri/Cargo.toml
pnpm build
git diff --check
```

Results:

- Rust tests: 59 passed.
- Frontend TypeScript/Vite build: passed.
- Whitespace diff check: passed.

Known warnings remain in Rust tests/build:

- Unused imports in `src-tauri/src/bili/extractor.rs`.
- Dead-code warnings for older QR auth helper types/functions.

These warnings predate or are adjacent to the current work and did not block verification.

## Important Context

User intent:

- Bilibili features are part of a broader personal media/resource workstation direction, not a separate side app.
- First functional slice should cover favorites, follows, and watch-later.
- LLM integration should help classify and organize user media metadata.
- Music-like favorite/watch-later videos should support batch audio extraction.
- Future direction includes subtitles, TTS/ASR when subtitles are unavailable, LLM video summaries, and possibly WeChat/Zhihu or other media sources.

Runtime preference:

- Use `pnpm tauri dev` for runtime app testing.
- Do not use standalone Vite dev server as the runtime verification path.
- If QR login must be verified again, start Tauri and ask the user to scan the QR code.

Tooling caveat:

- Earlier `/review` got stuck because a review child session entered a broken sandbox state and an MCP CodeGraph call hung for about 51 minutes.
- Direct CodeGraph status/context calls in the current full-access session were fast.
- Prefer local `rg`/shell for literal search and use CodeGraph only for structural questions.

Security caveat:

- Do not log raw Bilibili login responses, cookies, callback URLs, API keys, or LLM bearer tokens.
- Existing tests include a regression case for QR poll log redaction.

## Current User-Facing Flow

Search:

- Go to `Search`.
- Search music/media sources.
- Bilibili and YouTube URL extraction paths are available from the search UI.

Settings:

- Configure Bilibili login with QR code, full Cookie, or account/password.
- Configure LLM provider/base URL/model/API key.

Workstation:

- Go to `Workstation`.
- Click `Sync Bilibili`.
- Click `Analyze Metadata`.
- Click `Create Plan`.
- Click `Extract Audio` to execute music-video audio extraction.

Default output folder for extracted audio is `Downloads/yadig`.

## Known Limitations

- LLM analysis currently uses metadata only; it does not read subtitles, descriptions, comments, or transcript content.
- Subtitle extraction and ASR/TTS-backed summaries are not implemented.
- Batch Bilibili write operations such as moving favorites, deleting, or reorganizing remote collections are not implemented.
- LLM API key and other app settings are stored through Tauri Store; long-term secret storage could be improved with OS keychain integration.
- The local media library schema is new and currently focused on Bilibili resources.

## Suggested Next Steps

1. Push `main` to origin if the user wants the merged work published.
2. Optionally remove or archive `/tmp/yadig-bili-media-workstation` after confirming no longer needed.
3. Run `pnpm tauri dev` for a full manual smoke test:
   - Settings opens.
   - QR login or Cookie login works.
   - Workstation sync loads favorites/follows/watch-later.
   - LLM fallback warning appears when API key is absent.
   - Audio extraction plan can be generated from music-tagged videos.
4. Consider adding frontend/integration tests for `src/lib/db.ts` stale cleanup behavior.
5. Plan the next slice around subtitles/transcripts and LLM video summaries.

## Suggested Skills

- `superpowers:verification-before-completion` before claiming any merged or runtime state is good.
- `superpowers:receiving-code-review` if new review feedback arrives.
- `superpowers:systematic-debugging` if Bilibili login, sync, or LLM requests fail.
- `ffmpeg` if continuing work on audio extraction, remuxing, chapter splitting, or media transformations.
- `superpowers:writing-plans` before starting the subtitles/transcripts/summaries phase.

## Do Not Repeat

- Do not re-run `/review` if the environment still shows sandbox or CodeGraph hanging symptoms; use local review with `rg`, `git diff`, tests, and targeted CodeGraph calls.
- Do not paste or store real cookies, API keys, callback URLs, or user account identifiers in docs or logs.
- Do not start standalone Vite for runtime verification; use Tauri.
