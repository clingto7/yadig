use crate::bili::auth::{BiliAuth, BiliSession};
use crate::bili::extractor::{select_best_audio, AudioSegment, ExtractionResult, ExtractionType};
use crate::bili::ffmpeg;
use crate::bili::types::*;
use crate::bili::url::parse_bilibili_url;
use crate::bili::wbi::{self, WbiKeys};
use crate::error::{Result, YadigError};
use crate::library::{
    BiliResourceKind, BiliSyncResult, BiliSyncScope, LibraryCollection, LibraryItem,
    LibraryItemCollection,
};
use std::collections::BTreeMap;
use tokio::sync::Mutex;

/// Bilibili API client. Handles HTTP requests, auth cookies, and response parsing.
pub struct BiliClient {
    http: reqwest::Client,
    auth: BiliAuth,
    wbi_keys: Mutex<Option<WbiKeys>>,
}

#[derive(Default)]
struct FavoriteLibrarySnapshot {
    items: Vec<LibraryItem>,
    collections: Vec<LibraryCollection>,
    item_collections: Vec<LibraryItemCollection>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FavoriteMoveResource {
    pub id: String,
    pub resource_type: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FavoriteMoveBatch {
    pub source_media_id: String,
    pub target_media_id: String,
    pub resources: Vec<FavoriteMoveResource>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FavoriteWriteErrorKind {
    Failed,
    Blocked,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FavoriteWriteError {
    pub kind: FavoriteWriteErrorKind,
    pub message: String,
}

impl BiliClient {
    pub fn new(auth: BiliAuth) -> Self {
        Self {
            http: crate::http_client::build_client("yadig/0.1.0"),
            auth,
            wbi_keys: Mutex::new(None),
        }
    }

    /// Build request with auth cookies and referer header.
    fn request(&self, url: &str) -> reqwest::RequestBuilder {
        let mut req = self
            .http
            .get(url)
            .header("Referer", "https://www.bilibili.com");
        if let Some(session) = self.auth.session() {
            req = req.header("Cookie", session_cookie(&session));
        }
        req
    }

    /// Ensure WBI keys are available, fetching if needed.
    async fn ensure_wbi_keys(&self) -> Result<WbiKeys> {
        // Check cache (drop lock before await)
        {
            let guard = self.wbi_keys.lock().await;
            if let Some(ref keys) = *guard {
                return Ok(keys.clone());
            }
        }
        // Fetch new keys
        let keys = wbi::fetch_wbi_keys(&self.http)
            .await
            .map_err(|e| YadigError::Network(e))?;
        // Cache them
        *self.wbi_keys.lock().await = Some(keys.clone());
        Ok(keys)
    }

    /// Make a WBI-signed GET request to the given endpoint.
    async fn signed_get(
        &self,
        base_url: &str,
        params: &[(&str, &str)],
    ) -> Result<reqwest::Response> {
        let keys = self.ensure_wbi_keys().await?;
        let (w_rid, wts) = wbi::sign_params(params, &keys);

        let query: String = params
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join("&");
        let url = format!("{}?{}&w_rid={}&wts={}", base_url, query, w_rid, wts);

        let mut req = self
            .http
            .get(&url)
            .header("Referer", "https://www.bilibili.com");
        if let Some(session) = self.auth.session() {
            req = req.header("Cookie", session_cookie(&session));
        }
        req.send()
            .await
            .map_err(|e| YadigError::Network(format!("HTTP request failed: {}", e)))
    }

    async fn get_bili_data(&self, url: &str) -> Result<serde_json::Value> {
        let resp = self
            .request(url)
            .send()
            .await
            .map_err(|e| YadigError::Network(format!("Bilibili request failed: {}", e)))?;
        let body: BiliResponse<serde_json::Value> = resp
            .json()
            .await
            .map_err(|e| YadigError::Network(format!("Bilibili response parse error: {}", e)))?;

        if body.code != 0 {
            return Err(YadigError::Network(format!(
                "Bilibili API error ({}): {}",
                body.code, body.message
            )));
        }

        body.data
            .ok_or_else(|| YadigError::NotFound("Bilibili API returned no data".into()))
    }

    pub async fn sync_library(&self, scope: BiliSyncScope) -> Result<BiliSyncResult> {
        let session = self.auth.session().ok_or_else(|| {
            YadigError::NotFound("Bilibili login is required to sync library".into())
        })?;
        let mid = session.dede_user_id.trim();
        if mid.is_empty() {
            return Err(YadigError::NotFound(
                "Bilibili DedeUserID is required; use QR login or full cookie import".into(),
            ));
        }

        let mut items = Vec::new();
        let mut collections = Vec::new();
        let mut item_collections = Vec::new();
        if scope.favorites {
            let snapshot = self.favorite_library_snapshot(mid).await?;
            items.extend(snapshot.items);
            collections.extend(snapshot.collections);
            item_collections.extend(snapshot.item_collections);
        }
        if scope.watch_later {
            items.extend(self.watch_later_library_items().await?);
        }
        if scope.follows {
            items.extend(self.following_library_items(mid).await?);
        }

        Ok(BiliSyncResult {
            items,
            collections,
            item_collections,
            synced_favorites: scope.favorites,
            synced_follows: scope.follows,
            synced_watch_later: scope.watch_later,
        })
    }

    pub async fn move_favorite_resources(
        &self,
        batch: &FavoriteMoveBatch,
    ) -> std::result::Result<(), FavoriteWriteError> {
        if batch.resources.is_empty() {
            return Ok(());
        }

        let session = favorite_write_session(self.auth.session())?;
        let resp = self
            .http
            .post("https://api.bilibili.com/x/v3/fav/resource/move")
            .header("Referer", "https://www.bilibili.com")
            .header("Origin", "https://www.bilibili.com")
            .header("Cookie", session_cookie(&session))
            .form(&favorite_move_form(&session, batch))
            .send()
            .await
            .map_err(|err| FavoriteWriteError {
                kind: FavoriteWriteErrorKind::Failed,
                message: redact_bili_error(&format!(
                    "Bilibili favorite move request failed: {err}"
                )),
            })?;

        let status = resp.status();
        let body: BiliResponse<serde_json::Value> =
            resp.json().await.map_err(|err| FavoriteWriteError {
                kind: if status.as_u16() == 412 {
                    FavoriteWriteErrorKind::Blocked
                } else {
                    FavoriteWriteErrorKind::Failed
                },
                message: redact_bili_error(&format!(
                    "Bilibili favorite move response parse error: {err}"
                )),
            })?;

        if body.code != 0 {
            return Err(favorite_write_api_error(body.code, &body.message));
        }

        Ok(())
    }

    async fn favorite_library_snapshot(&self, mid: &str) -> Result<FavoriteLibrarySnapshot> {
        let url = format!(
            "https://api.bilibili.com/x/v3/fav/folder/created/list-all?up_mid={}",
            mid
        );
        let data = self.get_bili_data(&url).await?;
        let folders = data
            .get("list")
            .and_then(|value| value.as_array())
            .cloned()
            .unwrap_or_default();
        let mut snapshot = FavoriteLibrarySnapshot::default();

        for folder in folders {
            let Some(collection) = normalize_bili_favorite_folder(folder.clone()) else {
                continue;
            };
            let (items, memberships) = self.favorite_folder_items(&collection, &folder).await?;
            snapshot.collections.push(collection);
            snapshot.items.extend(items);
            snapshot.item_collections.extend(memberships);
        }

        Ok(snapshot)
    }

    async fn favorite_folder_items(
        &self,
        collection: &LibraryCollection,
        folder: &serde_json::Value,
    ) -> Result<(Vec<LibraryItem>, Vec<LibraryItemCollection>)> {
        let mut items = Vec::new();
        let mut memberships = Vec::new();
        for pn in 1..=50 {
            let url = format!(
                "https://api.bilibili.com/x/v3/fav/resource/list?media_id={}&platform=web&pn={}&ps=20",
                collection.external_id, pn
            );
            let data = self.get_bili_data(&url).await?;
            let medias = data
                .get("medias")
                .and_then(|value| value.as_array())
                .cloned()
                .unwrap_or_default();
            for media in medias {
                if let Some(item) = normalize_bili_video(
                    BiliResourceKind::FavoriteVideo,
                    media.clone(),
                    Some(folder),
                ) {
                    if let Some(membership) =
                        normalize_bili_favorite_membership(&item, collection, &media)
                    {
                        memberships.push(membership);
                    }
                    items.push(item);
                }
            }
            if !data
                .get("has_more")
                .and_then(|value| value.as_bool())
                .unwrap_or(false)
            {
                break;
            }
        }
        Ok((items, memberships))
    }

    async fn watch_later_library_items(&self) -> Result<Vec<LibraryItem>> {
        let data = self
            .get_bili_data("https://api.bilibili.com/x/v2/history/toview")
            .await?;
        let videos = data
            .get("list")
            .and_then(|value| value.as_array())
            .cloned()
            .unwrap_or_default();
        Ok(videos
            .into_iter()
            .filter_map(|video| {
                normalize_bili_video(BiliResourceKind::WatchLaterVideo, video, None)
            })
            .collect())
    }

    async fn following_library_items(&self, mid: &str) -> Result<Vec<LibraryItem>> {
        let mut items = Vec::new();
        for pn in 1..=200 {
            let url = format!(
                "https://api.bilibili.com/x/relation/followings?vmid={}&pn={}&ps=50&order_type=",
                mid, pn
            );
            let data = self.get_bili_data(&url).await?;
            let list = data
                .get("list")
                .and_then(|value| value.as_array())
                .cloned()
                .unwrap_or_default();
            if list.is_empty() {
                break;
            }
            for up in list {
                let Some(mid) = up.get("mid").and_then(number_as_string) else {
                    continue;
                };
                let name = up
                    .get("uname")
                    .or_else(|| up.get("name"))
                    .and_then(|value| value.as_str())
                    .unwrap_or("Unknown UP")
                    .to_string();
                items.push(LibraryItem::from_bili_followed_up(mid, name, up));
            }
            let total = data
                .get("total")
                .and_then(|value| value.as_u64())
                .unwrap_or(0);
            if total > 0 && items.len() as u64 >= total {
                break;
            }
        }
        Ok(items)
    }

    /// Fetch video info by BV号.
    pub async fn video_info(&self, bvid: &str) -> Result<VideoInfo> {
        let url = format!(
            "https://api.bilibili.com/x/web-interface/view?bvid={}",
            bvid
        );
        let resp = self
            .request(&url)
            .send()
            .await
            .map_err(|e| YadigError::Network(format!("video_info request failed: {}", e)))?;

        let body: BiliResponse<VideoInfo> = resp
            .json()
            .await
            .map_err(|e| YadigError::Network(format!("video_info parse error: {}", e)))?;

        if body.code != 0 {
            eprintln!(
                "[video_info] ERROR: code={} message='{}'",
                body.code, body.message
            );
            return Err(YadigError::NotFound(format!(
                "Bilibili API error ({}): {}",
                body.code, body.message
            )));
        }

        body.data
            .ok_or_else(|| YadigError::NotFound("video_info returned no data".into()))
    }

    /// Fetch DASH playurl for a specific video part.
    /// Returns the full PlayUrlResponse containing audio streams.
    pub async fn playurl(&self, aid: i64, cid: i64) -> Result<PlayUrlResponse> {
        let url = format!(
            "https://api.bilibili.com/x/player/playurl?avid={}&cid={}&fnval=16&fnver=0&fourk=1",
            aid, cid
        );
        let resp = self
            .request(&url)
            .send()
            .await
            .map_err(|e| YadigError::Network(format!("playurl request failed: {}", e)))?;

        let body: BiliResponse<PlayUrlResponse> = resp
            .json()
            .await
            .map_err(|e| YadigError::Network(format!("playurl parse error: {}", e)))?;

        if body.code != 0 {
            return Err(YadigError::Network(format!(
                "playurl API error ({}): {}",
                body.code, body.message
            )));
        }

        body.data
            .ok_or_else(|| YadigError::NotFound("playurl returned no data".into()))
    }

    /// Fetch player info including view_points (chapters) using WBI signing.
    pub async fn player_info(&self, aid: i64, cid: i64) -> Result<PlayerInfo> {
        let aid_str = aid.to_string();
        let cid_str = cid.to_string();
        let params: &[(&str, &str)] = &[("avid", &aid_str), ("cid", &cid_str)];
        let resp = self
            .signed_get("https://api.bilibili.com/x/player/wbi/v2", params)
            .await?;

        let body: BiliResponse<PlayerInfo> = resp
            .json()
            .await
            .map_err(|e| YadigError::Network(format!("player_info parse error: {}", e)))?;

        if body.code != 0 {
            return Err(YadigError::Network(format!(
                "player_info API error ({}): {}",
                body.code, body.message
            )));
        }

        body.data
            .ok_or_else(|| YadigError::NotFound("player_info returned no data".into()))
    }

    /// Download audio from a Bilibili URL and save to local file.
    /// For multi-part videos (分P), extracts all parts unless a specific ?p=N is given.
    pub async fn extract_audio(
        &self,
        url: &str,
        download_dir: &std::path::Path,
    ) -> Result<ExtractionResult> {
        let bili_url = parse_bilibili_url(url)?;

        match bili_url {
            crate::bili::url::BiliUrl::Video { bvid, page } => {
                let info = self.video_info(&bvid).await?;

                if let Some(p) = page {
                    // Specific page requested — extract single part
                    self.extract_page(&info, p, download_dir).await
                } else if info.pages.len() > 1 {
                    // Multi-part video — extract all pages
                    self.extract_all_pages(&info, download_dir).await
                } else {
                    // Single page video
                    self.extract_page(&info, 1, download_dir).await
                }
            }
            crate::bili::url::BiliUrl::ShortLink { url } => {
                let resp = self
                    .http
                    .get(&url)
                    .header("Referer", "https://www.bilibili.com")
                    .send()
                    .await
                    .map_err(|e| {
                        YadigError::Network(format!("Short link resolve failed: {}", e))
                    })?;
                let resolved_url = resp.url().to_string();
                Box::pin(self.extract_audio(&resolved_url, download_dir)).await
            }
            crate::bili::url::BiliUrl::Collection { mid, season_id } => {
                self.extract_collection(mid, season_id, download_dir).await
            }
        }
    }

    /// Extract audio for a single page (分P) by its page number.
    /// If the page has chapter markers (view_points) and FFmpeg is available,
    /// splits the audio into individual chapter files. Falls back gracefully.
    async fn extract_page(
        &self,
        info: &VideoInfo,
        page_num: u32,
        download_dir: &std::path::Path,
    ) -> Result<ExtractionResult> {
        let page_idx = page_num.saturating_sub(1) as usize;
        let page_info = info
            .pages
            .get(page_idx)
            .ok_or_else(|| YadigError::NotFound(format!("Page {} not found", page_num)))?;

        // Try to detect chapters, but fall back gracefully if unavailable
        let player = self.player_info(info.aid, page_info.cid).await.ok();
        let has_chapters = player
            .as_ref()
            .map(|p| !p.view_points.is_empty())
            .unwrap_or(false);

        if has_chapters && ffmpeg::is_available() {
            let player = player.unwrap(); // safe: has_chapters ensures Some
                                          // Download full audio to temp, then split by chapters
            let temp = ffmpeg::temp_path(download_dir, &info.title);
            let play_resp = self.playurl(info.aid, page_info.cid).await?;
            let dash = play_resp
                .dash
                .ok_or_else(|| YadigError::NotFound("No DASH streams available".into()))?;

            let has_session = self.auth.session().is_some();
            let is_premium = self.auth.is_premium();
            let best = select_best_audio(&dash.audio, has_session, is_premium)
                .ok_or_else(|| YadigError::NotFound("No audio streams available".into()))?;

            self.download_stream(&best.base_url, &temp).await?;

            // Build split segments using make_download_filename for safety
            let segments: Vec<ffmpeg::SplitSegment> = player
                .view_points
                .iter()
                .map(|vp| ffmpeg::SplitSegment {
                    start: vp.from,
                    end: vp.to,
                    output_path: download_dir
                        .join(make_download_filename(
                            &info.title,
                            Some(&vp.content),
                            "m4a",
                        ))
                        .to_string_lossy()
                        .to_string(),
                })
                .collect();

            let output_paths = ffmpeg::split_audio(&temp, &segments)?;

            // Clean up temp file
            let _ = std::fs::remove_file(&temp);

            let audio_segments: Vec<AudioSegment> = player
                .view_points
                .iter()
                .zip(output_paths.iter())
                .map(|(vp, path)| AudioSegment {
                    title: vp.content.clone(),
                    file_path: path.clone(),
                    duration: (vp.to - vp.from) as u32,
                    quality: best.id,
                    audio_url: best.base_url.clone(),
                })
                .collect();

            Ok(ExtractionResult {
                video_title: info.title.clone(),
                segments: audio_segments,
                extraction_type: ExtractionType::Chapters,
            })
        } else {
            // No chapters or FFmpeg not available — extract as single
            let segment = self
                .download_page_audio(info, page_info, download_dir)
                .await?;
            Ok(ExtractionResult {
                video_title: info.title.clone(),
                segments: vec![segment],
                extraction_type: if info.pages.len() > 1 {
                    ExtractionType::MultiPart
                } else {
                    ExtractionType::Single
                },
            })
        }
    }

    /// Extract audio for all pages in a multi-part video.
    async fn extract_all_pages(
        &self,
        info: &VideoInfo,
        download_dir: &std::path::Path,
    ) -> Result<ExtractionResult> {
        let mut segments = Vec::new();
        for page_info in &info.pages {
            let segment = self
                .download_page_audio(info, page_info, download_dir)
                .await?;
            segments.push(segment);
        }

        Ok(ExtractionResult {
            video_title: info.title.clone(),
            segments,
            extraction_type: ExtractionType::MultiPart,
        })
    }

    /// Download audio for a single page and return the segment metadata.
    async fn download_page_audio(
        &self,
        info: &VideoInfo,
        page_info: &Page,
        download_dir: &std::path::Path,
    ) -> Result<AudioSegment> {
        let play_resp = self.playurl(info.aid, page_info.cid).await?;
        let dash = play_resp
            .dash
            .ok_or_else(|| YadigError::NotFound("No DASH streams available".into()))?;

        let has_session = self.auth.session().is_some();
        let is_premium = self.auth.is_premium();
        let best = select_best_audio(&dash.audio, has_session, is_premium)
            .ok_or_else(|| YadigError::NotFound("No audio streams available".into()))?;

        let filename = if info.pages.len() > 1 {
            make_download_filename(&info.title, Some(&page_info.part), "m4a")
        } else {
            make_download_filename(&info.title, None, "m4a")
        };
        let filepath = download_dir.join(&filename);

        self.download_stream(&best.base_url, &filepath).await?;

        Ok(AudioSegment {
            title: page_info.part.clone(),
            file_path: filepath.to_string_lossy().to_string(),
            duration: page_info.duration,
            quality: best.id,
            audio_url: best.base_url.clone(),
        })
    }

    /// Fetch all videos in a collection (合集).
    pub async fn season_archives(&self, mid: i64, season_id: i64) -> Result<SeasonArchives> {
        let url = format!(
            "https://api.bilibili.com/x/polymer/web-space/seasons_archives_list?mid={}&season_id={}&page_num=1&page_size=100",
            mid, season_id
        );
        let resp =
            self.request(&url).send().await.map_err(|e| {
                YadigError::Network(format!("season_archives request failed: {}", e))
            })?;

        let body: BiliResponse<SeasonArchives> = resp
            .json()
            .await
            .map_err(|e| YadigError::Network(format!("season_archives parse error: {}", e)))?;

        if body.code != 0 {
            return Err(YadigError::Network(format!(
                "season_archives API error ({}): {}",
                body.code, body.message
            )));
        }

        body.data
            .ok_or_else(|| YadigError::NotFound("season_archives returned no data".into()))
    }

    /// Extract audio from a collection (合集) — enumerates all videos and downloads each.
    pub async fn extract_collection(
        &self,
        mid: i64,
        season_id: i64,
        download_dir: &std::path::Path,
    ) -> Result<ExtractionResult> {
        let season = self.season_archives(mid, season_id).await?;
        let safe_title = sanitize_filename(&season.meta.name);
        let collection_dir = download_dir.join(&safe_title);

        let mut segments = Vec::new();
        for archive in &season.archives {
            // Fetch video info to get the first page's cid
            let info = self.video_info(&archive.bvid).await?;
            let page_info = info
                .pages
                .first()
                .ok_or_else(|| YadigError::NotFound(format!("No pages in {}", archive.bvid)))?;

            match self
                .download_page_audio(&info, page_info, &collection_dir)
                .await
            {
                Ok(seg) => segments.push(seg),
                Err(e) => {
                    // Log error but continue with other videos
                    eprintln!("Failed to extract {}: {}", archive.bvid, e);
                }
            }
        }

        Ok(ExtractionResult {
            video_title: season.meta.name,
            segments,
            extraction_type: ExtractionType::Collection,
        })
    }

    /// Extract audio for a specific segment by cid (for selective extraction).
    pub async fn extract_segment(
        &self,
        bvid: &str,
        cid: i64,
        _title: &str,
        download_dir: &std::path::Path,
    ) -> Result<ExtractionResult> {
        let info = self.video_info(bvid).await?;
        let page_info = info
            .pages
            .iter()
            .find(|p| p.cid == cid)
            .ok_or_else(|| YadigError::NotFound(format!("Page with cid {} not found", cid)))?;

        let segment = self
            .download_page_audio(&info, page_info, download_dir)
            .await?;

        Ok(ExtractionResult {
            video_title: info.title.clone(),
            segments: vec![segment],
            extraction_type: ExtractionType::MultiPart,
        })
    }

    /// Download a stream URL to a local file, remuxing from fMP4 to standard MP4.
    /// Bilibili DASH audio is fragmented MP4 — remuxing ensures compatibility with
    /// players that don't support fMP4 (GPAC, deadbeef, foobar2000, etc.).
    async fn download_stream(&self, url: &str, path: &std::path::Path) -> Result<()> {
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| YadigError::Network(format!("Create dir error: {}", e)))?;
        }

        // Use a short temp file name to avoid ENAMETOOLONG
        let temp_dir = path.parent().unwrap_or(std::path::Path::new("."));
        let temp_path = temp_dir.join(".yadig_dl_tmp");

        let resp = self
            .http
            .get(url)
            .header("Referer", "https://www.bilibili.com")
            .send()
            .await
            .map_err(|e| YadigError::Network(format!("Download failed: {}", e)))?;

        if !resp.status().is_success() {
            return Err(YadigError::Network(format!(
                "Download HTTP error: {}",
                resp.status()
            )));
        }

        let bytes = resp
            .bytes()
            .await
            .map_err(|e| YadigError::Network(format!("Download read error: {}", e)))?;

        std::fs::write(&temp_path, &bytes)
            .map_err(|e| YadigError::Network(format!("File write error: {}", e)))?;

        // Remux from fMP4 to standard MP4 container
        let result = ffmpeg::remux_to_standard_mp4(&temp_path, path);

        // Clean up temp file regardless of remux result
        let _ = std::fs::remove_file(&temp_path);

        // If remux failed, fall back to saving the raw download directly
        if result.is_err() {
            std::fs::write(path, &bytes)
                .map_err(|e| YadigError::Network(format!("File write error: {}", e)))?;
        }

        Ok(())
    }
}

/// Sanitize a string for use as a filename.
/// Also truncates to avoid "File name too long" errors on some filesystems.
/// EXT4 max filename is 255 bytes. We reserve 50 bytes for path prefix and extensions.
fn sanitize_filename(name: &str) -> String {
    let mut safe: String = name
        .chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            c => c,
        })
        .collect::<String>()
        .trim()
        .to_string();

