use crate::errors::{AppError, AppResult};
use rusqlite::Connection;

/// Check if a plugin has a specific capability granted.
pub fn has_capability(conn: &Connection, plugin_id: &str, capability: &str) -> AppResult<bool> {
    let granted: bool = conn
        .query_row(
            "SELECT granted FROM plugin_capabilities
             WHERE plugin_id = ?1 AND capability = ?2",
            rusqlite::params![plugin_id, capability],
            |row| row.get::<_, i64>(0),
        )
        .map(|v| v != 0)
        .unwrap_or(false);
    Ok(granted)
}

/// Check if a plugin is enabled.
pub fn is_enabled(conn: &Connection, plugin_id: &str) -> AppResult<bool> {
    let enabled: bool = conn
        .query_row(
            "SELECT enabled FROM plugins WHERE id = ?1",
            [plugin_id],
            |row| row.get::<_, i64>(0),
        )
        .map(|v| v != 0)
        .unwrap_or(false);
    Ok(enabled)
}

/// Gate a plugin action: checks enabled + capability.
pub fn check_permission(conn: &Connection, plugin_id: &str, capability: &str) -> AppResult<()> {
    if !is_enabled(conn, plugin_id)? {
        return Err(AppError::InvalidInput(format!(
            "plugin {plugin_id} is disabled"
        )));
    }
    if !has_capability(conn, plugin_id, capability)? {
        return Err(AppError::InvalidInput(format!(
            "plugin {plugin_id} lacks '{capability}' capability"
        )));
    }
    Ok(())
}
