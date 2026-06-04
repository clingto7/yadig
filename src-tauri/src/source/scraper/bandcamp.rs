use async_trait::async_trait;
use crate::error::{Result, YadigError};
use crate::http_client;
use crate::source::provider::SourceProvider;
use crate::source::types::*;

/// Bandcamp source — scrapes HTML pages and uses internal APIs
/// No official API; uses web scraping for artist/label/tag discovery
pub struct BandcampSource {
    client: reqwest::Client,
}

impl BandcampSource {
    pub fn new() -> Self {
        Self {
            client: http_client::build_client("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36"),
        }
    }
}

#[async_trait]
impl SourceProvider for BandcampSource {
    fn id(&self) -> &str { "bandcamp" }
    fn name(&self) -> &str { "Bandcamp" }
    fn kind(&self) -> SourceKind { SourceKind::Scraper }
    fn base_url(&self) -> &str { "https://bandcamp.com" }

    async fn search(&self, query: &str, limit: usize, _page: usize) -> Result<Vec<ContentItem>> {
        let url = format!("https://bandcamp.com/search?q={}", query);
        let response = self.client.get(&url).send().await?;

        if !response.status().is_success() {
            return Err(YadigError::Feed(format!(
                "Bandcamp search error: {}", response.status()
            )));
        }

        let body = response.text().await?;
        let document = scraper::Html::parse_document(&body);

        // Bandcamp search results use .searchresult class with .heading and .subhead
        let selector = scraper::Selector::parse(".searchresult")
            .map_err(|_| YadigError::Feed("CSS selector parse error".into()))?;

        let heading_sel = scraper::Selector::parse(".heading a")
            .map_err(|_| YadigError::Feed("CSS selector parse error".into()))?;
        let subhead_sel = scraper::Selector::parse(".subhead")
            .map_err(|_| YadigError::Feed("CSS selector parse error".into()))?;
        let item_type_sel = scraper::Selector::parse(".itemtype")
            .map_err(|_| YadigError::Feed("CSS selector parse error".into()))?;
        let art_sel = scraper::Selector::parse(".art img")
            .map_err(|_| YadigError::Feed("CSS selector parse error".into()))?;

        let items: Vec<ContentItem> = document.select(&selector)
            .filter_map(|el| {
                let heading = el.select(&heading_sel).next()?;
                let title = heading.text().collect::<Vec<_>>().join(" ").trim().to_string();
                if title.is_empty() {
                    return None;
                }
                let link = heading.value().attr("href")
                    .map(String::from)
                    .unwrap_or_default();

                let subhead = el.select(&subhead_sel).next()
                    .map(|s| s.text().collect::<Vec<_>>().join(" ").trim().to_string());

                let item_type = el.select(&item_type_sel).next()
                    .map(|t| t.text().collect::<Vec<_>>().join(" ").trim().to_string());

                let image_url = el.select(&art_sel).next()
                    .and_then(|img| img.value().attr("src").map(String::from))
                    .or_else(|| heading.value().attr("href").and_then(|h| {
                        // Construct art URL from the item URL pattern
                        let _ = h; // Bandcamp art URLs are in the HTML
                        Option::None
                    }));

                let mut extra = serde_json::Map::new();
                if let Some(t) = item_type {
                    extra.insert("type".to_string(), serde_json::Value::String(t));
                }

                Some(ContentItem {
                    source_id: "bandcamp".to_string(),
                    title,
                    url: link,
                    summary: subhead,
                    author: None,
                    published_at: None,
                    image_url,
                    audio_url: None,
                    download_url: None,
                    duration: None,
                    license: None,
                    relevance_score: None,
                    extra: if extra.is_empty() { None } else { Some(serde_json::Value::Object(extra)) },
                })
            })
            .take(limit)
            .collect();

        Ok(items)
    }

