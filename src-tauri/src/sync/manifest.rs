use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::VersionVector;

/// A sync manifest describes the current state of all synced entities.
/// Stored encrypted in the cloud alongside individual entity payloads.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncManifest {
    /// Device that last wrote this manifest.
    pub device_id: String,
    /// Timestamp of last update.
    pub updated_at: String,
    /// Entity version vectors.
    pub entities: HashMap<String, ManifestEntry>,
}

/// Per-entity manifest entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestEntry {
    pub entity_type: String,
    pub version_vector: VersionVector,
    /// Opaque cloud object ID.
    pub object_id: String,
    /// SHA-256 of the encrypted payload for integrity verification.
    pub content_hash: String,
}

impl SyncManifest {
    pub fn new(device_id: String) -> Self {
        Self {
            device_id,
            updated_at: chrono::Utc::now().to_rfc3339(),
            entities: HashMap::new(),
        }
    }
}
