use serde::{Deserialize, Serialize};

/// Bilibili API response types for video info, player data, and streams.
/// These map to the JSON responses from Bilibili's web API.

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoInfo {
    pub bvid: String,
    pub aid: i64,
    pub title: String,
    pub videos: u32,
    pub pages: Vec<Page>,
    #[serde(default)]
    pub ugc_season: Option<UgcSeason>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Page {
    pub cid: i64,
    pub page: u32,
    pub part: String,
    pub duration: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UgcSeason {
    pub id: i64,
    pub title: String,
    pub sections: Vec<Section>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section {
    pub episodes: Vec<Episode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Episode {
    pub aid: i64,
    pub bvid: String,
    pub cid: i64,
    pub title: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerInfo {
    #[serde(default)]
    pub view_points: Vec<ViewPoint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewPoint {
    pub content: String,
    pub from: f64,
    pub to: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayUrlResponse {
    pub dash: Option<DashInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashInfo {
    pub audio: Vec<DashAudio>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashAudio {
    pub id: i32,
    pub base_url: String,
    pub bandwidth: i64,
    pub codecs: String,
}

/// Bilibili API wrapper response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiliResponse<T> {
    pub code: i32,
    pub data: Option<T>,
    pub message: String,
}
