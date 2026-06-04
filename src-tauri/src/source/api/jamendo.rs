use async_trait::async_trait;
use crate::error::{Result, YadigError};
use crate::http_client;
use crate::source::provider::SourceProvider;
use crate::source::types::*;

// Jamendo developer client_id — register at https://devportal.jamendo.com
// TODO: move to Arc<RwLock<Option<String>>> + settings UI for user-provided keys
const JAMENDO_CLIENT_ID: &str = "d0fca847";

/// Jamendo source — uses the official Jamendo API v3.0
/// Docs: https://developer.jamendo.com/v3.0
/// 600k+ independent tracks, Creative Commons licensed
pub struct JamendoSource {
    client: reqwest::Client,
}

impl JamendoSource {
    pub fn new() -> Self {
        Self { client: http_client::build_client("yadig/0.1.0 (music discovery)") }
    }

    fn parse_tracks(data: &serde_json::Value, limit: usize, base_score: f32) -> Vec<ContentItem> {
        let empty = vec![];
        let tracks = data["results"].as_array().unwrap_or(&empty);

        tracks
            .iter()
            .enumerate()
            .filter_map(|(i, t)| {
                let id = t["id"].as_str()?;
                let name = t["name"].as_str()?.to_string();
                let artist_name = t["artist_name"].as_str().map(String::from);

                let title = match &artist_name {
                    Some(a) => format!("{} — {}", a, name),
                    None => name,
                };

                let page_url = t["shareurl"].as_str()
                    .map(String::from)
                    .unwrap_or_else(|| format!("https://www.jamendo.com/track/{}", id));

                let audio_url = t["audio"].as_str().map(String::from);
                let download_url = t["audiodownload"].as_str().and_then(|u| {
                    if u.is_empty() { None } else { Some(u.to_string()) }
                });

                let image_url = t["image"].as_str()
                    .or_else(|| t["album_image"].as_str())
                    .map(String::from);

                let duration = t["duration"].as_u64().map(|d| d as u32);
                let license = t["license_ccurl"].as_str().map(String::from);
                let releasedate = t["releasedate"].as_str().map(String::from);
                let album_name = t["album_name"].as_str().map(String::from);

                // Build extra from musicinfo tags
                let mut extra = serde_json::Map::new();
                if let Some(genres) = t["musicinfo"]["tags"]["genres"].as_array() {
                    let genre_list: Vec<String> = genres.iter()
                        .filter_map(|g| g.as_str().map(String::from))
                        .collect();
                    if !genre_list.is_empty() {
                        extra.insert("genres".to_string(), serde_json::Value::Array(
                            genre_list.into_iter().map(serde_json::Value::String).collect()
                        ));
                    }
                }
                if let Some(speed) = t["musicinfo"]["speed"].as_str() {
                    extra.insert("speed".to_string(), serde_json::Value::String(speed.to_string()));
                }
                if let Some(vocal) = t["musicinfo"]["vocalinstrumental"].as_str() {
                    extra.insert("vocal_instrumental".to_string(), serde_json::Value::String(vocal.to_string()));
                }

                Some(ContentItem {
                    source_id: "jamendo".to_string(),
                    title,
                    url: page_url,
                    summary: album_name,
                    author: artist_name,
                    published_at: releasedate,
                    image_url,
                    audio_url,
                    download_url,
                    duration,
                    license,
                    relevance_score: Some(base_score - (i as f32 * 0.02)),
                    extra: if extra.is_empty() { None } else { Some(serde_json::Value::Object(extra)) },
                })
            })
            .take(limit)
            .collect()
    }
}

#[async_trait]
impl SourceProvider for JamendoSource {
    fn id(&self) -> &str { "jamendo" }
    fn name(&self) -> &str { "Jamendo" }
    fn kind(&self) -> SourceKind { SourceKind::Api }
    fn base_url(&self) -> &str { "https://www.jamendo.com" }

    async fn search(&self, query: &str, limit: usize, page: usize) -> Result<Vec<ContentItem>> {
        let offset = (page.saturating_sub(1)) * limit;
        let url = format!(
            "https://api.jamendo.com/v3.0/tracks/?client_id={}&search={}&format=json&include=musicinfo&audioformat=mp32&limit={}&offset={}",
            JAMENDO_CLIENT_ID,
            urlencoding::encode(query),
            limit.min(200),
            offset
        );

        let response = self.client.get(&url).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(YadigError::Network(format!("Jamendo API error {}: {}", status, body)));
        }

        let data: serde_json::Value = response.json().await
            .map_err(|e| YadigError::Network(format!("Jamendo JSON parse error: {}", e)))?;

        if data["headers"]["status"].as_str() != Some("success") {
            let msg = data["headers"]["error_message"].as_str().unwrap_or("unknown error");
            return Err(YadigError::Network(format!("Jamendo API error: {}", msg)));
        }

        Ok(Self::parse_tracks(&data, limit, 0.8))
    }

    async fn fetch_latest(&self, limit: usize) -> Result<Vec<ContentItem>> {
        let url = format!(
            "https://api.jamendo.com/v3.0/tracks/?client_id={}&format=json&include=musicinfo&audioformat=mp32&limit={}&order=popularity_week",
            JAMENDO_CLIENT_ID,
            limit.min(200)
        );

        let response = self.client.get(&url).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            return Err(YadigError::Network(format!("Jamendo API error: {}", status)));
        }

        let data: serde_json::Value = response.json().await
            .map_err(|e| YadigError::Network(format!("Jamendo JSON parse error: {}", e)))?;

        Ok(Self::parse_tracks(&data, limit, 0.75))
    }
}

// Minimal URL encoding — same as in discogs.rs
mod urlencoding {
    pub fn encode(s: &str) -> String {
        s.chars().map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' || c == '.' || c == '~' {
                c.to_string()
            } else {
                format!("%{:02X}", c as u8)
            }
        }).collect()
    }
}
