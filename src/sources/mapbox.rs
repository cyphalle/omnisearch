use crate::error::OmniResult;
use crate::sources::Source;
use crate::types::{Category, SearchResult};
use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;

pub struct Mapbox { pub api_key: String }

#[derive(Deserialize)]
struct MbResponse { features: Option<Vec<MbFeature>> }
#[derive(Deserialize)]
struct MbFeature { place_name: String, center: Vec<f64>, place_type: Vec<String> }

#[async_trait]
impl Source for Mapbox {
    fn name(&self) -> &str { "mapbox" }
    fn category(&self) -> Category { Category::Geo }
    fn requires_key(&self) -> bool { true }
    fn rate_limit_per_minute(&self) -> u32 { 60 }

    async fn search(&self, client: &Client, query: &str, max_results: usize) -> OmniResult<Vec<SearchResult>> {
        let limit = max_results.min(10);
        let url = format!("https://api.mapbox.com/geocoding/v5/mapbox.places/{}.json?access_token={}&limit={}", urlencoding::encode(query), self.api_key, limit);
        let data: MbResponse = client.get(&url).send().await?.json().await?;
        Ok(data.features.unwrap_or_default().into_iter().enumerate().map(|(i, f)| {
            let coords = if f.center.len() >= 2 { format!("{:.4}, {:.4}", f.center[1], f.center[0]) } else { "N/A".into() };
            SearchResult {
                title: f.place_name.clone(),
                url: format!("https://www.openstreetmap.org/#map=15/{}", coords.replace(", ", "/")),
                snippet: format!("Type: {} | Coordinates: {}", f.place_type.join(", "), coords),
                source: "mapbox".into(), category: Category::Geo,
                published: None, author: None,
                relevance_score: 0.85 - (i as f64 * 0.05),
            }
        }).collect())
    }
}
