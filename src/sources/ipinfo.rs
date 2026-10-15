use crate::error::OmniResult;
use crate::sources::Source;
use crate::types::{Category, SearchResult};
use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;

pub struct IpInfo { pub api_key: String }

#[derive(Deserialize)]
struct IpResponse { ip: Option<String>, hostname: Option<String>, city: Option<String>, region: Option<String>, country: Option<String>, org: Option<String>, loc: Option<String> }

#[async_trait]
impl Source for IpInfo {
    fn name(&self) -> &str { "ipinfo" }
    fn category(&self) -> Category { Category::Security }
    fn requires_key(&self) -> bool { true }
    fn rate_limit_per_minute(&self) -> u32 { 60 }

    async fn search(&self, client: &Client, query: &str, _max_results: usize) -> OmniResult<Vec<SearchResult>> {
        let url = format!("https://ipinfo.io/{}?token={}", urlencoding::encode(query), urlencoding::encode(&self.api_key));
        let data: IpResponse = client.get(&url).send().await?.json().await?;
        let ip = data.ip.unwrap_or_else(|| query.to_string());
        Ok(vec![SearchResult {
            title: format!("IP Info: {}", ip),
            url: format!("https://ipinfo.io/{}", ip),
            snippet: format!("Host: {} | Location: {}, {}, {} | Org: {} | Coords: {}",
                data.hostname.unwrap_or("N/A".into()), data.city.unwrap_or("N/A".into()),
                data.region.unwrap_or("N/A".into()), data.country.unwrap_or("N/A".into()),
                data.org.unwrap_or("N/A".into()), data.loc.unwrap_or("N/A".into())),
            source: "ipinfo".into(), category: Category::Security,
            published: None, author: None, relevance_score: 0.9,
        }])
    }
}
