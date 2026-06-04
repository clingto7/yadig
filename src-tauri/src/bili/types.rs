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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_video_info_with_pages() {
        let json = r#"{
            "bvid": "BV1GJ411x7h7",
            "aid": 969628065,
            "title": "Test Album",
            "videos": 3,
            "pages": [
                {"cid": 101, "page": 1, "part": "Track 1", "duration": 180},
                {"cid": 102, "page": 2, "part": "Track 2", "duration": 240},
                {"cid": 103, "page": 3, "part": "Track 3", "duration": 200}
            ]
        }"#;
        let info: VideoInfo = serde_json::from_str(json).unwrap();
        assert_eq!(info.bvid, "BV1GJ411x7h7");
        assert_eq!(info.aid, 969628065);
        assert_eq!(info.pages.len(), 3);
        assert_eq!(info.pages[0].part, "Track 1");
        assert_eq!(info.pages[2].cid, 103);
        assert!(info.ugc_season.is_none());
    }

    #[test]
    fn deserialize_video_info_with_ugc_season() {
        let json = r#"{
            "bvid": "BV1test",
            "aid": 1,
            "title": "Collection Video",
            "videos": 1,
            "pages": [{"cid": 10, "page": 1, "part": "Part 1", "duration": 300}],
            "ugc_season": {
                "id": 587216,
                "title": "My Music Collection",
                "sections": [
                    {
                        "episodes": [
                            {"aid": 1, "bvid": "BV1aaa", "cid": 10, "title": "Song A"},
                            {"aid": 2, "bvid": "BV1bbb", "cid": 20, "title": "Song B"}
                        ]
                    }
                ]
            }
        }"#;
        let info: VideoInfo = serde_json::from_str(json).unwrap();
        let season = info.ugc_season.unwrap();
        assert_eq!(season.id, 587216);
        assert_eq!(season.title, "My Music Collection");
        assert_eq!(season.sections[0].episodes.len(), 2);
        assert_eq!(season.sections[0].episodes[0].title, "Song A");
    }

    #[test]
    fn deserialize_dash_audio() {
        let json = r#"{
            "id": 30280,
            "base_url": "https://example.com/audio.m4a",
            "bandwidth": 192000,
            "codecs": "mp4a.40.2"
        }"#;
        let audio: DashAudio = serde_json::from_str(json).unwrap();
        assert_eq!(audio.id, 30280);
        assert_eq!(audio.bandwidth, 192000);
    }

    #[test]
    fn deserialize_bili_response() {
        let json = r#"{
            "code": 0,
            "message": "0",
            "data": {
                "bvid": "BV1test",
                "aid": 1,
                "title": "Test",
                "videos": 1,
                "pages": [{"cid": 1, "page": 1, "part": "Main", "duration": 100}]
            }
        }"#;
        let resp: BiliResponse<VideoInfo> = serde_json::from_str(json).unwrap();
        assert_eq!(resp.code, 0);
        assert!(resp.data.is_some());
        assert_eq!(resp.data.unwrap().title, "Test");
    }
}
