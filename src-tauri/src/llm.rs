use crate::error::{Result, YadigError};
use crate::library::LibraryItem;
use serde::{Deserialize, Serialize};

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
            Some(format!("LLM request failed; used local metadata fallback: {}", err)),
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
}
