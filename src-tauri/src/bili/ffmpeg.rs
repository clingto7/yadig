use std::path::Path;
use crate::error::{Result, YadigError};

/// Check if FFmpeg is available in PATH.
pub fn is_available() -> bool {
    std::process::Command::new("ffmpeg")
        .arg("-version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Remux a DASH/fMP4 audio file to standard MP4 with faststart.
/// Bilibili serves DASH audio as fragmented MP4 which many players
/// (GPAC, deadbeef, foobar2000) can't handle. This converts it to
/// a regular MP4 container with the moov atom at the front.
pub fn remux_to_standard_mp4(input: &Path, output: &Path) -> Result<()> {
    if !is_available() {
        return Err(YadigError::NotFound(
            "FFmpeg is not installed. Install it to enable audio extraction.".into()
        ));
    }

    let status = std::process::Command::new("ffmpeg")
        .arg("-y")
        .arg("-i").arg(input)
        .arg("-c").arg("copy")
        .arg("-movflags").arg("+faststart")
        .arg("-f").arg("mp4")
        .arg(output)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map_err(|e| YadigError::Network(format!("FFmpeg remux error: {}", e)))?;

    if !status.success() {
        return Err(YadigError::Network(
            "FFmpeg remux failed — downloaded stream may be corrupted".into()
        ));
    }

    Ok(())
}

/// A segment to extract from an audio file.
#[derive(Debug, Clone)]
pub struct SplitSegment {
    pub start: f64,
    pub end: f64,
    pub output_path: String,
}

/// Split an audio file into segments using FFmpeg.
/// Tries lossless copy first (-c copy), falls back to re-encoding on failure.
pub fn split_audio(input: &Path, segments: &[SplitSegment]) -> Result<Vec<String>> {
    if !is_available() {
        return Err(YadigError::NotFound(
            "FFmpeg is not installed. Install it to enable chapter splitting. \
             Full audio extraction still works without FFmpeg.".into()
        ));
    }

    let mut outputs = Vec::new();

    for seg in segments {
        let output = Path::new(&seg.output_path);

        // Ensure parent directory exists
        if let Some(parent) = output.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| YadigError::Network(format!("Create dir error: {}", e)))?;
        }

        // Try lossless copy first
        let success = run_ffmpeg(input, output, seg.start, seg.end, true)?;

        if !success {
            // Fallback to re-encoding
            run_ffmpeg(input, output, seg.start, seg.end, false)?;
        }

        outputs.push(seg.output_path.clone());
    }

    Ok(outputs)
}

/// Run FFmpeg with the given parameters.
/// Returns Ok(true) if successful, Ok(false) if the command failed (for retry).
fn run_ffmpeg(input: &Path, output: &Path, start: f64, end: f64, copy_codec: bool) -> Result<bool> {
    let start_str = format!("{:.3}", start);
    let end_str = format!("{:.3}", end);

    let mut cmd = std::process::Command::new("ffmpeg");
    cmd.arg("-y")  // overwrite
       .arg("-i").arg(input)
       .arg("-ss").arg(&start_str)
       .arg("-to").arg(&end_str);

    if copy_codec {
        cmd.arg("-c").arg("copy");
    } else {
        // Re-encode to AAC in MP4 container
        cmd.arg("-c:a").arg("aac")
           .arg("-b:a").arg("192k");
    }

    cmd.arg(output);

    let status = cmd.stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map_err(|e| YadigError::Network(format!("FFmpeg execution error: {}", e)))?;

    Ok(status.success())
}

/// Generate a temp file path for the full download before splitting.
/// Uses a short hash suffix to avoid "File name too long" errors.
pub fn temp_path(download_dir: &Path, title: &str) -> std::path::PathBuf {
    // Use a short hash of the title to keep the filename short
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    title.hash(&mut hasher);
    let hash = hasher.finish();
    download_dir.join(format!(".yadig_temp_{:016x}.m4a", hash))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_segment_debug_format() {
        let seg = SplitSegment {
            start: 0.0,
            end: 120.5,
            output_path: "/tmp/out.m4a".to_string(),
        };
        let debug = format!("{:?}", seg);
        assert!(debug.contains("0.0"));
        assert!(debug.contains("120.5"));
    }

    #[test]
    fn temp_path_generation() {
        let dir = Path::new("/tmp/downloads");
        let p = temp_path(dir, "Test: Video/Name");
        eprintln!("temp_path = {:?}", p);
        let s = p.to_string_lossy();
        assert!(s.contains(".yadig_temp_"), "should contain temp marker");
        assert_eq!(p.extension().unwrap_or_default(), "m4a");
        // Should use a hash, not the original title
        let stem = p.file_stem().unwrap().to_string_lossy();
        assert!(stem.contains("yadig_temp_"));
        let hex_part = stem.trim_start_matches(".yadig_temp_");
        // Just check it looks like a hash
        assert!(!hex_part.is_empty());
    }
}