    // Truncate to 180 bytes to leave room for extension and path prefix
    if safe.len() > 180 {
        let mut truncated = String::new();
        for c in safe.chars() {
            let c_len = c.len_utf8();
            if truncated.len() + c_len > 180 {
                break;
            }
            truncated.push(c);
        }
        safe = truncated;
    }

    safe
}

/// Build a safe filename for downloading. Sanitizes special chars and truncates.
/// This is the canonical entry point for all download filenames.
fn make_download_filename(title: &str, part: Option<&str>, ext: &str) -> String {
    let safe_title = sanitize_filename(title);
    let filename = match part {
        Some(p) => {
            let safe_part = sanitize_filename(p);
            format!("{} - {}.{}", safe_title, safe_part, ext)
        }
        None => format!("{}.{}", safe_title, ext),
    };
    // Final safety: ensure the full filename component is under 255 bytes
    if filename.len() > 200 {
        // Truncate from the base title, keeping part and extension
        let max_title_bytes = 200usize.saturating_sub(
            part.map(|p| sanitize_filename(p).len() + 5).unwrap_or(5) + ext.len() + 1,
        );
        let mut truncated_title = String::new();
        for c in safe_title.chars() {
            if truncated_title.len() + c.len_utf8() > max_title_bytes {
                break;
            }
            truncated_title.push(c);
        }
        match part {
            Some(p) => format!("{} - {}.{}", truncated_title, sanitize_filename(p), ext),
            None => format!("{}.{}", truncated_title, ext),
        }
    } else {
        filename
    }
}

