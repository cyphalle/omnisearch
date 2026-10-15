use crate::error::OmniResult;
use crate::sources::Source;
use crate::types::{Category, SearchResult};
use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;

pub struct EventRegistry { pub api_key: String }

#[derive(Deserialize)]
struct ErResponse { articles: Option<ErArticles> }
#[derive(Deserialize)]
struct ErArticles { results: Vec<ErArticle> }
#[derive(Deserialize)]
struct ErArticle { title: String, url: String, body: Option<String>, #[serde(rename = "dateTimePub")] date_time_pub: Option<String>, source: Option<ErSource> }
#[derive(Deserialize)]
struct ErSource { title: Option<String> }

#[async_trait]
impl Source for EventRegistry {
    fn name(&self) -> &str { "event_registry" }
    fn category(&self) -> Category { Category::News }
    fn requires_key(&self) -> bool { true }
    fn rate_limit_per_minute(&self) -> u32 { 30 }

    async fn search(&self, client: &Client, query: &str, max_results: usize) -> OmniResult<Vec<SearchResult>> {
        let limit = max_results.min(50);
        let url = format!("https://eventregistry.org/api/v1/article/getArticles?keyword={}&articlesCount={}&apiKey={}&resultType=articles", urlencoding::encode(query), limit, urlencoding::encode(&self.api_key));
        let data: ErResponse = client.get(&url).send().await?.json().await?;
        Ok(data.articles.map(|a| a.results.into_iter().enumerate().map(|(i, a)| {
            SearchResult {
                title: a.title, url: a.url,
                snippet: a.body.unwrap_or_default().chars().take(300).collect(),
                source: "event_registry".into(), category: Category::News,
                published: a.date_time_pub.and_then(|d| d.parse().ok()),
                author: a.source.and_then(|s| s.title),
                relevance_score: 0.85 - (i as f64 * 0.02),
            }
        }).collect()).unwrap_or_default())
    }
}
