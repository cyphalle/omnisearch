use omnisearch::cache::Cache;
use omnisearch::types::{Category, SearchResponse, SearchResult};
use std::time::Duration;

fn make_response(query: &str) -> SearchResponse {
    SearchResponse {
        query: query.to_string(),
        total_results: 1,
        sources_queried: vec!["wikipedia".to_string()],
        results: vec![SearchResult {
            title: "Rust".to_string(),
            url: "https://en.wikipedia.org/wiki/Rust_(programming_language)".to_string(),
            snippet: "Rust is a language".to_string(),
            source: "wikipedia".to_string(),
            category: Category::Reference,
            published: None,
            author: None,
            relevance_score: 0.95,
        }],
        errors: vec![],
    }
}

#[test]
fn test_cache_roundtrip() {
    let dir = tempfile::tempdir().unwrap();
    let cache = Cache::new(dir.path().to_path_buf(), Duration::from_secs(3600));

    let response = make_response("rust programming");

    cache
        .put("rust programming", &None, &None, &response)
        .unwrap();
    let cached = cache.get("rust programming", &None, &None).unwrap();

    assert!(cached.is_some());
    let cached = cached.unwrap();
    assert_eq!(cached.query, "rust programming");
    assert_eq!(cached.results.len(), 1);
    assert_eq!(cached.results[0].title, "Rust");
}

#[test]
fn test_cache_miss() {
    let dir = tempfile::tempdir().unwrap();
    let cache = Cache::new(dir.path().to_path_buf(), Duration::from_secs(3600));

    let result = cache.get("nonexistent", &None, &None).unwrap();
    assert!(result.is_none());
}

#[test]
fn test_cache_expired() {
    let dir = tempfile::tempdir().unwrap();
    let cache = Cache::new(dir.path().to_path_buf(), Duration::from_secs(1));

    let response = make_response("test");
    cache.put("test", &None, &None, &response).unwrap();

    std::thread::sleep(Duration::from_millis(1500));

    let cached = cache.get("test", &None, &None).unwrap();
    assert!(cached.is_none());
}

#[test]
fn test_cache_different_categories() {
    let dir = tempfile::tempdir().unwrap();
    let cache = Cache::new(dir.path().to_path_buf(), Duration::from_secs(3600));

    let response = make_response("test");

    let cats_a = Some(vec![Category::News]);
    let cats_b = Some(vec![Category::Academic]);

    cache.put("test", &cats_a, &None, &response).unwrap();

    // Different category = cache miss
    let cached = cache.get("test", &cats_b, &None).unwrap();
    assert!(cached.is_none());

    // Same category = cache hit
    let cached = cache.get("test", &cats_a, &None).unwrap();
    assert!(cached.is_some());
}
