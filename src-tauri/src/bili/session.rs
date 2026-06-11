use crate::bili::auth::BiliSession;

/// Build a login session from either a raw SESSDATA value or a browser Cookie header.
pub fn login_session_from_cookie_input(input: &str) -> Result<BiliSession, String> {
    let cookie = input.trim();
    if cookie.is_empty() {
        return Err("Enter a Bilibili Cookie or SESSDATA value.".to_string());
    }

    if let Some(session) = parse_cookie_session(cookie) {
        return Ok(session);
    }

    let sessdata = if cookie.contains('=') {
        cookie_value(cookie, "SESSDATA")
            .ok_or_else(|| "Cookie login requires SESSDATA.".to_string())?
    } else {
        cookie.to_string()
    };

    Ok(BiliSession {
        sessdata,
        bili_jct: String::new(),
        dede_user_id: String::new(),
        vip_status: 0,
    })
}

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

    #[test]
    fn builds_read_only_session_from_sessdata_value() {
        let session = login_session_from_cookie_input(" sess-only ").unwrap();

        assert_eq!(session.sessdata, "sess-only");
        assert_eq!(session.bili_jct, "");
        assert_eq!(session.dede_user_id, "");
        assert_eq!(session.vip_status, 0);
    }

    #[test]
    fn builds_write_capable_session_from_full_cookie_header() {
        let session = login_session_from_cookie_input(
            "SESSDATA=sess; bili_jct=csrf; DedeUserID=42; other=value",
        )
        .unwrap();

        assert_eq!(session.sessdata, "sess");
        assert_eq!(session.bili_jct, "csrf");
        assert_eq!(session.dede_user_id, "42");
        assert_eq!(session.vip_status, 0);
    }

    #[test]
    fn rejects_blank_cookie_login_input() {
        let error = login_session_from_cookie_input("   ").unwrap_err();

        assert_eq!(error, "Enter a Bilibili Cookie or SESSDATA value.");
    }
}
