//! Integration tests for the AI analysis layer.
//!
//! Tests storage round-trips, caching behavior, usage tracking,
//! and prompt construction. Does NOT require real LLM API keys.

use oxide_seo_lib::ai::prompts;
use oxide_seo_lib::ai::provider::ResponseFormat;
use oxide_seo_lib::storage::db::Database;
use oxide_seo_lib::storage::models::{AiAnalysisRow, AiCrawlSummaryRow};
use oxide_seo_lib::storage::queries;

/// Create a temporary in-memory database with all migrations applied.
fn test_db() -> Database {
    Database::new_in_memory().expect("Failed to create in-memory database")
}

/// Insert a test crawl into the database.
fn insert_test_crawl(db: &Database, crawl_id: &str, start_url: &str) {
    db.with_conn(|conn| {
        conn.execute(
            "INSERT INTO crawls (id, start_url, config_json, status, started_at, urls_crawled, urls_errored) \
             VALUES (?1, ?2, '{}', 'completed', datetime('now'), 10, 0)",
            rusqlite::params![crawl_id, start_url],
        )?;
        Ok(())
    })
    .expect("Failed to insert test crawl");
}

/// Insert a test page with body text.
fn insert_test_page(db: &Database, crawl_id: &str, page_id: i64, url: &str, body_text: &str) {
    let hash = blake3::hash(url.as_bytes());
    db.with_conn(|conn| {
        conn.execute(
            r#"INSERT INTO pages (id, crawl_id, url, url_hash, depth, status_code, state, body_text,
                                  title, meta_desc, is_js_rendered)
               VALUES (?1, ?2, ?3, ?4, 0, 200, 'analyzed', ?5, 'Test Page', 'A test page description', 0)"#,
            rusqlite::params![page_id, crawl_id, url, hash.as_bytes().to_vec(), body_text],
        )?;
        Ok(())
    })
    .expect("Failed to insert test page");
}

fn make_ai_analysis(
    crawl_id: &str,
    page_id: i64,
    analysis_type: &str,
    result_json: &str,
) -> AiAnalysisRow {
    AiAnalysisRow {
        id: 0,
        crawl_id: crawl_id.to_string(),
        page_id,
        analysis_type: analysis_type.to_string(),
        provider: "openai".to_string(),
        model: "gpt-4o".to_string(),
        result_json: result_json.to_string(),
        input_tokens: 500,
        output_tokens: 100,
        cost_usd: 0.005,
        latency_ms: 1200,
        created_at: String::new(),
    }
}

// ---------------------------------------------------------------------------
// AI analysis storage tests
// ---------------------------------------------------------------------------

#[test]
fn test_insert_and_retrieve_ai_analysis() {
    let db = test_db();
    let crawl_id = "crawl-ai-1";
    insert_test_crawl(&db, crawl_id, "https://example.com");
    insert_test_page(&db, crawl_id, 1, "https://example.com/", "Hello world");

    let row = make_ai_analysis(crawl_id, 1, "content_score", r#"{"overallScore": 75}"#);
    let content_hash = blake3::hash(b"Hello world").as_bytes().to_vec();

    db.with_conn(|conn| {
        queries::insert_ai_analysis_with_hash(conn, &row, &content_hash)?;
        Ok(())
    })
    .unwrap();

    // Retrieve by page.
    let results = db
        .with_conn(|conn| queries::select_ai_analyses_for_page(conn, crawl_id, 1))
        .unwrap();

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].analysis_type, "content_score");
    assert_eq!(results[0].provider, "openai");
    assert_eq!(results[0].input_tokens, 500);
    assert_eq!(results[0].cost_usd, 0.005);
}

