use crate::error::OmniResult;
use crate::sources::Source;
use crate::types::{Category, SearchResult};
use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;

pub struct Currents { pub api_key: String }

#[derive(Deserialize)]
struct CuResponse { news: Option<Vec<CuArticle>> }
#[derive(Deserialize)]
struct CuArticle { title: String, url: String, description: Option<String>, author: Option<String>, published: Option<String> }

#[async_trait]
impl Source for Currents {
    fn name(&self) -> &str { "currents" }
    fn category(&self) -> Category { Category::News }
    fn requires_key(&self) -> bool { true }

    async fn search(&self, client: &Client, query: &str, max_results: usize) -> OmniResult<Vec<SearchResult>> {
        let url = format!("https://api.currentsapi.services/v1/search?keywords={}&apiKey={}", urlencoding::encode(query), self.api_key);
        let data: CuResponse = client.get(&url).send().await?.json().await?;
        Ok(data.news.unwrap_or_default().into_iter().take(max_results).enumerate().map(|(i, a)| {
            SearchResult {
                title: a.title, url: a.url, snippet: a.description.unwrap_or_default(),
                source: "currents".into(), category: Category::News,
                published: a.published.and_then(|d| d.parse().ok()), author: a.author,
                relevance_score: 0.8 - (i as f64 * 0.03),
            }
        }).collect())
    }
}
