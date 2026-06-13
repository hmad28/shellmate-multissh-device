pub mod migrations;
pub mod schema;

use crate::errors::{AppError, AppResult};
use rusqlite::Connection;
use std::path::Path;

/// SQLCipher key length (256-bit).
const SQLCIPHER_KEY_LEN: usize = 32;

/// Open or create the SQLite database at the given path and run pending migrations.
/// If `db_key` is Some, the database is opened/created with SQLCipher encryption.
pub fn open(path: &Path, db_key: Option<&[u8; SQLCIPHER_KEY_LEN]>) -> AppResult<Connection> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let mut conn = Connection::open(path)?;

    // If a key is provided, set it before any other operations.
    if let Some(key) = db_key {
        let key_hex = hex::encode(key);
        conn.execute_batch(&format!("PRAGMA key = 'x\"{key_hex}\"';"))?;
    }

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

/// Check if the database at the given path is a plaintext SQLite database
/// (i.e., not encrypted with SQLCipher). Returns true if it can be read
/// without a key and contains the expected schema.
pub fn is_plaintext_db(path: &Path) -> bool {
    if !path.exists() {
        return false;
    }
    // Try to open without a key and read the settings table.
    match Connection::open(path) {
        Ok(conn) => {
            conn.execute_batch("PRAGMA foreign_keys = ON;").ok();
            conn.query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='settings'",
                [],
                |row| row.get::<_, i64>(0),
            )
            .map(|count| count > 0)
            .unwrap_or(false)
        }
        Err(_) => false,
    }
}

/// Migrate a plaintext SQLite database to SQLCipher encryption.
/// 1. Backs up the original DB to `path.bak`
/// 2. Opens the plaintext DB
/// 3. Creates a new encrypted DB
/// 4. Copies all data using `sqlcipher_export`
/// 5. Replaces the original file
pub fn migrate_to_encrypted(
    path: &Path,
    db_key: &[u8; SQLCIPHER_KEY_LEN],
) -> AppResult<()> {
    let backup_path = path.with_extension("db.bak");

    // Step 1: Backup the original
    std::fs::copy(path, &backup_path)?;

    // Step 2: Open the plaintext DB
    let plaintext_conn = Connection::open(path)?;

    // Step 3: Create a temp encrypted DB
    let temp_path = path.with_extension("db.encrypted");
    let key_hex = hex::encode(db_key);
    let encrypted_conn = Connection::open(&temp_path)?;
    encrypted_conn.execute_batch(&format!("PRAGMA key = 'x\"{key_hex}\"';"))?;

    // Step 4: Use sqlcipher_export to copy all data
    encrypted_conn.execute_batch("SELECT sqlcipher_export('main', 'main');")?;

    // Step 5: Copy encrypted DB over original
    encrypted_conn.close().map_err(|(_, e)| AppError::Internal(format!("close encrypted db: {e:?}")))?;
    plaintext_conn.close().map_err(|(_, e)| AppError::Internal(format!("close plaintext db: {e:?}")))?;

    std::fs::copy(&temp_path, path)?;
    std::fs::remove_file(&temp_path)?;

    log::info!("Database migrated to SQLCipher. Backup at {}", backup_path.display());
    Ok(())
}
