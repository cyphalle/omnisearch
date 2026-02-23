use crate::error::OmniResult;
use crate::sources::Source;
use crate::types::{Category, SearchResult};
use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;

pub struct UrlScan { pub api_key: String }

#[derive(Deserialize)]
struct UsResponse { results: Option<Vec<UsResult>> }
#[derive(Deserialize)]
struct UsResult { task: UsTask, page: Option<UsPage>, stats: Option<UsStats> }
#[derive(Deserialize)]
struct UsTask { url: String, time: Option<String> }
#[derive(Deserialize)]
struct UsPage { domain: Option<String>, ip: Option<String>, server: Option<String>, title: Option<String> }
#[derive(Deserialize)]
struct UsStats { #[serde(rename = "malicious")] malicious: Option<u32> }

#[async_trait]
impl Source for UrlScan {
    fn name(&self) -> &str { "urlscan" }
    fn category(&self) -> Category { Category::Security }
    fn requires_key(&self) -> bool { true }
    fn rate_limit_per_minute(&self) -> u32 { 60 }

    async fn search(&self, client: &Client, query: &str, max_results: usize) -> OmniResult<Vec<SearchResult>> {
        let limit = max_results.min(100);
        let url = format!("https://urlscan.io/api/v1/search/?q={}&size={}", urlencoding::encode(query), limit);
        let data: UsResponse = client.get(&url).header("API-Key", &self.api_key).send().await?.json().await?;
        Ok(data.results.unwrap_or_default().into_iter().enumerate().map(|(i, r)| {
            let page = r.page.unwrap_or(UsPage { domain: None, ip: None, server: None, title: None });
            SearchResult {
                title: page.title.unwrap_or_else(|| r.task.url.clone()),
                url: r.task.url,
                snippet: format!("Domain: {} | IP: {} | Server: {}", page.domain.unwrap_or("N/A".into()), page.ip.unwrap_or("N/A".into()), page.server.unwrap_or("N/A".into())),
                source: "urlscan".into(), category: Category::Security,
                published: r.task.time.and_then(|d| d.parse().ok()), author: None,
                relevance_score: 0.75 - (i as f64 * 0.02),
            }
        }).collect())
    }
}
