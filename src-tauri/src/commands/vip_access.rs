use crate::errors::{AppError, AppResult};
use crate::state::AppState;
use crate::vault::Vault;
use base64::Engine;
use ed25519_dalek::{SigningKey, VerifyingKey};
use parking_lot::Mutex;
use rusqlite::Connection;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use std::sync::Arc;
use tauri::State;

pub struct VipKeyStore {
    inner: Mutex<Option<VipKeyPair>>,
}

pub struct VipKeyPair {
    pub signing_key: SigningKey,
    pub verifying_key: VerifyingKey,
}

impl VipKeyStore {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(None),
        }
    }

    pub fn set(&self, keypair: VipKeyPair) {
        *self.inner.lock() = Some(keypair);
    }

    pub fn get(&self) -> Option<VipKeyPair> {
        self.inner.lock().clone()
    }

    pub fn clear(&self) {
        *self.inner.lock() = None;
    }
}

impl Clone for VipKeyPair {
    fn clone(&self) -> Self {
        Self {
            signing_key: self.signing_key.clone(),
            verifying_key: self.verifying_key,
        }
    }
}

fn get_authorized_keys_path() -> AppResult<PathBuf> {
    if cfg!(target_os = "windows") {
        let home = dirs::home_dir().ok_or_else(|| AppError::Internal("cannot determine home directory".into()))?;
        Ok(home.join(".ssh").join("authorized_keys"))
    } else if cfg!(target_os = "macos") || cfg!(target_os = "linux") {
        let home = dirs::home_dir().ok_or_else(|| AppError::Internal("cannot determine home directory".into()))?;
        Ok(home.join(".ssh").join("authorized_keys"))
    } else {
        Err(AppError::Internal("VIP access not supported on this platform".into()))
    }
}

#[tauri::command]
pub async fn vip_generate_keypair(
    state: State<'_, Arc<VipKeyStore>>,
    app_state: State<'_, AppState>,
) -> AppResult<String> {
    let mut csprng = rand_core::OsRng;
    let signing_key = SigningKey::generate(&mut csprng);
    let verifying_key = signing_key.verifying_key();

    let pubkey_hex = hex::encode(verifying_key.as_bytes());
    let privkey_bytes = signing_key.to_bytes();

    // Format private key as OpenSSH PEM
    let priv_b64 = base64::engine::general_purpose::STANDARD.encode(&privkey_bytes);
    let priv_pem = format!(
        "-----BEGIN OPENSSH PRIVATE KEY-----\n{}\n-----END OPENSSH PRIVATE KEY-----",
        priv_b64
    );

    // Encrypt private key with vault and store as credential
    let credential_id = {
        let conn = app_state.db.lock();
        let encrypted = app_state.vault.encrypt(priv_pem.as_bytes())?;
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO credentials (id, type, encrypted_data, nonce, created_at, updated_at)
             VALUES (?1, 'private_key', ?2, ?3, ?4, ?5)",
            rusqlite::params![id, encrypted.ciphertext, encrypted.nonce.to_vec(), now, now],
        )?;
        id
    };

    state.set(VipKeyPair {
        signing_key,
        verifying_key,
    });

    Ok(serde_json::json!({
        "publicKey": pubkey_hex,
        "credentialId": credential_id,
    })
    .to_string())
}