fn session_cookie(session: &crate::bili::auth::BiliSession) -> String {
    let mut parts = vec![format!("SESSDATA={}", session.sessdata)];
    if !session.bili_jct.is_empty() {
        parts.push(format!("bili_jct={}", session.bili_jct));
    }
    if !session.dede_user_id.is_empty() {
        parts.push(format!("DedeUserID={}", session.dede_user_id));
    }
    parts.join("; ")
}

fn favorite_write_session(
    session: Option<BiliSession>,
) -> std::result::Result<BiliSession, FavoriteWriteError> {
    let Some(session) = session else {
        return Err(FavoriteWriteError {
            kind: FavoriteWriteErrorKind::Blocked,
            message: "Bilibili write operations require SESSDATA, bili_jct, and DedeUserID"
                .to_string(),
        });
    };

    if session.sessdata.trim().is_empty()
        || session.bili_jct.trim().is_empty()
        || session.dede_user_id.trim().is_empty()
    {
        return Err(FavoriteWriteError {
            kind: FavoriteWriteErrorKind::Blocked,
            message: "Bilibili write operations require SESSDATA, bili_jct, and DedeUserID"
                .to_string(),
        });
    }

    Ok(session)
}

fn favorite_move_form(
    session: &BiliSession,
    batch: &FavoriteMoveBatch,
) -> BTreeMap<String, String> {
    let resources = batch
        .resources
        .iter()
        .map(|resource| format!("{}:{}", resource.id, resource.resource_type))
        .collect::<Vec<_>>()
        .join(",");

    BTreeMap::from([
        ("src_media_id".to_string(), batch.source_media_id.clone()),
        ("tar_media_id".to_string(), batch.target_media_id.clone()),
        ("mid".to_string(), session.dede_user_id.clone()),
        ("resources".to_string(), resources),
        ("platform".to_string(), "web".to_string()),
        ("csrf".to_string(), session.bili_jct.clone()),
    ])
}

