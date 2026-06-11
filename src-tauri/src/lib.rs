mod bili;
mod commands;
mod config;
mod error;
mod http_client;
mod library;
mod llm;
mod source;
mod youtube;

use bili::auth::BiliAuth;
use config::DiscogsKeys;
use source::api::bilibili::BiliSource;
use source::api::discogs::DiscogsSource;
use source::api::jamendo::JamendoSource;
use source::api::lastfm::LastFmSource;
use source::api::musicbrainz::MusicBrainzSource;
use source::api::youtube::YouTubeSource;
use source::registry::SourceRegistry;
use source::rss::feeds::{FaderSource, StereogumSource};
use source::rss::pitchfork::PitchforkSource;
use source::scraper::albumoftheyear::AlbumOfTheYearSource;
use source::scraper::bandcamp::BandcampSource;
use tauri_plugin_sql::{Migration, MigrationKind};
use youtube::YoutubeClient;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let migrations = vec![
        Migration {
            version: 1,
            description: "create_initial_tables",
            sql: include_str!("../migrations/001_initial_schema.sql"),
            kind: MigrationKind::Up,
        },
        Migration {
            version: 2,
            description: "create_search_history",
            sql: include_str!("../migrations/002_search_history.sql"),
            kind: MigrationKind::Up,
        },
        Migration {
            version: 3,
            description: "create_rss_feeds",
            sql: include_str!("../migrations/003_rss_feeds.sql"),
            kind: MigrationKind::Up,
        },
        Migration {
            version: 4,
            description: "create_library_tables",
            sql: include_str!("../migrations/004_library_tables.sql"),
            kind: MigrationKind::Up,
        },
        Migration {
            version: 5,
            description: "add_library_item_collection_metadata",
            sql: include_str!("../migrations/005_library_item_collection_metadata.sql"),
            kind: MigrationKind::Up,
        },
        Migration {
            version: 6,
            description: "add_operation_plan_item_remote_fields",
            sql: include_str!("../migrations/006_operation_plan_item_remote_fields.sql"),
            kind: MigrationKind::Up,
        },
        Migration {
            version: 7,
            description: "create_llm_classifications",
            sql: include_str!("../migrations/007_llm_classifications.sql"),
            kind: MigrationKind::Up,
        },
        Migration {
            version: 8,
            description: "add_operation_plan_item_metadata",
            sql: include_str!("../migrations/008_operation_plan_item_metadata.sql"),
            kind: MigrationKind::Up,
        },
    ];

    let mut registry = SourceRegistry::new();
    let discogs_keys = DiscogsKeys::new();
    let bili_auth = BiliAuth::new();

    // YouTube client — uses external yt-dlp CLI, output in Downloads/yadig
    let output_dir = dirs_next::download_dir()
        .map(|d| d.join("yadig"))
        .unwrap_or_else(|| std::path::PathBuf::from("yadig-output"));
    let youtube_client = YoutubeClient::new(output_dir);

    // Register built-in sources
    registry.register(Box::new(PitchforkSource::new()));
    registry.register(Box::new(DiscogsSource::new(discogs_keys.clone())));
    registry.register(Box::new(BandcampSource::new()));
    registry.register(Box::new(AlbumOfTheYearSource::new()));
    registry.register(Box::new(JamendoSource::new()));
    registry.register(Box::new(StereogumSource::new()));
    registry.register(Box::new(FaderSource::new()));
    registry.register(Box::new(BiliSource::new(bili_auth.clone())));
    registry.register(Box::new(MusicBrainzSource::new()));
    registry.register(Box::new(LastFmSource::new()));
    registry.register(Box::new(YouTubeSource::new()));

    tauri::Builder::default()
        .plugin(
            tauri_plugin_sql::Builder::default()
                .add_migrations("sqlite:yadig.db", migrations)
                .build(),
        )
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_notification::init())
        .manage(registry)
        .manage(discogs_keys)
        .manage(bili_auth)
        .manage(youtube_client)
        .invoke_handler(tauri::generate_handler![
            commands::search::search_sources,
            commands::search::fetch_latest,
            commands::search::list_sources,
            commands::search::set_source_enabled,
            commands::search::update_discogs_keys,
            commands::search::download_audio,
            commands::search::open_url,
            commands::bilibili::bili_qr_login_start,
            commands::bilibili::bili_qr_login_poll,
            commands::bilibili::bili_cookie_login,
            commands::bilibili::bili_restore_session,
            commands::bilibili::bili_password_login,
            commands::bilibili::bili_logout,
            commands::bilibili::bili_session_status,
            commands::bilibili::bili_extract_audio,
            commands::bilibili::bili_extract_segment,
            commands::bilibili::bili_extract_collection,
            commands::bilibili::bili_check_ffmpeg,
            commands::bilibili::bili_get_playurl,
            commands::library::bili_sync_library,
            commands::library::llm_analyze_items,
            commands::library::llm_classify_items,
            commands::library::llm_test_provider,
            commands::library::create_bili_audio_extraction_plan,
            commands::library::create_bili_favorite_operation_plan,
            commands::library::create_bili_favorite_folder_create_plan,
            commands::library::create_bili_favorite_folder_rename_plan,
            commands::library::execute_bili_audio_extraction_plan,
            commands::library::execute_bili_favorite_copy_plan,
            commands::library::execute_bili_favorite_move_plan,
            commands::library::execute_bili_favorite_delete_plan,
            commands::library::execute_bili_favorite_folder_create_plan,
            commands::library::execute_bili_favorite_folder_rename_plan,
            commands::youtube::youtube_extract_audio,
            commands::youtube::youtube_search,
            commands::youtube::youtube_check_ready,
        ])
        .run(tauri::generate_context!())
        .expect("error while running yadig");
}

