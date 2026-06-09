pub mod migrations;
pub mod schema;

use crate::errors::AppResult;
use rusqlite::Connection;
use std::path::Path;

/// Open or create the SQLite database at the given path and run pending migrations.
pub fn open(path: &Path) -> AppResult<Connection> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let mut conn = Connection::open(path)?;

    // Pragmas before any application schema:
    // - foreign_keys: enforce FK constraints (off by default in SQLite)
    // - journal_mode WAL: better concurrency for desktop app
    // - synchronous NORMAL: safe for WAL, faster than FULL
    conn.execute_batch(
        "PRAGMA foreign_keys = ON;
         PRAGMA journal_mode = WAL;
         PRAGMA synchronous = NORMAL;
         PRAGMA temp_store = MEMORY;",
    )?;

    migrations::run_migrations(&mut conn)?;
    Ok(conn)
}
