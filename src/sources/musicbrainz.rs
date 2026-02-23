use crate::error::OmniResult;
use crate::sources::Source;
use crate::types::{Category, SearchResult};
use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;

pub struct MusicBrainz;

#[derive(Deserialize)]
struct MbResponse {
    artists: Option<Vec<MbArtist>>,
    #[serde(rename = "release-groups")]
    release_groups: Option<Vec<MbReleaseGroup>>,
}

#[derive(Deserialize)]
struct MbArtist {
    id: String,
    name: String,
    disambiguation: Option<String>,
    #[serde(rename = "type")]
    artist_type: Option<String>,
}

#[derive(Deserialize)]
struct MbReleaseGroup {
    id: String,
    title: String,
    #[serde(rename = "primary-type")]
    primary_type: Option<String>,
    #[serde(rename = "first-release-date")]
    first_release_date: Option<String>,
    #[serde(rename = "artist-credit", default)]
    artist_credit: Vec<MbArtistCredit>,
}

#[derive(Deserialize)]
struct MbArtistCredit {
    name: String,
}

#[async_trait]
impl Source for MusicBrainz {
    fn name(&self) -> &str { "musicbrainz" }
    fn category(&self) -> Category { Category::Music }
    fn requires_key(&self) -> bool { false }
    fn rate_limit_per_minute(&self) -> u32 { 50 }

    async fn search(&self, client: &Client, query: &str, max_results: usize) -> OmniResult<Vec<SearchResult>> {
        let limit = max_results.min(25);
        let url = format!(
            "https://musicbrainz.org/ws/2/release-group?query={}&limit={}&fmt=json",
            urlencoding::encode(query), limit
        );
        let resp: MbResponse = client.get(&url).send().await?.json().await?;
        let results: Vec<SearchResult> = resp.release_groups.unwrap_or_default().into_iter().enumerate().map(|(i, rg)| {
            let artists: String = rg.artist_credit.iter().map(|a| a.name.as_str()).collect::<Vec<_>>().join(", ");
            let snippet = format!("Type: {} | Released: {} | Artists: {}", rg.primary_type.unwrap_or("Unknown".into()), rg.first_release_date.unwrap_or("N/A".into()), artists);
            SearchResult {
                title: rg.title,
                url: format!("https://musicbrainz.org/release-group/{}", rg.id),
                snippet, source: "musicbrainz".into(),
                category: Category::Music,
                published: None,
                author: if artists.is_empty() { None } else { Some(artists) },
                relevance_score: 0.8 - (i as f64 * 0.03),
            }
        }).collect();
        Ok(results)
    }
}
