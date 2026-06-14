use crate::errors::AppResult;
use crate::state::AppState;
use async_trait::async_trait;
use russh::client;
use russh::keys::key::PublicKey;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::State;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerStats {
    pub hostname: String,
    pub os_info: String,
    pub uptime: String,
    pub cpu_cores: u32,
    pub cpu_usage: f64,
    pub cpu_model: String,
    pub mem_total_mb: u64,
    pub mem_used_mb: u64,
    pub mem_available_mb: u64,
    pub mem_usage_percent: f64,
    pub disks: Vec<DiskInfo>,
    pub load_1m: f64,
    pub load_5m: f64,
    pub load_15m: f64,
    pub net_rx_mb: f64,
    pub net_tx_mb: f64,
    pub processes: Vec<ProcessInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiskInfo {
    pub filesystem: String,
    pub mount: String,
    pub total_gb: f64,
    pub used_gb: f64,
    pub usage_percent: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProcessInfo {
    pub pid: String,
    pub user: String,
    pub cpu_percent: f64,
    pub mem_percent: f64,
    pub command: String,
}

/// The stats-gathering script sent to remote hosts.
/// Uses only POSIX-compatible commands.
const STATS_SCRIPT: &str = r#"
echo "===OSINFO==="
(uname -srm 2>/dev/null || echo "unknown") | tr '\n' ' '
echo ""
echo "===UPTIME==="
(uptime -p 2>/dev/null || uptime 2>/dev/null || echo "unknown")
echo "===CPU==="
nproc 2>/dev/null || echo "0"
cat /proc/cpuinfo 2>/dev/null | grep "model name" | head -1 | cut -d: -f2 | xargs || echo "unknown"
echo "===CPUPERCENT==="
(top -bn1 2>/dev/null | grep "Cpu(s)" | awk '{print $2}' || echo "0")
echo "===MEMORY==="
free -m 2>/dev/null | awk '/^Mem:/ {print $2, $3, $4, $7}' || echo "0 0 0 0"
echo "===DISK==="
df -BG 2>/dev/null | awk 'NR>1 && $1!~"/dev/loop" && $1!~"tmpfs" {gsub("G",""); print $1, $6, $2, $3, $5}' || echo ""
echo "===LOAD==="
cat /proc/loadavg 2>/dev/null | awk '{print $1, $2, $3}' || echo "0 0 0"
echo "===NETWORK==="
cat /proc/net/dev 2>/dev/null | awk 'NR>2 && $1!~"lo:" {rx+=$2; tx+=$10} END {printf "%.2f %.2f", rx/1048576, tx/1048576}' || echo "0 0"
echo "===PROCESSES==="
ps aux --sort=-%cpu 2>/dev/null | head -11 | tail -10 | awk '{printf "%s|%s|%.1f|%.1f|%s\n", $2, $1, $3, $4, $11}' || echo ""
echo "===END==="
"#;

/// Execute stats-gathering commands on a remote host.
/// This creates a temporary SSH connection for non-interactive command execution.
#[tauri::command]
pub async fn server_stats_exec(
    state: State<'_, AppState>,
    host_id: String,
) -> AppResult<ServerStats> {
    // Load host config from DB.
    let (hostname, port, username, auth_type, credential_id, label) = {
        let conn = state.db.lock();
        conn.query_row(
            "SELECT hostname, port, username, auth_type, credential_id, label FROM hosts WHERE id = ?1",
            [&host_id],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, String>(5)?,
                ))
            },
        )
        .map_err(|_| crate::errors::AppError::NotFound("host not found".into()))?
    };

    // Decrypt credential.
    let credential = {
        let conn = state.db.lock();
        let (ct, nonce_bytes): (Vec<u8>, Vec<u8>) = conn.query_row(
            "SELECT encrypted_data, nonce FROM credentials WHERE id = ?1",
            [&credential_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )?;
        let mut nonce = [0u8; 12];
        nonce.copy_from_slice(&nonce_bytes);
        let blob = crate::crypto::EncryptedBlob {
            ciphertext: ct,
            nonce,
        };
        state.vault.decrypt(&blob)?
    };

    // Create a temporary SSH connection for command execution.
    let cred_str = String::from_utf8_lossy(&credential).to_string();
    let output = exec_command_over_ssh(
        &hostname,
        port as u16,
        &username,
        &auth_type,
        &cred_str,
        STATS_SCRIPT,
    )
    .await?;

    // Parse the output.
    let stats = parse_stats_output(&hostname, &output)?;
    Ok(stats)
}

