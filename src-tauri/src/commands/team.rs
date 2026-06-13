use crate::errors::AppResult;
use crate::state::AppState;
use crate::team::{CreateTeamInput, AddMemberInput, ShareHostInput, Team, TeamMember, TeamShare, TeamManager};
use tauri::State;

#[tauri::command]
pub async fn team_create(
    state: State<'_, AppState>,
    input: CreateTeamInput,
) -> AppResult<Team> {
    let conn = state.db.lock();
    TeamManager::create_team(&conn, &state.vault, &input)
}

#[tauri::command]
pub async fn team_list(state: State<'_, AppState>) -> AppResult<Vec<Team>> {
    let conn = state.db.lock();
    TeamManager::list_teams(&conn)
}

#[tauri::command]
pub async fn team_delete(state: State<'_, AppState>, team_id: String) -> AppResult<()> {
    let conn = state.db.lock();
    TeamManager::delete_team(&conn, &team_id)
}

#[tauri::command]
pub async fn team_add_member(
    state: State<'_, AppState>,
    input: AddMemberInput,
) -> AppResult<TeamMember> {
    let conn = state.db.lock();
    TeamManager::add_member(&conn, &state.vault, &input)
}

#[tauri::command]
pub async fn team_list_members(
    state: State<'_, AppState>,
    team_id: String,
) -> AppResult<Vec<TeamMember>> {
    let conn = state.db.lock();
    TeamManager::list_members(&conn, &team_id)
}

#[tauri::command]
pub async fn team_revoke_member(
    state: State<'_, AppState>,
    member_id: String,
) -> AppResult<()> {
    let conn = state.db.lock();
    TeamManager::revoke_member(&conn, &member_id)
}

#[tauri::command]
pub async fn team_share_host(
    state: State<'_, AppState>,
    input: ShareHostInput,
) -> AppResult<TeamShare> {
    let conn = state.db.lock();
    TeamManager::share_host(&conn, &state.vault, &input)
}

#[tauri::command]
pub async fn team_list_shares(
    state: State<'_, AppState>,
    team_id: String,
) -> AppResult<Vec<TeamShare>> {
    let conn = state.db.lock();
    TeamManager::list_shares(&conn, &team_id)
}

#[tauri::command]
pub async fn team_remove_share(
    state: State<'_, AppState>,
    share_id: String,
) -> AppResult<()> {
    let conn = state.db.lock();
    TeamManager::remove_share(&conn, &share_id)
}
