use crate::bili::auth::BiliAuth;
use crate::bili::ffmpeg;
use crate::bili::types::*;
use crate::bili::wbi::{self, WbiKeys};
use crate::bili::extractor::{select_best_audio, AudioSegment, ExtractionResult, ExtractionType};
use crate::bili::url::parse_bilibili_url;
use crate::error::{Result, YadigError};
use tokio::sync::Mutex;

/// Bilibili API client. Handles HTTP requests, auth cookies, and response parsing.
pub struct BiliClient {
    http: reqwest::Client,
    auth: BiliAuth,
    wbi_keys: Mutex<Option<WbiKeys>>,
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
        let mut req = self.http.get(url)
            .header("Referer", "https://www.bilibili.com");
        if let Some(session) = self.auth.session() {
            req = req.header("Cookie", format!("SESSDATA={}", session.sessdata));
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
    async fn signed_get(&self, base_url: &str, params: &[(&str, &str)]) -> Result<reqwest::Response> {
        let keys = self.ensure_wbi_keys().await?;
        let (w_rid, wts) = wbi::sign_params(params, &keys);

        let query: String = params.iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join("&");
        let url = format!("{}?{}&w_rid={}&wts={}", base_url, query, w_rid, wts);

        let mut req = self.http.get(&url)
            .header("Referer", "https://www.bilibili.com");
        if let Some(session) = self.auth.session() {
            req = req.header("Cookie", format!("SESSDATA={}", session.sessdata));
        }
        req.send().await.map_err(|e| YadigError::Network(format!("HTTP request failed: {}", e)))
    }

    /// Fetch video info by BV号.
    pub async fn video_info(&self, bvid: &str) -> Result<VideoInfo> {
        let url = format!(
            "https://api.bilibili.com/x/web-interface/view?bvid={}",
            bvid
        );
        let resp = self.request(&url).send().await
            .map_err(|e| YadigError::Network(format!("video_info request failed: {}", e)))?;

        let body: BiliResponse<VideoInfo> = resp.json().await
            .map_err(|e| YadigError::Network(format!("video_info parse error: {}", e)))?;

        if body.code != 0 {
            eprintln!("[video_info] ERROR: code={} message='{}'", body.code, body.message);
            return Err(YadigError::NotFound(format!(
                "Bilibili API error ({}): {}",
                body.code, body.message
            )));
        }

        body.data.ok_or_else(|| YadigError::NotFound("video_info returned no data".into()))
    }

    /// Fetch DASH playurl for a specific video part.
    /// Returns the full PlayUrlResponse containing audio streams.
    pub async fn playurl(&self, aid: i64, cid: i64) -> Result<PlayUrlResponse> {
        let url = format!(
            "https://api.bilibili.com/x/player/playurl?avid={}&cid={}&fnval=16&fnver=0&fourk=1",
            aid, cid
        );
        let resp = self.request(&url).send().await
            .map_err(|e| YadigError::Network(format!("playurl request failed: {}", e)))?;

        let body: BiliResponse<PlayUrlResponse> = resp.json().await
            .map_err(|e| YadigError::Network(format!("playurl parse error: {}", e)))?;

        if body.code != 0 {
            return Err(YadigError::Network(format!(
                "playurl API error ({}): {}", body.code, body.message
            )));
        }

        body.data.ok_or_else(|| YadigError::NotFound("playurl returned no data".into()))
    }

    /// Fetch player info including view_points (chapters) using WBI signing.
    pub async fn player_info(&self, aid: i64, cid: i64) -> Result<PlayerInfo> {
        let aid_str = aid.to_string();
        let cid_str = cid.to_string();
        let params: &[(&str, &str)] = &[("avid", &aid_str), ("cid", &cid_str)];
        let resp = self.signed_get(
            "https://api.bilibili.com/x/player/wbi/v2",
            params,
        ).await?;

        let body: BiliResponse<PlayerInfo> = resp.json().await
            .map_err(|e| YadigError::Network(format!("player_info parse error: {}", e)))?;

        if body.code != 0 {
            return Err(YadigError::Network(format!(
                "player_info API error ({}): {}", body.code, body.message
            )));
        }

        body.data.ok_or_else(|| YadigError::NotFound("player_info returned no data".into()))
    }

    /// Download audio from a Bilibili URL and save to local file.
    /// For multi-part videos (分P), extracts all parts unless a specific ?p=N is given.
    pub async fn extract_audio(&self, url: &str, download_dir: &std::path::Path) -> Result<ExtractionResult> {
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
                let resp = self.http.get(&url)
                    .header("Referer", "https://www.bilibili.com")
                    .send().await
                    .map_err(|e| YadigError::Network(format!("Short link resolve failed: {}", e)))?;
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
    async fn extract_page(&self, info: &VideoInfo, page_num: u32, download_dir: &std::path::Path) -> Result<ExtractionResult> {
        let page_idx = page_num.saturating_sub(1) as usize;
        let page_info = info.pages.get(page_idx)
            .ok_or_else(|| YadigError::NotFound(format!("Page {} not found", page_num)))?;

        // Try to detect chapters, but fall back gracefully if unavailable
        let player = self.player_info(info.aid, page_info.cid).await.ok();
        let has_chapters = player.as_ref()
            .map(|p| !p.view_points.is_empty())
            .unwrap_or(false);

        if has_chapters && ffmpeg::is_available() {
            let player = player.unwrap(); // safe: has_chapters ensures Some
            // Download full audio to temp, then split by chapters
            let temp = ffmpeg::temp_path(download_dir, &info.title);
            let play_resp = self.playurl(info.aid, page_info.cid).await?;
            let dash = play_resp.dash
                .ok_or_else(|| YadigError::NotFound("No DASH streams available".into()))?;

            let has_session = self.auth.session().is_some();
            let is_premium = self.auth.is_premium();
            let best = select_best_audio(&dash.audio, has_session, is_premium)
                .ok_or_else(|| YadigError::NotFound("No audio streams available".into()))?;

            self.download_stream(&best.base_url, &temp).await?;

            // Build split segments
            let safe_title = crate::bili::client::sanitize_filename(&info.title);
            let segments: Vec<ffmpeg::SplitSegment> = player.view_points.iter().map(|vp| {
                let safe_chapter = crate::bili::client::sanitize_filename(&vp.content);
                ffmpeg::SplitSegment {
                    start: vp.from,
                    end: vp.to,
                    output_path: download_dir.join(format!("{} - {}.m4a", safe_title, safe_chapter))
                        .to_string_lossy().to_string(),
                }
            }).collect();

            let output_paths = ffmpeg::split_audio(&temp, &segments)?;

            // Clean up temp file
            let _ = std::fs::remove_file(&temp);

            let audio_segments: Vec<AudioSegment> = player.view_points.iter().zip(output_paths.iter()).map(|(vp, path)| {
                AudioSegment {
                    title: vp.content.clone(),
                    file_path: path.clone(),
                    duration: (vp.to - vp.from) as u32,
                    quality: best.id,
                    audio_url: best.base_url.clone(),
                }
            }).collect();

            Ok(ExtractionResult {
                video_title: info.title.clone(),
                segments: audio_segments,
                extraction_type: ExtractionType::Chapters,
            })
        } else {
            // No chapters or FFmpeg not available — extract as single
            let segment = self.download_page_audio(info, page_info, download_dir).await?;
            Ok(ExtractionResult {
                video_title: info.title.clone(),
                segments: vec![segment],
                extraction_type: if info.pages.len() > 1 { ExtractionType::MultiPart } else { ExtractionType::Single },
            })
        }
    }

    /// Extract audio for all pages in a multi-part video.
    async fn extract_all_pages(&self, info: &VideoInfo, download_dir: &std::path::Path) -> Result<ExtractionResult> {
        let mut segments = Vec::new();
        for page_info in &info.pages {
            let segment = self.download_page_audio(info, page_info, download_dir).await?;
            segments.push(segment);
        }

        Ok(ExtractionResult {
            video_title: info.title.clone(),
            segments,
            extraction_type: ExtractionType::MultiPart,
        })
    }

    /// Download audio for a single page and return the segment metadata.
    async fn download_page_audio(&self, info: &VideoInfo, page_info: &Page, download_dir: &std::path::Path) -> Result<AudioSegment> {
        let play_resp = self.playurl(info.aid, page_info.cid).await?;
        let dash = play_resp.dash
            .ok_or_else(|| YadigError::NotFound("No DASH streams available".into()))?;

        let has_session = self.auth.session().is_some();
        let is_premium = self.auth.is_premium();
        let best = select_best_audio(&dash.audio, has_session, is_premium)
            .ok_or_else(|| YadigError::NotFound("No audio streams available".into()))?;

        let safe_title = sanitize_filename(&info.title);
        let safe_part = sanitize_filename(&page_info.part);
        let filename = if info.pages.len() > 1 {
            format!("{} - {}.m4a", safe_title, safe_part)
        } else {
            format!("{}.m4a", safe_title)
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
        let resp = self.request(&url).send().await
            .map_err(|e| YadigError::Network(format!("season_archives request failed: {}", e)))?;

        let body: BiliResponse<SeasonArchives> = resp.json().await
            .map_err(|e| YadigError::Network(format!("season_archives parse error: {}", e)))?;

        if body.code != 0 {
            return Err(YadigError::Network(format!(
                "season_archives API error ({}): {}", body.code, body.message
            )));
        }

        body.data.ok_or_else(|| YadigError::NotFound("season_archives returned no data".into()))
    }

    /// Extract audio from a collection (合集) — enumerates all videos and downloads each.
    pub async fn extract_collection(&self, mid: i64, season_id: i64, download_dir: &std::path::Path) -> Result<ExtractionResult> {
        let season = self.season_archives(mid, season_id).await?;
        let safe_title = sanitize_filename(&season.meta.name);
        let collection_dir = download_dir.join(&safe_title);

        let mut segments = Vec::new();
        for archive in &season.archives {
            // Fetch video info to get the first page's cid
            let info = self.video_info(&archive.bvid).await?;
            let page_info = info.pages.first()
                .ok_or_else(|| YadigError::NotFound(format!("No pages in {}", archive.bvid)))?;

            match self.download_page_audio(&info, page_info, &collection_dir).await {
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
    pub async fn extract_segment(&self, bvid: &str, cid: i64, _title: &str, download_dir: &std::path::Path) -> Result<ExtractionResult> {
        let info = self.video_info(bvid).await?;
        let page_info = info.pages.iter().find(|p| p.cid == cid)
            .ok_or_else(|| YadigError::NotFound(format!("Page with cid {} not found", cid)))?;

        let segment = self.download_page_audio(&info, page_info, download_dir).await?;

        Ok(ExtractionResult {
            video_title: info.title.clone(),
            segments: vec![segment],
            extraction_type: ExtractionType::MultiPart,
        })
    }

    /// Download a stream URL to a local file.
    async fn download_stream(&self, url: &str, path: &std::path::Path) -> Result<()> {
        let resp = self.http.get(url)
            .header("Referer", "https://www.bilibili.com")
            .send().await
            .map_err(|e| YadigError::Network(format!("Download failed: {}", e)))?;

        if !resp.status().is_success() {
            return Err(YadigError::Network(format!("Download HTTP error: {}", resp.status())));
        }

        let bytes = resp.bytes().await
            .map_err(|e| YadigError::Network(format!("Download read error: {}", e)))?;

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| YadigError::Network(format!("Create dir error: {}", e)))?;
        }

        std::fs::write(path, &bytes)
            .map_err(|e| YadigError::Network(format!("File write error: {}", e)))?;

        Ok(())
    }
}

/// Sanitize a string for use as a filename.
fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            c => c,
        })
        .collect::<String>()
        .trim()
        .to_string()
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
}
