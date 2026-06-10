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
                status: pending_status(),
                error: None,
                source_collection_external_id: None,
                source_collection_title: None,
                target_collection_external_id: None,
                target_collection_title: None,
                resource_id: None,
                resource_type: None,
            })
            .collect();

        Self {
            kind: OperationPlanKind::BiliBatchAudioExtraction,
            items,
        }
    }

    pub fn for_bili_favorite_operation(request: FavoriteOperationPlanRequest) -> Self {
        let kind = match request.action {
            FavoriteOperationAction::Move => OperationPlanKind::BiliBatchMove,
            FavoriteOperationAction::Delete => OperationPlanKind::BiliBatchDelete,
        };
        let action = request.action.as_str().to_string();

        let items = request
            .items
            .into_iter()
            .map(|candidate| {
                let missing_identity = candidate
                    .source_collection_external_id
                    .as_deref()
                    .map(str::trim)
                    .unwrap_or_default()
                    .is_empty()
                    || candidate
                        .resource_id
                        .as_deref()
                        .map(str::trim)
                        .unwrap_or_default()
                        .is_empty()
                    || candidate
                        .resource_type
                        .as_deref()
                        .map(str::trim)
                        .unwrap_or_default()
                        .is_empty();

                let (status, error) = if missing_identity {
                    (
                        "blocked".to_string(),
                        Some(
                            "Missing source favorite folder or remote resource identity"
                                .to_string(),
                        ),
                    )
                } else if request.action == FavoriteOperationAction::Move
                    && request
                        .target_collection_external_id
                        .as_deref()
                        .map(str::trim)
                        .unwrap_or_default()
                        .is_empty()
                {
                    (
                        "blocked".to_string(),
                        Some("Move target favorite folder is required".to_string()),
                    )
                } else if request.action == FavoriteOperationAction::Move
                    && candidate.source_collection_external_id
                        == request.target_collection_external_id
                {
                    (
                        "skipped".to_string(),
                        Some("Item is already in the target favorite folder".to_string()),
                    )
                } else {
                    (pending_status(), None)
                };

                OperationPlanItem {
                    external_id: candidate.external_id,
                    title: candidate.title,
                    action: action.clone(),
                    target: request.target_collection_external_id.clone(),
                    status,
                    error,
                    source_collection_external_id: candidate.source_collection_external_id,
                    source_collection_title: candidate.source_collection_title,
                    target_collection_external_id: request.target_collection_external_id.clone(),
                    target_collection_title: request.target_collection_title.clone(),
                    resource_id: candidate.resource_id,
                    resource_type: candidate.resource_type,
                }
            })
            .collect();

        Self { kind, items }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OperationPlanItem {
    pub external_id: String,
    pub title: String,
    pub action: String,
    pub target: Option<String>,
    #[serde(default = "pending_status")]
    pub status: String,
    #[serde(default)]
    pub error: Option<String>,
    #[serde(default)]
    pub source_collection_external_id: Option<String>,
    #[serde(default)]
    pub source_collection_title: Option<String>,
    #[serde(default)]
    pub target_collection_external_id: Option<String>,
    #[serde(default)]
    pub target_collection_title: Option<String>,
    #[serde(default)]
    pub resource_id: Option<String>,
    #[serde(default)]
    pub resource_type: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioExtractionCandidate {
    pub bvid: String,
    pub title: String,
    pub is_music: bool,
}

fn pending_status() -> String {
    "pending".to_string()
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FavoriteOperationAction {
    Move,
    Delete,
}

impl FavoriteOperationAction {
    fn as_str(&self) -> &'static str {
        match self {
            FavoriteOperationAction::Move => "move",
            FavoriteOperationAction::Delete => "delete",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FavoriteOperationCandidate {
    pub external_id: String,
    pub title: String,
    pub source_collection_external_id: Option<String>,
    pub source_collection_title: Option<String>,
    pub resource_id: Option<String>,
    pub resource_type: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FavoriteOperationPlanRequest {
    pub action: FavoriteOperationAction,
    pub target_collection_external_id: Option<String>,
    pub target_collection_title: Option<String>,
    pub items: Vec<FavoriteOperationCandidate>,
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

#[cfg(test)]
mod favorite_operation_plan_tests {
    use super::*;

    fn candidate() -> FavoriteOperationCandidate {
        FavoriteOperationCandidate {
            external_id: "BV1favorite".to_string(),
            title: "Favorite video".to_string(),
            source_collection_external_id: Some("100".to_string()),
            source_collection_title: Some("Inbox".to_string()),
            resource_id: Some("987654321".to_string()),
            resource_type: Some("2".to_string()),
        }
    }

    #[test]
    fn builds_pending_move_plan_for_valid_favorite_membership() {
        let plan = OperationPlan::for_bili_favorite_operation(FavoriteOperationPlanRequest {
            action: FavoriteOperationAction::Move,
            target_collection_external_id: Some("200".to_string()),
            target_collection_title: Some("Samples".to_string()),
            items: vec![candidate()],
        });

        assert_eq!(plan.kind, OperationPlanKind::BiliBatchMove);
        assert_eq!(plan.items.len(), 1);
        let item = &plan.items[0];
        assert_eq!(item.action, "move");
        assert_eq!(item.status, "pending");
        assert_eq!(item.error, None);
        assert_eq!(item.external_id, "BV1favorite");
        assert_eq!(item.source_collection_external_id.as_deref(), Some("100"));
        assert_eq!(item.source_collection_title.as_deref(), Some("Inbox"));
        assert_eq!(item.target_collection_external_id.as_deref(), Some("200"));
        assert_eq!(item.target_collection_title.as_deref(), Some("Samples"));
        assert_eq!(item.resource_id.as_deref(), Some("987654321"));
        assert_eq!(item.resource_type.as_deref(), Some("2"));
    }

    #[test]
    fn skips_move_plan_item_when_target_equals_source_folder() {
        let plan = OperationPlan::for_bili_favorite_operation(FavoriteOperationPlanRequest {
            action: FavoriteOperationAction::Move,
            target_collection_external_id: Some("100".to_string()),
            target_collection_title: Some("Inbox".to_string()),
            items: vec![candidate()],
        });

        let item = &plan.items[0];
        assert_eq!(plan.kind, OperationPlanKind::BiliBatchMove);
        assert_eq!(item.status, "skipped");
        assert_eq!(
            item.error.as_deref(),
            Some("Item is already in the target favorite folder")
        );
    }

    #[test]
    fn builds_pending_delete_plan_for_valid_favorite_membership() {
        let plan = OperationPlan::for_bili_favorite_operation(FavoriteOperationPlanRequest {
            action: FavoriteOperationAction::Delete,
            target_collection_external_id: None,
            target_collection_title: None,
            items: vec![candidate()],
        });

        let item = &plan.items[0];
        assert_eq!(plan.kind, OperationPlanKind::BiliBatchDelete);
        assert_eq!(item.action, "delete");
        assert_eq!(item.status, "pending");
        assert_eq!(item.target_collection_external_id, None);
        assert_eq!(item.resource_id.as_deref(), Some("987654321"));
        assert_eq!(item.resource_type.as_deref(), Some("2"));
    }

    #[test]
    fn blocks_plan_item_without_remote_identity() {
        let mut unsafe_candidate = candidate();
        unsafe_candidate.source_collection_external_id = None;
        unsafe_candidate.resource_type = None;

        let plan = OperationPlan::for_bili_favorite_operation(FavoriteOperationPlanRequest {
            action: FavoriteOperationAction::Delete,
            target_collection_external_id: None,
            target_collection_title: None,
            items: vec![unsafe_candidate],
        });

        let item = &plan.items[0];
        assert_eq!(item.status, "blocked");
        assert_eq!(
            item.error.as_deref(),
            Some("Missing source favorite folder or remote resource identity")
        );
    }
}