/// Execute a command over SSH and return stdout.
async fn exec_command_over_ssh(
    hostname: &str,
    port: u16,
    username: &str,
    auth_type: &str,
    credential: &str,
    command: &str,
) -> AppResult<String> {
use russh::client;
use russh::keys::key::PublicKey;

    struct ExecHandler;

    #[async_trait]
    impl client::Handler for ExecHandler {
        type Error = russh::Error;

        async fn check_server_key(
            &mut self,
            _server_public_key: &PublicKey,
        ) -> Result<bool, Self::Error> {
            Ok(true)
        }
    }

    let config = client::Config {
        inactivity_timeout: Some(std::time::Duration::from_secs(30)),
        ..Default::default()
    };

    let config = Arc::new(config);
    let handler = ExecHandler;

    let mut session = client::connect(config, (hostname, port), handler)
        .await
        .map_err(|e| crate::errors::AppError::Internal(format!("SSH connect: {e}")))?;

    // Authenticate.
    let authenticated = match auth_type {
        "password" => {
            session
                .authenticate_password(username, credential)
                .await
                .map_err(|e| crate::errors::AppError::Internal(format!("auth: {e}")))?
        }
        "key" | "key_passphrase" => {
            let key_pair = russh_keys::decode_secret_key(credential, None)
                .or_else(|_| russh_keys::decode_secret_key(credential, Some(credential)))
                .map_err(|e| crate::errors::AppError::Internal(format!("key decode: {e}")))?;
            session
                .authenticate_publickey(username, Arc::new(key_pair))
                .await
                .map_err(|e| crate::errors::AppError::Internal(format!("key auth: {e}")))?
        }
        _ => {
            return Err(crate::errors::AppError::InvalidInput(
                "unsupported auth type".into(),
            ));
        }
    };

    if !authenticated {
        return Err(crate::errors::AppError::Internal("auth failed".into()));
    }

    // Open a channel and execute the command.
    let mut channel = session
        .channel_open_session()
        .await
        .map_err(|e| crate::errors::AppError::Internal(format!("channel open: {e}")))?;

    channel
        .exec(true, command)
        .await
        .map_err(|e| crate::errors::AppError::Internal(format!("exec: {e}")))?;

    // Read output.
    let mut output = String::new();
    loop {
        let msg = channel
            .wait()
            .await
            .ok_or_else(|| crate::errors::AppError::Internal("channel closed".into()))?;

        match msg {
            russh::ChannelMsg::Data { data } => {
                output.push_str(&String::from_utf8_lossy(&data));
            }
            russh::ChannelMsg::ExtendedData { data, .. } => {
                output.push_str(&String::from_utf8_lossy(&data));
            }
            russh::ChannelMsg::ExitStatus { .. } | russh::ChannelMsg::Eof => {
                break;
            }
            _ => {}
        }
    }

    session
        .disconnect(russh::Disconnect::ByApplication, "", "en")
        .await
        .ok();

    Ok(output)
}

