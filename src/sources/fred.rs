use crate::error::OmniResult;
use crate::sources::Source;
use crate::types::{Category, SearchResult};
use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;

pub struct Fred { pub api_key: String }

#[derive(Deserialize)]
struct FredResponse { seriess: Option<Vec<FredSeries>> }
#[derive(Deserialize)]
struct FredSeries { id: String, title: String, notes: Option<String>, frequency: Option<String>, units: Option<String> }

#[async_trait]
impl Source for Fred {
    fn name(&self) -> &str { "fred" }
    fn category(&self) -> Category { Category::Finance }
    fn requires_key(&self) -> bool { true }
    fn rate_limit_per_minute(&self) -> u32 { 60 }

    async fn search(&self, client: &Client, query: &str, max_results: usize) -> OmniResult<Vec<SearchResult>> {
        let limit = max_results.min(50);
        let url = format!("https://api.stlouisfed.org/fred/series/search?search_text={}&limit={}&api_key={}&file_type=json", urlencoding::encode(query), limit, self.api_key);
        let data: FredResponse = client.get(&url).send().await?.json().await?;
        Ok(data.seriess.unwrap_or_default().into_iter().enumerate().map(|(i, s)| {
            SearchResult {
                title: format!("{} ({})", s.title, s.id),
                url: format!("https://fred.stlouisfed.org/series/{}", s.id),
                snippet: format!("Frequency: {} | Units: {} | {}", s.frequency.unwrap_or("N/A".into()), s.units.unwrap_or("N/A".into()), s.notes.unwrap_or_default().chars().take(200).collect::<String>()),
                source: "fred".into(), category: Category::Finance,
                published: None, author: None,
                relevance_score: 0.8 - (i as f64 * 0.03),
            }
        }).collect())
    }
}
