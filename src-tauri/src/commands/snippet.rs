use crate::errors::{AppError, AppResult};
use crate::state::AppState;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use tauri::State;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Snippet {
    pub id: String,
    pub title: String,
    pub command: String,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SnippetInput {
    pub title: String,
    pub command: String,
    pub description: Option<String>,
    pub tags: Vec<String>,
}

fn validate(input: &SnippetInput) -> AppResult<()> {
    if input.title.trim().is_empty() {
        return Err(AppError::InvalidInput("snippet title is required".into()));
    }
    if input.command.trim().is_empty() {
        return Err(AppError::InvalidInput("snippet command is required".into()));
    }
    Ok(())
}

fn map_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Snippet> {
    let tags_json: Option<String> = row.get(4)?;
    let tags = tags_json
        .and_then(|s| serde_json::from_str::<Vec<String>>(&s).ok())
        .unwrap_or_default();
    Ok(Snippet {
        id: row.get(0)?,
        title: row.get(1)?,
        command: row.get(2)?,
        description: row.get(3)?,
        tags,
        created_at: row.get(5)?,
        updated_at: row.get(6)?,
    })
}

#[tauri::command]
pub async fn get_snippets(state: State<'_, AppState>) -> AppResult<Vec<Snippet>> {
    let conn = state.db.lock();
    let mut stmt = conn.prepare(
        "SELECT id, title, command, description, tags, created_at, updated_at
         FROM snippets ORDER BY title ASC",
    )?;
    let rows = stmt.query_map([], map_row)?;
    let result: Result<Vec<_>, _> = rows.collect();
    Ok(result?)
}

#[tauri::command]
pub async fn create_snippet(
    state: State<'_, AppState>,
    input: SnippetInput,
) -> AppResult<Snippet> {
    validate(&input)?;
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    let tags_json = serde_json::to_string(&input.tags)?;

    let conn = state.db.lock();
    conn.execute(
        "INSERT INTO snippets (id, title, command, description, tags, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        rusqlite::params![
            id,
            input.title,
            input.command,
            input.description,
            tags_json,
            now,
            now,
        ],
    )?;

    Ok(Snippet {
        id,
        title: input.title,
        command: input.command,
        description: input.description,
        tags: input.tags,
        created_at: now.clone(),
        updated_at: now,
    })
}

#[tauri::command]
pub async fn update_snippet(
    state: State<'_, AppState>,
    id: String,
    input: SnippetInput,
) -> AppResult<Snippet> {
    validate(&input)?;
    let now = Utc::now().to_rfc3339();
    let tags_json = serde_json::to_string(&input.tags)?;

    let conn = state.db.lock();
    let updated = conn.execute(
        "UPDATE snippets SET title = ?1, command = ?2, description = ?3,
                              tags = ?4, updated_at = ?5
         WHERE id = ?6",
        rusqlite::params![
            input.title,
            input.command,
            input.description,
            tags_json,
            now,
            id,
        ],
    )?;
    if updated == 0 {
        return Err(AppError::NotFound(format!("snippet {id}")));
    }

    let created_at: String = conn.query_row(
        "SELECT created_at FROM snippets WHERE id = ?1",
        [&id],
        |row| row.get(0),
    )?;

    Ok(Snippet {
        id,
        title: input.title,
        command: input.command,
        description: input.description,
        tags: input.tags,
        created_at,
        updated_at: now,
    })
}

#[tauri::command]
pub async fn delete_snippet(state: State<'_, AppState>, id: String) -> AppResult<()> {
    let conn = state.db.lock();
    let deleted = conn.execute("DELETE FROM snippets WHERE id = ?1", [&id])?;
    if deleted == 0 {
        return Err(AppError::NotFound(format!("snippet {id}")));
    }
    Ok(())
}
