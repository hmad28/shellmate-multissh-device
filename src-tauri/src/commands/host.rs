use crate::errors::{AppError, AppResult};
use crate::state::AppState;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use tauri::State;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Host {
    pub id: String,
    pub label: String,
    pub hostname: String,
    pub port: u16,
    pub username: String,
    pub auth_type: String,
    pub credential_id: String,
    pub group_id: Option<String>,
    pub tags: Vec<String>,
    pub notes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HostInput {
    pub label: String,
    pub hostname: String,
    pub port: u16,
    pub username: String,
    pub auth_type: String,
    pub credential_id: String,
    pub group_id: Option<String>,
    pub tags: Vec<String>,
    pub notes: Option<String>,
}

fn validate(input: &HostInput) -> AppResult<()> {
    if input.label.trim().is_empty() {
        return Err(AppError::InvalidInput("label is required".into()));
    }
    if input.hostname.trim().is_empty() {
        return Err(AppError::InvalidInput("hostname is required".into()));
    }
    if input.username.trim().is_empty() {
        return Err(AppError::InvalidInput("username is required".into()));
    }
    if input.port == 0 {
        return Err(AppError::InvalidInput("port must be 1-65535".into()));
    }
    if !matches!(
        input.auth_type.as_str(),
        "password" | "key" | "key_passphrase"
    ) {
        return Err(AppError::InvalidInput(format!(
            "invalid auth_type: {}",
            input.auth_type
        )));
    }
    Ok(())
}

#[tauri::command]
pub async fn get_hosts(state: State<'_, AppState>) -> AppResult<Vec<Host>> {
    let conn = state.db.lock();
    let mut stmt = conn.prepare(
        "SELECT id, label, hostname, port, username, auth_type, credential_id,
                group_id, tags, notes, created_at, updated_at
         FROM hosts ORDER BY label ASC",
    )?;
    let rows = stmt.query_map([], |row| {
        let tags_json: Option<String> = row.get(8)?;
        let tags = tags_json
            .and_then(|s| serde_json::from_str::<Vec<String>>(&s).ok())
            .unwrap_or_default();
        Ok(Host {
            id: row.get(0)?,
            label: row.get(1)?,
            hostname: row.get(2)?,
            port: row.get::<_, i64>(3)? as u16,
            username: row.get(4)?,
            auth_type: row.get(5)?,
            credential_id: row.get(6)?,
            group_id: row.get(7)?,
            tags,
            notes: row.get(9)?,
            created_at: row.get(10)?,
            updated_at: row.get(11)?,
        })
    })?;
    let hosts: Result<Vec<_>, _> = rows.collect();
    Ok(hosts?)
}

#[tauri::command]
pub async fn create_host(state: State<'_, AppState>, input: HostInput) -> AppResult<Host> {
    validate(&input)?;
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    let tags_json = serde_json::to_string(&input.tags)?;

    let conn = state.db.lock();
    conn.execute(
        "INSERT INTO hosts (id, label, hostname, port, username, auth_type,
                            credential_id, group_id, tags, notes, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
        rusqlite::params![
            id,
            input.label,
            input.hostname,
            input.port as i64,
            input.username,
            input.auth_type,
            input.credential_id,
            input.group_id,
            tags_json,
            input.notes,
            now,
            now,
        ],
    )?;

    Ok(Host {
        id,
        label: input.label,
        hostname: input.hostname,
        port: input.port,
        username: input.username,
        auth_type: input.auth_type,
        credential_id: input.credential_id,
        group_id: input.group_id,
        tags: input.tags,
        notes: input.notes,
        created_at: now.clone(),
        updated_at: now,
    })
}

#[tauri::command]
pub async fn update_host(
    state: State<'_, AppState>,
    id: String,
    input: HostInput,
) -> AppResult<Host> {
    validate(&input)?;
    let now = Utc::now().to_rfc3339();
    let tags_json = serde_json::to_string(&input.tags)?;

    let conn = state.db.lock();
    let updated = conn.execute(
        "UPDATE hosts SET label = ?1, hostname = ?2, port = ?3, username = ?4,
                          auth_type = ?5, credential_id = ?6, group_id = ?7,
                          tags = ?8, notes = ?9, updated_at = ?10
         WHERE id = ?11",
        rusqlite::params![
            input.label,
            input.hostname,
            input.port as i64,
            input.username,
            input.auth_type,
            input.credential_id,
            input.group_id,
            tags_json,
            input.notes,
            now,
            id,
        ],
    )?;

    if updated == 0 {
        return Err(AppError::NotFound(format!("host {id}")));
    }

    let created_at: String =
        conn.query_row("SELECT created_at FROM hosts WHERE id = ?1", [&id], |row| {
            row.get(0)
        })?;

    Ok(Host {
        id,
        label: input.label,
        hostname: input.hostname,
        port: input.port,
        username: input.username,
        auth_type: input.auth_type,
        credential_id: input.credential_id,
        group_id: input.group_id,
        tags: input.tags,
        notes: input.notes,
        created_at,
        updated_at: now,
    })
}

#[tauri::command]
pub async fn delete_host(state: State<'_, AppState>, id: String) -> AppResult<()> {
    let conn = state.db.lock();
    let deleted = conn.execute("DELETE FROM hosts WHERE id = ?1", [&id])?;
    if deleted == 0 {
        return Err(AppError::NotFound(format!("host {id}")));
    }
    Ok(())
}

/// Free-text search across host label, hostname, username, group name, and tags.
/// Case-insensitive substring match.
#[tauri::command]
pub async fn search_hosts(state: State<'_, AppState>, query: String) -> AppResult<Vec<Host>> {
    let q = query.trim();
    if q.is_empty() {
        return get_hosts(state).await;
    }
    // Escape SQL LIKE wildcards to prevent unintended pattern matching
    let escaped = q
        .replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_");
    let pattern = format!("%{}%", escaped.to_lowercase());

    let conn = state.db.lock();
    let mut stmt = conn.prepare(
        "SELECT h.id, h.label, h.hostname, h.port, h.username, h.auth_type,
                h.credential_id, h.group_id, h.tags, h.notes,
                h.created_at, h.updated_at
         FROM hosts h
         LEFT JOIN groups g ON g.id = h.group_id
         WHERE LOWER(h.label) LIKE ?1 ESCAPE '\\'
            OR LOWER(h.hostname) LIKE ?1 ESCAPE '\\'
            OR LOWER(h.username) LIKE ?1 ESCAPE '\\'
            OR LOWER(COALESCE(g.name, '')) LIKE ?1 ESCAPE '\\'
            OR LOWER(COALESCE(h.tags, '')) LIKE ?1 ESCAPE '\\'
            OR LOWER(COALESCE(h.notes, '')) LIKE ?1 ESCAPE '\\'
         ORDER BY h.label ASC",
    )?;
    let rows = stmt.query_map([&pattern], |row| {
        let tags_json: Option<String> = row.get(8)?;
        let tags = tags_json
            .and_then(|s| serde_json::from_str::<Vec<String>>(&s).ok())
            .unwrap_or_default();
        Ok(Host {
            id: row.get(0)?,
            label: row.get(1)?,
            hostname: row.get(2)?,
            port: row.get::<_, i64>(3)? as u16,
            username: row.get(4)?,
            auth_type: row.get(5)?,
            credential_id: row.get(6)?,
            group_id: row.get(7)?,
            tags,
            notes: row.get(9)?,
            created_at: row.get(10)?,
            updated_at: row.get(11)?,
        })
    })?;
    let hosts: Result<Vec<_>, _> = rows.collect();
    Ok(hosts?)
}
