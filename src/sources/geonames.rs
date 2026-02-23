use crate::error::OmniResult;
use crate::sources::Source;
use crate::types::{Category, SearchResult};
use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;

pub struct GeoNames;

#[derive(Deserialize)]
struct GnResponse {
    geonames: Option<Vec<GnEntry>>,
}

#[derive(Deserialize)]
struct GnEntry {
    #[serde(rename = "geonameId")]
    geoname_id: u64,
    name: String,
    #[serde(rename = "countryName")]
    country_name: Option<String>,
    #[serde(rename = "adminName1")]
    admin_name1: Option<String>,
    population: Option<u64>,
    lat: Option<String>,
    lng: Option<String>,
    #[serde(rename = "fcodeName")]
    fcode_name: Option<String>,
}

#[async_trait]
impl Source for GeoNames {
    fn name(&self) -> &str { "geonames" }
    fn category(&self) -> Category { Category::Geo }
    fn requires_key(&self) -> bool { false }
    fn rate_limit_per_minute(&self) -> u32 { 60 }

    async fn search(&self, client: &Client, query: &str, max_results: usize) -> OmniResult<Vec<SearchResult>> {
        let limit = max_results.min(20);
        let url = format!(
            "http://api.geonames.org/searchJSON?q={}&maxRows={}&username=demo",
            urlencoding::encode(query), limit
        );
        let resp = client.get(&url).send().await?;
        if !resp.status().is_success() { return Ok(vec![]); }
        let data: GnResponse = resp.json().await.unwrap_or(GnResponse { geonames: None });

        Ok(data.geonames.unwrap_or_default().into_iter().enumerate().map(|(i, e)| {
            let snippet = format!("Type: {} | Country: {} | Region: {} | Population: {} | Coords: {}, {}",
                e.fcode_name.unwrap_or("N/A".into()),
                e.country_name.unwrap_or("N/A".into()),
                e.admin_name1.unwrap_or("N/A".into()),
                e.population.unwrap_or(0),
                e.lat.unwrap_or("N/A".into()),
                e.lng.unwrap_or("N/A".into()),
            );
            SearchResult {
                title: e.name,
                url: format!("https://www.geonames.org/{}", e.geoname_id),
                snippet, source: "geonames".into(),
                category: Category::Geo,
                published: None, author: None,
                relevance_score: 0.7 - (i as f64 * 0.03),
            }
        }).collect())
    }
}
