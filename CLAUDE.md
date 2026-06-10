# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

```bash
# Frontend dev server (port 1420)
pnpm dev

# Full Tauri desktop app (frontend + Rust backend)
pnpm tauri dev

# TypeScript type-check + build
pnpm build

# Rust checks (run from src-tauri/)
cargo check          # fast compile check, no binary
cargo test           # run all Rust tests
cargo test <name>    # run a single test
cargo build          # full build
```

## Architecture

Tauri 2 desktop app — React 19 + TypeScript frontend, Rust backend. SQLite for user data, tauri-plugin-store (JSON file) for settings.

```
Frontend (React)                    Backend (Rust)
src/                                src-tauri/src/
  lib/tauri.ts ──invoke()──>         commands/search.rs   (Tauri IPC commands)
  lib/db.ts ──plugin-sql──>          source/              (music providers)
  pages/                             bili/                (bilibili auth + video)
  components/                        config.rs, error.rs
```

**Core pattern**: `SourceProvider` trait in `src-tauri/src/source/provider.rs`. Every music source implements `search()` and `fetch_latest()`. Registered in `lib.rs`, executed in parallel via `SourceRegistry` using `futures::future::join_all`.

**Path alias**: `@/` → `./src` (vite.config.ts).

## Key Modules

### source/ — Music providers (Pitchfork, Discogs, Bandcamp, AOTY, Jamendo, Stereogum, Fader)

Three categories: `api/` (REST), `rss/` (XML feeds), `scraper/` (HTML). All implement `SourceProvider` trait. Search fans out to all enabled providers in parallel; individual failures are logged and skipped.

### bili/ — Bilibili video chapter detection (recently added)

`auth.rs` — QR-code and password login, session extraction.  
`wbi.rs` — WBI signing (img_key/sub_key → mixin key → md5 signature). Keys fetched from nav endpoint, cached in `Mutex<Option<WbiKeys>>`.  
`client.rs` — Authenticated HTTP client wrapping WBI-signed requests with SESSDATA cookie.  
`extractor.rs` — Extracts video chapters from Bilibili video pages.  
`ffmpeg.rs` — FFmpeg-based audio/video processing.  
`url.rs` — Bilibili URL parsing (BV/AV IDs, episode detection).

### commands/ — Tauri IPC commands

Registered in `lib.rs` via `.invoke_handler(tauri::generate_handler![...])`:
- `search_sources`, `fetch_latest`, `list_sources`, `set_source_enabled`
- `update_discogs_keys`, `update_jamendo_client_id`
- `download_audio`, `open_url`
- `bili_login_qr`, `bili_login_password`, `bili_check_login`, `bili_get_chapters`, `bili_logout`

### lib/ — Frontend utilities

`tauri.ts` — Typed wrappers around `invoke()` for all Tauri commands.  
`db.ts` — Direct SQLite via `@tauri-apps/plugin-sql`.  
`player-context.tsx` — React context for audio playback state.

## Rust Patterns in This Codebase

- **Interior mutability for runtime config**: `DiscogsKeys` wraps fields in `Arc<RwLock<Option<String>>>` — shared between Tauri managed state and source implementations, updated at runtime without restart.
- **Error handling**: `YadigError` enum via `thiserror`, serializes to string for Tauri IPC. `From` impls auto-convert external errors. Type alias `Result<T>` throughout.
- **MutexGuard before .await**: Guard clones must happen before async boundaries since `MutexGuard` is `!Send`.
- **serde(rename_all = "snake_case")**: Rust snake_case auto-maps to TypeScript camelCase over IPC.

## Database

Migrations in `src-tauri/migrations/`: favorites, search_history, rss_feeds, articles.

## Hard-Won Lessons

### Filename Truncation — sanitize AND truncate at every entry point

Bilibili video titles can be 130+ bytes (Chinese = 3 bytes/char). Adding chapter names can push filenames past EXT4's 255-byte limit → `ENAMETOOLONG`.

**Always truncate at every filename entry point:**
- `commands/search.rs` `download_audio()` — backend receives filename from frontend
- `bili/client.rs` `sanitize_filename()` — used for local file output paths
- `bili/ffmpeg.rs` `temp_path()` — use hash instead of full title
- Frontend `safeName` construction — strip special chars AND limit length

Threshold: 200 bytes per filename component, leaving room for extension and prefix.

### Bilibili API Quirks

- `/x/player/wbi/v2` needs WBI signing (chapter data). Without it returns -400.
- `/x/player/playurl` does NOT need WBI signing (DASH audio URLs).
- URL parser: always `split('?')` BEFORE `trim_end_matches('/')` — query params hide trailing slashes.
- QR login session is in `data.url` query params (not Set-Cookie headers).

## Related Docs

- `CODEBUDDY.md` — detailed architecture with full source table and deep-dive notes
- `DEVELOPER_GUIDE.md` — Rust tutorial for contributors new to the language
- `ROADMAP.md` — phased development plan
