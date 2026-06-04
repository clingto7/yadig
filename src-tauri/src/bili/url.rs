use crate::error::{Result, YadigError};

/// Parsed Bilibili URL variants
#[derive(Debug, Clone, PartialEq)]
pub enum BiliUrl {
    /// Standard video URL: bilibili.com/video/BVxxx
    Video {
        bvid: String,
        page: Option<u32>,
    },
    /// Short link: b23.tv/xxx (needs HTTP redirect resolution)
    ShortLink {
        url: String,
    },
    /// Collection URL: space.bilibili.com/xxx/channel/collectiondetail?sid=123
    Collection {
        mid: i64,
        season_id: i64,
    },
}

/// Parse a Bilibili URL into its components.
/// Supports standard video URLs, short links, and collection URLs.
pub fn parse_bilibili_url(url: &str) -> Result<BiliUrl> {
    let url = url.trim();

    // Short link: b23.tv/xxx
    if url.contains("b23.tv") {
        return Ok(BiliUrl::ShortLink { url: url.to_string() });
    }

    // Collection URL: space.bilibili.com/{mid}/channel/collectiondetail?sid={season_id}
    if url.contains("space.bilibili.com") && url.contains("collectiondetail") {
        let mid = extract_path_segment(url, "space.bilibili.com")
            .and_then(|s| s.parse::<i64>().ok())
            .ok_or_else(|| YadigError::NotFound("Invalid collection URL: missing mid".into()))?;
        let season_id = extract_query_param(url, "sid")
            .and_then(|s| s.parse::<i64>().ok())
            .ok_or_else(|| YadigError::NotFound("Invalid collection URL: missing sid".into()))?;
        return Ok(BiliUrl::Collection { mid, season_id });
    }

    // Video URL: bilibili.com/video/BVxxx
    if url.contains("bilibili.com/video/") {
        let bvid = url
            .split("bilibili.com/video/")
            .nth(1)
            .map(|s| s.trim_end_matches('/').split('?').next().unwrap_or(""))
            .filter(|s| !s.is_empty())
            .ok_or_else(|| YadigError::NotFound("Invalid video URL: missing BV id".into()))?
            .to_string();
        let page = extract_query_param(url, "p").and_then(|s| s.parse::<u32>().ok());
        return Ok(BiliUrl::Video { bvid, page });
    }

    Err(YadigError::NotFound("Not a Bilibili URL".into()))
}

fn extract_path_segment(url: &str, after_host: &str) -> Option<String> {
    let idx = url.find(after_host)?;
    let rest = &url[idx + after_host.len()..];
    let segment = rest.trim_start_matches('/').split('/').next()?;
    if segment.is_empty() { None } else { Some(segment.to_string()) }
}

fn extract_query_param(url: &str, key: &str) -> Option<String> {
    let query_start = url.find('?')?;
    let query = &url[query_start + 1..];
    for pair in query.split('&') {
        let mut parts = pair.splitn(2, '=');
        if parts.next()? == key {
            return parts.next().map(|s| s.to_string());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_standard_video_url() {
        let url = "https://www.bilibili.com/video/BV1GJ411x7h7";
        let result = parse_bilibili_url(url).unwrap();
        assert_eq!(
            result,
            BiliUrl::Video {
                bvid: "BV1GJ411x7h7".to_string(),
                page: None,
            }
        );
    }

    #[test]
    fn parse_video_url_with_page() {
        let url = "https://www.bilibili.com/video/BV1GJ411x7h7?p=2";
        let result = parse_bilibili_url(url).unwrap();
        assert_eq!(
            result,
            BiliUrl::Video {
                bvid: "BV1GJ411x7h7".to_string(),
                page: Some(2),
            }
        );
    }

    #[test]
    fn parse_short_link() {
        let url = "https://b23.tv/abcdef";
        let result = parse_bilibili_url(url).unwrap();
        assert_eq!(
            result,
            BiliUrl::ShortLink {
                url: "https://b23.tv/abcdef".to_string(),
            }
        );
    }

    #[test]
    fn parse_collection_url() {
        let url = "https://space.bilibili.com/37737161/channel/collectiondetail?sid=1227671";
        let result = parse_bilibili_url(url).unwrap();
        assert_eq!(
            result,
            BiliUrl::Collection {
                mid: 37737161,
                season_id: 1227671,
            }
        );
    }

    #[test]
    fn reject_non_bilibili_url() {
        let url = "https://www.youtube.com/watch?v=dQw4w9WgXcQ";
        let result = parse_bilibili_url(url);
        assert!(result.is_err());
    }
}
