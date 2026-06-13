use serde::{Deserialize, Serialize};

/// Plugin manifest — describes a plugin's metadata and capabilities.
/// Stored as JSON alongside the WASM binary.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    /// Unique plugin ID (e.g., "com.example.my-plugin").
    pub id: String,
    /// Human-readable name.
    pub name: String,
    /// Semantic version.
    pub version: String,
    /// Plugin author.
    pub author: String,
    /// Short description.
    pub description: Option<String>,
    /// ShellMate API version this plugin targets.
    pub api_version: String,
    /// Declared capabilities the plugin requests.
    pub capabilities: Vec<CapabilityDecl>,
    /// Ed25519 public key for signature verification (base64).
    pub signature_pubkey: Option<String>,
}

/// A capability declaration in the manifest.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityDecl {
    /// Capability name (e.g., "log", "network", "filesystem", "panel", "terminal_data", "secrets").
    pub name: String,
    /// Optional configuration (e.g., allowed hosts for network, scoped paths for filesystem).
    pub config: Option<String>,
}

/// Validate a manifest for required fields and format.
pub fn validate_manifest(manifest: &PluginManifest) -> Result<(), String> {
    if manifest.id.is_empty() {
        return Err("plugin id is required".into());
    }
    if manifest.name.is_empty() {
        return Err("plugin name is required".into());
    }
    if manifest.version.is_empty() {
        return Err("plugin version is required".into());
    }
    if manifest.author.is_empty() {
        return Err("plugin author is required".into());
    }
    if manifest.api_version.is_empty() {
        return Err("api_version is required".into());
    }

    // Validate capability names.
    let valid_caps = [
        "log",
        "panel",
        "terminal_data",
        "network",
        "filesystem",
        "secrets",
    ];
    for cap in &manifest.capabilities {
        if !valid_caps.contains(&cap.name.as_str()) {
            return Err(format!(
                "unknown capability: '{}'. Valid: {:?}",
                cap.name, valid_caps
            ));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_manifest() -> PluginManifest {
        PluginManifest {
            id: "test.plugin".into(),
            name: "Test Plugin".into(),
            version: "1.0.0".into(),
            author: "Test".into(),
            description: None,
            api_version: "1".into(),
            capabilities: vec![],
            signature_pubkey: None,
        }
    }

    #[test]
    fn valid_manifest_passes() {
        assert!(validate_manifest(&valid_manifest()).is_ok());
    }

    #[test]
    fn empty_id_fails() {
        let mut m = valid_manifest();
        m.id = "".into();
        assert!(validate_manifest(&m).is_err());
    }

    #[test]
    fn unknown_capability_fails() {
        let mut m = valid_manifest();
        m.capabilities.push(CapabilityDecl {
            name: "invalid_cap".into(),
            config: None,
        });
        assert!(validate_manifest(&m).is_err());
    }
}