#[test]
fn test_cache_lookup_by_content_hash() {
    let db = test_db();
    let crawl_id = "crawl-ai-2";
    insert_test_crawl(&db, crawl_id, "https://example.com");
    insert_test_page(&db, crawl_id, 1, "https://example.com/", "Some content");

    let row = make_ai_analysis(crawl_id, 1, "meta_desc", r#"{"suggested": "A great page"}"#);
    let content_hash = blake3::hash(b"Some content").as_bytes().to_vec();

    db.with_conn(|conn| {
        queries::insert_ai_analysis_with_hash(conn, &row, &content_hash)?;
        Ok(())
    })
    .unwrap();

    // Cache hit: same hash and type.
    let cached = db
        .with_conn(|conn| {
            queries::select_ai_analysis_by_hash(conn, crawl_id, &content_hash, "meta_desc")
        })
        .unwrap();
    assert!(cached.is_some());
    assert_eq!(
        cached.unwrap().result_json,
        r#"{"suggested": "A great page"}"#
    );

    // Cache miss: different type.
    let miss = db
        .with_conn(|conn| {
            queries::select_ai_analysis_by_hash(conn, crawl_id, &content_hash, "content_score")
        })
        .unwrap();
    assert!(miss.is_none());

    // Cache miss: different hash.
    let different_hash = blake3::hash(b"Different content").as_bytes().to_vec();
    let miss2 = db
        .with_conn(|conn| {
            queries::select_ai_analysis_by_hash(conn, crawl_id, &different_hash, "meta_desc")
        })
        .unwrap();
    assert!(miss2.is_none());
}

#[test]
fn test_ai_usage_tracking() {
    let db = test_db();
    let crawl_id = "crawl-ai-3";
    insert_test_crawl(&db, crawl_id, "https://example.com");

    // First request.
    db.with_conn(|conn| {
        queries::upsert_ai_usage(conn, crawl_id, "openai", "gpt-4o", 500, 100, 0.005)?;
        Ok(())
    })
    .unwrap();

    // Second request (same model).
    db.with_conn(|conn| {
        queries::upsert_ai_usage(conn, crawl_id, "openai", "gpt-4o", 300, 80, 0.003)?;
        Ok(())
    })
    .unwrap();

    let usage = db
        .with_conn(|conn| queries::select_ai_usage(conn, crawl_id))
        .unwrap();

    assert_eq!(usage.len(), 1);
    assert_eq!(usage[0].total_input_tokens, 800);
    assert_eq!(usage[0].total_output_tokens, 180);
    assert!((usage[0].total_cost_usd - 0.008).abs() < 1e-6);
    assert_eq!(usage[0].request_count, 2);
}

#[test]
fn test_ai_usage_multiple_models() {
    let db = test_db();
    let crawl_id = "crawl-ai-4";
    insert_test_crawl(&db, crawl_id, "https://example.com");

    db.with_conn(|conn| {
        queries::upsert_ai_usage(conn, crawl_id, "openai", "gpt-4o", 500, 100, 0.005)?;
        queries::upsert_ai_usage(conn, crawl_id, "anthropic", "claude-sonnet", 400, 90, 0.004)?;
        Ok(())
    })
    .unwrap();

    let usage = db
        .with_conn(|conn| queries::select_ai_usage(conn, crawl_id))
        .unwrap();

    assert_eq!(usage.len(), 2);
}

#[test]
fn test_crawl_summary_storage() {
    let db = test_db();
    let crawl_id = "crawl-ai-5";
    insert_test_crawl(&db, crawl_id, "https://example.com");

    let summary = AiCrawlSummaryRow {
        id: 0,
        crawl_id: crawl_id.to_string(),
        provider: "openai".to_string(),
        model: "gpt-4o".to_string(),
        summary_json: r#"{"summary": "Good site"}"#.to_string(),
        input_tokens: 1000,
        output_tokens: 500,
        cost_usd: 0.02,
        created_at: String::new(),
    };

    db.with_conn(|conn| {
        queries::insert_ai_crawl_summary(conn, &summary)?;
        Ok(())
    })
    .unwrap();

    let retrieved = db
        .with_conn(|conn| queries::select_ai_crawl_summary(conn, crawl_id))
        .unwrap();

    assert!(retrieved.is_some());
    let r = retrieved.unwrap();
    assert_eq!(r.provider, "openai");
    assert_eq!(r.input_tokens, 1000);
}

#[test]
fn test_crawl_summary_not_found() {
    let db = test_db();
    let crawl_id = "crawl-ai-nonexistent";
    insert_test_crawl(&db, crawl_id, "https://example.com");

    let result = db
        .with_conn(|conn| queries::select_ai_crawl_summary(conn, crawl_id))
        .unwrap();
    assert!(result.is_none());
}

#[test]
fn test_pages_for_ai_analysis_filters() {
    let db = test_db();
    let crawl_id = "crawl-ai-6";
    insert_test_crawl(&db, crawl_id, "https://example.com");
    insert_test_page(&db, crawl_id, 1, "https://example.com/", "Page one content");
    insert_test_page(
        &db,
        crawl_id,
        2,
        "https://example.com/about",
        "Page two content",
    );

    // Insert an issue for page 1 only.
    db.with_conn(|conn| {
        conn.execute(
            "INSERT INTO issues (crawl_id, page_id, rule_id, severity, category, message) VALUES (?1, ?2, 'test.rule', 'warning', 'meta', 'Test issue')",
            rusqlite::params![crawl_id, 1],
        )?;
        Ok(())
    })
    .unwrap();

    // All pages.
    let all = db
        .with_conn(|conn| queries::select_pages_for_ai_analysis(conn, crawl_id, false, false, 100))
        .unwrap();
    assert_eq!(all.len(), 2);

    // Only with issues.
    let with_issues = db
        .with_conn(|conn| queries::select_pages_for_ai_analysis(conn, crawl_id, true, false, 100))
        .unwrap();
    assert_eq!(with_issues.len(), 1);
    assert_eq!(with_issues[0].id, 1);

    // Max pages limit.
    let limited = db
        .with_conn(|conn| queries::select_pages_for_ai_analysis(conn, crawl_id, false, false, 1))
        .unwrap();
    assert_eq!(limited.len(), 1);
}

// ---------------------------------------------------------------------------
// Prompt construction tests
// ---------------------------------------------------------------------------

#[test]
fn test_content_quality_prompt_structure() {
    let req = prompts::content_quality_request(
        "This is some page content about SEO best practices.",
        "https://example.com/seo-guide",
        Some("SEO Guide"),
    );

    assert!(req.system_prompt.contains("overallScore"));
    assert!(req.user_prompt.contains("https://example.com/seo-guide"));
    assert!(req.user_prompt.contains("SEO Guide"));
    assert!(req.user_prompt.contains("SEO best practices"));
    assert_eq!(req.response_format, ResponseFormat::Json);
    assert!(req.max_tokens > 0);
    assert!(req.temperature < 1.0);
}

#[test]
fn test_meta_description_prompt_includes_current() {
    let req = prompts::meta_description_request(
        "Page about coffee brewing",
        Some("Coffee Brewing Guide"),
        Some("Old description here"),
    );

    assert!(req.user_prompt.contains("Old description here"));
    assert!(req.user_prompt.contains("Coffee Brewing Guide"));
    assert_eq!(req.response_format, ResponseFormat::Json);
}

#[test]
fn test_crawl_summary_prompt_includes_stats() {
    let stats = r#"{"totalPages": 500, "errors": 12, "warnings": 45}"#;
    let req = prompts::crawl_summary_request(stats);

    assert!(req.user_prompt.contains("500"));
    assert!(req.user_prompt.contains("12"));
    assert!(req.system_prompt.contains("topActions"));
    assert_eq!(req.response_format, ResponseFormat::Json);
}

// ---------------------------------------------------------------------------
// Export streaming tests
// ---------------------------------------------------------------------------

#[test]
fn test_for_each_ai_analysis_streaming() {
    let db = test_db();
    let crawl_id = "crawl-ai-7";
    insert_test_crawl(&db, crawl_id, "https://example.com");
    insert_test_page(&db, crawl_id, 1, "https://example.com/", "Content");

    let content_hash = blake3::hash(b"Content").as_bytes().to_vec();

    // Insert two analyses for the page.
    let row1 = make_ai_analysis(crawl_id, 1, "content_score", r#"{"score": 80}"#);
    let row2 = make_ai_analysis(crawl_id, 1, "meta_desc", r#"{"suggested": "Desc"}"#);

    db.with_conn(|conn| {
        queries::insert_ai_analysis_with_hash(conn, &row1, &content_hash)?;
        queries::insert_ai_analysis_with_hash(conn, &row2, &content_hash)?;
        Ok(())
    })
    .unwrap();

    let mut count = 0;
    db.with_conn(|conn| {
        queries::for_each_ai_analysis(conn, crawl_id, |_analysis| {
            count += 1;
            Ok(())
        })
    })
    .unwrap();

    assert_eq!(count, 2);
}
