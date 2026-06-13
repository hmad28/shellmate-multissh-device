use crate::biometric;
use crate::errors::AppResult;
use crate::state::AppState;
use serde::Serialize;
use tauri::State;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BiometricStatus {
    pub available: bool,
    pub enabled: bool,
    pub platform: &'static str,
}

#[tauri::command]
pub async fn biometric_status(state: State<'_, AppState>) -> AppResult<BiometricStatus> {
    let available = state.biometric.is_available();

    let enabled = {
        let conn = state.db.lock();
        biometric_db::is_enabled(&conn)?
    };

    #[cfg(target_os = "windows")]
    let platform = "Windows Hello";
    #[cfg(target_os = "macos")]
    let platform = "Touch ID";
    #[cfg(target_os = "ios")]
    let platform = "Face ID / Touch ID";
    #[cfg(target_os = "android")]
    let platform = "Biometric";
    #[cfg(target_os = "linux")]
    let platform = "Not Supported";

    Ok(BiometricStatus {
        available,
        enabled,
        platform,
    })
}

#[tauri::command]
pub async fn biometric_enable(state: State<'_, AppState>) -> AppResult<()> {
    if !state.biometric.is_available() {
        return Err(crate::errors::AppError::InvalidInput(
            "Biometric authentication is not available on this device".into(),
        ));
    }

    // Vault must be unlocked to enroll (we need the master key).
    let master_key = state
        .vault
        .get_db_key()
        .ok_or_else(|| crate::errors::AppError::InvalidInput("vault is locked".into()))?;

    // Verify user via biometric.
    if !state.biometric.verify_user("Enroll biometric unlock for ShellMate") {
        return Err(crate::errors::AppError::InvalidInput(
            "Biometric verification failed".into(),
        ));
    }

    // Generate device secret and wrap the master key.
    let device_secret = biometric::generate_device_secret();
    let (wrapped, nonce) = biometric::wrap_master_key(&master_key, &device_secret)?;

    // Store in database.
    let conn = state.db.lock();
    biometric_db::set_enabled(&conn, true, &wrapped, &nonce, &device_secret)?;

    log::info!("Biometric unlock enrolled");
    Ok(())
}

#[tauri::command]
pub async fn biometric_disable(state: State<'_, AppState>) -> AppResult<()> {
    let conn = state.db.lock();
    biometric_db::set_disabled(&conn)?;
    log::info!("Biometric unlock disabled");
    Ok(())
}

#[tauri::command]
pub async fn biometric_unlock(state: State<'_, AppState>) -> AppResult<()> {
    // Load biometric state from DB.
    let (wrapped, nonce, device_secret) = {
        let conn = state.db.lock();
        biometric_db::load_state(&conn)?.ok_or_else(|| {
            crate::errors::AppError::InvalidInput("biometric not enrolled".into())
        })?
    };

    // Verify user via biometric.
    if !state.biometric.verify_user("Unlock ShellMate vault") {
        return Err(crate::errors::AppError::InvalidInput(
            "Biometric verification failed".into(),
        ));
    }

    // Unwrap the master key.
    let master_key = biometric::unwrap_master_key(&wrapped, &nonce, &device_secret)?;

    // Use the master key to derive vault/db keys and unlock the vault.
    // The vault's unlock method expects a password, but we have the derived key.
    // We need a way to unlock with the key directly.
    // For now, we'll set the vault keys directly.
    state.vault.unlock_with_key(&master_key)?;

    // If DB is encrypted, reopen with the db_key.
    if let Some(db_key) = state.vault.get_db_key() {
        let db_path = &state.db_path;
        if db_path.exists() {
            let new_conn = crate::db::open(db_path, Some(&db_key))?;
            state.swap_db(new_conn);
        }
    }

    log::info!("Vault unlocked via biometric");
    Ok(())
}

/// Database operations for biometric state.
mod biometric_db {
    use crate::errors::AppResult;
    use rusqlite::Connection;

    pub fn is_enabled(conn: &Connection) -> AppResult<bool> {
        let enabled = conn
            .query_row(
                "SELECT enabled FROM biometric_state WHERE id = 'default'",
                [],
                |row| row.get::<_, i64>(0),
            )
            .unwrap_or(0);
        Ok(enabled != 0)
    }

    pub fn set_enabled(
        conn: &Connection,
        enabled: bool,
        wrapped: &[u8],
        nonce: &[u8; 12],
        device_secret: &[u8; 32],
    ) -> AppResult<()> {
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO biometric_state (id, enabled, wrapped_master_key, device_secret_nonce, os_handle, enrolled_at)
             VALUES ('default', ?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(id) DO UPDATE SET
                enabled = excluded.enabled,
                wrapped_master_key = excluded.wrapped_master_key,
                device_secret_nonce = excluded.device_secret_nonce,
                os_handle = excluded.os_handle,
                enrolled_at = excluded.enrolled_at",
            rusqlite::params![
                enabled as i64,
                wrapped,
                nonce.to_vec(),
                hex::encode(device_secret),
                now,
            ],
        )?;
        Ok(())
    }

    pub fn set_disabled(conn: &Connection) -> AppResult<()> {
        conn.execute(
            "UPDATE biometric_state SET enabled = 0, wrapped_master_key = NULL, device_secret_nonce = NULL, os_handle = NULL WHERE id = 'default'",
            [],
        )?;
        Ok(())
    }

    pub fn load_state(
        conn: &Connection,
    ) -> AppResult<Option<(Vec<u8>, [u8; 12], [u8; 32])>> {
        let result = conn.query_row(
            "SELECT enabled, wrapped_master_key, device_secret_nonce, os_handle FROM biometric_state WHERE id = 'default'",
            [],
            |row| {
                let enabled: i64 = row.get(0)?;
                let wrapped: Option<Vec<u8>> = row.get(1)?;
                let nonce: Option<Vec<u8>> = row.get(2)?;
                let os_handle: Option<String> = row.get(3)?;
                Ok((enabled, wrapped, nonce, os_handle))
            },
        );

        match result {
            Ok((1, Some(wrapped), Some(nonce), Some(handle))) => {
                if nonce.len() != 12 {
                    return Err(crate::errors::AppError::Internal(
                        "invalid nonce length".into(),
                    ));
                }
                let mut nonce_arr = [0u8; 12];
                nonce_arr.copy_from_slice(&nonce);

                let secret_bytes = hex::decode(&handle)
                    .map_err(|e| crate::errors::AppError::Internal(format!("invalid os_handle: {e}")))?;
                if secret_bytes.len() != 32 {
                    return Err(crate::errors::AppError::Internal(
                        "invalid device secret length".into(),
                    ));
                }
                let mut secret = [0u8; 32];
                secret.copy_from_slice(&secret_bytes);

                Ok(Some((wrapped, nonce_arr, secret)))
            }
            _ => Ok(None),
        }
    }
}
