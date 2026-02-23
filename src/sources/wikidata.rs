use crate::error::OmniResult;
use crate::sources::Source;
use crate::types::{Category, SearchResult};
use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;

pub struct Wikidata;

#[derive(Deserialize)]
struct WdResponse {
    search: Vec<WdEntity>,
}

#[derive(Deserialize)]
struct WdEntity {
    id: String,
    label: String,
    description: Option<String>,
    #[serde(rename = "concepturi")]
    concept_uri: Option<String>,
}

#[async_trait]
impl Source for Wikidata {
    fn name(&self) -> &str { "wikidata" }
    fn category(&self) -> Category { Category::Reference }
    fn requires_key(&self) -> bool { false }
    fn rate_limit_per_minute(&self) -> u32 { 200 }

    async fn search(&self, client: &Client, query: &str, max_results: usize) -> OmniResult<Vec<SearchResult>> {
        let limit = max_results.min(50);
        let url = format!(
            "https://www.wikidata.org/w/api.php?action=wbsearchentities&search={}&language=en&limit={}&format=json",
            urlencoding::encode(query), limit
        );
        let resp: WdResponse = client.get(&url).send().await?.json().await?;
        Ok(resp.search.into_iter().enumerate().map(|(i, e)| {
            SearchResult {
                title: e.label,
                url: e.concept_uri.unwrap_or_else(|| format!("https://www.wikidata.org/wiki/{}", e.id)),
                snippet: e.description.unwrap_or_default(),
                source: "wikidata".into(),
                category: Category::Reference,
                published: None, author: None,
                relevance_score: 0.8 - (i as f64 * 0.05),
            }
        }).collect())
    }
}
