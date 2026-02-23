use crate::error::OmniResult;
use crate::sources::Source;
use crate::types::{Category, SearchResult};
use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;

pub struct CommonCrawl;

#[derive(Deserialize)]
struct CcResult {
    url: String,
    #[serde(rename = "urlkey")]
    url_key: String,
    timestamp: String,
    mime: Option<String>,
    status: Option<String>,
    length: Option<String>,
}

#[async_trait]
impl Source for CommonCrawl {
    fn name(&self) -> &str { "common_crawl" }
    fn category(&self) -> Category { Category::General }
    fn requires_key(&self) -> bool { false }
    fn rate_limit_per_minute(&self) -> u32 { 10 }

    async fn search(&self, client: &Client, query: &str, max_results: usize) -> OmniResult<Vec<SearchResult>> {
        let limit = max_results.min(20);
        let url = format!(
            "https://index.commoncrawl.org/CC-MAIN-2024-51-index?url=*.{}*&output=json&limit={}",
            urlencoding::encode(query), limit
        );
        let resp = client.get(&url).send().await?;
        let text = resp.text().await?;
        let results: Vec<SearchResult> = text.lines().filter_map(|line| {
            let cc: CcResult = serde_json::from_str(line).ok()?;
            Some(SearchResult {
                title: cc.url.clone(),
                url: cc.url,
                snippet: format!("Crawled: {} | MIME: {} | Status: {}", cc.timestamp, cc.mime.unwrap_or("N/A".into()), cc.status.unwrap_or("N/A".into())),
                source: "common_crawl".into(),
                category: Category::General,
                published: None, author: None,
                relevance_score: 0.5,
            })
        }).take(max_results).collect();
        Ok(results)
    }
}
