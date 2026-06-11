use crate::bili::auth::BiliAuth;
use crate::bili::client::{
    BiliClient, FavoriteDeleteBatch, FavoriteMoveBatch, FavoriteMoveResource, FavoriteWriteError,
    FavoriteWriteErrorKind,
};
use crate::error::{Result, YadigError};
use crate::library::{
    AudioExtractionCandidate, BiliSyncResult, BiliSyncScope, FavoriteOperationPlanRequest,
    OperationPlan, OperationPlanItem, OperationPlanKind,
};
use crate::llm::{
    analyze_items, classify_items, test_llm_provider, LlmAnalysisResponse, LlmAnalyzeItemsRequest,
    LlmClassificationResponse, LlmClassifyItemsRequest, LlmProviderConfig, LlmProviderTestError,
    LlmProviderTestResult,
};
use std::future::Future;
use std::sync::Arc;
use std::time::Duration;
use tauri::State;

const FAVORITE_MOVE_BATCH_SIZE: usize = 10;
const FAVORITE_MOVE_BATCH_PAUSE_MS: u64 = 1_500;
const FAVORITE_DELETE_BATCH_SIZE: usize = 10;
const FAVORITE_DELETE_BATCH_PAUSE_MS: u64 = 1_500;

#[tauri::command]
pub async fn bili_sync_library(
    auth: State<'_, BiliAuth>,
    scope: BiliSyncScope,
) -> Result<BiliSyncResult> {
    BiliClient::new((*auth).clone()).sync_library(scope).await
}

#[tauri::command]
pub async fn llm_analyze_items(request: LlmAnalyzeItemsRequest) -> Result<LlmAnalysisResponse> {
    analyze_items(request).await
}

#[tauri::command]
pub async fn llm_classify_items(
    request: LlmClassifyItemsRequest,
) -> Result<LlmClassificationResponse> {
    classify_items(request).await
}

#[tauri::command]
pub async fn llm_test_provider(
    provider: LlmProviderConfig,
) -> std::result::Result<LlmProviderTestResult, LlmProviderTestError> {
    test_llm_provider(provider).await
}

#[tauri::command]
pub async fn create_bili_audio_extraction_plan(
    candidates: Vec<AudioExtractionCandidate>,
) -> Result<OperationPlan> {
    Ok(OperationPlan::for_bili_audio_extraction(candidates))
}

#[tauri::command]
pub async fn create_bili_favorite_operation_plan(
    request: FavoriteOperationPlanRequest,
) -> Result<OperationPlan> {
    Ok(OperationPlan::for_bili_favorite_operation(request))
}

#[tauri::command]
pub async fn execute_bili_audio_extraction_plan(
    auth: State<'_, BiliAuth>,
    plan: OperationPlan,
) -> Result<BiliAudioExtractionExecutionResult> {
    if plan.kind != OperationPlanKind::BiliBatchAudioExtraction {
        return Err(YadigError::Network(
            "Only Bilibili audio extraction plans can be executed by this command".into(),
        ));
    }

    let downloads = dirs_next::download_dir()
        .ok_or_else(|| YadigError::Network("Could not find Downloads folder".into()))?;
    let download_dir = downloads.join("yadig");
    let client = BiliClient::new((*auth).clone());
    let mut results = Vec::new();

    for item in plan.items {
        let url = format!("https://www.bilibili.com/video/{}", item.external_id);
        let result = match client.extract_audio(&url, &download_dir).await {
            Ok(extraction) => BiliAudioExtractionItemResult {
                external_id: item.external_id,
                title: item.title,
                status: "success".to_string(),
                output_paths: extraction
                    .segments
                    .into_iter()
                    .map(|segment| segment.file_path)
                    .collect(),
                error: None,
            },
            Err(err) => BiliAudioExtractionItemResult {
                external_id: item.external_id,
                title: item.title,
                status: "failed".to_string(),
                output_paths: Vec::new(),
                error: Some(err.to_string()),
            },
        };
        results.push(result);
    }

    Ok(BiliAudioExtractionExecutionResult { results })
}

