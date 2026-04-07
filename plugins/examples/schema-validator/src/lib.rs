//! OxideSEO WASM Plugin: JSON-LD Schema Validator
//!
//! Checks pages for valid JSON-LD structured data (`<script type="application/ld+json">`).
//! Reports issues for missing, invalid, or incomplete schema markup.
//!
//! Build: `cargo build --target wasm32-wasip2 --release`
//! Output: `target/wasm32-wasip2/release/schema_validator.wasm`

// TODO(phase-8): Wire wit-bindgen macro once the WIT toolchain is fully integrated.
// For now, this is a reference implementation showing the intended plugin logic.
//
// wit_bindgen::generate!({
//     world: "seo-rule-plugin",
//     path: "../../../src-tauri/wit",
// });

/// Check the body text for JSON-LD blocks and validate their structure.
///
/// In the full WASM integration, this function will be called by the
/// `evaluate` export with the `PluginParsedPage` data.
pub fn check_json_ld(body_text: &str) -> Vec<JsonLdIssue> {
    let mut issues = Vec::new();

    // Look for JSON-LD script blocks in body text.
    // Note: In the full integration, the host would extract these and pass
    // them as a dedicated field. For now we search the body text.
    let json_ld_pattern = "application/ld+json";

    if !body_text.contains(json_ld_pattern) {
        issues.push(JsonLdIssue {
            rule_id: "missing_schema".into(),
            severity: "info".into(),
            message: "No JSON-LD structured data found on this page".into(),
        });
        return issues;
    }

    // Extract JSON-LD blocks (simplified — real implementation would parse HTML).
    for block in extract_json_ld_blocks(body_text) {
        match serde_json::from_str::<serde_json::Value>(&block) {
            Ok(value) => {
                if value.get("@context").is_none() {
                    issues.push(JsonLdIssue {
                        rule_id: "missing_context".into(),
                        severity: "warning".into(),
                        message: "JSON-LD block is missing @context property".into(),
                    });
                }
                if value.get("@type").is_none() {
                    issues.push(JsonLdIssue {
                        rule_id: "missing_type".into(),
                        severity: "warning".into(),
                        message: "JSON-LD block is missing @type property".into(),
                    });
                }
            }
            Err(_) => {
                issues.push(JsonLdIssue {
                    rule_id: "invalid_json".into(),
                    severity: "error".into(),
                    message: "JSON-LD block contains invalid JSON".into(),
                });
            }
        }
    }

    issues
}

pub struct JsonLdIssue {
    pub rule_id: String,
    pub severity: String,
    pub message: String,
}

/// Extract JSON content from JSON-LD script blocks (simplified).
fn extract_json_ld_blocks(text: &str) -> Vec<String> {
    let mut blocks = Vec::new();
    let marker = "application/ld+json";

    let mut search_from = 0;
    while let Some(pos) = text[search_from..].find(marker) {
        let abs_pos = search_from + pos + marker.len();
        // Look for the next '>' after the type attribute.
        if let Some(gt_pos) = text[abs_pos..].find('>') {
            let content_start = abs_pos + gt_pos + 1;
            // Find the closing </script> tag.
            if let Some(end_pos) = text[content_start..].find("</script>") {
                let block = text[content_start..content_start + end_pos].trim();
                if !block.is_empty() {
                    blocks.push(block.to_string());
                }
                search_from = content_start + end_pos;
            } else {
                break;
            }
        } else {
            break;
        }
    }

    blocks
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_json_ld() {
        let issues = check_json_ld("<html><body>Hello world</body></html>");
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].rule_id, "missing_schema");
    }

    #[test]
    fn test_valid_json_ld() {
        let html = r#"<script type="application/ld+json">{"@context":"https://schema.org","@type":"WebPage","name":"Test"}</script>"#;
        let issues = check_json_ld(html);
        assert!(issues.is_empty());
    }

    #[test]
    fn test_missing_context() {
        let html = r#"<script type="application/ld+json">{"@type":"WebPage"}</script>"#;
        let issues = check_json_ld(html);
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].rule_id, "missing_context");
    }

    #[test]
    fn test_missing_type() {
        let html = r#"<script type="application/ld+json">{"@context":"https://schema.org"}</script>"#;
        let issues = check_json_ld(html);
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].rule_id, "missing_type");
    }

    #[test]
    fn test_invalid_json() {
        let html = r#"<script type="application/ld+json">{ invalid json }</script>"#;
        let issues = check_json_ld(html);
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].rule_id, "invalid_json");
    }
}
