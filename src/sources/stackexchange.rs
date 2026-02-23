use crate::error::OmniResult;
use crate::sources::Source;
use crate::types::{Category, SearchResult};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::Deserialize;

pub struct StackExchange;

#[derive(Deserialize)]
struct SeResponse {
    items: Vec<SeQuestion>,
}

#[derive(Deserialize)]
struct SeQuestion {
    title: String,
    link: String,
    score: i32,
    answer_count: u32,
    creation_date: i64,
    owner: Option<SeOwner>,
    #[serde(default)]
    tags: Vec<String>,
}

#[derive(Deserialize)]
struct SeOwner {
    display_name: Option<String>,
}

#[async_trait]
impl Source for StackExchange {
    fn name(&self) -> &str {
        "stackexchange"
    }
    fn category(&self) -> Category {
        Category::Code
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
            "https://api.stackexchange.com/2.3/search/advanced?q={}&pagesize={}&order=desc&sort=relevance&site=stackoverflow",
            urlencoding::encode(query),
            limit
        );

        let resp: SeResponse = client.get(&url).send().await?.json().await?;

        let results = resp
            .items
            .into_iter()
            .enumerate()
            .map(|(i, q)| {
                let published = DateTime::<Utc>::from_timestamp(q.creation_date, 0);
                let tags_str = q.tags.join(", ");
                let snippet = format!(
                    "[{}] Score: {} | Answers: {} | Tags: {}",
                    if q.answer_count > 0 {
                        "answered"
                    } else {
                        "unanswered"
                    },
                    q.score,
                    q.answer_count,
                    tags_str
                );
                let author = q.owner.and_then(|o| o.display_name);
                let score_boost = (q.score as f64 / 100.0).min(0.3);

                SearchResult {
                    title: html_entities_decode(&q.title),
                    url: q.link,
                    snippet,
                    source: "stackexchange".to_string(),
                    category: Category::Code,
                    published,
                    author,
                    relevance_score: (0.85 - (i as f64 * 0.03) + score_boost).min(1.0),
                }
            })
            .collect();

        Ok(results)
    }
}

fn html_entities_decode(s: &str) -> String {
    s.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
}