#[tauri::command]
pub async fn execute_bili_favorite_move_plan(
    auth: State<'_, BiliAuth>,
    plan: OperationPlan,
    confirmed: bool,
) -> Result<BiliFavoriteMoveExecutionResult> {
    let client = Arc::new(BiliClient::new((*auth).clone()));
    execute_favorite_move_plan_with_runner(
        plan,
        confirmed,
        FAVORITE_MOVE_BATCH_SIZE,
        move |batch| {
            let client = Arc::clone(&client);
            async move {
                let result = client.move_favorite_resources(&batch).await;
                if result.is_ok() {
                    tokio::time::sleep(Duration::from_millis(FAVORITE_MOVE_BATCH_PAUSE_MS)).await;
                }
                result
            }
        },
    )
    .await
}

#[tauri::command]
pub async fn execute_bili_favorite_delete_plan(
    auth: State<'_, BiliAuth>,
    plan: OperationPlan,
    confirmation_text: String,
) -> Result<BiliFavoriteDeleteExecutionResult> {
    let client = Arc::new(BiliClient::new((*auth).clone()));
    execute_favorite_delete_plan_with_runner(
        plan,
        &confirmation_text,
        FAVORITE_DELETE_BATCH_SIZE,
        move |batch| {
            let client = Arc::clone(&client);
            async move {
                let result = client.delete_favorite_resources(&batch).await;
                if result.is_ok() {
                    tokio::time::sleep(Duration::from_millis(FAVORITE_DELETE_BATCH_PAUSE_MS)).await;
                }
                result
            }
        },
    )
    .await
}

async fn execute_favorite_move_plan_with_runner<F, Fut>(
    mut plan: OperationPlan,
    confirmed: bool,
    batch_size: usize,
    mut runner: F,
) -> Result<BiliFavoriteMoveExecutionResult>
where
    F: FnMut(FavoriteMoveBatch) -> Fut,
    Fut: Future<Output = std::result::Result<(), FavoriteWriteError>>,
{
    if plan.kind != OperationPlanKind::BiliBatchMove {
        return Err(YadigError::Network(
            "Only Bilibili favorite move plans can be executed by this command".into(),
        ));
    }
    if !confirmed {
        return Err(YadigError::Network(
            "Bilibili favorite move execution requires explicit confirmation".into(),
        ));
    }
    if batch_size == 0 {
        return Err(YadigError::Network(
            "Bilibili favorite move batch size must be greater than zero".into(),
        ));
    }

    let mut pending_batch = PendingFavoriteMoveBatch::default();
    let mut stopped = false;

    for index in 0..plan.items.len() {
        if plan.items[index].status != "pending" {
            continue;
        }

        let Some(entry) = FavoriteMovePlanEntry::from_item(index, &plan.items[index]) else {
            plan.items[index].status = "blocked".to_string();
            plan.items[index].error = Some(
                "Missing source favorite folder, target favorite folder, or remote resource identity"
                    .to_string(),
            );
            continue;
        };

        if pending_batch.should_flush_before(&entry, batch_size) {
            if flush_favorite_move_batch(&mut plan, &mut pending_batch, &mut runner).await? {
                stopped = true;
                block_remaining_pending_items(
                    &mut plan,
                    "Execution stopped after Bilibili blocked a move batch",
                );
                break;
            }
        }

        pending_batch.push(entry);
    }

    if !stopped
        && !pending_batch.indices.is_empty()
        && flush_favorite_move_batch(&mut plan, &mut pending_batch, &mut runner).await?
    {
        stopped = true;
        block_remaining_pending_items(
            &mut plan,
            "Execution stopped after Bilibili blocked a move batch",
        );
    }

    Ok(BiliFavoriteMoveExecutionResult { plan, stopped })
}

