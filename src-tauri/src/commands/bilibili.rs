use crate::bili::auth::{BiliAuth, BiliSession};
use crate::bili::client::BiliClient;
use crate::bili::extractor::ExtractionResult;
use crate::bili::session::parse_cookie_session;
use crate::error::{Result, YadigError};
use tauri::State;

/// Response for QR login start
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QrLoginStartResponse {
    pub url: String,
    pub qrcode_key: String,
}

/// Response for QR login poll
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QrLoginPollResponse {
    pub code: i32,
    pub message: String,
    pub session: Option<BiliSession>,
}

/// Response for session status check
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionStatusResponse {
    pub logged_in: bool,
    pub username: Option<String>,
    pub is_premium: bool,
}

/// Start QR login flow by calling Bilibili's generate endpoint.
#[tauri::command]
pub async fn bili_qr_login_start(_auth: State<'_, BiliAuth>) -> Result<QrLoginStartResponse> {
    let client = crate::http_client::build_client("yadig/0.1.0");
    let resp = client
        .get("https://passport.bilibili.com/x/passport-login/web/qrcode/generate")
        .send()
        .await
        .map_err(|e| YadigError::Network(format!("QR generate failed: {}", e)))?;

    let data: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| YadigError::Network(format!("QR generate parse error: {}", e)))?;

    let url = data["data"]["url"]
        .as_str()
        .ok_or_else(|| YadigError::Network("Missing QR url in response".into()))?
        .to_string();
    let qrcode_key = data["data"]["qrcode_key"]
        .as_str()
        .ok_or_else(|| YadigError::Network("Missing qrcode_key in response".into()))?
        .to_string();

    Ok(QrLoginStartResponse { url, qrcode_key })
}

/// Poll QR login status. Returns the current status and session on success.
#[tauri::command]
pub async fn bili_qr_login_poll(
    auth: State<'_, BiliAuth>,
    qrcode_key: String,
) -> Result<QrLoginPollResponse> {
    let client = crate::http_client::build_client("yadig/0.1.0");
    let resp = client
        .get("https://passport.bilibili.com/x/passport-login/web/qrcode/poll")
        .query(&[("qrcode_key", &qrcode_key)])
        .send()
        .await
        .map_err(|e| YadigError::Network(format!("QR poll failed: {}", e)))?;

    let data: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| YadigError::Network(format!("QR poll parse error: {}", e)))?;

    // Inner code indicates QR status (86101=not scanned, 86090=scanned, 0=success, 86038=expired)
    let code = data["data"]["code"].as_i64().unwrap_or(-1) as i32;
    let message = data["data"]["message"].as_str().unwrap_or("").to_string();
    if !matches!(code, 86101 | 86090) {
        eprintln!("[bili_qr_login_poll] {}", qr_poll_status_summary(&data));
    }

    // On success (code 0), extract session from data.url query parameters
    let session = if code == 0 {
        let callback_url = data["data"]["url"].as_str().unwrap_or("");
        let sessdata = extract_url_param(callback_url, "SESSDATA");
        let bili_jct = extract_url_param(callback_url, "bili_jct");
        let dede_user_id = extract_url_param(callback_url, "DedeUserID");

        if let (Some(sessdata), Some(bili_jct), Some(dede_user_id)) =
            (sessdata, bili_jct, dede_user_id)
        {
            let s = BiliSession {
                sessdata,
                bili_jct,
                dede_user_id,
                vip_status: 0, // will be verified later via nav API
            };
            auth.set_session(s.clone());
            Some(s)
        } else {
            None
        }
    } else {
        None
    };

    Ok(QrLoginPollResponse {
        code,
        message,
        session,
    })
}

/// Login with Bilibili cookie (SESSDATA).
#[tauri::command]
pub async fn bili_cookie_login(auth: State<'_, BiliAuth>, sessdata: String) -> Result<()> {
    let cookie = sessdata.trim();
    if cookie.is_empty() {
        return Err(YadigError::NotFound("SESSDATA cannot be empty".into()));
    }
    if let Some(session) = parse_cookie_session(cookie) {
        auth.set_session(session);
    } else {
        auth.set_cookie(cookie);
    }
    Ok(())
}

