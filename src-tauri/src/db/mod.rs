pub mod seed;

use rusqlite::Connection;
use std::path::PathBuf;
use std::sync::Mutex;

/// Application database state managed by Tauri.
pub struct Database {
    pub conn: Mutex<Connection>,
}

impl Database {
    /// Open (or create) the SQLite database at the given path and run migrations.
    pub fn new(db_path: &PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        // Ensure parent directory exists
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let conn = Connection::open(db_path)?;

        // Enable WAL mode for better concurrent read performance
        conn.execute_batch("PRAGMA journal_mode=WAL;")?;
        // Enable foreign key enforcement
        conn.execute_batch("PRAGMA foreign_keys=ON;")?;

        // Run migrations
        Self::run_migrations(&conn)?;

        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    /// Run all pending SQL migrations in order.
    fn run_migrations(conn: &Connection) -> Result<(), Box<dyn std::error::Error>> {
        // Create migrations tracking table
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS _migrations (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE,
                applied_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
            );"
        )?;

        // Check if initial migration has been applied
        let already_applied: bool = conn
            .prepare("SELECT COUNT(*) FROM _migrations WHERE name = ?1")?
            .query_row(["001_initial"], |row| row.get::<_, i64>(0))
            .map(|count| count > 0)?;

        if !already_applied {
            let migration_sql = include_str!("migrations/001_initial.sql");
            conn.execute_batch(migration_sql)?;
            conn.execute(
                "INSERT INTO _migrations (name) VALUES (?1)",
                ["001_initial"],
            )?;
            log::info!("Applied migration: 001_initial");
        }

        // Migration 002: Add body_size_bytes column and new settings keys
        let already_applied_002: bool = conn
            .prepare("SELECT COUNT(*) FROM _migrations WHERE name = ?1")?
            .query_row(["002_add_body_size_bytes"], |row| row.get::<_, i64>(0))
            .map(|count| count > 0)?;

        if !already_applied_002 {
            let migration_sql = include_str!("migrations/002_add_body_size_bytes.sql");
            conn.execute_batch(migration_sql)?;
            conn.execute(
                "INSERT INTO _migrations (name) VALUES (?1)",
                ["002_add_body_size_bytes"],
            )?;
            log::info!("Applied migration: 002_add_body_size_bytes");
        }

        Ok(())
    }
}

/// Get the default database path for the application.
pub fn get_db_path(app_handle: &tauri::AppHandle) -> PathBuf {
    use tauri::Manager;
    let app_dir = app_handle
        .path()
        .app_data_dir()
        .expect("Failed to get app data directory");
    app_dir.join("cyber-news-reader.db")
}
