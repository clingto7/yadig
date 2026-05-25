use async_trait::async_trait;
use crate::error::{Result, YadigError};
use crate::source::provider::SourceProvider;
use crate::source::types::*;

/// Album of the Year source — scrapes HTML pages
/// No API, no RSS, protected by Cloudflare
/// Note: Cloudflare may block requests — users may need to configure a proxy or scraping API
pub struct AlbumOfTheYearSource {
    client: reqwest::Client,
}

impl AlbumOfTheYearSource {
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .user_agent("yadig/0.1.0 (music discovery)")
            .build()
            .expect("Failed to build HTTP client");
        Self { client }
    }
}

#[async_trait]
impl SourceProvider for AlbumOfTheYearSource {
    fn id(&self) -> &str { "albumoftheyear" }
    fn name(&self) -> &str { "Album of the Year" }
    fn kind(&self) -> SourceKind { SourceKind::Scraper }
    fn base_url(&self) -> &str { "https://www.albumoftheyear.org" }

    async fn search(&self, query: &str, limit: usize, _page: usize) -> Result<Vec<ContentItem>> {
        let url = format!("https://www.albumoftheyear.org/search?q={}", query);
        let response = self.client.get(&url).send().await?;

        if !response.status().is_success() {
            return Err(YadigError::Feed(format!(
                "AOTY search error: {}", response.status()
            )));
        }

        let body = response.text().await?;
        let document = scraper::Html::parse_document(&body);

        let selector = scraper::Selector::parse(".albumBlock, .album-block, .search-item")
            .map_err(|_| YadigError::Feed("CSS selector parse error".into()))?;

        let items: Vec<ContentItem> = document.select(&selector)
            .filter_map(|el| {
                let title_el = el.select(&scraper::Selector::parse("a, .albumTitle, .title").ok()?).next()?;
                let title = title_el.text().collect::<Vec<_>>().join(" ").trim().to_string();
                let link = title_el.value().attr("href")
                    .map(|h| {
                        if h.starts_with("http") {
                            h.to_string()
                        } else {
                            format!("https://www.albumoftheyear.org{}", h)
                        }
                    })
                    .unwrap_or_default();

                let image_url = el.select(&scraper::Selector::parse("img").ok()?)
                    .next()
                    .and_then(|img| img.value().attr("src").map(String::from));

                let rating = el.select(&scraper::Selector::parse(".rating, .score").ok()?)
                    .next()
                    .map(|r| r.text().collect::<Vec<_>>().join(" ").trim().to_string());

                let mut extra = serde_json::Map::new();
                if let Some(r) = rating {
                    extra.insert("rating".to_string(), serde_json::Value::String(r));
                }

                if title.is_empty() {
                    return None;
                }

                Some(ContentItem {
                    source_id: "albumoftheyear".to_string(),
                    title,
                    url: link,
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

    async fn fetch_latest(&self, limit: usize) -> Result<Vec<ContentItem>> {
        let url = "https://www.albumoftheyear.org/albums/new/";
        let response = self.client.get(url).send().await?;

        if !response.status().is_success() {
            return Err(YadigError::Feed(format!(
                "AOTY fetch error: {}", response.status()
            )));
        }

        let body = response.text().await?;
        let document = scraper::Html::parse_document(&body);

        let selector = scraper::Selector::parse(".albumBlock, .album-block")
            .map_err(|_| YadigError::Feed("CSS selector parse error".into()))?;

        let items: Vec<ContentItem> = document.select(&selector)
            .filter_map(|el| {
                let title_el = el.select(&scraper::Selector::parse("a").ok()?).next()?;
                let title = title_el.text().collect::<Vec<_>>().join(" ").trim().to_string();
                let link = title_el.value().attr("href")
                    .map(|h| {
                        if h.starts_with("http") { h.to_string() }
                        else { format!("https://www.albumoftheyear.org{}", h) }
                    })
                    .unwrap_or_default();

                let image_url = el.select(&scraper::Selector::parse("img").ok()?)
                    .next()
                    .and_then(|img| img.value().attr("src").map(String::from));

                Some(ContentItem {
                    source_id: "albumoftheyear".to_string(),
                    title,
                    url: link,
                    summary: None,
                    author: None,
                    published_at: None,
                    image_url,
                    extra: None,
                })
            })
            .take(limit)
            .collect();

        Ok(items)
    }
}
