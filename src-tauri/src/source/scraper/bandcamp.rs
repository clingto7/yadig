use async_trait::async_trait;
use crate::error::{Result, YadigError};
use crate::source::provider::SourceProvider;
use crate::source::types::*;

/// Bandcamp source — scrapes HTML pages
/// No official API; uses web scraping for artist/label/tag discovery
pub struct BandcampSource {
    client: reqwest::Client,
}

impl BandcampSource {
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .user_agent("yadig/0.1.0 (music discovery)")
            .build()
            .expect("Failed to build HTTP client");
        Self { client }
    }
}

#[async_trait]
impl SourceProvider for BandcampSource {
    fn id(&self) -> &str { "bandcamp" }
    fn name(&self) -> &str { "Bandcamp" }
    fn kind(&self) -> SourceKind { SourceKind::Scraper }

    async fn search(&self, query: &str, limit: usize) -> Result<Vec<ContentItem>> {
        let url = format!("https://bandcamp.com/search?q={}", query);
        let response = self.client.get(&url).send().await?;

        if !response.status().is_success() {
            return Err(YadigError::Feed(format!(
                "Bandcamp search error: {}", response.status()
            )));
        }

        let body = response.text().await?;
        let document = scraper::Html::parse_document(&body);

        let selector = scraper::Selector::parse(".searchresultdata, .heading")
            .map_err(|_| YadigError::Feed("CSS selector parse error".into()))?;

        let items: Vec<ContentItem> = document.select(&selector)
            .filter_map(|el| {
                let title = el.text().collect::<Vec<_>>().join(" ").trim().to_string();
                if title.is_empty() {
                    return None;
                }
                let link = el.select(&scraper::Selector::parse("a").ok()?)
                    .next()
                    .and_then(|a| a.value().attr("href"))
                    .map(String::from)
                    .unwrap_or_default();

                Some(ContentItem {
                    source_id: "bandcamp".to_string(),
                    title,
                    url: link,
                    summary: None,
                    author: None,
                    published_at: None,
                    image_url: None,
                    extra: None,
                })
            })
            .take(limit)
            .collect();

        Ok(items)
    }

    async fn fetch_latest(&self, _limit: usize) -> Result<Vec<ContentItem>> {
        // Bandcamp doesn't have a unified "latest" page — would need specific tag/artist URLs
        Ok(Vec::new())
    }

    async fn get_item(&self, url: &str) -> Result<ContentItem> {
        let response = self.client.get(url).send().await?;
        let body = response.text().await?;
        let document = scraper::Html::parse_document(&body);

        let title_selector = scraper::Selector::parse("title")
            .map_err(|_| YadigError::Feed("CSS selector parse error".into()))?;

        let title = document.select(&title_selector)
            .next()
            .map(|el| el.text().collect::<Vec<_>>().join(" "))
            .unwrap_or_else(|| "Unknown".to_string());

        Ok(ContentItem {
            source_id: "bandcamp".to_string(),
            title,
            url: url.to_string(),
            summary: None,
            author: None,
            published_at: None,
            image_url: None,
            extra: None,
        })
    }
}
