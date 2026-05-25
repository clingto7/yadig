use tauri::State;
use crate::source::registry::SourceRegistry;
use crate::source::types::*;
use crate::error::Result;

#[tauri::command]
pub async fn search_sources(
    registry: State<'_, SourceRegistry>,
    query: String,
    source_ids: Option<Vec<String>>,
    limit: Option<usize>,
) -> Result<SearchResult> {
    let source_ids = source_ids.unwrap_or_default();
    let limit = limit.unwrap_or(20);
    registry.search(&query, &source_ids, limit).await
}

#[tauri::command]
pub async fn fetch_latest(
    registry: State<'_, SourceRegistry>,
    source_ids: Option<Vec<String>>,
    limit: Option<usize>,
) -> Result<Vec<ContentItem>> {
    let source_ids = source_ids.unwrap_or_default();
    let limit = limit.unwrap_or(20);
    registry.fetch_latest(&source_ids, limit).await
}

#[tauri::command]
pub async fn list_sources(
    registry: State<'_, SourceRegistry>,
) -> Result<Vec<Source>> {
    Ok(registry.list_sources())
}
