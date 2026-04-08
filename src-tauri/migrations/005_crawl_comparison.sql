-- Index to support cross-crawl URL joins for the comparison feature.
CREATE INDEX IF NOT EXISTS idx_pages_crawl_url ON pages(crawl_id, url);
