//! SQLite database connection management and migration runner.
//!
//! Each crawl is stored in a separate `.seocrawl` SQLite file. The app also
//! maintains a global metadata database for settings and crawl history.

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use anyhow::{Context, Result};
use rusqlite::Connection;

/// Embedded migration SQL files, ordered by version.
const MIGRATIONS: &[(&str, &str)] = &[
    ("001_initial", include_str!("../../migrations/001_initial.sql")),
];

/// Application database handle.
///
/// Wraps a SQLite connection in a Mutex for thread-safe access.
/// The storage writer thread holds the primary write lock; read queries
/// from Tauri commands acquire the lock briefly.
pub struct Database {
    /// Path to the database file.
    pub path: PathBuf,
    /// The SQLite connection, protected by a mutex.
    conn: Mutex<Connection>,
}

impl Database {
    /// Initialize the application database.
    ///
    /// Creates the database file in the Tauri app data directory if it
    /// doesn't exist, enables WAL mode, and runs pending migrations.
    pub fn init(app_handle: &tauri::AppHandle) -> Result<Self> {
        let app_data_dir = app_handle
            .path()
            .app_data_dir()
            .context("Failed to resolve app data directory")?;

        std::fs::create_dir_all(&app_data_dir)
            .context("Failed to create app data directory")?;

        let db_path = app_data_dir.join("oxide-seo.db");
        let conn = Connection::open(&db_path)
            .context("Failed to open SQLite database")?;

        // Enable WAL mode for concurrent read/write performance.
        conn.execute_batch("PRAGMA journal_mode=WAL;")?;
        conn.execute_batch("PRAGMA foreign_keys=ON;")?;
        conn.execute_batch("PRAGMA busy_timeout=5000;")?;

        let db = Self {
            path: db_path,
            conn: Mutex::new(conn),
        };

        db.run_migrations()?;

        Ok(db)
    }

    /// Open an existing `.seocrawl` database file (read-only).
    pub fn open_crawl_file(path: &std::path::Path) -> Result<Self> {
        let conn = Connection::open_with_flags(
            path,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY
                | rusqlite::OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )?;

        Ok(Self {
            path: path.to_path_buf(),
            conn: Mutex::new(conn),
        })
    }

    /// Create a new crawl database file.
    ///
    /// Each crawl gets its own `.seocrawl` SQLite file for portability.
    pub fn create_crawl_db(dir: &std::path::Path, crawl_id: &str) -> Result<Self> {
        std::fs::create_dir_all(dir)?;
        let db_path = dir.join(format!("{}.seocrawl", crawl_id));
        let conn = Connection::open(&db_path)?;

        conn.execute_batch("PRAGMA journal_mode=WAL;")?;
        conn.execute_batch("PRAGMA foreign_keys=ON;")?;
        conn.execute_batch("PRAGMA busy_timeout=5000;")?;

        let db = Self {
            path: db_path,
            conn: Mutex::new(conn),
        };

        db.run_migrations()?;
        Ok(db)
    }

    /// Run all pending migrations.
    fn run_migrations(&self) -> Result<()> {
        let conn = self.conn.lock().map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))?;

        // Create migrations tracking table.
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS _migrations (
                name TEXT PRIMARY KEY,
                applied_at TEXT NOT NULL DEFAULT (datetime('now'))
            );"
        )?;

        for (name, sql) in MIGRATIONS {
            let applied: bool = conn
                .query_row(
                    "SELECT COUNT(*) > 0 FROM _migrations WHERE name = ?1",
                    [name],
                    |row| row.get(0),
                )
                .unwrap_or(false);

            if !applied {
                tracing::info!(migration = %name, "Running migration");
                conn.execute_batch(sql)
                    .with_context(|| format!("Failed to run migration: {}", name))?;
                conn.execute(
                    "INSERT INTO _migrations (name) VALUES (?1)",
                    [name],
                )?;
            }
        }

        Ok(())
    }

    /// Execute a closure with the database connection.
    ///
    /// This is the primary API for all database operations. It acquires
    /// the mutex, passes the connection to the closure, and releases.
    pub fn with_conn<F, T>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&Connection) -> Result<T>,
    {
        let conn = self.conn.lock().map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))?;
        f(&conn)
    }

    /// Execute a closure with a mutable connection reference (for transactions).
    pub fn with_conn_mut<F, T>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&mut Connection) -> Result<T>,
    {
        let mut conn = self.conn.lock().map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))?;
        f(&mut conn)
    }
}

// Required for Tauri managed state.
unsafe impl Send for Database {}
unsafe impl Sync for Database {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_in_memory_and_migrate() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA journal_mode=WAL;").unwrap();
        conn.execute_batch("PRAGMA foreign_keys=ON;").unwrap();

        let db = Database {
            path: PathBuf::from(":memory:"),
            conn: Mutex::new(conn),
        };

        db.run_migrations().unwrap();

        // Verify tables exist.
        db.with_conn(|conn| {
            let tables: Vec<String> = conn
                .prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")?
                .query_map([], |row| row.get(0))?
                .filter_map(|r| r.ok())
                .collect();

            assert!(tables.contains(&"crawls".to_string()));
            assert!(tables.contains(&"pages".to_string()));
            assert!(tables.contains(&"links".to_string()));
            assert!(tables.contains(&"issues".to_string()));
            Ok(())
        })
        .unwrap();
    }
}
