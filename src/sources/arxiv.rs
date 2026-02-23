use crate::error::OmniResult;
use crate::sources::Source;
use crate::types::{Category, SearchResult};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::Deserialize;

pub struct Arxiv;

#[derive(Deserialize)]
struct ArxivFeed {
    #[serde(rename = "entry", default)]
    entries: Vec<ArxivEntry>,
}

#[derive(Deserialize)]
struct ArxivEntry {
    title: String,
    summary: String,
    id: String,
    published: String,
    #[serde(rename = "author", default)]
    authors: Vec<ArxivAuthor>,
}

#[derive(Deserialize)]
struct ArxivAuthor {
    name: String,
}

#[async_trait]
impl Source for Arxiv {
    fn name(&self) -> &str {
        "arxiv"
    }
    fn category(&self) -> Category {
        Category::Academic
    }
    fn requires_key(&self) -> bool {
        false
    }
    fn rate_limit_per_minute(&self) -> u32 {
        30
    }

    async fn search(
        &self,
        client: &Client,
        query: &str,
        max_results: usize,
    ) -> OmniResult<Vec<SearchResult>> {
        let limit = max_results.min(50);
        let url = format!(
            "https://export.arxiv.org/api/query?search_query=all:{}&start=0&max_results={}",
            urlencoding::encode(query),
            limit
        );

        let resp_text = client.get(&url).send().await?.text().await?;

        let feed: ArxivFeed = quick_xml::de::from_str(&resp_text).unwrap_or(ArxivFeed {
            entries: Vec::new(),
        });

        let results = feed
            .entries
            .into_iter()
            .enumerate()
            .map(|(i, entry)| {
                let published = entry
                    .published
                    .parse::<DateTime<Utc>>()
                    .ok();
                let authors = entry
                    .authors
                    .iter()
                    .map(|a| a.name.as_str())
                    .collect::<Vec<_>>()
                    .join(", ");
                let snippet = entry
                    .summary
                    .replace('\n', " ")
                    .chars()
                    .take(300)
                    .collect::<String>();

                SearchResult {
                    title: entry.title.replace('\n', " ").trim().to_string(),
                    url: entry.id,
                    snippet,
                    source: "arxiv".to_string(),
                    category: Category::Academic,
                    published,
                    author: if authors.is_empty() {
                        None
                    } else {
                        Some(authors)
                    },
                    relevance_score: 0.9 - (i as f64 * 0.03),
                }
            })
            .collect();

        Ok(results)
    }
}
