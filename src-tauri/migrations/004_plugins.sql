-- Phase 8: Plugin management tables.

CREATE TABLE IF NOT EXISTS plugins (
    name         TEXT PRIMARY KEY,
    version      TEXT NOT NULL,
    kind         TEXT NOT NULL,
    enabled      BOOLEAN NOT NULL DEFAULT 0,
    config_json  TEXT,
    installed_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at   TEXT NOT NULL DEFAULT (datetime('now'))
);
