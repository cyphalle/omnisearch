use crate::error::OmniResult;
use crate::sources::Source;
use crate::types::{Category, SearchResult};
use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;

pub struct MediaStack { pub api_key: String }

#[derive(Deserialize)]
struct MsResponse { data: Option<Vec<MsArticle>> }
#[derive(Deserialize)]
struct MsArticle { title: Option<String>, url: Option<String>, description: Option<String>, author: Option<String>, published_at: Option<String>, source: Option<String> }

#[async_trait]
impl Source for MediaStack {
    fn name(&self) -> &str { "mediastack" }
    fn category(&self) -> Category { Category::News }
    fn requires_key(&self) -> bool { true }
    fn rate_limit_per_minute(&self) -> u32 { 60 }

    async fn search(&self, client: &Client, query: &str, max_results: usize) -> OmniResult<Vec<SearchResult>> {
        let limit = max_results.min(100);
        let url = format!("http://api.mediastack.com/v1/news?access_key={}&keywords={}&limit={}", self.api_key, urlencoding::encode(query), limit);
        let data: MsResponse = client.get(&url).send().await?.json().await?;
        Ok(data.data.unwrap_or_default().into_iter().enumerate().map(|(i, a)| {
            SearchResult {
                title: a.title.unwrap_or_default(), url: a.url.unwrap_or_default(),
                snippet: a.description.unwrap_or_default(),
                source: "mediastack".into(), category: Category::News,
                published: a.published_at.and_then(|d| d.parse().ok()),
                author: a.author,
                relevance_score: 0.8 - (i as f64 * 0.02),
            }
        }).collect())
    }
}
