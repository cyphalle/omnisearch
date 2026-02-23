use crate::error::OmniResult;
use crate::sources::Source;
use crate::types::{Category, SearchResult};
use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;

pub struct Tmdb { pub api_key: String }

#[derive(Deserialize)]
struct TmdbResponse { results: Option<Vec<TmdbItem>> }
#[derive(Deserialize)]
struct TmdbItem { id: u64, title: Option<String>, name: Option<String>, overview: Option<String>, release_date: Option<String>, first_air_date: Option<String>, media_type: Option<String> }

#[async_trait]
impl Source for Tmdb {
    fn name(&self) -> &str { "tmdb" }
    fn category(&self) -> Category { Category::Media }
    fn requires_key(&self) -> bool { true }
    fn rate_limit_per_minute(&self) -> u32 { 40 }

    async fn search(&self, client: &Client, query: &str, max_results: usize) -> OmniResult<Vec<SearchResult>> {
        let url = format!("https://api.themoviedb.org/3/search/multi?query={}&api_key={}", urlencoding::encode(query), self.api_key);
        let data: TmdbResponse = client.get(&url).send().await?.json().await?;
        Ok(data.results.unwrap_or_default().into_iter().take(max_results).enumerate().map(|(i, item)| {
            let title = item.title.or(item.name).unwrap_or_default();
            let media_type = item.media_type.unwrap_or("unknown".into());
            let date = item.release_date.or(item.first_air_date).unwrap_or_default();
            let path = match media_type.as_str() { "movie" => "movie", "tv" => "tv", _ => "movie" };
            SearchResult {
                title, url: format!("https://www.themoviedb.org/{}/{}", path, item.id),
                snippet: format!("[{}] {} | {}", media_type, date, item.overview.unwrap_or_default().chars().take(250).collect::<String>()),
                source: "tmdb".into(), category: Category::Media,
                published: None, author: None,
                relevance_score: 0.85 - (i as f64 * 0.03),
            }
        }).collect())
    }
}
