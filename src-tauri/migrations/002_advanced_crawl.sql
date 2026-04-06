-- migrations/002_advanced_crawl.sql
-- Phase 6: Advanced crawl features — sitemap tracking, external link checking,
-- JS rendering flag, and custom CSS extraction storage.

CREATE TABLE IF NOT EXISTS sitemap_urls (
  id          INTEGER PRIMARY KEY AUTOINCREMENT,
  crawl_id    TEXT NOT NULL REFERENCES crawls(id),
  url         TEXT NOT NULL,
  lastmod     TEXT,
  changefreq  TEXT,
  priority    REAL,
  source      TEXT NOT NULL,  -- 'robots_txt', 'sitemap_index', 'sitemap_xml'
  UNIQUE(crawl_id, url)
);
CREATE INDEX IF NOT EXISTS idx_sitemap_urls_crawl ON sitemap_urls(crawl_id);

CREATE TABLE IF NOT EXISTS external_links (
  id              INTEGER PRIMARY KEY AUTOINCREMENT,
  crawl_id        TEXT NOT NULL REFERENCES crawls(id),
  source_page     INTEGER NOT NULL REFERENCES pages(id),
  target_url      TEXT NOT NULL,
  status_code     INTEGER,
  response_time_ms INTEGER,
  error_message   TEXT,
  checked_at      TEXT
);
CREATE INDEX IF NOT EXISTS idx_external_links_crawl ON external_links(crawl_id);

ALTER TABLE pages ADD COLUMN is_js_rendered BOOLEAN NOT NULL DEFAULT 0;
ALTER TABLE pages ADD COLUMN custom_extractions TEXT;
