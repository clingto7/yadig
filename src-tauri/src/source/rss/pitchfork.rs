use async_trait::async_trait;
use crate::error::Result;
use crate::source::provider::SourceProvider;
use crate::source::types::*;

/// Pitchfork source — uses RSS feeds
/// Feed URLs: https://pitchfork.com/rss/news/ /reviews/albums/ /best/
pub struct PitchforkSource {
    client: reqwest::Client,
    feed_urls: Vec<String>,
}

impl PitchforkSource {
    pub fn new() -> Self {
        let feed_urls = vec![
            "https://pitchfork.com/rss/news/".to_string(),
            "https://pitchfork.com/rss/reviews/albums/".to_string(),
            "https://pitchfork.com/rss/best/".to_string(),
        ];
        let client = reqwest::Client::builder()
            .user_agent("yadig/0.1.0 (music discovery)")
            .build()
            .expect("Failed to build HTTP client");
        Self { client, feed_urls }
    }

    async fn fetch_feed(&self, feed_url: &str) -> Result<Vec<ContentItem>> {
        let response = self.client.get(feed_url).send().await?;
        let body = response.text().await?;

        // Try parsing as RSS first, then Atom
        let items = if let Ok(channel) = rss::Channel::read_from(body.as_bytes()) {
            channel.items.into_iter().filter_map(|item| {
                let title = item.title?;
                let link = item.link?;
                Some(ContentItem {
                    source_id: self.id().to_string(),
                    title,
                    url: link,
                    summary: item.description,
                    author: item.author,
                    published_at: item.pub_date,
                    image_url: None,
                    extra: None,
                })
            }).collect::<Vec<_>>()
        } else if let Ok(feed) = atom_syndication::Feed::read_from(body.as_bytes()) {
            feed.entries.into_iter().filter_map(|entry| {
                let title = entry.title.to_string();
                let link = entry.links.first()?.href.clone();
                Some(ContentItem {
                    source_id: self.id().to_string(),
                    title,
                    url: link,
                    summary: entry.summary().map(|s| s.to_string()),
                    author: entry.authors.first().map(|a| a.name.clone()),
                    published_at: entry.published.map(|d| d.to_rfc3339()),
                    image_url: None,
                    extra: None,
                })
            }).collect::<Vec<_>>()
        } else {
            Vec::new()
        };

        Ok(items)
    }
}

#[async_trait]
impl SourceProvider for PitchforkSource {
    fn id(&self) -> &str { "pitchfork" }
    fn name(&self) -> &str { "Pitchfork" }
    fn kind(&self) -> SourceKind { SourceKind::Rss }
    fn base_url(&self) -> &str { "https://pitchfork.com" }

    async fn search(&self, query: &str, limit: usize, _page: usize) -> Result<Vec<ContentItem>> {
        // RSS doesn't support search natively — fetch latest and filter by keyword
        let items = self.fetch_latest(100).await?;
        let query_lower = query.to_lowercase();
        let filtered: Vec<ContentItem> = items
            .into_iter()
            .filter(|item| {
                item.title.to_lowercase().contains(&query_lower)
                    || item.summary.as_ref().map_or(false, |s| s.to_lowercase().contains(&query_lower))
            })
            .take(limit)
            .collect();
        Ok(filtered)
    }

    async fn fetch_latest(&self, limit: usize) -> Result<Vec<ContentItem>> {
        let mut all_items = Vec::new();

        for feed_url in &self.feed_urls {
            match self.fetch_feed(feed_url).await {
                Ok(items) => all_items.extend(items),
                Err(e) => eprintln!("Pitchfork RSS error for {}: {}", feed_url, e),
            }
        }

        all_items.sort_by(|a, b| {
            b.published_at.cmp(&a.published_at)
        });
        all_items.truncate(limit);

        Ok(all_items)
    }
}
