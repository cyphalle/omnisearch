use governor::{DefaultDirectRateLimiter, Quota, RateLimiter};
use std::collections::HashMap;
use std::num::NonZeroU32;
use std::sync::Arc;

pub struct RateLimiterRegistry {
    limiters: HashMap<String, Arc<DefaultDirectRateLimiter>>,
}

impl Default for RateLimiterRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl RateLimiterRegistry {
    pub fn new() -> Self {
        Self {
            limiters: HashMap::new(),
        }
    }

    pub fn register(&mut self, source: &str, requests_per_minute: u32) {
        let rpm = NonZeroU32::new(requests_per_minute).unwrap_or(NonZeroU32::new(60).unwrap());
        let quota = Quota::per_minute(rpm);
        let limiter = Arc::new(RateLimiter::direct(quota));
        self.limiters.insert(source.to_string(), limiter);
    }

    pub fn get(&self, source: &str) -> Option<Arc<DefaultDirectRateLimiter>> {
        self.limiters.get(source).cloned()
    }
}
