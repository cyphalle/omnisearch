use crate::error::OmniResult;
use crate::sources::Source;
use crate::types::{Category, SearchResult};
use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;

pub struct CompaniesHouse {
    pub api_key: String,
}

#[derive(Deserialize)]
struct ChResponse {
    items: Option<Vec<ChCompany>>,
}

#[derive(Deserialize)]
struct ChCompany {
    title: Option<String>,
    company_number: Option<String>,
    company_status: Option<String>,
    company_type: Option<String>,
    date_of_creation: Option<String>,
    address_snippet: Option<String>,
    snippet: Option<String>,
    description: Option<String>,
}

#[async_trait]
impl Source for CompaniesHouse {
    fn name(&self) -> &str {
        "companies_house"
    }
    fn category(&self) -> Category {
        Category::Market
    }
    fn requires_key(&self) -> bool {
        true
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
        let limit = max_results.min(20);
        let url = format!(
            "https://api.company-information.service.gov.uk/search/companies?q={}&items_per_page={}",
            urlencoding::encode(query),
            limit,
        );

        let resp: ChResponse = client
            .get(&url)
            .basic_auth(&self.api_key, Option::<&str>::None)
            .send()
            .await?
            .json()
            .await?;

        let results = resp
            .items
            .unwrap_or_default()
            .into_iter()
            .enumerate()
            .map(|(i, co)| {
                let name = co.title.unwrap_or_default();
                let number = co.company_number.unwrap_or_default();
                let status = co.company_status.unwrap_or_default();
                let co_type = co.company_type.unwrap_or_default();
                let created = co.date_of_creation.unwrap_or_default();
                let address = co.address_snippet.unwrap_or_default();

                let snippet = format!(
                    "{co_type} — {status} — Created: {created}\n{address}"
                );

                SearchResult {
                    title: format!("{name} ({number})"),
                    url: format!(
                        "https://find-and-update.company-information.service.gov.uk/company/{}",
                        number
                    ),
                    snippet,
                    source: "companies_house".to_string(),
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
