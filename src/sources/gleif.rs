use crate::error::OmniResult;
use crate::sources::Source;
use crate::types::{Category, SearchResult};
use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;

pub struct Gleif;

#[derive(Deserialize)]
struct GleifResponse {
    data: Option<Vec<GleifRecord>>,
}

#[derive(Deserialize)]
struct GleifRecord {
    id: Option<String>,
    attributes: GleifAttributes,
}

#[derive(Deserialize)]
struct GleifAttributes {
    lei: Option<String>,
    entity: GleifEntity,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct GleifEntity {
    legal_name: Option<GleifName>,
    legal_address: Option<GleifAddress>,
    jurisdiction: Option<String>,
    status: Option<String>,
    legal_form: Option<GleifLegalForm>,
    creation_date: Option<String>,
}

#[derive(Deserialize)]
struct GleifName {
    name: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct GleifAddress {
    city: Option<String>,
    country: Option<String>,
    region: Option<String>,
}

#[derive(Deserialize)]
struct GleifLegalForm {
    id: Option<String>,
}

#[async_trait]
impl Source for Gleif {
    fn name(&self) -> &str {
        "gleif"
    }
    fn category(&self) -> Category {
        Category::Market
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
            "https://api.gleif.org/api/v1/lei-records?filter%5Bfulltext%5D={}&page%5Bsize%5D={}",
            urlencoding::encode(query),
            limit,
        );

        let resp: GleifResponse = client.get(&url).send().await?.json().await?;

        let results = resp
            .data
            .unwrap_or_default()
            .into_iter()
            .enumerate()
            .map(|(i, record)| {
                let lei = record.attributes.lei.unwrap_or_default();
                let entity = record.attributes.entity;
                let name = entity
                    .legal_name
                    .and_then(|n| n.name)
                    .unwrap_or_default();
                let status = entity.status.unwrap_or_default();
                let jurisdiction = entity.jurisdiction.unwrap_or_default();
                let legal_form = entity
                    .legal_form
                    .and_then(|f| f.id)
                    .unwrap_or_default();
                let created = entity.creation_date.unwrap_or_default();

                let (city, country, region) = entity
                    .legal_address
                    .map(|a| {
                        (
                            a.city.unwrap_or_default(),
                            a.country.unwrap_or_default(),
                            a.region.unwrap_or_default(),
                        )
                    })
                    .unwrap_or_default();

                let snippet = format!(
                    "{status} — {jurisdiction} — {legal_form}\n{city}, {region}, {country}\nCreated: {created}"
                );

                SearchResult {
                    title: format!("{name} (LEI: {lei})"),
                    url: format!("https://search.gleif.org/#/record/{lei}"),
                    snippet,
                    source: "gleif".to_string(),
                    category: Category::Market,
                    published: None,
                    author: None,
                    relevance_score: 1.0 - (i as f64 * 0.02),
                }
            })
            .collect();

        Ok(results)
    }
}
