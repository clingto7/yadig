use tauri::State;
use crate::error::{Result, YadigError};
use crate::youtube::types::*;
use crate::youtube::YoutubeClient;

/// Extract audio from a YouTube URL.
/// Downloads the best audio stream to Downloads/yadig/.
#[tauri::command]
pub async fn youtube_extract_audio(
    client: State<'_, YoutubeClient>,
    url: String,
) -> Result<YoutubeExtractionResult> {
    if url.trim().is_empty() {
        return Err(YadigError::NotFound("URL cannot be empty".into()));
    }

    // Basic URL validation
    if !url.contains("youtube.com/watch") && !url.contains("youtu.be/") {
        return Err(YadigError::NotFound(
            "Not a valid YouTube URL. Expected format: https://www.youtube.com/watch?v=...".into(),
        ));
    }

    let result = client.extract_audio(&url).await?;
    Ok(result)
}

/// Search YouTube for videos matching the query.
#[tauri::command]
pub async fn youtube_search(
    client: State<'_, YoutubeClient>,
    query: String,
    limit: Option<usize>,
) -> Result<Vec<crate::source::types::ContentItem>> {
    let limit = limit.unwrap_or(10).min(50);
    let items = client.search(&query, limit).await?;
    Ok(items)
}

/// Check if yt-dlp is installed and ready.
#[tauri::command]
pub async fn youtube_check_ready(client: State<'_, YoutubeClient>) -> Result<bool> {
    match client.ensure_initialized() {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}
