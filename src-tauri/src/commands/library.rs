use crate::bili::auth::BiliAuth;
use crate::bili::client::BiliClient;
use crate::error::{Result, YadigError};
use crate::library::{
    AudioExtractionCandidate, BiliSyncResult, BiliSyncScope, OperationPlan, OperationPlanKind,
};
use crate::llm::{analyze_items, LlmAnalysisResponse, LlmAnalyzeItemsRequest};
use tauri::State;

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
pub async fn create_bili_audio_extraction_plan(
    candidates: Vec<AudioExtractionCandidate>,
) -> Result<OperationPlan> {
    Ok(OperationPlan::for_bili_audio_extraction(candidates))
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