#[cfg(test)]
mod media_workstation_tests {
    use crate::bili::auth::BiliSession;
    use crate::bili::session::parse_cookie_session;
    use crate::library::{
        AudioExtractionCandidate, BiliResourceKind, LibraryItem, LibraryItemType, OperationPlan,
        OperationPlanKind,
    };
    use crate::llm::{parse_llm_analysis, LlmSuggestedAction};

    #[test]
    fn parses_full_bilibili_cookie_for_write_operations() {
        let session = parse_cookie_session(
            "buvid3=abc; SESSDATA=sess; bili_jct=csrf-token; DedeUserID=12345;",
        )
        .expect("full cookie should parse");

        assert_eq!(
            session,
            BiliSession {
                sessdata: "sess".to_string(),
                bili_jct: "csrf-token".to_string(),
                dede_user_id: "12345".to_string(),
                vip_status: 0,
            }
        );
    }

    #[test]
    fn normalizes_bili_video_metadata_into_library_item() {
        let item = LibraryItem::from_bili_video(
            BiliResourceKind::FavoriteVideo,
            "BV1abc".to_string(),
            "城市流行采样合集".to_string(),
            Some("音乐UP".to_string()),
            serde_json::json!({
                "tid": 3,
                "tname": "音乐",
                "play": 12000,
                "fav_time": 1781070000
            }),
        );

        assert_eq!(item.source, "bilibili");
        assert_eq!(item.external_id, "BV1abc");
        assert_eq!(item.item_type, LibraryItemType::BiliFavoriteVideo);
        assert_eq!(item.title, "城市流行采样合集");
        assert_eq!(item.author.as_deref(), Some("音乐UP"));
        assert_eq!(item.raw_metadata["tname"].as_str(), Some("音乐"));
    }

    #[test]
    fn parses_structured_llm_suggestions() {
        let response = r#"{
          "items": [
            {
              "external_id": "BV1abc",
              "suggested_tags": ["音乐", "采样"],
              "reason": "标题和分区都指向音乐内容",
              "confidence": 0.91,
              "suggested_action": {
                "kind": "extract_audio",
                "target": "music-audio"
              }
            }
          ]
        }"#;

        let parsed = parse_llm_analysis(response).expect("valid structured response");
        assert_eq!(parsed.items.len(), 1);
        assert_eq!(parsed.items[0].external_id, "BV1abc");
        assert_eq!(parsed.items[0].suggested_tags, vec!["音乐", "采样"]);
        assert_eq!(
            parsed.items[0].suggested_action,
            Some(LlmSuggestedAction {
                kind: "extract_audio".to_string(),
                target: Some("music-audio".to_string()),
            })
        );
    }

    #[test]
    fn builds_audio_extraction_operation_plan_for_music_videos() {
        let candidates = vec![
            AudioExtractionCandidate {
                bvid: "BV1music".to_string(),
                title: "现场音乐".to_string(),
                is_music: true,
            },
            AudioExtractionCandidate {
                bvid: "BV1game".to_string(),
                title: "游戏攻略".to_string(),
                is_music: false,
            },
        ];

        let plan = OperationPlan::for_bili_audio_extraction(candidates);

        assert_eq!(plan.kind, OperationPlanKind::BiliBatchAudioExtraction);
        assert_eq!(plan.items.len(), 1);
        assert_eq!(plan.items[0].external_id, "BV1music");
        assert_eq!(plan.items[0].action, "extract_audio");
    }
}
