use crate::bili::auth::BiliSession;

/// Parse a browser Cookie header/string into the full session needed by write APIs.
pub fn parse_cookie_session(cookie: &str) -> Option<BiliSession> {
    let sessdata = cookie_value(cookie, "SESSDATA")?;
    let bili_jct = cookie_value(cookie, "bili_jct")?;
    let dede_user_id = cookie_value(cookie, "DedeUserID")?;

    Some(BiliSession {
        sessdata,
        bili_jct,
        dede_user_id,
        vip_status: 0,
    })
}

pub fn cookie_value(cookie: &str, key: &str) -> Option<String> {
    cookie
        .split(';')
        .filter_map(|part| {
            let trimmed = part.trim();
            let (name, value) = trimmed.split_once('=')?;
            Some((name.trim(), value.trim()))
        })
        .find_map(|(name, value)| {
            if name == key && !value.is_empty() {
                Some(value.to_string())
            } else {
                None
            }
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn returns_none_for_sessdata_only_cookie() {
        assert!(parse_cookie_session("SESSDATA=sess").is_none());
    }
}
