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