async fn execute_favorite_delete_plan_with_runner<F, Fut>(
    mut plan: OperationPlan,
    confirmation_text: &str,
    batch_size: usize,
    mut runner: F,
) -> Result<BiliFavoriteDeleteExecutionResult>
where
    F: FnMut(FavoriteDeleteBatch) -> Fut,
    Fut: Future<Output = std::result::Result<(), FavoriteWriteError>>,
{
    if plan.kind != OperationPlanKind::BiliBatchDelete {
        return Err(YadigError::Network(
            "Only Bilibili favorite delete plans can be executed by this command".into(),
        ));
    }
    if confirmation_text.trim() != "DELETE" {
        return Err(YadigError::Network(
            "Bilibili favorite delete execution is destructive; type DELETE to confirm".into(),
        ));
    }
    if batch_size == 0 {
        return Err(YadigError::Network(
            "Bilibili favorite delete batch size must be greater than zero".into(),
        ));
    }

    let mut pending_batch = PendingFavoriteDeleteBatch::default();
    let mut stopped = false;

    for index in 0..plan.items.len() {
        if plan.items[index].status != "pending" {
            continue;
        }

        let Some(entry) = FavoriteDeletePlanEntry::from_item(index, &plan.items[index]) else {
            plan.items[index].status = "blocked".to_string();
            plan.items[index].error =
                Some("Missing source favorite folder or remote resource identity".to_string());
            continue;
        };

        if pending_batch.should_flush_before(&entry, batch_size) {
            if flush_favorite_delete_batch(&mut plan, &mut pending_batch, &mut runner).await? {
                stopped = true;
                block_remaining_pending_items(
                    &mut plan,
                    "Execution stopped after Bilibili blocked a delete batch",
                );
                break;
            }
        }

        pending_batch.push(entry);
    }

    if !stopped
        && !pending_batch.indices.is_empty()
        && flush_favorite_delete_batch(&mut plan, &mut pending_batch, &mut runner).await?
    {
        stopped = true;
        block_remaining_pending_items(
            &mut plan,
            "Execution stopped after Bilibili blocked a delete batch",
        );
    }

    Ok(BiliFavoriteDeleteExecutionResult { plan, stopped })
}

async fn flush_favorite_move_batch<F, Fut>(
    plan: &mut OperationPlan,
    pending_batch: &mut PendingFavoriteMoveBatch,
    runner: &mut F,
) -> Result<bool>
where
    F: FnMut(FavoriteMoveBatch) -> Fut,
    Fut: Future<Output = std::result::Result<(), FavoriteWriteError>>,
{
    let Some(batch) = pending_batch.take_batch() else {
        return Ok(false);
    };
    let indices = std::mem::take(&mut pending_batch.indices);

    match runner(batch).await {
        Ok(()) => {
            for index in indices {
                mark_plan_item(&mut plan.items[index], "success", None);
            }
            Ok(false)
        }
        Err(err) => {
            let status = match err.kind {
                FavoriteWriteErrorKind::Failed => "failed",
                FavoriteWriteErrorKind::Blocked => "blocked",
            };
            for index in indices {
                mark_plan_item(&mut plan.items[index], status, Some(err.message.clone()));
            }
            Ok(err.kind == FavoriteWriteErrorKind::Blocked)
        }
    }
}

async fn flush_favorite_delete_batch<F, Fut>(
    plan: &mut OperationPlan,
    pending_batch: &mut PendingFavoriteDeleteBatch,
    runner: &mut F,
) -> Result<bool>
where
    F: FnMut(FavoriteDeleteBatch) -> Fut,
    Fut: Future<Output = std::result::Result<(), FavoriteWriteError>>,
{
    let Some(batch) = pending_batch.take_batch() else {
        return Ok(false);
    };
    let indices = std::mem::take(&mut pending_batch.indices);

    match runner(batch).await {
        Ok(()) => {
            for index in indices {
                mark_plan_item(&mut plan.items[index], "success", None);
            }
            Ok(false)
        }
        Err(err) => {
            let status = match err.kind {
                FavoriteWriteErrorKind::Failed => "failed",
                FavoriteWriteErrorKind::Blocked => "blocked",
            };
            for index in indices {
                mark_plan_item(&mut plan.items[index], status, Some(err.message.clone()));
            }
            Ok(err.kind == FavoriteWriteErrorKind::Blocked)
        }
    }
}

