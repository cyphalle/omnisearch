use crate::error::OmniResult;
use crate::sources::Source;
use crate::types::{Category, SearchResult};
use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;

pub struct Shodan { pub api_key: String }

#[derive(Deserialize)]
struct ShodanResponse { matches: Option<Vec<ShodanMatch>>, total: Option<u64> }
#[derive(Deserialize)]
struct ShodanMatch { ip_str: String, port: u16, org: Option<String>, os: Option<String>, product: Option<String>, data: Option<String> }

#[async_trait]
impl Source for Shodan {
    fn name(&self) -> &str { "shodan" }
    fn category(&self) -> Category { Category::Security }
    fn requires_key(&self) -> bool { true }
    fn rate_limit_per_minute(&self) -> u32 { 1 }

    async fn search(&self, client: &Client, query: &str, max_results: usize) -> OmniResult<Vec<SearchResult>> {
        let url = format!("https://api.shodan.io/shodan/host/search?query={}&key={}", urlencoding::encode(query), self.api_key);
        let data: ShodanResponse = client.get(&url).send().await?.json().await?;
        Ok(data.matches.unwrap_or_default().into_iter().take(max_results).enumerate().map(|(i, m)| {
            SearchResult {
                title: format!("{}:{}", m.ip_str, m.port),
                url: format!("https://www.shodan.io/host/{}", m.ip_str),
                snippet: format!("Org: {} | OS: {} | Product: {} | {}", m.org.unwrap_or("N/A".into()), m.os.unwrap_or("N/A".into()), m.product.unwrap_or("N/A".into()), m.data.unwrap_or_default().chars().take(200).collect::<String>()),
                source: "shodan".into(), category: Category::Security,
                published: None, author: None,
                relevance_score: 0.8 - (i as f64 * 0.03),
            }
        }).collect())
    }
}
