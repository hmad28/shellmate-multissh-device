use crate::errors::AppResult;
use crate::state::AppState;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use tauri::State;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandHistoryEntry {
    pub id: String,
    pub session_id: String,
    pub command: String,
    pub exit_code: Option<i32>,
    pub working_dir: Option<String>,
    pub executed_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddHistoryInput {
    pub session_id: String,
    pub command: String,
    pub exit_code: Option<i32>,
    pub working_dir: Option<String>,
}

#[tauri::command]
pub async fn history_add(
    state: State<'_, AppState>,
    input: AddHistoryInput,
) -> AppResult<String> {
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    let conn = state.db.lock();
    conn.execute(
        "INSERT INTO command_history (id, session_id, command, exit_code, working_dir, executed_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        rusqlite::params![
            id,
            input.session_id,
            input.command,
            input.exit_code,
            input.working_dir,
            now,
        ],
    )?;
    Ok(id)
}

#[tauri::command]
pub async fn history_list(
    state: State<'_, AppState>,
    session_id: Option<String>,
    limit: Option<u32>,
) -> AppResult<Vec<CommandHistoryEntry>> {
    let conn = state.db.lock();
    let limit = limit.unwrap_or(100).min(1000);

    let (sql, params): (&str, Vec<Box<dyn rusqlite::types::ToSql>>) = match &session_id {
        Some(sid) => (
            "SELECT id, session_id, command, exit_code, working_dir, executed_at
             FROM command_history WHERE session_id = ?1
             ORDER BY executed_at DESC LIMIT ?2",
            vec![
                Box::new(sid.clone()) as Box<dyn rusqlite::types::ToSql>,
                Box::new(limit),
            ],
        ),
        None => (
            "SELECT id, session_id, command, exit_code, working_dir, executed_at
             FROM command_history ORDER BY executed_at DESC LIMIT ?1",
            vec![Box::new(limit) as Box<dyn rusqlite::types::ToSql>],
        ),
    };

    let mut stmt = conn.prepare(sql)?;
    let rows = stmt.query_map(rusqlite::params_from_iter(params.iter()), |row| {
        Ok(CommandHistoryEntry {
            id: row.get(0)?,
            session_id: row.get(1)?,
            command: row.get(2)?,
            exit_code: row.get(3)?,
            working_dir: row.get(4)?,
            executed_at: row.get(5)?,
        })
    })?;
    let entries: Result<Vec<_>, _> = rows.collect();
    Ok(entries?)
}

#[tauri::command]
pub async fn history_search(
    state: State<'_, AppState>,
    query: String,
    limit: Option<u32>,
) -> AppResult<Vec<CommandHistoryEntry>> {
    let q = query.trim();
    if q.is_empty() {
        return history_list(state, None, limit).await;
    }
    let escaped = q.replace('\\', "\\\\").replace('%', "\\%").replace('_', "\\_");
    let pattern = format!("%{}%", escaped.to_lowercase());
    let limit = limit.unwrap_or(50).min(500);

    let conn = state.db.lock();
    let mut stmt = conn.prepare(
        "SELECT id, session_id, command, exit_code, working_dir, executed_at
         FROM command_history WHERE LOWER(command) LIKE ?1 ESCAPE '\\'
         ORDER BY executed_at DESC LIMIT ?2",
    )?;
    let rows = stmt.query_map(rusqlite::params![pattern, limit], |row| {
        Ok(CommandHistoryEntry {
            id: row.get(0)?,
            session_id: row.get(1)?,
            command: row.get(2)?,
            exit_code: row.get(3)?,
            working_dir: row.get(4)?,
            executed_at: row.get(5)?,
        })
    })?;
    let entries: Result<Vec<_>, _> = rows.collect();
    Ok(entries?)
}

#[tauri::command]
pub async fn history_clear(
    state: State<'_, AppState>,
    session_id: Option<String>,
) -> AppResult<()> {
    let conn = state.db.lock();
    match session_id {
        Some(sid) => {
            conn.execute("DELETE FROM command_history WHERE session_id = ?1", [&sid])?;
        }
        None => {
            conn.execute("DELETE FROM command_history", [])?;
        }
    }
    Ok(())
}
