use crate::bili::auth::BiliAuth;
use crate::bili::types::{SearchResponse, SearchResultItem};
use crate::error::{Result, YadigError};
use crate::http_client;
use crate::source::provider::SourceProvider;
use crate::source::types::*;
use async_trait::async_trait;
use std::time::Duration;
use tokio::time::timeout;

const BILI_SEARCH_TIMEOUT: Duration = Duration::from_millis(2_500);

/// Bilibili search source — searches Bilibili for music videos.
/// Audio streams are not pre-fetched (too slow); use bili_get_playurl on demand.
pub struct BiliSource {
    client: reqwest::Client,
    auth: BiliAuth,
}

impl BiliSource {
    pub fn new(auth: BiliAuth) -> Self {
        Self {
            client: http_client::build_client("yadig/0.1.0 (music discovery)"),
            auth,
        }
    }
}

#[async_trait]
impl SourceProvider for BiliSource {
    fn id(&self) -> &str {
        "bilibili"
    }
    fn name(&self) -> &str {
        "Bilibili"
    }
    fn kind(&self) -> SourceKind {
        SourceKind::Api
    }
    fn base_url(&self) -> &str {
        "https://www.bilibili.com"
    }

    async fn search(&self, query: &str, limit: usize, _page: usize) -> Result<Vec<ContentItem>> {
        let encoded = urlencoding::encode(query);
        let api_url = format!(
            "https://api.bilibili.com/x/web-interface/search/type?search_type=video&keyword={}&page=1&page_size={}",
            encoded, limit.min(50)
        );

        let mut req = self
            .client
            .get(&api_url)
            .header("Referer", "https://www.bilibili.com");
        if let Some(session) = self.auth.session() {
            req = req.header("Cookie", format!("SESSDATA={}", session.sessdata));
        }

        let data: SearchResponse = timeout(BILI_SEARCH_TIMEOUT, async {
            let resp = req
                .send()
                .await
                .map_err(|e| YadigError::Network(format!("Bilibili search failed: {}", e)))?;

            resp.json()
                .await
                .map_err(|e| YadigError::Network(format!("Bilibili search parse error: {}", e)))
        })
        .await
        .map_err(|_| {
            YadigError::Network(format!(
                "Bilibili search timed out after {}ms",
                BILI_SEARCH_TIMEOUT.as_millis()
            ))
        })??;

        if data.code != 0 {
            return Err(YadigError::Network(format!(
                "Bilibili search API error: {}",
                data.message
            )));
        }

        let results = data.data.map(|data| data.result).unwrap_or_default();

        let items: Vec<ContentItem> = results
            .into_iter()
            .enumerate()
            .filter_map(|(i, result)| map_search_result_item(result, i))
            .collect();

        Ok(items)
    }

    async fn fetch_latest(&self, _limit: usize) -> Result<Vec<ContentItem>> {
        // Bilibili ranking API for music zone — return empty for now
        Ok(Vec::new())
    }
}

fn map_search_result_item(result: SearchResultItem, index: usize) -> Option<ContentItem> {
    let bvid = result.bvid?;
    let title_html = result.title.unwrap_or_default();
    let title = strip_html_tags(&title_html);
    let duration_str = result.duration.as_deref().unwrap_or("0:00");
    let duration = parse_duration(duration_str);
    let image_url = result.pic.map(|pic| {
        if pic.starts_with("//") {
            format!("https:{}", pic)
        } else {
            pic
        }
    });
    let cid = result.id.unwrap_or(0);
    let url = format!("https://www.bilibili.com/video/{}", bvid);

    Some(ContentItem {
        source_id: "bilibili".to_string(),
        title,
        url,
        summary: result.description,
        author: result.author,
        published_at: None,
        image_url,
        audio_url: None,
        download_url: None,
        duration: Some(duration),
        license: None,
        extra: Some(serde_json::json!({
            "bvid": bvid,
            "cid": cid,
        })),
        relevance_score: Some(1.0 - (index as f32 * 0.02)),
    })
}

/// Strip HTML tags from a string.
fn strip_html_tags(s: &str) -> String {
    let mut result = String::new();
    let mut in_tag = false;
    for c in s.chars() {
        match c {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => result.push(c),
            _ => {}
        }
    }
    result
}

/// Parse duration string like "3:45" into seconds.
fn parse_duration(s: &str) -> u32 {
    let parts: Vec<&str> = s.split(':').collect();
    match parts.len() {
        2 => {
            let mins = parts[0].parse::<u32>().unwrap_or(0);
            let secs = parts[1].parse::<u32>().unwrap_or(0);
            mins * 60 + secs
        }
        3 => {
            let hours = parts[0].parse::<u32>().unwrap_or(0);
            let mins = parts[1].parse::<u32>().unwrap_or(0);
            let secs = parts[2].parse::<u32>().unwrap_or(0);
            hours * 3600 + mins * 60 + secs
        }
        _ => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strip_html_basic() {
        assert_eq!(strip_html_tags("Hello <em>World</em>"), "Hello World");
        assert_eq!(strip_html_tags("No tags"), "No tags");
        assert_eq!(strip_html_tags("<b>Bold</b> &amp;"), "Bold &amp;");
    }

    #[test]
    fn parse_duration_mm_ss() {
        assert_eq!(parse_duration("3:45"), 225);
        assert_eq!(parse_duration("0:30"), 30);
        assert_eq!(parse_duration("10:00"), 600);
    }

    #[test]
    fn parse_duration_hh_mm_ss() {
        assert_eq!(parse_duration("1:30:00"), 5400);
    }

    #[test]
    fn parse_duration_invalid() {
        assert_eq!(parse_duration("abc"), 0);
        assert_eq!(parse_duration(""), 0);
    }

    #[test]
    fn maps_search_result_item_into_content_item_contract() {
        let item = crate::bili::types::SearchResultItem {
            bvid: Some("BV1contract".to_string()),
            title: Some("Live <em>cover</em>".to_string()),
            author: Some("Music UP".to_string()),
            duration: Some("1:02:03".to_string()),
            pic: Some("//i0.hdslb.com/bfs/archive/test.jpg".to_string()),
            description: Some("Session description".to_string()),
            id: Some(998877),
        };

        let mapped = map_search_result_item(item, 0).expect("valid Bilibili result should map");

        assert_eq!(mapped.source_id, "bilibili");
        assert_eq!(mapped.title, "Live cover");
        assert_eq!(mapped.url, "https://www.bilibili.com/video/BV1contract");
        assert_eq!(
            mapped.image_url.as_deref(),
            Some("https://i0.hdslb.com/bfs/archive/test.jpg")
        );
        assert_eq!(mapped.author.as_deref(), Some("Music UP"));
        assert_eq!(mapped.duration, Some(3723));
        assert_eq!(mapped.audio_url, None);
        assert_eq!(mapped.download_url, None);
        assert_eq!(
            mapped
                .extra
                .as_ref()
                .and_then(|extra| extra["bvid"].as_str()),
            Some("BV1contract")
        );
        assert_eq!(
            mapped
                .extra
                .as_ref()
                .and_then(|extra| extra["cid"].as_i64()),
            Some(998877)
        );
    }

    #[test]
    fn bili_search_timeout_budget_stays_under_three_seconds() {
        assert!(BILI_SEARCH_TIMEOUT < std::time::Duration::from_secs(3));
        assert!(BILI_SEARCH_TIMEOUT >= std::time::Duration::from_secs(1));
    }
}
