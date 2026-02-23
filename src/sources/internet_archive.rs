use crate::error::OmniResult;
use crate::sources::Source;
use crate::types::{Category, SearchResult};
use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;

pub struct InternetArchive;

#[derive(Deserialize)]
struct IaResponse {
    response: IaResponseInner,
}

#[derive(Deserialize)]
struct IaResponseInner {
    docs: Vec<IaDoc>,
}

#[derive(Deserialize)]
struct IaDoc {
    identifier: String,
    title: Option<String>,
    description: Option<serde_json::Value>,
    creator: Option<serde_json::Value>,
    date: Option<String>,
    mediatype: Option<String>,
}

#[async_trait]
impl Source for InternetArchive {
    fn name(&self) -> &str { "internet_archive" }
    fn category(&self) -> Category { Category::General }
    fn requires_key(&self) -> bool { false }
    fn rate_limit_per_minute(&self) -> u32 { 60 }

    async fn search(&self, client: &Client, query: &str, max_results: usize) -> OmniResult<Vec<SearchResult>> {
        let limit = max_results.min(50);
        let url = format!(
            "https://archive.org/advancedsearch.php?q={}&output=json&rows={}&fl[]=identifier,title,description,creator,date,mediatype",
            urlencoding::encode(query), limit
        );
        let resp: IaResponse = client.get(&url).send().await?.json().await?;
        Ok(resp.response.docs.into_iter().enumerate().map(|(i, doc)| {
            let title = doc.title.unwrap_or_else(|| doc.identifier.clone());
            let desc = match doc.description {
                Some(serde_json::Value::String(s)) => s,
                Some(serde_json::Value::Array(arr)) => arr.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>().join(" "),
                _ => String::new(),
            };
            let creator = match doc.creator {
                Some(serde_json::Value::String(s)) => Some(s),
                Some(serde_json::Value::Array(arr)) => Some(arr.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>().join(", ")),
                _ => None,
            };
            let snippet = format!("{} | Type: {}", desc.chars().take(250).collect::<String>(), doc.mediatype.unwrap_or("unknown".into()));
            SearchResult {
                title,
                url: format!("https://archive.org/details/{}", doc.identifier),
                snippet, source: "internet_archive".into(),
                category: Category::General,
                published: None, author: creator,
                relevance_score: 0.75 - (i as f64 * 0.03),
            }
        }).collect())
    }
}
