use crate::bili::auth::BiliAuth;
use crate::bili::types::*;
use crate::bili::extractor::{select_best_audio, AudioSegment, ExtractionResult, ExtractionType};
use crate::bili::url::parse_bilibili_url;
use crate::error::{Result, YadigError};

/// Bilibili API client. Handles HTTP requests, auth cookies, and response parsing.
pub struct BiliClient {
    http: reqwest::Client,
    auth: BiliAuth,
}

impl BiliClient {
    pub fn new(auth: BiliAuth) -> Self {
        Self {
            http: crate::http_client::build_client("yadig/0.1.0"),
            auth,
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
            return Err(YadigError::NotFound(format!(
                "Bilibili API error ({}): {}", body.code, body.message
            )));
        }

        body.data.ok_or_else(|| YadigError::NotFound("video_info returned no data".into()))
    }

    /// Fetch DASH playurl for a specific video part.
    /// Returns the full PlayUrlResponse containing audio streams.
    pub async fn playurl(&self, aid: i64, cid: i64) -> Result<PlayUrlResponse> {
        let url = format!(
            "https://api.bilibili.com/x/player/wbi/playurl?avid={}&cid={}&fnval=16&fnver=0&fourk=1",
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

    /// Download audio from a Bilibili URL and save to local file.
    /// Returns the extraction result with file paths.
    pub async fn extract_audio(&self, url: &str, download_dir: &std::path::Path) -> Result<ExtractionResult> {
        let bili_url = parse_bilibili_url(url)?;

        match bili_url {
            crate::bili::url::BiliUrl::Video { bvid, page } => {
                let info = self.video_info(&bvid).await?;
                let page_idx = page.unwrap_or(1).saturating_sub(1) as usize;
                let page_info = info.pages.get(page_idx)
                    .ok_or_else(|| YadigError::NotFound(format!("Page {} not found", page_idx + 1)))?;

                let play_resp = self.playurl(info.aid, page_info.cid).await?;
                let dash = play_resp.dash
                    .ok_or_else(|| YadigError::NotFound("No DASH streams available".into()))?;

                let has_session = self.auth.session().is_some();
                let is_premium = self.auth.is_premium();
                let best = select_best_audio(&dash.audio, has_session, is_premium)
                    .ok_or_else(|| YadigError::NotFound("No audio streams available".into()))?;

                // Download audio
                let safe_title = sanitize_filename(&info.title);
                let filename = format!("{}.m4a", safe_title);
                let filepath = download_dir.join(&filename);

                self.download_stream(&best.base_url, &filepath).await?;

                Ok(ExtractionResult {
                    video_title: info.title,
                    segments: vec![AudioSegment {
                        title: page_info.part.clone(),
                        file_path: filepath.to_string_lossy().to_string(),
                        duration: page_info.duration,
                        quality: best.id,
                        audio_url: best.base_url.clone(),
                    }],
                    extraction_type: ExtractionType::Single,
                })
            }
            crate::bili::url::BiliUrl::ShortLink { url } => {
                // Resolve short link by following redirect
                let resp = self.http.get(&url)
                    .header("Referer", "https://www.bilibili.com")
                    .send().await
                    .map_err(|e| YadigError::Network(format!("Short link resolve failed: {}", e)))?;
                let resolved_url = resp.url().to_string();
                Box::pin(self.extract_audio(&resolved_url, download_dir)).await
            }
            _ => Err(YadigError::NotFound(
                "URL is not a single video. Use bili_extract_collection for collections.".into()
            )),
        }
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
}
