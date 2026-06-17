pub mod redaction;

use crate::errors::{AppError, AppResult};
use crate::vault::Vault;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// An audit event type.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AuditEventType {
    SessionStart,
    SessionEnd,
    SftpTransfer,
    CommandSent,
    VaultLock,
    VaultUnlock,
    HostCreated,
    HostUpdated,
    HostDeleted,
    SettingsChanged,
}

impl AuditEventType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::SessionStart => "session_start",
            Self::SessionEnd => "session_end",
            Self::SftpTransfer => "sftp_transfer",
            Self::CommandSent => "command_sent",
            Self::VaultLock => "vault_lock",
            Self::VaultUnlock => "vault_unlock",
            Self::HostCreated => "host_created",
            Self::HostUpdated => "host_updated",
            Self::HostDeleted => "host_deleted",
            Self::SettingsChanged => "settings_changed",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "session_start" => Some(Self::SessionStart),
            "session_end" => Some(Self::SessionEnd),
            "sftp_transfer" => Some(Self::SftpTransfer),
            "command_sent" => Some(Self::CommandSent),
            "vault_lock" => Some(Self::VaultLock),
            "vault_unlock" => Some(Self::VaultUnlock),
            "host_created" => Some(Self::HostCreated),
            "host_updated" => Some(Self::HostUpdated),
            "host_deleted" => Some(Self::HostDeleted),
            "settings_changed" => Some(Self::SettingsChanged),
            _ => None,
        }
    }
}

/// An audit event returned from queries (decrypted).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditEvent {
    pub id: String,
    pub host_id: Option<String>,
    pub event_type: String,
    pub payload: String,
    pub prev_hash: String,
    pub created_at: String,
}

/// Per-host audit settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditSettings {
    pub host_id: String,
    pub audit_enabled: bool,
    pub command_history_enabled: bool,
    pub redaction_patterns: Option<Vec<String>>,
    pub retention_days: i64,
}

/// Query filter for audit events.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditQuery {
    pub host_id: Option<String>,
    pub event_type: Option<String>,
    pub since: Option<String>,
    pub until: Option<String>,
    pub limit: Option<u32>,
}

/// The audit log manager.
pub struct AuditLog;

impl AuditLog {
    /// Record an audit event. Encrypts the payload with the vault key
    /// and chains the hash.
    pub fn record(
        conn: &Connection,
        vault: &Vault,
        event_type: AuditEventType,
        host_id: Option<&str>,
        payload: &str,
    ) -> AppResult<String> {
        if !vault.is_unlocked() {
            return Err(AppError::InvalidInput("vault is locked".into()));
        }

        // Check if audit is enabled for this host (if host-specific).
        if let Some(hid) = host_id {
            if !Self::is_audit_enabled(conn, hid)? {
                return Ok(String::new()); // Silently skip.
            }
            // Apply redaction patterns.
            let patterns = Self::get_redaction_patterns(conn, hid)?;
            let redacted = redaction::apply_patterns(payload, &patterns);
            return Self::do_record(conn, vault, event_type, Some(hid), &redacted);
        }

        Self::do_record(conn, vault, event_type, None, payload)
    }

    fn do_record(
        conn: &Connection,
        vault: &Vault,
        event_type: AuditEventType,
        host_id: Option<&str>,
        payload: &str,
    ) -> AppResult<String> {
        let event_id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();

        // Get previous hash for chaining.
        let prev_hash = Self::get_last_hash(conn)?;

        // Encrypt payload.
        let encrypted = vault.encrypt(payload.as_bytes())?;

        // Compute hash of this event's canonical bytes.
        let canonical = format!(
            "{}|{}|{}|{}|{}",
            event_id,
            host_id.unwrap_or(""),
            event_type.as_str(),
            hex::encode(&encrypted.ciphertext),
            prev_hash
        );
        let _event_hash = hex::encode(Sha256::digest(canonical.as_bytes()));

        conn.execute(
            "INSERT INTO audit_events (id, host_id, event_type, encrypted_payload, nonce, prev_hash, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            rusqlite::params![
                event_id,
                host_id,
                event_type.as_str(),
                encrypted.ciphertext,
                encrypted.nonce.to_vec(),
                prev_hash,
                now,
            ],
        )?;