#[tauri::command]
pub async fn vip_inject_authorized_keys(
    pubkey_hex: String,
    as_admin: Option<bool>,
) -> AppResult<String> {
    // Convert hex-encoded key to base64 for authorized_keys format
    let key_bytes = hex::decode(&pubkey_hex)
        .map_err(|e| AppError::InvalidInput(format!("invalid hex public key: {e}")))?;
    let pubkey_b64 = base64::engine::general_purpose::STANDARD.encode(&key_bytes);
    let public_key_line = format!("ssh-ed25519 {} shellmate-vip", pubkey_b64);

    if as_admin.unwrap_or(false) {
        if cfg!(target_os = "windows") {
            // Write elevated PowerShell script to inject the key into C:\ProgramData\ssh\administrators_authorized_keys
            // and configure the permissions properly (only SYSTEM and Administrators).
            let ps_script = format!(
                r#"
$path = 'C:\ProgramData\ssh\administrators_authorized_keys'
$key = '{}'
if (!(Test-Path 'C:\ProgramData\ssh')) {{
    New-Item -ItemType Directory -Path 'C:\ProgramData\ssh' -Force | Out-Null
}}
$existing = ''
if (Test-Path $path) {{
    $existing = Get-Content $path -Raw
}}
if (!$existing -or !$existing.Contains($key)) {{
    $content = ($existing.Trim() + "`r`n" + $key).Trim()
    Set-Content -Path $path -Value $content -Force
}}
$acl = Get-Acl $path
$acl.SetAccessRuleProtection($true, $false)
$ar1 = New-Object System.Security.AccessControl.FileSystemAccessRule('Administrators', 'FullControl', 'Allow')
$acl.AddAccessRule($ar1)
$ar2 = New-Object System.Security.AccessControl.FileSystemAccessRule('SYSTEM', 'FullControl', 'Allow')
$acl.AddAccessRule($ar2)
Set-Acl $path $acl
"#,
                public_key_line
            );

            let temp_dir = std::env::temp_dir();
            let temp_file_path = temp_dir.join("shellmate_admin_ssh_setup.ps1");
            {
                let mut file = std::fs::File::create(&temp_file_path)?;
                file.write_all(ps_script.as_bytes())?;
            }

            let status = Command::new("powershell")
                .arg("-NoProfile")
                .arg("-NonInteractive")
                .arg("-Command")
                .arg(format!(
                    "Start-Process powershell -ArgumentList '-NoProfile -ExecutionPolicy Bypass -File \"{}\"' -Verb RunAs -Wait",
                    temp_file_path.to_string_lossy()
                ))
                .status();

            let _ = std::fs::remove_file(&temp_file_path);

            match status {
                Ok(s) if s.success() => Ok("Admin key injected and permissions configured successfully".to_string()),
                Ok(_) => Err(AppError::Internal("Failed to elevate and inject admin key. User might have cancelled UAC prompt.".into())),
                Err(e) => Err(AppError::Internal(format!("Failed to run elevated key injection script: {e}"))),
            }
        } else if cfg!(target_os = "macos") {
            let script = format!(
                "do shell script \"mkdir -p /var/root/.ssh && touch /var/root/.ssh/authorized_keys && (grep -q '{}' /var/root/.ssh/authorized_keys || echo '{}' >> /var/root/.ssh/authorized_keys) && chmod 700 /var/root/.ssh && chmod 600 /var/root/.ssh/authorized_keys\" with administrator privileges",
                public_key_line, public_key_line
            );
            let status = Command::new("osascript")
                .arg("-e")
                .arg(&script)
                .status();
            match status {
                Ok(s) if s.success() => Ok("Admin key injected to /var/root/.ssh/authorized_keys successfully".to_string()),
                Ok(_) => Err(AppError::Internal("Failed to inject root key via osascript. Permission denied.".into())),
                Err(e) => Err(AppError::Internal(format!("Failed to run osascript for root key injection: {e}"))),
            }
        } else {
            // Linux
            let cmd = format!(
                "mkdir -p /root/.ssh && touch /root/.ssh/authorized_keys && (grep -q '{}' /root/.ssh/authorized_keys || echo '{}' >> /root/.ssh/authorized_keys) && chmod 700 /root/.ssh && chmod 600 /root/.ssh/authorized_keys",
                public_key_line, public_key_line
            );
            let status = Command::new("pkexec")
                .arg("sh")
                .arg("-c")
                .arg(&cmd)
                .status();
            match status {
                Ok(s) if s.success() => Ok("Admin key injected to /root/.ssh/authorized_keys successfully".to_string()),
                Ok(_) => Err(AppError::Internal("Failed to inject root key via pkexec. Permission denied.".into())),
                Err(e) => Err(AppError::Internal(format!("Failed to run pkexec for root key injection: {e}"))),
            }
        }
    } else {
        // Standard user injection
        let key_path = get_authorized_keys_path()?;

        // Ensure .ssh directory exists
        if let Some(parent) = key_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Read existing authorized_keys
        let existing = std::fs::read_to_string(&key_path).unwrap_or_default();

        // Check if key already exists
        if existing.contains(&public_key_line) {
            return Ok("Key already exists in authorized_keys".to_string());
        }

        // Append the key
        let mut content = existing.trim_end().to_string();
        if !content.is_empty() {
            content.push('\n');
        }
        content.push_str(&public_key_line);
        content.push('\n');

        std::fs::write(&key_path, content)?;

        Ok(format!("Injected public key into {}", key_path.display()))
    }
}