fn favorite_write_api_error(code: i32, message: &str) -> FavoriteWriteError {
    let blocking = matches!(code, -101 | -111 | -400 | 412)
        || message.contains("csrf")
        || message.contains("CSRF")
        || message.contains("验证码")
        || message.contains("风控")
        || message.to_ascii_lowercase().contains("captcha")
        || message.contains("未登录")
        || message.contains("账号");
    let kind = if blocking {
        FavoriteWriteErrorKind::Blocked
    } else {
        FavoriteWriteErrorKind::Failed
    };
    let prefix = if blocking {
        "Bilibili favorite write blocked"
    } else {
        "Bilibili favorite write failed"
    };

    FavoriteWriteError {
        kind,
        message: redact_bili_error(&format!("{prefix} ({code}): {message}")),
    }
}

fn redact_bili_error(message: &str) -> String {
    regex::Regex::new(r"(SESSDATA|bili_jct|DedeUserID)=([^&;\s]+)")
        .map(|re| re.replace_all(message, "$1=<redacted>").to_string())
        .unwrap_or_else(|_| message.to_string())
}

fn number_as_string(value: &serde_json::Value) -> Option<String> {
    value
        .as_str()
        .map(ToString::to_string)
        .or_else(|| value.as_i64().map(|n| n.to_string()))
        .or_else(|| value.as_u64().map(|n| n.to_string()))
}

