pub mod types;

use std::path::PathBuf;
use std::sync::Mutex;

use crate::error::{Result, YadigError};
use crate::youtube::types::*;

/// Check if yt-dlp is available in PATH.
pub fn is_available() -> bool {
    std::process::Command::new("yt-dlp")
        .arg("--version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Client for YouTube audio extraction.
///
/// Uses the yt-dlp CLI (external binary) to fetch video info and download audio.
/// Pattern matches the existing FFmpeg usage in ffmpeg.rs.
#[derive(Clone)]
pub struct YoutubeClient {
    output_dir: PathBuf,
    ready: std::sync::Arc<Mutex<bool>>,
}

impl YoutubeClient {
    pub fn new(output_dir: PathBuf) -> Self {
        Self {
            output_dir,
            ready: std::sync::Arc::new(Mutex::new(false)),
        }
    }

    /// Check if yt-dlp is installed and ready.
    pub fn ensure_initialized(&self) -> Result<()> {
        let mut ready = self.ready.lock().unwrap();
        if *ready {
            return Ok(());
        }

        if !is_available() {
            return Err(YadigError::NotFound(
                "yt-dlp is not installed. Install it first:\n  pip install yt-dlp\nor: brew install yt-dlp\nor: winget install yt-dlp"
                    .into(),
            ));
        }

        std::fs::create_dir_all(&self.output_dir)
            .map_err(|e| YadigError::Network(format!("Failed to create output dir: {}", e)))?;

        *ready = true;
        eprintln!("[youtube] yt-dlp is ready");
        Ok(())
    }

    /// Extract audio from a YouTube URL.
    /// Downloads the best available audio stream and saves it to the output directory.
    pub async fn extract_audio(&self, url: &str) -> Result<YoutubeExtractionResult> {
        self.ensure_initialized()?;

        // Step 1: Fetch video info as JSON
        let info = self.fetch_video_info(url)?;
        let video_title = info_get_string(&info, "title").unwrap_or("Unknown");
        let duration = info_get_f64(&info, "duration").unwrap_or(0.0);
        let thumbnail_url = info_get_string(&info, "thumbnail").map(|s| s.to_string());
        let has_chapters = info["chapters"].is_array() && !info["chapters"].as_array().unwrap().is_empty();

        let safe_title = sanitize_filename(video_title);
        let filename = format!("{}.mp3", safe_title);
        let output_path = self.output_dir.join(&filename);

        // Step 2: Download audio if not already cached
        if !output_path.exists() {
            eprintln!("[youtube] Downloading audio: {} -> {:?}", video_title, output_path);

            let status = tokio::task::spawn_blocking({
                let output = output_path.clone();
                let url = url.to_string();
                move || {
                    std::process::Command::new("yt-dlp")
                        .arg("-x") // extract audio
                        .arg("--audio-format").arg("mp3")
                        .arg("-o").arg(&output)
                        .arg("--no-playlist")
                        .arg("--no-warnings")
                        .arg(&url)
                        .stdout(std::process::Stdio::null())
                        .stderr(std::process::Stdio::piped())
                        .output()
                }
            })
            .await
            .map_err(|e| YadigError::Network(format!("yt-dlp spawn error: {}", e)))?
            .map_err(|e| YadigError::Network(format!("yt-dlp execution error: {}", e)))?;

            if !status.status.success() {
                let stderr = String::from_utf8_lossy(&status.stderr);
                return Err(YadigError::Network(format!("yt-dlp failed: {}", stderr)));
            }
        }

        let file_path = output_path.to_string_lossy().to_string();

        let segment = YoutubeAudioSegment {
            title: video_title.to_string(),
            file_path: file_path.clone(),
            duration,
            audio_url: file_path.clone(),
            ext: "mp3".to_string(),
        };

        Ok(YoutubeExtractionResult {
            video_title: video_title.to_string(),
            video_url: url.to_string(),
            thumbnail_url,
            duration,
            segments: vec![segment],
            has_chapters,
        })
    }

    /// Fetch video metadata as JSON using `yt-dlp --dump-json`.
    fn fetch_video_info(&self, url: &str) -> Result<serde_json::Value> {
        let output = std::process::Command::new("yt-dlp")
            .arg("--dump-json")
            .arg("--no-playlist")
            .arg("--no-warnings")
            .arg(url)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .output()
            .map_err(|e| YadigError::Network(format!("yt-dlp execution error: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(YadigError::Network(format!(
                "yt-dlp error: {}\n\nIf you're behind a firewall, set HTTP_PROXY/HTTPS_PROXY env vars, e.g.:\n  export HTTPS_PROXY=http://127.0.0.1:7890",
                stderr.trim()
            )));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        serde_json::from_str(&stdout)
            .map_err(|e| YadigError::Network(format!("Failed to parse yt-dlp output: {}", e)))
    }

    /// Search YouTube for videos.
    pub async fn search(&self, query: &str, limit: usize) -> Result<Vec<crate::source::types::ContentItem>> {
        self.ensure_initialized()?;

        let search_query = format!("ytsearch{}:{}", limit, query);

        let output = tokio::task::spawn_blocking({
            let sq = search_query.clone();
            move || {
                std::process::Command::new("yt-dlp")
                    .arg("--dump-json")
                    .arg("--no-playlist")
                    .arg("--no-warnings")
                    .arg("--flat-playlist")
                    .arg(&sq)
                    .stdout(std::process::Stdio::piped())
                    .stderr(std::process::Stdio::null())
                    .output()
            }
        })
        .await
        .map_err(|e| YadigError::Network(format!("yt-dlp spawn error: {}", e)))?
        .map_err(|e| YadigError::Network(format!("yt-dlp execution error: {}", e)))?;

        if !output.status.success() {
            return Ok(Vec::new());
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut items = Vec::new();

        for line in stdout.lines() {
            if let Ok(info) = serde_json::from_str::<serde_json::Value>(line) {
                let title = info_get_string(&info, "title").unwrap_or("Unknown");
                let video_url = info_get_string(&info, "webpage_url")
                    .or_else(|| info_get_string(&info, "url"))
                    .unwrap_or("");
                let author = info_get_string(&info, "uploader")
                    .or_else(|| info_get_string(&info, "channel"));
                let duration = info_get_f64(&info, "duration");
                let image_url = info_get_string(&info, "thumbnail");

                items.push(crate::source::types::ContentItem {
                    source_id: "youtube".to_string(),
                    title: title.to_string(),
                    url: video_url.to_string(),
                    summary: None,
                    author: author.map(|s| s.to_string()),
                    published_at: info_get_string(&info, "upload_date").map(|s| s.to_string()),
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

/// Helper: get a string field from JSON, handling various formats.
fn info_get_string<'a>(info: &'a serde_json::Value, key: &str) -> Option<&'a str> {
    info.get(key).and_then(|v| v.as_str())
}

/// Helper: get a f64 field from JSON.
fn info_get_f64(info: &serde_json::Value, key: &str) -> Option<f64> {
    info.get(key).and_then(|v| v.as_f64())
}

/// Sanitize a string for use as a filename.
fn sanitize_filename(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            c if c.is_control() => '_',
            c => c,
        })
        .collect::<String>()
        .trim()
        .to_string()
}
