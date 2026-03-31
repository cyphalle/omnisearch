use crate::error::OmniResult;
use crate::types::{Category, SearchResponse};
use sha2::{Digest, Sha256};
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

pub struct Cache {
    dir: PathBuf,
    ttl: Duration,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct CacheEntry {
    timestamp: u64,
    response: SearchResponse,
}

impl Cache {
    pub fn new(dir: PathBuf, ttl: Duration) -> Self {
        Self { dir, ttl }
    }

    pub fn default_dir() -> PathBuf {
        dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join("omnisearch")
    }

    pub fn get(
        &self,
        query: &str,
        categories: &Option<Vec<Category>>,
        sources: &Option<Vec<String>>,
    ) -> OmniResult<Option<SearchResponse>> {
        let path = self.cache_path(query, categories, sources);
        if !path.exists() {
            return Ok(None);
        }

        let content = std::fs::read_to_string(&path)?;
        let entry: CacheEntry = match serde_json::from_str(&content) {
            Ok(e) => e,
            Err(_) => {
                let _ = std::fs::remove_file(&path);
                return Ok(None);
            }
        };

        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        if now.saturating_sub(entry.timestamp) > self.ttl.as_secs() {
            let _ = std::fs::remove_file(&path);
            return Ok(None);
        }

        Ok(Some(entry.response))
    }

    pub fn put(
        &self,
        query: &str,
        categories: &Option<Vec<Category>>,
        sources: &Option<Vec<String>>,
        response: &SearchResponse,
    ) -> OmniResult<()> {
        std::fs::create_dir_all(&self.dir)?;

        let entry = CacheEntry {
            timestamp: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            response: response.clone(),
        };

        let path = self.cache_path(query, categories, sources);
        let content = serde_json::to_string(&entry)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    fn cache_path(
        &self,
        query: &str,
        categories: &Option<Vec<Category>>,
        sources: &Option<Vec<String>>,
    ) -> PathBuf {
        let key = Self::cache_key(query, categories, sources);
        self.dir.join(format!("{key}.json"))
    }

    fn cache_key(
        query: &str,
        categories: &Option<Vec<Category>>,
        sources: &Option<Vec<String>>,
    ) -> String {
        let mut hasher = Sha256::new();
        hasher.update(query.as_bytes());
        if let Some(cats) = categories {
            let mut cat_strs: Vec<String> = cats.iter().map(|c| c.to_string()).collect();
            cat_strs.sort();
            hasher.update(cat_strs.join(",").as_bytes());
        }
        if let Some(srcs) = sources {
            let mut sorted = srcs.clone();
            sorted.sort();
            hasher.update(sorted.join(",").as_bytes());
        }
        format!("{:x}", hasher.finalize())
    }
}
