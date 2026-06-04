use async_trait::async_trait;
use crate::config::DiscogsKeys;
use crate::error::{Result, YadigError};
use crate::http_client;
use crate::source::provider::SourceProvider;
use crate::source::types::*;

/// Discogs source — uses the official REST API
/// Docs: https://www.discogs.com/developers
pub struct DiscogsSource {
    client: reqwest::Client,
    keys: DiscogsKeys,
}

impl DiscogsSource {
    pub fn new(keys: DiscogsKeys) -> Self {
        let client = http_client::build_client("yadig/0.1.0 (music discovery)");
        Self { client, keys }
    }

    fn search_url(&self, query: &str, r#type: &str, limit: usize, page: usize) -> String {
        let mut url = format!(
            "https://api.discogs.com/database/search?q={}&type={}&per_page={}&page={}",
            urlencoding::encode(query),
            r#type,
            limit,
            page
        );
        if let Some(key) = &*self.keys.key.read().unwrap() {
            url.push_str(&format!("&key={}", key));
        }
        if let Some(secret) = &*self.keys.secret.read().unwrap() {
            url.push_str(&format!("&secret={}", secret));
        }
        url
    }
}

#[async_trait]
impl SourceProvider for DiscogsSource {
    fn id(&self) -> &str { "discogs" }
    fn name(&self) -> &str { "Discogs" }
    fn kind(&self) -> SourceKind { SourceKind::Api }
    fn base_url(&self) -> &str { "https://www.discogs.com" }

    async fn search(&self, query: &str, limit: usize, page: usize) -> Result<Vec<ContentItem>> {
        let mut all_items = Vec::new();

        // 1. Search for artists first (highest relevance)
        let artist_url = self.search_url(query, "artist", 3, 1);
        if let Ok(resp) = self.client.get(&artist_url).send().await {
            if resp.status().is_success() {
                if let Ok(data) = resp.json::<serde_json::Value>().await {
                    let empty = vec![];
                    let results = data["results"].as_array().unwrap_or(&empty);
                    for (i, r) in results.iter().enumerate() {
                        let title = r["title"].as_str().unwrap_or("").to_string();
                        let id = r["id"].as_i64().unwrap_or(0);
                        let url = format!("https://www.discogs.com/artist/{}", id);
                        let image_url = r["cover_image"].as_str().map(String::from);
                        all_items.push(ContentItem {
                            source_id: "discogs".to_string(),
                            title: format!("{} (Artist)", title),
                            url,
                            summary: Some("Artist".to_string()),
                            author: None,
                            published_at: None,
                            image_url,
                            audio_url: None,
                            download_url: None,
                            duration: None,
                            license: None,
                            relevance_score: Some(1.0 - (i as f32 * 0.05)),
                            extra: None,
                        });
                    }
                }
            }
        }

        // 2. Search for releases
        let url = self.search_url(query, "release", limit, page);
        let response = self.client.get(&url).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(YadigError::Discogs(format!("API error {}: {}", status, body)));
        }

        let data: serde_json::Value = response.json().await?;
        let empty = vec![];
        let results = data["results"].as_array().unwrap_or(&empty);

        let release_items: Vec<ContentItem> = results
            .iter()
            .enumerate()
            .filter_map(|(i, r)| {
                let title = r["title"].as_str()?.to_string();
                let id = r["id"].as_i64()?;
                let url = format!("https://www.discogs.com/release/{}", id);

                let image_url = r["cover_image"].as_str().map(String::from);
                let year = r["year"].as_str().map(String::from);
                let genre = r["genre"].as_array().map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect::<Vec<_>>()
                });

                let mut extra = serde_json::Map::new();
                if let Some(y) = year {
                    extra.insert("year".to_string(), serde_json::Value::String(y));
                }
                if let Some(g) = genre {
                    extra.insert("genres".to_string(), serde_json::Value::Array(
                        g.into_iter().map(serde_json::Value::String).collect()
                    ));
                }

                Some(ContentItem {
                    source_id: "discogs".to_string(),
                    title,
                    url,
                    summary: None,
                    author: None,
                    published_at: None,
                    image_url,
                    audio_url: None,
                    download_url: None,
                    duration: None,
                    license: None,
                    relevance_score: Some(0.7 - (i as f32 * 0.02)),
                    extra: Some(serde_json::Value::Object(extra)),
                })
            })
            .take(limit)
            .collect();

        all_items.extend(release_items);
        all_items.truncate(limit + 3); // Allow a few extra for artist results
        Ok(all_items)
    }

    async fn fetch_latest(&self, _limit: usize) -> Result<Vec<ContentItem>> {
        // Discogs API doesn't have a "latest" endpoint — return empty
        Ok(Vec::new())
    }
}

// Minimal URL encoding — avoids adding a dependency just for this
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
