# CODEBUDDY.md

This file provides guidance to CodeBuddy Code when working with code in this repository.

## Project Overview

**yadig** is a desktop music discovery app built with Tauri 2 (Rust backend + React frontend). It aggregates music information from multiple sources (RSS, API, HTML scrapers) and provides unified search and feed experiences.

## Common Commands

### Development
```bash
# Start full desktop app in development mode (frontend + backend)
pnpm tauri dev

# Start frontend dev server only (port 1420)
pnpm dev

# Build frontend for production
pnpm build

# Build full desktop application
pnpm tauri build
```

### Rust Backend
```bash
# Navigate to Rust project
cd src-tauri

# Check Rust code compiles
cargo check

# Run Rust tests
cargo test

# Build Rust library
cargo build
```

## Architecture

### Tech Stack
- **Frontend**: React 19 + TypeScript + Tailwind CSS v4
- **Backend**: Rust with Tauri 2 framework
- **Database**: SQLite via tauri-plugin-sql
- **State Management**: @tanstack/react-query for server state
- **Persistence**: tauri-plugin-store for settings, SQLite for user data

### Project Structure
```
yadig/
├── src/                    # Frontend React code
│   ├── main.tsx           # App entry point
│   ├── App.tsx            # Root component with routing
│   ├── components/        # Reusable UI components
│   │   ├── layout/        # AppLayout, AppSidebar
│   │   ├── audio-player.tsx   # Global floating audio player
│   │   └── error-boundary.tsx
│   ├── pages/             # Route pages (search, feed, detail, chat, settings)
│   ├── lib/               # Utilities and Tauri wrappers
│   │   ├── tauri.ts       # Tauri IPC invoke wrappers
│   │   ├── db.ts          # Direct SQLite access via tauri-plugin-sql
│   │   ├── player-context.tsx  # Audio playback state (React context)
│   │   └── utils.ts       # General utilities (cn, etc.)
│   └── types/             # TypeScript interfaces
├── src-tauri/             # Rust backend
│   ├── src/
│   │   ├── main.rs        # Entry point
│   │   ├── lib.rs         # App setup and command registration
│   │   ├── commands/      # Tauri IPC commands
│   │   ├── config.rs      # Runtime config (DiscogsKeys)
│   │   ├── http_client.rs # Shared HTTP client with proxy support
│   │   ├── error.rs       # YadigError enum
│   │   └── source/        # Music source providers
│   │       ├── provider.rs    # SourceProvider trait
│   │       ├── registry.rs    # SourceRegistry (parallel execution)
│   │       ├── types.rs       # ContentItem, SearchResult structs
│   │       ├── api/           # REST API sources (Discogs, Jamendo)
│   │       ├── rss/           # RSS sources (Pitchfork, Stereogum, Fader)
│   │       └── scraper/       # HTML scraping sources (Bandcamp, AOTY)
│   ├── migrations/        # SQLite schema migrations
│   └── Cargo.toml         # Rust dependencies
└── dist/                  # Built frontend output
```

### Core Architecture Pattern: SourceProvider Trait

The backend uses a **SourceProvider trait** pattern for music sources:

```rust
#[async_trait]
pub trait SourceProvider: Send + Sync {
    fn id(&self) -> &str;
    fn name(&self) -> &str;
    fn kind(&self) -> SourceKind;
    fn base_url(&self) -> &str;
    async fn search(&self, query: &str, limit: usize, page: usize) -> Result<Vec<ContentItem>>;
    async fn fetch_latest(&self, limit: usize) -> Result<Vec<ContentItem>>;
}
```

**Current implementations:**
- **Pitchfork** (RSS) - RSS feed parsing
- **Discogs** (API) - REST API with optional authentication
- **Bandcamp** (Scraper) - HTML scraping
- **Album of the Year** (Scraper) - HTML scraping
- **Jamendo** (API) - Independent music, CC licensed, provides audio URLs
- **Stereogum** (RSS) - Music blog RSS feed
- **Fader** (RSS) - Music magazine RSS feed

**To add a new source:** Implement `SourceProvider` trait and register in `lib.rs`.

### Frontend-Backend Communication

Frontend calls Rust functions via Tauri IPC:
```typescript
// src/lib/tauri.ts
import { invoke } from '@tauri-apps/api/core';

export async function searchSources(query: string, sources?: string[]) {
  return invoke<SearchResult>('search_sources', { query, sources });
}
```

### Data Flow
1. User input → React component
2. React Query → Tauri invoke
3. Rust `SourceRegistry` → parallel source queries
4. Results aggregated → returned to frontend
5. Frontend renders → saves to SQLite if needed

### Dual Persistence Pattern
- **SQLite**: User data (favorites, search history, RSS feeds)
- **tauri-plugin-store**: App settings (API keys, source states)
- **Tauri managed state**: In-memory state hydrated from store on startup

### Registered Tauri Commands
Defined in `src-tauri/src/commands/search.rs`, registered in `lib.rs`:
- `search_sources` — parallel search across enabled sources
- `fetch_latest` — get latest content from all sources
- `list_sources` — list registered sources with enabled/disabled state
- `set_source_enabled` — toggle a source on/off
- `update_discogs_keys` — update Discogs API credentials at runtime
- `download_audio` — download audio file to local filesystem
- `open_url` — open URL in default browser

### Key Files for Understanding Architecture
- `src-tauri/src/source/provider.rs` - Core SourceProvider trait definition
- `src-tauri/src/source/registry.rs` - Source registration and parallel execution
- `src/lib/tauri.ts` - Frontend Tauri command wrappers
- `src/lib/db.ts` - Direct SQLite access patterns
- `src/App.tsx` - App initialization and settings hydration

