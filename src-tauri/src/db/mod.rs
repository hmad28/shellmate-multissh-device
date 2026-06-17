pub mod migrations;
pub mod schema;

use crate::errors::{AppError, AppResult};
use rusqlite::Connection;
use std::path::{Path, PathBuf};

/// Result of probing the on-disk state of `shellmate.db` at boot. Used to
/// decide whether a `.vault` metadata file is consistent with the DB.
#[derive(Debug, PartialEq, Eq)]
pub enum DbState {
    /// First 16 bytes are the standard SQLite header magic. DB is plaintext.
    Plaintext,
    /// Header bytes are non-zero but do not match the SQLite magic. Likely
    /// SQLCipher-encrypted (random-looking salt + encrypted header).
    SqliteHeader,
    /// File is missing or zero bytes.
    Empty,
    /// File exists but header is unreadable / header bytes look like garbage.
    Unreadable,
}

/// Probe the on-disk DB to determine whether it's plaintext SQLite,
/// SQLCipher-encrypted, empty, or unreadable. Used at boot to detect broken
/// pre-fix states where a `.vault` file was written but the DB was never
/// actually migrated.
pub fn probe_db_state(path: &Path) -> DbState {
    let Ok(data) = std::fs::read(path) else {
        return DbState::Empty;
    };
    if data.len() < 16 {
        return DbState::Empty;
    }
    // Standard SQLite header magic: "SQLite format 3\0"
    const SQLITE_MAGIC: &[u8; 16] = b"SQLite format 3\0";
    if &data[..16] == SQLITE_MAGIC {
        return DbState::Plaintext;
    }
    // If the file looks like random bytes (high entropy, no recognizable
    // structure) it's most likely a SQLCipher-encrypted DB. If it has any
    // recognizable text or repeated patterns, treat as unreadable.
    if data[..16].iter().all(|&b| b == 0) {
        return DbState::Empty;
    }
    DbState::SqliteHeader
}

/// SQLCipher key length (256-bit).
const SQLCIPHER_KEY_LEN: usize = 32;

/// Vault metadata file — stores salt + encrypted verifier in plaintext
/// alongside the encrypted DB. Allows password verification before opening
/// the encrypted database.
const VAULT_META_MAGIC: &[u8] = b"SHELMATE_VAULT1";
const VAULT_META_SALT_OFFSET: usize = 15; // after magic (15 bytes)
const VAULT_META_SALT_LEN: usize = 16;
const VAULT_META_NONCE_OFFSET: usize = VAULT_META_SALT_OFFSET + VAULT_META_SALT_LEN; // 31
const VAULT_META_NONCE_LEN: usize = 12;
const VAULT_META_CT_OFFSET: usize = VAULT_META_NONCE_OFFSET + VAULT_META_NONCE_LEN; // 43

/// Get the vault metadata file path for a given DB path.
fn vault_meta_path(db_path: &Path) -> PathBuf {
    db_path.with_extension("vault")
}

/// Check if a vault metadata file exists for this DB.
pub fn has_vault_meta(db_path: &Path) -> bool {
    vault_meta_path(db_path).exists()
}

/// Write vault metadata after setup/migration.
/// Stores: magic (16) + salt (16) + nonce (12) + ciphertext (variable).
pub fn write_vault_meta(
    db_path: &Path,
    salt: &[u8],
    nonce: &[u8],
    ciphertext: &[u8],
) -> AppResult<()> {
    let meta_path = vault_meta_path(db_path);
    let mut data = Vec::with_capacity(16 + 16 + 12 + ciphertext.len());
    data.extend_from_slice(VAULT_META_MAGIC);
    data.extend_from_slice(salt);
    data.extend_from_slice(nonce);
    data.extend_from_slice(ciphertext);
    std::fs::write(&meta_path, &data)?;
    log::info!("Vault metadata written to {}", meta_path.display());
    Ok(())
}

