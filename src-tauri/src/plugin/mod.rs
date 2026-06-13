pub mod manifest;
pub mod permissions;
pub mod runtime;

use crate::errors::{AppError, AppResult};
use manifest::PluginManifest;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};

/// A registered plugin.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plugin {
    pub id: String,
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: Option<String>,
    pub enabled: bool,
    pub installed_at: String,
    pub updated_at: String,
}

/// Plugin capability with grant status.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginCapability {
    pub plugin_id: String,
    pub capability: String,
    pub granted: bool,
    pub config: Option<String>,
}

/// Plugin manager handles installation, removal, and execution.
pub struct PluginManager;

impl PluginManager {
    /// Install a plugin from a manifest and WASM bytes.
    pub fn install(
        conn: &Connection,
        manifest: &PluginManifest,
        wasm_path: &str,
    ) -> AppResult<Plugin> {
        let now = chrono::Utc::now().to_rfc3339();
        let manifest_json =
            serde_json::to_string(manifest).map_err(|e| AppError::Internal(e.to_string()))?;

        conn.execute(
            "INSERT INTO plugins (id, name, version, author, description, wasm_path, manifest_json, enabled, installed_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 1, ?8, ?8)
             ON CONFLICT(id) DO UPDATE SET
                name = excluded.name,
                version = excluded.version,
                author = excluded.author,
                description = excluded.description,
                wasm_path = excluded.wasm_path,
                manifest_json = excluded.manifest_json,
                updated_at = excluded.updated_at",
            rusqlite::params![
                manifest.id,
                manifest.name,
                manifest.version,
                manifest.author,
                manifest.description,
                wasm_path,
                manifest_json,
                now,
            ],
        )?;

        // Register declared capabilities.
        for cap in &manifest.capabilities {
            conn.execute(
                "INSERT INTO plugin_capabilities (plugin_id, capability, granted, config)
                 VALUES (?1, ?2, 0, ?3)
                 ON CONFLICT(plugin_id, capability) DO UPDATE SET config = excluded.config",
                rusqlite::params![manifest.id, cap.name, cap.config],
            )?;
        }

        Ok(Plugin {
            id: manifest.id.clone(),
            name: manifest.name.clone(),
            version: manifest.version.clone(),
            author: manifest.author.clone(),
            description: manifest.description.clone(),
            enabled: true,
            installed_at: now.clone(),
            updated_at: now,
        })
    }

    /// List all installed plugins.
    pub fn list(conn: &Connection) -> AppResult<Vec<Plugin>> {
        let mut stmt = conn.prepare(
            "SELECT id, name, version, author, description, enabled, installed_at, updated_at
             FROM plugins ORDER BY name",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(Plugin {
                id: row.get(0)?,
                name: row.get(1)?,
                version: row.get(2)?,
                author: row.get(3)?,
                description: row.get(4)?,
                enabled: row.get::<_, i64>(5)? != 0,
                installed_at: row.get(6)?,
                updated_at: row.get(7)?,
            })
        })?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    /// Uninstall a plugin.
    pub fn uninstall(conn: &Connection, plugin_id: &str) -> AppResult<()> {
        // Get wasm_path to optionally clean up.
        let wasm_path: Option<String> = conn
            .query_row(
                "SELECT wasm_path FROM plugins WHERE id = ?1",
                [plugin_id],
                |row| row.get(0),
            )
            .ok();

        conn.execute("DELETE FROM plugins WHERE id = ?1", [plugin_id])?;

        // Clean up WASM file if it exists.
        if let Some(path) = wasm_path {
            let _ = std::fs::remove_file(&path);
        }

        Ok(())
    }

    /// Enable or disable a plugin.
    pub fn set_enabled(conn: &Connection, plugin_id: &str, enabled: bool) -> AppResult<()> {
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "UPDATE plugins SET enabled = ?1, updated_at = ?2 WHERE id = ?3",
            rusqlite::params![enabled as i64, now, plugin_id],
        )?;
        Ok(())
    }

    /// Grant or revoke a capability for a plugin.
    pub fn set_capability(
        conn: &Connection,
        plugin_id: &str,
        capability: &str,
        granted: bool,
    ) -> AppResult<()> {
        conn.execute(
            "UPDATE plugin_capabilities SET granted = ?1
             WHERE plugin_id = ?2 AND capability = ?3",
            rusqlite::params![granted as i64, plugin_id, capability],
        )?;
        Ok(())
    }

    /// List capabilities for a plugin.
    pub fn list_capabilities(
        conn: &Connection,
        plugin_id: &str,
    ) -> AppResult<Vec<PluginCapability>> {
        let mut stmt = conn.prepare(
            "SELECT plugin_id, capability, granted, config
             FROM plugin_capabilities WHERE plugin_id = ?1 ORDER BY capability",
        )?;
        let rows = stmt.query_map([plugin_id], |row| {
            Ok(PluginCapability {
                plugin_id: row.get(0)?,
                capability: row.get(1)?,
                granted: row.get::<_, i64>(2)? != 0,
                config: row.get(3)?,
            })
        })?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    /// Get plugin info by ID.
    pub fn get(conn: &Connection, plugin_id: &str) -> AppResult<Plugin> {
        conn.query_row(
            "SELECT id, name, version, author, description, enabled, installed_at, updated_at
             FROM plugins WHERE id = ?1",
            [plugin_id],
            |row| {
                Ok(Plugin {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    version: row.get(2)?,
                    author: row.get(3)?,
                    description: row.get(4)?,
                    enabled: row.get::<_, i64>(5)? != 0,
                    installed_at: row.get(6)?,
                    updated_at: row.get(7)?,
                })
            },
        )
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => {
                AppError::NotFound(format!("plugin {plugin_id} not found"))
            }
            other => other.into(),
        })
    }
}
