# PRD: Bilibili Video Audio Extraction

## Problem Statement

As a music enthusiast using yadig, I want to extract audio from Bilibili videos so that I can discover and collect music that is only available as video content on Bilibili. Many music resources on Bilibili exist only in video form — full album uploads, live performances, DJ sets, compilations — and there is no way to get the audio separately. The videos come in three structural forms: single long videos with chapter markers (each chapter = one song), multi-part videos (分P, each part = one song), and video collections/series (合集, each video = one song). I want yadig to handle all three cases and extract the highest quality audio possible, using my Bilibili account to access premium audio streams.

## Solution

Add Bilibili as a new audio-capable source in yadig. Users authenticate with their Bilibili account (cookie-based or QR code login) to unlock the highest audio quality their account tier allows. Given a Bilibili video URL or BV号, yadig automatically detects the video structure (single/chapters/parts/collection), extracts audio streams from the DASH format, and presents them for playback or download. For long videos with chapter markers, yadig can split the audio into individual tracks using FFmpeg.

## User Stories

1. As a yadig user, I want to paste a Bilibili video URL (e.g., `bilibili.com/video/BV1xxx`) into the search bar, so that yadig recognizes it as a Bilibili link and fetches the video's audio streams
2. As a yadig user, I want to log in to my Bilibili account in yadig's settings, so that I can access 192K quality audio streams (and higher if I have a premium account)
3. As a yadig user, I want to log in via QR code scan, so that I don't need to manually extract cookies or type my password into a third-party app
4. As a yadig user, I want to log in by pasting my SESSDATA cookie, so that I have an alternative login method if QR code is inconvenient
5. As a yadig user, I want yadig to detect whether a Bilibili video has multiple parts (分P), so that I can see all parts listed and choose which ones to extract audio from
6. As a yadig user, I want yadig to detect chapter markers in a long video, so that I can see the individual songs/sections and optionally extract each as a separate audio file
7. As a yadig user, I want yadig to detect if a video belongs to a collection (合集/ugc_season), so that I can browse and extract audio from all videos in the collection
8. As a yadig user, I want to play extracted Bilibili audio directly in yadig's audio player, so that I can preview the audio before downloading
9. As a yadig user, I want to download the audio stream to my Downloads folder, so that I can listen offline
10. As a yadig user, I want yadig to automatically select the highest quality audio stream available for my account tier, so that I always get the best possible audio
11. As a yadig user, I want to see the audio quality (bitrate) of each available stream, so that I can make an informed choice
12. As a yadig user, I want yadig to use my configured proxy settings when connecting to Bilibili APIs, so that the app works behind firewalls
13. As a yadig user, I want yadig to search Bilibili for music videos by keyword, so that I can discover music content on Bilibili from within yadig
14. As a yadig user, I want to see Bilibili search results alongside results from other sources (Discogs, Pitchfork, etc.), so that I have a unified music discovery experience
15. As a yadig user, I want yadig to remember my Bilibili login across app restarts, so that I don't need to re-authenticate every time
16. As a yadig user, I want to log out of my Bilibili account in settings, so that I can switch accounts or remove my credentials
17. As a yadig user, I want yadig to handle authentication errors gracefully (expired session, invalid cookie), so that I'm prompted to re-login instead of seeing cryptic errors
18. As a yadig user, I want to extract audio from a Bilibili video collection (合集) URL, so that I can batch-download an entire album or playlist
19. As a yadig user, I want chapter-based audio splitting to produce files named with the chapter title, so that the output files are organized and labeled
20. As a yadig user, I want to see the duration and title of each 分P or chapter before extracting, so that I can identify the content

## Implementation Decisions

### Module Overview

The implementation requires these new/modified modules:

| Module | Type | Responsibility |
|---|---|---|
| `bili_client` | New (Rust) | Low-level Bilibili API client: HTTP requests, Wbi签名, response parsing |
| `bili_auth` | New (Rust) | Login flows (QR code, cookie), session persistence, token refresh |
| `bili_types` | New (Rust) | Bilibili API response types (VideoInfo, Page, UgcSeason, DashAudio, ViewPoint, etc.) |
| `source/bilibili` | New (Rust) | SourceProvider implementation for Bilibili search and latest content |
| `bili_extractor` | New (Rust) | Audio stream extraction logic: URL parsing, structure detection, stream selection |
| `commands/bilibili` | New (Rust) | Tauri IPC commands for Bilibili operations |
| `config.rs` | Modify (Rust) | Add BiliAuth managed state alongside DiscogsKeys |
| `lib.rs` | Modify (Rust) | Register Bilibili source and commands |
| `settings-page.tsx` | Modify (Frontend) | Add Bilibili login section with QR code and cookie input |
| `tauri.ts` | Modify (Frontend) | Add Bilibili command wrappers |
| `search-page.tsx` | Modify (Frontend) | Handle Bilibili URL detection and display Bilibili results with audio controls |
| `types/source.ts` | Modify (Frontend) | Add Bilibili-specific types if needed |

