use reqwest::Client;
use std::time::Duration;

pub fn build_client(timeout_secs: u64) -> Client {
    Client::builder()
        .timeout(Duration::from_secs(timeout_secs))
        .user_agent("omnisearch/0.1.0")
        .pool_max_idle_per_host(10)
        .build()
        .expect("Failed to build HTTP client")
}

/// Read a response body, failing once it exceeds `max_bytes` instead of
/// buffering an unbounded payload in memory.
pub async fn read_body_capped(
    mut resp: reqwest::Response,
    max_bytes: usize,
    source: &str,
) -> crate::error::OmniResult<Vec<u8>> {
    let too_large = || crate::error::OmniError::Source {
        src: source.to_string(),
        message: format!("response larger than {max_bytes} bytes, ignored"),
    };
    if resp.content_length().is_some_and(|len| len > max_bytes as u64) {
        return Err(too_large());
    }
    let mut body = Vec::new();
    while let Some(chunk) = resp.chunk().await? {
        if body.len() + chunk.len() > max_bytes {
            return Err(too_large());
        }
        body.extend_from_slice(&chunk);
    }
    Ok(body)
}
