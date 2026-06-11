use crate::error::{Result, YadigError};
use crate::library::LibraryItem;
use serde::{Deserialize, Serialize};

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

pub fn build_metadata_analysis_prompt(request: &LlmAnalyzeItemsRequest) -> String {
    let compact_items = request
        .items
        .iter()
        .map(|item| {
            serde_json::json!({
                "external_id": item.external_id,
                "type": item.item_type,
                "title": item.title,
                "author": item.author,
                "metadata": item.raw_metadata,
            })
        })
        .collect::<Vec<_>>();

    format!(
        "你是个人媒体资源整理助手。只根据元数据给出分类、理由、置信度和可选操作建议，不要直接执行远端操作。\n用户任务：{}\n条目：{}",
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
