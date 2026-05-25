use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub enum YadigError {
    #[error("Discogs API error: {0}")]
    Discogs(String),

    #[error("Feed error: {0}")]
    Feed(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error("{0}")]
    NotFound(String),
}

impl Serialize for YadigError {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

pub type Result<T> = std::result::Result<T, YadigError>;

impl From<reqwest::Error> for YadigError {
    fn from(e: reqwest::Error) -> Self {
        YadigError::Network(e.to_string())
    }
}
