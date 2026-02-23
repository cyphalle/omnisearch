use crate::types::SearchResult;
use std::collections::HashSet;

pub fn dedup_and_rank(mut results: Vec<SearchResult>) -> Vec<SearchResult> {
    let mut seen_urls = HashSet::new();
    results.retain(|r| {
        let normalized = normalize_url(&r.url);
        seen_urls.insert(normalized)
    });

    results.sort_by(|a, b| {
        b.relevance_score
            .partial_cmp(&a.relevance_score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    results
}

fn normalize_url(url: &str) -> String {
    let url = url.trim_end_matches('/');
    let url = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))
        .unwrap_or(url);
    let url = url.strip_prefix("www.").unwrap_or(url);
    url.to_lowercase()
}
