//! Politeness controls: per-domain rate limiting and concurrency caps.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::sync::{Mutex, Semaphore};

/// Manages per-domain politeness: minimum delay between requests and
/// per-host concurrency limits.
pub struct PolitenessController {
    /// Minimum delay between requests to the same host.
    default_delay: Duration,
    /// Per-host concurrency semaphores.
    per_host_semaphores: Mutex<HashMap<String, Arc<Semaphore>>>,
    /// Default per-host concurrency limit.
    per_host_limit: usize,
    /// Last request time per domain.
    last_request_times: Mutex<HashMap<String, Instant>>,
    /// Per-domain delay overrides (from robots.txt Crawl-delay).
    delay_overrides: Mutex<HashMap<String, Duration>>,
}

impl PolitenessController {
    pub fn new(default_delay_ms: u64, per_host_limit: usize) -> Self {
        Self {
            default_delay: Duration::from_millis(default_delay_ms),
            per_host_semaphores: Mutex::new(HashMap::new()),
            per_host_limit,
            last_request_times: Mutex::new(HashMap::new()),
            delay_overrides: Mutex::new(HashMap::new()),
        }
    }

    /// Set a custom delay for a specific domain (e.g., from robots.txt Crawl-delay).
    pub async fn set_domain_delay(&self, domain: &str, delay: Duration) {
        self.delay_overrides
            .lock()
            .await
            .insert(domain.to_string(), delay);
    }

    /// Get the per-host semaphore, creating one if it doesn't exist.
    pub async fn acquire_host_permit(&self, domain: &str) -> Arc<Semaphore> {
        let mut semaphores = self.per_host_semaphores.lock().await;
        semaphores
            .entry(domain.to_string())
            .or_insert_with(|| Arc::new(Semaphore::new(self.per_host_limit)))
            .clone()
    }

    /// Wait until the politeness delay has elapsed for the given domain.
    pub async fn wait_for_politeness(&self, domain: &str) {
        let delay = {
            let overrides = self.delay_overrides.lock().await;
            overrides.get(domain).copied().unwrap_or(self.default_delay)
        };

        let elapsed = {
            let times = self.last_request_times.lock().await;
            times.get(domain).map(|t| t.elapsed())
        };

        if let Some(elapsed) = elapsed {
            if elapsed < delay {
                tokio::time::sleep(delay - elapsed).await;
            }
        }

        // Record this request time.
        self.last_request_times
            .lock()
            .await
            .insert(domain.to_string(), Instant::now());
    }
}