fn block_remaining_pending_items(plan: &mut OperationPlan, message: &str) {
    for item in &mut plan.items {
        if item.status == "pending" {
            mark_plan_item(item, "blocked", Some(message.to_string()));
        }
    }
}

fn mark_plan_item(item: &mut OperationPlanItem, status: &str, error: Option<String>) {
    item.status = status.to_string();
    item.error = error;
}

#[derive(Default)]
struct PendingFavoriteDeleteBatch {
    source_media_id: Option<String>,
    resources: Vec<FavoriteMoveResource>,
    indices: Vec<usize>,
}

impl PendingFavoriteDeleteBatch {
    fn should_flush_before(&self, entry: &FavoriteDeletePlanEntry, batch_size: usize) -> bool {
        if self.indices.is_empty() {
            return false;
        }
        self.indices.len() >= batch_size
            || self.source_media_id.as_deref() != Some(entry.source_media_id.as_str())
    }

    fn push(&mut self, entry: FavoriteDeletePlanEntry) {
        if self.indices.is_empty() {
            self.source_media_id = Some(entry.source_media_id);
        }
        self.resources.push(FavoriteMoveResource {
            id: entry.resource_id,
            resource_type: entry.resource_type,
        });
        self.indices.push(entry.index);
    }

    fn take_batch(&mut self) -> Option<FavoriteDeleteBatch> {
        if self.indices.is_empty() {
            return None;
        }
        Some(FavoriteDeleteBatch {
            source_media_id: self.source_media_id.take()?,
            resources: std::mem::take(&mut self.resources),
        })
    }
}

#[derive(Default)]
struct PendingFavoriteMoveBatch {
    source_media_id: Option<String>,
    target_media_id: Option<String>,
    resources: Vec<FavoriteMoveResource>,
    indices: Vec<usize>,
}

impl PendingFavoriteMoveBatch {
    fn should_flush_before(&self, entry: &FavoriteMovePlanEntry, batch_size: usize) -> bool {
        if self.indices.is_empty() {
            return false;
        }
        self.indices.len() >= batch_size
            || self.source_media_id.as_deref() != Some(entry.source_media_id.as_str())
            || self.target_media_id.as_deref() != Some(entry.target_media_id.as_str())
    }

    fn push(&mut self, entry: FavoriteMovePlanEntry) {
        if self.indices.is_empty() {
            self.source_media_id = Some(entry.source_media_id);
            self.target_media_id = Some(entry.target_media_id);
        }
        self.resources.push(FavoriteMoveResource {
            id: entry.resource_id,
            resource_type: entry.resource_type,
        });
        self.indices.push(entry.index);
    }

    fn take_batch(&mut self) -> Option<FavoriteMoveBatch> {
        if self.indices.is_empty() {
            return None;
        }
        Some(FavoriteMoveBatch {
            source_media_id: self.source_media_id.take()?,
            target_media_id: self.target_media_id.take()?,
            resources: std::mem::take(&mut self.resources),
        })
    }
}

struct FavoriteDeletePlanEntry {
    index: usize,
    source_media_id: String,
    resource_id: String,
    resource_type: String,
}

impl FavoriteDeletePlanEntry {
    fn from_item(index: usize, item: &OperationPlanItem) -> Option<Self> {
        let source_media_id = non_empty(item.source_collection_external_id.as_deref())?;
        let resource_id = non_empty(item.resource_id.as_deref())?;
        let resource_type = non_empty(item.resource_type.as_deref())?;

        Some(Self {
            index,
            source_media_id,
            resource_id,
            resource_type,
        })
    }
}

struct FavoriteMovePlanEntry {
    index: usize,
    source_media_id: String,
    target_media_id: String,
    resource_id: String,
    resource_type: String,
}

impl FavoriteMovePlanEntry {
    fn from_item(index: usize, item: &OperationPlanItem) -> Option<Self> {
        let source_media_id = non_empty(item.source_collection_external_id.as_deref())?;
        let target_media_id = non_empty(item.target_collection_external_id.as_deref())?;
        let resource_id = non_empty(item.resource_id.as_deref())?;
        let resource_type = non_empty(item.resource_type.as_deref())?;

        Some(Self {
            index,
            source_media_id,
            target_media_id,
            resource_id,
            resource_type,
        })
    }
}

