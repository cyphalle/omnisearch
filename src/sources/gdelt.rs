use crate::error::OmniResult;
use crate::sources::Source;
use crate::types::{Category, SearchResult};
use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;

pub struct Gdelt;

#[derive(Deserialize)]
struct GdeltResponse {
    articles: Option<Vec<GdeltArticle>>,
}

#[derive(Deserialize)]
struct GdeltArticle {
    title: String,
    url: String,
    #[serde(rename = "seendate")]
    seen_date: String,
    domain: String,
    socialimage: Option<String>,
}

#[async_trait]
impl Source for Gdelt {
    fn name(&self) -> &str { "gdelt" }
    fn category(&self) -> Category { Category::News }
    fn requires_key(&self) -> bool { false }
    fn rate_limit_per_minute(&self) -> u32 { 60 }

    async fn search(&self, client: &Client, query: &str, max_results: usize) -> OmniResult<Vec<SearchResult>> {
        let limit = max_results.min(250);
        let url = format!(
            "https://api.gdeltproject.org/api/v2/doc/doc?query={}&mode=ArtList&maxrecords={}&format=json",
            urlencoding::encode(query), limit
        );
        let resp: GdeltResponse = client.get(&url).send().await?.json().await?;
        Ok(resp.articles.unwrap_or_default().into_iter().enumerate().map(|(i, a)| {
            let published = chrono::NaiveDateTime::parse_from_str(&a.seen_date, "%Y%m%dT%H%M%S").ok().map(|dt| dt.and_utc());
            SearchResult {
                title: a.title,
                url: a.url,
                snippet: format!("Source: {}", a.domain),
                source: "gdelt".into(),
                category: Category::News,
                published,
                author: None,
                relevance_score: 0.8 - (i as f64 * 0.02),
            }
        }).collect())
    }
}