    async fn fetch_latest(&self, limit: usize) -> Result<Vec<ContentItem>> {
        // Use Bandcamp's internal discover API
        let url = format!(
            "https://bandcamp.com/api/discover/3/get_web?g_id=0&s_id=0&f_=genre&t_=r&p={}&c=5&gn=0",
            (limit / 5).max(1)
        );

        let response = self.client.get(&url).send().await?;

        if !response.status().is_success() {
            // Fallback: try scraping the discover page
            return self.fetch_latest_scrape(limit).await;
        }

        let data: serde_json::Value = response.json().await.map_err(|e| {
            YadigError::Feed(format!("Bandcamp discover API parse error: {}", e))
        })?;

        let empty = vec![];
        let results = data["items"].as_array().unwrap_or(&empty);

        let items: Vec<ContentItem> = results
            .iter()
            .filter_map(|r| {
                let title = r["title"].as_str()?.to_string();
                let artist = r["band_name"].as_str().map(String::from);
                let url = r["item_url"].as_str()
                    .or_else(|| r["url"].as_str())
                    .map(String::from)
                    .unwrap_or_default();

                let image_url = r["art_id"].as_i64()
                    .map(|id| format!("https://f4.bcbits.com/img/a{}.jpg", id));

                let display_title = match &artist {
                    Some(a) => format!("{} — {}", a, title),
                    None => title,
                };

                Some(ContentItem {
                    source_id: "bandcamp".to_string(),
                    title: display_title,
                    url,
                    summary: artist.clone(),
                    author: artist,
                    published_at: None,
                    image_url,
                    audio_url: None,
                    download_url: None,
                    duration: None,
                    license: None,
                    relevance_score: None,
                    extra: None,
                })
            })
            .take(limit)
            .collect();

        if items.is_empty() {
            // API might have changed, fall back to scraping
            return self.fetch_latest_scrape(limit).await;
        }

        Ok(items)
    }
}

impl BandcampSource {
    async fn fetch_latest_scrape(&self, limit: usize) -> Result<Vec<ContentItem>> {
        let url = "https://bandcamp.com/#discover";
        let response = self.client.get(url).send().await?;

        if !response.status().is_success() {
            return Ok(Vec::new());
        }

        let body = response.text().await?;
        let document = scraper::Html::parse_document(&body);

        // Try to find discover items in the HTML
        let selector = scraper::Selector::parse(".discover-item, .collection-item-container")
            .unwrap_or_else(|_| scraper::Selector::parse("div").unwrap());

        let title_sel = scraper::Selector::parse(".title, .item-title").ok();
        let artist_sel = scraper::Selector::parse(".artist, .item-artist").ok();
        let link_sel = scraper::Selector::parse("a").ok();
        let img_sel = scraper::Selector::parse("img").ok();

        let items: Vec<ContentItem> = document.select(&selector)
            .filter_map(|el| {
                let title = title_sel.as_ref()
                    .and_then(|s| el.select(s).next())
                    .map(|t| t.text().collect::<Vec<_>>().join(" ").trim().to_string())
                    .unwrap_or_default();

                if title.is_empty() {
                    return None;
                }

                let artist = artist_sel.as_ref()
                    .and_then(|s| el.select(s).next())
                    .map(|a| a.text().collect::<Vec<_>>().join(" ").trim().to_string());

                let link = link_sel.as_ref()
                    .and_then(|s| el.select(s).next())
                    .and_then(|a| a.value().attr("href").map(String::from))
                    .unwrap_or_default();

                let image_url = img_sel.as_ref()
                    .and_then(|s| el.select(s).next())
                    .and_then(|img| img.value().attr("src").map(String::from));

                let display_title = match &artist {
                    Some(a) => format!("{} — {}", a, title),
                    None => title,
                };

                Some(ContentItem {
                    source_id: "bandcamp".to_string(),
                    title: display_title,
                    url: link,
                    summary: artist.clone(),
                    author: artist,
                    published_at: None,
                    image_url,
                    audio_url: None,
                    download_url: None,
                    duration: None,
                    license: None,
                    relevance_score: None,
                    extra: None,
                })
            })
            .take(limit)
            .collect();

        Ok(items)
    }
}