fn non_empty(value: Option<&str>) -> Option<String> {
    let value = value?.trim();
    (!value.is_empty()).then(|| value.to_string())
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BiliFavoriteMoveExecutionResult {
    pub plan: OperationPlan,
    pub stopped: bool,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BiliFavoriteDeleteExecutionResult {
    pub plan: OperationPlan,
    pub stopped: bool,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BiliAudioExtractionExecutionResult {
    pub results: Vec<BiliAudioExtractionItemResult>,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BiliAudioExtractionItemResult {
    pub external_id: String,
    pub title: String,
    pub status: String,
    pub output_paths: Vec<String>,
    pub error: Option<String>,
}

#[cfg(test)]
mod favorite_move_execution_tests {
    use super::*;
    use crate::bili::client::{FavoriteMoveBatch, FavoriteWriteError, FavoriteWriteErrorKind};
    use crate::library::OperationPlanItem;
    use std::cell::RefCell;

    fn move_item(index: usize) -> OperationPlanItem {
        OperationPlanItem {
            external_id: format!("BV{index:010}"),
            title: format!("Favorite {index}"),
            action: "move".to_string(),
            target: Some("200".to_string()),
            status: "pending".to_string(),
            error: None,
            source_collection_external_id: Some("100".to_string()),
            source_collection_title: Some("Inbox".to_string()),
            target_collection_external_id: Some("200".to_string()),
            target_collection_title: Some("Samples".to_string()),
            resource_id: Some(format!("987654{index}")),
            resource_type: Some("2".to_string()),
        }
    }

    fn move_plan(count: usize) -> OperationPlan {
        OperationPlan {
            kind: OperationPlanKind::BiliBatchMove,
            items: (0..count).map(move_item).collect(),
        }
    }

    #[test]
    fn favorite_move_execution_requires_explicit_confirmation() {
        let calls = RefCell::new(Vec::<FavoriteMoveBatch>::new());

        let err = futures::executor::block_on(execute_favorite_move_plan_with_runner(
            move_plan(1),
            false,
            10,
            |batch| {
                calls.borrow_mut().push(batch);
                async { Ok(()) }
            },
        ))
        .expect_err("confirmation is required");

        assert!(err.to_string().contains("explicit confirmation"));
        assert!(calls.borrow().is_empty());
    }

    #[test]
    fn favorite_move_execution_batches_pending_items_and_marks_success() {
        let calls = RefCell::new(Vec::<FavoriteMoveBatch>::new());

        let result = futures::executor::block_on(execute_favorite_move_plan_with_runner(
            move_plan(11),
            true,
            10,
            |batch| {
                calls.borrow_mut().push(batch);
                async { Ok(()) }
            },
        ))
        .expect("move execution should succeed");

        let calls = calls.borrow();
        assert_eq!(calls.len(), 2);
        assert_eq!(calls[0].source_media_id, "100");
        assert_eq!(calls[0].target_media_id, "200");
        assert_eq!(calls[0].resources.len(), 10);
        assert_eq!(calls[0].resources[0].id, "9876540");
        assert_eq!(calls[0].resources[0].resource_type, "2");
        assert_eq!(calls[1].resources.len(), 1);
        assert!(result
            .plan
            .items
            .iter()
            .all(|item| item.status == "success"));
        assert!(!result.stopped);
    }

    #[test]
    fn favorite_move_execution_blocks_remaining_items_after_security_failure() {
        let calls = RefCell::new(0usize);

        let result = futures::executor::block_on(execute_favorite_move_plan_with_runner(
            move_plan(11),
            true,
            10,
            |_batch| {
                *calls.borrow_mut() += 1;
                async {
                    Err(FavoriteWriteError {
                        kind: FavoriteWriteErrorKind::Blocked,
                        message: "Bilibili favorite write blocked (-111): csrf failed".to_string(),
                    })
                }
            },
        ))
        .expect("blocked item statuses should be returned");

        assert_eq!(*calls.borrow(), 1);
        assert!(result.stopped);
        assert!(result.plan.items[..10].iter().all(|item| {
            item.status == "blocked"
                && item
                    .error
                    .as_deref()
                    .unwrap_or_default()
                    .contains("csrf failed")
        }));
        assert_eq!(result.plan.items[10].status, "blocked");
        assert!(result.plan.items[10]
            .error
            .as_deref()
            .unwrap_or_default()
            .contains("Execution stopped"));
    }
}

#[cfg(test)]
mod favorite_delete_execution_tests {
    use super::*;
    use crate::bili::client::{FavoriteDeleteBatch, FavoriteWriteError, FavoriteWriteErrorKind};
    use crate::library::OperationPlanItem;
    use std::cell::RefCell;

    fn delete_item(index: usize) -> OperationPlanItem {
        OperationPlanItem {
            external_id: format!("BV{index:010}"),
            title: format!("Favorite {index}"),
            action: "delete".to_string(),
            target: None,
            status: "pending".to_string(),
            error: None,
            source_collection_external_id: Some("100".to_string()),
            source_collection_title: Some("Inbox".to_string()),
            target_collection_external_id: None,
            target_collection_title: None,
            resource_id: Some(format!("987654{index}")),
            resource_type: Some("2".to_string()),
        }
    }

    fn delete_plan(count: usize) -> OperationPlan {
        OperationPlan {
            kind: OperationPlanKind::BiliBatchDelete,
            items: (0..count).map(delete_item).collect(),
        }
    }

    #[test]
    fn favorite_delete_execution_requires_destructive_confirmation_text() {
        let calls = RefCell::new(Vec::<FavoriteDeleteBatch>::new());

        let err = futures::executor::block_on(execute_favorite_delete_plan_with_runner(
            delete_plan(1),
            "delete",
            10,
            |batch| {
                calls.borrow_mut().push(batch);
                async { Ok(()) }
            },
        ))
        .expect_err("destructive confirmation is required");

        assert!(err.to_string().contains("type DELETE"));
        assert!(calls.borrow().is_empty());
    }

    #[test]
    fn favorite_delete_execution_batches_pending_items_and_marks_success() {
        let calls = RefCell::new(Vec::<FavoriteDeleteBatch>::new());

        let result = futures::executor::block_on(execute_favorite_delete_plan_with_runner(
            delete_plan(11),
            "DELETE",
            10,
            |batch| {
                calls.borrow_mut().push(batch);
                async { Ok(()) }
            },
        ))
        .expect("delete execution should succeed");

        let calls = calls.borrow();
        assert_eq!(calls.len(), 2);
        assert_eq!(calls[0].source_media_id, "100");
        assert_eq!(calls[0].resources.len(), 10);
        assert_eq!(calls[0].resources[0].id, "9876540");
        assert_eq!(calls[0].resources[0].resource_type, "2");
        assert_eq!(calls[1].resources.len(), 1);
        assert!(result
            .plan
            .items
            .iter()
            .all(|item| item.status == "success"));
        assert!(!result.stopped);
    }

    #[test]
    fn favorite_delete_execution_blocks_remaining_items_after_security_failure() {
        let calls = RefCell::new(0usize);

        let result = futures::executor::block_on(execute_favorite_delete_plan_with_runner(
            delete_plan(11),
            "DELETE",
            10,
            |_batch| {
                *calls.borrow_mut() += 1;
                async {
                    Err(FavoriteWriteError {
                        kind: FavoriteWriteErrorKind::Blocked,
                        message: "Bilibili favorite write blocked (-111): csrf failed".to_string(),
                    })
                }
            },
        ))
        .expect("blocked item statuses should be returned");

        assert_eq!(*calls.borrow(), 1);
        assert!(result.stopped);
        assert!(result.plan.items[..10].iter().all(|item| {
            item.status == "blocked"
                && item
                    .error
                    .as_deref()
                    .unwrap_or_default()
                    .contains("csrf failed")
        }));
        assert_eq!(result.plan.items[10].status, "blocked");
        assert!(result.plan.items[10]
            .error
            .as_deref()
            .unwrap_or_default()
            .contains("Execution stopped"));
    }
}
