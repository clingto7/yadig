use async_trait::async_trait;
use crate::error::Result;
use crate::source::types::*;

/// The core trait that every information source implements.
/// Regardless of whether it's RSS, API, or scraper — they all conform to this interface.
#[async_trait]
pub trait SourceProvider: Send + Sync {
    /// Unique identifier for this source (e.g., "pitchfork", "discogs")
    fn id(&self) -> &str;

    /// Human-readable name
    fn name(&self) -> &str;

    /// What kind of source this is
    fn kind(&self) -> SourceKind;

    /// Search for content matching the query
    async fn search(&self, query: &str, limit: usize) -> Result<Vec<ContentItem>>;

    /// Fetch the latest content (for feed/timeline view)
    async fn fetch_latest(&self, limit: usize) -> Result<Vec<ContentItem>>;

    /// Fetch a specific item by URL
    async fn get_item(&self, url: &str) -> Result<ContentItem>;
}
