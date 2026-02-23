use thiserror::Error;

#[derive(Error, Debug)]
pub enum OmniError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("JSON parse error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("XML parse error: {0}")]
    Xml(#[from] quick_xml::DeError),

    #[error("Config error: {0}")]
    Config(String),

    #[error("Source error ({src}): {message}")]
    Source { src: String, message: String },

    #[error("Timeout querying {src}")]
    Timeout { src: String },

    #[error("API key required for source: {0}")]
    ApiKeyRequired(String),

    #[error("Rate limited: {0}")]
    RateLimited(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("URL parse error: {0}")]
    UrlParse(#[from] url::ParseError),
}

pub type OmniResult<T> = Result<T, OmniError>;
