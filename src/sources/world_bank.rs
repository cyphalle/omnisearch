use crate::error::OmniResult;
use crate::sources::Source;
use crate::types::{Category, SearchResult};
use async_trait::async_trait;
use reqwest::Client;

pub struct WorldBank;

#[async_trait]
impl Source for WorldBank {
    fn name(&self) -> &str { "world_bank" }
    fn category(&self) -> Category { Category::Finance }
    fn requires_key(&self) -> bool { false }
    fn rate_limit_per_minute(&self) -> u32 { 60 }

    async fn search(&self, client: &Client, query: &str, max_results: usize) -> OmniResult<Vec<SearchResult>> {
        let limit = max_results.min(50);
        let url = format!(
            "https://search.worldbank.org/api/v2/wds?qterm={}&rows={}&format=json",
            urlencoding::encode(query), limit
        );
        let resp = client.get(&url).send().await?;
        let text = resp.text().await?;
        let json: serde_json::Value = serde_json::from_str(&text).unwrap_or(serde_json::Value::Null);

        let mut results = Vec::new();
        if let Some(documents) = json.get("documents") {
            if let Some(obj) = documents.as_object() {
                for (i, (key, doc)) in obj.iter().enumerate() {
                    if key == "facets" || i >= max_results { continue; }
                    let title = doc.get("display_title").or(doc.get("title")).and_then(|v| v.as_str()).unwrap_or("Untitled").to_string();
                    let url = doc.get("url").and_then(|v| v.as_str()).unwrap_or("").to_string();
                    let snippet = doc.get("abstracts").and_then(|v| v.as_str()).unwrap_or("").chars().take(300).collect();
                    let author = doc.get("authors").and_then(|v| v.as_str()).map(String::from);
                    results.push(SearchResult {
                        title, url, snippet,
                        source: "world_bank".into(),
                        category: Category::Finance,
                        published: None, author,
                        relevance_score: 0.75 - (results.len() as f64 * 0.03),
                    });
                }
            }
        }
        Ok(results)
    }
}
