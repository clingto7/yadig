use async_trait::async_trait;
use crate::error::{Result, YadigError};
use crate::source::provider::SourceProvider;
use crate::source::types::*;

/// YouTube search source — uses yt-dlp CLI for searching.
///
/// Searches YouTube via `yt-dlp ytsearchN:query --dump-json` and
/// returns results as ContentItems. Requires yt-dlp to be installed.
pub struct YouTubeSource;

impl YouTubeSource {
    pub fn new() -> Self {
        Self
    }

    fn check_ytdlp() -> Result<()> {
        let status = std::process::Command::new("yt-dlp")
            .arg("--version")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .map_err(|_| YadigError::Network("yt-dlp not found".into()))?;

        if !status.success() {
            return Err(YadigError::NotFound(
                "yt-dlp is not installed. Install it first:\n  pipx install yt-dlp\nor: sudo apt install yt-dlp".into(),
            ));
        }
        Ok(())
    }

    fn search_ytdlp(query: &str, limit: usize) -> Result<Vec<ContentItem>> {
        let search_query = format!("ytsearch{}:{}", limit, query);

        let output = std::process::Command::new("yt-dlp")
            .arg("--dump-json")
            .arg("--no-playlist")
            .arg("--no-warnings")
            .arg("--flat-playlist")
            .arg(&search_query)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::null())
            .output()
            .map_err(|e| YadigError::Network(format!("yt-dlp error: {}", e)))?;

        if !output.status.success() {
            return Ok(Vec::new());
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut items = Vec::new();

        for line in stdout.lines() {
            if let Ok(info) = serde_json::from_str::<serde_json::Value>(line) {
                let title = info.get("title").and_then(|v| v.as_str()).unwrap_or("Unknown");
                let video_url = info.get("webpage_url")
                    .and_then(|v| v.as_str())
                    .or_else(|| info.get("url").and_then(|v| v.as_str()))
                    .unwrap_or("");
                let author = info.get("uploader").and_then(|v| v.as_str())
                    .or_else(|| info.get("channel").and_then(|v| v.as_str()));
                let duration = info.get("duration").and_then(|v| v.as_f64());
                let image_url = info.get("thumbnail").and_then(|v| v.as_str());
                let view_count = info.get("view_count").and_then(|v| v.as_u64());

                // Build a richer summary from channel + views
                let summary = author.map(|a| {
                    let views = view_count.map(|v| format!("{} views", v)).unwrap_or_default();
                    format!("{} — {}", a, views)
                });

                items.push(ContentItem {
                    source_id: "youtube".to_string(),
                    title: title.to_string(),
                    url: video_url.to_string(),
                    summary,
                    author: author.map(|s| s.to_string()),
                    published_at: info.get("upload_date").and_then(|v| v.as_str()).map(|s| s.to_string()),
                    image_url: image_url.map(|s| s.to_string()),
                    audio_url: None,
                    download_url: None,
                    duration: duration.map(|d| d as u32),
                    license: None,
                    extra: None,
                    relevance_score: Some(0.5),
                });
            }
        }

        Ok(items)
    }
}

#[async_trait]
impl SourceProvider for YouTubeSource {
    fn id(&self) -> &str { "youtube" }
    fn name(&self) -> &str { "YouTube" }
    fn kind(&self) -> SourceKind { SourceKind::Api }
    fn base_url(&self) -> &str { "https://www.youtube.com" }

    async fn search(&self, query: &str, limit: usize, _page: usize) -> Result<Vec<ContentItem>> {
        Self::check_ytdlp()?;
        Self::search_ytdlp(query, limit)
    }

    async fn fetch_latest(&self, _limit: usize) -> Result<Vec<ContentItem>> {
        // YouTube doesn't have a "latest" endpoint like RSS feeds;
        // returning empty is consistent with Discogs pattern.
        Ok(Vec::new())
    }
}