/// Login with Bilibili username and password.
/// Note: This may trigger CAPTCHA verification. If login fails with captcha error,
/// use bili_cookie_login with SESSDATA instead.
#[tauri::command]
pub async fn bili_password_login(
    auth: State<'_, BiliAuth>,
    username: String,
    password: String,
) -> Result<String> {
    if username.trim().is_empty() || password.trim().is_empty() {
        return Err(YadigError::NotFound(
            "Username and password cannot be empty".into(),
        ));
    }

    let client = crate::http_client::build_client("yadig/0.1.0");
    let params = [
        ("username", username.as_str()),
        ("password", password.as_str()),
        ("keep_login", "true"),
    ];

    let resp = client
        .post("https://passport.bilibili.com/x/passport-login/oauth2/login")
        .header("Referer", "https://www.bilibili.com")
        .form(&params)
        .send()
        .await
        .map_err(|e| YadigError::Network(format!("Password login failed: {}", e)))?;

    let data: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| YadigError::Network(format!("Password login parse error: {}", e)))?;

    let code = data["code"].as_i64().unwrap_or(-1);
    let message = data["message"]
        .as_str()
        .unwrap_or("unknown error")
        .to_string();

    if code != 0 {
        return Err(YadigError::Network(format!(
            "Login failed ({}): {}. If CAPTCHA is required, use Cookie Login instead (copy SESSDATA from browser).",
            code, message
        )));
    }

    // Extract session from cookie_info
    let cookies = &data["data"]["cookie_info"]["cookies"];
    let sessdata = cookies
        .as_array()
        .and_then(|arr| arr.iter().find(|c| c["name"] == "SESSDATA"))
        .and_then(|c| c["value"].as_str().map(String::from));
    let bili_jct = cookies
        .as_array()
        .and_then(|arr| arr.iter().find(|c| c["name"] == "bili_jct"))
        .and_then(|c| c["value"].as_str().map(String::from));
    let dede_user_id = cookies
        .as_array()
        .and_then(|arr| arr.iter().find(|c| c["name"] == "DedeUserID"))
        .and_then(|c| c["value"].as_str().map(String::from));

    if let (Some(sessdata), Some(bili_jct), Some(dede_user_id)) = (sessdata, bili_jct, dede_user_id)
    {
        let session = BiliSession {
            sessdata,
            bili_jct,
            dede_user_id,
            vip_status: 0,
        };
        auth.set_session(session);
        Ok("Login successful".to_string())
    } else {
        Err(YadigError::NotFound(
            "Login succeeded but couldn't extract session cookies".into(),
        ))
    }
}

/// Logout — clear session.
#[tauri::command]
pub async fn bili_logout(auth: State<'_, BiliAuth>) -> Result<()> {
    auth.logout();
    Ok(())
}

/// Get current session status by verifying with Bilibili's nav API.
#[tauri::command]
pub async fn bili_session_status(auth: State<'_, BiliAuth>) -> Result<SessionStatusResponse> {
    let session = auth.session();
    if let Some(session) = session {
        // Verify session by calling nav API
        let client = crate::http_client::build_client("yadig/0.1.0");
        let resp = client
            .get("https://api.bilibili.com/x/web-interface/nav")
            .header("Cookie", format!("SESSDATA={}", session.sessdata))
            .send()
            .await;

        match resp {
            Ok(r) => {
                if let Ok(data) = r.json::<serde_json::Value>().await {
                    let is_login = data["data"]["isLogin"].as_bool().unwrap_or(false);
                    if is_login {
                        let username = data["data"]["uname"].as_str().map(|s| s.to_string());
                        let vip_status = data["data"]["vipStatus"].as_i64().unwrap_or(0) as i32;
                        // Update session with vip_status
                        let mut updated = session.clone();
                        updated.vip_status = vip_status;
                        auth.set_session(updated);
                        Ok(SessionStatusResponse {
                            logged_in: true,
                            username,
                            is_premium: vip_status == 1,
                        })
                    } else {
                        auth.logout();
                        Ok(SessionStatusResponse {
                            logged_in: false,
                            username: None,
                            is_premium: false,
                        })
                    }
                } else {
                    Ok(SessionStatusResponse {
                        logged_in: true,
                        username: None,
                        is_premium: false,
                    })
                }
            }
            Err(_) => Ok(SessionStatusResponse {
                logged_in: true, // assume logged in if we can't verify
                username: None,
                is_premium: false,
            }),
        }
    } else {
        Ok(SessionStatusResponse {
            logged_in: false,
            username: None,
            is_premium: false,
        })
    }
}

/// Response for playurl query
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayUrlInfo {
    pub audio_url: String,
    pub quality: i32,
    pub bandwidth: i64,
}

/// Extract audio from a Bilibili video URL and save to Downloads.
#[tauri::command]
pub async fn bili_extract_audio(
    auth: State<'_, BiliAuth>,
    url: String,
) -> Result<ExtractionResult> {
    let client = BiliClient::new((*auth).clone());
    let downloads = dirs_next::download_dir()
        .ok_or_else(|| YadigError::Network("Could not find Downloads folder".into()))?;
    let download_dir = downloads.join("yadig");
    client.extract_audio(&url, &download_dir).await
}

/// Extract audio for a specific 分P by cid.
#[tauri::command]
pub async fn bili_extract_segment(
    auth: State<'_, BiliAuth>,
    bvid: String,
    cid: i64,
    title: String,
) -> Result<ExtractionResult> {
    let client = BiliClient::new((*auth).clone());
    let downloads = dirs_next::download_dir()
        .ok_or_else(|| YadigError::Network("Could not find Downloads folder".into()))?;
    let download_dir = downloads.join("yadig");
    client
        .extract_segment(&bvid, cid, &title, &download_dir)
        .await
}

/// Extract audio from a collection (合集) by mid and season_id.
#[tauri::command]
pub async fn bili_extract_collection(
    auth: State<'_, BiliAuth>,
    mid: i64,
    season_id: i64,
) -> Result<ExtractionResult> {
    let client = BiliClient::new((*auth).clone());
    let downloads = dirs_next::download_dir()
        .ok_or_else(|| YadigError::Network("Could not find Downloads folder".into()))?;
    let download_dir = downloads.join("yadig");
    client
        .extract_collection(mid, season_id, &download_dir)
        .await
}