#[tauri::command]
pub async fn vip_create_localhost_host(
    app_state: State<'_, AppState>,
    credential_id: String,
    label: Option<String>,
    username: Option<String>,
    as_admin: Option<bool>,
) -> AppResult<String> {
    let conn = app_state.db.lock();

    // Check if a localhost VIP host already exists with the same elevation status
    let existing: Option<String> = if as_admin.unwrap_or(false) {
        conn.query_row(
            "SELECT id FROM hosts WHERE hostname = 'localhost' AND label LIKE 'VIP%Admin%' LIMIT 1",
            [],
            |row| row.get(0),
        )
        .ok()
    } else {
        conn.query_row(
            "SELECT id FROM hosts WHERE hostname = 'localhost' AND label LIKE 'VIP%' AND label NOT LIKE '%Admin%' LIMIT 1",
            [],
            |row| row.get(0),
        )
        .ok()
    };

    if let Some(id) = existing {
        return Ok(id);
    }

    let host_id = uuid::Uuid::new_v4().to_string();
    let host_label = label.unwrap_or_else(|| {
        if as_admin.unwrap_or(false) {
            "VIP Localhost (Admin)".to_string()
        } else {
            "VIP Localhost".to_string()
        }
    });
    let now = chrono::Utc::now().to_rfc3339();

    let resolved_username = username.unwrap_or_else(|| {
        if as_admin.unwrap_or(false) {
            if cfg!(target_os = "windows") {
                "Administrator".to_string()
            } else {
                "root".to_string()
            }
        } else {
            if cfg!(target_os = "windows") {
                std::env::var("USERNAME").unwrap_or_else(|_| "Administrator".to_string())
            } else {
                std::env::var("USER").unwrap_or_else(|_| "root".to_string())
            }
        }
    });

    conn.execute(
        "INSERT INTO hosts (id, label, hostname, port, username, auth_type, credential_id, tags, created_at, updated_at)
         VALUES (?1, ?2, 'localhost', 22, ?3, 'key', ?4, '[\"vip\"]', ?5, ?6)",
        rusqlite::params![host_id, host_label, resolved_username, credential_id, now, now],
    )?;

    Ok(host_id)
}

#[tauri::command]
pub async fn vip_get_key_status(
    app_state: State<'_, AppState>,
) -> AppResult<serde_json::Value> {
    let conn = app_state.db.lock();

    // Check if VIP host exists
    let host_exists: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM hosts WHERE hostname = 'localhost' AND label LIKE 'VIP%' AND label NOT LIKE '%Admin%'",
            [],
            |row| row.get::<_, i64>(0),
        )
        .map(|c| c > 0)
        .unwrap_or(false);

    // Check if admin VIP host exists
    let admin_host_exists: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM hosts WHERE hostname = 'localhost' AND label LIKE 'VIP%Admin%'",
            [],
            |row| row.get::<_, i64>(0),
        )
        .map(|c| c > 0)
        .unwrap_or(false);

    // Check authorized_keys
    let auth_keys_injected = get_authorized_keys_path()
        .map(|path| {
            std::fs::read_to_string(&path)
                .map(|content| content.contains("shellmate-vip"))
                .unwrap_or(false)
        })
        .unwrap_or(false);

    // Check admin keys
    let admin_keys_injected = if cfg!(target_os = "windows") {
        std::fs::read_to_string("C:\\ProgramData\\ssh\\administrators_authorized_keys")
            .map(|content| content.contains("shellmate-vip"))
            .unwrap_or(false)
    } else {
        let path = if cfg!(target_os = "macos") {
            "/var/root/.ssh/authorized_keys"
        } else {
            "/root/.ssh/authorized_keys"
        };
        std::fs::read_to_string(path)
            .map(|content| content.contains("shellmate-vip"))
            .unwrap_or(false)
    };

    Ok(serde_json::json!({
        "hostExists": host_exists,
        "adminHostExists": admin_host_exists,
        "authorizedKeysInjected": auth_keys_injected,
        "adminKeysInjected": admin_keys_injected,
    }))
}
