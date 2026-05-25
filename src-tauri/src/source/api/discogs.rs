use async_trait::async_trait;
use crate::error::{Result, YadigError};
use crate::source::provider::SourceProvider;
use crate::source::types::*;

/// Discogs source — uses the official REST API
/// Docs: https://www.discogs.com/developers
pub struct DiscogsSource {
    client: reqwest::Client,
    consumer_key: Option<String>,
    consumer_secret: Option<String>,
}

impl DiscogsSource {
    pub fn new(consumer_key: Option<String>, consumer_secret: Option<String>) -> Self {
        let client = reqwest::Client::builder()
            .user_agent("yadig/0.1.0 (music discovery)")
            .build()
            .expect("Failed to build HTTP client");
        Self {
            client,
            consumer_key,
            consumer_secret,
        }
    }

    fn auth_header(&self) -> Option<String> {
        // Discogs uses key/secret in query params, not headers
        None
    }

    fn search_url(&self, query: &str, r#type: &str, limit: usize) -> String {
        let mut url = format!(
            "https://api.discogs.com/database/search?q={}&type={}&per_page={}",
            urlencoding::encode(query),
            r#type,
            limit
        );
        if let Some(key) = &self.consumer_key {
            url.push_str(&format!("&key={}", key));
        }
        if let Some(secret) = &self.consumer_secret {
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

    async fn search(&self, query: &str, limit: usize) -> Result<Vec<ContentItem>> {
        let url = self.search_url(query, "release", limit);
        let response = self.client.get(&url).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(YadigError::Discogs(format!("API error {}: {}", status, body)));
        }

        let data: serde_json::Value = response.json().await?;
        let empty = vec![];
        let results = data["results"].as_array().unwrap_or(&empty);

        let items: Vec<ContentItem> = results
            .iter()
            .filter_map(|r| {
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
                    extra: Some(serde_json::Value::Object(extra)),
                })
            })
            .take(limit)
            .collect();

        Ok(items)
    }

    async fn fetch_latest(&self, _limit: usize) -> Result<Vec<ContentItem>> {
        // Discogs API doesn't have a "latest" endpoint — return empty
        // User would need to search by keyword or browse specific lists
        Ok(Vec::new())
    }

    async fn get_item(&self, url: &str) -> Result<ContentItem> {
        // Extract release ID from URL and fetch details
        let id = url.split('/').last().ok_or_else(|| {
            YadigError::NotFound(format!("Invalid Discogs URL: {}", url))
        })?;

        let mut api_url = format!("https://api.discogs.com/releases/{}", id);
        if let Some(key) = &self.consumer_key {
            api_url.push_str(&format!("?key={}", key));
        }
        if let Some(secret) = &self.consumer_secret {
            api_url.push_str(&format!("&secret={}", secret));
        }

        let response = self.client.get(&api_url).send().await?;

        if !response.status().is_success() {
            return Err(YadigError::Discogs(format!(
                "API error fetching release {}", id
            )));
        }

        let data: serde_json::Value = response.json().await?;

        let title = data["title"].as_str().unwrap_or("Unknown").to_string();
        let artist = data["artists"].as_array()
            .and_then(|a| a.first())
            .and_then(|a| a["name"].as_str())
            .map(String::from);

        Ok(ContentItem {
            source_id: "discogs".to_string(),
            title,
            url: url.to_string(),
            summary: None,
            author: artist,
            published_at: data["year"].as_str().map(String::from),
            image_url: data["images"].as_array()
                .and_then(|imgs| imgs.first())
                .and_then(|img| img["uri"].as_str())
                .map(String::from),
            extra: Some(data),
        })
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
