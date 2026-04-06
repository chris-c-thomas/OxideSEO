//! JavaScript rendering pipeline using hidden Tauri webview windows.
//!
//! For pages detected as potentially JS-rendered (low word count + multiple scripts),
//! creates a hidden webview to execute JavaScript, then extracts the rendered HTML
//! for re-parsing with lol_html.

use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context, Result};
use tauri::Listener;
use tokio::sync::Semaphore;

use crate::commands::crawl::CrawlConfig;
use crate::crawler::ParsedPage;

/// Determines whether a parsed page should be re-rendered with JavaScript.
///
/// Checks (in order):
/// 1. JS rendering must be enabled in config
/// 2. URL must not match any never-render pattern
/// 3. URL must match an always-render pattern (if any), OR
/// 4. Heuristic: word count < 50 AND >= 2 script tags (likely a SPA)
pub(crate) fn should_js_render(
    page: &ParsedPage,
    config: &CrawlConfig,
    patterns: &super::engine::CompiledPatterns,
) -> bool {
    if !config.enable_js_rendering {
        return false;
    }
    if patterns.js_never.iter().any(|r| r.is_match(&page.url)) {
        return false;
    }
    if patterns.js_always.iter().any(|r| r.is_match(&page.url)) {
        return true;
    }
    // Heuristic: sparse content with multiple scripts suggests a SPA.
    page.word_count < 50 && page.scripts.len() >= 2
}

/// Manages concurrent JS rendering via hidden Tauri webview windows.
pub struct JsRenderer {
    app_handle: tauri::AppHandle,
    semaphore: Arc<Semaphore>,
    /// Monotonic counter for unique webview labels.
    counter: std::sync::atomic::AtomicU64,
}

impl JsRenderer {
    /// Create a new renderer with the given concurrency limit.
    pub fn new(app_handle: tauri::AppHandle, max_concurrent: u32) -> Self {
        Self {
            app_handle,
            semaphore: Arc::new(Semaphore::new(max_concurrent as usize)),
            counter: std::sync::atomic::AtomicU64::new(0),
        }
    }

    /// Render a URL in a hidden webview and return the rendered HTML.
    ///
    /// 1. Acquires a semaphore permit
    /// 2. Creates a hidden `WebviewWindow`
    /// 3. Navigates to the URL, waits for load + 2s for JS execution
    /// 4. Extracts rendered HTML via Tauri event channel
    /// 5. Closes the webview
    pub async fn render(&self, url: &str) -> Result<Vec<u8>> {
        let _permit = self
            .semaphore
            .acquire()
            .await
            .context("JS render semaphore closed")?;

        let id = self
            .counter
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let label = format!("js-render-{}", id);
        let event_name = format!("js-render-result-{}", id);

        tracing::debug!(url = %url, label = %label, "Starting JS render");

        // Set up a oneshot channel to receive the rendered HTML via Tauri events.
        let (tx, rx) = tokio::sync::oneshot::channel::<String>();
        let tx = std::sync::Mutex::new(Some(tx));

        let unlisten = self.app_handle.listen(&event_name, move |event| {
            if let Some(sender) = tx.lock().ok().and_then(|mut opt| opt.take()) {
                let _ = sender.send(event.payload().to_string());
            }
        });

        // Create a hidden webview window navigating to the target URL.
        let window = tauri::WebviewWindowBuilder::new(
            &self.app_handle,
            &label,
            tauri::WebviewUrl::External(url.parse().context("Invalid URL for JS render")?),
        )
        .title("OxideSEO JS Render")
        .visible(false)
        .build()
        .context("Failed to create hidden webview for JS render")?;

        // Wait for the page to load and JS to execute.
        tokio::time::sleep(Duration::from_secs(2)).await;

        // Inject script that extracts the rendered HTML and emits it as a Tauri event.
        let inject_js = format!(
            r#"
            if (window.__TAURI__) {{
                window.__TAURI__.event.emit('{}', document.documentElement.outerHTML);
            }}
            "#,
            event_name
        );

        window
            .eval(&inject_js)
            .context("Failed to eval JS in render webview")?;

        // Wait for the event with a timeout.
        let html_result = tokio::time::timeout(Duration::from_secs(5), rx).await;

        // Clean up.
        self.app_handle.unlisten(unlisten);
        if let Err(e) = window.close() {
            tracing::warn!(label = %label, error = %e, "Failed to close JS render webview");
        }

        match html_result {
            Ok(Ok(payload)) => {
                // The payload is JSON-encoded — parse it to get the raw HTML string.
                let html_str = serde_json::from_str::<String>(&payload).unwrap_or(payload);
                tracing::debug!(
                    url = %url,
                    html_len = html_str.len(),
                    "JS render complete"
                );
                Ok(html_str.into_bytes())
            }
            Ok(Err(_)) => {
                anyhow::bail!("JS render event channel dropped for URL: {}", url)
            }
            Err(_) => {
                anyhow::bail!("JS render timed out for URL: {}", url)
            }
        }
    }
}
