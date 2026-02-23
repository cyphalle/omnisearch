use crate::config::Config;
use crate::error::OmniResult;
use crate::types::{Category, SearchResult, SourceInfo};
use async_trait::async_trait;
use reqwest::Client;

pub mod arxiv;
pub mod common_crawl;
pub mod crossref;
pub mod crtsh;
pub mod datamuse;
pub mod dbpedia;
pub mod duckduckgo;
pub mod frankfurter;
pub mod gdelt;
pub mod geonames;
pub mod hacker_news;
pub mod haveibeenpwned;
pub mod internet_archive;
pub mod musicbrainz;
pub mod open_meteo;
pub mod openalex;
pub mod pubmed;
pub mod restcountries;
pub mod stackexchange;
pub mod wikidata;
pub mod wikipedia;
pub mod world_bank;

// Key-required sources
pub mod alpha_vantage;
pub mod brave;
pub mod coingecko;
pub mod currents;
pub mod event_registry;
pub mod fred;
pub mod github;
pub mod gnews;
pub mod google_custom_search;
pub mod guardian;
pub mod ipinfo;
pub mod mapbox;
pub mod mediastack;
pub mod newsapi;
pub mod nyt;
pub mod omdb;
pub mod open_exchange_rates;
pub mod openweathermap;
pub mod pexels;
pub mod polygon;
pub mod semantic_scholar;
pub mod shodan;
pub mod tmdb;
pub mod unsplash;
pub mod urlscan;
pub mod virustotal;
pub mod wolfram_alpha;
pub mod youtube;

#[async_trait]
pub trait Source: Send + Sync {
    fn name(&self) -> &str;
    fn category(&self) -> Category;
    fn requires_key(&self) -> bool;
    fn rate_limit_per_minute(&self) -> u32 {
        60
    }

    async fn search(
        &self,
        client: &Client,
        query: &str,
        max_results: usize,
    ) -> OmniResult<Vec<SearchResult>>;
}

pub fn build_sources(config: &Config) -> Vec<Box<dyn Source>> {
    let mut sources: Vec<Box<dyn Source>> = Vec::new();

    // No-key sources
    if config.is_source_enabled("wikipedia") {
        sources.push(Box::new(wikipedia::Wikipedia));
    }
    if config.is_source_enabled("duckduckgo") {
        sources.push(Box::new(duckduckgo::DuckDuckGo));
    }
    if config.is_source_enabled("arxiv") {
        sources.push(Box::new(arxiv::Arxiv));
    }
    if config.is_source_enabled("hacker_news") {
        sources.push(Box::new(hacker_news::HackerNews));
    }
    if config.is_source_enabled("stackexchange") {
        sources.push(Box::new(stackexchange::StackExchange));
    }
    if config.is_source_enabled("wikidata") {
        sources.push(Box::new(wikidata::Wikidata));
    }
    if config.is_source_enabled("openalex") {
        sources.push(Box::new(openalex::OpenAlex));
    }
    if config.is_source_enabled("crossref") {
        sources.push(Box::new(crossref::CrossRef));
    }
    if config.is_source_enabled("pubmed") {
        sources.push(Box::new(pubmed::PubMed));
    }
    if config.is_source_enabled("gdelt") {
        sources.push(Box::new(gdelt::Gdelt));
    }
    if config.is_source_enabled("open_meteo") {
        sources.push(Box::new(open_meteo::OpenMeteo));
    }
    if config.is_source_enabled("restcountries") {
        sources.push(Box::new(restcountries::RestCountries));
    }
    if config.is_source_enabled("frankfurter") {
        sources.push(Box::new(frankfurter::Frankfurter));
    }
    if config.is_source_enabled("datamuse") {
        sources.push(Box::new(datamuse::Datamuse));
    }
    if config.is_source_enabled("internet_archive") {
        sources.push(Box::new(internet_archive::InternetArchive));
    }
    if config.is_source_enabled("musicbrainz") {
        sources.push(Box::new(musicbrainz::MusicBrainz));
    }
    if config.is_source_enabled("dbpedia") {
        sources.push(Box::new(dbpedia::DBpedia));
    }
    if config.is_source_enabled("world_bank") {
        sources.push(Box::new(world_bank::WorldBank));
    }
    if config.is_source_enabled("common_crawl") {
        sources.push(Box::new(common_crawl::CommonCrawl));
    }
    if config.is_source_enabled("crtsh") {
        sources.push(Box::new(crtsh::CrtSh));
    }
    if config.is_source_enabled("haveibeenpwned") {
        sources.push(Box::new(haveibeenpwned::HaveIBeenPwned));
    }
    if config.is_source_enabled("geonames") {
        sources.push(Box::new(geonames::GeoNames));
    }

    // Key-required sources — only added if key is configured
    macro_rules! add_keyed_source {
        ($name:expr, $key:expr, $ctor:expr) => {
            if config.is_source_enabled($name) {
                if let Some(key) = config.get_api_key($key) {
                    sources.push(Box::new($ctor(key.to_string())));
                }
            }
        };
    }

    add_keyed_source!("brave", "brave", |k| brave::Brave { api_key: k });
    add_keyed_source!("newsapi", "newsapi", |k| newsapi::NewsApi { api_key: k });
    if config.is_source_enabled("google_custom_search") {
        if let (Some(key), Some(cx)) = (config.get_api_key("google_custom_search"), config.get_api_key("google_cx")) {
            sources.push(Box::new(google_custom_search::GoogleCustomSearch { api_key: key.to_string(), cx: cx.to_string() }));
        }
    }
    add_keyed_source!("guardian", "guardian", |k| guardian::Guardian { api_key: k });
    add_keyed_source!("nyt", "nyt", |k| nyt::Nyt { api_key: k });
    add_keyed_source!("gnews", "gnews", |k| gnews::GNews { api_key: k });
    add_keyed_source!("mediastack", "mediastack", |k| mediastack::MediaStack { api_key: k });
    add_keyed_source!("currents", "currents", |k| currents::Currents { api_key: k });
    add_keyed_source!("event_registry", "event_registry", |k| event_registry::EventRegistry { api_key: k });
    add_keyed_source!("alpha_vantage", "alpha_vantage", |k| alpha_vantage::AlphaVantage { api_key: k });
    add_keyed_source!("coingecko", "coingecko", |k| coingecko::CoinGecko { api_key: k });
    add_keyed_source!("polygon", "polygon", |k| polygon::Polygon { api_key: k });
    add_keyed_source!("fred", "fred", |k| fred::Fred { api_key: k });
    add_keyed_source!("open_exchange_rates", "open_exchange_rates", |k| open_exchange_rates::OpenExchangeRates { api_key: k });
    add_keyed_source!("openweathermap", "openweathermap", |k| openweathermap::OpenWeatherMap { api_key: k });
    add_keyed_source!("youtube", "youtube", |k| youtube::YouTube { api_key: k });
    add_keyed_source!("tmdb", "tmdb", |k| tmdb::Tmdb { api_key: k });
    add_keyed_source!("omdb", "omdb", |k| omdb::Omdb { api_key: k });
    add_keyed_source!("unsplash", "unsplash", |k| unsplash::Unsplash { api_key: k });
    add_keyed_source!("pexels", "pexels", |k| pexels::Pexels { api_key: k });
    add_keyed_source!("github", "github", |k| github::GitHub { api_key: k });
    add_keyed_source!("semantic_scholar", "semantic_scholar", |k| semantic_scholar::SemanticScholar { api_key: k });
    add_keyed_source!("wolfram_alpha", "wolfram_alpha", |k| wolfram_alpha::WolframAlpha { api_key: k });
    add_keyed_source!("shodan", "shodan", |k| shodan::Shodan { api_key: k });
    add_keyed_source!("virustotal", "virustotal", |k| virustotal::VirusTotal { api_key: k });
    add_keyed_source!("urlscan", "urlscan", |k| urlscan::UrlScan { api_key: k });
    add_keyed_source!("ipinfo", "ipinfo", |k| ipinfo::IpInfo { api_key: k });
    add_keyed_source!("mapbox", "mapbox", |k| mapbox::Mapbox { api_key: k });

    sources
}

