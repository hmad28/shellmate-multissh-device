use crate::errors::AppResult;
use crate::state::AppState;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ParsedHost {
    pub label: String,
    pub hostname: String,
    pub port: u16,
    pub username: String,
    pub auth_type: String,
    pub identity_file: Option<String>,
    pub proxy_jump: Option<String>,
    pub forward_agent: bool,
    pub compression: bool,
}

/// Parse an OpenSSH config file and return discovered hosts.
#[tauri::command]
pub async fn ssh_import_config(
    _state: State<'_, AppState>,
    content: String,
) -> AppResult<Vec<ParsedHost>> {
    parse_ssh_config(&content)
}

/// Parse OpenSSH config content into host definitions.
fn parse_ssh_config(content: &str) -> AppResult<Vec<ParsedHost>> {
    let mut hosts = Vec::new();
    let mut current_host: Option<String> = None;
    let mut current_config = HostConfig::default();

    for line in content.lines() {
        let line = line.trim();

        // Skip comments and empty lines.
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let parts: Vec<&str> = line.splitn(2, char::is_whitespace).collect();
        if parts.len() < 2 {
            continue;
        }

        let key = parts[0].to_lowercase();
        let value = parts[1].trim();

        match key.as_str() {
            "host" => {
                // Save previous host if any.
                if let Some(label) = current_host.take() {
                    if label != "*" {
                        if let Some(host) = current_config.to_host(&label) {
                            hosts.push(host);
                        }
                    }
                }
                current_host = Some(value.to_string());
                current_config = HostConfig::default();
            }
            "hostname" => current_config.hostname = Some(value.to_string()),
            "port" => current_config.port = value.parse().ok(),
            "user" => current_config.user = Some(value.to_string()),
            "identityfile" => current_config.identity_file = Some(value.to_string()),
            "proxyjump" => current_config.proxy_jump = Some(value.to_string()),
            "forwardagent" => current_config.forward_agent = value.to_lowercase() == "yes",
            "compression" => current_config.compression = value.to_lowercase() == "yes",
            _ => {} // Ignore unknown directives.
        }
    }

    // Save last host.
    if let Some(label) = current_host.take() {
        if label != "*" {
            if let Some(host) = current_config.to_host(&label) {
                hosts.push(host);
            }
        }
    }

    Ok(hosts)
}

#[derive(Default)]
struct HostConfig {
    hostname: Option<String>,
    port: Option<u16>,
    user: Option<String>,
    identity_file: Option<String>,
    proxy_jump: Option<String>,
    forward_agent: bool,
    compression: bool,
}

impl HostConfig {
    fn to_host(&self, label: &str) -> Option<ParsedHost> {
        // Only include hosts that have a hostname (not just aliases).
        let hostname = self.hostname.as_deref().unwrap_or(label);

        Some(ParsedHost {
            label: label.to_string(),
            hostname: hostname.to_string(),
            port: self.port.unwrap_or(22),
            username: self.user.as_deref().unwrap_or("root").to_string(),
            auth_type: if self.identity_file.is_some() {
                "key".to_string()
            } else {
                "password".to_string()
            },
            identity_file: self.identity_file.clone(),
            proxy_jump: self.proxy_jump.clone(),
            forward_agent: self.forward_agent,
            compression: self.compression,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_simple_config() {
        let config = r#"
Host web-server
    HostName 192.168.1.10
    Port 22
    User admin

Host db-server
    HostName 192.168.1.20
    User root
    IdentityFile ~/.ssh/id_ed25519
"#;
        let hosts = parse_ssh_config(config).unwrap();
        assert_eq!(hosts.len(), 2);
        assert_eq!(hosts[0].label, "web-server");
        assert_eq!(hosts[0].hostname, "192.168.1.10");
        assert_eq!(hosts[0].port, 22);
        assert_eq!(hosts[0].username, "admin");
        assert_eq!(hosts[0].auth_type, "password");

        assert_eq!(hosts[1].label, "db-server");
        assert_eq!(hosts[1].auth_type, "key");
        assert_eq!(
            hosts[1].identity_file,
            Some("~/.ssh/id_ed25519".to_string())
        );
    }

    #[test]
    fn skip_wildcard_host() {
        let config = r#"
Host *
    Compression yes

Host myhost
    HostName example.com
"#;
        let hosts = parse_ssh_config(config).unwrap();
        assert_eq!(hosts.len(), 1);
        assert_eq!(hosts[0].label, "myhost");
    }

    #[test]
    fn parse_proxy_jump() {
        let config = r#"
Host target
    HostName 10.0.0.5
    ProxyJump bastion.example.com
    ForwardAgent yes
"#;
        let hosts = parse_ssh_config(config).unwrap();
        assert_eq!(hosts[0].proxy_jump, Some("bastion.example.com".to_string()));
        assert!(hosts[0].forward_agent);
    }
}
