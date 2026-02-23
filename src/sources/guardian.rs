use crate::error::OmniResult;
use crate::sources::Source;
use crate::types::{Category, SearchResult};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::Deserialize;

pub struct Guardian { pub api_key: String }

#[derive(Deserialize)]
struct GResponse { response: GInner }
#[derive(Deserialize)]
struct GInner { results: Vec<GArticle> }
#[derive(Deserialize)]
struct GArticle {
    #[serde(rename = "webTitle")]
    web_title: String,
    #[serde(rename = "webUrl")]
    web_url: String,
    #[serde(rename = "sectionName")]
    section_name: Option<String>,
    #[serde(rename = "webPublicationDate")]
    web_publication_date: Option<String>,
}

#[async_trait]
impl Source for Guardian {
    fn name(&self) -> &str { "guardian" }
    fn category(&self) -> Category { Category::News }
    fn requires_key(&self) -> bool { true }
    fn rate_limit_per_minute(&self) -> u32 { 12 }

    async fn search(&self, client: &Client, query: &str, max_results: usize) -> OmniResult<Vec<SearchResult>> {
        let limit = max_results.min(50);
        let url = format!("https://content.guardianapis.com/search?q={}&page-size={}&api-key={}", urlencoding::encode(query), limit, self.api_key);
        let data: GResponse = client.get(&url).send().await?.json().await?;
        Ok(data.response.results.into_iter().enumerate().map(|(i, a)| {
            SearchResult {
                title: a.web_title, url: a.web_url,
                snippet: format!("Section: {}", a.section_name.unwrap_or("N/A".into())),
                source: "guardian".into(), category: Category::News,
                published: a.web_publication_date.and_then(|d| d.parse::<DateTime<Utc>>().ok()),
                author: None,
                relevance_score: 0.9 - (i as f64 * 0.02),
            }
        }).collect())
    }
}
