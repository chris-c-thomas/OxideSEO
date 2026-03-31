-- migrations/001_initial.sql
-- OxideSEO core schema: crawls, pages, links, issues.

CREATE TABLE IF NOT EXISTS crawls (
  id            TEXT PRIMARY KEY,
  start_url     TEXT NOT NULL,
  config_json   TEXT NOT NULL,
  status        TEXT NOT NULL DEFAULT 'created',
  started_at    TEXT,
  completed_at  TEXT,
  urls_crawled  INTEGER NOT NULL DEFAULT 0,
  urls_errored  INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS pages (
  id            INTEGER PRIMARY KEY AUTOINCREMENT,
  crawl_id      TEXT NOT NULL REFERENCES crawls(id),
  url           TEXT NOT NULL,
  url_hash      BLOB NOT NULL,
  depth         INTEGER NOT NULL,
  status_code   INTEGER,
  content_type  TEXT,
  response_time_ms INTEGER,
  body_size     INTEGER,
  title         TEXT,
  meta_desc     TEXT,
  h1            TEXT,
  canonical     TEXT,
  robots_directives TEXT,
  state         TEXT NOT NULL DEFAULT 'discovered',
  fetched_at    TEXT,
  error_message TEXT
);

CREATE INDEX IF NOT EXISTS idx_pages_crawl ON pages(crawl_id);
CREATE INDEX IF NOT EXISTS idx_pages_state ON pages(crawl_id, state);
CREATE UNIQUE INDEX IF NOT EXISTS idx_pages_url ON pages(crawl_id, url_hash);

CREATE TABLE IF NOT EXISTS links (
  id            INTEGER PRIMARY KEY AUTOINCREMENT,
  crawl_id      TEXT NOT NULL REFERENCES crawls(id),
  source_page   INTEGER NOT NULL REFERENCES pages(id),
  target_url    TEXT NOT NULL,
  anchor_text   TEXT,
  link_type     TEXT NOT NULL,
  is_internal   BOOLEAN NOT NULL,
  nofollow      BOOLEAN NOT NULL DEFAULT 0
);

CREATE INDEX IF NOT EXISTS idx_links_source ON links(source_page);
CREATE INDEX IF NOT EXISTS idx_links_crawl ON links(crawl_id);
CREATE INDEX IF NOT EXISTS idx_links_target ON links(crawl_id, target_url);

CREATE TABLE IF NOT EXISTS issues (
  id            INTEGER PRIMARY KEY AUTOINCREMENT,
  crawl_id      TEXT NOT NULL REFERENCES crawls(id),
  page_id       INTEGER NOT NULL REFERENCES pages(id),
  rule_id       TEXT NOT NULL,
  severity      TEXT NOT NULL,
  category      TEXT NOT NULL,
  message       TEXT NOT NULL,
  detail_json   TEXT
);

CREATE INDEX IF NOT EXISTS idx_issues_crawl ON issues(crawl_id);
CREATE INDEX IF NOT EXISTS idx_issues_page ON issues(page_id);
CREATE INDEX IF NOT EXISTS idx_issues_severity ON issues(crawl_id, severity);
CREATE INDEX IF NOT EXISTS idx_issues_rule ON issues(crawl_id, rule_id);

-- Application-level settings table.
CREATE TABLE IF NOT EXISTS settings (
  key   TEXT PRIMARY KEY,
  value TEXT NOT NULL
);

-- Per-crawl-profile rule configuration overrides.
CREATE TABLE IF NOT EXISTS rule_config (
  id        INTEGER PRIMARY KEY AUTOINCREMENT,
  profile   TEXT NOT NULL DEFAULT 'default',
  rule_id   TEXT NOT NULL,
  enabled   BOOLEAN,
  severity  TEXT,
  params    TEXT,
  UNIQUE(profile, rule_id)
);
