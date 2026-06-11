use crate::errors::{AppError, AppResult};
use crate::known_hosts::KnownHostsManager;
use crate::ssh::handler::ClientHandler;
use crate::ssh::session::{AuthMaterial, ConnectParams};
use parking_lot::Mutex as PlMutex;
use russh::client;
use russh::keys::decode_secret_key;
use russh_sftp::client::SftpSession;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use tokio::fs::File as TokioFile;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use uuid::Uuid;

const EVENT_SFTP_PROGRESS_PREFIX: &str = "sftp:progress:";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SftpFile {
    pub name: String,
    pub size: u64,
    pub permissions: u32,
    pub modified: i64,
    pub is_dir: bool,
    pub is_symlink: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SftpProgressEvent {
    pub transfer_id: String,
    pub bytes_transferred: u64,
    pub total_bytes: u64,
    pub filename: String,
}

pub struct SftpSessionWrapper {
    sftp: SftpSession,
    cwd: String,
}

pub struct SftpManager {
    sessions: PlMutex<HashMap<String, Arc<tokio::sync::Mutex<SftpSessionWrapper>>>>,
    ssh_params: PlMutex<HashMap<String, ConnectParams>>,
}

impl SftpManager {
    pub fn new() -> Self {
        Self {
            sessions: PlMutex::new(HashMap::new()),
            ssh_params: PlMutex::new(HashMap::new()),
        }
    }

    pub fn register_ssh_session(&self, session_id: &str, params: ConnectParams) {
        self.ssh_params
            .lock()
            .insert(session_id.to_string(), params);
    }

    pub async fn open_sftp(
        &self,
        app: AppHandle,
        known_hosts: Arc<KnownHostsManager>,
        session_id: String,
    ) -> AppResult<String> {
        let params = {
            let params_lock = self.ssh_params.lock();
            params_lock
                .get(&session_id)
                .ok_or_else(|| AppError::NotFound(format!("SSH session {}", session_id)))?
                .clone()
        };

        let config = client::Config::default();
        let mut handle = client::connect(
            Arc::new(config),
            (params.hostname.as_str(), params.port),
            ClientHandler::new(
                known_hosts,
                params.hostname.clone(),
                params.port,
                app.clone(),
                session_id.clone(),
            ),
        )
        .await
        .map_err(|e| AppError::Internal(format!("SFTP SSH connect failed: {}", e)))?;

        let auth_ok = match params.auth {
            AuthMaterial::Password { password } => {
                handle
                    .authenticate_password(&params.username, password)
                    .await
                    .map_err(|e| AppError::Internal(format!("SFTP auth error: {}", e)))?
            }
            AuthMaterial::PrivateKey {
                private_key,
                passphrase,
            } => {
                let key = decode_secret_key(&private_key, passphrase.as_deref())
                    .map_err(|e| AppError::InvalidInput(format!("invalid private key: {}", e)))?;
                handle
                    .authenticate_publickey(&params.username, Arc::new(key))
                    .await
                    .map_err(|e| AppError::Internal(format!("SFTP auth error: {}", e)))?
            }
        };

        if !auth_ok {
            return Err(AppError::InvalidInput(
                "SFTP authentication failed".into(),
            ));
        }

        let channel = handle
            .channel_open_session()
            .await
            .map_err(|e| AppError::Internal(format!("SFTP channel open failed: {}", e)))?;

        channel
            .request_subsystem(true, "sftp")
            .await
            .map_err(|e| AppError::Internal(format!("SFTP subsystem request failed: {}", e)))?;

        let sftp = SftpSession::new(channel.into_stream())
            .await
            .map_err(|e| AppError::Internal(format!("SFTP session init failed: {}", e)))?;

        let sftp_id = Uuid::new_v4().to_string();
        self.sessions.lock().insert(
            sftp_id.clone(),
            Arc::new(tokio::sync::Mutex::new(SftpSessionWrapper {
                sftp,
                cwd: ".".to_string(),
            })),
        );

        Ok(sftp_id)
    }

    pub async fn list_directory(&self, sftp_id: &str, path: Option<String>) -> AppResult<Vec<SftpFile>> {
        let wrapper_arc = {
            let sessions = self.sessions.lock();
            sessions
                .get(sftp_id)
                .cloned()
                .ok_or_else(|| AppError::NotFound(format!("SFTP session {}", sftp_id)))?
        };

        let mut wrapper = wrapper_arc.lock().await;

        let target_path = path.unwrap_or_else(|| wrapper.cwd.clone());
        
        let entries = wrapper
            .sftp
            .read_dir(&target_path)
            .await
            .map_err(|e| AppError::Internal(format!("SFTP readdir failed: {}", e)))?;

        let mut files = Vec::new();
        for entry in entries {
            let name = entry.file_name();
            
            if name == "." || name == ".." {
                continue;
            }

            let metadata = entry.metadata();
            let is_dir = metadata.is_dir();
            let is_symlink = metadata.is_symlink();
            let size = metadata.len();
            let permissions = metadata.permissions.unwrap_or(0);
            let modified = metadata.mtime.unwrap_or(0) as i64;

            files.push(SftpFile {
                name,
                size,
                permissions,
                modified,
                is_dir,
                is_symlink,
            });
        }

        wrapper.cwd = target_path;
        files.sort_by(|a, b| {
            if a.is_dir && !b.is_dir {
                std::cmp::Ordering::Less
            } else if !a.is_dir && b.is_dir {
                std::cmp::Ordering::Greater
            } else {
                a.name.to_lowercase().cmp(&b.name.to_lowercase())
            }
        });

        Ok(files)
    }

    pub async fn upload_file(
        &self,
        app: AppHandle,
        sftp_id: &str,
        local_path: PathBuf,
        remote_path: String,
    ) -> AppResult<()> {
        let transfer_id = Uuid::new_v4().to_string();
        let filename = local_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        let mut local_file = TokioFile::open(&local_path)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to open local file: {}", e)))?;

        let total_bytes = local_file
            .metadata()
            .await
            .map_err(|e| AppError::Internal(format!("Failed to read local file metadata: {}", e)))?
            .len();

        let wrapper_arc = {
            let sessions = self.sessions.lock();
            sessions
                .get(sftp_id)
                .cloned()
                .ok_or_else(|| AppError::NotFound(format!("SFTP session {}", sftp_id)))?
        };

        let mut wrapper = wrapper_arc.lock().await;

        let mut remote_file = wrapper
            .sftp
            .create(&remote_path)
            .await
            .map_err(|e| AppError::Internal(format!("SFTP create file failed: {}", e)))?;

        let mut buffer = vec![0u8; 64 * 1024];
        let mut bytes_transferred = 0u64;
        let mut last_progress = 0u64;

        loop {
            let bytes_read = local_file
                .read(&mut buffer)
                .await
                .map_err(|e| AppError::Internal(format!("Failed to read local file: {}", e)))?;

            if bytes_read == 0 {
                break;
            }

            remote_file
                .write_all(&buffer[..bytes_read])
                .await
                .map_err(|e| AppError::Internal(format!("SFTP write failed: {}", e)))?;

            bytes_transferred += bytes_read as u64;

            let progress_percent = if total_bytes > 0 {
                (bytes_transferred * 100) / total_bytes
            } else {
                0
            };

            if progress_percent > last_progress {
                emit_progress(
                    &app,
                    &transfer_id,
                    bytes_transferred,
                    total_bytes,
                    &filename,
                );
                last_progress = progress_percent;
            }
        }

        remote_file
            .shutdown()
            .await
            .map_err(|e| AppError::Internal(format!("SFTP file close failed: {}", e)))?;

        emit_progress(&app, &transfer_id, total_bytes, total_bytes, &filename);
        Ok(())
    }

    pub async fn download_file(
        &self,
        app: AppHandle,
        sftp_id: &str,
        remote_path: String,
        local_path: PathBuf,
    ) -> AppResult<()> {
        let transfer_id = Uuid::new_v4().to_string();
        let filename = Path::new(&remote_path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        let wrapper_arc = {
            let sessions = self.sessions.lock();
            sessions
                .get(sftp_id)
                .cloned()
                .ok_or_else(|| AppError::NotFound(format!("SFTP session {}", sftp_id)))?
        };

        let mut wrapper = wrapper_arc.lock().await;

        let mut remote_file = wrapper
            .sftp
            .open(&remote_path)
            .await
            .map_err(|e| AppError::Internal(format!("SFTP open file failed: {}", e)))?;

        let metadata = wrapper
            .sftp
            .metadata(&remote_path)
            .await
            .map_err(|e| AppError::Internal(format!("SFTP metadata failed: {}", e)))?;

        let total_bytes = metadata.len();

        let mut local_file = TokioFile::create(&local_path)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to create local file: {}", e)))?;

        let mut buffer = vec![0u8; 64 * 1024];
        let mut bytes_transferred = 0u64;
        let mut last_progress = 0u64;

        loop {
            let bytes_read = remote_file
                .read(&mut buffer)
                .await
                .map_err(|e| AppError::Internal(format!("SFTP read failed: {}", e)))?;

            if bytes_read == 0 {
                break;
            }

            local_file
                .write_all(&buffer[..bytes_read])
                .await
                .map_err(|e| AppError::Internal(format!("Failed to write local file: {}", e)))?;

            bytes_transferred += bytes_read as u64;

            let progress_percent = if total_bytes > 0 {
                (bytes_transferred * 100) / total_bytes
            } else {
                0
            };

            if progress_percent > last_progress {
                emit_progress(
                    &app,
                    &transfer_id,
                    bytes_transferred,
                    total_bytes,
                    &filename,
                );
                last_progress = progress_percent;
            }
        }

        local_file
            .shutdown()
            .await
            .map_err(|e| AppError::Internal(format!("Failed to close local file: {}", e)))?;

        emit_progress(&app, &transfer_id, total_bytes, total_bytes, &filename);
        Ok(())
    }

    pub async fn rename(&self, sftp_id: &str, old_path: String, new_path: String) -> AppResult<()> {
        let wrapper_arc = {
            let sessions = self.sessions.lock();
            sessions
                .get(sftp_id)
                .cloned()
                .ok_or_else(|| AppError::NotFound(format!("SFTP session {}", sftp_id)))?
        };

        let wrapper = wrapper_arc.lock().await;

        wrapper
            .sftp
            .rename(&old_path, &new_path)
            .await
            .map_err(|e| AppError::Internal(format!("SFTP rename failed: {}", e)))?;

        Ok(())
    }

    pub async fn remove(&self, sftp_id: &str, path: String) -> AppResult<()> {
        let wrapper_arc = {
            let sessions = self.sessions.lock();
            sessions
                .get(sftp_id)
                .cloned()
                .ok_or_else(|| AppError::NotFound(format!("SFTP session {}", sftp_id)))?
        };

        let wrapper = wrapper_arc.lock().await;

        let metadata = wrapper
            .sftp
            .metadata(&path)
            .await
            .map_err(|e| AppError::Internal(format!("SFTP metadata failed: {}", e)))?;

        if metadata.is_dir() {
            wrapper
                .sftp
                .remove_dir(&path)
                .await
                .map_err(|e| AppError::Internal(format!("SFTP remove dir failed: {}", e)))?;
        } else {
            wrapper
                .sftp
                .remove_file(&path)
                .await
                .map_err(|e| AppError::Internal(format!("SFTP remove file failed: {}", e)))?;
        }

        Ok(())
    }

    pub async fn mkdir(&self, sftp_id: &str, path: String) -> AppResult<()> {
        let wrapper_arc = {
            let sessions = self.sessions.lock();
            sessions
                .get(sftp_id)
                .cloned()
                .ok_or_else(|| AppError::NotFound(format!("SFTP session {}", sftp_id)))?
        };

        let wrapper = wrapper_arc.lock().await;

        wrapper
            .sftp
            .create_dir(&path)
            .await
            .map_err(|e| AppError::Internal(format!("SFTP mkdir failed: {}", e)))?;

        Ok(())
    }

    pub fn close(&self, sftp_id: &str) -> AppResult<()> {
        let mut sessions = self.sessions.lock();
        sessions.remove(sftp_id);
        Ok(())
    }

    pub fn cleanup_ssh_session(&self, session_id: &str) {
        self.ssh_params.lock().remove(session_id);
    }
}

impl Default for SftpManager {
    fn default() -> Self {
        Self::new()
    }
}

fn emit_progress(
    app: &AppHandle,
    transfer_id: &str,
    bytes_transferred: u64,
    total_bytes: u64,
    filename: &str,
) {
    let event = SftpProgressEvent {
        transfer_id: transfer_id.to_string(),
        bytes_transferred,
        total_bytes,
        filename: filename.to_string(),
    };
    let _ = app.emit(&format!("{}{}", EVENT_SFTP_PROGRESS_PREFIX, transfer_id), &event);
}