## Database Schema

Migrations in `src-tauri/migrations/`:
1. `001_initial_schema.sql` - Favorites table
2. `002_search_history.sql` - Search history for autocomplete
3. `003_rss_feeds.sql` - Custom RSS feeds and articles

## Development Notes

### Path Aliases
- `@/` maps to `./src` (configured in vite.config.ts)

### Error Handling
- Rust errors use `YadigError` enum with `thiserror`
- Errors serialize to strings for Tauri IPC
- Individual source failures don't fail entire search

### Styling
- Tailwind CSS v4 with custom dark theme
- Color palette: dark base, green primary, red destructive accents

### Current Status
Phase 0-1 complete. Phase 2 partially done (Jamendo API source, audio player). LLM integration and additional audio sources planned. Chat page and LLM settings are placeholders.

### Audio Playback
- `src/lib/player-context.tsx` — React context providing play/pause/seek/volume state
- `src/components/audio-player.tsx` — Global floating player UI
- Audio sources provide `audio_url` in `ContentItem.extra` (Jamendo) or directly in the struct
- `download_audio` Tauri command available for saving audio files locally

### HTTP Client & Proxy
`src-tauri/src/http_client.rs` builds a shared `reqwest::Client` that reads `HTTPS_PROXY`/`HTTP_PROXY`/`ALL_PROXY` env vars. Important for users behind firewalls (e.g., China). All sources should use `http_client::build_client()` instead of building clients directly.

---

## Deep Architecture Notes

### Concurrent Source Execution

`SourceRegistry::search()` fans out to all enabled providers in parallel using `futures::future::join_all`. Individual source failures are logged and skipped — partial results are returned. Key pattern:

```rust
// Snapshot MutexGuard before .await (MutexGuard is !Send)
let disabled_ids = self.disabled.lock().unwrap().clone();
// Now safe to .await across threads
let results = futures::future::join_all(futures).await;
```

### Interior Mutability for Runtime Config

`DiscogsKeys` uses `Arc<RwLock<Option<String>>>` — shared ownership between Tauri managed state and `DiscogsSource`. The `update_discogs_keys` command writes to the `RwLock`; the Discogs source reads it on each API call. No restart needed.

### Source-Specific Behaviors

| Source | Search Strategy | fetch_latest | Notes |
|---|---|---|---|
| Pitchfork | Fetches 100 items, client-side keyword filter | Merges 3 RSS feeds, sorts by date | RSS has no search API |
| Discogs | REST API `database/search` | Returns empty (no endpoint) | Optional auth via DiscogsKeys |
| Bandcamp | HTML scrape of search page | Internal discover API, HTML fallback | Uses browser User-Agent |
| AOTY | HTML scrape of search page | Scrapes `/albums/new/` | Cloudflare-protected |
| Jamendo | REST API v3.0 `/tracks/` | REST API `/tracks/` with ordering | Returns audio URLs directly; CC licensed |
| Stereogum | Fetches RSS, client-side keyword filter | Latest from RSS feed | RSS has no search API |
| Fader | Fetches RSS, client-side keyword filter | Latest from RSS feed | RSS has no search API |

### TypeScript-Rust Type Mapping

Rust `snake_case` fields auto-convert to TypeScript `camelCase` via `serde(rename_all)`. TypeScript interfaces in `src/types/source.ts` mirror Rust structs in `src-tauri/src/source/types.rs`.

### Startup Hydration

`App.tsx` useEffect loads persisted settings from `tauri-plugin-store` and pushes them into Rust via `invoke("update_discogs_keys")` and `invoke("set_source_enabled")`.

---

## Hard-Won Lessons

### Filename Sanitization — ALWAYS truncate at every entry point

`File name too long (os error 36)` on EXT4 occurs when a filename exceeds 255 bytes. Bilibili video titles can be 130+ bytes (Chinese chars are 3 bytes each). Adding chapter names (`" - {chapter}.m4a"`) pushes it over the limit.

**Rule: Sanitize AND truncate at EVERY entry point where a filename is constructed.**

Places that need truncation (all bitten before):
- `commands/search.rs` `download_audio()` — the Rust side receives a filename from the frontend
- `bili/client.rs` `sanitize_filename()` — used for local file output
- `bili/ffmpeg.rs` `temp_path()` — temp files during download

**Don't rely on frontend truncation** — the frontend's `safeName` regex only strips special chars but doesn't limit length. Always truncate in the backend too.

Use `200 bytes` as the threshold (leaving ~55 bytes for path prefix + extensions).

### Bilibili API Endpoints

Some endpoints require WBI signing (`/x/player/wbi/v2`), others don't (`/x/player/playurl`):
- `/x/web-interface/view` — no signing needed, returns video info
- `/x/player/playurl` — no signing needed, returns DASH audio URLs
- `/x/player/wbi/v2` — **requires** WBI signing, returns chapter data (view_points)
- `/x/player/v2` — also requires WBI signing (returns -400 without it)

The WBI algorithm: fetch `img_key`/`sub_key` from nav API → mix with fixed index table → URL-encode and sort params → MD5 the result → append `w_rid` and `wts`.

### URL Parsing — split before trim

When parsing Bilibili video URLs with query params (`?spm_id_from=...`):
```
// WRONG: trim before split
s.trim_end_matches('/').split('?').next()  // "BV1xxx/" ← trailing slash!
// RIGHT: split before trim
s.split('?').next().unwrap_or("").trim_end_matches('/')  // "BV1xxx" ✓
```
