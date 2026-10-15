use crate::error::OmniResult;
use crate::sources::Source;
use crate::types::{Category, SearchResult};
use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;

pub struct AlphaVantage { pub api_key: String }

#[derive(Deserialize)]
struct AvResponse { #[serde(rename = "bestMatches")] best_matches: Option<Vec<AvMatch>> }
#[derive(Deserialize)]
struct AvMatch {
    #[serde(rename = "1. symbol")] symbol: String,
    #[serde(rename = "2. name")] name: String,
    #[serde(rename = "3. type")] match_type: String,
    #[serde(rename = "4. region")] region: String,
    #[serde(rename = "8. currency")] currency: String,
}

#[async_trait]
impl Source for AlphaVantage {
    fn name(&self) -> &str { "alpha_vantage" }
    fn category(&self) -> Category { Category::Finance }
    fn requires_key(&self) -> bool { true }
    fn rate_limit_per_minute(&self) -> u32 { 5 }

    async fn search(&self, client: &Client, query: &str, max_results: usize) -> OmniResult<Vec<SearchResult>> {
        let url = format!("https://www.alphavantage.co/query?function=SYMBOL_SEARCH&keywords={}&apikey={}", urlencoding::encode(query), urlencoding::encode(&self.api_key));
        let data: AvResponse = client.get(&url).send().await?.json().await?;
        Ok(data.best_matches.unwrap_or_default().into_iter().take(max_results).enumerate().map(|(i, m)| {
            SearchResult {
                title: format!("{} ({})", m.name, m.symbol),
                url: format!("https://finance.yahoo.com/quote/{}", m.symbol),
                snippet: format!("Type: {} | Region: {} | Currency: {}", m.match_type, m.region, m.currency),
                source: "alpha_vantage".into(), category: Category::Finance,
                published: None, author: None,
                relevance_score: 0.85 - (i as f64 * 0.05),
            }
        }).collect())
    }
}