### Module 1: `bili_client` — Bilibili API Client

A deep module that encapsulates all Bilibili API interactions behind a simple interface. Does NOT depend on Tauri — pure Rust library logic.

**Key responsibilities:**
- Build authenticated HTTP requests with proper headers (`Referer`, `User-Agent`, `Cookie`)
- Implement Wbi签名 (signing algorithm for protected endpoints)
- Parse API responses and handle error codes (rate limiting, auth failures, geo-blocking)

**Key API endpoints used:**
- `GET /x/web-interface/view` — video info (pages, ugc_season)
- `GET /x/player/wbi/v2` — player data (view_points/chapters)
- `GET /x/player/wbi/playurl` — DASH audio/video stream URLs
- `GET /x/web-interface/wbi/search/type` — search videos by keyword
- `GET /x/polymer/web-space/seasons_archives_list` — collection video list
- `GET /passport/x/qrcode/*` — QR login flow

**Interface sketch:**
```rust
pub struct BiliClient {
    http: reqwest::Client,
    auth: Arc<RwLock<Option<BiliSession>>>,
}

impl BiliClient {
    pub async fn video_info(&self, bvid: &str) -> Result<VideoInfo>;
    pub async fn player_info(&self, aid: i64, cid: i64) -> Result<PlayerInfo>;
    pub async fn playurl(&self, aid: i64, cid: i64, qn: u32) -> Result<PlayUrlResponse>;
    pub async fn search_videos(&self, keyword: &str, page: u32) -> Result<SearchResult>;
    pub async fn season_archives(&self, mid: i64, season_id: i64, page: u32) -> Result<SeasonArchives>;
}
```

### Module 2: `bili_auth` — Authentication

Supports two login methods, both storing a `BiliSession` (SESSDATA + bili_jct + dede_user_id):

1. **QR Code Login**: Generate QR code URL → user scans with Bilibili app → poll status → receive cookies
2. **Cookie Login**: User pastes SESSDATA string directly (simpler but less user-friendly)

Session is persisted in `tauri-plugin-store` (like DiscogsKeys) and hydrated into `BiliClient` on startup.

**Interface sketch:**
```rust
pub struct BiliSession {
    pub sessdata: String,
    pub bili_jct: String,
    pub dede_user_id: String,
}

pub struct BiliAuth {
    session: Arc<RwLock<Option<BiliSession>>>,
}

impl BiliAuth {
    pub async fn qr_login_start(&self) -> Result<QrLoginInfo>; // returns URL + key
    pub async fn qr_login_poll(&self, qrcode_key: &str) -> Result<QrLoginStatus>;
    pub fn set_cookie(&self, sessdata: &str);
    pub fn session(&self) -> Option<BiliSession>;
    pub fn logout(&self);
}
```

### Module 3: `bili_types` — API Response Types

Rust structs that map to Bilibili's JSON responses. Key types:

- `VideoInfo`: `bvid`, `aid`, `title`, `pages: Vec<Page>`, `ugc_season: Option<UgcSeason>`
- `Page`: `cid`, `part` (title), `duration` (seconds)
- `UgcSeason`: `id`, `title`, `sections: Vec<Section>` → `episodes: Vec<Episode>`
- `PlayerInfo`: `view_points: Vec<ViewPoint>`
- `ViewPoint`: `content` (title), `from` (seconds), `to` (seconds)
- `PlayUrlResponse`: `dash: DashInfo`
- `DashInfo`: `audio: Vec<DashAudio>`
- `DashAudio`: `id` (quality code), `base_url`, `bandwidth`, `codecs`

### Module 4: `source/bilibili` — SourceProvider Implementation

Implements `SourceProvider` trait so Bilibili appears as a searchable source in yadig's unified search.

