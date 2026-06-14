use crate::db;
use crate::errors::AppResult;
use crate::state::AppState;
use crate::vault::Vault;
use serde::Serialize;
use tauri::State;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VaultStatus {
    pub initialized: bool,
    pub unlocked: bool,
    pub db_encrypted: bool,
}

/// After obtaining the db_key (from setup or unlock), migrate the database to
/// SQLCipher if it is still plaintext, then reopen the encrypted connection and
/// swap it into AppState.
fn encrypt_and_swap_db(
    state: &AppState,
    db_key: &[u8; 32],
) -> AppResult<()> {
    let db_path = &state.db_path;

    // Check if the DB is still plaintext (pre-Phase-7 builds).
    if db::is_plaintext_db(db_path) {
        log::info!("Migrating plaintext database to SQLCipher encryption");
        db::migrate_to_encrypted(db_path, db_key)?;
    }

    // Open a new encrypted connection and swap it in.
    // If this fails, try to recover from backup.
    let new_conn = match db::open(db_path, Some(db_key)) {
        Ok(conn) => conn,
        Err(e) => {
            log::warn!("Failed to open encrypted DB: {e}. Attempting recovery.");
            let backup_path = db_path.with_extension("db.bak");
            if backup_path.exists() {
                std::fs::copy(&backup_path, db_path)?;
                db::migrate_to_encrypted(db_path, db_key)?;
                db::open(db_path, Some(db_key))?
            } else {
                return Err(e);
            }
        }
    };
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

    let salt = hex::decode(&salt_hex).map_err(|e| crate::errors::AppError::Internal(format!("salt hex: {e}")))?;
    let nonce = hex::decode(&nonce_hex).map_err(|e| crate::errors::AppError::Internal(format!("nonce hex: {e}")))?;
    let ct = hex::decode(&ct_hex).map_err(|e| crate::errors::AppError::Internal(format!("ct hex: {e}")))?;

    db::write_vault_meta(&state.db_path, &salt, &nonce, &ct)
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
pub async fn vault_setup(
    state: State<'_, AppState>,
    password: String,
) -> AppResult<()> {
    let db_key = {
        let conn = state.db.lock();
        state.vault.setup(&conn, &password)?
    };

    // Write vault metadata file BEFORE encrypting DB.
    write_meta_from_db(&state)?;

    // Verify the vault file was written.
    if !db::has_vault_meta(&state.db_path) {
        return Err(crate::errors::AppError::Internal(
            "vault metadata file was not written".into(),
        ));
    }
    log::info!("Vault metadata file verified at {:?}", state.db_path.with_extension("vault"));

    // Encrypt the database with the newly derived key.
    encrypt_and_swap_db(&state, &db_key)?;

    Ok(())
}

#[tauri::command]
pub async fn vault_unlock(
    state: State<'_, AppState>,
    password: String,
) -> AppResult<()> {
    let db_path = &state.db_path;

    if db::has_vault_meta(db_path) {
        // DB is encrypted. Verify password from metadata file, then open DB.
        let master_key = Vault::verify_from_meta(db_path, &password)?;
        let (vault_key, db_key) = crate::crypto::derive_vault_and_db_keys(&master_key);

        // Set vault keys in memory.
        state.vault.unlock_with_key(&master_key)?;

        // Open encrypted DB and swap.
        let new_conn = db::open(db_path, Some(&db_key))?;
        state.swap_db(new_conn);
        log::info!("Vault unlocked via metadata file, DB reopened encrypted");
    } else {
        // Plaintext DB — use the old flow.
        let db_key = {
            let conn = state.db.lock();
            state.vault.unlock(&conn, &password)?
        };
        encrypt_and_swap_db(&state, &db_key)?;
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
    if let Some(new_db_key) = state.vault.get_db_key() {
        let key_hex = hex::encode(new_db_key);
        conn.execute_batch(&format!("PRAGMA rekey = 'x\"{key_hex}\"'"))?;
        log::info!("SQLCipher DB key rotated successfully");
    }

    // Update vault metadata file with new verifier.
    drop(conn);
    write_meta_from_db(&state)?;

    Ok(())
}
