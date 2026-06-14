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
    // If this fails, the DB might be corrupted from a previous buggy migration.
    // Try to recover from backup.
    let new_conn = match db::open(db_path, Some(db_key)) {
        Ok(conn) => conn,
        Err(e) => {
            log::warn!("Failed to open encrypted DB: {e}. Attempting recovery from backup.");
            let backup_path = db_path.with_extension("db.bak");
            if backup_path.exists() {
                // Restore backup and re-migrate.
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

#[tauri::command]
pub async fn vault_status(state: State<'_, AppState>) -> AppResult<VaultStatus> {
    let initialized = {
        let conn = state.db.lock();
        Vault::is_initialized(&conn)?
    };
    Ok(VaultStatus {
        initialized,
        unlocked: state.vault.is_unlocked(),
        db_encrypted: state.vault.get_db_key().is_some(),
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

    // Encrypt the database with the newly derived key.
    encrypt_and_swap_db(&state, &db_key)?;

    Ok(())
}

#[tauri::command]
pub async fn vault_unlock(
    state: State<'_, AppState>,
    password: String,
) -> AppResult<()> {
    // First, try to unlock against the current (possibly plaintext) connection.
    let db_key = {
        let conn = state.db.lock();
        state.vault.unlock(&conn, &password)?
    };

    // Now that we have the db_key, ensure the DB is encrypted and reopen.
    encrypt_and_swap_db(&state, &db_key)?;

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

    // Step 1: Derive the new db_key preview from the new password so we can
    // rekey SQLCipher after the vault commits.
    // We need the salt that will be used. The vault layer generates a fresh salt
    // internally. We can't preview it, but we CAN derive it after vault commits
    // because `get_db_key()` returns the in-memory key.
    //
    // Strategy: vault.change_master_password commits the credential re-encryption
    // and stores the new db_key in memory. The SQLCipher connection still has the
    // old key at that point (PRAGMA key was set when the connection was opened).
    // We call PRAGMA rekey immediately after vault commits, while old key is still
    // active on the connection.

    state
        .vault
        .change_master_password(&mut conn, &current_password, &new_password)?;

    // Step 2: Rotate SQLCipher key. Connection still has old PRAGMA key.
    if let Some(new_db_key) = state.vault.get_db_key() {
        let key_hex = hex::encode(new_db_key);
        conn.execute_batch(&format!("PRAGMA rekey = 'x\"{key_hex}\"'"))?;
        log::info!("SQLCipher DB key rotated successfully");
    }

    Ok(())
}
