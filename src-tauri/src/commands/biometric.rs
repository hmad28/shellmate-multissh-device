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
    let provider_available = state.biometric.is_available();

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
        // Biometric unlock is fail-closed until the wrapping secret is backed
        // by an OS-protected key instead of app-readable database state.
        available: false,
        enabled: enabled && provider_available,
        platform,
    })
}

#[tauri::command]
pub async fn biometric_enable(_state: State<'_, AppState>) -> AppResult<()> {
    Err(crate::errors::AppError::InvalidInput(
        "Biometric unlock is disabled until OS-protected key wrapping is implemented".into(),
    ))
}

#[tauri::command]
pub async fn biometric_disable(state: State<'_, AppState>) -> AppResult<()> {
    let conn = state.db.lock();
    biometric_db::set_disabled(&conn)?;
    log::info!("Biometric unlock disabled");
    Ok(())
}

#[tauri::command]
pub async fn biometric_unlock(_state: State<'_, AppState>) -> AppResult<()> {
    Err(crate::errors::AppError::InvalidInput(
        "Biometric unlock is disabled until OS-protected key wrapping is implemented".into(),
    ))
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

    pub fn set_disabled(conn: &Connection) -> AppResult<()> {
        conn.execute(
            "UPDATE biometric_state SET enabled = 0, wrapped_master_key = NULL, device_secret_nonce = NULL, os_handle = NULL WHERE id = 'default'",
            [],
        )?;
        Ok(())
    }
}
