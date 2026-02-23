use crate::error::OmniResult;
use crate::sources::Source;
use crate::types::{Category, SearchResult};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::Deserialize;

pub struct YouTube { pub api_key: String }

#[derive(Deserialize)]
struct YtResponse { items: Option<Vec<YtItem>> }
#[derive(Deserialize)]
struct YtItem { id: YtId, snippet: Option<YtSnippet> }
#[derive(Deserialize)]
struct YtId { #[serde(rename = "videoId")] video_id: Option<String> }
#[derive(Deserialize)]
struct YtSnippet { title: String, description: String, #[serde(rename = "publishedAt")] published_at: Option<String>, #[serde(rename = "channelTitle")] channel_title: Option<String> }

#[async_trait]
impl Source for YouTube {
    fn name(&self) -> &str { "youtube" }
    fn category(&self) -> Category { Category::Media }
    fn requires_key(&self) -> bool { true }
    fn rate_limit_per_minute(&self) -> u32 { 30 }

    async fn search(&self, client: &Client, query: &str, max_results: usize) -> OmniResult<Vec<SearchResult>> {
        let limit = max_results.min(50);
        let url = format!("https://www.googleapis.com/youtube/v3/search?part=snippet&q={}&maxResults={}&type=video&key={}", urlencoding::encode(query), limit, self.api_key);
        let data: YtResponse = client.get(&url).send().await?.json().await?;
        Ok(data.items.unwrap_or_default().into_iter().enumerate().filter_map(|(i, item)| {
            let snippet = item.snippet?;
            let video_id = item.id.video_id?;
            Some(SearchResult {
                title: snippet.title, url: format!("https://www.youtube.com/watch?v={video_id}"),
                snippet: snippet.description.chars().take(300).collect(),
                source: "youtube".into(), category: Category::Media,
                published: snippet.published_at.and_then(|d| d.parse::<DateTime<Utc>>().ok()),
                author: snippet.channel_title,
                relevance_score: 0.9 - (i as f64 * 0.03),
            })
        }).collect())
    }
}
