use thiserror::Error;

#[derive(Error, Debug)]
pub enum OmniError {
    /// Built only through `From<reqwest::Error>`, which strips the request URL:
    /// many sources pass their API key in the query string.
    #[error("HTTP error: {0}")]
    Http(reqwest::Error),

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

impl From<reqwest::Error> for OmniError {
    fn from(e: reqwest::Error) -> Self {
        OmniError::Http(e.without_url())
    }
}

pub type OmniResult<T> = Result<T, OmniError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn http_error_does_not_leak_api_key() {
        // Port 1 on localhost refuses the connection: the request fails offline.
        let err = reqwest::Client::new()
            .get("http://127.0.0.1:1/search?q=rust&api-key=SECRET")
            .send()
            .await
            .expect_err("connection to port 1 should fail");
        assert!(
            format!("{err}").contains("SECRET"),
            "precondition: reqwest puts the URL in its error"
        );

        let omni: OmniError = err.into();
        assert!(!format!("{omni}").contains("SECRET"));
        assert!(!format!("{omni:?}").contains("SECRET"));
    }
}
