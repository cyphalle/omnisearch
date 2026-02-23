use crate::error::OmniResult;
use crate::sources::Source;
use crate::types::{Category, SearchResult};
use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;

pub struct VirusTotal { pub api_key: String }

#[derive(Deserialize)]
struct VtResponse { data: Option<Vec<VtItem>> }
#[derive(Deserialize)]
struct VtItem { id: String, #[serde(rename = "type")] item_type: String, attributes: Option<serde_json::Value> }

#[async_trait]
impl Source for VirusTotal {
    fn name(&self) -> &str { "virustotal" }
    fn category(&self) -> Category { Category::Security }
    fn requires_key(&self) -> bool { true }
    fn rate_limit_per_minute(&self) -> u32 { 4 }

    async fn search(&self, client: &Client, query: &str, max_results: usize) -> OmniResult<Vec<SearchResult>> {
        let url = format!("https://www.virustotal.com/api/v3/search?query={}&limit={}", urlencoding::encode(query), max_results.min(20));
        let data: VtResponse = client.get(&url).header("x-apikey", &self.api_key).send().await?.json().await?;
        Ok(data.data.unwrap_or_default().into_iter().enumerate().map(|(i, item)| {
            let snippet = item.attributes.as_ref().and_then(|a| {
                let malicious = a.get("last_analysis_stats").and_then(|s| s.get("malicious")).and_then(|v| v.as_u64()).unwrap_or(0);
                let total = a.get("last_analysis_stats").and_then(|s| s.get("undetected")).and_then(|v| v.as_u64()).unwrap_or(0) + malicious;
                Some(format!("Type: {} | Detections: {}/{}", item.item_type, malicious, total))
            }).unwrap_or_else(|| format!("Type: {}", item.item_type));
            SearchResult {
                title: item.id.clone(), url: format!("https://www.virustotal.com/gui/search/{}", item.id),
                snippet, source: "virustotal".into(), category: Category::Security,
                published: None, author: None,
                relevance_score: 0.8 - (i as f64 * 0.05),
            }
        }).collect())
    }
}
