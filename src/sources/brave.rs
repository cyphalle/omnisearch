use crate::error::OmniResult;
use crate::sources::Source;
use crate::types::{Category, SearchResult};
use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;

pub struct Brave {
    pub api_key: String,
}

#[derive(Deserialize)]
struct BraveResponse {
    web: Option<BraveWeb>,
}

#[derive(Deserialize)]
struct BraveWeb {
    results: Vec<BraveResult>,
}

#[derive(Deserialize)]
struct BraveResult {
    title: String,
    url: String,
    description: Option<String>,
}

#[async_trait]
impl Source for Brave {
    fn name(&self) -> &str { "brave" }
    fn category(&self) -> Category { Category::General }
    fn requires_key(&self) -> bool { true }
    fn rate_limit_per_minute(&self) -> u32 { 15 }

    async fn search(&self, client: &Client, query: &str, max_results: usize) -> OmniResult<Vec<SearchResult>> {
        let limit = max_results.min(20);
        let url = format!("https://api.search.brave.com/res/v1/web/search?q={}&count={}", urlencoding::encode(query), limit);
        let resp = client.get(&url).header("X-Subscription-Token", &self.api_key).header("Accept", "application/json").send().await?;
        let data: BraveResponse = resp.json().await?;
        Ok(data.web.map(|w| w.results.into_iter().enumerate().map(|(i, r)| {
            SearchResult {
                title: r.title, url: r.url,
                snippet: r.description.unwrap_or_default(),
                source: "brave".into(), category: Category::General,
                published: None, author: None,
                relevance_score: 1.0 - (i as f64 * 0.03),
            }
        }).collect()).unwrap_or_default())
    }
}
