use crate::known_hosts::KnownHostsManager;
use async_trait::async_trait;
use russh::client;
use russh::keys::key::PublicKey;
use std::sync::Arc;
use tauri::Emitter;

/// SSH client handler with TOFU (Trust On First Use) host key verification.
/// Verifies server keys against the known_hosts database.
pub struct ClientHandler {
    known_hosts: Arc<KnownHostsManager>,
    hostname: String,
    port: u16,
    app_handle: tauri::AppHandle,
    session_id: String,
}

impl ClientHandler {
    pub fn new(
        known_hosts: Arc<KnownHostsManager>,
        hostname: String,
        port: u16,
        app_handle: tauri::AppHandle,
        session_id: String,
    ) -> Self {
        Self {
            known_hosts,
            hostname,
            port,
            app_handle,
            session_id,
        }
    }
}

#[async_trait]
impl client::Handler for ClientHandler {
    type Error = russh::Error;

    async fn check_server_key(
        &mut self,
        server_public_key: &PublicKey,
    ) -> Result<bool, Self::Error> {
        // Extract key type and blob
        let key_type = match server_public_key {
            PublicKey::Ed25519(_) => "ssh-ed25519",
            PublicKey::RSA { .. } => "ssh-rsa",
            PublicKey::EC { .. } => "ecdsa-sha2-nistp256",
            _ => "unknown",
        };

        // Serialize public key to blob for storage
        let public_key_blob = server_public_key.public_key_bytes();

        // Verify the host key against known_hosts
        let result = self
            .known_hosts
            .verify_host_key(&self.hostname, self.port, key_type, &public_key_blob);

        match result {
            Ok(verification) => {
                if verification.verified {
                    // Key is known and trusted
                    log::info!(
                        "Host key verified for {}:{}",
                        self.hostname,
                        self.port
                    );
                    Ok(true)
                } else if verification.is_new {
                    // New host - emit event for user verification
                    log::info!(
                        "New host {}:{}, requesting user verification. Fingerprint: {}",
                        self.hostname,
                        self.port,
                        verification.presented_fingerprint
                    );
                    
                    // Emit verification request event
                    let _ = self.app_handle.emit(
                        "ssh:host-key-verification",
                        serde_json::json!({
                            "sessionId": self.session_id,
                            "hostname": self.hostname,
                            "port": self.port,
                            "keyType": key_type,
                            "fingerprint": verification.presented_fingerprint,
                        }),
                    );
                    
                    // Reject connection - frontend will show dialog and retry if trusted
                    Ok(false)
                } else {
                    // Fingerprint mismatch - potential MITM attack
                    log::error!(
                        "HOST KEY MISMATCH for {}:{}\n\
                         Stored: {:?}\n\
                         Presented: {}",
                        self.hostname,
                        self.port,
                        verification.stored_fingerprint,
                        verification.presented_fingerprint
                    );
                    Ok(false)
                }
            }
            Err(e) => {
                log::error!("Host key verification error: {}", e);
                // On error, reject the connection
                Ok(false)
            }
        }
    }
}