- `id()`: `"bilibili"`
- `name()`: `"Bilibili"`
- `kind()`: `SourceKind::Api`
- `search()`: Uses `/x/web-interface/wbi/search/type` with `search_type=video`, filters for music-related content, populates `audio_url` from DASH streams where available
- `fetch_latest()`: Could use Bilibili's music zone ranking API or return empty initially

**Key decision:** The SourceProvider `search()` returns basic video metadata. The actual audio stream extraction (getting playurl) happens lazily — when the user clicks play or download on a result, the frontend calls a dedicated command to fetch the stream URL. This avoids slowing down search results with per-video API calls.

### Module 5: `bili_extractor` — Audio Extraction Logic

The core intelligence module. Given a Bilibili URL, it:

1. **Parses the URL** to determine type: single video (BV号), collection URL (`/space/channel/collectiondetail?sid=xxx`), or series
2. **Fetches video info** to detect structure:
   - Has `ugc_season`? → collection mode
   - Has `pages.len() > 1`? → multi-part mode
   - Single page? → check for `view_points` (chapters)
3. **For each audio segment**, calls `playurl` with `fnval=16` (DASH) to get audio stream URLs
4. **Selects best audio stream** from `dash.audio[]` based on account tier (192K for logged-in, 64K for anonymous, Dolby/Hi-Res for premium)
5. **Returns structured result** with all segments and their audio URLs

For chapter-based splitting, FFmpeg is needed to cut the audio at `from`/`to` timestamps. This is a secondary feature — the primary goal is extracting the full audio stream.

**Interface sketch:**
```rust
pub struct AudioSegment {
    pub title: String,
    pub audio_url: String,
    pub duration: u32,
    pub quality: AudioQuality,
}

pub struct ExtractionResult {
    pub video_title: String,
    pub segments: Vec<AudioSegment>,
    pub extraction_type: ExtractionType, // Single, MultiPart, Chapters, Collection
}

pub enum ExtractionType {
    Single,           // one video, one audio
    MultiPart,        // 分P: multiple pages
    Chapters,         // view_points: multiple chapters in one video
    Collection,       // ugc_season: multiple videos in a collection
}

pub async fn extract_audio(client: &BiliClient, url: &str) -> Result<ExtractionResult>;
```

### Module 6: `commands/bilibili` — Tauri IPC Commands

New commands exposed to the frontend:

| Command | Params | Returns | Description |
|---|---|---|---|
| `bili_qr_login_start` | none | `{ url, qrcode_key }` | Start QR login flow |
| `bili_qr_login_poll` | `qrcode_key` | `{ status, session? }` | Poll QR login status |
| `bili_cookie_login` | `sessdata` | `()` | Login with cookie string |
| `bili_logout` | none | `()` | Clear session |
| `bili_extract_audio` | `url` | `ExtractionResult` | Extract audio from Bilibili URL |
| `bili_search` | `query, page?` | `SearchResult` | Search Bilibili for videos |
| `bili_get_playurl` | `bvid, cid` | `{ audio_url, quality }` | Get audio stream URL for a specific video part |

### Module 7: Frontend Changes

**Settings page — Bilibili login section:**
- QR code display (rendered from URL returned by `bili_qr_login_start`)
- Polling indicator with auto-detection of scan success
- Alternative: SESSDATA cookie input field
- Login status display (logged in as username, account tier)
- Logout button

**Search page — Bilibili URL detection:**
- When user pastes a Bilibili URL, detect it via regex (`bilibili.com/video/BV`, `b23.tv/`)
- Show a "Extract Audio" action instead of normal search
- Display extraction results as a list of audio segments with play/download buttons

**Search results — Bilibili audio cards:**
- Show play button (triggers `bili_get_playurl` then plays via PlayerContext)
- Show download button (triggers `download_audio`)
- Display quality badge (64K/192K/etc.)

### Authentication Persistence Pattern

Follows the existing DiscogsKeys pattern:
1. `BiliAuth` is created in `lib.rs` and `.manage()`d on Tauri builder
2. Session is persisted in `tauri-plugin-store` under key `"bilibili_session"`
3. On app startup, `App.tsx` useEffect loads session from store and calls `bili_set_session` command
4. `BiliClient` reads session from `BiliAuth` on each API call (via `Arc<RwLock>`)

### Proxy Support

`BiliClient` uses `http_client::build_client()` (already exists) which reads `HTTPS_PROXY`/`HTTP_PROXY`/`ALL_PROXY` env vars. This is critical for Chinese users behind GFW.

