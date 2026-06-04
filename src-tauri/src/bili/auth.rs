use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};

/// Bilibili session data obtained from login.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BiliSession {
    pub sessdata: String,
    pub bili_jct: String,
    pub dede_user_id: String,
    /// 1 = VIP (大会员), 0 = standard
    #[serde(default)]
    pub vip_status: i32,
}

/// QR login status codes from Bilibili's poll API.
#[derive(Debug, Clone, PartialEq)]
pub enum QrLoginStatus {
    /// Code 86101 — not yet scanned
    NotScanned,
    /// Code 86090 — scanned but not confirmed
    Scanned,
    /// Code 0 — login successful
    Confirmed(BiliSession),
    /// Code 86038 — QR code expired
    Expired,
}

/// Result of QR code generation.
#[derive(Debug, Clone)]
pub struct QrLoginInfo {
    /// The URL to encode in the QR code
    pub url: String,
    /// Key for polling login status
    pub qrcode_key: String,
}

/// Manages Bilibili authentication state.
/// Thread-safe via Arc<RwLock<Option<BiliSession>>> (same pattern as DiscogsKeys).
#[derive(Clone)]
pub struct BiliAuth {
    session: Arc<RwLock<Option<BiliSession>>>,
}

impl BiliAuth {
    pub fn new() -> Self {
        Self {
            session: Arc::new(RwLock::new(None)),
        }
    }

    /// Set session from cookie string (SESSDATA only, minimal session).
    pub fn set_cookie(&self, sessdata: &str) {
        let session = BiliSession {
            sessdata: sessdata.to_string(),
            bili_jct: String::new(),
            dede_user_id: String::new(),
            vip_status: 0,
        };
        *self.session.write().unwrap() = Some(session);
    }

    /// Set a full session (e.g., after QR login or from persisted store).
    pub fn set_session(&self, session: BiliSession) {
        *self.session.write().unwrap() = Some(session);
    }

    /// Get current session (cloned).
    pub fn session(&self) -> Option<BiliSession> {
        self.session.read().unwrap().clone()
    }

    /// Clear session.
    pub fn logout(&self) {
        *self.session.write().unwrap() = None;
    }

    /// Check if current session has premium (大会员) status.
    pub fn is_premium(&self) -> bool {
        self.session
            .read()
            .unwrap()
            .as_ref()
            .map(|s| s.vip_status == 1)
            .unwrap_or(false)
    }

    /// Parse QR login poll response code into status.
    pub fn parse_qr_status(code: i32, session: Option<BiliSession>) -> QrLoginStatus {
        match code {
            0 => QrLoginStatus::Confirmed(session.unwrap()),
            86101 => QrLoginStatus::NotScanned,
            86090 => QrLoginStatus::Scanned,
            86038 => QrLoginStatus::Expired,
            _ => QrLoginStatus::Expired,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_auth_has_no_session() {
        let auth = BiliAuth::new();
        assert!(auth.session().is_none());
    }

    #[test]
    fn set_cookie_creates_session() {
        let auth = BiliAuth::new();
        auth.set_cookie("test_sessdata");
        let session = auth.session().unwrap();
        assert_eq!(session.sessdata, "test_sessdata");
        assert_eq!(session.vip_status, 0);
    }

    #[test]
    fn set_session_overwrites() {
        let auth = BiliAuth::new();
        auth.set_cookie("old");
        auth.set_session(BiliSession {
            sessdata: "new".to_string(),
            bili_jct: "ct".to_string(),
            dede_user_id: "123".to_string(),
            vip_status: 1,
        });
        assert_eq!(auth.session().unwrap().sessdata, "new");
        assert_eq!(auth.session().unwrap().bili_jct, "ct");
    }

    #[test]
    fn logout_clears_session() {
        let auth = BiliAuth::new();
        auth.set_cookie("test");
        assert!(auth.session().is_some());
        auth.logout();
        assert!(auth.session().is_none());
    }

    #[test]
    fn is_premium_returns_false_for_standard() {
        let auth = BiliAuth::new();
        auth.set_cookie("test");
        assert!(!auth.is_premium());
    }

    #[test]
    fn is_premium_returns_true_for_vip() {
        let auth = BiliAuth::new();
        auth.set_session(BiliSession {
            sessdata: "test".to_string(),
            bili_jct: String::new(),
            dede_user_id: String::new(),
            vip_status: 1,
        });
        assert!(auth.is_premium());
    }

    #[test]
    fn is_premium_returns_false_when_no_session() {
        let auth = BiliAuth::new();
        assert!(!auth.is_premium());
    }

    #[test]
    fn parse_qr_status_not_scanned() {
        assert_eq!(
            BiliAuth::parse_qr_status(86101, None),
            QrLoginStatus::NotScanned
        );
    }

    #[test]
    fn parse_qr_status_scanned() {
        assert_eq!(
            BiliAuth::parse_qr_status(86090, None),
            QrLoginStatus::Scanned
        );
    }

    #[test]
    fn parse_qr_status_confirmed() {
        let session = BiliSession {
            sessdata: "sd".to_string(),
            bili_jct: "ct".to_string(),
            dede_user_id: "42".to_string(),
            vip_status: 0,
        };
        let status = BiliAuth::parse_qr_status(0, Some(session));
        match status {
            QrLoginStatus::Confirmed(s) => assert_eq!(s.dede_user_id, "42"),
            _ => panic!("Expected Confirmed"),
        }
    }

    #[test]
    fn parse_qr_status_expired() {
        assert_eq!(
            BiliAuth::parse_qr_status(86038, None),
            QrLoginStatus::Expired
        );
    }

    #[test]
    fn session_serialization_roundtrip() {
        let session = BiliSession {
            sessdata: "test_sd".to_string(),
            bili_jct: "test_ct".to_string(),
            dede_user_id: "999".to_string(),
            vip_status: 1,
        };
        let json = serde_json::to_string(&session).unwrap();
        let deserialized: BiliSession = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.sessdata, "test_sd");
        assert_eq!(deserialized.bili_jct, "test_ct");
        assert_eq!(deserialized.dede_user_id, "999");
        assert_eq!(deserialized.vip_status, 1);
    }
}
