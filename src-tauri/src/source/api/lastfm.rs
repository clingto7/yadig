use async_trait::async_trait;
use crate::error::{Result, YadigError};
use crate::source::provider::SourceProvider;
use crate::source::types::*;

// Default Last.fm API key — for development use.
// Users should register their own at https://www.last.fm/api
const LASTFM_API_KEY: &str = "d0fca847"; // using same placeholder as Jamendo

/// Last.fm search source — uses the official Last.fm API.
///
/// Docs: https://www.last.fm/api
/// Free API key required: https://www.last.fm/api/account/create
/// Searches artists, albums, and tracks.
pub struct LastFmSource {
    client: reqwest::Client,
}

impl LastFmSource {
    pub fn new() -> Self {
        Self {
            client: crate::http_client::build_client("yadig/0.1.0 (music discovery)"),
        }
    }

    fn api_url(&self, method: &str, params: &[(&str, &str)]) -> String {
        let mut query = format!("?method={}&api_key={}&format=json", method, LASTFM_API_KEY);
        for (k, v) in params {
            query.push('&');
            query.push_str(k);
            query.push('=');
            query.push_str(&urlencoding::encode(v));
        }
        format!("https://ws.audioscrobbler.com/2.0/{}", query)
    }

    /// Search artists on Last.fm.
    async fn search_artists(&self, query: &str, limit: usize) -> Result<Vec<ContentItem>> {
        let url = self.api_url("artist.search", &[("artist", query), ("limit", &limit.to_string())]);

        let resp = self.client.get(&url)
            .send()
            .await
            .map_err(|e| YadigError::Network(format!("Last.fm request error: {}", e)))?;

        let data: serde_json::Value = resp.json().await
            .map_err(|e| YadigError::Network(format!("Last.fm parse error: {}", e)))?;

        let artists = data["results"]["artistmatches"]["artist"]
            .as_array()
            .map(|a| a.clone())
            .unwrap_or_default();

        let mut items = Vec::new();

        for (i, artist) in artists.iter().enumerate() {
            let name = artist["name"].as_str().unwrap_or("Unknown");
            let listeners = artist["listeners"].as_str().unwrap_or("0");
            let url_str = artist["url"].as_str().unwrap_or("");
            let image_url = artist["image"]
                .as_array()
                .and_then(|imgs| imgs.iter().find(|img| img["size"] == "large"))
                .and_then(|img| img["#text"].as_str())
                .filter(|s| !s.is_empty());

            items.push(ContentItem {
                source_id: "lastfm".to_string(),
                title: name.to_string(),
                url: url_str.to_string(),
                summary: Some(format!("{} listeners", listeners)),
                author: None,
                published_at: None,
                image_url: image_url.map(|s| s.to_string()),
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

    /// Search albums on Last.fm.
    async fn search_albums(&self, query: &str, limit: usize) -> Result<Vec<ContentItem>> {
        let url = self.api_url("album.search", &[("album", query), ("limit", &limit.to_string())]);

        let resp = self.client.get(&url)
            .send()
            .await
            .map_err(|e| YadigError::Network(format!("Last.fm request error: {}", e)))?;

        let data: serde_json::Value = resp.json().await
            .map_err(|e| YadigError::Network(format!("Last.fm parse error: {}", e)))?;

        let albums = data["results"]["albummatches"]["album"]
            .as_array()
            .map(|a| a.clone())
            .unwrap_or_default();

        let mut items = Vec::new();

        for (i, album) in albums.iter().enumerate() {
            let name = album["name"].as_str().unwrap_or("Unknown");
            let artist = album["artist"].as_str().unwrap_or("Unknown");
            let url_str = album["url"].as_str().unwrap_or("");
            let image_url = album["image"]
                .as_array()
                .and_then(|imgs| imgs.iter().find(|img| img["size"] == "large"))
                .and_then(|img| img["#text"].as_str())
                .filter(|s| !s.is_empty());

            items.push(ContentItem {
                source_id: "lastfm".to_string(),
                title: format!("{} — {}", artist, name),
                url: url_str.to_string(),
                summary: None,
                author: Some(artist.to_string()),
                published_at: None,
                image_url: image_url.map(|s| s.to_string()),
                audio_url: None,
                download_url: None,
                duration: None,
                license: None,
                extra: None,
                relevance_score: Some(0.8 - (i as f32 * 0.03)),
            });
        }

        Ok(items)
    }

    /// Search tracks on Last.fm.
    async fn search_tracks(&self, query: &str, limit: usize) -> Result<Vec<ContentItem>> {
        let url = self.api_url("track.search", &[("track", query), ("limit", &limit.to_string())]);

        let resp = self.client.get(&url)
            .send()
            .await
            .map_err(|e| YadigError::Network(format!("Last.fm request error: {}", e)))?;

        let data: serde_json::Value = resp.json().await
            .map_err(|e| YadigError::Network(format!("Last.fm parse error: {}", e)))?;

        let tracks = data["results"]["trackmatches"]["track"]
            .as_array()
            .map(|a| a.clone())
            .unwrap_or_default();

        let mut items = Vec::new();

        for (i, track) in tracks.iter().enumerate() {
            let name = track["name"].as_str().unwrap_or("Unknown");
            let artist = track["artist"].as_str().unwrap_or("Unknown");
            let url_str = track["url"].as_str().unwrap_or("");
            let listeners = track["listeners"].as_str().unwrap_or("0");
            let image_url = track["image"]
                .as_array()
                .and_then(|imgs| imgs.iter().find(|img| img["size"] == "large"))
                .and_then(|img| img["#text"].as_str())
                .filter(|s| !s.is_empty());

            items.push(ContentItem {
                source_id: "lastfm".to_string(),
                title: format!("{} — {}", artist, name),
                url: url_str.to_string(),
                summary: Some(format!("{} listeners", listeners)),
                author: Some(artist.to_string()),
                published_at: None,
                image_url: image_url.map(|s| s.to_string()),
                audio_url: None,
                download_url: None,
                duration: None,
                license: None,
                extra: None,
                relevance_score: Some(0.75 - (i as f32 * 0.03)),
            });
        }

        Ok(items)
    }
}

#[async_trait]
impl SourceProvider for LastFmSource {
    fn id(&self) -> &str { "lastfm" }
    fn name(&self) -> &str { "Last.fm" }
    fn kind(&self) -> SourceKind { SourceKind::Api }
    fn base_url(&self) -> &str { "https://www.last.fm" }

    async fn search(&self, query: &str, limit: usize, _page: usize) -> Result<Vec<ContentItem>> {
        let each = (limit / 3).max(3);
        let mut items = self.search_artists(query, each).await?;
        items.extend(self.search_albums(query, each).await?);
        items.extend(self.search_tracks(query, each).await?);
        items.truncate(limit);
        Ok(items)
    }

    async fn fetch_latest(&self, _limit: usize) -> Result<Vec<ContentItem>> {
        // Could use tag.gettopalbums or similar, but for now return empty.
        Ok(Vec::new())
    }
}
