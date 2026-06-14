use crate::errors::{AppError, AppResult};
use crate::vault::Vault;
use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
use aes_gcm::aead::Aead;
use hkdf::Hkdf;
use rand::RngCore;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use sha2::Sha256;
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
        let mut stmt = conn.prepare(
            "SELECT id, name, created_at FROM team ORDER BY name",
        )?;
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

    /// Add a member to a team. Wraps the team master key with a per-member
    /// secret. The wrapping key is derived from a random secret stored alongside
    /// the member record, NOT from the public key string (which is not secret).
    pub fn add_member(
        conn: &Connection,
        vault: &Vault,
        input: &AddMemberInput,
    ) -> AppResult<TeamMember> {
        if !vault.is_unlocked() {
            return Err(AppError::InvalidInput("vault is locked".into()));
        }

        // Get the team master key (encrypted).
        let (wrapped_key, key_nonce): (Vec<u8>, Vec<u8>) = conn.query_row(
            "SELECT team_master_key_wrapped, team_master_key_nonce FROM team WHERE id = ?1",
            [&input.team_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )?;

        // Decrypt the team master key with vault key.
        let nonce_arr = nonce_from_vec(&key_nonce)?;
        let blob = crate::crypto::EncryptedBlob {
            ciphertext: wrapped_key,
            nonce: nonce_arr,
        };
        let team_key = vault.decrypt(&blob)?;

        // Generate a random per-member wrapping secret.
        // This secret is stored encrypted with the vault key alongside the member record.
        // The wrapping key is derived from this secret via HKDF, NOT from the public key.
        let mut member_secret = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut member_secret);

        let wrap_key = derive_member_wrap_key(&member_secret);
        let cipher = Aes256Gcm::new_from_slice(&wrap_key)
            .map_err(|e| AppError::Internal(format!("AES init: {e}")))?;

        let mut member_nonce = [0u8; 12];
        rand::thread_rng().fill_bytes(&mut member_nonce);
        let nonce = Nonce::from_slice(&member_nonce);

        let wrapped_team_key = cipher
            .encrypt(nonce, team_key.as_ref())
            .map_err(|e| AppError::Internal(format!("encrypt: {e}")))?;

        // Combine nonce + ciphertext for storage.
        let mut wrapped_with_nonce = Vec::with_capacity(12 + wrapped_team_key.len());
        wrapped_with_nonce.extend_from_slice(&member_nonce);
        wrapped_with_nonce.extend_from_slice(&wrapped_team_key);

        // Encrypt the member secret with vault key for storage.
        let encrypted_secret = vault.encrypt(&member_secret)?;

        let member_id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();

        conn.execute(
            "INSERT INTO team_members (id, team_id, member_pubkey, member_label, wrapped_team_key, added_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![
                member_id,
                input.team_id,
                input.member_pubkey,
                input.member_label,
                wrapped_with_nonce,
                now,
            ],
        )?;

        // Store the encrypted member secret in a separate table.
        conn.execute(
            "INSERT INTO team_member_secrets (member_id, encrypted_secret, secret_nonce)
             VALUES (?1, ?2, ?3)",
            rusqlite::params![
                member_id,
                encrypted_secret.ciphertext,
                encrypted_secret.nonce.to_vec(),
            ],
        )?;

        Ok(TeamMember {
            id: member_id,
            team_id: input.team_id.clone(),
            member_pubkey: input.member_pubkey.clone(),
            member_label: input.member_label.clone(),
            added_at: now,
            revoked_at: None,
        })
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
    pub fn share_host(
        conn: &Connection,
        vault: &Vault,
        input: &ShareHostInput,
    ) -> AppResult<TeamShare> {
        if !vault.is_unlocked() {
            return Err(AppError::InvalidInput("vault is locked".into()));
        }

        if input.permission != "read" && input.permission != "edit" {
            return Err(AppError::InvalidInput(
                "permission must be 'read' or 'edit'".into(),
            ));
        }

        // Verify host exists.
        let host_exists: bool = conn.query_row(
            "SELECT COUNT(*) FROM hosts WHERE id = ?1",
            [&input.host_id],
            |row| row.get::<_, i64>(0),
        )? > 0;
        if !host_exists {
            return Err(AppError::NotFound("host not found".into()));
        }

        let share_id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();

        conn.execute(
            "INSERT INTO team_shares (id, team_id, host_id, permission, shared_at)
             VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(team_id, host_id) DO UPDATE SET
                permission = excluded.permission,
                shared_at = excluded.shared_at",
            rusqlite::params![share_id, input.team_id, input.host_id, input.permission, now],
        )?;

        Ok(TeamShare {
            id: share_id,
            team_id: input.team_id.clone(),
            host_id: input.host_id.clone(),
            permission: input.permission.clone(),
            shared_at: now,
        })
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

/// Derive a wrapping key from a random per-member secret using HKDF.
/// The secret is stored encrypted with the vault key, NOT derived from public key.
fn derive_member_wrap_key(member_secret: &[u8]) -> [u8; 32] {
    let hk = Hkdf::<Sha256>::new(Some(b"shellmate-team-member-v1"), member_secret);
    let mut key = [0u8; 32];
    hk.expand(b"team-key-wrap", &mut key).expect("HKDF expand failed");
    key
}

fn nonce_from_vec(v: &[u8]) -> AppResult<[u8; 12]> {
    if v.len() != 12 {
        return Err(AppError::Internal(format!(
            "expected 12-byte nonce, got {}",
            v.len()
        )));
    }
    let mut arr = [0u8; 12];
    arr.copy_from_slice(v);
    Ok(arr)
}
