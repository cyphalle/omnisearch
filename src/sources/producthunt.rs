use crate::error::OmniResult;
use crate::sources::Source;
use crate::types::{Category, SearchResult};
use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;

pub struct ProductHunt {
    pub api_key: String,
}

#[derive(Deserialize)]
struct PhGraphQLResponse {
    data: Option<PhData>,
}

#[derive(Deserialize)]
struct PhData {
    posts: Option<PhPosts>,
}

#[derive(Deserialize)]
struct PhPosts {
    edges: Vec<PhEdge>,
}

#[derive(Deserialize)]
struct PhEdge {
    node: PhNode,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct PhNode {
    name: Option<String>,
    tagline: Option<String>,
    url: Option<String>,
    votes_count: Option<u64>,
    #[serde(rename = "createdAt")]
    created_at: Option<String>,
}

#[async_trait]
impl Source for ProductHunt {
    fn name(&self) -> &str {
        "producthunt"
    }
    fn category(&self) -> Category {
        Category::Market
    }
    fn requires_key(&self) -> bool {
        true
    }
    fn rate_limit_per_minute(&self) -> u32 {
        10
    }

    async fn search(
        &self,
        client: &Client,
        query: &str,
        max_results: usize,
    ) -> OmniResult<Vec<SearchResult>> {
        let limit = max_results.min(20);
        let graphql_query = format!(
            r#"{{ posts(first: {limit}, order: VOTES, topic: "{query}") {{ edges {{ node {{ name tagline url votesCount createdAt }} }} }} }}"#,
            limit = limit,
            query = query.replace('"', "\\\""),
        );

        let body = serde_json::json!({ "query": graphql_query });

        let resp: PhGraphQLResponse = client
            .post("https://api.producthunt.com/v2/api/graphql")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&body)
            .send()
            .await?
            .json()
            .await?;

        let edges = resp
            .data
            .and_then(|d| d.posts)
            .map(|p| p.edges)
            .unwrap_or_default();

        let results = edges
            .into_iter()
            .enumerate()
            .map(|(i, edge)| {
                let node = edge.node;
                let name = node.name.unwrap_or_default();
                let tagline = node.tagline.unwrap_or_default();
                let url = node
                    .url
                    .unwrap_or_else(|| "https://www.producthunt.com".to_string());
                let votes = node.votes_count.unwrap_or(0);

                SearchResult {
                    title: name,
                    url,
                    snippet: format!("{tagline} ({votes} votes)"),
                    source: "producthunt".to_string(),
                    category: Category::Market,
                    published: None,
                    author: None,
                    relevance_score: 1.0 - (i as f64 * 0.05),
                }
            })
            .collect();

        Ok(results)
    }
}
