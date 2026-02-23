use crate::error::{OmniError, OmniResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub general: GeneralConfig,
    #[serde(default)]
    pub api_keys: HashMap<String, String>,
    #[serde(default)]
    pub sources: HashMap<String, SourceConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    #[serde(default = "default_timeout")]
    pub timeout_secs: u64,
    #[serde(default = "default_max_results")]
    pub max_results: usize,
    #[serde(default = "default_format")]
    pub default_format: String,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            timeout_secs: default_timeout(),
            max_results: default_max_results(),
            default_format: default_format(),
        }
    }
}

fn default_timeout() -> u64 {
    5
}
fn default_max_results() -> usize {
    20
}
fn default_format() -> String {
    "json".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub rate_limit_per_minute: Option<u32>,
}

fn default_true() -> bool {
    true
}

impl Default for Config {
    fn default() -> Self {
        Self {
            general: GeneralConfig::default(),
            api_keys: HashMap::new(),
            sources: HashMap::new(),
        }
    }
}

impl Config {
    pub fn config_dir() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("~/.config"))
            .join("omnisearch")
    }

    pub fn config_path() -> PathBuf {
        Self::config_dir().join("config.toml")
    }

    pub fn load() -> OmniResult<Self> {
        let path = Self::config_path();
        if path.exists() {
            let content = std::fs::read_to_string(&path)?;
            toml::from_str(&content).map_err(|e| OmniError::Config(e.to_string()))
        } else {
            Ok(Config::default())
        }
    }

    pub fn save(&self) -> OmniResult<()> {
        let dir = Self::config_dir();
        std::fs::create_dir_all(&dir)?;
        let content =
            toml::to_string_pretty(self).map_err(|e| OmniError::Config(e.to_string()))?;
        std::fs::write(Self::config_path(), content)?;
        Ok(())
    }

    pub fn get_api_key(&self, source: &str) -> Option<&str> {
        self.api_keys.get(source).map(|s| s.as_str())
    }

    pub fn set_api_key(&mut self, source: String, key: String) {
        self.api_keys.insert(source, key);
    }

    pub fn is_source_enabled(&self, source: &str) -> bool {
        self.sources
            .get(source)
            .map(|s| s.enabled)
            .unwrap_or(true)
    }

    pub fn init_default() -> OmniResult<()> {
        let config = Config::default();
        config.save()?;
        Ok(())
    }

    pub fn generate_template() -> String {
        r#"# Omnisearch Configuration
# Place this file at ~/.config/omnisearch/config.toml

[general]
# Default timeout per source in seconds
timeout_secs = 5
# Default maximum results to return
max_results = 20
# Default output format: json, table, markdown
default_format = "json"

[api_keys]
# Add your API keys here. All keys are for free tiers.
# brave = "BSA..."
# newsapi = "..."
# google_custom_search = "..."
# google_cx = "..."
# guardian = "..."
# nyt = "..."
# gnews = "..."
# mediastack = "..."
# currents = "..."
# event_registry = "..."
# alpha_vantage = "..."
# polygon = "..."
# fred = "..."
# open_exchange_rates = "..."
# openweathermap = "..."
# youtube = "..."
# tmdb = "..."
# omdb = "..."
# unsplash = "..."
# pexels = "..."
# github = "..."
# semantic_scholar = "..."
# wolfram_alpha = "..."
# shodan = "..."
# virustotal = "..."
# urlscan = "..."
# ipinfo = "..."
# mapbox = "..."
# coingecko = "..."

# [sources.wikipedia]
# enabled = true
# rate_limit_per_minute = 200
"#
        .to_string()
    }
}
