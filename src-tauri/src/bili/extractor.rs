use crate::bili::types::{DashAudio, Page, UgcSeason, VideoInfo};
use serde::{Deserialize, Serialize};

/// A single extracted audio segment with its metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioSegment {
    pub title: String,
    pub file_path: String,
    pub duration: u32,
    pub quality: i32,
    pub audio_url: String,
}

/// Result of an audio extraction operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractionResult {
    pub video_title: String,
    pub segments: Vec<AudioSegment>,
    pub extraction_type: ExtractionType,
}

/// Extraction type based on video structure
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ExtractionType {
    /// Single video, single part
    Single,
    /// Multiple pages (分P)
    MultiPart,
    /// Single page with chapter markers (view_points)
    Chapters,
    /// Part of a collection (ugc_season)
    Collection,
}

/// Detect the structure of a video to determine extraction strategy.
pub fn detect_structure(info: &VideoInfo) -> ExtractionType {
    if info.ugc_season.is_some() {
        return ExtractionType::Collection;
    }
    if info.pages.len() > 1 {
        return ExtractionType::MultiPart;
    }
    if !info.view_points.is_empty() {
        return ExtractionType::Chapters;
    }
    ExtractionType::Single
}

/// Select the best audio stream based on account tier.
/// Anonymous: 64K (id=30216), Logged in: 192K (id=30280),
/// Premium: Dolby (id=30250) or Hi-Res (id=30251).
pub fn select_best_audio<'a>(
    streams: &'a [DashAudio],
    has_session: bool,
    is_premium: bool,
) -> Option<&'a DashAudio> {
    if streams.is_empty() {
        return None;
    }

    // Priority order for premium: Hi-Res > Dolby > 192K > 132K > 64K
    // Priority order for logged-in: 192K > 132K > 64K
    // Priority order for anonymous: 64K
    let preferred_ids: Vec<i32> = if is_premium {
        vec![30251, 30250, 30280, 30232, 30216]
    } else if has_session {
        vec![30280, 30232, 30216]
    } else {
        vec![30216]
    };

    for &pref_id in &preferred_ids {
        if let Some(stream) = streams.iter().find(|s| s.id == pref_id) {
            return Some(stream);
        }
    }

    // Fallback: return the stream with highest bandwidth
    streams.iter().max_by_key(|s| s.bandwidth)
}

/// Helper to build a test VideoInfo
fn make_video_info(pages: Vec<Page>, ugc_season: Option<UgcSeason>) -> VideoInfo {
    VideoInfo {
        bvid: "BV1test".to_string(),
        aid: 1,
        title: "Test Video".to_string(),
        videos: pages.len() as u32,
        pages,
        view_points: Vec::new(),
        ugc_season,
    }
}

fn make_page(cid: i64, part: &str, duration: u32) -> Page {
    Page {
        cid,
        page: 1,
        part: part.to_string(),
        duration,
    }
}

fn make_audio(id: i32, bandwidth: i64) -> DashAudio {
    DashAudio {
        id,
        base_url: format!("https://example.com/audio/{}", id),
        bandwidth,
        codecs: "mp4a.40.2".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bili::types::{Episode, Section};

    #[test]
    fn detect_single_video() {
        let info = make_video_info(vec![make_page(1, "Full Video", 300)], None);
        assert_eq!(detect_structure(&info), ExtractionType::Single);
    }

    #[test]
    fn detect_chapter_video() {
        let mut info = make_video_info(vec![make_page(1, "Full Video", 300)], None);
        info.view_points = vec![
            crate::bili::types::ViewPoint {
                content: "Intro".to_string(),
                from: 0.0,
                to: 60.0,
            },
            crate::bili::types::ViewPoint {
                content: "Main".to_string(),
                from: 60.0,
                to: 300.0,
            },
        ];
        assert_eq!(detect_structure(&info), ExtractionType::Chapters);
    }

    #[test]
    fn detect_multi_part_video() {
        let info = make_video_info(
            vec![
                make_page(1, "Song A", 180),
                make_page(2, "Song B", 200),
                make_page(3, "Song C", 150),
            ],
            None,
        );
        assert_eq!(detect_structure(&info), ExtractionType::MultiPart);
    }

    #[test]
    fn detect_collection_video() {
        let season = UgcSeason {
            id: 123,
            title: "My Album".to_string(),
            sections: vec![Section {
                episodes: vec![
                    Episode {
                        aid: 1,
                        bvid: "BV1a".to_string(),
                        cid: 10,
                        title: "Track 1".to_string(),
                    },
                    Episode {
                        aid: 2,
                        bvid: "BV1b".to_string(),
                        cid: 20,
                        title: "Track 2".to_string(),
                    },
                ],
            }],
        };
        let info = make_video_info(vec![make_page(1, "Part 1", 300)], Some(season));
        assert_eq!(detect_structure(&info), ExtractionType::Collection);
    }

    #[test]
    fn select_best_audio_anonymous() {
        let streams = vec![make_audio(30216, 64000), make_audio(30280, 192000)];
        let best = select_best_audio(&streams, false, false).unwrap();
        assert_eq!(best.id, 30216); // anonymous gets 64K only
    }

    #[test]
    fn select_best_audio_logged_in() {
        let streams = vec![make_audio(30216, 64000), make_audio(30280, 192000)];
        let best = select_best_audio(&streams, true, false).unwrap();
        assert_eq!(best.id, 30280); // logged in gets 192K
    }

    #[test]
    fn select_best_audio_premium() {
        let streams = vec![
            make_audio(30216, 64000),
            make_audio(30280, 192000),
            make_audio(30251, 320000),
        ];
        let best = select_best_audio(&streams, true, true).unwrap();
        assert_eq!(best.id, 30251); // premium gets Hi-Res
    }

    #[test]
    fn select_best_audio_empty_streams() {
        let streams = vec![];
        assert!(select_best_audio(&streams, true, true).is_none());
    }
}
