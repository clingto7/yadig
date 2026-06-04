mod bili;
mod commands;
mod config;
mod error;
mod http_client;
mod source;

use bili::auth::BiliAuth;
use config::DiscogsKeys;
use source::registry::SourceRegistry;
use source::rss::pitchfork::PitchforkSource;
use source::rss::feeds::{StereogumSource, FaderSource};
use source::api::bilibili::BiliSource;
use source::api::discogs::DiscogsSource;
use source::api::jamendo::JamendoSource;
use source::scraper::bandcamp::BandcampSource;
use source::scraper::albumoftheyear::AlbumOfTheYearSource;
use tauri_plugin_sql::{Migration, MigrationKind};

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
    ];

    let mut registry = SourceRegistry::new();
    let discogs_keys = DiscogsKeys::new();
    let bili_auth = BiliAuth::new();

    // Register built-in sources
    registry.register(Box::new(PitchforkSource::new()));
    registry.register(Box::new(DiscogsSource::new(discogs_keys.clone())));
    registry.register(Box::new(BandcampSource::new()));
    registry.register(Box::new(AlbumOfTheYearSource::new()));
    registry.register(Box::new(JamendoSource::new()));
    registry.register(Box::new(StereogumSource::new()));
    registry.register(Box::new(FaderSource::new()));
    registry.register(Box::new(BiliSource::new(bili_auth.clone())));

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
            commands::bilibili::bili_password_login,
            commands::bilibili::bili_logout,
            commands::bilibili::bili_session_status,
            commands::bilibili::bili_extract_audio,
            commands::bilibili::bili_extract_segment,
            commands::bilibili::bili_extract_collection,
            commands::bilibili::bili_check_ffmpeg,
            commands::bilibili::bili_get_playurl,
        ])
        .run(tauri::generate_context!())
        .expect("error while running yadig");
}
