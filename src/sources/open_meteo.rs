use crate::error::OmniResult;
use crate::sources::Source;
use crate::types::{Category, SearchResult};
use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;

pub struct OpenMeteo;

#[derive(Deserialize)]
struct GeoResponse {
    results: Option<Vec<GeoResult>>,
}

#[derive(Deserialize)]
struct GeoResult {
    name: String,
    latitude: f64,
    longitude: f64,
    country: Option<String>,
    admin1: Option<String>,
}

#[derive(Deserialize)]
struct WeatherResponse {
    current: Option<CurrentWeather>,
}

#[derive(Deserialize)]
struct CurrentWeather {
    temperature_2m: Option<f64>,
    wind_speed_10m: Option<f64>,
    relative_humidity_2m: Option<f64>,
}

#[async_trait]
impl Source for OpenMeteo {
    fn name(&self) -> &str { "open_meteo" }
    fn category(&self) -> Category { Category::Weather }
    fn requires_key(&self) -> bool { false }
    fn rate_limit_per_minute(&self) -> u32 { 60 }

    async fn search(&self, client: &Client, query: &str, _max_results: usize) -> OmniResult<Vec<SearchResult>> {
        let geo_url = format!("https://geocoding-api.open-meteo.com/v1/search?name={}&count=1", urlencoding::encode(query));
        let geo: GeoResponse = client.get(&geo_url).send().await?.json().await?;
        let Some(locations) = geo.results else { return Ok(vec![]); };
        let Some(loc) = locations.first() else { return Ok(vec![]); };

        let weather_url = format!(
            "https://api.open-meteo.com/v1/forecast?latitude={}&longitude={}&current=temperature_2m,wind_speed_10m,relative_humidity_2m",
            loc.latitude, loc.longitude
        );
        let weather: WeatherResponse = client.get(&weather_url).send().await?.json().await?;

        let mut results = Vec::new();
        if let Some(current) = weather.current {
            let location_name = format!("{}{}{}", loc.name, loc.admin1.as_ref().map(|a| format!(", {a}")).unwrap_or_default(), loc.country.as_ref().map(|c| format!(", {c}")).unwrap_or_default());
            let snippet = format!(
                "Temperature: {}°C | Wind: {} km/h | Humidity: {}%",
                current.temperature_2m.map(|t| format!("{t:.1}")).unwrap_or("N/A".into()),
                current.wind_speed_10m.map(|w| format!("{w:.1}")).unwrap_or("N/A".into()),
                current.relative_humidity_2m.map(|h| format!("{h:.0}")).unwrap_or("N/A".into()),
            );
            results.push(SearchResult {
                title: format!("Weather in {location_name}"),
                url: format!("https://open-meteo.com/en/docs#latitude={}&longitude={}", loc.latitude, loc.longitude),
                snippet, source: "open_meteo".into(),
                category: Category::Weather,
                published: Some(chrono::Utc::now()),
                author: None,
                relevance_score: 0.9,
            });
        }
        Ok(results)
    }
}
