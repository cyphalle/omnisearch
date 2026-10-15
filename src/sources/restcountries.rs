use crate::error::OmniResult;
use crate::sources::Source;
use crate::types::{Category, SearchResult};
use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;

pub struct RestCountries;

#[derive(Deserialize)]
struct Country {
    name: CountryName,
    capital: Option<Vec<String>>,
    region: Option<String>,
    subregion: Option<String>,
    population: Option<u64>,
    #[serde(default)]
    languages: Option<serde_json::Value>,
    #[serde(default)]
    currencies: Option<serde_json::Value>,
    cca2: Option<String>,
}

#[derive(Deserialize)]
struct CountryName {
    common: String,
    official: String,
}

#[async_trait]
impl Source for RestCountries {
    fn name(&self) -> &str { "restcountries" }
    fn category(&self) -> Category { Category::Geo }
    fn requires_key(&self) -> bool { false }
    fn rate_limit_per_minute(&self) -> u32 { 60 }

    async fn search(&self, client: &Client, query: &str, max_results: usize) -> OmniResult<Vec<SearchResult>> {
        let url = format!("https://restcountries.com/v3.1/name/{}", urlencoding::encode(query));
        let resp = client.get(&url).send().await?;
        if !resp.status().is_success() { return Ok(vec![]); }
        let countries: Vec<Country> = resp.json().await.unwrap_or_default();

        Ok(countries.into_iter().take(max_results).enumerate().map(|(i, c)| {
            let capital = c.capital.map(|caps| caps.join(", ")).unwrap_or("N/A".into());
            let snippet = format!(
                "Official: {} | Capital: {} | Region: {} | Population: {}",
                c.name.official, capital,
                c.region.unwrap_or("N/A".into()),
                c.population.map(format_number).unwrap_or("N/A".into()),
            );
            let code = c.cca2.unwrap_or_default().to_lowercase();
            SearchResult {
                title: c.name.common,
                url: format!("https://en.wikipedia.org/wiki/{}", code.to_uppercase()),
                snippet, source: "restcountries".into(),
                category: Category::Geo,
                published: None, author: None,
                relevance_score: 0.9 - (i as f64 * 0.05),
            }
        }).collect())
    }
}

fn format_number(n: u64) -> String {
    let s = n.to_string();
    let mut result = String::new();
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 { result.push(','); }
        result.push(c);
    }
    result.chars().rev().collect()
}
