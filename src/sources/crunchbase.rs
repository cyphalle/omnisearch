use crate::error::OmniResult;
use crate::sources::Source;
use crate::types::{Category, SearchResult};
use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;

pub struct Crunchbase {
    pub api_key: String,
}

#[derive(Deserialize)]
struct CbResponse {
    entities: Option<Vec<CbEntity>>,
}

#[derive(Deserialize)]
struct CbEntity {
    identifier: CbIdentifier,
    short_description: Option<String>,
}

#[derive(Deserialize)]
struct CbIdentifier {
    #[serde(alias = "value")]
    name: Option<String>,
    permalink: Option<String>,
    entity_def_id: Option<String>,
}

#[async_trait]
impl Source for Crunchbase {
    fn name(&self) -> &str {
        "crunchbase"
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
        let limit = max_results.min(25);
        let url = format!(
            "https://api.crunchbase.com/api/v4/autocompletes?query={}&limit={}&user_key={}",
            urlencoding::encode(query),
            limit,
            urlencoding::encode(&self.api_key),
        );

        let resp: CbResponse = client.get(&url).send().await?.json().await?;

        let results = resp
            .entities
            .unwrap_or_default()
            .into_iter()
            .enumerate()
            .map(|(i, entity)| {
                let name = entity.identifier.name.unwrap_or_default();
                let permalink = entity.identifier.permalink.unwrap_or_default();
                let entity_type = entity
                    .identifier
                    .entity_def_id
                    .unwrap_or_else(|| "organization".to_string());
                let description = entity.short_description.unwrap_or_default();

                let url = format!("https://www.crunchbase.com/{entity_type}/{permalink}");

                SearchResult {
                    title: name,
                    url,
                    snippet: description,
                    source: "crunchbase".to_string(),
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
