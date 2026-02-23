use crate::error::OmniResult;
use crate::sources::Source;
use crate::types::{Category, SearchResult};
use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;

pub struct PubMed;

#[derive(Deserialize)]
struct PmSearchResponse {
    esearchresult: PmSearchResult,
}

#[derive(Deserialize)]
struct PmSearchResult {
    #[serde(default)]
    idlist: Vec<String>,
}

#[derive(Deserialize)]
struct PmSummaryResponse {
    result: Option<serde_json::Value>,
}

#[async_trait]
impl Source for PubMed {
    fn name(&self) -> &str { "pubmed" }
    fn category(&self) -> Category { Category::Academic }
    fn requires_key(&self) -> bool { false }
    fn rate_limit_per_minute(&self) -> u32 { 10 }

    async fn search(&self, client: &Client, query: &str, max_results: usize) -> OmniResult<Vec<SearchResult>> {
        let limit = max_results.min(20);
        let search_url = format!(
            "https://eutils.ncbi.nlm.nih.gov/entrez/eutils/esearch.fcgi?db=pubmed&term={}&retmax={}&retmode=json",
            urlencoding::encode(query), limit
        );
        let search_resp: PmSearchResponse = client.get(&search_url).send().await?.json().await?;
        let ids = &search_resp.esearchresult.idlist;
        if ids.is_empty() { return Ok(vec![]); }

        let ids_str = ids.join(",");
        let summary_url = format!(
            "https://eutils.ncbi.nlm.nih.gov/entrez/eutils/esummary.fcgi?db=pubmed&id={}&retmode=json",
            ids_str
        );
        let summary_resp: PmSummaryResponse = client.get(&summary_url).send().await?.json().await?;

        let mut results = Vec::new();
        if let Some(serde_json::Value::Object(map)) = summary_resp.result {
            for (i, id) in ids.iter().enumerate() {
                if let Some(serde_json::Value::Object(article)) = map.get(id) {
                    let title = article.get("title").and_then(|v| v.as_str()).unwrap_or("").to_string();
                    let authors: Vec<String> = article.get("authors")
                        .and_then(|v| v.as_array())
                        .map(|arr| arr.iter().filter_map(|a| a.get("name").and_then(|n| n.as_str()).map(String::from)).collect())
                        .unwrap_or_default();
                    let source_journal = article.get("source").and_then(|v| v.as_str()).unwrap_or("");
                    let pubdate = article.get("pubdate").and_then(|v| v.as_str()).unwrap_or("");

                    results.push(SearchResult {
                        title,
                        url: format!("https://pubmed.ncbi.nlm.nih.gov/{}/", id),
                        snippet: format!("{} ({})", source_journal, pubdate),
                        source: "pubmed".into(),
                        category: Category::Academic,
                        published: None,
                        author: if authors.is_empty() { None } else { Some(authors.join(", ")) },
                        relevance_score: 0.85 - (i as f64 * 0.03),
                    });
                }
            }
        }
        Ok(results)
    }
}
