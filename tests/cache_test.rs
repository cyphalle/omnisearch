use omnisearch::cache::Cache;
use omnisearch::types::{Category, SearchResponse, SearchResult, SourceError};
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

    // Timestamps have one-second resolution: 1.5 s can read as 1 s elapsed,
    // which is not past a 1 s TTL. Wait long enough to always cross it.
    std::thread::sleep(Duration::from_millis(2100));

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

#[test]
fn test_cache_drops_errors_and_is_private() {
    let dir = tempfile::tempdir().unwrap();
    let cache_dir = dir.path().join("omnisearch");
    let cache = Cache::new(cache_dir.clone(), Duration::from_secs(3600));

    let mut response = make_response("secret query");
    response.errors.push(SourceError {
        source: "guardian".to_string(),
        error: "HTTP error: api-key=SECRET".to_string(),
    });
    cache.put("secret query", &None, &None, &response).unwrap();

    let cached = cache.get("secret query", &None, &None).unwrap().unwrap();
    assert!(cached.errors.is_empty());

    let entries: Vec<_> = std::fs::read_dir(&cache_dir).unwrap().collect();
    assert_eq!(entries.len(), 1);
    let file = entries[0].as_ref().unwrap().path();
    assert!(!std::fs::read_to_string(&file).unwrap().contains("SECRET"));

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let file_mode = std::fs::metadata(&file).unwrap().permissions().mode() & 0o777;
        let dir_mode = std::fs::metadata(&cache_dir).unwrap().permissions().mode() & 0o777;
        assert_eq!(file_mode, 0o600);
        assert_eq!(dir_mode, 0o700);
    }
}
