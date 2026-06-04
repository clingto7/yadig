use std::time::{SystemTime, UNIX_EPOCH};

/// WBI signing for Bilibili API requests.
/// The img_key and sub_key are fetched from Bilibili's nav API.

const MIXIN_KEY_TABLE: [usize; 64] = [
    46, 47, 18, 2, 53, 8, 23, 32, 15, 50, 10, 31, 58, 3, 45, 35,
    27, 43, 5, 49, 33, 9, 42, 19, 29, 28, 14, 37, 12, 52, 56, 7,
    0, 16, 24, 40, 55, 38, 61, 6, 60, 39, 13, 47, 21, 44, 11, 22,
    25, 1, 48, 57, 20, 34, 41, 51, 54, 17, 4, 59, 36, 62, 49, 63,
];

/// WBI signing keys fetched from Bilibili's API.
#[derive(Debug, Clone)]
pub struct WbiKeys {
    pub img_key: String,
    pub sub_key: String,
}

/// Get the mixed key from img_key and sub_key (first 32 chars).
fn get_mixin_key(img_key: &str, sub_key: &str) -> String {
    let combined = format!("{}{}", img_key, sub_key);
    let bytes: Vec<u8> = combined.bytes().collect();
    MIXIN_KEY_TABLE
        .iter()
        .filter_map(|&i| bytes.get(i).map(|&b| b as char))
        .take(32)
        .collect()
}

/// Sign parameters with WBI algorithm.
/// Returns (w_rid_hex, wts).
pub fn sign_params(params: &[(&str, &str)], keys: &WbiKeys) -> (String, String) {
    let mixin_key = get_mixin_key(&keys.img_key, &keys.sub_key);
    let wts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
        .to_string();

    // Build sorted query with URL encoding
    let mut sorted: Vec<(&str, &str)> = params.to_vec();
    sorted.push(("wts", &wts));
    sorted.sort_by(|a, b| a.0.cmp(b.0));

    // URL-encode keys and values, filter !'()*
    let query: String = sorted
        .iter()
        .map(|(k, v)| format!("{}={}", urlencode(k), urlencode(v)))
        .collect::<Vec<_>>()
        .join("&");

    // w_rid = md5(query + mixin_key)
    let to_sign = format!("{}{}", query, mixin_key);
    let w_rid = format!("{:x}", md5::compute(to_sign.as_bytes()));

    (w_rid, wts)
}

/// Minimal URL encoding matching Bilibili's encodeURIComponent behavior.
fn urlencode(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => c.to_string(),
            '!' | '\'' | '(' | ')' | '*' => String::new(), // filtered out
            c => {
                let bytes = c.to_string().into_bytes();
                bytes.iter().map(|&b| format!("%{:02X}", b)).collect()
            }
        })
        .collect()
}

/// Fetch WBI keys from Bilibili's nav API.
pub async fn fetch_wbi_keys(client: &reqwest::Client) -> std::result::Result<WbiKeys, String> {
    let resp = client
        .get("https://api.bilibili.com/x/web-interface/nav")
        .header("User-Agent", "yadig/0.1.0")
        .header("Referer", "https://www.bilibili.com")
        .send()
        .await
        .map_err(|e| format!("Failed to fetch WBI keys: {}", e))?;

    let data: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("Failed to parse WBI keys response: {}", e))?;

    let img_key = data["data"]["wbi_img"]["img_url"]
        .as_str()
        .unwrap_or("")
        .split('/')
        .last()
        .unwrap_or("")
        .split('.')
        .next()
        .unwrap_or("")
        .to_string();

    let sub_key = data["data"]["wbi_img"]["sub_url"]
        .as_str()
        .unwrap_or("")
        .split('/')
        .last()
        .unwrap_or("")
        .split('.')
        .next()
        .unwrap_or("")
        .to_string();

    if img_key.is_empty() || sub_key.is_empty() {
        return Err("Failed to extract WBI keys from nav response".to_string());
    }

    Ok(WbiKeys { img_key, sub_key })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mixin_key_generation() {
        let mixin = get_mixin_key("testimgkey123456", "testsubkey123456");
        assert!(!mixin.is_empty());
        assert!(mixin.len() <= 32);
    }

    #[test]
    fn test_sign_params_differs_by_params() {
        let keys = WbiKeys {
            img_key: "aaa".to_string(),
            sub_key: "bbb".to_string(),
        };
        let (sig1, ts1) = sign_params(&[("bvid", "BV1xxx"), ("cid", "123")], &keys);
        let (sig2, _) = sign_params(&[("bvid", "BV1yyy"), ("cid", "456")], &keys);
        assert_ne!(sig1, sig2);
        assert!(!ts1.is_empty());
    }

    #[test]
    fn test_urlencode_spaces() {
        assert_eq!(urlencode("hello world"), "hello%20world");
    }

    #[test]
    fn test_urlencode_chinese() {
        assert!(urlencode("中文").contains('%'));
    }

    #[test]
    fn test_urlencode_filters_special() {
        assert_eq!(urlencode("a!b'c(d)e*f"), "abcdef");
    }
}
