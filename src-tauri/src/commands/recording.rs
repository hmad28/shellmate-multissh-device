use crate::errors::AppResult;
use crate::state::AppState;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionRecording {
    pub id: String,
    pub session_id: String,
    pub host_id: Option<String>,
    pub host_label: String,
    pub started_at: String,
    pub ended_at: Option<String>,
    pub duration_secs: Option<i64>,
    pub event_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionEvent {
    pub timestamp_ms: i64,
    pub event_type: String,
    pub data: String,
}

/// Start recording a session.
#[tauri::command]
pub async fn recording_start(
    state: State<'_, AppState>,
    session_id: String,
    host_id: Option<String>,
    host_label: String,
) -> AppResult<String> {
    let recording_id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    let conn = state.db.lock();
    conn.execute(
        "INSERT INTO session_recordings (id, session_id, host_id, host_label, started_at, event_count)
         VALUES (?1, ?2, ?3, ?4, ?5, 0)",
        rusqlite::params![recording_id, session_id, host_id, host_label, now],
    )?;

    Ok(recording_id)
}

/// Stop recording a session.
#[tauri::command]
pub async fn recording_stop(
    state: State<'_, AppState>,
    recording_id: String,
) -> AppResult<()> {
    let now = chrono::Utc::now().to_rfc3339();
    let conn = state.db.lock();

    // Calculate duration and event count.
    let (started_at, event_count): (String, i64) = conn.query_row(
        "SELECT started_at, event_count FROM session_recordings WHERE id = ?1",
        [&recording_id],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )?;

    let started = chrono::DateTime::parse_from_rfc3339(&started_at)
        .map(|dt| dt.with_timezone(&chrono::Utc))
        .unwrap_or_else(|_| chrono::Utc::now());
    let duration = (chrono::Utc::now() - started).num_seconds();

    conn.execute(
        "UPDATE session_recordings SET ended_at = ?1, duration_secs = ?2, event_count = ?3 WHERE id = ?4",
        rusqlite::params![now, duration, event_count, recording_id],
    )?;

    Ok(())
}

/// Record an event (terminal output) for a session.
#[tauri::command]
pub async fn recording_event(
    state: State<'_, AppState>,
    recording_id: String,
    event_type: String,
    data: String,
) -> AppResult<()> {
    let now_ms = chrono::Utc::now().timestamp_millis();
    let conn = state.db.lock();

    conn.execute(
        "INSERT INTO session_events (recording_id, timestamp_ms, event_type, data) VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![recording_id, now_ms, event_type, data],
    )?;

    conn.execute(
        "UPDATE session_recordings SET event_count = event_count + 1 WHERE id = ?1",
        [&recording_id],
    )?;

    Ok(())
}

/// List all session recordings.
#[tauri::command]
pub async fn recording_list(state: State<'_, AppState>) -> AppResult<Vec<SessionRecording>> {
    let conn = state.db.lock();
    let mut stmt = conn.prepare(
        "SELECT id, session_id, host_id, host_label, started_at, ended_at, duration_secs, event_count
         FROM session_recordings ORDER BY started_at DESC LIMIT 100",
    )?;

    let rows = stmt.query_map([], |row| {
        Ok(SessionRecording {
            id: row.get(0)?,
            session_id: row.get(1)?,
            host_id: row.get(2)?,
            host_label: row.get(3)?,
            started_at: row.get(4)?,
            ended_at: row.get(5)?,
            duration_secs: row.get(6)?,
            event_count: row.get(7)?,
        })
    })?;

    Ok(rows.filter_map(|r| r.ok()).collect())
}

/// Get events for a recording (for playback).
#[tauri::command]
pub async fn recording_events(
    state: State<'_, AppState>,
    recording_id: String,
) -> AppResult<Vec<SessionEvent>> {
    let conn = state.db.lock();
    let mut stmt = conn.prepare(
        "SELECT timestamp_ms, event_type, data FROM session_events WHERE recording_id = ?1 ORDER BY timestamp_ms",
    )?;

    let rows = stmt.query_map([&recording_id], |row| {
        Ok(SessionEvent {
            timestamp_ms: row.get(0)?,
            event_type: row.get(1)?,
            data: row.get(2)?,
        })
    })?;

    Ok(rows.filter_map(|r| r.ok()).collect())
}

/// Delete a recording and its events.
#[tauri::command]
pub async fn recording_delete(
    state: State<'_, AppState>,
    recording_id: String,
) -> AppResult<()> {
    let conn = state.db.lock();
    conn.execute("DELETE FROM session_events WHERE recording_id = ?1", [&recording_id])?;
    conn.execute("DELETE FROM session_recordings WHERE id = ?1", [&recording_id])?;
    Ok(())
}
