//! HTTP fetcher: configurable reqwest client with redirect chain tracking.

use std::time::{Duration, Instant};

use anyhow::Result;
use reqwest::header::{HeaderMap, HeaderValue};

use crate::commands::crawl::CrawlConfig;
use crate::crawler::{FetchResult, RedirectHop};

/// Default user-agent string.
const DEFAULT_USER_AGENT: &str = "OxideSEO/0.1 (+https://github.com/oxide-seo/oxide-seo)";

/// Maximum redirect hops before aborting.
const MAX_REDIRECTS: usize = 10;

/// Maximum response body size (default 10MB).
const MAX_BODY_SIZE: usize = 10 * 1024 * 1024;

/// Wrapper around reqwest::Client with crawl-specific configuration.
pub struct Fetcher {
    client: reqwest::Client,
    max_body_size: usize,
}

impl Fetcher {
    /// Build a new Fetcher from the crawl configuration.
    pub fn new(config: &CrawlConfig) -> Result<Self> {
        let user_agent = config.user_agent.as_deref().unwrap_or(DEFAULT_USER_AGENT);

        let mut default_headers = HeaderMap::new();
        for (key, value) in &config.custom_headers {
            if let (Ok(name), Ok(val)) = (
                reqwest::header::HeaderName::from_bytes(key.as_bytes()),
                HeaderValue::from_str(value),
            ) {
                default_headers.insert(name, val);
            }
        }

        let client = reqwest::Client::builder()
            .user_agent(user_agent)
            .timeout(Duration::from_secs(config.request_timeout_secs as u64))
            .connect_timeout(Duration::from_secs(10))
            // Disable automatic redirects — we track chains manually.
            .redirect(reqwest::redirect::Policy::none())
            .pool_max_idle_per_host(config.per_host_concurrency as usize)
            .pool_idle_timeout(Duration::from_secs(90))
            .tcp_keepalive(Duration::from_secs(60))
            .gzip(true)
            .brotli(true)
            .default_headers(default_headers)
            .build()?;

        Ok(Self {
            client,
            max_body_size: MAX_BODY_SIZE,
        })
    }

    /// Fetch a URL, manually following redirect chains.
    ///
    /// Returns the final response data along with the full redirect chain.
    pub async fn fetch(&self, url: &str) -> Result<FetchResult> {
        let start = Instant::now();
        let mut current_url = url.to_string();
        let mut redirect_chain: Vec<RedirectHop> = Vec::new();

        for _hop in 0..=MAX_REDIRECTS {
            let response = self.client.get(&current_url).send().await?;

            let status = response.status().as_u16();
            let headers: Vec<(String, String)> = response
                .headers()
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
                .collect();

            let content_type = response
                .headers()
                .get(reqwest::header::CONTENT_TYPE)
                .and_then(|v| v.to_str().ok())
                .map(|s| s.to_string());

            // If redirect, record hop and follow.
            if (300..400).contains(&(status as i32)) {
                if let Some(location) = response.headers().get(reqwest::header::LOCATION) {
                    let location_str = location.to_str().unwrap_or("").to_string();
                    redirect_chain.push(RedirectHop {
                        url: current_url.clone(),
                        status_code: status,
                    });

                    // Resolve relative redirect URLs.
                    current_url = match url::Url::parse(&location_str) {
                        Ok(absolute) => absolute.to_string(),
                        Err(_) => {
                            let base = url::Url::parse(&current_url)?;
                            base.join(&location_str)?.to_string()
                        }
                    };
                    continue;
                }
            }

            // Read body with size cap.
            let body_bytes = self.read_body_capped(response).await?;
            let body_size = body_bytes.len();
            let response_time_ms = start.elapsed().as_millis() as u32;

            return Ok(FetchResult {
                url: url.to_string(),
                final_url: current_url,
                status_code: status,
                headers,
                body_bytes,
                body_size,
                content_type,
                response_time_ms,
                redirect_chain,
            });
        }

        anyhow::bail!(
            "Redirect chain exceeded {} hops for URL: {}",
            MAX_REDIRECTS,
            url
        )
    }

    /// Read response body up to `max_body_size` bytes.
    async fn read_body_capped(&self, response: reqwest::Response) -> Result<Vec<u8>> {
        // TODO(phase-2): Stream body, compute blake3 hash as bytes arrive,
        // abort if exceeding max_body_size.
        let bytes = response.bytes().await?;
        if bytes.len() > self.max_body_size {
            Ok(bytes[..self.max_body_size].to_vec())
        } else {
            Ok(bytes.to_vec())
        }
    }
}
