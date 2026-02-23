use crate::error::OmniError;
use crate::rate_limit::RateLimiterRegistry;
use crate::sources::Source;
use crate::types::{SearchQuery, SearchResult, SourceError};
use futures::future::join_all;
use reqwest::Client;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::timeout;

pub enum ProgressEvent {
    Starting { source: String },
    Done { source: String, count: usize },
    Failed { source: String, error: String },
}

pub struct SearchEngine {
    client: Client,
    sources: Vec<Box<dyn Source>>,
    rate_limiters: Arc<RateLimiterRegistry>,
    timeout_secs: u64,
}

impl SearchEngine {
    pub fn new(
        client: Client,
        sources: Vec<Box<dyn Source>>,
        rate_limiters: RateLimiterRegistry,
        timeout_secs: u64,
    ) -> Self {
        Self {
            client,
            sources,
            rate_limiters: Arc::new(rate_limiters),
            timeout_secs,
        }
    }

    pub fn source_count(&self, query: &SearchQuery) -> usize {
        self.sources
            .iter()
            .filter(|s| {
                if let Some(ref cats) = query.categories {
                    if !cats.contains(&s.category()) {
                        return false;
                    }
                }
                if let Some(ref sources) = query.sources {
                    if !sources.contains(&s.name().to_string()) {
                        return false;
                    }
                }
                true
            })
            .count()
    }

    pub async fn search(
        &self,
        query: &SearchQuery,
        progress_tx: Option<mpsc::UnboundedSender<ProgressEvent>>,
    ) -> (Vec<SearchResult>, Vec<SourceError>, Vec<String>) {
        let filtered_sources: Vec<&Box<dyn Source>> = self
            .sources
            .iter()
            .filter(|s| {
                if let Some(ref cats) = query.categories {
                    if !cats.contains(&s.category()) {
                        return false;
                    }
                }
                if let Some(ref sources) = query.sources {
                    if !sources.contains(&s.name().to_string()) {
                        return false;
                    }
                }
                true
            })
            .collect();

        let sources_queried: Vec<String> =
            filtered_sources.iter().map(|s| s.name().to_string()).collect();

        let futures: Vec<_> = filtered_sources
            .into_iter()
            .map(|source| {
                let client = self.client.clone();
                let query_str = query.query.clone();
                let max_results = query.max_results;
                let source_name = source.name().to_string();
                let rate_limiters = self.rate_limiters.clone();
                let timeout_duration = Duration::from_secs(self.timeout_secs);
                let tx = progress_tx.clone();

                async move {
                    if let Some(ref tx) = tx {
                        let _ = tx.send(ProgressEvent::Starting {
                            source: source_name.clone(),
                        });
                    }

                    // Check rate limit
                    if let Some(limiter) = rate_limiters.get(&source_name) {
                        if limiter.check().is_err() {
                            if let Some(ref tx) = tx {
                                let _ = tx.send(ProgressEvent::Failed {
                                    source: source_name.clone(),
                                    error: "rate limited".into(),
                                });
                            }
                            return Err(SourceError {
                                source: source_name,
                                error: "Rate limited".to_string(),
                            });
                        }
                    }

                    let result = match timeout(
                        timeout_duration,
                        source.search(&client, &query_str, max_results),
                    )
                    .await
                    {
                        Ok(Ok(results)) => {
                            if let Some(ref tx) = tx {
                                let _ = tx.send(ProgressEvent::Done {
                                    source: source_name.clone(),
                                    count: results.len(),
                                });
                            }
                            Ok(results)
                        }
                        Ok(Err(e)) => {
                            let err_str = e.to_string();
                            if let Some(ref tx) = tx {
                                let _ = tx.send(ProgressEvent::Failed {
                                    source: source_name.clone(),
                                    error: err_str.clone(),
                                });
                            }
                            Err(SourceError {
                                source: source_name,
                                error: err_str,
                            })
                        }
                        Err(_) => {
                            if let Some(ref tx) = tx {
                                let _ = tx.send(ProgressEvent::Failed {
                                    source: source_name.clone(),
                                    error: "timeout".into(),
                                });
                            }
                            Err(SourceError {
                                source: source_name.clone(),
                                error: OmniError::Timeout {
                                    src: source_name,
                                }
                                .to_string(),
                            })
                        }
                    };

                    result
                }
            })
            .collect();

        let results = join_all(futures).await;

        let mut all_results = Vec::new();
        let mut errors = Vec::new();

        for result in results {
            match result {
                Ok(mut items) => all_results.append(&mut items),
                Err(err) => errors.push(err),
            }
        }

        (all_results, errors, sources_queried)
    }
}
