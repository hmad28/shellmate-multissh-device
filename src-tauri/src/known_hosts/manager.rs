use crate::errors::{AppError, AppResult};
use base64::Engine;
use parking_lot::Mutex;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KnownHost {
    pub id: String,
    pub hostname: String,
    pub port: u16,
    pub key_type: String,
    pub fingerprint: String,
    pub public_key_blob: Vec<u8>,
    pub trusted: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HostKeyVerificationResult {
    pub verified: bool,
    #[serde(rename = "isNewHost")]
    pub is_new: bool,
    pub stored_fingerprint: Option<String>,
    pub presented_fingerprint: String,
    pub stored_key_type: Option<String>,
    #[serde(rename = "keyType")]
    pub presented_key_type: String,
    pub host_id: Option<String>,
}

#[derive(Debug)]
pub struct KnownHostsManager {
    db: Arc<Mutex<Connection>>,
}

impl KnownHostsManager {
    pub fn new(db: Arc<Mutex<Connection>>) -> Self {
        Self { db }
    }

    pub fn list(&self) -> AppResult<Vec<KnownHost>> {
        let db = self.db.lock();
        let mut stmt = db.prepare(
            "SELECT id, hostname, port, key_type, fingerprint, public_key_blob, trusted, created_at, updated_at
             FROM known_hosts
             ORDER BY hostname, port",
        )?;

        let hosts = stmt.query_map([], |row| {
            Ok(KnownHost {
                id: row.get(0)?,
                hostname: row.get(1)?,
                port: row.get::<_, i32>(2)? as u16,
                key_type: row.get(3)?,
                fingerprint: row.get(4)?,
                public_key_blob: row.get(5)?,
                trusted: row.get(6)?,
                created_at: row.get(7)?,
                updated_at: row.get(8)?,
            })
        })?;

        let mut result = Vec::new();
        for host in hosts {
            result.push(host?);
        }
        Ok(result)
    }

    pub fn verify_host_key(
        &self,
        hostname: &str,
        port: u16,
        key_type: &str,
        public_key_blob: &[u8],
    ) -> AppResult<HostKeyVerificationResult> {
        let db = self.db.lock();

        // Calculate fingerprint of presented key
        let presented_fingerprint = calculate_fingerprint(public_key_blob);

        // Look up existing host key
        let mut stmt = db.prepare(
            "SELECT id, fingerprint, key_type, trusted FROM known_hosts WHERE hostname = ?1 AND port = ?2",
        )?;

        let result = stmt.query_row(params![hostname, port as i32], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, bool>(3)?,
            ))
        });

        match result {
            Ok((host_id, stored_fingerprint, stored_key_type, trusted)) => {
                // Host exists - check if key matches
                let verified = stored_fingerprint == presented_fingerprint
                    && stored_key_type == key_type
                    && trusted;

                Ok(HostKeyVerificationResult {
                    verified,
                    is_new: false,
                    stored_fingerprint: Some(stored_fingerprint),
                    presented_fingerprint,
                    stored_key_type: Some(stored_key_type),
                    presented_key_type: key_type.to_string(),
                    host_id: Some(host_id),
                })
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => {
                // New host - TOFU
                Ok(HostKeyVerificationResult {
                    verified: false,
                    is_new: true,
                    stored_fingerprint: None,
                    presented_fingerprint,
                    stored_key_type: None,
                    presented_key_type: key_type.to_string(),
                    host_id: None,
                })
            }
            Err(e) => Err(AppError::Database(e)),
        }
    }

    pub fn trust_host_key(
        &self,
        hostname: &str,
        port: u16,
        key_type: &str,
        public_key_blob: &[u8],
    ) -> AppResult<String> {
        let db = self.db.lock();
        let fingerprint = calculate_fingerprint(public_key_blob);
        let id = Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();

        db.execute(
            "INSERT OR REPLACE INTO known_hosts (id, hostname, port, key_type, fingerprint, public_key_blob, trusted, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                id,
                hostname,
                port as i32,
                key_type,
                fingerprint,
                public_key_blob,
                true,
                now,
                now,
            ],
        )?;

        Ok(id)
    }

    pub fn update_trust(&self, id: &str, trusted: bool) -> AppResult<()> {
        let db = self.db.lock();
        let now = chrono::Utc::now().to_rfc3339();

        db.execute(
            "UPDATE known_hosts SET trusted = ?1, updated_at = ?2 WHERE id = ?3",
            params![trusted, now, id],
        )?;

        Ok(())
    }

    pub fn remove(&self, id: &str) -> AppResult<()> {
        let db = self.db.lock();
        db.execute("DELETE FROM known_hosts WHERE id = ?1", params![id])?;
        Ok(())
    }
}

fn calculate_fingerprint(public_key_blob: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(public_key_blob);
    let result = hasher.finalize();
    format!(
        "SHA256:{}",
        base64::engine::general_purpose::STANDARD.encode(result)
    )
}
