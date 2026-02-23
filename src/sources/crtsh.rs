use crate::error::OmniResult;
use crate::sources::Source;
use crate::types::{Category, SearchResult};
use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;

pub struct CrtSh;

#[derive(Deserialize)]
struct CrtEntry {
    id: u64,
    common_name: String,
    name_value: String,
    issuer_name: String,
    not_before: String,
    not_after: String,
}

#[async_trait]
impl Source for CrtSh {
    fn name(&self) -> &str { "crtsh" }
    fn category(&self) -> Category { Category::Security }
    fn requires_key(&self) -> bool { false }
    fn rate_limit_per_minute(&self) -> u32 { 30 }

    async fn search(&self, client: &Client, query: &str, max_results: usize) -> OmniResult<Vec<SearchResult>> {
        let url = format!("https://crt.sh/?q={}&output=json", urlencoding::encode(query));
        let resp = client.get(&url).send().await?;
        if !resp.status().is_success() { return Ok(vec![]); }
        let entries: Vec<CrtEntry> = resp.json().await.unwrap_or_default();

        Ok(entries.into_iter().take(max_results).enumerate().map(|(i, e)| {
            SearchResult {
                title: format!("Certificate: {}", e.common_name),
                url: format!("https://crt.sh/?id={}", e.id),
                snippet: format!("Names: {} | Issuer: {} | Valid: {} to {}", e.name_value.replace('\n', ", "), e.issuer_name, e.not_before, e.not_after),
                source: "crtsh".into(),
                category: Category::Security,
                published: None, author: None,
                relevance_score: 0.7 - (i as f64 * 0.02),
            }
        }).collect())
    }
}
