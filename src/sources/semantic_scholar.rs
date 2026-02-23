use crate::error::OmniResult;
use crate::sources::Source;
use crate::types::{Category, SearchResult};
use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;

pub struct SemanticScholar { pub api_key: String }

#[derive(Deserialize)]
struct SsResponse { data: Option<Vec<SsPaper>> }
#[derive(Deserialize)]
struct SsPaper { #[serde(rename = "paperId")] paper_id: String, title: Option<String>, #[serde(rename = "abstract")] paper_abstract: Option<String>, year: Option<u32>, #[serde(rename = "citationCount")] citation_count: Option<u32>, authors: Option<Vec<SsAuthor>> }
#[derive(Deserialize)]
struct SsAuthor { name: Option<String> }

#[async_trait]
impl Source for SemanticScholar {
    fn name(&self) -> &str { "semantic_scholar" }
    fn category(&self) -> Category { Category::Academic }
    fn requires_key(&self) -> bool { true }
    fn rate_limit_per_minute(&self) -> u32 { 100 }

    async fn search(&self, client: &Client, query: &str, max_results: usize) -> OmniResult<Vec<SearchResult>> {
        let limit = max_results.min(100);
        let url = format!("https://api.semanticscholar.org/graph/v1/paper/search?query={}&limit={}&fields=title,abstract,year,citationCount,authors", urlencoding::encode(query), limit);
        let data: SsResponse = client.get(&url).header("x-api-key", &self.api_key).send().await?.json().await?;
        Ok(data.data.unwrap_or_default().into_iter().enumerate().map(|(i, p)| {
            let authors: Vec<String> = p.authors.unwrap_or_default().iter().filter_map(|a| a.name.clone()).collect();
            let cite_boost = (p.citation_count.unwrap_or(0) as f64 / 1000.0).min(0.3);
            SearchResult {
                title: p.title.unwrap_or_default(),
                url: format!("https://www.semanticscholar.org/paper/{}", p.paper_id),
                snippet: format!("Year: {} | Citations: {} | {}", p.year.map(|y| y.to_string()).unwrap_or("N/A".into()), p.citation_count.unwrap_or(0), p.paper_abstract.unwrap_or_default().chars().take(250).collect::<String>()),
                source: "semantic_scholar".into(), category: Category::Academic,
                published: None,
                author: if authors.is_empty() { None } else { Some(authors.join(", ")) },
                relevance_score: (0.9 - (i as f64 * 0.02) + cite_boost).min(1.0),
            }
        }).collect())
    }
}
