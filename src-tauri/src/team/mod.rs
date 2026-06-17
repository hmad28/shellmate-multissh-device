use crate::errors::{AppError, AppResult};
use crate::vault::Vault;
use rand::RngCore;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use zeroize::Zeroize;

const TEAM_KEY_LEN: usize = 32;

/// A team with its metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Team {
    pub id: String,
    pub name: String,
    pub created_at: String,
}

/// A team member.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamMember {
    pub id: String,
    pub team_id: String,
    pub member_pubkey: String,
    pub member_label: String,
    pub added_at: String,
    pub revoked_at: Option<String>,
}

/// A host shared with a team.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamShare {
    pub id: String,
    pub team_id: String,
    pub host_id: String,
    pub permission: String,
    pub shared_at: String,
}

/// Input for creating a team.
#[derive(Debug, Deserialize)]
pub struct CreateTeamInput {
    pub name: String,
}

/// Input for adding a member.
#[derive(Debug, Deserialize)]
pub struct AddMemberInput {
    pub team_id: String,
    pub member_pubkey: String,
    pub member_label: String,
}

/// Input for sharing a host.
#[derive(Debug, Deserialize)]
pub struct ShareHostInput {
    pub team_id: String,
    pub host_id: String,
    pub permission: String,
}

/// Team manager handles team CRUD, member management, and host sharing.
pub struct TeamManager;

impl TeamManager {
    /// Create a new team. Generates a random team master key and wraps it
    /// with the vault key.
    pub fn create_team(
        conn: &Connection,
        vault: &Vault,
        input: &CreateTeamInput,
    ) -> AppResult<Team> {
        if !vault.is_unlocked() {
            return Err(AppError::InvalidInput("vault is locked".into()));
        }

        let team_id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();

        // Generate random team master key.
        let mut team_key = [0u8; TEAM_KEY_LEN];
        rand::thread_rng().fill_bytes(&mut team_key);
        // Wrap team key with vault key.
        let wrapped = vault.encrypt(&team_key)?;
        team_key.zeroize();

        conn.execute(
            "INSERT INTO team (id, name, team_master_key_wrapped, team_master_key_nonce, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![
                team_id,
                input.name,
                wrapped.ciphertext,
                wrapped.nonce.to_vec(),
                now,
            ],
        )?;

        Ok(Team {
            id: team_id,
            name: input.name.clone(),
            created_at: now,
        })
    }

    /// List all teams.
    pub fn list_teams(conn: &Connection) -> AppResult<Vec<Team>> {
        let mut stmt = conn.prepare("SELECT id, name, created_at FROM team ORDER BY name")?;
        let rows = stmt.query_map([], |row| {
            Ok(Team {
                id: row.get(0)?,
                name: row.get(1)?,
                created_at: row.get(2)?,
            })
        })?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    /// Delete a team and all its members/shares.
    pub fn delete_team(conn: &Connection, team_id: &str) -> AppResult<()> {
        conn.execute("DELETE FROM team WHERE id = ?1", [team_id])?;
        Ok(())
    }

    /// Add a member to a team.
    ///
    /// Disabled until the team key can be wrapped with the member's public key
    /// and revoked through key rotation.
    pub fn add_member(
        conn: &Connection,
        vault: &Vault,
        input: &AddMemberInput,
    ) -> AppResult<TeamMember> {
        let _ = (conn, vault, input);
        Err(AppError::InvalidInput(
            "team member sharing is disabled until public-key wrapping and key rotation are implemented".into(),
        ))
    }

    /// List members of a team.
    pub fn list_members(conn: &Connection, team_id: &str) -> AppResult<Vec<TeamMember>> {
        let mut stmt = conn.prepare(
            "SELECT id, team_id, member_pubkey, member_label, added_at, revoked_at
             FROM team_members WHERE team_id = ?1 ORDER BY added_at",
        )?;
        let rows = stmt.query_map([team_id], |row| {
            Ok(TeamMember {
                id: row.get(0)?,
                team_id: row.get(1)?,
                member_pubkey: row.get(2)?,
                member_label: row.get(3)?,
                added_at: row.get(4)?,
                revoked_at: row.get(5)?,
            })
        })?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    /// Revoke a member. Sets `revoked_at` timestamp.
    pub fn revoke_member(conn: &Connection, member_id: &str) -> AppResult<()> {
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "UPDATE team_members SET revoked_at = ?1 WHERE id = ?2",
            rusqlite::params![now, member_id],
        )?;
        Ok(())
    }

    /// Share a host with a team.
    ///
    /// Disabled until encrypted team sharing semantics are complete.
    pub fn share_host(
        conn: &Connection,
        vault: &Vault,
        input: &ShareHostInput,
    ) -> AppResult<TeamShare> {
        let _ = (conn, vault, input);
        Err(AppError::InvalidInput(
            "host sharing is disabled until team key wrapping and revocation are implemented"
                .into(),
        ))
    }

    /// List hosts shared with a team.
    pub fn list_shares(conn: &Connection, team_id: &str) -> AppResult<Vec<TeamShare>> {
        let mut stmt = conn.prepare(
            "SELECT id, team_id, host_id, permission, shared_at
             FROM team_shares WHERE team_id = ?1 ORDER BY shared_at",
        )?;
        let rows = stmt.query_map([team_id], |row| {
            Ok(TeamShare {
                id: row.get(0)?,
                team_id: row.get(1)?,
                host_id: row.get(2)?,
                permission: row.get(3)?,
                shared_at: row.get(4)?,
            })
        })?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    /// Remove a host share.
    pub fn remove_share(conn: &Connection, share_id: &str) -> AppResult<()> {
        conn.execute("DELETE FROM team_shares WHERE id = ?1", [share_id])?;
        Ok(())
    }
}
