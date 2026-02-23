use crate::error::OmniResult;
use crate::sources::Source;
use crate::types::{Category, SearchResult};
use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;

pub struct DBpedia;

#[derive(Deserialize)]
struct DbpResponse {
    results: Vec<DbpResult>,
}

#[derive(Deserialize)]
struct DbpResult {
    label: String,
    uri: String,
    description: String,
}

#[async_trait]
impl Source for DBpedia {
    fn name(&self) -> &str { "dbpedia" }
    fn category(&self) -> Category { Category::Reference }
    fn requires_key(&self) -> bool { false }
    fn rate_limit_per_minute(&self) -> u32 { 60 }

    async fn search(&self, client: &Client, query: &str, max_results: usize) -> OmniResult<Vec<SearchResult>> {
        let limit = max_results.min(20);
        let url = format!(
            "https://lookup.dbpedia.org/api/search?query={}&maxResults={}&format=json",
            urlencoding::encode(query), limit
        );
        let resp = client.get(&url).header("Accept", "application/json").send().await?;
        if !resp.status().is_success() { return Ok(vec![]); }
        let data: DbpResponse = resp.json().await.unwrap_or(DbpResponse { results: vec![] });
        Ok(data.results.into_iter().enumerate().map(|(i, r)| {
            SearchResult {
                title: r.label,
                url: r.uri,
                snippet: r.description.chars().take(300).collect(),
                source: "dbpedia".into(),
                category: Category::Reference,
                published: None, author: None,
                relevance_score: 0.75 - (i as f64 * 0.03),
            }
        }).collect())
    }
}
