use crate::error::OmniResult;
use crate::sources::Source;
use crate::types::{Category, SearchResult};
use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;

pub struct Wikipedia;

#[derive(Deserialize)]
struct WikiResponse {
    query: Option<WikiQuery>,
}

#[derive(Deserialize)]
struct WikiQuery {
    search: Vec<WikiSearchResult>,
}

#[derive(Deserialize)]
struct WikiSearchResult {
    title: String,
    #[allow(dead_code)]
    pageid: u64,
    snippet: String,
}

#[async_trait]
impl Source for Wikipedia {
    fn name(&self) -> &str {
        "wikipedia"
    }
    fn category(&self) -> Category {
        Category::Reference
    }
    fn requires_key(&self) -> bool {
        false
    }
    fn rate_limit_per_minute(&self) -> u32 {
        200
    }

    async fn search(
        &self,
        client: &Client,
        query: &str,
        max_results: usize,
    ) -> OmniResult<Vec<SearchResult>> {
        let limit = max_results.min(50);
        let url = format!(
            "https://en.wikipedia.org/w/api.php?action=query&list=search&srsearch={}&srlimit={}&format=json",
            urlencoding::encode(query),
            limit
        );

        let resp: WikiResponse = client.get(&url).send().await?.json().await?;

        let results = resp
            .query
            .map(|q| {
                q.search
                    .into_iter()
                    .enumerate()
                    .map(|(i, r)| {
                        let snippet = r
                            .snippet
                            .replace("<span class=\"searchmatch\">", "")
                            .replace("</span>", "");
                        SearchResult {
                            title: r.title.clone(),
                            url: format!(
                                "https://en.wikipedia.org/wiki/{}",
                                r.title.replace(' ', "_")
                            ),
                            snippet,
                            source: "wikipedia".to_string(),
                            category: Category::Reference,
                            published: None,
                            author: None,
                            relevance_score: 1.0 - (i as f64 * 0.05),
                        }
                    })
                    .collect()
            })
            .unwrap_or_default();

        Ok(results)
    }
}
