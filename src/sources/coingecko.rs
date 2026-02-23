use crate::error::OmniResult;
use crate::sources::Source;
use crate::types::{Category, SearchResult};
use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;

pub struct CoinGecko { pub api_key: String }

#[derive(Deserialize)]
struct CgResponse { coins: Option<Vec<CgCoin>> }
#[derive(Deserialize)]
struct CgCoin { id: String, name: String, symbol: String, market_cap_rank: Option<u64>, thumb: Option<String> }

#[async_trait]
impl Source for CoinGecko {
    fn name(&self) -> &str { "coingecko" }
    fn category(&self) -> Category { Category::Finance }
    fn requires_key(&self) -> bool { true }
    fn rate_limit_per_minute(&self) -> u32 { 30 }

    async fn search(&self, client: &Client, query: &str, max_results: usize) -> OmniResult<Vec<SearchResult>> {
        let url = format!("https://api.coingecko.com/api/v3/search?query={}", urlencoding::encode(query));
        let data: CgResponse = client.get(&url).header("x-cg-demo-api-key", &self.api_key).send().await?.json().await?;
        Ok(data.coins.unwrap_or_default().into_iter().take(max_results).enumerate().map(|(i, c)| {
            SearchResult {
                title: format!("{} ({})", c.name, c.symbol.to_uppercase()),
                url: format!("https://www.coingecko.com/en/coins/{}", c.id),
                snippet: format!("Market Cap Rank: #{}", c.market_cap_rank.map(|r| r.to_string()).unwrap_or("N/A".into())),
                source: "coingecko".into(), category: Category::Finance,
                published: None, author: None,
                relevance_score: 0.8 - (i as f64 * 0.05),
            }
        }).collect())
    }
}
