use crate::error::OmniResult;
use crate::sources::Source;
use crate::types::{Category, SearchResult};
use async_trait::async_trait;
use chrono::NaiveDate;
use reqwest::Client;
use serde::Deserialize;

pub struct CrossRef;

#[derive(Deserialize)]
struct CrResponse {
    message: CrMessage,
}

#[derive(Deserialize)]
struct CrMessage {
    items: Vec<CrItem>,
}

#[derive(Deserialize)]
struct CrItem {
    #[serde(rename = "DOI")]
    doi: String,
    #[serde(default)]
    title: Vec<String>,
    #[serde(default, rename = "abstract")]
    item_abstract: Option<String>,
    #[serde(default)]
    author: Vec<CrAuthor>,
    #[serde(default)]
    created: Option<CrDate>,
}

#[derive(Deserialize)]
struct CrAuthor {
    given: Option<String>,
    family: Option<String>,
}

#[derive(Deserialize)]
struct CrDate {
    #[serde(rename = "date-parts")]
    date_parts: Vec<Vec<i32>>,
}

#[async_trait]
impl Source for CrossRef {
    fn name(&self) -> &str { "crossref" }
    fn category(&self) -> Category { Category::Academic }
    fn requires_key(&self) -> bool { false }
    fn rate_limit_per_minute(&self) -> u32 { 50 }

    async fn search(&self, client: &Client, query: &str, max_results: usize) -> OmniResult<Vec<SearchResult>> {
        let limit = max_results.min(50);
        let url = format!(
            "https://api.crossref.org/works?query={}&rows={}&mailto=omnisearch@example.com",
            urlencoding::encode(query), limit
        );
        let resp: CrResponse = client.get(&url).send().await?.json().await?;
        Ok(resp.message.items.into_iter().enumerate().map(|(i, item)| {
            let title = item.title.first().cloned().unwrap_or_else(|| "Untitled".into());
            let authors: Vec<String> = item.author.iter().map(|a| {
                format!("{} {}", a.given.as_deref().unwrap_or(""), a.family.as_deref().unwrap_or("")).trim().to_string()
            }).collect();
            let published = item.created.and_then(|d| {
                let parts = d.date_parts.first()?;
                let y = *parts.first()?;
                let m = parts.get(1).copied().unwrap_or(1) as u32;
                let d = parts.get(2).copied().unwrap_or(1) as u32;
                NaiveDate::from_ymd_opt(y, m, d).map(|nd| nd.and_hms_opt(0,0,0).unwrap().and_utc())
            });
            let snippet = item.item_abstract.unwrap_or_default().replace("<jats:p>","").replace("</jats:p>","").chars().take(300).collect();
            SearchResult {
                title,
                url: format!("https://doi.org/{}", item.doi),
                snippet, source: "crossref".into(),
                category: Category::Academic,
                published,
                author: if authors.is_empty() { None } else { Some(authors.join(", ")) },
                relevance_score: 0.85 - (i as f64 * 0.03),
            }
        }).collect())
    }
}
