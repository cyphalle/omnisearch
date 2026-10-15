use crate::error::{OmniError, OmniResult};
use crate::fsutil::{create_private_dir, write_private_file};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(Default)]
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
    #[serde(default = "default_cache_ttl")]
    pub cache_ttl_secs: u64,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            timeout_secs: default_timeout(),
            max_results: default_max_results(),
            default_format: default_format(),
            cache_ttl_secs: default_cache_ttl(),
        }
    }
}

fn default_timeout() -> u64 {
    8
}
fn default_max_results() -> usize {
    20
}
fn default_format() -> String {
    "json".to_string()
}
fn default_cache_ttl() -> u64 {
    3600
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub rate_limit_per_minute: Option<u32>,
}

/// Show only the last 4 characters of a secret; short secrets are fully hidden.
pub fn mask_secret(secret: &str) -> String {
    let chars: Vec<char> = secret.chars().collect();
    if chars.len() <= 8 {
        return "****".to_string();
    }
    let tail: String = chars[chars.len() - 4..].iter().collect();
    format!("****{tail}")
}

fn default_true() -> bool {
    true
}


impl Config {
    pub fn config_dir() -> OmniResult<PathBuf> {
        dirs::config_dir()
            .map(|d| d.join("omnisearch"))
            .ok_or_else(|| OmniError::Config("cannot determine the user config directory".into()))
    }

    pub fn config_path() -> OmniResult<PathBuf> {
        Ok(Self::config_dir()?.join("config.toml"))
    }

    pub fn load() -> OmniResult<Self> {
        let path = Self::config_path()?;
        if path.exists() {
            let content = std::fs::read_to_string(&path)?;
            toml::from_str(&content).map_err(|e| OmniError::Config(e.to_string()))
        } else {
            Ok(Config::default())
        }
    }

    /// Write the config with mode 0600 in a 0700 directory: it holds API keys.
    pub fn save(&self) -> OmniResult<()> {
        create_private_dir(&Self::config_dir()?)?;
        let content =
            toml::to_string_pretty(self).map_err(|e| OmniError::Config(e.to_string()))?;
        write_private_file(&Self::config_path()?, content.as_bytes())?;
        Ok(())
    }

    /// Copy of the config with every API key masked to its last 4 characters.
    pub fn masked(&self) -> Self {
        let mut copy = self.clone();
        for value in copy.api_keys.values_mut() {
            *value = mask_secret(value);
        }
        copy
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
timeout_secs = 8
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mask_secret_keeps_last_four() {
        assert_eq!(mask_secret("abcdefghijkl"), "****ijkl");
        assert_eq!(mask_secret("short"), "****");
    }

    #[test]
    fn masked_config_hides_keys() {
        let mut config = Config::default();
        config.set_api_key("guardian".into(), "super-secret-1234".into());
        let shown = toml::to_string_pretty(&config.masked()).unwrap();
        assert!(!shown.contains("super-secret"));
        assert!(shown.contains("****1234"));
    }
}
