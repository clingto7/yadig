use tauri::State;
use crate::config::DiscogsKeys;
use crate::source::registry::SourceRegistry;
use crate::source::types::*;
use crate::error::Result;

#[tauri::command]
pub async fn search_sources(
    registry: State<'_, SourceRegistry>,
    query: String,
    source_ids: Option<Vec<String>>,
    limit: Option<usize>,
    page: Option<usize>,
) -> Result<SearchResult> {
    let source_ids = source_ids.unwrap_or_default();
    let limit = limit.unwrap_or(20);
    let page = page.unwrap_or(1);
    registry.search(&query, &source_ids, limit, page).await
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

#[tauri::command]
pub async fn set_source_enabled(
    registry: State<'_, SourceRegistry>,
    id: String,
    enabled: bool,
) -> Result<()> {
    registry.set_enabled(&id, enabled);
    Ok(())
}

#[tauri::command]
pub async fn update_discogs_keys(
    keys: State<'_, DiscogsKeys>,
    key: String,
    secret: String,
) -> Result<()> {
    *keys.key.write().unwrap() = if key.is_empty() { None } else { Some(key) };
    *keys.secret.write().unwrap() = if secret.is_empty() { None } else { Some(secret) };
    Ok(())
}
