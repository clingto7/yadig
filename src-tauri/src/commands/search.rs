use crate::config::DiscogsKeys;
use crate::error::{Result, YadigError};
use crate::source::registry::SourceRegistry;
use crate::source::types::*;
use tauri::State;

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
pub async fn list_sources(registry: State<'_, SourceRegistry>) -> Result<Vec<Source>> {
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
    *keys.secret.write().unwrap() = if secret.is_empty() {
        None
    } else {
        Some(secret)
    };
    Ok(())
}

/// Download an audio file to the user's Downloads folder
#[tauri::command]
pub async fn download_audio(url: String, filename: String) -> Result<String> {
    let downloads = dirs_next::download_dir()
        .ok_or_else(|| YadigError::Network("Could not find Downloads folder".into()))?;

    // Sanitize and truncate filename to avoid ENAMETOOLONG
    let safe_name: String = filename
        .chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            c => c,
        })
        .collect();
    // Truncate to 200 bytes to leave room for the full path prefix (EXT4: 255 per component)
    let truncated = if safe_name.len() > 200 {
        let mut s = String::new();
        for c in safe_name.chars() {
            if s.len() + c.len_utf8() > 200 {
                break;
            }
            s.push(c);
        }
        s
    } else {
        safe_name
    };

    let filepath = downloads.join(&truncated);
    eprintln!(
        "[download_audio] writing to: {:?} (filename {} bytes)",
        filepath,
        truncated.len()
    );

    let client = reqwest::Client::new();
    let resp = client
        .get(&url)
        .send()
        .await
        .map_err(|e| YadigError::Network(format!("Download failed: {}", e)))?;

    if !resp.status().is_success() {
        return Err(YadigError::Network(format!(
            "Download HTTP error: {}",
            resp.status()
        )));
    }

    let bytes = resp
        .bytes()
        .await
        .map_err(|e| YadigError::Network(format!("Download read error: {}", e)))?;

    std::fs::write(&filepath, &bytes)
        .map_err(|e| YadigError::Network(format!("File write error: {}", e)))?;

    Ok(filepath.to_string_lossy().to_string())
}

/// Open a URL in the system default browser
#[tauri::command]
pub async fn open_url(url: String) -> Result<()> {
    open::that(&url).map_err(|e| YadigError::Network(format!("Failed to open URL: {}", e)))?;
    Ok(())
}

/// Open a local file or folder with the system default application.
#[tauri::command]
pub async fn open_path(path: String) -> Result<()> {
    open::that(&path).map_err(|e| YadigError::Network(format!("Failed to open path: {}", e)))?;
    Ok(())
}