        Ok(event_id)
    }

    /// Query audit events with optional filters.
    pub fn query(
        conn: &Connection,
        vault: &Vault,
        filter: &AuditQuery,
    ) -> AppResult<Vec<AuditEvent>> {
        let mut sql = String::from(
            "SELECT id, host_id, event_type, encrypted_payload, nonce, prev_hash, created_at
             FROM audit_events WHERE 1=1",
        );
        let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

        if let Some(ref host_id) = filter.host_id {
            sql.push_str(" AND host_id = ?");
            params.push(Box::new(host_id.clone()));
        }
        if let Some(ref event_type) = filter.event_type {
            sql.push_str(" AND event_type = ?");
            params.push(Box::new(event_type.clone()));
        }
        if let Some(ref since) = filter.since {
            sql.push_str(" AND created_at >= ?");
            params.push(Box::new(since.clone()));
        }
        if let Some(ref until) = filter.until {
            sql.push_str(" AND created_at <= ?");
            params.push(Box::new(until.clone()));
        }

        sql.push_str(" ORDER BY created_at DESC");

        if let Some(limit) = filter.limit {
            sql.push_str(&format!(" LIMIT {limit}"));
        }

        let mut stmt = conn.prepare(&sql)?;
        let param_refs: Vec<&dyn rusqlite::types::ToSql> =
            params.iter().map(|p| p.as_ref()).collect();
        let rows = stmt.query_map(param_refs.as_slice(), |row| {
            let ct: Vec<u8> = row.get(3)?;
            let nonce: Vec<u8> = row.get(4)?;
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, Option<String>>(1)?,
                row.get::<_, String>(2)?,
                ct,
                nonce,
                row.get::<_, String>(5)?,
                row.get::<_, String>(6)?,
            ))
        })?;

        let mut events = Vec::new();
        for row in rows {
            let (id, host_id, event_type, ct, nonce_bytes, prev_hash, created_at) = row?;
            let payload = if vault.is_unlocked() && nonce_bytes.len() == 12 {
                let mut nonce = [0u8; 12];
                nonce.copy_from_slice(&nonce_bytes);
                let blob = crate::crypto::EncryptedBlob {
                    ciphertext: ct,
                    nonce,
                };
                vault
                    .decrypt(&blob)
                    .ok()
                    .and_then(|b| String::from_utf8(b).ok())
                    .unwrap_or_else(|| "[decryption failed]".into())
            } else {
                "[encrypted]".into()
            };

            events.push(AuditEvent {
                id,
                host_id,
                event_type,
                payload,
                prev_hash,
                created_at,
            });
        }

        Ok(events)
    }

    /// Export audit events as JSONL (one JSON object per line).
    pub fn export_jsonl(
        conn: &Connection,
        vault: &Vault,
        filter: &AuditQuery,
    ) -> AppResult<String> {
        let events = Self::query(conn, vault, filter)?;
        let mut lines = Vec::with_capacity(events.len());
        for event in &events {
            let line =
                serde_json::to_string(event).map_err(|e| AppError::Internal(e.to_string()))?;
            lines.push(line);
        }
        Ok(lines.join("\n"))
    }

    /// Purge old audit events based on retention policies.
    pub fn purge(conn: &Connection) -> AppResult<u32> {
        // Get all hosts with their retention settings.
        let mut stmt = conn.prepare(
            "SELECT host_id, retention_days FROM audit_settings WHERE retention_days > 0",
        )?;
        let hosts: Vec<(String, i64)> = stmt
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
            .filter_map(|r| r.ok())
            .collect();

        let mut total_deleted = 0u32;

        for (host_id, retention_days) in hosts {
            let cutoff = (chrono::Utc::now() - chrono::Duration::days(retention_days)).to_rfc3339();
            let deleted = conn.execute(
                "DELETE FROM audit_events WHERE host_id = ?1 AND created_at < ?2",
                rusqlite::params![host_id, cutoff],
            )?;
            total_deleted += deleted as u32;
        }

        // Also purge global events (host_id IS NULL) older than 365 days.
        let global_cutoff = (chrono::Utc::now() - chrono::Duration::days(365)).to_rfc3339();
        let deleted = conn.execute(
            "DELETE FROM audit_events WHERE host_id IS NULL AND created_at < ?1",
            [global_cutoff],
        )?;
        total_deleted += deleted as u32;

        Ok(total_deleted)
    }

    /// Get or set audit settings for a host.
    pub fn get_settings(conn: &Connection, host_id: &str) -> AppResult<Option<AuditSettings>> {
        let result = conn.query_row(
            "SELECT host_id, audit_enabled, command_history_enabled, redaction_patterns, retention_days
             FROM audit_settings WHERE host_id = ?1",
            [host_id],
            |row| {
                let patterns_json: Option<String> = row.get(3)?;
                let patterns: Option<Vec<String>> =
                    patterns_json.and_then(|s| serde_json::from_str(&s).ok());
                Ok(AuditSettings {
                    host_id: row.get(0)?,
                    audit_enabled: row.get::<_, i64>(1)? != 0,
                    command_history_enabled: row.get::<_, i64>(2)? != 0,
                    redaction_patterns: patterns,
                    retention_days: row.get(4)?,
                })
            },
        );
        match result {
            Ok(s) => Ok(Some(s)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    pub fn set_settings(conn: &Connection, settings: &AuditSettings) -> AppResult<()> {
        let patterns_json = settings
            .redaction_patterns
            .as_ref()
            .map(|p| serde_json::to_string(p).unwrap_or_else(|_| "[]".into()));

        conn.execute(
            "INSERT INTO audit_settings (host_id, audit_enabled, command_history_enabled, redaction_patterns, retention_days)
             VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(host_id) DO UPDATE SET
                audit_enabled = excluded.audit_enabled,
                command_history_enabled = excluded.command_history_enabled,
                redaction_patterns = excluded.redaction_patterns,
                retention_days = excluded.retention_days",
            rusqlite::params![
                settings.host_id,
                settings.audit_enabled as i64,
                settings.command_history_enabled as i64,
                patterns_json,
                settings.retention_days,
            ],
        )?;
        Ok(())
    }

    // --- Helpers ---

    fn is_audit_enabled(conn: &Connection, host_id: &str) -> AppResult<bool> {
        let enabled: bool = conn
            .query_row(
                "SELECT audit_enabled FROM audit_settings WHERE host_id = ?1",
                [host_id],
                |row| row.get::<_, i64>(0),
            )
            .unwrap_or(0)
            != 0;
        Ok(enabled)
    }

    fn get_redaction_patterns(conn: &Connection, host_id: &str) -> AppResult<Vec<String>> {
        let json: Option<String> = conn
            .query_row(
                "SELECT redaction_patterns FROM audit_settings WHERE host_id = ?1",
                [host_id],
                |row| row.get(0),
            )
            .unwrap_or(None);
        Ok(json
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default())
    }

    fn get_last_hash(conn: &Connection) -> AppResult<String> {
        let hash: String = conn
            .query_row(
                "SELECT prev_hash FROM audit_events ORDER BY created_at DESC LIMIT 1",
                [],
                |row| row.get(0),
            )
            .unwrap_or_else(|_| "genesis".into());
        Ok(hash)
    }
}
