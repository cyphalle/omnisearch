use crate::error::OmniResult;
use crate::sources::Source;
use crate::types::{Category, SearchResult};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::Deserialize;

pub struct NewsApi { pub api_key: String }

#[derive(Deserialize)]
struct NaResponse { articles: Option<Vec<NaArticle>> }

#[derive(Deserialize)]
struct NaArticle {
    title: Option<String>, url: Option<String>,
    description: Option<String>, author: Option<String>,
    #[serde(rename = "publishedAt")]
    published_at: Option<String>,
    source: Option<NaSource>,
}

#[derive(Deserialize)]
struct NaSource { name: Option<String> }

#[async_trait]
impl Source for NewsApi {
    fn name(&self) -> &str { "newsapi" }
    fn category(&self) -> Category { Category::News }
    fn requires_key(&self) -> bool { true }
    fn rate_limit_per_minute(&self) -> u32 { 60 }

    async fn search(&self, client: &Client, query: &str, max_results: usize) -> OmniResult<Vec<SearchResult>> {
        let limit = max_results.min(100);
        let url = format!("https://newsapi.org/v2/everything?q={}&pageSize={}&apiKey={}", urlencoding::encode(query), limit, urlencoding::encode(&self.api_key));
        let data: NaResponse = client.get(&url).send().await?.json().await?;
        Ok(data.articles.unwrap_or_default().into_iter().enumerate().map(|(i, a)| {
            SearchResult {
                title: a.title.unwrap_or_default(), url: a.url.unwrap_or_default(),
                snippet: a.description.unwrap_or_default(),
                source: "newsapi".into(), category: Category::News,
                published: a.published_at.and_then(|d| d.parse::<DateTime<Utc>>().ok()),
                author: a.author,
                relevance_score: 0.9 - (i as f64 * 0.02),
            }
        }).collect())
    }
}