fn normalize_bili_favorite_folder(folder: serde_json::Value) -> Option<LibraryCollection> {
    let media_id = folder.get("id").and_then(number_as_string)?;
    let title = folder
        .get("title")
        .and_then(|value| value.as_str())
        .unwrap_or("Untitled favorite folder")
        .to_string();

    Some(LibraryCollection::from_bili_favorite_folder(
        media_id, title, folder,
    ))
}

fn normalize_bili_video(
    kind: BiliResourceKind,
    media: serde_json::Value,
    folder: Option<&serde_json::Value>,
) -> Option<LibraryItem> {
    let bvid = media
        .get("bvid")
        .or_else(|| media.get("bv_id"))
        .and_then(|value| value.as_str())
        .map(ToString::to_string)
        .or_else(|| {
            media
                .get("uri")
                .and_then(|value| value.as_str())
                .and_then(extract_bvid_from_text)
        })?;
    let title = media
        .get("title")
        .and_then(|value| value.as_str())
        .unwrap_or("Untitled Bilibili video")
        .to_string();
    let author = media
        .get("upper")
        .and_then(|upper| upper.get("name"))
        .or_else(|| media.get("author_name"))
        .or_else(|| media.get("owner").and_then(|owner| owner.get("name")))
        .and_then(|value| value.as_str())
        .map(ToString::to_string);
    let mut raw_metadata = media;
    if let Some(folder) = folder {
        let source_folder_id = folder.get("id").and_then(number_as_string);
        raw_metadata["collection"] = serde_json::json!({
            "id": folder.get("id").cloned().unwrap_or(serde_json::Value::Null),
            "title": folder.get("title").cloned().unwrap_or(serde_json::Value::Null),
        });
        raw_metadata["favorite"] = serde_json::json!({
            "resourceId": raw_metadata.get("id").and_then(number_as_string),
            "resourceType": raw_metadata.get("type").and_then(number_as_string),
            "bvid": bvid,
            "sourceFolderId": source_folder_id,
            "sourceFolderTitle": folder.get("title").and_then(|value| value.as_str()),
            "favTime": raw_metadata.get("fav_time").and_then(|value| value.as_i64()),
        });
    }

    Some(LibraryItem::from_bili_video(
        kind,
        bvid,
        title,
        author,
        raw_metadata,
    ))
}

