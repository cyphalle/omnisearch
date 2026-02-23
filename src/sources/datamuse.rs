use crate::error::OmniResult;
use crate::sources::Source;
use crate::types::{Category, SearchResult};
use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;

pub struct Datamuse;

#[derive(Deserialize)]
struct DmWord {
    word: String,
    score: Option<u64>,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default)]
    defs: Vec<String>,
}

#[async_trait]
impl Source for Datamuse {
    fn name(&self) -> &str { "datamuse" }
    fn category(&self) -> Category { Category::Reference }
    fn requires_key(&self) -> bool { false }
    fn rate_limit_per_minute(&self) -> u32 { 100 }

    async fn search(&self, client: &Client, query: &str, max_results: usize) -> OmniResult<Vec<SearchResult>> {
        let limit = max_results.min(50);
        let url = format!("https://api.datamuse.com/words?ml={}&max={}&md=d", urlencoding::encode(query), limit);
        let words: Vec<DmWord> = client.get(&url).send().await?.json().await?;

        Ok(words.into_iter().enumerate().map(|(i, w)| {
            let definition = w.defs.first().cloned().unwrap_or_else(|| format!("Related to: {}", query));
            let snippet = format!("Score: {} | Tags: {} | {}", w.score.unwrap_or(0), w.tags.join(", "), definition);
            SearchResult {
                title: w.word.clone(),
                url: format!("https://www.datamuse.com/search?q={}", urlencoding::encode(&w.word)),
                snippet, source: "datamuse".into(),
                category: Category::Reference,
                published: None, author: None,
                relevance_score: 0.7 - (i as f64 * 0.03),
            }
        }).collect())
    }
}
