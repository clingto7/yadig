use tauri::State;
use crate::bili::auth::{BiliAuth, BiliSession, QrLoginInfo, QrLoginStatus};
use crate::bili::client::BiliClient;
use crate::bili::extractor::ExtractionResult;
use crate::bili::types::DashAudio;
use crate::error::{Result, YadigError};

/// Response for QR login start
#[derive(serde::Serialize)]
pub struct QrLoginStartResponse {
    pub url: String,
    pub qrcode_key: String,
}

/// Response for QR login poll
#[derive(serde::Serialize)]
pub struct QrLoginPollResponse {
    pub code: i32,
    pub message: String,
    pub session: Option<BiliSession>,
}

/// Response for session status check
#[derive(serde::Serialize)]
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

    // Extract cookies from response headers before parsing JSON
    let cookies: Vec<String> = resp
        .headers()
        .get_all("set-cookie")
        .iter()
        .filter_map(|v| v.to_str().ok().map(|s| s.to_string()))
        .collect();

    let data: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| YadigError::Network(format!("QR poll parse error: {}", e)))?;

    let code = data["data"]["code"].as_i64().unwrap_or(-1) as i32;
    let message = data["data"]["message"]
        .as_str()
        .unwrap_or("")
        .to_string();

    // On success (code 0), extract session from cookies
    let session = if code == 0 {
        let sessdata = extract_cookie(&cookies, "SESSDATA");
        let bili_jct = extract_cookie(&cookies, "bili_jct");
        let dede_user_id = extract_cookie(&cookies, "DedeUserID");

        if let (Some(sessdata), Some(bili_jct), Some(dede_user_id)) = (sessdata, bili_jct, dede_user_id) {
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

/// Login with SESSDATA cookie string.
#[tauri::command]
pub async fn bili_cookie_login(auth: State<'_, BiliAuth>, sessdata: String) -> Result<()> {
    if sessdata.trim().is_empty() {
        return Err(YadigError::NotFound("SESSDATA cannot be empty".into()));
    }
    auth.set_cookie(&sessdata);
    Ok(())
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
pub struct PlayUrlInfo {
    pub audio_url: String,
    pub quality: i32,
    pub bandwidth: i64,
}

/// Extract audio from a Bilibili video URL and save to Downloads.
#[tauri::command]
pub async fn bili_extract_audio(auth: State<'_, BiliAuth>, url: String) -> Result<ExtractionResult> {
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
    client.extract_segment(&bvid, cid, &title, &download_dir).await
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
    client.extract_collection(mid, season_id, &download_dir).await
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
    let dash = play_resp.dash
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

fn extract_cookie(cookies: &[String], name: &str) -> Option<String> {
    for cookie in cookies {
        for part in cookie.split(';') {
            let part = part.trim();
            if let Some(value) = part.strip_prefix(&format!("{}=", name)) {
                // URL-decode the value, strip any attributes after semicolon
                let value = value.split(';').next().unwrap_or(value).trim();
                return Some(value.to_string());
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::extract_cookie;

    #[test]
    fn extract_cookie_from_set_cookie_header() {
        let cookies = vec![
            "SESSDATA=abc123; Domain=.bilibili.com; Path=/".to_string(),
            "bili_jct=def456; Domain=.bilibili.com".to_string(),
            "DedeUserID=42; Path=/".to_string(),
        ];
        assert_eq!(extract_cookie(&cookies, "SESSDATA"), Some("abc123".to_string()));
        assert_eq!(extract_cookie(&cookies, "bili_jct"), Some("def456".to_string()));
        assert_eq!(extract_cookie(&cookies, "DedeUserID"), Some("42".to_string()));
        assert_eq!(extract_cookie(&cookies, "missing"), None);
    }
}