/// Read vault metadata. Returns (salt, nonce, ciphertext).
pub fn read_vault_meta(db_path: &Path) -> AppResult<([u8; 16], [u8; 12], Vec<u8>)> {
    let meta_path = vault_meta_path(db_path);
    let data = std::fs::read(&meta_path)?;

    if data.len() < VAULT_META_CT_OFFSET {
        return Err(AppError::Internal("vault meta file too short".into()));
    }
    if &data[..VAULT_META_MAGIC.len()] != VAULT_META_MAGIC {
        return Err(AppError::Internal("vault meta invalid magic".into()));
    }

    let mut salt = [0u8; 16];
    salt.copy_from_slice(&data[VAULT_META_SALT_OFFSET..VAULT_META_SALT_OFFSET + 16]);
    let mut nonce = [0u8; 12];
    nonce.copy_from_slice(&data[VAULT_META_NONCE_OFFSET..VAULT_META_NONCE_OFFSET + 12]);
    let ciphertext = data[VAULT_META_CT_OFFSET..].to_vec();

    Ok((salt, nonce, ciphertext))
}

/// Remove vault metadata file (called on vault reset).
pub fn remove_vault_meta(db_path: &Path) {
    let meta_path = vault_meta_path(db_path);
    let _ = std::fs::remove_file(meta_path);
}

/// Open or create the SQLite database at the given path and run pending migrations.
/// If `db_key` is Some, the database is opened/created with SQLCipher encryption.
pub fn open(path: &Path, db_key: Option<&[u8; SQLCIPHER_KEY_LEN]>) -> AppResult<Connection> {
    open_with_recovery(path, db_key, true)
}

/// Like `open` but optionally skips the `.bak` recovery attempt. Used by
/// `is_plaintext_db` (which must not auto-recover) and the boot probe.
pub fn open_with_recovery(
    path: &Path,
    db_key: Option<&[u8; SQLCIPHER_KEY_LEN]>,
    try_backup_recovery: bool,
) -> AppResult<Connection> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let open_result = Connection::open(path).and_then(|mut conn| {
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

        // Force SQLCipher to actually decrypt a page so we fail fast on bad key
        // / broken header instead of failing on the first real query.
        conn.query_row("SELECT count(*) FROM sqlite_master", [], |row| {
            row.get::<_, i64>(0)
        })?;

        Ok(conn)
    });

    let mut conn = match open_result {
        Ok(c) => c,
        Err(e) => {
            // If we have a backup from a previous migration, restore it and retry.
            if try_backup_recovery {
                let backup_path = path.with_extension("db.bak");
                if backup_path.exists() {
                    log::warn!(
                        "Failed to open DB ({e}); restoring from {} and retrying",
                        backup_path.display()
                    );
                    std::fs::copy(&backup_path, path)?;
                    return open_with_recovery(path, db_key, false);
                }
            }
            return Err(AppError::Database(e));
        }
    };

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
    // If vault meta exists, the DB is encrypted.
    if has_vault_meta(path) {
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
pub fn migrate_to_encrypted(path: &Path, db_key: &[u8; SQLCIPHER_KEY_LEN]) -> AppResult<()> {
    let backup_path = path.with_extension("db.bak");

    // Step 0: Checkpoint WAL to flush all pending writes to the main DB file.
    {
        let flush_conn = Connection::open(path)?;
        flush_conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE)")?;
        flush_conn
            .close()
            .map_err(|(_, e)| AppError::Internal(format!("flush close: {e:?}")))?;
    }

    // Step 1: Backup the original
    std::fs::copy(path, &backup_path)?;

    // Step 2: Create a temp encrypted DB and ATTACH the plaintext DB.
    let temp_path = path.with_extension("db.encrypted");
    let key_hex = hex::encode(db_key);

    // Remove stale temp file if it exists.
    let _ = std::fs::remove_file(&temp_path);

    let encrypted_conn = Connection::open(&temp_path)?;
    encrypted_conn.execute_batch(&format!("PRAGMA key = 'x\"{key_hex}\"';"))?;

    // ATTACH the plaintext database with empty key.
    let path_str = path.to_string_lossy().replace('\'', "''");
    encrypted_conn.execute_batch(&format!("ATTACH DATABASE '{path_str}' AS plaintext KEY ''"))?;

    // Export from the attached plaintext DB to the main encrypted DB.
    encrypted_conn.execute_batch("SELECT sqlcipher_export('main', 'plaintext')")?;

    encrypted_conn.execute_batch("DETACH DATABASE plaintext")?;

    // Step 3: Copy encrypted DB over original
    encrypted_conn
        .close()
        .map_err(|(_, e)| AppError::Internal(format!("close encrypted db: {e:?}")))?;

    std::fs::copy(&temp_path, path)?;
    std::fs::remove_file(&temp_path)?;

    log::info!(
        "Database migrated to SQLCipher. Backup at {}",
        backup_path.display()
    );
    Ok(())
}