### URL Parsing

Bilibili URLs come in several forms:
- `https://www.bilibili.com/video/BV1xxxxx` — single video
- `https://www.bilibili.com/video/BV1xxxxx?p=2` — specific 分P
- `https://b23.tv/xxxxx` — short link (needs redirect resolution)
- `https://space.bilibili.com/xxx/channel/collectiondetail?sid=123` — collection
- `https://www.bilibili.com/bangumi/play/ep123` — bangumi (out of scope for now)

The extractor should handle all non-bangumi forms.

## Testing Decisions

### What makes a good test
- Test external behavior (API response shapes, extraction results), not internal HTTP call details
- Use mock HTTP responses for Bilibili API calls to avoid hitting real APIs in tests
- Test URL parsing logic in isolation (pure function, no network)
- Test quality selection logic in isolation (given account tier + available streams → selected stream)

### Modules to test
1. **`bili_types`** — Deserialization of real Bilibili JSON response shapes (use captured fixtures)
2. **URL parsing** — Extract BV号, cid, season_id from various URL formats
3. **Quality selection** — Given auth state and available audio streams, pick the right one
4. **Structure detection** — Given a VideoInfo, correctly classify as Single/MultiPart/Chapters/Collection
5. **`bili_client`** — Mock-based integration tests for API calls (use `mockall` or similar)

### Prior art
- Existing sources (Discogs, Bandcamp) have no tests yet. This PRD introduces the first testable pure-logic modules (URL parsing, quality selection, structure detection) that should have tests as a foundation.

## Out of Scope

- **Bangumi/番剧 audio extraction** — Requires different API endpoints and potentially different auth (VIP-only content). Can be added later.
- **Live stream audio capture** — Real-time audio capture from Bilibili live streams is a different problem domain.
- **Video download** — yadig focuses on audio extraction, not full video download.
- **FFmpeg chapter splitting in Phase 1** — The initial implementation extracts full audio streams. Chapter-based splitting into individual files is a Phase 2 enhancement that requires FFmpeg integration.
- **Bilibili account creation** — Users must already have a Bilibili account.
- **Reverse engineering of non-public APIs** — We use documented/semi-documented APIs. If Bilibili changes their API, we adapt.

## Further Notes

### Audio Quality Matrix

| Account Tier | Max Audio Quality | API Requirement |
|---|---|---|
| Anonymous | 64K (30216) | No auth |
| Logged in | 192K (30280) | SESSDATA cookie |
| Premium (大会员) | Dolby Atmos (30250) / Hi-Res (30251) | SESSDATA + premium |

### DASH Audio Stream Format

Bilibili's DASH format provides audio as separate streams. The `dash.audio[]` array contains entries with:
- `id`: quality code (30216=64K, 30232=132K, 30280=192K, 30250=Dolby, 30251=Hi-Res)
- `baseUrl`: primary stream URL (expires in 120 minutes)
- `backupUrl`: backup URLs
- `bandwidth`: required bandwidth in bytes/sec
- `mimeType`: always `audio/mp4`
- `codecs`: codec info (typically `mp4a.40.2` for AAC)

Stream URLs require:
- `Referer: https://www.bilibili.com` header
- URLs expire after 120 minutes — must be fetched on-demand, not cached long-term

### Dependency: `bpi-rs` Crate (Hybrid Approach — Decision)

We use a **hybrid approach**: depend on `bpi-rs` for authentication (QR login, cookie management, Wbi签名) but implement stream extraction and audio processing ourselves.

- **bpi-rs handles**: Login flows, Wbi签名, cookie management, generic API request signing
- **We implement**: playurl fetching, DASH audio stream parsing, structure detection (分P/chapters/collection), FFmpeg-based audio extraction to local files, quality selection

### Audio Extraction to Local Files (Decision)

Audio is extracted **directly to local files** in the user's Downloads folder (or configured download directory). The flow is:
1. Fetch DASH audio stream URL via playurl API
2. Download the audio stream to a temporary file
3. For chapter-based splitting: use FFmpeg to cut at `from`/`to` timestamps
4. Save final files as `{video_title} - {segment_title}.m4a` (AAC in MP4 container, matching Bilibili's native format)

### Existing Rust Tools Reference
- `bpi-rs`: Full Bilibili API SDK in Rust (322 APIs, login flows, type-safe responses)
- `biliget` (crates.io): CLI tool for Bilibili video download, uses FFmpeg for merging
