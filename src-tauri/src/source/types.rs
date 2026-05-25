use serde::{Deserialize, Serialize};

/// The type of source — determines how we fetch data from it
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum SourceKind {
    /// RSS/Atom feed — structured XML, lowest maintenance
    Rss,
    /// REST API with known endpoints (e.g., Discogs)
    Api,
    /// Website that requires HTML scraping
    Scraper,
}

impl std::fmt::Display for SourceKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SourceKind::Rss => write!(f, "rss"),
            SourceKind::Api => write!(f, "api"),
            SourceKind::Scraper => write!(f, "scraper"),
        }
    }
}

impl std::str::FromStr for SourceKind {
    type Err = String;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "rss" => Ok(SourceKind::Rss),
            "api" => Ok(SourceKind::Api),
            "scraper" => Ok(SourceKind::Scraper),
            _ => Err(format!("Unknown source kind: {}", s)),
        }
    }
}

/// A registered information source in the app
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Source {
    pub id: String,
    pub name: String,
    pub kind: SourceKind,
    pub base_url: String,
    pub is_active: bool,
}

/// A single content item fetched from any source
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentItem {
    pub source_id: String,
    pub title: String,
    pub url: String,
    pub summary: Option<String>,
    pub author: Option<String>,
    pub published_at: Option<String>,
    pub image_url: Option<String>,
    /// Source-specific structured data (e.g., album rating, genre tags)
    pub extra: Option<serde_json::Value>,
}

/// Pagination info for a search result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchPage {
    pub page: usize,
    pub has_more: bool,
}

/// A search result across sources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub query: String,
    pub items: Vec<ContentItem>,
    pub total_results: usize,
    pub elapsed_ms: u64,
    pub page: SearchPage,
}
