use crate::error::OmniResult;
use crate::sources::Source;
use crate::types::{Category, SearchResult};
use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;

pub struct Omdb { pub api_key: String }

#[derive(Deserialize)]
struct OmdbResponse { #[serde(rename = "Search")] search: Option<Vec<OmdbItem>> }
#[derive(Deserialize)]
struct OmdbItem { #[serde(rename = "Title")] title: String, #[serde(rename = "Year")] year: String, #[serde(rename = "imdbID")] imdb_id: String, #[serde(rename = "Type")] item_type: String, #[serde(rename = "Poster")] poster: Option<String> }

#[async_trait]
impl Source for Omdb {
    fn name(&self) -> &str { "omdb" }
    fn category(&self) -> Category { Category::Media }
    fn requires_key(&self) -> bool { true }
    fn rate_limit_per_minute(&self) -> u32 { 60 }

    async fn search(&self, client: &Client, query: &str, max_results: usize) -> OmniResult<Vec<SearchResult>> {
        let url = format!("https://www.omdbapi.com/?s={}&apikey={}", urlencoding::encode(query), urlencoding::encode(&self.api_key));
        let data: OmdbResponse = client.get(&url).send().await?.json().await?;
        Ok(data.search.unwrap_or_default().into_iter().take(max_results).enumerate().map(|(i, item)| {
            SearchResult {
                title: format!("{} ({})", item.title, item.year),
                url: format!("https://www.imdb.com/title/{}/", item.imdb_id),
                snippet: format!("Type: {}", item.item_type),
                source: "omdb".into(), category: Category::Media,
                published: None, author: None,
                relevance_score: 0.8 - (i as f64 * 0.03),
            }
        }).collect())
    }
}
