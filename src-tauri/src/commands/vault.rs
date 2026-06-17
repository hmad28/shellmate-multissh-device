use crate::db;
use crate::errors::{AppError, AppResult};
use crate::state::AppState;
use crate::vault::Vault;
use serde::Serialize;
use tauri::State;
use zeroize::Zeroize;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VaultStatus {
    pub initialized: bool,
    pub unlocked: bool,
    pub db_encrypted: bool,
}

/// After obtaining the db_key (from setup or unlock), migrate the database to
/// SQLCipher if it is still plaintext, then reopen the encrypted connection and
/// swap it into AppState. `db::open` auto-recovers from `.bak` on failure.
fn encrypt_and_swap_db(state: &AppState, db_key: &[u8; 32]) -> AppResult<()> {
    let db_path = &state.db_path;

    if db::is_plaintext_db(db_path) {
        log::info!("Migrating plaintext database to SQLCipher encryption");
        db::migrate_to_encrypted(db_path, db_key)?;
    }

    let new_conn = db::open(db_path, Some(db_key))?;
    state.swap_db(new_conn);
    log::info!("Database reopened with SQLCipher encryption");
    Ok(())
}

/// Write vault metadata file from DB settings (after setup has stored them).
fn write_meta_from_db(state: &AppState) -> AppResult<()> {
    let conn = state.db.lock();
    let salt_hex: String = conn.query_row(
        "SELECT value FROM settings WHERE key = 'vault.salt'",
        [],
        |row| row.get(0),
    )?;
    let ct_hex: String = conn.query_row(
        "SELECT value FROM settings WHERE key = 'vault.verifier.ciphertext'",
        [],
        |row| row.get(0),
    )?;
    let nonce_hex: String = conn.query_row(
        "SELECT value FROM settings WHERE key = 'vault.verifier.nonce'",
        [],
        |row| row.get(0),
    )?;

    let salt = hex::decode(&salt_hex)
        .map_err(|e| crate::errors::AppError::Internal(format!("salt hex: {e}")))?;
    let nonce = hex::decode(&nonce_hex)
        .map_err(|e| crate::errors::AppError::Internal(format!("nonce hex: {e}")))?;
    let ct = hex::decode(&ct_hex)
        .map_err(|e| crate::errors::AppError::Internal(format!("ct hex: {e}")))?;

    db::write_vault_meta(&state.db_path, &salt, &nonce, &ct)
}

fn sqlcipher_enabled() -> bool {
    !cfg!(any(target_os = "android", target_os = "ios"))
}

#[tauri::command]
pub async fn vault_status(state: State<'_, AppState>) -> AppResult<VaultStatus> {
    // Check if vault metadata file exists — if so, DB is encrypted.
    let db_encrypted = db::has_vault_meta(&state.db_path);

    if db_encrypted {
        // Don't try to open the encrypted DB — just report status from meta file.
        return Ok(VaultStatus {
            initialized: true,
            unlocked: state.vault.is_unlocked(),
            db_encrypted: true,
        });
    }

    // Plaintext DB — safe to query.
    let initialized = {
        let conn = state.db.lock();
        Vault::is_initialized(&conn)?
    };
    Ok(VaultStatus {
        initialized,
        unlocked: state.vault.is_unlocked(),
        db_encrypted: false,
    })
}

#[tauri::command]
pub async fn vault_setup(state: State<'_, AppState>, password: String) -> AppResult<()> {
    // Step 1: Setup vault in DB (writes verifier, salt, etc.)
    let db_key = {
        let conn = state.db.lock();
        state.vault.setup(&conn, &password)?
    };

    if !sqlcipher_enabled() {
        let conn = state.db.lock();
        conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE)").ok();
        log::info!("Vault setup complete without SQLCipher on mobile target");
        return Ok(());
    }

    // Step 2: Drop the old connection and flush WAL so the on-disk plaintext
    // file contains all vault settings before migration reads it.
    {
        let mut conn = state.db.lock();
        conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE)").ok();
        // Close the old connection by replacing with a temporary in-memory one.
        let temp_conn = rusqlite::Connection::open_in_memory()
            .map_err(|e| AppError::Internal(format!("temp conn: {e}")))?;
        *conn = temp_conn;
    }

    // Step 3: Migrate DB to encrypted (if still plaintext) and reopen with key.
    // IMPORTANT: write the .vault metadata file ONLY after a successful
    // migration + open, otherwise the next launch will see a stale .vault file
    // for a plaintext DB and fail with "file is not a database" when trying
    // to open it with SQLCipher. db::open auto-recovers from .bak on failure.
    let db_path = &state.db_path;
    if db::is_plaintext_db(db_path) {
        db::migrate_to_encrypted(db_path, &db_key)?;
    }
    let new_conn = db::open(db_path, Some(&db_key))?;
    state.swap_db(new_conn);

    // Step 4: Now that the encrypted DB is open and verified, persist the
    // .vault metadata file so future launches know to prompt for unlock.
    if let Err(e) = write_meta_from_db(&state) {
        // Don't fail setup if meta write fails — the encrypted DB is fine and
        // we can reconstruct meta from the verifier settings on next launch.
        log::warn!("Failed to write vault metadata file: {e}");
    }

    Ok(())
}

