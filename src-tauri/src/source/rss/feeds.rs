use async_trait::async_trait;
use crate::error::Result;
use crate::http_client;
use crate::source::provider::SourceProvider;
use crate::source::types::*;

/// Stereogum source — RSS feed
pub struct StereogumSource {
    client: reqwest::Client,
}

impl StereogumSource {
    pub fn new() -> Self {
        Self { client: http_client::build_client("yadig/0.1.0 (music discovery)") }
    }
}

#[async_trait]
impl SourceProvider for StereogumSource {
    fn id(&self) -> &str { "stereogum" }
    fn name(&self) -> &str { "Stereogum" }
    fn kind(&self) -> SourceKind { SourceKind::Rss }
    fn base_url(&self) -> &str { "https://www.stereogum.com" }

    async fn search(&self, query: &str, limit: usize, _page: usize) -> Result<Vec<ContentItem>> {
        let items = self.fetch_latest(100).await?;
        let query_lower = query.to_lowercase();
        Ok(items.into_iter()
            .filter(|item| {
                item.title.to_lowercase().contains(&query_lower)
                    || item.summary.as_ref().map_or(false, |s| s.to_lowercase().contains(&query_lower))
            })
            .take(limit)
            .collect())
    }

    async fn fetch_latest(&self, limit: usize) -> Result<Vec<ContentItem>> {
        let response = self.client.get("https://www.stereogum.com/feed/").send().await?;
        let body = response.text().await?;
        let channel = rss::Channel::read_from(body.as_bytes())
            .map_err(|e| crate::error::YadigError::Feed(format!("Stereogum RSS parse error: {}", e)))?;

        Ok(channel.items.into_iter().filter_map(|item| {
            let title = item.title?;
            let link = item.link?;
            // Extract image from media:thumbnail or enclosure
            let image_url = item.extensions.iter()
                .find_map(|(ns, exts)| {
                    if ns == "media" {
                        exts.get("thumbnail")?.first()?.attrs.get("url").cloned()
                    } else { None }
                });
            Some(ContentItem {
                source_id: "stereogum".to_string(),
                title,
                url: link,
                summary: item.description,
                author: item.author,
                published_at: item.pub_date,
                image_url,
                audio_url: None,
                download_url: None,
                duration: None,
                license: None,
                relevance_score: None,
                extra: None,
            })
        }).take(limit).collect())
    }
}

/// The Fader source — RSS feed
pub struct FaderSource {
    client: reqwest::Client,
}

impl FaderSource {
    pub fn new() -> Self {
        Self { client: http_client::build_client("yadig/0.1.0 (music discovery)") }
    }
}

#[async_trait]
impl SourceProvider for FaderSource {
    fn id(&self) -> &str { "fader" }
    fn name(&self) -> &str { "The Fader" }
    fn kind(&self) -> SourceKind { SourceKind::Rss }
    fn base_url(&self) -> &str { "https://www.thefader.com" }

    async fn search(&self, query: &str, limit: usize, _page: usize) -> Result<Vec<ContentItem>> {
        let items = self.fetch_latest(100).await?;
        let query_lower = query.to_lowercase();
        Ok(items.into_iter()
            .filter(|item| {
                item.title.to_lowercase().contains(&query_lower)
                    || item.summary.as_ref().map_or(false, |s| s.to_lowercase().contains(&query_lower))
            })
            .take(limit)
            .collect())
    }

    async fn fetch_latest(&self, limit: usize) -> Result<Vec<ContentItem>> {
        let response = self.client.get("https://www.thefader.com/feed").send().await?;
        let body = response.text().await?;
        let channel = rss::Channel::read_from(body.as_bytes())
            .map_err(|e| crate::error::YadigError::Feed(format!("Fader RSS parse error: {}", e)))?;

        Ok(channel.items.into_iter().filter_map(|item| {
            let title = item.title?;
            let link = item.link?;
            let image_url = item.extensions.iter()
                .find_map(|(ns, exts)| {
                    if ns == "media" {
                        exts.get("thumbnail")?.first()?.attrs.get("url").cloned()
                    } else { None }
                });
            Some(ContentItem {
                source_id: "fader".to_string(),
                title,
                url: link,
                summary: item.description,
                author: item.author,
                published_at: item.pub_date,
                image_url,
                audio_url: None,
                download_url: None,
                duration: None,
                license: None,
                relevance_score: None,
                extra: None,
            })
        }).take(limit).collect())
    }
}
