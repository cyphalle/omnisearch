use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Category {
    General,
    Academic,
    News,
    Finance,
    Weather,
    Media,
    Code,
    Reference,
    Security,
    Geo,
    Music,
}

impl fmt::Display for Category {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Category::General => write!(f, "general"),
            Category::Academic => write!(f, "academic"),
            Category::News => write!(f, "news"),
            Category::Finance => write!(f, "finance"),
            Category::Weather => write!(f, "weather"),
            Category::Media => write!(f, "media"),
            Category::Code => write!(f, "code"),
            Category::Reference => write!(f, "reference"),
            Category::Security => write!(f, "security"),
            Category::Geo => write!(f, "geo"),
            Category::Music => write!(f, "music"),
        }
    }
}

impl std::str::FromStr for Category {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "general" => Ok(Category::General),
            "academic" => Ok(Category::Academic),
            "news" => Ok(Category::News),
            "finance" => Ok(Category::Finance),
            "weather" => Ok(Category::Weather),
            "media" => Ok(Category::Media),
            "code" => Ok(Category::Code),
            "reference" => Ok(Category::Reference),
            "security" => Ok(Category::Security),
            "geo" => Ok(Category::Geo),
            "music" => Ok(Category::Music),
            _ => Err(format!("Unknown category: {s}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub title: String,
    pub url: String,
    pub snippet: String,
    pub source: String,
    pub category: Category,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub published: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    pub relevance_score: f64,
}

#[derive(Debug, Clone)]
pub struct SearchQuery {
    pub query: String,
    pub max_results: usize,
    pub categories: Option<Vec<Category>>,
    pub sources: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceInfo {
    pub name: String,
    pub category: Category,
    pub requires_key: bool,
    pub key_configured: bool,
    pub enabled: bool,
    pub rate_limit: String,
}

#[derive(Debug, Serialize)]
pub struct SearchResponse {
    pub query: String,
    pub total_results: usize,
    pub sources_queried: Vec<String>,
    pub results: Vec<SearchResult>,
    pub errors: Vec<SourceError>,
}

#[derive(Debug, Serialize)]
pub struct SourceError {
    pub source: String,
    pub error: String,
}