pub fn all_source_info(config: &Config) -> Vec<SourceInfo> {
    let all: Vec<(&str, Category, bool)> = vec![
        // No-key sources
        ("wikipedia", Category::Reference, false),
        ("wikidata", Category::Reference, false),
        ("arxiv", Category::Academic, false),
        ("openalex", Category::Academic, false),
        ("crossref", Category::Academic, false),
        ("pubmed", Category::Academic, false),
        ("gdelt", Category::News, false),
        ("duckduckgo", Category::General, false),
        ("open_meteo", Category::Weather, false),
        ("restcountries", Category::Geo, false),
        ("frankfurter", Category::Finance, false),
        ("datamuse", Category::Reference, false),
        ("internet_archive", Category::General, false),
        ("musicbrainz", Category::Music, false),
        ("dbpedia", Category::Reference, false),
        ("world_bank", Category::Finance, false),
        ("stackexchange", Category::Code, false),
        ("hacker_news", Category::Code, false),
        ("common_crawl", Category::General, false),
        ("crtsh", Category::Security, false),
        ("haveibeenpwned", Category::Security, false),
        ("geonames", Category::Geo, false),
        // Key-required sources
        ("brave", Category::General, true),
        ("newsapi", Category::News, true),
        ("google_custom_search", Category::General, true),
        ("guardian", Category::News, true),
        ("nyt", Category::News, true),
        ("gnews", Category::News, true),
        ("mediastack", Category::News, true),
        ("currents", Category::News, true),
        ("event_registry", Category::News, true),
        ("alpha_vantage", Category::Finance, true),
        ("coingecko", Category::Finance, true),
        ("polygon", Category::Finance, true),
        ("fred", Category::Finance, true),
        ("open_exchange_rates", Category::Finance, true),
        ("openweathermap", Category::Weather, true),
        ("youtube", Category::Media, true),
        ("tmdb", Category::Media, true),
        ("omdb", Category::Media, true),
        ("unsplash", Category::Media, true),
        ("pexels", Category::Media, true),
        ("github", Category::Code, true),
        ("semantic_scholar", Category::Academic, true),
        ("wolfram_alpha", Category::Reference, true),
        ("shodan", Category::Security, true),
        ("virustotal", Category::Security, true),
        ("urlscan", Category::Security, true),
        ("ipinfo", Category::Security, true),
        ("mapbox", Category::Geo, true),
    ];

    all.into_iter()
        .map(|(name, category, requires_key)| {
            let key_configured = if requires_key {
                config.get_api_key(name).is_some()
            } else {
                true
            };
            SourceInfo {
                name: name.to_string(),
                category,
                requires_key,
                key_configured,
                enabled: config.is_source_enabled(name),
                rate_limit: format!(
                    "{}/min",
                    config
                        .sources
                        .get(name)
                        .and_then(|s| s.rate_limit_per_minute)
                        .unwrap_or(60)
                ),
            }
        })
        .collect()
}
