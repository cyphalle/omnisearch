use crate::error::OmniResult;
use crate::sources::Source;
use crate::types::{Category, SearchResult};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::Deserialize;

pub struct HackerNews;

#[derive(Deserialize)]
struct HnResponse {
    hits: Vec<HnHit>,
}

#[derive(Deserialize)]
struct HnHit {
    title: Option<String>,
    url: Option<String>,
    #[serde(rename = "objectID")]
    object_id: String,
    author: Option<String>,
    #[serde(rename = "story_text")]
    story_text: Option<String>,
    #[serde(rename = "comment_text")]
    comment_text: Option<String>,
    created_at: Option<String>,
    points: Option<u32>,
}

#[async_trait]
impl Source for HackerNews {
    fn name(&self) -> &str {
        "hacker_news"
    }
    fn category(&self) -> Category {
        Category::Code
    }
    fn requires_key(&self) -> bool {
        false
    }
    fn rate_limit_per_minute(&self) -> u32 {
        60
    }

    async fn search(
        &self,
        client: &Client,
        query: &str,
        max_results: usize,
    ) -> OmniResult<Vec<SearchResult>> {
        let limit = max_results.min(50);
        let url = format!(
            "https://hn.algolia.com/api/v1/search?query={}&hitsPerPage={}&tags=story",
            urlencoding::encode(query),
            limit
        );

        let resp: HnResponse = client.get(&url).send().await?.json().await?;

        let results = resp
            .hits
            .into_iter()
            .enumerate()
            .filter_map(|(i, hit)| {
                let title = hit.title?;
                let url = hit.url.unwrap_or_else(|| {
                    format!("https://news.ycombinator.com/item?id={}", hit.object_id)
                });
                let snippet = hit
                    .story_text
                    .or(hit.comment_text)
                    .unwrap_or_default()
                    .replace("<p>", " ")
                    .replace("</p>", "")
                    .chars()
                    .take(300)
                    .collect::<String>();

                let published = hit
                    .created_at
                    .and_then(|d| d.parse::<DateTime<Utc>>().ok());

                let points_boost = hit.points.unwrap_or(0) as f64 / 1000.0;

                Some(SearchResult {
                    title,
                    url,
                    snippet,
                    source: "hacker_news".to_string(),
                    category: Category::Code,
                    published,
                    author: hit.author,
                    relevance_score: (0.85 - (i as f64 * 0.03) + points_boost).min(1.0),
                })
            })
            .collect();

        Ok(results)
    }
}
