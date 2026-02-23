use crate::error::OmniResult;
use crate::sources::Source;
use crate::types::{Category, SearchResult};
use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;

pub struct HaveIBeenPwned;

#[derive(Deserialize)]
struct HibpBreach {
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "Title")]
    title: String,
    #[serde(rename = "Domain")]
    domain: String,
    #[serde(rename = "BreachDate")]
    breach_date: String,
    #[serde(rename = "PwnCount")]
    pwn_count: u64,
    #[serde(rename = "Description")]
    description: String,
    #[serde(rename = "DataClasses")]
    data_classes: Vec<String>,
}

#[async_trait]
impl Source for HaveIBeenPwned {
    fn name(&self) -> &str { "haveibeenpwned" }
    fn category(&self) -> Category { Category::Security }
    fn requires_key(&self) -> bool { false }
    fn rate_limit_per_minute(&self) -> u32 { 10 }

    async fn search(&self, client: &Client, query: &str, max_results: usize) -> OmniResult<Vec<SearchResult>> {
        // HIBP public API: search breaches (no key needed for breach list)
        let url = "https://haveibeenpwned.com/api/v3/breaches";
        let resp = client.get(url).send().await?;
        if !resp.status().is_success() { return Ok(vec![]); }
        let breaches: Vec<HibpBreach> = resp.json().await.unwrap_or_default();

        let query_lower = query.to_lowercase();
        let filtered: Vec<SearchResult> = breaches.into_iter()
            .filter(|b| b.name.to_lowercase().contains(&query_lower) || b.title.to_lowercase().contains(&query_lower) || b.domain.to_lowercase().contains(&query_lower))
            .take(max_results)
            .enumerate()
            .map(|(i, b)| {
                let snippet = format!("Domain: {} | Date: {} | Accounts: {} | Data: {}",
                    b.domain, b.breach_date, b.pwn_count, b.data_classes.join(", "));
                SearchResult {
                    title: b.title,
                    url: format!("https://haveibeenpwned.com/PwnedWebsites#{}", b.name),
                    snippet, source: "haveibeenpwned".into(),
                    category: Category::Security,
                    published: None, author: None,
                    relevance_score: 0.8 - (i as f64 * 0.05),
                }
            }).collect();
        Ok(filtered)
    }
}
