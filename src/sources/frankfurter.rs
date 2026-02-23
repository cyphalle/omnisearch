use crate::error::OmniResult;
use crate::sources::Source;
use crate::types::{Category, SearchResult};
use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;
use std::collections::HashMap;

pub struct Frankfurter;

#[derive(Deserialize)]
struct FrankResponse {
    base: String,
    date: String,
    rates: HashMap<String, f64>,
}

#[async_trait]
impl Source for Frankfurter {
    fn name(&self) -> &str { "frankfurter" }
    fn category(&self) -> Category { Category::Finance }
    fn requires_key(&self) -> bool { false }
    fn rate_limit_per_minute(&self) -> u32 { 60 }

    async fn search(&self, client: &Client, query: &str, max_results: usize) -> OmniResult<Vec<SearchResult>> {
        let currency = query.to_uppercase();
        let url = format!("https://api.frankfurter.dev/v1/latest?base={}", urlencoding::encode(&currency));
        let resp = client.get(&url).send().await?;
        if !resp.status().is_success() { return Ok(vec![]); }
        let data: FrankResponse = resp.json().await?;

        Ok(data.rates.into_iter().take(max_results).enumerate().map(|(i, (code, rate))| {
            SearchResult {
                title: format!("{} to {} exchange rate", data.base, code),
                url: format!("https://www.frankfurter.app/latest?base={}&symbols={}", data.base, code),
                snippet: format!("1 {} = {:.4} {} (as of {})", data.base, rate, code, data.date),
                source: "frankfurter".into(),
                category: Category::Finance,
                published: None, author: None,
                relevance_score: 0.7 - (i as f64 * 0.02),
            }
        }).collect())
    }
}
