use crate::error::OmniResult;
use crate::sources::Source;
use crate::types::{Category, SearchResult};
use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;

pub struct Polygon { pub api_key: String }

#[derive(Deserialize)]
struct PolyResponse { results: Option<Vec<PolyTicker>> }
#[derive(Deserialize)]
struct PolyTicker { ticker: String, name: String, market: Option<String>, locale: Option<String>, currency_name: Option<String> }

#[async_trait]
impl Source for Polygon {
    fn name(&self) -> &str { "polygon" }
    fn category(&self) -> Category { Category::Finance }
    fn requires_key(&self) -> bool { true }
    fn rate_limit_per_minute(&self) -> u32 { 5 }

    async fn search(&self, client: &Client, query: &str, max_results: usize) -> OmniResult<Vec<SearchResult>> {
        let limit = max_results.min(50);
        let url = format!("https://api.polygon.io/v3/reference/tickers?search={}&limit={}&apiKey={}", urlencoding::encode(query), limit, self.api_key);
        let data: PolyResponse = client.get(&url).send().await?.json().await?;
        Ok(data.results.unwrap_or_default().into_iter().enumerate().map(|(i, t)| {
            SearchResult {
                title: format!("{} ({})", t.name, t.ticker),
                url: format!("https://polygon.io/quote/{}", t.ticker),
                snippet: format!("Market: {} | Currency: {}", t.market.unwrap_or("N/A".into()), t.currency_name.unwrap_or("N/A".into())),
                source: "polygon".into(), category: Category::Finance,
                published: None, author: None,
                relevance_score: 0.8 - (i as f64 * 0.03),
            }
        }).collect())
    }
}
