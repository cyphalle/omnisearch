use crate::error::OmniResult;
use crate::sources::Source;
use crate::types::{Category, SearchResult};
use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;

pub struct Nyt { pub api_key: String }

#[derive(Deserialize)]
struct NytResponse { response: Option<NytInner> }
#[derive(Deserialize)]
struct NytInner { docs: Vec<NytDoc> }
#[derive(Deserialize)]
struct NytDoc {
    headline: NytHeadline,
    web_url: String,
    snippet: Option<String>,
    pub_date: Option<String>,
    byline: Option<NytByline>,
}
#[derive(Deserialize)]
struct NytHeadline { main: String }
#[derive(Deserialize)]
struct NytByline { original: Option<String> }

#[async_trait]
impl Source for Nyt {
    fn name(&self) -> &str { "nyt" }
    fn category(&self) -> Category { Category::News }
    fn requires_key(&self) -> bool { true }
    fn rate_limit_per_minute(&self) -> u32 { 5 }

    async fn search(&self, client: &Client, query: &str, max_results: usize) -> OmniResult<Vec<SearchResult>> {
        let url = format!("https://api.nytimes.com/svc/search/v2/articlesearch.json?q={}&api-key={}", urlencoding::encode(query), urlencoding::encode(&self.api_key));
        let data: NytResponse = client.get(&url).send().await?.json().await?;
        Ok(data.response.map(|r| r.docs.into_iter().take(max_results).enumerate().map(|(i, d)| {
            SearchResult {
                title: d.headline.main, url: d.web_url,
                snippet: d.snippet.unwrap_or_default(),
                source: "nyt".into(), category: Category::News,
                published: d.pub_date.and_then(|d| d.parse().ok()),
                author: d.byline.and_then(|b| b.original),
                relevance_score: 0.9 - (i as f64 * 0.03),
            }
        }).collect()).unwrap_or_default())
    }
}