fn normalize_bili_favorite_membership(
    item: &LibraryItem,
    collection: &LibraryCollection,
    media: &serde_json::Value,
) -> Option<LibraryItemCollection> {
    let resource_id = media.get("id").and_then(number_as_string)?;
    let resource_type = media.get("type").and_then(number_as_string)?;

    Some(LibraryItemCollection {
        source: "bilibili".to_string(),
        item_external_id: item.external_id.clone(),
        item_type: item.item_type.clone(),
        collection_external_id: collection.external_id.clone(),
        collection_type: collection.collection_type.clone(),
        raw_metadata: serde_json::json!({
            "resourceId": resource_id,
            "resourceType": resource_type,
            "bvid": item.external_id,
            "sourceFolderId": collection.external_id,
            "sourceFolderTitle": collection.title,
            "favTime": media.get("fav_time").and_then(|value| value.as_i64()),
        }),
    })
}

fn extract_bvid_from_text(text: &str) -> Option<String> {
    text.split(|c: char| !c.is_ascii_alphanumeric())
        .find(|part| part.starts_with("BV") && part.len() >= 10)
        .map(ToString::to_string)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_filename_removes_special_chars() {
        assert_eq!(sanitize_filename("Test: Song/Name"), "Test_ Song_Name");
        assert_eq!(sanitize_filename("Normal Title"), "Normal Title");
        assert_eq!(sanitize_filename("A*B?C"), "A_B_C");
    }

    #[test]
    fn sanitize_filename_preserves_unicode() {
        assert_eq!(sanitize_filename("测试歌曲"), "测试歌曲");
        assert_eq!(sanitize_filename("Bilibilié"), "Bilibilié");
    }

    #[test]
    fn multipart_filename_format() {
        // Multi-part: "{title} - {part}.m4a"
        let title = sanitize_filename("My Album");
        let part = sanitize_filename("Track 1");
        let filename = format!("{} - {}.m4a", title, part);
        assert_eq!(filename, "My Album - Track 1.m4a");
    }

    #[test]
    fn single_filename_format() {
        // Single: "{title}.m4a"
        let title = sanitize_filename("My Song");
        let filename = format!("{}.m4a", title);
        assert_eq!(filename, "My Song.m4a");
    }

    #[test]
    fn multipart_filename_sanitizes_both_parts() {
        let title = sanitize_filename("Album: Vol. 1");
        let part = sanitize_filename("Track 1/2");
        let filename = format!("{} - {}.m4a", title, part);
        assert_eq!(filename, "Album_ Vol. 1 - Track 1_2.m4a");
    }

    #[test]
    fn normalizes_favorite_folder_into_library_collection() {
        let folder = serde_json::json!({
            "id": 456,
            "title": "Samples",
            "media_count": 2
        });

        let collection = normalize_bili_favorite_folder(folder).expect("folder should normalize");

        assert_eq!(collection.source, "bilibili");
        assert_eq!(collection.external_id, "456");
        assert_eq!(
            collection.collection_type,
            crate::library::LibraryCollectionType::BiliFavoriteFolder
        );
        assert_eq!(collection.title, "Samples");
        assert_eq!(collection.raw_metadata["media_count"].as_i64(), Some(2));
    }

    #[test]
    fn normalizes_favorite_video_membership_with_remote_identity() {
        let folder = serde_json::json!({
            "id": 456,
            "title": "Samples"
        });
        let collection =
            normalize_bili_favorite_folder(folder.clone()).expect("folder should normalize");
        let media = serde_json::json!({
            "id": 987654321,
            "type": 2,
            "bvid": "BV1remote1234",
            "title": "Remote favorite video",
            "upper": { "name": "Music UP" },
            "fav_time": 1781070000
        });

        let item = normalize_bili_video(
            BiliResourceKind::FavoriteVideo,
            media.clone(),
            Some(&folder),
        )
        .expect("video should normalize");
        let membership = normalize_bili_favorite_membership(&item, &collection, &media)
            .expect("membership should normalize");

        assert_eq!(item.external_id, "BV1remote1234");
        assert_eq!(
            item.raw_metadata["favorite"]["resourceId"].as_str(),
            Some("987654321")
        );
        assert_eq!(
            item.raw_metadata["favorite"]["resourceType"].as_str(),
            Some("2")
        );
        assert_eq!(
            item.raw_metadata["favorite"]["sourceFolderId"].as_str(),
            Some("456")
        );
        assert_eq!(membership.item_external_id, "BV1remote1234");
        assert_eq!(membership.collection_external_id, "456");
        assert_eq!(
            membership.raw_metadata["resourceId"].as_str(),
            Some("987654321")
        );
        assert_eq!(membership.raw_metadata["resourceType"].as_str(), Some("2"));
        assert_eq!(
            membership.raw_metadata["bvid"].as_str(),
            Some("BV1remote1234")
        );
        assert_eq!(
            membership.raw_metadata["sourceFolderId"].as_str(),
            Some("456")
        );
        assert_eq!(
            membership.raw_metadata["favTime"].as_i64(),
            Some(1781070000)
        );
    }

    #[test]
    fn requires_complete_session_for_favorite_writes() {
        let sessdata_only = crate::bili::auth::BiliSession {
            sessdata: "sess-secret".to_string(),
            bili_jct: String::new(),
            dede_user_id: "42".to_string(),
            vip_status: 0,
        };

        let err = favorite_write_session(Some(sessdata_only)).expect_err("csrf is required");

        assert_eq!(err.kind, FavoriteWriteErrorKind::Blocked);
        assert!(err.message.contains("SESSDATA, bili_jct, and DedeUserID"));
        assert!(!err.message.contains("sess-secret"));
        assert!(!err.message.contains("42"));
    }

    #[test]
    fn builds_favorite_move_form_for_resource_pairs() {
        let session = crate::bili::auth::BiliSession {
            sessdata: "sess-secret".to_string(),
            bili_jct: "csrf-secret".to_string(),
            dede_user_id: "233333".to_string(),
            vip_status: 0,
        };
        let batch = FavoriteMoveBatch {
            source_media_id: "100".to_string(),
            target_media_id: "200".to_string(),
            resources: vec![
                FavoriteMoveResource {
                    id: "987654321".to_string(),
                    resource_type: "2".to_string(),
                },
                FavoriteMoveResource {
                    id: "123456789".to_string(),
                    resource_type: "2".to_string(),
                },
            ],
        };

        let form = favorite_move_form(&session, &batch);

        assert_eq!(form.get("src_media_id").map(String::as_str), Some("100"));
        assert_eq!(form.get("tar_media_id").map(String::as_str), Some("200"));
        assert_eq!(form.get("mid").map(String::as_str), Some("233333"));
        assert_eq!(
            form.get("resources").map(String::as_str),
            Some("987654321:2,123456789:2")
        );
        assert_eq!(form.get("platform").map(String::as_str), Some("web"));
        assert_eq!(form.get("csrf").map(String::as_str), Some("csrf-secret"));
    }

    #[test]
    fn classifies_security_favorite_write_errors_as_blocking_and_redacted() {
        let err = favorite_write_api_error(
            -111,
            "csrf failed SESSDATA=sess-secret&bili_jct=csrf-secret&DedeUserID=233333",
        );

        assert_eq!(err.kind, FavoriteWriteErrorKind::Blocked);
        assert!(err.message.contains("Bilibili favorite write blocked"));
        assert!(!err.message.contains("sess-secret"));
        assert!(!err.message.contains("csrf-secret"));
        assert!(!err.message.contains("233333"));
    }
}
