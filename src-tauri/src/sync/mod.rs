pub mod backend;
pub mod conflict;
pub mod encrypt;
pub mod manifest;

use crate::errors::{AppError, AppResult};
use crate::sync::encrypt::{decrypt_credentials, encrypt_credentials, derive_sync_payload_key};
use crate::vault::Vault;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use parking_lot::RwLock;

/// Entity types that participate in sync.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EntityType {
    Host, Group, Snippet, Setting, Theme,
}

impl EntityType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Host => "host", Self::Group => "group", Self::Snippet => "snippet",
            Self::Setting => "setting", Self::Theme => "theme",
        }
    }
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "host" => Some(Self::Host), "group" => Some(Self::Group), "snippet" => Some(Self::Snippet),
            "setting" => Some(Self::Setting), "theme" => Some(Self::Theme), _ => None,
        }
    }
}

pub type VersionVector = HashMap<String, u64>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncEntityState {
    pub entity_type: EntityType,
    pub entity_id: String,
    pub version_vector: VersionVector,
    pub last_synced_at: Option<String>,
    pub pending_change: bool,
    pub remote_object_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncConfig {
    pub backend_type: String,
    pub endpoint_url: String,
    pub enabled: bool,
    pub last_sync_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncStatus {
    pub configured: bool,
    pub enabled: bool,
    pub backend_type: Option<String>,
    pub endpoint_url: Option<String>,
    pub last_sync_at: Option<String>,
    pub pending_changes: u32,
    pub synced_entities: u32,
    pub device_id: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncResult {
    pub uploaded: u32,
    pub downloaded: u32,
    pub conflicts: u32,
}

pub struct SyncEngine {
    device_id: RwLock<Option<String>>,
}

impl SyncEngine {
    pub fn new() -> Self {
        Self { device_id: RwLock::new(None) }
    }

    pub fn device_id(&self, conn: &Connection) -> String {
        let mut id = self.device_id.write();
        if id.is_none() {
            let stored: Option<String> = conn
                .query_row("SELECT value FROM settings WHERE key = 'sync.device_id'", [], |row| row.get(0))
                .ok();
            if let Some(stored_id) = stored {
                *id = Some(stored_id);
            } else {
                let new_id = uuid::Uuid::new_v4().to_string();
                conn.execute(
                    "INSERT INTO settings (key, value) VALUES ('sync.device_id', ?1) ON CONFLICT(key) DO UPDATE SET value = excluded.value",
                    rusqlite::params![new_id],
                ).ok();
                *id = Some(new_id);
            }
        }
        id.clone().unwrap()
    }

    pub fn status(&self, conn: &Connection) -> AppResult<SyncStatus> {
        let config = load_config(conn)?;
        let pending = count_pending(conn)?;
        let synced = count_synced(conn)?;
        let device_id = self.device_id(conn);
        Ok(SyncStatus {
            configured: config.is_some(),
            enabled: config.as_ref().map(|c| c.enabled).unwrap_or(false),
            backend_type: config.as_ref().map(|c| c.backend_type.clone()),
            endpoint_url: config.as_ref().map(|c| c.endpoint_url.clone()),
            last_sync_at: config.as_ref().and_then(|c| c.last_sync_at.clone()),
            pending_changes: pending,
            synced_entities: synced,
            device_id,
        })
    }

    pub fn configure(&self, conn: &Connection, vault: &Vault, backend_type: &str, endpoint_url: &str, credentials_json: &str) -> AppResult<()> {
        if !vault.is_unlocked() {
            return Err(AppError::InvalidInput("vault must be unlocked to configure sync".into()));
        }
        let vault_key = vault.get_vault_key()
            .ok_or_else(|| AppError::InvalidInput("vault key not available".into()))?;
        let (encrypted_creds, nonce) = encrypt_credentials(credentials_json.as_bytes(), &vault_key)?;
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO sync_config (id, backend_type, endpoint_url, encrypted_credentials, credentials_nonce, enabled, created_at, updated_at)
             VALUES ('default', ?1, ?2, ?3, ?4, 1, ?5, ?5)
             ON CONFLICT(id) DO UPDATE SET backend_type=excluded.backend_type, endpoint_url=excluded.endpoint_url,
             encrypted_credentials=excluded.encrypted_credentials, credentials_nonce=excluded.credentials_nonce, updated_at=excluded.updated_at",
            rusqlite::params![backend_type, endpoint_url, encrypted_creds, nonce.to_vec(), now],
        )?;
        Ok(())
    }

    pub fn set_enabled(&self, conn: &Connection, enabled: bool) -> AppResult<()> {
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute("UPDATE sync_config SET enabled=?1, updated_at=?2 WHERE id='default'", rusqlite::params![enabled as i64, now])?;
        Ok(())
    }

    pub fn mark_changed(&self, conn: &Connection, entity_type: EntityType, entity_id: &str) -> AppResult<()> {
        let device_id = self.device_id(conn);
        let existing: Option<String> = conn
            .query_row("SELECT version_vector FROM sync_state WHERE entity_type=?1 AND entity_id=?2",
                rusqlite::params![entity_type.as_str(), entity_id], |row| row.get(0)).ok();
        let mut vv: VersionVector = existing.and_then(|s| serde_json::from_str(&s).ok()).unwrap_or_default();
        *vv.entry(device_id).or_insert(0) += 1;
        let vv_json = serde_json::to_string(&vv).unwrap_or_else(|_| "{}".into());
        conn.execute(
            "INSERT INTO sync_state (entity_type, entity_id, version_vector, pending_change) VALUES (?1,?2,?3,1)
             ON CONFLICT(entity_type, entity_id) DO UPDATE SET version_vector=excluded.version_vector, pending_change=1",
            rusqlite::params![entity_type.as_str(), entity_id, vv_json],
        )?;
        Ok(())
    }

    pub async fn sync_now(&self, state: &crate::state::AppState) -> AppResult<SyncResult> {
        let (config, credentials) = {
            let conn = state.db.lock();
            if !state.vault.is_unlocked() {
                return Err(AppError::InvalidInput("vault must be unlocked to sync".into()));
            }
            let config = load_config(&conn)?.ok_or_else(|| AppError::InvalidInput("sync not configured".into()))?;
            if !config.enabled {
                return Err(AppError::InvalidInput("sync is disabled".into()));
            }
            let vault_key = state.vault.get_vault_key()
                .ok_or_else(|| AppError::Internal("vault key not available".into()))?;
            let creds = load_credentials(&conn)?
                .and_then(|(ct, nonce)| decrypt_credentials(&ct, &nonce, &vault_key).ok())
                .and_then(|b| String::from_utf8(b).ok());
            (config, creds)
        };

        let backend = backend::create_backend(&config.backend_type, &config.endpoint_url, credentials.as_deref())?;
        let sync_key = {
            let vault_key = state.vault.get_vault_key()
                .ok_or_else(|| AppError::Internal("vault key not available".into()))?;
            derive_sync_payload_key(&vault_key)
        };
        let device_id = { let conn = state.db.lock(); self.device_id(&conn) };
        let mut uploaded = 0u32;
        let mut downloaded = 0u32;
        let mut conflicts = 0u32;

        // Phase 1: Upload pending.
        let pending = { let conn = state.db.lock(); list_pending(&conn)? };
        for entity in &pending {
            let payload = { let conn = state.db.lock(); export_entity(&conn, entity)? };
            if let Some(payload) = payload {
                let encrypted = encrypt::encrypt_payload(&payload, &sync_key)?;
                let object_id = entity.remote_object_id.clone().unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
                backend.put(&object_id, &encrypted).await?;
                let now = chrono::Utc::now().to_rfc3339();
                let conn = state.db.lock();
                conn.execute("UPDATE sync_state SET pending_change=0, remote_object_id=?1, last_synced_at=?2 WHERE entity_type=?3 AND entity_id=?4",
                    rusqlite::params![object_id, now, entity.entity_type.as_str(), entity.entity_id])?;
                uploaded += 1;
            }
        }

        // Phase 2: Download remote.
        let remote_objects = backend.list().await?;
        for object_id in &remote_objects {
            let encrypted = backend.get(object_id).await?;
            let payload = encrypt::decrypt_payload(&encrypted, &sync_key)?;
            let remote_entity: SyncEntityState = serde_json::from_slice(&payload)
                .map_err(|e| AppError::Internal(format!("invalid sync payload: {e}")))?;
            let local_state = { let conn = state.db.lock(); get_entity_state(&conn, remote_entity.entity_type, &remote_entity.entity_id)? };
            match local_state {
                Some(local) => {
                    if conflict::has_conflict(&local.version_vector, &remote_entity.version_vector) {
                        match conflict::resolve_lww(&local, &remote_entity) {
                            conflict::Resolution::UseLocal => { conflicts += 1; }
                            conflict::Resolution::UseRemote => {
                                let conn = state.db.lock(); import_entity(&conn, &remote_entity, &payload)?; downloaded += 1;
                            }
                        }
                    } else if conflict::is_remote_newer(&local.version_vector, &remote_entity.version_vector) {
                        let conn = state.db.lock(); import_entity(&conn, &remote_entity, &payload)?; downloaded += 1;
                    }
                }
                None => { let conn = state.db.lock(); import_entity(&conn, &remote_entity, &payload)?; downloaded += 1; }
            }
        }

        { let now = chrono::Utc::now().to_rfc3339(); let conn = state.db.lock();
          conn.execute("UPDATE sync_config SET last_sync_at=?1 WHERE id='default'", [now])?; }
        Ok(SyncResult { uploaded, downloaded, conflicts })
    }
}

// --- DB helpers ---

fn load_config(conn: &Connection) -> AppResult<Option<SyncConfig>> {
    match conn.query_row(
        "SELECT backend_type, endpoint_url, enabled, last_sync_at FROM sync_config WHERE id='default'", [],
        |row| Ok(SyncConfig { backend_type: row.get(0)?, endpoint_url: row.get(1)?, enabled: row.get::<_, i64>(2)? != 0, last_sync_at: row.get(3)? }),
    ) {
        Ok(c) => Ok(Some(c)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.into()),
    }
}

fn load_credentials(conn: &Connection) -> AppResult<Option<(Vec<u8>, [u8; 12])>> {
    match conn.query_row(
        "SELECT encrypted_credentials, credentials_nonce FROM sync_config WHERE id='default'", [],
        |row| { let ct: Vec<u8> = row.get(0)?; let n: Vec<u8> = row.get(1)?; Ok((ct, n)) },
    ) {
        Ok((ct, nv)) => {
            if nv.len() != 12 { return Err(AppError::Internal("invalid nonce length".into())); }
            let mut nonce = [0u8; 12]; nonce.copy_from_slice(&nv); Ok(Some((ct, nonce)))
        }
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.into()),
    }
}

fn count_pending(conn: &Connection) -> AppResult<u32> {
    Ok(conn.query_row("SELECT COUNT(*) FROM sync_state WHERE pending_change=1", [], |r| r.get::<_, i64>(0))? as u32)
}

fn count_synced(conn: &Connection) -> AppResult<u32> {
    Ok(conn.query_row("SELECT COUNT(*) FROM sync_state WHERE last_synced_at IS NOT NULL", [], |r| r.get::<_, i64>(0))? as u32)
}

fn list_pending(conn: &Connection) -> AppResult<Vec<SyncEntityState>> {
    let mut stmt = conn.prepare("SELECT entity_type, entity_id, version_vector, last_synced_at, pending_change, remote_object_id FROM sync_state WHERE pending_change=1")?;
    let mut result = Vec::new();
    let mut rows = stmt.query([])?;
    while let Some(row) = rows.next()? {
        let et: String = row.get(0)?;
        let vv: String = row.get(2)?;
        result.push(SyncEntityState {
            entity_type: EntityType::from_str(&et).unwrap_or(EntityType::Host),
            entity_id: row.get(1)?,
            version_vector: serde_json::from_str(&vv).unwrap_or_default(),
            last_synced_at: row.get(3)?,
            pending_change: row.get::<_, i64>(4)? != 0,
            remote_object_id: row.get(5)?,
        });
    }
    Ok(result)
}

fn get_entity_state(conn: &Connection, entity_type: EntityType, entity_id: &str) -> AppResult<Option<SyncEntityState>> {
    match conn.query_row(
        "SELECT entity_type, entity_id, version_vector, last_synced_at, pending_change, remote_object_id FROM sync_state WHERE entity_type=?1 AND entity_id=?2",
        rusqlite::params![entity_type.as_str(), entity_id],
        |row| {
            let et: String = row.get(0)?;
            let vv: String = row.get(2)?;
            Ok(SyncEntityState {
                entity_type: EntityType::from_str(&et).unwrap_or(EntityType::Host),
                entity_id: row.get(1)?,
                version_vector: serde_json::from_str(&vv).unwrap_or_default(),
                last_synced_at: row.get(3)?,
                pending_change: row.get::<_, i64>(4)? != 0,
                remote_object_id: row.get(5)?,
            })
        },
    ) {
        Ok(s) => Ok(Some(s)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.into()),
    }
}

fn export_entity(conn: &Connection, entity: &SyncEntityState) -> AppResult<Option<Vec<u8>>> {
    let data = match entity.entity_type {
        EntityType::Host => conn.query_row("SELECT json_object('id',id,'label',label,'hostname',hostname,'port',port,'username',username,'auth_type',auth_type,'group_id',group_id,'tags',tags,'notes',notes,'created_at',created_at,'updated_at',updated_at) FROM hosts WHERE id=?1", [&entity.entity_id], |r| r.get::<_, String>(0)).ok(),
        EntityType::Group => conn.query_row("SELECT json_object('id',id,'name',name,'color',color,'parent_id',parent_id,'sort_order',sort_order) FROM groups WHERE id=?1", [&entity.entity_id], |r| r.get::<_, String>(0)).ok(),
        EntityType::Snippet => conn.query_row("SELECT json_object('id',id,'title',title,'command',command,'description',description,'tags',tags,'created_at',created_at,'updated_at',updated_at) FROM snippets WHERE id=?1", [&entity.entity_id], |r| r.get::<_, String>(0)).ok(),
        EntityType::Setting => conn.query_row("SELECT json_object('key',key,'value',value) FROM settings WHERE key=?1", [&entity.entity_id], |r| r.get::<_, String>(0)).ok(),
        EntityType::Theme => conn.query_row("SELECT json_object('id',id,'name',name,'base',base,'definition',definition) FROM themes WHERE id=?1", [&entity.entity_id], |r| r.get::<_, String>(0)).ok(),
    };
    Ok(data.map(|s| s.into_bytes()))
}

fn import_entity(conn: &Connection, entity: &SyncEntityState, _payload: &[u8]) -> AppResult<()> {
    let vv_json = serde_json::to_string(&entity.version_vector).unwrap_or_else(|_| "{}".into());
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO sync_state (entity_type, entity_id, version_vector, last_synced_at, pending_change, remote_object_id)
         VALUES (?1,?2,?3,?4,0,?5)
         ON CONFLICT(entity_type, entity_id) DO UPDATE SET version_vector=excluded.version_vector, last_synced_at=excluded.last_synced_at, pending_change=0, remote_object_id=excluded.remote_object_id",
        rusqlite::params![entity.entity_type.as_str(), entity.entity_id, vv_json, now, entity.remote_object_id],
    )?;
    Ok(())
}
