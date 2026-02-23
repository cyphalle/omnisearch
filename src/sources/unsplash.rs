use crate::error::OmniResult;
use crate::sources::Source;
use crate::types::{Category, SearchResult};
use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;

pub struct Unsplash { pub api_key: String }

#[derive(Deserialize)]
struct UsResponse { results: Vec<UsPhoto> }
#[derive(Deserialize)]
struct UsPhoto { id: String, description: Option<String>, alt_description: Option<String>, urls: UsUrls, user: UsUser, likes: Option<u32> }
#[derive(Deserialize)]
struct UsUrls { regular: String }
#[derive(Deserialize)]
struct UsUser { name: String }

#[async_trait]
impl Source for Unsplash {
    fn name(&self) -> &str { "unsplash" }
    fn category(&self) -> Category { Category::Media }
    fn requires_key(&self) -> bool { true }
    fn rate_limit_per_minute(&self) -> u32 { 50 }

    async fn search(&self, client: &Client, query: &str, max_results: usize) -> OmniResult<Vec<SearchResult>> {
        let limit = max_results.min(30);
        let url = format!("https://api.unsplash.com/search/photos?query={}&per_page={}", urlencoding::encode(query), limit);
        let data: UsResponse = client.get(&url).header("Authorization", format!("Client-ID {}", self.api_key)).send().await?.json().await?;
        Ok(data.results.into_iter().enumerate().map(|(i, p)| {
            SearchResult {
                title: p.description.or(p.alt_description).unwrap_or_else(|| format!("Photo by {}", p.user.name)),
                url: format!("https://unsplash.com/photos/{}", p.id),
                snippet: format!("By {} | Likes: {} | {}", p.user.name, p.likes.unwrap_or(0), p.urls.regular),
                source: "unsplash".into(), category: Category::Media,
                published: None, author: Some(p.user.name),
                relevance_score: 0.75 - (i as f64 * 0.03),
            }
        }).collect())
    }
}
