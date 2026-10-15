use crate::error::OmniResult;
use crate::sources::Source;
use crate::types::{Category, SearchResult};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::Deserialize;

pub struct GNews { pub api_key: String }

#[derive(Deserialize)]
struct GnResponse { articles: Option<Vec<GnArticle>> }
#[derive(Deserialize)]
struct GnArticle { title: String, url: String, description: Option<String>, #[serde(rename = "publishedAt")] published_at: Option<String>, source: Option<GnSource> }
#[derive(Deserialize)]
struct GnSource { name: Option<String> }

#[async_trait]
impl Source for GNews {
    fn name(&self) -> &str { "gnews" }
    fn category(&self) -> Category { Category::News }
    fn requires_key(&self) -> bool { true }
    fn rate_limit_per_minute(&self) -> u32 { 60 }

    async fn search(&self, client: &Client, query: &str, max_results: usize) -> OmniResult<Vec<SearchResult>> {
        let limit = max_results.min(10);
        let url = format!("https://gnews.io/api/v4/search?q={}&max={}&token={}", urlencoding::encode(query), limit, urlencoding::encode(&self.api_key));
        let data: GnResponse = client.get(&url).send().await?.json().await?;
        Ok(data.articles.unwrap_or_default().into_iter().enumerate().map(|(i, a)| {
            SearchResult {
                title: a.title, url: a.url,
                snippet: a.description.unwrap_or_default(),
                source: "gnews".into(), category: Category::News,
                published: a.published_at.and_then(|d| d.parse::<DateTime<Utc>>().ok()),
                author: a.source.and_then(|s| s.name),
                relevance_score: 0.85 - (i as f64 * 0.03),
            }
        }).collect())
    }
}
