use serde::{Deserialize, Serialize};

/// A single extracted audio segment from a YouTube video.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YoutubeAudioSegment {
    pub title: String,
    pub file_path: String,
    pub duration: f64,
    pub audio_url: String,
    pub ext: String,
}

/// Result of a YouTube audio extraction operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YoutubeExtractionResult {
    pub video_title: String,
    pub video_url: String,
    pub thumbnail_url: Option<String>,
    pub duration: f64,
    pub segments: Vec<YoutubeAudioSegment>,
    pub has_chapters: bool,
}
