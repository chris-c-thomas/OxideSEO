-- migrations/003_ai_analysis.sql
-- Phase 7: AI integration — analysis caching, usage tracking, crawl summaries,
-- and body text storage for LLM input.

-- Cached AI analysis results, keyed by content hash for reuse on re-crawl.
CREATE TABLE IF NOT EXISTS ai_analyses (
  id              INTEGER PRIMARY KEY AUTOINCREMENT,
  crawl_id        TEXT NOT NULL REFERENCES crawls(id),
  page_id         INTEGER NOT NULL REFERENCES pages(id),
  analysis_type   TEXT NOT NULL,  -- 'content_score', 'meta_desc', 'title_tag', 'structured_data', 'accessibility'
  content_hash    BLOB NOT NULL,  -- blake3 hash of input content for cache invalidation
  provider        TEXT NOT NULL,  -- 'openai', 'anthropic', 'ollama'
  model           TEXT NOT NULL,
  result_json     TEXT NOT NULL,
  input_tokens    INTEGER NOT NULL DEFAULT 0,
  output_tokens   INTEGER NOT NULL DEFAULT 0,
  cost_usd        REAL NOT NULL DEFAULT 0.0,
  latency_ms      INTEGER NOT NULL DEFAULT 0,
  created_at      TEXT NOT NULL DEFAULT (datetime('now')),
  UNIQUE(crawl_id, page_id, analysis_type)
);
CREATE INDEX IF NOT EXISTS idx_ai_analyses_crawl ON ai_analyses(crawl_id);
CREATE INDEX IF NOT EXISTS idx_ai_analyses_page ON ai_analyses(page_id);
CREATE INDEX IF NOT EXISTS idx_ai_analyses_hash ON ai_analyses(content_hash);

-- Per-crawl AI usage tracking for budget enforcement and cost UI.
CREATE TABLE IF NOT EXISTS ai_usage (
  id                  INTEGER PRIMARY KEY AUTOINCREMENT,
  crawl_id            TEXT NOT NULL REFERENCES crawls(id),
  provider            TEXT NOT NULL,
  model               TEXT NOT NULL,
  total_input_tokens  INTEGER NOT NULL DEFAULT 0,
  total_output_tokens INTEGER NOT NULL DEFAULT 0,
  total_cost_usd      REAL NOT NULL DEFAULT 0.0,
  request_count       INTEGER NOT NULL DEFAULT 0,
  updated_at          TEXT NOT NULL DEFAULT (datetime('now')),
  UNIQUE(crawl_id, provider, model)
);
CREATE INDEX IF NOT EXISTS idx_ai_usage_crawl ON ai_usage(crawl_id);

-- Crawl-level AI summary (one per crawl, generated on demand).
CREATE TABLE IF NOT EXISTS ai_crawl_summaries (
  id              INTEGER PRIMARY KEY AUTOINCREMENT,
  crawl_id        TEXT NOT NULL UNIQUE REFERENCES crawls(id),
  provider        TEXT NOT NULL,
  model           TEXT NOT NULL,
  summary_json    TEXT NOT NULL,
  input_tokens    INTEGER NOT NULL DEFAULT 0,
  output_tokens   INTEGER NOT NULL DEFAULT 0,
  cost_usd        REAL NOT NULL DEFAULT 0.0,
  created_at      TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Store truncated visible text for AI analysis without re-fetching.
ALTER TABLE pages ADD COLUMN body_text TEXT;
