use crate::error::OmniResult;
use crate::sources::Source;
use crate::types::{Category, SearchResult};
use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;

pub struct Pexels { pub api_key: String }

#[derive(Deserialize)]
struct PxResponse { photos: Option<Vec<PxPhoto>> }
#[derive(Deserialize)]
struct PxPhoto { id: u64, url: String, photographer: String, alt: Option<String>, src: PxSrc }
#[derive(Deserialize)]
struct PxSrc { medium: String }

#[async_trait]
impl Source for Pexels {
    fn name(&self) -> &str { "pexels" }
    fn category(&self) -> Category { Category::Media }
    fn requires_key(&self) -> bool { true }
    fn rate_limit_per_minute(&self) -> u32 { 60 }

    async fn search(&self, client: &Client, query: &str, max_results: usize) -> OmniResult<Vec<SearchResult>> {
        let limit = max_results.min(80);
        let url = format!("https://api.pexels.com/v1/search?query={}&per_page={}", urlencoding::encode(query), limit);
        let data: PxResponse = client.get(&url).header("Authorization", &self.api_key).send().await?.json().await?;
        Ok(data.photos.unwrap_or_default().into_iter().enumerate().map(|(i, p)| {
            SearchResult {
                title: p.alt.unwrap_or_else(|| format!("Photo by {}", p.photographer)),
                url: p.url,
                snippet: format!("By {} | {}", p.photographer, p.src.medium),
                source: "pexels".into(), category: Category::Media,
                published: None, author: Some(p.photographer),
                relevance_score: 0.75 - (i as f64 * 0.03),
            }
        }).collect())
    }
}