fn parse_stats_output(hostname: &str, output: &str) -> AppResult<ServerStats> {
    let sections: Vec<&str> = output.split("===").collect();

    fn get_section<'a>(sections: &[&'a str], name: &str) -> &'a str {
        for (i, s) in sections.iter().enumerate() {
            if s.trim() == name && i + 1 < sections.len() {
                return sections[i + 1].trim();
            }
        }
        ""
    }

    let os_info = get_section(&sections, "OSINFO").to_string();
    let uptime = get_section(&sections, "UPTIME").to_string();

    let cpu_section = get_section(&sections, "CPU");
    let cpu_lines: Vec<&str> = cpu_section.lines().collect();
    let cpu_cores: u32 = cpu_lines.first().and_then(|l| l.parse().ok()).unwrap_or(1);
    let cpu_model = cpu_lines.get(1).unwrap_or(&"unknown").to_string();

    let cpu_usage: f64 = get_section(&sections, "CPUPERCENT").parse().unwrap_or(0.0);

    let mem_section = get_section(&sections, "MEMORY");
    let mem_parts: Vec<u64> = mem_section
        .split_whitespace()
        .filter_map(|s| s.parse().ok())
        .collect();
    let mem_total = mem_parts.first().copied().unwrap_or(0);
    let mem_used = mem_parts.get(1).copied().unwrap_or(0);
    let mem_available = mem_parts.get(3).copied().unwrap_or(mem_parts.get(2).copied().unwrap_or(0));
    let mem_usage = if mem_total > 0 {
        (mem_used as f64 / mem_total as f64) * 100.0
    } else {
        0.0
    };

    let disk_section = get_section(&sections, "DISK");
    let mut disks = Vec::new();
    for line in disk_section.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 5 {
            disks.push(DiskInfo {
                filesystem: parts[0].to_string(),
                mount: parts[1].to_string(),
                total_gb: parts[2].parse().unwrap_or(0.0),
                used_gb: parts[3].parse().unwrap_or(0.0),
                usage_percent: parts[4].trim_end_matches('%').parse().unwrap_or(0.0),
            });
        }
    }

    let load_section = get_section(&sections, "LOAD");
    let load_parts: Vec<f64> = load_section
        .split_whitespace()
        .filter_map(|s| s.parse().ok())
        .collect();

    let net_section = get_section(&sections, "NETWORK");
    let net_parts: Vec<f64> = net_section
        .split_whitespace()
        .filter_map(|s| s.parse().ok())
        .collect();

    let proc_section = get_section(&sections, "PROCESSES");
    let mut processes = Vec::new();
    for line in proc_section.lines() {
        let parts: Vec<&str> = line.splitn(5, '|').collect();
        if parts.len() >= 5 {
            processes.push(ProcessInfo {
                pid: parts[0].to_string(),
                user: parts[1].to_string(),
                cpu_percent: parts[2].parse().unwrap_or(0.0),
                mem_percent: parts[3].parse().unwrap_or(0.0),
                command: parts[4].to_string(),
            });
        }
    }

    Ok(ServerStats {
        hostname: hostname.to_string(),
        os_info,
        uptime,
        cpu_cores,
        cpu_usage,
        cpu_model,
        mem_total_mb: mem_total,
        mem_used_mb: mem_used,
        mem_available_mb: mem_available,
        mem_usage_percent: mem_usage,
        disks,
        load_1m: load_parts.first().copied().unwrap_or(0.0),
        load_5m: load_parts.get(1).copied().unwrap_or(0.0),
        load_15m: load_parts.get(2).copied().unwrap_or(0.0),
        net_rx_mb: net_parts.first().copied().unwrap_or(0.0),
        net_tx_mb: net_parts.get(1).copied().unwrap_or(0.0),
        processes,
    })
}

/// Execute a raw command on a remote host.
#[tauri::command]
pub async fn remote_exec(
    state: State<'_, AppState>,
    host_id: String,
    command: String,
) -> AppResult<String> {
    let (hostname, port, username, auth_type, credential_id) = {
        let conn = state.db.lock();
        conn.query_row(
            "SELECT hostname, port, username, auth_type, credential_id FROM hosts WHERE id = ?1",
            [&host_id],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                ))
            },
        )
        .map_err(|_| crate::errors::AppError::NotFound("host not found".into()))?
    };

    let credential = {
        let conn = state.db.lock();
        let (ct, nonce_bytes): (Vec<u8>, Vec<u8>) = conn.query_row(
            "SELECT encrypted_data, nonce FROM credentials WHERE id = ?1",
            [&credential_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )?;
        let mut nonce = [0u8; 12];
        nonce.copy_from_slice(&nonce_bytes);
        let blob = crate::crypto::EncryptedBlob {
            ciphertext: ct,
            nonce,
        };
        state.vault.decrypt(&blob)?
    };

    let cred_str = String::from_utf8_lossy(&credential).to_string();
    exec_command_over_ssh(&hostname, port as u16, &username, &auth_type, &cred_str, &command).await
}
