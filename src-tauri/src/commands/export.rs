use crate::errors::AppResult;
use crate::state::AppState;
use crate::vault::Vault;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use tauri::State;

/// Exported host data (decrypted for export, re-encrypted for file).
#[derive(Debug, Serialize, Deserialize)]
struct ExportedHost {
    label: String,
    hostname: String,
    port: i64,
    username: String,
    auth_type: String,
    credential_plaintext: String,
    group_name: Option<String>,
    tags: Option<String>,
    notes: Option<String>,
}

/// The export file format.
#[derive(Debug, Serialize, Deserialize)]
struct ExportFile {
    version: u32,
    exported_at: String,
    hosts: Vec<ExportedHost>,
}

#[tauri::command]
pub async fn export_hosts_encrypted(
    state: State<'_, AppState>,
    export_password: String,
) -> AppResult<String> {
    if export_password.len() < 8 {
        return Err(crate::errors::AppError::InvalidInput(
            "export password must be at least 8 characters".into(),
        ));
    }

    // Collect hosts with decrypted credentials.
    let hosts = {
        let conn = state.db.lock();
        collect_hosts(&conn, &state.vault)?
    };

    let export = ExportFile {
        version: 1,
        exported_at: chrono::Utc::now().to_rfc3339(),
        hosts,
    };

    let json = serde_json::to_string(&export)
        .map_err(|e| crate::errors::AppError::Internal(e.to_string()))?;

    // Encrypt the entire export with the export password.
    let salt = crate::crypto::generate_salt();
    let key = crate::crypto::derive_key(export_password.as_bytes(), &salt)?;
    let encrypted = crate::crypto::encrypt(&key, json.as_bytes())?;

    // Build the file: salt (16) + nonce (12) + ciphertext.
    let mut output = Vec::with_capacity(16 + 12 + encrypted.ciphertext.len() + 8);
    output.extend_from_slice(&[0x53, 0x4D, 0x45, 0x58]); // "SMEX" magic
    output.extend_from_slice(&[1u8]); // version
    output.extend_from_slice(&salt);
    output.extend_from_slice(&encrypted.nonce);
    output.extend_from_slice(&encrypted.ciphertext);

    // Base64 encode for easy transport.
    use base64::Engine;
    Ok(base64::engine::general_purpose::STANDARD.encode(&output))
}

#[tauri::command]
pub async fn import_hosts_encrypted(
    state: State<'_, AppState>,
    export_data: String,
    export_password: String,
) -> AppResult<u32> {
    use base64::Engine;

    let data = base64::engine::general_purpose::STANDARD
        .decode(&export_data)
        .map_err(|_| crate::errors::AppError::InvalidInput("invalid base64 data".into()))?;

    // Parse header: magic (4) + version (1) + salt (16) + nonce (12) + ciphertext.
    if data.len() < 33 {
        return Err(crate::errors::AppError::InvalidInput(
            "data too short".into(),
        ));
    }
    if &data[..4] != b"SMEX" {
        return Err(crate::errors::AppError::InvalidInput(
            "invalid file format".into(),
        ));
    }

    let salt: [u8; 16] = data[5..21].try_into().unwrap();
    let nonce: [u8; 12] = data[21..33].try_into().unwrap();
    let ciphertext = &data[33..];

    let key = crate::crypto::derive_key(export_password.as_bytes(), &salt)?;
    let blob = crate::crypto::EncryptedBlob {
        ciphertext: ciphertext.to_vec(),
        nonce,
    };
    let plaintext = crate::crypto::decrypt(&key, &blob).map_err(|_| {
        crate::errors::AppError::InvalidInput("wrong password or corrupted data".into())
    })?;

    let export: ExportFile = serde_json::from_slice(&plaintext)
        .map_err(|e| crate::errors::AppError::InvalidInput(format!("invalid export: {e}")))?;

    if export.version > 1 {
        return Err(crate::errors::AppError::InvalidInput(
            "unsupported export version".into(),
        ));
    }

    // Import hosts.
    let mut conn = state.db.lock();
    let mut imported = 0u32;

    for host in &export.hosts {
        // Save credential via vault.
        let cred_id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        let encrypted_cred = state.vault.encrypt(host.credential_plaintext.as_bytes())?;

        conn.execute(
            "INSERT OR IGNORE INTO credentials (id, type, encrypted_data, nonce, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?5)",
            rusqlite::params![
                cred_id,
                host.auth_type,
                encrypted_cred.ciphertext,
                encrypted_cred.nonce.to_vec(),
                now,
            ],
        )?;

        // Find or create group.
        let group_id = if let Some(ref group_name) = host.group_name {
            let existing: Option<String> = conn
                .query_row(
                    "SELECT id FROM groups WHERE name = ?1",
                    [group_name],
                    |row| row.get(0),
                )
                .ok();
            existing.unwrap_or_else(|| {
                let gid = uuid::Uuid::new_v4().to_string();
                conn.execute(
                    "INSERT INTO groups (id, name, sort_order) VALUES (?1, ?2, 0)",
                    rusqlite::params![gid, group_name],
                )
                .ok();
                gid
            })
        } else {
            return Err(crate::errors::AppError::InvalidInput(
                "host missing group_name".into(),
            ));
        };

        let host_id = uuid::Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO hosts (id, label, hostname, port, username, auth_type, credential_id, group_id, tags, notes, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?11)",
            rusqlite::params![
                host_id,
                host.label,
                host.hostname,
                host.port,
                host.username,
                host.auth_type,
                cred_id,
                group_id,
                host.tags,
                host.notes,
                now,
            ],
        )?;

        imported += 1;
    }

    Ok(imported)
}

fn collect_hosts(conn: &Connection, vault: &Vault) -> AppResult<Vec<ExportedHost>> {
    let mut stmt = conn.prepare(
        "SELECT h.label, h.hostname, h.port, h.username, h.auth_type,
                c.encrypted_data, c.nonce,
                g.name, h.tags, h.notes
         FROM hosts h
         JOIN credentials c ON h.credential_id = c.id
         LEFT JOIN groups g ON h.group_id = g.id
         ORDER BY h.label",
    )?;

    let rows = stmt.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, i64>(2)?,
            row.get::<_, String>(3)?,
            row.get::<_, String>(4)?,
            row.get::<_, Vec<u8>>(5)?,
            row.get::<_, Vec<u8>>(6)?,
            row.get::<_, Option<String>>(7)?,
            row.get::<_, Option<String>>(8)?,
            row.get::<_, Option<String>>(9)?,
        ))
    })?;

    let mut hosts = Vec::new();
    for row in rows {
        let (label, hostname, port, username, auth_type, ct, nonce_bytes, group, tags, notes) =
            row?;

        // Decrypt credential.
        let credential_plaintext = if nonce_bytes.len() == 12 && vault.is_unlocked() {
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
                .unwrap_or_default()
        } else {
            String::new()
        };

        hosts.push(ExportedHost {
            label,
            hostname,
            port,
            username,
            auth_type,
            credential_plaintext,
            group_name: group,
            tags,
            notes,
        });
    }

    Ok(hosts)
}
