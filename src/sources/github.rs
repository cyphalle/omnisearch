use crate::error::OmniResult;
use crate::sources::Source;
use crate::types::{Category, SearchResult};
use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;

pub struct GitHub { pub api_key: String }

#[derive(Deserialize)]
struct GhResponse { items: Option<Vec<GhRepo>> }
#[derive(Deserialize)]
struct GhRepo { full_name: String, html_url: String, description: Option<String>, stargazers_count: Option<u32>, language: Option<String>, owner: Option<GhOwner> }
#[derive(Deserialize)]
struct GhOwner { login: String }

#[async_trait]
impl Source for GitHub {
    fn name(&self) -> &str { "github" }
    fn category(&self) -> Category { Category::Code }
    fn requires_key(&self) -> bool { true }
    fn rate_limit_per_minute(&self) -> u32 { 30 }

    async fn search(&self, client: &Client, query: &str, max_results: usize) -> OmniResult<Vec<SearchResult>> {
        let limit = max_results.min(30);
        let url = format!("https://api.github.com/search/repositories?q={}&per_page={}", urlencoding::encode(query), limit);
        let data: GhResponse = client.get(&url).header("Authorization", format!("token {}", self.api_key)).header("Accept", "application/vnd.github.v3+json").send().await?.json().await?;
        Ok(data.items.unwrap_or_default().into_iter().enumerate().map(|(i, r)| {
            let stars_boost = (r.stargazers_count.unwrap_or(0) as f64 / 10000.0).min(0.3);
            SearchResult {
                title: r.full_name, url: r.html_url,
                snippet: format!("{} | Stars: {} | Language: {}", r.description.unwrap_or_default(), r.stargazers_count.unwrap_or(0), r.language.unwrap_or("N/A".into())),
                source: "github".into(), category: Category::Code,
                published: None, author: r.owner.map(|o| o.login),
                relevance_score: (0.9 - (i as f64 * 0.02) + stars_boost).min(1.0),
            }
        }).collect())
    }
}
