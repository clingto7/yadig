use crate::bili::auth::BiliAuth;
use crate::bili::types::SearchResponse;
use crate::error::{Result, YadigError};
use crate::http_client;
use crate::source::provider::SourceProvider;
use crate::source::types::*;
use async_trait::async_trait;

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

        let resp = req
            .send()
            .await
            .map_err(|e| YadigError::Network(format!("Bilibili search failed: {}", e)))?;

        let data: SearchResponse = resp
            .json()
            .await
            .map_err(|e| YadigError::Network(format!("Bilibili search parse error: {}", e)))?;

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
            .filter_map(|(i, r)| {
                let bvid = r.bvid?;
                let title_html = r.title.unwrap_or_default();
                // Strip HTML tags from title
                let title = strip_html_tags(&title_html);
                let author = r.author;
                let duration_str = r.duration.as_deref().unwrap_or("0:00");
                let duration = parse_duration(duration_str);
                let pic = r.pic.map(|p| {
                    if p.starts_with("//") {
                        format!("https:{}", p)
                    } else {
                        p.to_string()
                    }
                });
                let url = format!("https://www.bilibili.com/video/{}", bvid);
                let cid = r.id.unwrap_or(0);

                Some(ContentItem {
                    source_id: "bilibili".to_string(),
                    title,
                    url,
                    summary: r.description,
                    author,
                    published_at: None,
                    image_url: pic,
                    audio_url: None, // lazy — fetched on demand via bili_get_playurl
                    download_url: None,
                    duration: Some(duration),
                    license: None,
                    extra: Some(serde_json::json!({
                        "bvid": bvid,
                        "cid": cid,
                    })),
                    relevance_score: Some(1.0 - (i as f32 * 0.02)),
                })
            })
            .collect();

        Ok(items)
    }

    async fn fetch_latest(&self, _limit: usize) -> Result<Vec<ContentItem>> {
        // Bilibili ranking API for music zone — return empty for now
        Ok(Vec::new())
    }
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
}
