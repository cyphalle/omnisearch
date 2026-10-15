use crate::error::OmniResult;
use crate::sources::Source;
use crate::types::{Category, SearchResult};
use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;

pub struct OpenWeatherMap { pub api_key: String }

#[derive(Deserialize)]
struct OwmGeo { name: String, lat: f64, lon: f64, country: Option<String>, state: Option<String> }

#[derive(Deserialize)]
struct OwmWeather { main: Option<OwmMain>, weather: Option<Vec<OwmCondition>>, wind: Option<OwmWind>, name: Option<String> }
#[derive(Deserialize)]
struct OwmMain { temp: f64, humidity: u32, feels_like: Option<f64> }
#[derive(Deserialize)]
struct OwmCondition { description: String }
#[derive(Deserialize)]
struct OwmWind { speed: f64 }

#[async_trait]
impl Source for OpenWeatherMap {
    fn name(&self) -> &str { "openweathermap" }
    fn category(&self) -> Category { Category::Weather }
    fn requires_key(&self) -> bool { true }
    fn rate_limit_per_minute(&self) -> u32 { 60 }

    async fn search(&self, client: &Client, query: &str, _max_results: usize) -> OmniResult<Vec<SearchResult>> {
        let geo_url = format!("https://api.openweathermap.org/geo/1.0/direct?q={}&limit=1&appid={}", urlencoding::encode(query), urlencoding::encode(&self.api_key));
        let geos: Vec<OwmGeo> = client.get(&geo_url).send().await?.json().await?;
        let Some(geo) = geos.first() else { return Ok(vec![]); };

        let weather_url = format!("https://api.openweathermap.org/data/2.5/weather?lat={}&lon={}&appid={}&units=metric", geo.lat, geo.lon, urlencoding::encode(&self.api_key));
        let weather: OwmWeather = client.get(&weather_url).send().await?.json().await?;

        let mut results = Vec::new();
        if let Some(main) = weather.main {
            let condition = weather.weather.and_then(|w| w.first().map(|c| c.description.clone())).unwrap_or_default();
            let wind = weather.wind.map(|w| w.speed).unwrap_or(0.0);
            let location = format!("{}{}", geo.name, geo.country.as_ref().map(|c| format!(", {c}")).unwrap_or_default());
            results.push(SearchResult {
                title: format!("Weather in {location}"),
                url: "https://openweathermap.org/city".to_string(),
                snippet: format!("{condition} | Temp: {:.1}°C (feels {:.1}°C) | Humidity: {}% | Wind: {:.1} m/s", main.temp, main.feels_like.unwrap_or(main.temp), main.humidity, wind),
                source: "openweathermap".into(), category: Category::Weather,
                published: Some(chrono::Utc::now()), author: None,
                relevance_score: 0.9,
            });
        }
        Ok(results)
    }
}
