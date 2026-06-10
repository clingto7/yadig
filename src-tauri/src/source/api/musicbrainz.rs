use async_trait::async_trait;
use crate::error::{Result, YadigError};
use crate::source::provider::SourceProvider;
use crate::source::types::*;

/// MusicBrainz search source — uses the official MusicBrainz API.
///
/// Docs: https://musicbrainz.org/doc/MusicBrainz_API
/// No API key required. Rate limit: 1 request/second.
/// Searches artists, releases, and recordings.
pub struct MusicBrainzSource {
    client: reqwest::Client,
}

impl MusicBrainzSource {
    pub fn new() -> Self {
        Self {
            client: crate::http_client::build_client("yadig/0.1.0 (music discovery)"),
        }
    }

    /// Search artists on MusicBrainz.
    async fn search_artists(&self, query: &str, limit: usize) -> Result<Vec<ContentItem>> {
        let url = format!(
            "https://musicbrainz.org/ws/2/artist/?query={}&fmt=json&limit={}",
            urlencoding::encode(query),
            limit.min(25)
        );

        let resp = self.client.get(&url)
            .header("Accept", "application/json")
            .send()
            .await
            .map_err(|e| YadigError::Network(format!("MusicBrainz request error: {}", e)))?;

        let data: serde_json::Value = resp.json().await
            .map_err(|e| YadigError::Network(format!("MusicBrainz parse error: {}", e)))?;

        let artists = data["artists"].as_array().map(|a| a.clone()).unwrap_or_default();
        let mut items = Vec::new();

        for (i, artist) in artists.iter().enumerate() {
            let id = artist["id"].as_str().unwrap_or("");
            let name = artist["name"].as_str().unwrap_or("Unknown");
            let disambig = artist["disambiguation"].as_str();
            let country = artist["country"].as_str();
            let tags: Vec<String> = artist["tags"].as_array()
                .map(|t| t.iter().filter_map(|t| t["name"].as_str().map(String::from)).collect())
                .unwrap_or_default();

            let mut summary_parts = Vec::new();
            if let Some(c) = country { summary_parts.push(format!("🇺🇳 {}", c)); }
            if !tags.is_empty() { summary_parts.push(tags.join(", ")); }
            if let Some(d) = disambig { summary_parts.push(d.to_string()); }

            let summary = if summary_parts.is_empty() { None } else { Some(summary_parts.join(" · ")) };

            // Use MusicBrainz as cover art source if available
            let image_url = format!("https://commons.wikimedia.org/w/index.php?title=Special:Redirect/file/MusicBrainz_logo.svg&width=100");

            items.push(ContentItem {
                source_id: "musicbrainz".to_string(),
                title: name.to_string(),
                url: format!("https://musicbrainz.org/artist/{}", id),
                summary,
                author: None,
                published_at: None,
                image_url: Some(image_url),
                audio_url: None,
                download_url: None,
                duration: None,
                license: None,
                extra: None,
                relevance_score: Some(0.9 - (i as f32 * 0.05)),
            });
        }

        Ok(items)
    }

    /// Search releases (albums) on MusicBrainz.
    async fn search_releases(&self, query: &str, limit: usize) -> Result<Vec<ContentItem>> {
        let url = format!(
            "https://musicbrainz.org/ws/2/release/?query={}&fmt=json&limit={}",
            urlencoding::encode(query),
            limit.min(25)
        );

        let resp = self.client.get(&url)
            .header("Accept", "application/json")
            .send()
            .await
            .map_err(|e| YadigError::Network(format!("MusicBrainz request error: {}", e)))?;

        let data: serde_json::Value = resp.json().await
            .map_err(|e| YadigError::Network(format!("MusicBrainz parse error: {}", e)))?;

        let releases = data["releases"].as_array().map(|a| a.clone()).unwrap_or_default();
        let mut items = Vec::new();

        for (i, release) in releases.iter().enumerate() {
            let id = release["id"].as_str().unwrap_or("");
            let title = release["title"].as_str().unwrap_or("Unknown");
            let artist = release["artist-credit"]
                .as_array()
                .and_then(|c| c.first())
                .and_then(|c| c["name"].as_str())
                .or_else(|| release["artist-credit-phrase"].as_str());
            let date = release["date"].as_str();
            let country = release["country"].as_str();
            let track_count = release["track-count"].as_u64();
            let status = release["status"].as_str();

            let mut summary_parts = Vec::new();
            if let Some(d) = date { summary_parts.push(d.to_string()); }
            if let Some(c) = country { summary_parts.push(c.to_string()); }
            if let Some(t) = track_count { summary_parts.push(format!("{} tracks", t)); }
            if let Some(s) = status { summary_parts.push(s.to_string()); }

            items.push(ContentItem {
                source_id: "musicbrainz".to_string(),
                title: format!("{} — {}", artist.unwrap_or("Unknown"), title),
                url: format!("https://musicbrainz.org/release/{}", id),
                summary: if summary_parts.is_empty() { None } else { Some(summary_parts.join(" · ")) },
                author: artist.map(|s| s.to_string()),
                published_at: date.map(|s| s.to_string()),
                image_url: None, // Cover Art Archive requires separate API call
                audio_url: None,
                download_url: None,
                duration: None,
                license: None,
                extra: None,
                relevance_score: Some(0.85 - (i as f32 * 0.04)),
            });
        }

        Ok(items)
    }
}

#[async_trait]
impl SourceProvider for MusicBrainzSource {
    fn id(&self) -> &str { "musicbrainz" }
    fn name(&self) -> &str { "MusicBrainz" }
    fn kind(&self) -> SourceKind { SourceKind::Api }
    fn base_url(&self) -> &str { "https://musicbrainz.org" }

    async fn search(&self, query: &str, limit: usize, _page: usize) -> Result<Vec<ContentItem>> {
        let half = limit / 2 + 1;
        let mut items = self.search_artists(query, half).await?;
        let releases = self.search_releases(query, half).await?;
        items.extend(releases);
        items.truncate(limit);
        Ok(items)
    }

    async fn fetch_latest(&self, _limit: usize) -> Result<Vec<ContentItem>> {
        // MusicBrainz doesn't have a "latest" feed endpoint.
        Ok(Vec::new())
    }
}
