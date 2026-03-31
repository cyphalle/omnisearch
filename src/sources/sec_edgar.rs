use crate::error::OmniResult;
use crate::sources::Source;
use crate::types::{Category, SearchResult};
use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;

pub struct SecEdgar;

#[derive(Deserialize)]
struct EdgarResponse {
    hits: EdgarHits,
}

#[derive(Deserialize)]
struct EdgarHits {
    hits: Vec<EdgarHit>,
}

#[derive(Deserialize)]
struct EdgarHit {
    #[serde(rename = "_source")]
    source: EdgarSource,
}

#[derive(Deserialize)]
struct EdgarSource {
    #[serde(default)]
    display_names: Vec<String>,
    file_date: Option<String>,
    form: Option<String>,
    file_description: Option<String>,
    file_type: Option<String>,
    adsh: Option<String>,
    #[serde(default)]
    ciks: Vec<String>,
}

#[async_trait]
impl Source for SecEdgar {
    fn name(&self) -> &str {
        "sec_edgar"
    }
    fn category(&self) -> Category {
        Category::Market
    }
    fn requires_key(&self) -> bool {
        false
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
        let limit = max_results.min(40);
        let url = format!(
            "https://efts.sec.gov/LATEST/search-index?q={}&from=0&size={}",
            urlencoding::encode(query),
            limit
        );

        let resp: EdgarResponse = client
            .get(&url)
            .header("User-Agent", "omnisearch/0.1.0 (contact@omnisearch.dev)")
            .send()
            .await?
            .json()
            .await?;

        let results = resp
            .hits
            .hits
            .into_iter()
            .enumerate()
            .filter_map(|(i, hit)| {
                let entity = hit.source.display_names.first().cloned().unwrap_or_default();
                if entity.is_empty() {
                    return None;
                }
                let form = hit
                    .source
                    .form
                    .or(hit.source.file_type)
                    .unwrap_or_default();
                let date = hit.source.file_date.unwrap_or_default();
                let desc = hit
                    .source
                    .file_description
                    .filter(|d| !d.is_empty())
                    .unwrap_or_else(|| format!("{form} filing"));
                let adsh = hit.source.adsh.unwrap_or_default();
                let cik = hit.source.ciks.first().cloned().unwrap_or_default();

                // Build direct link to filing
                let filing_url = if !adsh.is_empty() && !cik.is_empty() {
                    let adsh_nodash = adsh.replace('-', "");
                    format!(
                        "https://www.sec.gov/Archives/edgar/data/{cik}/{adsh_nodash}/{adsh}-index.htm"
                    )
                } else {
                    format!(
                        "https://www.sec.gov/cgi-bin/browse-edgar?company={}&type={}&action=getcompany",
                        urlencoding::encode(&entity),
                        urlencoding::encode(&form),
                    )
                };

                Some(SearchResult {
                    title: format!("{entity} — {form} ({date})"),
                    url: filing_url,
                    snippet: desc,
                    source: "sec_edgar".to_string(),
                    category: Category::Market,
                    published: None,
                    author: Some(entity),
                    relevance_score: 1.0 - (i as f64 * 0.05),
                })
            })
            .collect();

        Ok(results)
    }
}
