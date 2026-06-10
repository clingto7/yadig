use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LibraryItemType {
    BiliFavoriteVideo,
    BiliWatchLaterVideo,
    BiliFollowedUp,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LibraryCollectionType {
    BiliFavoriteFolder,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BiliResourceKind {
    FavoriteVideo,
    WatchLaterVideo,
    FollowedUp,
}

impl BiliResourceKind {
    fn item_type(&self) -> LibraryItemType {
        match self {
            BiliResourceKind::FavoriteVideo => LibraryItemType::BiliFavoriteVideo,
            BiliResourceKind::WatchLaterVideo => LibraryItemType::BiliWatchLaterVideo,
            BiliResourceKind::FollowedUp => LibraryItemType::BiliFollowedUp,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryCollection {
    pub source: String,
    pub external_id: String,
    pub collection_type: LibraryCollectionType,
    pub title: String,
    pub raw_metadata: serde_json::Value,
}

impl LibraryCollection {
    pub fn from_bili_favorite_folder(
        media_id: String,
        title: String,
        raw_metadata: serde_json::Value,
    ) -> Self {
        Self {
            source: "bilibili".to_string(),
            external_id: media_id,
            collection_type: LibraryCollectionType::BiliFavoriteFolder,
            title,
            raw_metadata,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryItem {
    pub source: String,
    pub external_id: String,
    pub item_type: LibraryItemType,
    pub title: String,
    pub author: Option<String>,
    pub url: Option<String>,
    pub image_url: Option<String>,
    pub raw_metadata: serde_json::Value,
}

impl LibraryItem {
    pub fn from_bili_video(
        kind: BiliResourceKind,
        bvid: String,
        title: String,
        author: Option<String>,
        raw_metadata: serde_json::Value,
    ) -> Self {
        Self {
            source: "bilibili".to_string(),
            external_id: bvid.clone(),
            item_type: kind.item_type(),
            title,
            author,
            url: Some(format!("https://www.bilibili.com/video/{}", bvid)),
            image_url: raw_metadata
                .get("cover")
                .and_then(|value| value.as_str())
                .map(ToString::to_string),
            raw_metadata,
        }
    }

    pub fn from_bili_followed_up(
        mid: String,
        name: String,
        raw_metadata: serde_json::Value,
    ) -> Self {
        Self {
            source: "bilibili".to_string(),
            external_id: mid.clone(),
            item_type: LibraryItemType::BiliFollowedUp,
            title: name,
            author: None,
            url: Some(format!("https://space.bilibili.com/{}", mid)),
            image_url: raw_metadata
                .get("face")
                .and_then(|value| value.as_str())
                .map(ToString::to_string),
            raw_metadata,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryItemCollection {
    pub source: String,
    pub item_external_id: String,
    pub item_type: LibraryItemType,
    pub collection_external_id: String,
    pub collection_type: LibraryCollectionType,
    pub raw_metadata: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OperationPlanKind {
    BiliBatchAudioExtraction,
    BiliBatchMove,
    BiliBatchDelete,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OperationPlan {
    pub kind: OperationPlanKind,
    pub items: Vec<OperationPlanItem>,
}

impl OperationPlan {
    pub fn for_bili_audio_extraction(candidates: Vec<AudioExtractionCandidate>) -> Self {
        let items = candidates
            .into_iter()
            .filter(|candidate| candidate.is_music)
            .map(|candidate| OperationPlanItem {
                external_id: candidate.bvid,
                title: candidate.title,
                action: "extract_audio".to_string(),
                target: None,
            })
            .collect();

        Self {
            kind: OperationPlanKind::BiliBatchAudioExtraction,
            items,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OperationPlanItem {
    pub external_id: String,
    pub title: String,
    pub action: String,
    pub target: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioExtractionCandidate {
    pub bvid: String,
    pub title: String,
    pub is_music: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BiliSyncScope {
    pub favorites: bool,
    pub follows: bool,
    pub watch_later: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BiliSyncResult {
    pub items: Vec<LibraryItem>,
    pub collections: Vec<LibraryCollection>,
    pub item_collections: Vec<LibraryItemCollection>,
    pub synced_favorites: bool,
    pub synced_follows: bool,
    pub synced_watch_later: bool,
}
