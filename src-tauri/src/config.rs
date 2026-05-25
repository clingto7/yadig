use std::sync::Arc;
use std::sync::RwLock;

/// Shared runtime state for Discogs API credentials.
///
/// Cloned cheaply (Arc bumps refcount) and passed to DiscogsSource so
/// that the Tauri `update_discogs_keys` command can write new keys
/// without rebuilding the source registry or mutating the SourceProvider
/// trait bound.
#[derive(Clone)]
pub struct DiscogsKeys {
    pub key: Arc<RwLock<Option<String>>>,
    pub secret: Arc<RwLock<Option<String>>>,
}

impl DiscogsKeys {
    pub fn new() -> Self {
        Self {
            key: Arc::new(RwLock::new(None)),
            secret: Arc::new(RwLock::new(None)),
        }
    }
}
