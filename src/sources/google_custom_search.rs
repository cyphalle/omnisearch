use crate::error::OmniResult;
use crate::sources::Source;
use crate::types::{Category, SearchResult};
use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;

pub struct GoogleCustomSearch { pub api_key: String, pub cx: String }

#[derive(Deserialize)]
struct GcsResponse { items: Option<Vec<GcsItem>> }

#[derive(Deserialize)]
struct GcsItem { title: String, link: String, snippet: Option<String> }

#[async_trait]
impl Source for GoogleCustomSearch {
    fn name(&self) -> &str { "google_custom_search" }
    fn category(&self) -> Category { Category::General }
    fn requires_key(&self) -> bool { true }
    fn rate_limit_per_minute(&self) -> u32 { 10 }

    async fn search(&self, client: &Client, query: &str, max_results: usize) -> OmniResult<Vec<SearchResult>> {
        let limit = max_results.min(10);
        let url = format!("https://www.googleapis.com/customsearch/v1?q={}&key={}&cx={}&num={}", urlencoding::encode(query), urlencoding::encode(&self.api_key), urlencoding::encode(&self.cx), limit);
        let data: GcsResponse = client.get(&url).send().await?.json().await?;
        Ok(data.items.unwrap_or_default().into_iter().enumerate().map(|(i, item)| {
            SearchResult {
                title: item.title, url: item.link,
                snippet: item.snippet.unwrap_or_default(),
                source: "google_custom_search".into(), category: Category::General,
                published: None, author: None,
                relevance_score: 1.0 - (i as f64 * 0.05),
            }
        }).collect())
    }
}
