use crate::bili::auth::BiliAuth;
use crate::bili::client::{
    BiliClient, FavoriteDeleteBatch, FavoriteFolderCreateRequest, FavoriteFolderRenameRequest,
    FavoriteMoveBatch, FavoriteMoveResource, FavoriteWriteError, FavoriteWriteErrorKind,
};
use crate::error::{Result, YadigError};
use crate::library::{
    AudioExtractionCandidate, BiliSyncResult, BiliSyncScope, FavoriteFolderCreatePlanRequest,
    FavoriteFolderRenamePlanRequest, FavoriteOperationPlanRequest, LibraryCollection,
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
const FAVORITE_COPY_BATCH_SIZE: usize = 10;
const FAVORITE_COPY_BATCH_PAUSE_MS: u64 = 1_500;
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
pub async fn create_bili_favorite_folder_create_plan(
    request: FavoriteFolderCreatePlanRequest,
) -> Result<OperationPlan> {
    Ok(OperationPlan::for_bili_favorite_folder_create(request))
}

#[tauri::command]
pub async fn create_bili_favorite_folder_rename_plan(
    request: FavoriteFolderRenamePlanRequest,
) -> Result<OperationPlan> {
    Ok(OperationPlan::for_bili_favorite_folder_rename(request))
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
pub async fn execute_bili_favorite_copy_plan(
    auth: State<'_, BiliAuth>,
    plan: OperationPlan,
    confirmed: bool,
) -> Result<BiliFavoriteMoveExecutionResult> {
    let client = Arc::new(BiliClient::new((*auth).clone()));
    execute_favorite_copy_plan_with_runner(
        plan,
        confirmed,
        FAVORITE_COPY_BATCH_SIZE,
        move |batch| {
            let client = Arc::clone(&client);
            async move {
                let result = client.copy_favorite_resources(&batch).await;
                if result.is_ok() {
                    tokio::time::sleep(Duration::from_millis(FAVORITE_COPY_BATCH_PAUSE_MS)).await;
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

#[tauri::command]
pub async fn execute_bili_favorite_folder_create_plan(
    auth: State<'_, BiliAuth>,
    plan: OperationPlan,
    confirmed: bool,
) -> Result<BiliFavoriteFolderCreateExecutionResult> {
    let client = Arc::new(BiliClient::new((*auth).clone()));
    execute_favorite_folder_create_plan_with_runner(plan, confirmed, move |request| {
        let client = Arc::clone(&client);
        async move { client.create_favorite_folder(&request).await }
    })
    .await
}

#[tauri::command]
pub async fn execute_bili_favorite_folder_rename_plan(
    auth: State<'_, BiliAuth>,
    plan: OperationPlan,
    confirmed: bool,
) -> Result<BiliFavoriteFolderRenameExecutionResult> {
    let client = Arc::new(BiliClient::new((*auth).clone()));
    execute_favorite_folder_rename_plan_with_runner(plan, confirmed, move |request| {
        let client = Arc::clone(&client);
        async move { client.rename_favorite_folder(&request).await }
    })
    .await
}

async fn execute_favorite_move_plan_with_runner<F, Fut>(
    plan: OperationPlan,
    confirmed: bool,
    batch_size: usize,
    runner: F,
) -> Result<BiliFavoriteMoveExecutionResult>
where
    F: FnMut(FavoriteMoveBatch) -> Fut,
    Fut: Future<Output = std::result::Result<(), FavoriteWriteError>>,
{
    execute_favorite_transfer_plan_with_runner(
        plan,
        confirmed,
        batch_size,
        OperationPlanKind::BiliBatchMove,
        "move",
        runner,
    )
    .await
}

async fn execute_favorite_copy_plan_with_runner<F, Fut>(
    plan: OperationPlan,
    confirmed: bool,
    batch_size: usize,
    runner: F,
) -> Result<BiliFavoriteMoveExecutionResult>
where
    F: FnMut(FavoriteMoveBatch) -> Fut,
    Fut: Future<Output = std::result::Result<(), FavoriteWriteError>>,
{
    execute_favorite_transfer_plan_with_runner(
        plan,
        confirmed,
        batch_size,
        OperationPlanKind::BiliBatchCopy,
        "copy",
        runner,
    )
    .await
}

async fn execute_favorite_transfer_plan_with_runner<F, Fut>(
    mut plan: OperationPlan,
    confirmed: bool,
    batch_size: usize,
    expected_kind: OperationPlanKind,
    action_label: &str,
    mut runner: F,
) -> Result<BiliFavoriteMoveExecutionResult>
where
    F: FnMut(FavoriteMoveBatch) -> Fut,
    Fut: Future<Output = std::result::Result<(), FavoriteWriteError>>,
{
    if plan.kind != expected_kind {
        return Err(YadigError::Network(format!(
            "Only Bilibili favorite {action_label} plans can be executed by this command"
        )));
    }
    if !confirmed {
        return Err(YadigError::Network(format!(
            "Bilibili favorite {action_label} execution requires explicit confirmation"
        )));
    }
    if batch_size == 0 {
        return Err(YadigError::Network(format!(
            "Bilibili favorite {action_label} batch size must be greater than zero"
        )));
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
                    &format!("Execution stopped after Bilibili blocked a {action_label} batch"),
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
            &format!("Execution stopped after Bilibili blocked a {action_label} batch"),
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

async fn execute_favorite_folder_create_plan_with_runner<F, Fut>(
    mut plan: OperationPlan,
    confirmed: bool,
    mut runner: F,
) -> Result<BiliFavoriteFolderCreateExecutionResult>
where
    F: FnMut(FavoriteFolderCreateRequest) -> Fut,
    Fut: Future<Output = std::result::Result<LibraryCollection, FavoriteWriteError>>,
{
    if plan.kind != OperationPlanKind::BiliFavoriteFolderCreate {
        return Err(YadigError::Network(
            "Only Bilibili favorite folder create plans can be executed by this command".into(),
        ));
    }
    if !confirmed {
        return Err(YadigError::Network(
            "Bilibili favorite folder creation requires explicit confirmation".into(),
        ));
    }

    for index in 0..plan.items.len() {
        if plan.items[index].status != "pending" {
            continue;
        }

        let Some(request) = favorite_folder_create_request_from_item(&plan.items[index]) else {
            mark_plan_item(
                &mut plan.items[index],
                "blocked",
                Some("Missing favorite folder title or privacy setting".to_string()),
            );
            continue;
        };

        match runner(request).await {
            Ok(collection) => {
                plan.items[index].external_id = collection.external_id.clone();
                plan.items[index].target_collection_external_id = Some(collection.external_id);
                mark_plan_item(&mut plan.items[index], "success", None);
            }
            Err(err) => {
                let status = match err.kind {
                    FavoriteWriteErrorKind::Failed => "failed",
                    FavoriteWriteErrorKind::Blocked => "blocked",
                };
                mark_plan_item(&mut plan.items[index], status, Some(err.message));
            }
        }
    }

    Ok(BiliFavoriteFolderCreateExecutionResult { plan })
}

async fn execute_favorite_folder_rename_plan_with_runner<F, Fut>(
    mut plan: OperationPlan,
    confirmed: bool,
    mut runner: F,
) -> Result<BiliFavoriteFolderRenameExecutionResult>
where
    F: FnMut(FavoriteFolderRenameRequest) -> Fut,
    Fut: Future<Output = std::result::Result<LibraryCollection, FavoriteWriteError>>,
{
    if plan.kind != OperationPlanKind::BiliFavoriteFolderRename {
        return Err(YadigError::Network(
            "Only Bilibili favorite folder rename plans can be executed by this command".into(),
        ));
    }
    if !confirmed {
        return Err(YadigError::Network(
            "Bilibili favorite folder rename requires explicit confirmation".into(),
        ));
    }

    for index in 0..plan.items.len() {
        if plan.items[index].status != "pending" {
            continue;
        }

        let Some(request) = favorite_folder_rename_request_from_item(&plan.items[index]) else {
            mark_plan_item(
                &mut plan.items[index],
                "blocked",
                Some("Missing favorite folder id, new title, or preserved metadata".to_string()),
            );
            continue;
        };

        match runner(request).await {
            Ok(collection) => {
                plan.items[index].external_id = collection.external_id.clone();
                plan.items[index].title = collection.title;
                plan.items[index].target_collection_external_id = Some(collection.external_id);
                plan.items[index].metadata = collection.raw_metadata;
                mark_plan_item(&mut plan.items[index], "success", None);
            }
            Err(err) => {
                let status = match err.kind {
                    FavoriteWriteErrorKind::Failed => "failed",
                    FavoriteWriteErrorKind::Blocked => "blocked",
                };
                mark_plan_item(&mut plan.items[index], status, Some(err.message));
            }
        }
    }

    Ok(BiliFavoriteFolderRenameExecutionResult { plan })
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

fn favorite_folder_create_request_from_item(
    item: &OperationPlanItem,
) -> Option<FavoriteFolderCreateRequest> {
    let title = non_empty(Some(&item.title))?;
    let privacy = item.target.as_deref()?.trim().parse::<i32>().ok()?;
    Some(FavoriteFolderCreateRequest {
        title,
        intro: item
            .target_collection_title
            .as_deref()
            .unwrap_or_default()
            .trim()
            .to_string(),
        privacy,
    })
}

fn favorite_folder_rename_request_from_item(
    item: &OperationPlanItem,
) -> Option<FavoriteFolderRenameRequest> {
    let media_id = non_empty(item.source_collection_external_id.as_deref())
        .or_else(|| non_empty(Some(&item.external_id)))?;
    let title = non_empty(item.target.as_deref())
        .or_else(|| non_empty(item.target_collection_title.as_deref()))?;
    let intro = item
        .metadata
        .get("intro")
        .and_then(|value| value.as_str())
        .unwrap_or_default()
        .to_string();
    let privacy = item
        .metadata
        .get("privacy")
        .and_then(|value| {
            value
                .as_i64()
                .or_else(|| value.as_str().and_then(|text| text.parse::<i64>().ok()))
        })
        .and_then(|value| i32::try_from(value).ok())?;
    let cover = item
        .metadata
        .get("cover")
        .and_then(|value| value.as_str())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string);

    Some(FavoriteFolderRenameRequest {
        media_id,
        title,
        intro,
        privacy,
        cover,
    })
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
pub struct BiliFavoriteFolderCreateExecutionResult {
    pub plan: OperationPlan,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BiliFavoriteFolderRenameExecutionResult {
    pub plan: OperationPlan,
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
            metadata: serde_json::json!({}),
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
mod favorite_copy_execution_tests {
    use super::*;
    use crate::bili::client::{FavoriteMoveBatch, FavoriteWriteError, FavoriteWriteErrorKind};
    use crate::library::OperationPlanItem;
    use std::cell::RefCell;

    fn copy_item(index: usize) -> OperationPlanItem {
        OperationPlanItem {
            external_id: format!("BV{index:010}"),
            title: format!("Favorite {index}"),
            action: "copy".to_string(),
            target: Some("200".to_string()),
            status: "pending".to_string(),
            error: None,
            source_collection_external_id: Some("100".to_string()),
            source_collection_title: Some("Inbox".to_string()),
            target_collection_external_id: Some("200".to_string()),
            target_collection_title: Some("Samples".to_string()),
            resource_id: Some(format!("987654{index}")),
            resource_type: Some("2".to_string()),
            metadata: serde_json::json!({}),
        }
    }

    fn copy_plan(count: usize) -> OperationPlan {
        OperationPlan {
            kind: OperationPlanKind::BiliBatchCopy,
            items: (0..count).map(copy_item).collect(),
        }
    }

    #[test]
    fn favorite_copy_execution_requires_explicit_confirmation() {
        let calls = RefCell::new(Vec::<FavoriteMoveBatch>::new());

        let err = futures::executor::block_on(execute_favorite_copy_plan_with_runner(
            copy_plan(1),
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
    fn favorite_copy_execution_batches_pending_items_and_marks_success() {
        let calls = RefCell::new(Vec::<FavoriteMoveBatch>::new());

        let result = futures::executor::block_on(execute_favorite_copy_plan_with_runner(
            copy_plan(11),
            true,
            10,
            |batch| {
                calls.borrow_mut().push(batch);
                async { Ok(()) }
            },
        ))
        .expect("copy execution should succeed");

        let calls = calls.borrow();
        assert_eq!(calls.len(), 2);
        assert_eq!(calls[0].source_media_id, "100");
        assert_eq!(calls[0].target_media_id, "200");
        assert_eq!(calls[0].resources.len(), 10);
        assert_eq!(calls[1].resources.len(), 1);
        assert!(result
            .plan
            .items
            .iter()
            .all(|item| item.status == "success"));
        assert!(!result.stopped);
    }

    #[test]
    fn favorite_copy_execution_blocks_remaining_items_after_security_failure() {
        let calls = RefCell::new(0usize);

        let result = futures::executor::block_on(execute_favorite_copy_plan_with_runner(
            copy_plan(11),
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
mod favorite_folder_create_execution_tests {
    use super::*;
    use crate::bili::client::{FavoriteFolderCreateRequest, FavoriteWriteError, FavoriteWriteErrorKind};
    use crate::library::OperationPlanItem;
    use std::cell::RefCell;

    fn create_plan() -> OperationPlan {
        OperationPlan {
            kind: OperationPlanKind::BiliFavoriteFolderCreate,
            items: vec![OperationPlanItem {
                external_id: "pending".to_string(),
                title: "Disposable".to_string(),
                action: "create_folder".to_string(),
                target: Some("1".to_string()),
                status: "pending".to_string(),
                error: None,
                source_collection_external_id: None,
                source_collection_title: None,
                target_collection_external_id: None,
                target_collection_title: Some("Temporary test folder".to_string()),
                resource_id: None,
                resource_type: None,
                metadata: serde_json::json!({}),
            }],
        }
    }

    #[test]
    fn favorite_folder_create_execution_requires_explicit_confirmation() {
        let calls = RefCell::new(Vec::<FavoriteFolderCreateRequest>::new());

        let err = futures::executor::block_on(execute_favorite_folder_create_plan_with_runner(
            create_plan(),
            false,
            |request| {
                calls.borrow_mut().push(request);
                async {
                    Ok(LibraryCollection::from_bili_favorite_folder(
                        "300".to_string(),
                        "Disposable".to_string(),
                        serde_json::json!({"id": 300, "title": "Disposable"}),
                    ))
                }
            },
        ))
        .expect_err("confirmation is required");

        assert!(err.to_string().contains("explicit confirmation"));
        assert!(calls.borrow().is_empty());
    }

    #[test]
    fn favorite_folder_create_execution_marks_success_with_remote_id() {
        let calls = RefCell::new(Vec::<FavoriteFolderCreateRequest>::new());

        let result = futures::executor::block_on(execute_favorite_folder_create_plan_with_runner(
            create_plan(),
            true,
            |request| {
                calls.borrow_mut().push(request);
                async {
                    Ok(LibraryCollection::from_bili_favorite_folder(
                        "300".to_string(),
                        "Disposable".to_string(),
                        serde_json::json!({"id": 300, "title": "Disposable"}),
                    ))
                }
            },
        ))
        .expect("folder create execution should succeed");

        let calls = calls.borrow();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].title, "Disposable");
        assert_eq!(calls[0].intro, "Temporary test folder");
        assert_eq!(calls[0].privacy, 1);
        assert_eq!(result.plan.items[0].status, "success");
        assert_eq!(result.plan.items[0].external_id, "300");
        assert_eq!(
            result.plan.items[0].target_collection_external_id.as_deref(),
            Some("300")
        );
    }

    #[test]
    fn favorite_folder_create_execution_marks_blocked_errors() {
        let result = futures::executor::block_on(execute_favorite_folder_create_plan_with_runner(
            create_plan(),
            true,
            |_request| async {
                Err(FavoriteWriteError {
                    kind: FavoriteWriteErrorKind::Blocked,
                    message: "Bilibili favorite write blocked (-111): csrf failed".to_string(),
                })
            },
        ))
        .expect("blocked item status should be returned");

        assert_eq!(result.plan.items[0].status, "blocked");
        assert!(result.plan.items[0]
            .error
            .as_deref()
            .unwrap_or_default()
            .contains("csrf failed"));
    }
}

#[cfg(test)]
mod favorite_folder_rename_execution_tests {
    use super::*;
    use crate::bili::client::{
        FavoriteFolderRenameRequest, FavoriteWriteError, FavoriteWriteErrorKind,
    };
    use crate::library::OperationPlanItem;
    use std::cell::RefCell;

    fn rename_plan() -> OperationPlan {
        OperationPlan {
            kind: OperationPlanKind::BiliFavoriteFolderRename,
            items: vec![OperationPlanItem {
                external_id: "300".to_string(),
                title: "Old folder".to_string(),
                action: "rename_folder".to_string(),
                target: Some("New folder".to_string()),
                status: "pending".to_string(),
                error: None,
                source_collection_external_id: Some("300".to_string()),
                source_collection_title: Some("Old folder".to_string()),
                target_collection_external_id: Some("300".to_string()),
                target_collection_title: Some("New folder".to_string()),
                resource_id: None,
                resource_type: None,
                metadata: serde_json::json!({
                    "intro": "Keep this intro",
                    "privacy": 1,
                    "cover": "https://i0.hdslb.com/cover.jpg"
                }),
            }],
        }
    }

    #[test]
    fn favorite_folder_rename_execution_requires_explicit_confirmation() {
        let calls = RefCell::new(Vec::<FavoriteFolderRenameRequest>::new());

        let err = futures::executor::block_on(execute_favorite_folder_rename_plan_with_runner(
            rename_plan(),
            false,
            |request| {
                calls.borrow_mut().push(request);
                async {
                    Ok(LibraryCollection::from_bili_favorite_folder(
                        "300".to_string(),
                        "New folder".to_string(),
                        serde_json::json!({"id": 300, "title": "New folder"}),
                    ))
                }
            },
        ))
        .expect_err("confirmation is required");

        assert!(err.to_string().contains("explicit confirmation"));
        assert!(calls.borrow().is_empty());
    }

    #[test]
    fn favorite_folder_rename_execution_marks_success_and_updates_title() {
        let calls = RefCell::new(Vec::<FavoriteFolderRenameRequest>::new());

        let result = futures::executor::block_on(execute_favorite_folder_rename_plan_with_runner(
            rename_plan(),
            true,
            |request| {
                calls.borrow_mut().push(request);
                async {
                    Ok(LibraryCollection::from_bili_favorite_folder(
                        "300".to_string(),
                        "New folder".to_string(),
                        serde_json::json!({"id": 300, "title": "New folder"}),
                    ))
                }
            },
        ))
        .expect("folder rename execution should succeed");

        let calls = calls.borrow();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].media_id, "300");
        assert_eq!(calls[0].title, "New folder");
        assert_eq!(calls[0].intro, "Keep this intro");
        assert_eq!(calls[0].privacy, 1);
        assert_eq!(
            calls[0].cover.as_deref(),
            Some("https://i0.hdslb.com/cover.jpg")
        );
        assert_eq!(result.plan.items[0].status, "success");
        assert_eq!(result.plan.items[0].title, "New folder");
        assert_eq!(
            result.plan.items[0].source_collection_title.as_deref(),
            Some("Old folder")
        );
        assert_eq!(
            result.plan.items[0].target_collection_title.as_deref(),
            Some("New folder")
        );
    }

    #[test]
    fn favorite_folder_rename_execution_marks_blocked_errors() {
        let result = futures::executor::block_on(execute_favorite_folder_rename_plan_with_runner(
            rename_plan(),
            true,
            |_request| async {
                Err(FavoriteWriteError {
                    kind: FavoriteWriteErrorKind::Blocked,
                    message: "Bilibili favorite write blocked (-111): csrf failed".to_string(),
                })
            },
        ))
        .expect("blocked item status should be returned");

        assert_eq!(result.plan.items[0].status, "blocked");
        assert!(result.plan.items[0]
            .error
            .as_deref()
            .unwrap_or_default()
            .contains("csrf failed"));
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
            metadata: serde_json::json!({}),
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
