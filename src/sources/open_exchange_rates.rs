use crate::error::OmniResult;
use crate::sources::Source;
use crate::types::{Category, SearchResult};
use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;
use std::collections::HashMap;

pub struct OpenExchangeRates { pub api_key: String }

#[derive(Deserialize)]
struct OerResponse { base: String, rates: HashMap<String, f64> }

#[async_trait]
impl Source for OpenExchangeRates {
    fn name(&self) -> &str { "open_exchange_rates" }
    fn category(&self) -> Category { Category::Finance }
    fn requires_key(&self) -> bool { true }
    fn rate_limit_per_minute(&self) -> u32 { 60 }

    async fn search(&self, client: &Client, query: &str, max_results: usize) -> OmniResult<Vec<SearchResult>> {
        let url = format!("https://openexchangerates.org/api/latest.json?app_id={}", urlencoding::encode(&self.api_key));
        let data: OerResponse = client.get(&url).send().await?.json().await?;
        let query_upper = query.to_uppercase();
        let filtered: Vec<_> = data.rates.iter().filter(|(k, _)| k.contains(&query_upper) || query_upper.contains(k.as_str())).collect();
        Ok(filtered.into_iter().take(max_results).enumerate().map(|(i, (code, rate))| {
            SearchResult {
                title: format!("{} to {} rate", data.base, code),
                url: "https://openexchangerates.org/api/latest.json".to_string(),
                snippet: format!("1 {} = {:.4} {}", data.base, rate, code),
                source: "open_exchange_rates".into(), category: Category::Finance,
                published: None, author: None,
                relevance_score: 0.7 - (i as f64 * 0.05),
            }
        }).collect())
    }
}
