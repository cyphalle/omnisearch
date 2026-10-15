use crate::error::OmniResult;
use crate::sources::Source;
use crate::types::{Category, SearchResult};
use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;

pub struct WolframAlpha { pub api_key: String }

#[derive(Deserialize)]
struct WaResponse { queryresult: WaQueryResult }
#[derive(Deserialize)]
struct WaQueryResult { success: bool, pods: Option<Vec<WaPod>> }
#[derive(Deserialize)]
struct WaPod { title: String, subpods: Vec<WaSubpod> }
#[derive(Deserialize)]
struct WaSubpod { plaintext: Option<String> }

#[async_trait]
impl Source for WolframAlpha {
    fn name(&self) -> &str { "wolfram_alpha" }
    fn category(&self) -> Category { Category::Reference }
    fn requires_key(&self) -> bool { true }
    fn rate_limit_per_minute(&self) -> u32 { 60 }

    async fn search(&self, client: &Client, query: &str, max_results: usize) -> OmniResult<Vec<SearchResult>> {
        let url = format!("https://api.wolframalpha.com/v2/query?input={}&format=plaintext&output=json&appid={}", urlencoding::encode(query), urlencoding::encode(&self.api_key));
        let data: WaResponse = client.get(&url).send().await?.json().await?;
        if !data.queryresult.success { return Ok(vec![]); }
        Ok(data.queryresult.pods.unwrap_or_default().into_iter().take(max_results).enumerate().map(|(i, pod)| {
            let text = pod.subpods.iter().filter_map(|s| s.plaintext.as_ref()).cloned().collect::<Vec<_>>().join("\n");
            SearchResult {
                title: pod.title,
                url: format!("https://www.wolframalpha.com/input?i={}", urlencoding::encode(query)),
                snippet: text.chars().take(500).collect(),
                source: "wolfram_alpha".into(), category: Category::Reference,
                published: None, author: None,
                relevance_score: 0.95 - (i as f64 * 0.05),
            }
        }).collect())
    }
}