/// Check if FFmpeg is available for chapter splitting.
#[tauri::command]
pub async fn bili_check_ffmpeg() -> Result<bool> {
    Ok(crate::bili::ffmpeg::is_available())
}

/// Get the best audio stream URL for a specific video (without downloading).
#[tauri::command]
pub async fn bili_get_playurl(
    auth: State<'_, BiliAuth>,
    bvid: String,
    cid: i64,
) -> Result<PlayUrlInfo> {
    let client = BiliClient::new((*auth).clone());
    let info = client.video_info(&bvid).await?;
    let play_resp = client.playurl(info.aid, cid).await?;
    let dash = play_resp
        .dash
        .ok_or_else(|| YadigError::NotFound("No DASH streams available".into()))?;

    let has_session = auth.session().is_some();
    let is_premium = auth.is_premium();
    let best = crate::bili::extractor::select_best_audio(&dash.audio, has_session, is_premium)
        .ok_or_else(|| YadigError::NotFound("No audio streams available".into()))?;

    Ok(PlayUrlInfo {
        audio_url: best.base_url.clone(),
        quality: best.id,
        bandwidth: best.bandwidth,
    })
}

/// Extract a query parameter from a URL string.
fn extract_url_param(url: &str, key: &str) -> Option<String> {
    let parsed = url::Url::parse(url).ok()?;
    parsed
        .query_pairs()
        .find_map(|(name, value)| (name == key).then(|| value.into_owned()))
}

fn qr_poll_status_summary(data: &serde_json::Value) -> String {
    let code = data["data"]["code"].as_i64().unwrap_or(-1);
    let message = data["data"]["message"]
        .as_str()
        .unwrap_or("")
        .replace(['\n', '\r'], " ");
    format!("code={} message='{}'", code, message)
}

#[cfg(test)]
mod tests {
    use super::{extract_url_param, qr_poll_status_summary, QrLoginStartResponse, SessionStatusResponse};
    use crate::bili::auth::BiliSession;
    use serde_json::json;

    #[test]
    fn extract_params_from_callback_url() {
        let url = "https://passport.bilibili.com/crossDomain?DedeUserID=12345&DedeUserID__ckMd5=abc&SESSDATA=test%2Cdata&bili_jct=ctoken&gourl=https%3A%2F%2Fwww.bilibili.com";
        assert_eq!(
            extract_url_param(url, "SESSDATA"),
            Some("test,data".to_string())
        );
        assert_eq!(
            extract_url_param(url, "bili_jct"),
            Some("ctoken".to_string())
        );
        assert_eq!(
            extract_url_param(url, "DedeUserID"),
            Some("12345".to_string())
        );
        assert_eq!(extract_url_param(url, "missing"), None);
    }

    #[test]
    fn qr_login_responses_match_frontend_camel_case_contract() {
        let start = serde_json::to_value(QrLoginStartResponse {
            url: "https://passport.bilibili.com/qrcode".to_string(),
            qrcode_key: "qr-key".to_string(),
        })
        .unwrap();
        assert_eq!(start["qrcodeKey"], json!("qr-key"));
        assert!(start.get("qrcode_key").is_none());

        let status = serde_json::to_value(SessionStatusResponse {
            logged_in: true,
            username: Some("name".to_string()),
            is_premium: false,
        })
        .unwrap();
        assert_eq!(status["loggedIn"], json!(true));
        assert_eq!(status["isPremium"], json!(false));
        assert!(status.get("logged_in").is_none());
        assert!(status.get("is_premium").is_none());
    }

    #[test]
    fn qr_login_session_matches_frontend_camel_case_contract() {
        let session = serde_json::to_value(BiliSession {
            sessdata: "sess".to_string(),
            bili_jct: "csrf".to_string(),
            dede_user_id: "42".to_string(),
            vip_status: 1,
        })
        .unwrap();

        assert_eq!(session["biliJct"], json!("csrf"));
        assert_eq!(session["dedeUserId"], json!("42"));
        assert_eq!(session["vipStatus"], json!(1));
        assert!(session.get("bili_jct").is_none());
        assert!(session.get("dede_user_id").is_none());
        assert!(session.get("vip_status").is_none());
    }

    #[test]
    fn qr_poll_log_summary_does_not_include_callback_url_or_cookies() {
        let data = json!({
            "data": {
                "code": 0,
                "message": "0",
                "url": "https://passport.bilibili.com/crossDomain?SESSDATA=secret&bili_jct=csrf&DedeUserID=42"
            }
        });

        let summary = qr_poll_status_summary(&data);

        assert!(summary.contains("code=0"));
        assert!(summary.contains("message='0'"));
        assert!(!summary.contains("url"));
        assert!(!summary.contains("SESSDATA"));
        assert!(!summary.contains("secret"));
        assert!(!summary.contains("bili_jct"));
        assert!(!summary.contains("DedeUserID"));
    }
}
