use crate::errors::{AppError, AppResult};
use crate::state::AppState;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Theme {
    pub id: String,
    pub name: String,
    pub base: String, // 'dark' | 'light'
    /// JSON ThemeDefinition (see frontend src/types/theme.ts)
    pub definition: String,
    pub is_builtin: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThemeInput {
    pub id: String,
    pub name: String,
    pub base: String,
    pub definition: String,
}

fn validate(input: &ThemeInput) -> AppResult<()> {
    if input.id.trim().is_empty() {
        return Err(AppError::InvalidInput("theme id is required".into()));
    }
    if input.name.trim().is_empty() {
        return Err(AppError::InvalidInput("theme name is required".into()));
    }
    if !matches!(input.base.as_str(), "dark" | "light") {
        return Err(AppError::InvalidInput(format!(
            "invalid base: {} (expected 'dark' or 'light')",
            input.base
        )));
    }
    // Validate definition is parseable JSON
    serde_json::from_str::<serde_json::Value>(&input.definition)
        .map_err(|e| AppError::InvalidInput(format!("definition is not valid JSON: {e}")))?;
    Ok(())
}

#[tauri::command]
pub async fn get_themes(state: State<'_, AppState>) -> AppResult<Vec<Theme>> {
    let conn = state.db.lock();
    let mut stmt = conn.prepare(
        "SELECT id, name, base, definition, is_builtin, created_at, updated_at
         FROM themes ORDER BY is_builtin DESC, name ASC",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(Theme {
            id: row.get(0)?,
            name: row.get(1)?,
            base: row.get(2)?,
            definition: row.get(3)?,
            is_builtin: row.get::<_, i64>(4)? != 0,
            created_at: row.get(5)?,
            updated_at: row.get(6)?,
        })
    })?;
    let result: Result<Vec<_>, _> = rows.collect();
    Ok(result?)
}

#[tauri::command]
pub async fn save_theme(state: State<'_, AppState>, input: ThemeInput) -> AppResult<Theme> {
    validate(&input)?;
    let now = Utc::now().to_rfc3339();

    let conn = state.db.lock();

    // Reject overwriting builtin themes
    let existing_builtin: Option<i64> = conn
        .query_row(
            "SELECT is_builtin FROM themes WHERE id = ?1",
            [&input.id],
            |row| row.get(0),
        )
        .ok();
    if existing_builtin == Some(1) {
        return Err(AppError::InvalidInput(format!(
            "cannot modify builtin theme: {}",
            input.id
        )));
    }

    conn.execute(
        "INSERT INTO themes (id, name, base, definition, is_builtin, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, 0, ?5, ?5)
         ON CONFLICT(id) DO UPDATE SET
           name = excluded.name,
           base = excluded.base,
           definition = excluded.definition,
           updated_at = excluded.updated_at",
        rusqlite::params![input.id, input.name, input.base, input.definition, now],
    )?;

    let created_at: String = conn.query_row(
        "SELECT created_at FROM themes WHERE id = ?1",
        [&input.id],
        |row| row.get(0),
    )?;

    Ok(Theme {
        id: input.id,
        name: input.name,
        base: input.base,
        definition: input.definition,
        is_builtin: false,
        created_at,
        updated_at: now,
    })
}

#[tauri::command]
pub async fn delete_theme(state: State<'_, AppState>, id: String) -> AppResult<()> {
    let conn = state.db.lock();

    let is_builtin: Option<i64> = conn
        .query_row(
            "SELECT is_builtin FROM themes WHERE id = ?1",
            [&id],
            |row| row.get(0),
        )
        .ok();
    if is_builtin.is_none() {
        return Err(AppError::NotFound(format!("theme {id}")));
    }
    if is_builtin == Some(1) {
        return Err(AppError::InvalidInput("cannot delete builtin theme".into()));
    }

    conn.execute("DELETE FROM themes WHERE id = ?1", [&id])?;
    Ok(())
}
