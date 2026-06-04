use async_trait::async_trait;
use crate::error::Result;
use crate::http_client;
use crate::source::provider::SourceProvider;
use crate::source::types::*;

/// Simple regex capture helper — returns Vec of capture groups for each match.
fn regex_find_all<'a>(text: &'a str, pattern: &str) -> Vec<Vec<&'a str>> {
    let re = match regex::Regex::new(pattern) {
        Ok(r) => r,
        Err(_) => return Vec::new(),
    };
    re.captures_iter(text)
        .map(|cap| {
            (0..cap.len()).filter_map(|i| cap.get(i).map(|m| m.as_str())).collect()
        })
        .collect()
}

/// Pitchfork source — uses RSS feeds + HTML scraping for search
/// Feed URLs: https://pitchfork.com/feed/...
pub struct PitchforkSource {
    client: reqwest::Client,
    feed_urls: Vec<String>,
}

impl PitchforkSource {
    pub fn new() -> Self {
        let feed_urls = vec![
            "https://pitchfork.com/feed/feed-news/rss".to_string(),
            "https://pitchfork.com/feed/feed-album-reviews/rss".to_string(),
        ];
        let client = http_client::build_client("yadig/0.1.0 (music discovery)");
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
                    audio_url: None,
                    download_url: None,
                    duration: None,
                    license: None,
                    relevance_score: None,
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
                    audio_url: None,
                    download_url: None,
                    duration: None,
                    license: None,
                    relevance_score: None,
                    extra: None,
                })
            }).collect::<Vec<_>>()
        } else {
            Vec::new()
        };

        Ok(items)
    }

    /// Scrape Pitchfork search results page for album reviews and artist pages.
    /// Pitchfork is a React SPA but embeds search results in the HTML as anchor tags.
    async fn scrape_search(&self, query: &str, limit: usize) -> Result<Vec<ContentItem>> {
        let url = format!("https://pitchfork.com/search/?query={}", query);
        let resp = self.client.get(&url).send().await?;
        if !resp.status().is_success() {
            return Ok(Vec::new());
        }
        let body = resp.text().await?;

        let mut items = Vec::new();
        let mut seen = std::collections::HashSet::new();

        // Extract artist pages (high relevance — artist matches should come first)
        for cap in regex_find_all(&body, r#"href="(/artists/[^"]+)"[^>]*>([^<]+)"#) {
            let path = &cap[1];
            let name = cap[2].trim().to_string();
            if name.is_empty() || name.len() < 2 { continue; }
            let full_url = format!("https://pitchfork.com{}", path);
            if !seen.insert(full_url.clone()) { continue; }
            items.push(ContentItem {
                source_id: "pitchfork".to_string(),
                title: name,
                url: full_url,
                summary: Some("Artist".to_string()),
                author: None,
                published_at: None,
                image_url: None,
                audio_url: None,
                download_url: None,
                duration: None,
                license: None,
                relevance_score: Some(0.95), // Artist matches are highly relevant
                extra: None,
            });
        }

        // Extract album review links
        for cap in regex_find_all(&body, r#"href="(/reviews/albums/[^"]+/)"[^>]*>([^<]+)"#) {
            let path = &cap[1];
            let title = cap[2].trim().to_string();
            if title.is_empty() || title.len() < 2 { continue; }
            let full_url = format!("https://pitchfork.com{}", path);
            if !seen.insert(full_url.clone()) { continue; }
            items.push(ContentItem {
                source_id: "pitchfork".to_string(),
                title,
                url: full_url,
                summary: Some("Album Review".to_string()),
                author: None,
                published_at: None,
                image_url: None,
                audio_url: None,
                download_url: None,
                duration: None,
                license: None,
                relevance_score: Some(0.8),
                extra: None,
            });
        }

        // Extract news/article links
        for cap in regex_find_all(&body, r#"href="(/news/[^"]+/)"[^>]*>([^<]+)"#) {
            let path = &cap[1];
            let title = cap[2].trim().to_string();
            if title.is_empty() || title.len() < 3 { continue; }
            let full_url = format!("https://pitchfork.com{}", path);
            if !seen.insert(full_url.clone()) { continue; }
            items.push(ContentItem {
                source_id: "pitchfork".to_string(),
                title,
                url: full_url,
                summary: Some("News".to_string()),
                author: None,
                published_at: None,
                image_url: None,
                audio_url: None,
                download_url: None,
                duration: None,
                license: None,
                relevance_score: Some(0.6),
                extra: None,
            });
        }

        items.truncate(limit);
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
        // First: RSS keyword filter from recent articles
        let rss_items = self.fetch_latest(100).await?;
        let query_lower = query.to_lowercase();
        let mut results: Vec<ContentItem> = rss_items
            .into_iter()
            .filter(|item| {
                item.title.to_lowercase().contains(&query_lower)
                    || item.summary.as_ref().map_or(false, |s| s.to_lowercase().contains(&query_lower))
            })
            .collect();

        // Second: scrape Pitchfork search page for album reviews
        match self.scrape_search(query, limit).await {
            Ok(scraped) => results.extend(scraped),
            Err(e) => eprintln!("Pitchfork scrape search error: {}", e),
        }

        // Deduplicate by URL
        let mut seen = std::collections::HashSet::new();
        results.retain(|item| seen.insert(item.url.clone()));
        results.truncate(limit);
        Ok(results)
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
