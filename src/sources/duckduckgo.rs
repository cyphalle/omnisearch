use crate::error::OmniResult;
use crate::sources::Source;
use crate::types::{Category, SearchResult};
use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;

pub struct DuckDuckGo;

#[derive(Deserialize)]
struct DdgResponse {
    #[serde(rename = "Abstract")]
    r#abstract: String,
    #[serde(rename = "AbstractURL")]
    abstract_url: String,
    #[serde(rename = "AbstractText")]
    abstract_text: String,
    #[serde(rename = "Heading")]
    heading: String,
    #[serde(rename = "RelatedTopics")]
    related_topics: Vec<DdgTopic>,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum DdgTopic {
    Topic {
        #[serde(rename = "Text")]
        text: String,
        #[serde(rename = "FirstURL")]
        first_url: String,
    },
    Group {
        #[serde(rename = "Topics")]
        topics: Vec<DdgTopicInner>,
    },
}

#[derive(Deserialize)]
struct DdgTopicInner {
    #[serde(rename = "Text")]
    text: String,
    #[serde(rename = "FirstURL")]
    first_url: String,
}

#[async_trait]
impl Source for DuckDuckGo {
    fn name(&self) -> &str {
        "duckduckgo"
    }
    fn category(&self) -> Category {
        Category::General
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
        let url = format!(
            "https://api.duckduckgo.com/?q={}&format=json&no_html=1&skip_disambig=1",
            urlencoding::encode(query)
        );

        let resp: DdgResponse = client.get(&url).send().await?.json().await?;

        let mut results = Vec::new();

        // Add abstract if present
        if !resp.r#abstract.is_empty() {
            results.push(SearchResult {
                title: resp.heading.clone(),
                url: resp.abstract_url.clone(),
                snippet: resp.abstract_text,
                source: "duckduckgo".to_string(),
                category: Category::General,
                published: None,
                author: None,
                relevance_score: 1.0,
            });
        }

        // Add related topics
        for topic in resp.related_topics {
            if results.len() >= max_results {
                break;
            }
            match topic {
                DdgTopic::Topic { text, first_url } => {
                    if !first_url.is_empty() {
                        results.push(SearchResult {
                            title: text.chars().take(100).collect(),
                            url: first_url,
                            snippet: text,
                            source: "duckduckgo".to_string(),
                            category: Category::General,
                            published: None,
                            author: None,
                            relevance_score: 0.8 - (results.len() as f64 * 0.05),
                        });
                    }
                }
                DdgTopic::Group { topics } => {
                    for t in topics {
                        if results.len() >= max_results {
                            break;
                        }
                        if !t.first_url.is_empty() {
                            results.push(SearchResult {
                                title: t.text.chars().take(100).collect(),
                                url: t.first_url,
                                snippet: t.text,
                                source: "duckduckgo".to_string(),
                                category: Category::General,
                                published: None,
                                author: None,
                                relevance_score: 0.6 - (results.len() as f64 * 0.05),
                            });
                        }
                    }
                }
            }
        }

        Ok(results)
    }
}
