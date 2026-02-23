use crate::error::OmniResult;
use crate::sources::Source;
use crate::types::{Category, SearchResult};
use async_trait::async_trait;
use chrono::NaiveDate;
use reqwest::Client;
use serde::Deserialize;

pub struct OpenAlex;

#[derive(Deserialize)]
struct OaResponse {
    results: Vec<OaWork>,
}

#[derive(Deserialize)]
struct OaWork {
    title: Option<String>,
    doi: Option<String>,
    publication_date: Option<String>,
    #[serde(default)]
    authorships: Vec<OaAuthorship>,
    #[serde(rename = "abstract_inverted_index")]
    abstract_index: Option<serde_json::Value>,
}

#[derive(Deserialize)]
struct OaAuthorship {
    author: OaAuthor,
}

#[derive(Deserialize)]
struct OaAuthor {
    display_name: Option<String>,
}

#[async_trait]
impl Source for OpenAlex {
    fn name(&self) -> &str { "openalex" }
    fn category(&self) -> Category { Category::Academic }
    fn requires_key(&self) -> bool { false }
    fn rate_limit_per_minute(&self) -> u32 { 100 }

    async fn search(&self, client: &Client, query: &str, max_results: usize) -> OmniResult<Vec<SearchResult>> {
        let limit = max_results.min(50);
        let url = format!(
            "https://api.openalex.org/works?search={}&per_page={}&mailto=omnisearch@example.com",
            urlencoding::encode(query), limit
        );
        let resp: OaResponse = client.get(&url).send().await?.json().await?;
        Ok(resp.results.into_iter().enumerate().map(|(i, w)| {
            let title = w.title.unwrap_or_else(|| "Untitled".into());
            let url = w.doi.unwrap_or_default();
            let authors: Vec<String> = w.authorships.iter().filter_map(|a| a.author.display_name.clone()).collect();
            let published = w.publication_date.and_then(|d| NaiveDate::parse_from_str(&d, "%Y-%m-%d").ok()).map(|d| d.and_hms_opt(0,0,0).unwrap().and_utc());
            SearchResult {
                title, url,
                snippet: reconstruct_abstract(&w.abstract_index),
                source: "openalex".into(),
                category: Category::Academic,
                published,
                author: if authors.is_empty() { None } else { Some(authors.join(", ")) },
                relevance_score: 0.85 - (i as f64 * 0.03),
            }
        }).collect())
    }
}

fn reconstruct_abstract(index: &Option<serde_json::Value>) -> String {
    let Some(serde_json::Value::Object(map)) = index else { return String::new() };
    let pairs: Vec<(String, Vec<usize>)> = map.iter().filter_map(|(word, positions)| {
        let pos: Vec<usize> = match positions {
            serde_json::Value::Array(arr) => arr.iter().filter_map(|v| v.as_u64().map(|n| n as usize)).collect(),
            _ => return None,
        };
        Some((word.clone(), pos))
    }).collect();
    let max_pos = pairs.iter().flat_map(|(_, ps)| ps.iter()).max().copied().unwrap_or(0);
    let mut words = vec![String::new(); max_pos + 1];
    for (word, positions) in &pairs {
        for &pos in positions {
            if pos < words.len() { words[pos] = word.clone(); }
        }
    }
    words.join(" ").chars().take(300).collect()
}