#[tauri::command]
pub async fn vault_unlock(state: State<'_, AppState>, password: String) -> AppResult<()> {
    let db_path = &state.db_path;

    if sqlcipher_enabled() && db::has_vault_meta(db_path) {
        // DB is encrypted. Verify password from metadata file, then open DB.
        let master_key = Vault::verify_from_meta(db_path, &password)?;
        let (vault_key, db_key) = crate::crypto::derive_vault_and_db_keys(&master_key);
        let mut master_key = master_key;

        // Set vault keys in memory.
        state.vault.unlock_with_key(&master_key)?;

        // Drop old connection before opening new one.
        {
            let mut conn = state.db.lock();
            let temp_conn = rusqlite::Connection::open_in_memory()
                .map_err(|e| AppError::Internal(format!("temp conn: {e}")))?;
            *conn = temp_conn;
        }

        // Open encrypted DB and swap. If the on-disk file is unreadable (broken
        // header from older buggy builds, or the key was rotated) db::open
        // automatically restores from `.bak`. As a last resort, if the file is
        // actually plaintext despite having a .vault file (e.g. metadata is
        // stale), strip .vault and run the plaintext unlock path.
        let open_result = db::open(db_path, Some(&db_key));
        let new_conn = match open_result {
            Ok(conn) => conn,
            Err(e) => {
                log::warn!("Failed to open encrypted DB on unlock: {e}");
                if db::is_plaintext_db(db_path) {
                    log::info!("Stale vault metadata file detected; removing and switching to plaintext unlock");
                    db::remove_vault_meta(db_path);
                    state.vault.lock();
                    master_key.zeroize();
                    return unlock_plaintext(&state, db_path, &password);
                }
                return Err(e);
            }
        };
        state.swap_db(new_conn);
        master_key.zeroize();
        log::info!("Vault unlocked via metadata file, DB reopened encrypted");
    } else {
        return unlock_plaintext(&state, db_path, &password);
    }

    Ok(())
}

/// Plaintext-DB unlock + migration path. Extracted so it can also be called
/// when a stale .vault file is detected on the encrypted path.
fn unlock_plaintext(state: &AppState, db_path: &std::path::Path, password: &str) -> AppResult<()> {
    let db_key = {
        let conn = state.db.lock();
        state.vault.unlock(&conn, password)?
    };

    if !sqlcipher_enabled() {
        return Ok(());
    }

    // Drop old connection before migration.
    {
        let mut conn = state.db.lock();
        conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE)").ok();
        let temp_conn = rusqlite::Connection::open_in_memory()
            .map_err(|e| AppError::Internal(format!("temp conn: {e}")))?;
        *conn = temp_conn;
    }

    // Migrate and swap. Only write .vault meta AFTER a successful open of the
    // encrypted DB, otherwise next launch will see a stale .vault for a
    // still-plaintext DB and crash with "file is not a database". db::open
    // auto-recovers from `.bak` if the post-migration file is unreadable.
    if db::is_plaintext_db(db_path) {
        db::migrate_to_encrypted(db_path, &db_key)?;
    }
    let new_conn = db::open(db_path, Some(&db_key))?;
    state.swap_db(new_conn);

    if let Err(e) = write_meta_from_db(state) {
        log::warn!("Failed to write vault metadata file: {e}");
    }
    Ok(())
}

#[tauri::command]
pub async fn vault_lock(state: State<'_, AppState>) -> AppResult<()> {
    state.vault.lock();
    Ok(())
}

#[tauri::command]
pub async fn vault_check_idle(state: State<'_, AppState>) -> AppResult<bool> {
    Ok(state.vault.check_idle())
}

#[tauri::command]
pub async fn vault_record_activity(state: State<'_, AppState>) -> AppResult<()> {
    state.vault.record_activity();
    Ok(())
}

#[tauri::command]
pub async fn vault_change_master_password(
    state: State<'_, AppState>,
    current_password: String,
    new_password: String,
) -> AppResult<()> {
    let mut conn = state.db.lock();

    state
        .vault
        .change_master_password(&mut conn, &current_password, &new_password)?;

    // Rotate SQLCipher key if DB is encrypted.
    if sqlcipher_enabled() {
        if let Some(new_db_key) = state.vault.get_db_key() {
            let key_hex = hex::encode(new_db_key);
            conn.execute_batch(&format!("PRAGMA rekey = 'x\"{key_hex}\"'"))?;
            log::info!("SQLCipher DB key rotated successfully");
        }
    }

    // Update vault metadata file with new verifier.
    drop(conn);
    if sqlcipher_enabled() {
        write_meta_from_db(&state)?;
    }

    Ok(())
}
