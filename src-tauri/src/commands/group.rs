use crate::errors::{AppError, AppResult};
use crate::state::AppState;
use serde::{Deserialize, Serialize};
use tauri::State;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Group {
    pub id: String,
    pub name: String,
    pub color: Option<String>,
    pub parent_id: Option<String>,
    pub sort_order: i64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupInput {
    pub name: String,
    pub color: Option<String>,
    pub parent_id: Option<String>,
    pub sort_order: Option<i64>,
}

fn validate(input: &GroupInput) -> AppResult<()> {
    if input.name.trim().is_empty() {
        return Err(AppError::InvalidInput("group name is required".into()));
    }
    if let Some(color) = &input.color {
        if !color.is_empty() && !is_valid_hex_color(color) {
            return Err(AppError::InvalidInput(format!(
                "invalid color: {color} (expected #RGB or #RRGGBB)"
            )));
        }
    }
    Ok(())
}

fn is_valid_hex_color(s: &str) -> bool {
    let bytes = s.as_bytes();
    if !s.starts_with('#') || (bytes.len() != 4 && bytes.len() != 7) {
        return false;
    }
    s[1..].chars().all(|c| c.is_ascii_hexdigit())
}

#[tauri::command]
pub async fn get_groups(state: State<'_, AppState>) -> AppResult<Vec<Group>> {
    let conn = state.db.lock();
    let mut stmt = conn.prepare(
        "SELECT id, name, color, parent_id, sort_order
         FROM groups
         ORDER BY sort_order ASC, name ASC",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(Group {
            id: row.get(0)?,
            name: row.get(1)?,
            color: row.get(2)?,
            parent_id: row.get(3)?,
            sort_order: row.get(4)?,
        })
    })?;
    let groups: Result<Vec<_>, _> = rows.collect();
    Ok(groups?)
}

#[tauri::command]
pub async fn create_group(
    state: State<'_, AppState>,
    input: GroupInput,
) -> AppResult<Group> {
    validate(&input)?;
    let id = Uuid::new_v4().to_string();
    let sort_order = input.sort_order.unwrap_or(0);

    let conn = state.db.lock();
    conn.execute(
        "INSERT INTO groups (id, name, color, parent_id, sort_order)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![id, input.name, input.color, input.parent_id, sort_order],
    )?;

    Ok(Group {
        id,
        name: input.name,
        color: input.color,
        parent_id: input.parent_id,
        sort_order,
    })
}

#[tauri::command]
pub async fn update_group(
    state: State<'_, AppState>,
    id: String,
    input: GroupInput,
) -> AppResult<Group> {
    validate(&input)?;
    let sort_order = input.sort_order.unwrap_or(0);

    // Prevent self-parent or cycle (simple check: parent != self).
    if input.parent_id.as_deref() == Some(id.as_str()) {
        return Err(AppError::InvalidInput("group cannot be its own parent".into()));
    }

    let conn = state.db.lock();
    let updated = conn.execute(
        "UPDATE groups SET name = ?1, color = ?2, parent_id = ?3, sort_order = ?4
         WHERE id = ?5",
        rusqlite::params![
            input.name,
            input.color,
            input.parent_id,
            sort_order,
            id,
        ],
    )?;

    if updated == 0 {
        return Err(AppError::NotFound(format!("group {id}")));
    }

    Ok(Group {
        id,
        name: input.name,
        color: input.color,
        parent_id: input.parent_id,
        sort_order,
    })
}

/// Delete a group. Hosts in the group become ungrouped (group_id = NULL).
/// Sub-groups become detached (parent_id = NULL).
#[tauri::command]
pub async fn delete_group(
    state: State<'_, AppState>,
    id: String,
) -> AppResult<()> {
    let conn = state.db.lock();

    // Detach hosts and sub-groups before deleting
    conn.execute(
        "UPDATE hosts SET group_id = NULL WHERE group_id = ?1",
        [&id],
    )?;
    conn.execute(
        "UPDATE groups SET parent_id = NULL WHERE parent_id = ?1",
        [&id],
    )?;

    let deleted = conn.execute("DELETE FROM groups WHERE id = ?1", [&id])?;
    if deleted == 0 {
        return Err(AppError::NotFound(format!("group {id}")));
    }
    Ok(())
}

/// Move a host to a different group (or to ungrouped if `group_id` is None).
#[tauri::command]
pub async fn move_host_to_group(
    state: State<'_, AppState>,
    host_id: String,
    group_id: Option<String>,
) -> AppResult<()> {
    let conn = state.db.lock();
    let updated = conn.execute(
        "UPDATE hosts SET group_id = ?1, updated_at = ?2 WHERE id = ?3",
        rusqlite::params![
            group_id,
            chrono::Utc::now().to_rfc3339(),
            host_id,
        ],
    )?;
    if updated == 0 {
        return Err(AppError::NotFound(format!("host {host_id}")));
    }
    Ok(())
}
