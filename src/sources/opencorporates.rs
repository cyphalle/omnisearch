use crate::error::OmniResult;
use crate::sources::Source;
use crate::types::{Category, SearchResult};
use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;

pub struct OpenCorporates {
    pub api_key: String,
}

#[derive(Deserialize)]
struct OcResponse {
    results: Option<OcResults>,
}

#[derive(Deserialize)]
struct OcResults {
    companies: Option<Vec<OcCompanyWrapper>>,
}

#[derive(Deserialize)]
struct OcCompanyWrapper {
    company: OcCompany,
}

#[derive(Deserialize)]
struct OcCompany {
    name: Option<String>,
    company_number: Option<String>,
    jurisdiction_code: Option<String>,
    incorporation_date: Option<String>,
    company_type: Option<String>,
    current_status: Option<String>,
    opencorporates_url: Option<String>,
    registered_address_in_full: Option<String>,
}

#[async_trait]
impl Source for OpenCorporates {
    fn name(&self) -> &str {
        "opencorporates"
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
        let limit = max_results.min(30);
        let url = format!(
            "https://api.opencorporates.com/v0.4/companies/search?q={}&per_page={}&api_token={}",
            urlencoding::encode(query),
            limit,
            urlencoding::encode(&self.api_key),
        );

        let resp: OcResponse = client.get(&url).send().await?.json().await?;

        let results = resp
            .results
            .and_then(|r| r.companies)
            .unwrap_or_default()
            .into_iter()
            .enumerate()
            .map(|(i, wrapper)| {
                let co = wrapper.company;
                let name = co.name.unwrap_or_default();
                let number = co.company_number.unwrap_or_default();
                let jurisdiction = co.jurisdiction_code.unwrap_or_default();
                let inc_date = co.incorporation_date.unwrap_or_default();
                let co_type = co.company_type.unwrap_or_default();
                let status = co.current_status.unwrap_or_default();
                let url = co
                    .opencorporates_url
                    .unwrap_or_else(|| "https://opencorporates.com".to_string());
                let address = co.registered_address_in_full.unwrap_or_default();

                let snippet = format!(
                    "{jurisdiction} — {co_type} — {status} — Inc: {inc_date}\n{address}"
                );

                SearchResult {
                    title: format!("{name} ({number})"),
                    url,
                    snippet,
                    source: "opencorporates".to_string(),
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
