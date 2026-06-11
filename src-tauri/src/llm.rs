use crate::error::{Result, YadigError};
use crate::library::{LibraryItem, LibraryItemType};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

pub const LLM_CLASSIFICATION_CHUNK_SIZE: usize = 8;
const SUPPORTED_CLASSIFICATION_ACTIONS: [&str; 4] = ["copy", "move", "delete", "none"];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LlmProviderTestErrorKind {
    MissingConfig,
    Auth,
    Network,
    IncompatibleResponse,
    InvalidJson,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LlmProviderTestError {
    pub kind: LlmProviderTestErrorKind,
    pub message: String,
}

impl LlmProviderTestError {
    fn new(kind: LlmProviderTestErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: redact_llm_error(&message.into()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LlmProviderTestResult {
    pub ok: bool,
    pub provider: String,
    pub model: String,
    pub used_response_format: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LlmAnalysisResponse {
    pub items: Vec<LlmItemAnalysis>,
    #[serde(default = "default_llm_analysis_source")]
    pub source: LlmAnalysisSource,
    #[serde(default)]
    pub warning: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LlmAnalysisSource {
    Llm,
    MetadataFallback,
}

fn default_llm_analysis_source() -> LlmAnalysisSource {
    LlmAnalysisSource::Llm
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LlmItemAnalysis {
    #[serde(alias = "external_id")]
    pub external_id: String,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    #[serde(alias = "suggested_tags")]
    pub suggested_tags: Vec<String>,
    pub reason: String,
    pub confidence: f32,
    #[serde(default)]
    #[serde(alias = "suggested_action")]
    pub suggested_action: Option<LlmSuggestedAction>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LlmSuggestedAction {
    pub kind: String,
    pub target: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LlmAnalyzeItemsRequest {
    pub instruction: String,
    pub items: Vec<LibraryItem>,
    pub provider: Option<LlmProviderConfig>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LlmProviderConfig {
    pub provider: String,
    pub base_url: Option<String>,
    pub api_key: Option<String>,
    pub model: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LlmClassificationProvenance {
    Llm,
    LocalMetadata,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LlmClassificationMode {
    Llm,
    LocalMetadata,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LlmClassifyItemsRequest {
    pub instruction: String,
    pub items: Vec<LibraryItem>,
    pub provider: Option<LlmProviderConfig>,
    pub mode: LlmClassificationMode,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LlmClassificationItem {
    #[serde(alias = "external_id")]
    pub external_id: String,
    pub category: String,
    #[serde(default)]
    #[serde(alias = "suggested_tags")]
    pub suggested_tags: Vec<String>,
    pub reason: String,
    pub confidence: f32,
    #[serde(default)]
    #[serde(alias = "suggested_action")]
    pub suggested_action: Option<LlmSuggestedAction>,
    pub provenance: LlmClassificationProvenance,
    pub provider: String,
    pub model: String,
    pub analysis_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LlmClassificationChunkFailure {
    pub chunk_index: usize,
    pub item_external_ids: Vec<String>,
    pub error: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LlmClassificationResponse {
    pub items: Vec<LlmClassificationItem>,
    pub chunk_failures: Vec<LlmClassificationChunkFailure>,
}

pub fn validate_llm_provider_config(
    provider: &LlmProviderConfig,
) -> std::result::Result<(), LlmProviderTestError> {
    if provider.provider.trim().is_empty() {
        return Err(LlmProviderTestError::new(
            LlmProviderTestErrorKind::MissingConfig,
            "LLM provider label is required",
        ));
    }
    if provider.model.trim().is_empty() {
        return Err(LlmProviderTestError::new(
            LlmProviderTestErrorKind::MissingConfig,
            "LLM model is required",
        ));
    }
    if provider
        .base_url
        .as_deref()
        .map(str::trim)
        .unwrap_or_default()
        .is_empty()
    {
        return Err(LlmProviderTestError::new(
            LlmProviderTestErrorKind::MissingConfig,
            "LLM base URL is required",
        ));
    }
    if provider
        .api_key
        .as_deref()
        .map(str::trim)
        .unwrap_or_default()
        .is_empty()
    {
        return Err(LlmProviderTestError::new(
            LlmProviderTestErrorKind::MissingConfig,
            "LLM API key is required",
        ));
    }

    Ok(())
}

pub fn parse_llm_analysis(raw: &str) -> Result<LlmAnalysisResponse> {
    serde_json::from_str(clean_json_response(raw))
        .map_err(|e| YadigError::Network(format!("LLM analysis parse error: {}", e)))
}

pub fn parse_llm_classification_response(
    raw: &str,
    requested_items: &[LibraryItem],
    provenance: LlmClassificationProvenance,
    provider: &str,
    model: &str,
    analysis_at: &str,
) -> Result<LlmClassificationResponse> {
    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct RawResponse {
        items: Vec<RawClassificationItem>,
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct RawClassificationItem {
        #[serde(alias = "external_id")]
        external_id: String,
        category: String,
        #[serde(default)]
        #[serde(alias = "suggested_tags")]
        suggested_tags: Vec<String>,
        reason: String,
        confidence: f32,
        #[serde(default)]
        #[serde(alias = "suggested_action")]
        suggested_action: Option<LlmSuggestedAction>,
    }

    let cleaned = clean_json_response(raw);
    let value: serde_json::Value = serde_json::from_str(cleaned)
        .map_err(|err| YadigError::Network(format!("LLM classification parse error: {err}")))?;
    if !value.get("items").is_some_and(|items| items.is_array()) {
        return Err(YadigError::Network(
            "LLM classification response items must be an array".into(),
        ));
    }
    let parsed: RawResponse = serde_json::from_value(value)
        .map_err(|err| YadigError::Network(format!("LLM classification parse error: {err}")))?;
    let requested_ids = requested_items
        .iter()
        .map(|item| item.external_id.as_str())
        .collect::<HashSet<_>>();
    let mut seen = HashSet::new();
    let mut items = Vec::new();

    for raw_item in parsed.items {
        let external_id = raw_item.external_id.trim().to_string();
        if external_id.is_empty() || !requested_ids.contains(external_id.as_str()) {
            continue;
        }
        if !seen.insert(external_id.clone()) {
            continue;
        }

        let category = raw_item.category.trim().to_string();
        if category.is_empty() {
            continue;
        }
        let reason = raw_item.reason.trim().to_string();
        if reason.is_empty() {
            continue;
        }

        items.push(LlmClassificationItem {
            external_id,
            category,
            suggested_tags: normalize_tags(raw_item.suggested_tags),
            reason,
            confidence: raw_item.confidence.clamp(0.0, 1.0),
            suggested_action: normalize_suggested_action(raw_item.suggested_action),
            provenance: provenance.clone(),
            provider: provider.to_string(),
            model: model.to_string(),
            analysis_at: analysis_at.to_string(),
        });
    }

    Ok(LlmClassificationResponse {
        items,
        chunk_failures: Vec::new(),
    })
}

fn clean_json_response(raw: &str) -> &str {
    let trimmed = raw.trim();
    if !trimmed.starts_with("```") {
        return trimmed;
    }

    let Some(first_newline) = trimmed.find('\n') else {
        return trimmed;
    };
    let body = &trimmed[first_newline + 1..];
    body.strip_suffix("```").unwrap_or(body).trim()
}

fn normalize_tags(tags: Vec<String>) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut normalized = Vec::new();
    for tag in tags {
        let trimmed = tag.trim();
        if trimmed.is_empty() || !seen.insert(trimmed.to_string()) {
            continue;
        }
        normalized.push(trimmed.to_string());
    }
    normalized
}

fn normalize_suggested_action(action: Option<LlmSuggestedAction>) -> Option<LlmSuggestedAction> {
    let action = action?;
    let kind = action.kind.trim().to_ascii_lowercase();
    if kind.is_empty()
        || kind == "none"
        || !SUPPORTED_CLASSIFICATION_ACTIONS.contains(&kind.as_str())
    {
        return None;
    }
    let target = action.target.and_then(|value| {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    });
    Some(LlmSuggestedAction { kind, target })
}

fn classification_metadata(item: &LibraryItem) -> serde_json::Value {
    let mut metadata = serde_json::Map::new();
    for key in ["tid", "tname", "duration", "pubdate", "favTime"] {
        if let Some(value) = item.raw_metadata.get(key) {
            if value.is_string() || value.is_number() || value.is_boolean() {
                metadata.insert(key.to_string(), value.clone());
            }
        }
    }

    serde_json::json!({
        "external_id": item.external_id,
        "item_type": item.item_type,
        "title": item.title,
        "author": item.author,
        "metadata": metadata,
    })
}

pub fn build_metadata_analysis_prompt(request: &LlmAnalyzeItemsRequest) -> String {
    let compact_items = request
        .items
        .iter()
        .map(classification_metadata)
        .collect::<Vec<_>>();

    format!(
        "你是个人媒体资源整理助手。只根据元数据给出分类、理由、置信度和可选操作建议，不要直接执行远端操作。\n用户任务：{}\n条目：{}",
        request.instruction,
        serde_json::to_string(&compact_items).unwrap_or_else(|_| "[]".to_string())
    )
}

pub fn build_classification_prompt(request: &LlmClassifyItemsRequest) -> String {
    let compact_items = request
        .items
        .iter()
        .filter(|item| {
            item.source == "bilibili" && item.item_type == LibraryItemType::BiliFavoriteVideo
        })
        .map(classification_metadata)
        .collect::<Vec<_>>();

    format!(
        "你是个人 Bilibili 收藏整理助手。只根据提供的条目元数据进行分类，不要执行远端操作。\n用户任务：{}\n条目：{}",
        request.instruction,
        serde_json::to_string(&compact_items).unwrap_or_else(|_| "[]".to_string())
    )
}

pub async fn analyze_items(request: LlmAnalyzeItemsRequest) -> Result<LlmAnalysisResponse> {
    if request.items.is_empty() {
        return Ok(LlmAnalysisResponse {
            items: Vec::new(),
            source: LlmAnalysisSource::MetadataFallback,
            warning: None,
        });
    }

    let Some(provider) = request.provider.clone() else {
        return Ok(metadata_fallback_analysis(
            &request,
            Some("LLM provider is not configured; used local metadata fallback.".to_string()),
        ));
    };

    if provider.api_key.as_deref().unwrap_or("").trim().is_empty() {
        return Ok(metadata_fallback_analysis(
            &request,
            Some("LLM API key is not configured; used local metadata fallback.".to_string()),
        ));
    }

    match call_openai_compatible(&provider, &request).await {
        Ok(mut response) => {
            response.source = LlmAnalysisSource::Llm;
            response.warning = None;
            Ok(response)
        }
        Err(err) => Ok(metadata_fallback_analysis(
            &request,
            Some(format!(
                "LLM request failed; used local metadata fallback: {}",
                err
            )),
        )),
    }
}

pub fn build_openai_compatible_payload(
    model: &str,
    request: &LlmAnalyzeItemsRequest,
) -> serde_json::Value {
    serde_json::json!({
        "model": model,
        "temperature": 0.2,
        "response_format": { "type": "json_object" },
        "messages": [
            {
                "role": "system",
                "content": "你是个人媒体资源整理助手。必须输出 JSON 对象，格式为 {\"items\":[{\"external_id\":\"...\",\"suggested_tags\":[\"...\"],\"reason\":\"...\",\"confidence\":0.0,\"suggested_action\":{\"kind\":\"extract_audio\",\"target\":\"music-audio\"}}]}。suggested_action 可为 null。不要执行远端操作。"
            },
            {
                "role": "user",
                "content": build_metadata_analysis_prompt(request)
            }
        ]
    })
}

pub fn build_classification_payload(
    model: &str,
    request: &LlmClassifyItemsRequest,
) -> serde_json::Value {
    serde_json::json!({
        "model": model,
        "temperature": 0.2,
        "response_format": { "type": "json_object" },
        "messages": [
            {
                "role": "system",
                "content": "你是个人媒体资源整理助手。必须输出 JSON 对象，格式为 {\"items\":[{\"external_id\":\"...\",\"category\":\"...\",\"suggested_tags\":[\"...\"],\"reason\":\"...\",\"confidence\":0.0,\"suggested_action\":{\"kind\":\"copy|move|delete|none\",\"target\":\"目标收藏夹名或 null\"}}]}。suggested_action 可为 null。不要执行远端操作。"
            },
            {
                "role": "user",
                "content": build_classification_prompt(request)
            }
        ]
    })
}

pub fn chunk_classification_items(
    items: &[LibraryItem],
    chunk_size: usize,
) -> Vec<Vec<LibraryItem>> {
    let effective_chunk_size = chunk_size.max(1);
    items
        .chunks(effective_chunk_size)
        .map(|chunk| chunk.to_vec())
        .collect()
}

pub fn classify_items_with_local_metadata(
    request: &LlmClassifyItemsRequest,
    analysis_at: &str,
) -> LlmClassificationResponse {
    let items = request
        .items
        .iter()
        .filter(|item| {
            item.source == "bilibili" && item.item_type == LibraryItemType::BiliFavoriteVideo
        })
        .map(|item| {
            let tname = item
                .raw_metadata
                .get("tname")
                .and_then(|value| value.as_str())
                .unwrap_or("");
            let title = item.title.to_lowercase();
            let is_music = tname.contains("音乐")
                || title.contains("music")
                || title.contains("song")
                || item.title.contains("音乐")
                || item.title.contains("歌");
            LlmClassificationItem {
                external_id: item.external_id.clone(),
                category: if is_music { "music" } else { "uncategorized" }.to_string(),
                suggested_tags: if is_music {
                    vec!["音乐".to_string()]
                } else {
                    vec!["待分类".to_string()]
                },
                reason: if is_music {
                    "元数据中的分区或标题显示这是音乐相关内容".to_string()
                } else {
                    "当前元数据不足以稳定判断具体领域".to_string()
                },
                confidence: if is_music { 0.72 } else { 0.35 },
                suggested_action: if is_music {
                    Some(LlmSuggestedAction {
                        kind: "copy".to_string(),
                        target: Some("music-audio".to_string()),
                    })
                } else {
                    None
                },
                provenance: LlmClassificationProvenance::LocalMetadata,
                provider: "local-metadata".to_string(),
                model: "local-metadata".to_string(),
                analysis_at: analysis_at.to_string(),
            }
        })
        .collect();

    LlmClassificationResponse {
        items,
        chunk_failures: Vec::new(),
    }
}

pub async fn classify_items(request: LlmClassifyItemsRequest) -> Result<LlmClassificationResponse> {
    if request.items.is_empty() {
        return Ok(LlmClassificationResponse {
            items: Vec::new(),
            chunk_failures: Vec::new(),
        });
    }

    let analysis_at = Utc::now().to_rfc3339();
    if request.mode == LlmClassificationMode::LocalMetadata {
        return Ok(classify_items_with_local_metadata(&request, &analysis_at));
    }

    let provider = request.provider.clone().ok_or_else(|| {
        YadigError::Network("LLM provider is required for explicit classification".into())
    })?;
    validate_llm_provider_config(&provider).map_err(|err| YadigError::Network(err.message))?;

    let mut items = Vec::new();
    let mut chunk_failures = Vec::new();
    let chunks = chunk_classification_items(&request.items, LLM_CLASSIFICATION_CHUNK_SIZE);
    for (chunk_index, chunk_items) in chunks.into_iter().enumerate() {
        let chunk_request = LlmClassifyItemsRequest {
            instruction: request.instruction.clone(),
            items: chunk_items.clone(),
            provider: Some(provider.clone()),
            mode: LlmClassificationMode::Llm,
        };
        match call_openai_compatible_classification(&provider, &chunk_request, &analysis_at).await {
            Ok(mut response) => items.append(&mut response.items),
            Err(err) => chunk_failures.push(LlmClassificationChunkFailure {
                chunk_index,
                item_external_ids: chunk_items
                    .iter()
                    .map(|item| item.external_id.clone())
                    .collect(),
                error: redact_llm_error(&err.to_string()),
            }),
        }
    }

    Ok(LlmClassificationResponse {
        items,
        chunk_failures,
    })
}

pub fn build_llm_provider_test_payload(
    model: &str,
    include_response_format: bool,
) -> serde_json::Value {
    let mut payload = serde_json::json!({
        "model": model,
        "temperature": 0.0,
        "messages": [
            {
                "role": "system",
                "content": "Return only a compact JSON object. No markdown."
            },
            {
                "role": "user",
                "content": "Return JSON exactly matching {\"ok\":true,\"provider\":\"test\"}."
            }
        ]
    });

    if include_response_format {
        payload["response_format"] = serde_json::json!({ "type": "json_object" });
    }

    payload
}

pub fn parse_llm_provider_test_response(
    body: &serde_json::Value,
) -> std::result::Result<serde_json::Value, LlmProviderTestError> {
    let content = body["choices"][0]["message"]["content"]
        .as_str()
        .ok_or_else(|| {
            LlmProviderTestError::new(
                LlmProviderTestErrorKind::IncompatibleResponse,
                "LLM test response missing message content",
            )
        })?;

    let parsed: serde_json::Value =
        serde_json::from_str(clean_json_response(content)).map_err(|err| {
            LlmProviderTestError::new(
                LlmProviderTestErrorKind::InvalidJson,
                format!("LLM test response was not valid JSON: {err}"),
            )
        })?;

    if !parsed.is_object() {
        return Err(LlmProviderTestError::new(
            LlmProviderTestErrorKind::InvalidJson,
            "LLM test response JSON must be an object",
        ));
    }

    Ok(parsed)
}

pub fn is_response_format_compatibility_error(status: u16, body: &str) -> bool {
    if !matches!(status, 400 | 404 | 422) {
        return false;
    }
    let lower = body.to_ascii_lowercase();
    lower.contains("response_format") || lower.contains("json_object")
}

pub fn llm_provider_http_error(status: u16, body: &str) -> LlmProviderTestError {
    if is_response_format_compatibility_error(status, body) {
        return LlmProviderTestError::new(
            LlmProviderTestErrorKind::IncompatibleResponse,
            format!("response_format is not compatible with this provider: {body}"),
        );
    }
    let kind = if matches!(status, 401 | 403) {
        LlmProviderTestErrorKind::Auth
    } else {
        LlmProviderTestErrorKind::Network
    };
    LlmProviderTestError::new(
        kind,
        format!("LLM provider test failed with status {status}: {body}"),
    )
}

pub fn redact_llm_error(message: &str) -> String {
    let mut redacted = message.to_string();
    let patterns = [
        (r"(?i)Bearer\s+[A-Za-z0-9._~+/=-]+", "Bearer <redacted>"),
        (r"(?i)(api[_-]?key=)[^&\s]+", "$1<redacted>"),
        (r"(?i)(token=)[^&\s]+", "$1<redacted>"),
        (r"\btp-[A-Za-z0-9_-]+", "<redacted>"),
        (r"\bsk-[A-Za-z0-9_-]+", "<redacted>"),
    ];

    for (pattern, replacement) in patterns {
        if let Ok(re) = regex::Regex::new(pattern) {
            redacted = re.replace_all(&redacted, replacement).to_string();
        }
    }

    redacted
}

pub async fn test_llm_provider(
    provider: LlmProviderConfig,
) -> std::result::Result<LlmProviderTestResult, LlmProviderTestError> {
    validate_llm_provider_config(&provider)?;
    let result = call_llm_provider_test(&provider, true).await;
    match result {
        Ok(()) => Ok(LlmProviderTestResult {
            ok: true,
            provider: provider.provider,
            model: provider.model,
            used_response_format: true,
        }),
        Err(err)
            if err.kind == LlmProviderTestErrorKind::IncompatibleResponse
                && err.message.contains("response_format") =>
        {
            call_llm_provider_test(&provider, false).await?;
            Ok(LlmProviderTestResult {
                ok: true,
                provider: provider.provider,
                model: provider.model,
                used_response_format: false,
            })
        }
        Err(err) => Err(err),
    }
}

async fn call_llm_provider_test(
    provider: &LlmProviderConfig,
    include_response_format: bool,
) -> std::result::Result<(), LlmProviderTestError> {
    let api_key = provider.api_key.as_deref().unwrap_or("").trim();
    let base_url = provider
        .base_url
        .as_deref()
        .unwrap_or("https://api.openai.com/v1")
        .trim_end_matches('/');
    let url = format!("{}/chat/completions", base_url);
    let payload = build_llm_provider_test_payload(&provider.model, include_response_format);
    let client = crate::http_client::build_client("yadig/0.1.0");
    let response = client
        .post(&url)
        .bearer_auth(api_key)
        .json(&payload)
        .send()
        .await
        .map_err(|err| {
            LlmProviderTestError::new(
                LlmProviderTestErrorKind::Network,
                format!("LLM provider test request failed: {err}"),
            )
        })?;

    let status = response.status();
    let status_code = status.as_u16();
    let body_text = response.text().await.map_err(|err| {
        LlmProviderTestError::new(
            LlmProviderTestErrorKind::Network,
            format!("LLM provider test response read failed: {err}"),
        )
    })?;

    if !status.is_success() {
        return Err(llm_provider_http_error(status_code, &body_text));
    }

    let body: serde_json::Value = serde_json::from_str(&body_text).map_err(|err| {
        LlmProviderTestError::new(
            LlmProviderTestErrorKind::IncompatibleResponse,
            format!("LLM provider test HTTP response was not JSON: {err}"),
        )
    })?;
    parse_llm_provider_test_response(&body)?;
    Ok(())
}

async fn call_openai_compatible(
    provider: &LlmProviderConfig,
    request: &LlmAnalyzeItemsRequest,
) -> Result<LlmAnalysisResponse> {
    let api_key = provider.api_key.as_deref().unwrap_or("").trim();
    if api_key.is_empty() {
        return Ok(metadata_fallback_analysis(
            request,
            Some("LLM API key is not configured; used local metadata fallback.".to_string()),
        ));
    }

    let base_url = provider
        .base_url
        .as_deref()
        .filter(|url| !url.trim().is_empty())
        .unwrap_or("https://api.openai.com/v1")
        .trim_end_matches('/');
    let url = format!("{}/chat/completions", base_url);
    let payload = build_openai_compatible_payload(&provider.model, request);
    let client = crate::http_client::build_client("yadig/0.1.0");
    let response = client
        .post(&url)
        .bearer_auth(api_key)
        .json(&payload)
        .send()
        .await
        .map_err(|e| YadigError::Network(format!("LLM request failed: {}", e)))?;

    if !response.status().is_success() {
        return Err(YadigError::Network(format!(
            "LLM request failed with status {}",
            response.status()
        )));
    }

    let body: serde_json::Value = response
        .json()
        .await
        .map_err(|e| YadigError::Network(format!("LLM response parse error: {}", e)))?;
    let content = body["choices"][0]["message"]["content"]
        .as_str()
        .ok_or_else(|| YadigError::Network("LLM response missing message content".into()))?;

    parse_llm_analysis(content)
}

async fn call_openai_compatible_classification(
    provider: &LlmProviderConfig,
    request: &LlmClassifyItemsRequest,
    analysis_at: &str,
) -> Result<LlmClassificationResponse> {
    let api_key = provider.api_key.as_deref().unwrap_or("").trim();
    if api_key.is_empty() {
        return Err(YadigError::Network("LLM API key is required".into()));
    }

    let base_url = provider
        .base_url
        .as_deref()
        .filter(|url| !url.trim().is_empty())
        .unwrap_or("https://api.openai.com/v1")
        .trim_end_matches('/');
    let url = format!("{}/chat/completions", base_url);
    let payload = build_classification_payload(&provider.model, request);
    let client = crate::http_client::build_client("yadig/0.1.0");
    let response = client
        .post(&url)
        .bearer_auth(api_key)
        .json(&payload)
        .send()
        .await
        .map_err(|err| YadigError::Network(format!("LLM classification request failed: {err}")))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(YadigError::Network(redact_llm_error(&format!(
            "LLM classification request failed with status {status}: {body}"
        ))));
    }

    let body: serde_json::Value = response.json().await.map_err(|err| {
        YadigError::Network(format!("LLM classification response parse error: {err}"))
    })?;
    let content = body["choices"][0]["message"]["content"]
        .as_str()
        .ok_or_else(|| {
            YadigError::Network("LLM classification response missing message content".into())
        })?;

    parse_llm_classification_response(
        content,
        &request.items,
        LlmClassificationProvenance::Llm,
        &provider.provider,
        &provider.model,
        analysis_at,
    )
}

fn metadata_fallback_analysis(
    request: &LlmAnalyzeItemsRequest,
    warning: Option<String>,
) -> LlmAnalysisResponse {
    let items = request
        .items
        .iter()
        .map(|item| {
            let tname = item
                .raw_metadata
                .get("tname")
                .and_then(|value| value.as_str())
                .unwrap_or("");
            let title = item.title.to_lowercase();
            let is_music = tname.contains("音乐")
                || title.contains("music")
                || title.contains("song")
                || item.title.contains("音乐")
                || item.title.contains("歌");
            LlmItemAnalysis {
                external_id: item.external_id.clone(),
                category: Some(if is_music { "music" } else { "uncategorized" }.to_string()),
                suggested_tags: if is_music {
                    vec!["音乐".to_string()]
                } else {
                    vec!["待分类".to_string()]
                },
                reason: if is_music {
                    "元数据中的分区或标题显示这是音乐相关内容".to_string()
                } else {
                    "当前元数据不足以稳定判断具体领域".to_string()
                },
                confidence: if is_music { 0.72 } else { 0.35 },
                suggested_action: if is_music {
                    Some(LlmSuggestedAction {
                        kind: "extract_audio".to_string(),
                        target: Some("music-audio".to_string()),
                    })
                } else {
                    None
                },
            }
        })
        .collect();

    LlmAnalysisResponse {
        items,
        source: LlmAnalysisSource::MetadataFallback,
        warning,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::library::{BiliResourceKind, LibraryItem};

    #[test]
    fn fallback_marks_music_metadata_for_audio_extraction() {
        let request = LlmAnalyzeItemsRequest {
            instruction: "找出音乐类视频".to_string(),
            provider: None,
            items: vec![LibraryItem::from_bili_video(
                BiliResourceKind::WatchLaterVideo,
                "BV1song".to_string(),
                "一首歌".to_string(),
                None,
                serde_json::json!({ "tname": "音乐" }),
            )],
        };

        let result = metadata_fallback_analysis(&request, None);
        assert_eq!(result.items[0].suggested_tags, vec!["音乐"]);
        assert_eq!(
            result.items[0]
                .suggested_action
                .as_ref()
                .map(|a| a.kind.as_str()),
            Some("extract_audio")
        );
    }

    #[test]
    fn extracts_json_from_fenced_llm_response() {
        let raw = "```json\n{\"items\":[]}\n```";
        let parsed = parse_llm_analysis(raw).unwrap();
        assert!(parsed.items.is_empty());
    }

    #[test]
    fn builds_openai_compatible_payload_with_json_instruction() {
        let request = LlmAnalyzeItemsRequest {
            instruction: "分类".to_string(),
            provider: None,
            items: vec![LibraryItem::from_bili_video(
                BiliResourceKind::FavoriteVideo,
                "BV1abc".to_string(),
                "音乐视频".to_string(),
                None,
                serde_json::json!({ "tname": "音乐" }),
            )],
        };

        let payload = build_openai_compatible_payload("gpt-test", &request);
        assert_eq!(payload["model"], "gpt-test");
        assert!(payload["messages"][0]["content"]
            .as_str()
            .unwrap()
            .contains("items"));
        assert!(payload["response_format"]["type"]
            .as_str()
            .unwrap()
            .contains("json_object"));
    }

    #[test]
    fn metadata_fallback_reports_source_and_warning() {
        let request = LlmAnalyzeItemsRequest {
            instruction: "分类".to_string(),
            provider: None,
            items: vec![LibraryItem::from_bili_video(
                BiliResourceKind::FavoriteVideo,
                "BV1abc".to_string(),
                "音乐视频".to_string(),
                None,
                serde_json::json!({ "tname": "音乐" }),
            )],
        };

        let result = metadata_fallback_analysis(
            &request,
            Some("LLM provider is not configured; used local metadata fallback.".to_string()),
        );

        assert_eq!(result.source, LlmAnalysisSource::MetadataFallback);
        assert_eq!(
            result.warning.as_deref(),
            Some("LLM provider is not configured; used local metadata fallback.")
        );
    }

    #[test]
    fn validates_structured_classification_results() {
        let items = vec![LibraryItem::from_bili_video(
            BiliResourceKind::FavoriteVideo,
            "BV1known".to_string(),
            "爵士现场".to_string(),
            Some("音乐UP".to_string()),
            serde_json::json!({ "tname": "音乐" }),
        )];
        let raw = r#"{
          "items": [
            {
              "external_id": "BV1known",
              "category": "music",
              "suggested_tags": ["爵士", "现场"],
              "reason": "标题和分区都指向音乐内容",
              "confidence": 1.3,
              "suggested_action": {
                "kind": "copy",
                "target": "Jazz"
              }
            },
            {
              "external_id": "BV1unknown",
              "category": "noise",
              "suggested_tags": ["忽略"],
              "reason": "不在请求中",
              "confidence": 0.8,
              "suggested_action": {
                "kind": "archive",
                "target": ""
              }
            }
          ]
        }"#;

        let response = parse_llm_classification_response(
            raw,
            &items,
            LlmClassificationProvenance::Llm,
            "openai-compatible",
            "mimo-v2.5-pro",
            "2026-06-11T00:00:00Z",
        )
        .expect("valid known item should be normalized");

        assert_eq!(response.items.len(), 1);
        let classification = &response.items[0];
        assert_eq!(classification.external_id, "BV1known");
        assert_eq!(classification.category, "music");
        assert_eq!(classification.suggested_tags, vec!["爵士", "现场"]);
        assert_eq!(classification.confidence, 1.0);
        assert_eq!(
            classification.suggested_action,
            Some(LlmSuggestedAction {
                kind: "copy".to_string(),
                target: Some("Jazz".to_string()),
            })
        );
        assert_eq!(classification.provenance, LlmClassificationProvenance::Llm);
        assert_eq!(classification.provider, "openai-compatible");
        assert_eq!(classification.model, "mimo-v2.5-pro");
        assert_eq!(classification.analysis_at, "2026-06-11T00:00:00Z");
    }

    #[test]
    fn rejects_malformed_classification_json() {
        let items = vec![LibraryItem::from_bili_video(
            BiliResourceKind::FavoriteVideo,
            "BV1known".to_string(),
            "爵士现场".to_string(),
            None,
            serde_json::json!({}),
        )];

        let err = parse_llm_classification_response(
            "{\"items\":\"not-an-array\"}",
            &items,
            LlmClassificationProvenance::Llm,
            "openai-compatible",
            "mimo-v2.5-pro",
            "2026-06-11T00:00:00Z",
        )
        .expect_err("malformed items should fail");

        assert!(err.to_string().contains("items"));
    }

    #[test]
    fn chunks_classification_requests_conservatively() {
        let items = (0..17)
            .map(|index| {
                LibraryItem::from_bili_video(
                    BiliResourceKind::FavoriteVideo,
                    format!("BV{index:02}"),
                    format!("视频 {index}"),
                    None,
                    serde_json::json!({ "tname": "音乐" }),
                )
            })
            .collect::<Vec<_>>();

        let chunks = chunk_classification_items(&items, 8);

        assert_eq!(chunks.len(), 3);
        assert_eq!(chunks[0].len(), 8);
        assert_eq!(chunks[1].len(), 8);
        assert_eq!(chunks[2].len(), 1);
    }

    #[test]
    fn local_metadata_classification_uses_explicit_local_provenance() {
        let request = LlmClassifyItemsRequest {
            instruction: "分类".to_string(),
            items: vec![LibraryItem::from_bili_video(
                BiliResourceKind::FavoriteVideo,
                "BV1music".to_string(),
                "一首歌".to_string(),
                None,
                serde_json::json!({ "tname": "音乐" }),
            )],
            provider: None,
            mode: LlmClassificationMode::LocalMetadata,
        };

        let response = classify_items_with_local_metadata(&request, "2026-06-11T00:00:00Z");

        assert_eq!(response.items.len(), 1);
        assert_eq!(
            response.items[0].provenance,
            LlmClassificationProvenance::LocalMetadata
        );
        assert_eq!(response.items[0].provider, "local-metadata");
        assert_eq!(response.items[0].model, "local-metadata");
        assert!(response.chunk_failures.is_empty());
    }

    #[test]
    fn validates_provider_config_for_explicit_test() {
        let missing_key = LlmProviderConfig {
            provider: "openai-compatible".to_string(),
            base_url: Some("https://example.com/v1".to_string()),
            api_key: Some("   ".to_string()),
            model: "mimo-v2.5-pro".to_string(),
        };

        let err = validate_llm_provider_config(&missing_key).expect_err("api key is required");

        assert_eq!(err.kind, LlmProviderTestErrorKind::MissingConfig);
        assert_eq!(err.message, "LLM API key is required");
    }

    #[test]
    fn builds_provider_test_payload_with_and_without_response_format() {
        let with_response_format = build_llm_provider_test_payload("mimo-v2.5-pro", true);
        assert_eq!(with_response_format["model"], "mimo-v2.5-pro");
        assert_eq!(
            with_response_format["response_format"]["type"].as_str(),
            Some("json_object")
        );

        let prompt_only = build_llm_provider_test_payload("mimo-v2.5-pro", false);
        assert!(prompt_only.get("response_format").is_none());
        assert!(prompt_only["messages"][0]["content"]
            .as_str()
            .unwrap()
            .contains("JSON"));
    }

    #[test]
    fn parses_provider_test_success_response_content() {
        let body = serde_json::json!({
            "choices": [{
                "message": {
                    "content": "```json\n{\"ok\":true,\"provider\":\"mimo\"}\n```"
                }
            }]
        });

        let parsed = parse_llm_provider_test_response(&body).expect("valid response parses");

        assert_eq!(parsed["ok"].as_bool(), Some(true));
        assert_eq!(parsed["provider"].as_str(), Some("mimo"));
    }

    #[test]
    fn classifies_response_format_incompatibility() {
        let body = "{\"error\":{\"message\":\"response_format is not supported\"}}";
        let model_body = "{\"error\":{\"message\":\"model is unsupported\"}}";

        assert!(is_response_format_compatibility_error(400, body));
        assert!(!is_response_format_compatibility_error(401, body));
        assert!(!is_response_format_compatibility_error(400, model_body));
    }

    #[test]
    fn classifies_provider_test_http_errors() {
        let auth = llm_provider_http_error(401, "{\"error\":\"bad Bearer sk-test-secret\"}");
        assert_eq!(auth.kind, LlmProviderTestErrorKind::Auth);
        assert!(!auth.message.contains("sk-test-secret"));

        let incompatible =
            llm_provider_http_error(400, "{\"error\":\"response_format is not supported\"}");
        assert_eq!(
            incompatible.kind,
            LlmProviderTestErrorKind::IncompatibleResponse
        );

        let network = llm_provider_http_error(429, "{\"error\":\"rate limited\"}");
        assert_eq!(network.kind, LlmProviderTestErrorKind::Network);
    }

    #[test]
    fn redacts_llm_api_secrets_from_errors() {
        let raw = "Authorization: Bearer sk-test-secret api_key=tp-secret-token token-plan-cn.xiaomimimo.com/v1?api_key=abc";

        let redacted = redact_llm_error(raw);

        assert!(!redacted.contains("sk-test-secret"));
        assert!(!redacted.contains("tp-secret-token"));
        assert!(!redacted.contains("api_key=abc"));
        assert!(redacted.contains("Bearer <redacted>"));
        assert!(redacted.contains("api_key=<redacted>"));
    }
}
