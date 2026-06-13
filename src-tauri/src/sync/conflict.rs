use super::{SyncEntityState, VersionVector};

/// Resolution strategy for conflicts.
pub enum Resolution {
    UseLocal,
    UseRemote,
}

/// Check if two version vectors indicate a conflict (concurrent edits).
/// A conflict exists when both sides have changes the other doesn't.
pub fn has_conflict(local: &VersionVector, remote: &VersionVector) -> bool {
    let local_newer = local.iter().any(|(device, &count)| {
        remote.get(device).map_or(true, |&remote_count| count > remote_count)
    });
    let remote_newer = remote.iter().any(|(device, &count)| {
        local.get(device).map_or(true, |&local_count| count > local_count)
    });
    local_newer && remote_newer
}

/// Check if the remote version is strictly newer (all devices have >= counters,
/// and at least one has >).
pub fn is_remote_newer(local: &VersionVector, remote: &VersionVector) -> bool {
    let mut has_newer = false;
    for (device, &remote_count) in remote {
        let local_count = local.get(device).copied().unwrap_or(0);
        if remote_count < local_count {
            return false;
        }
        if remote_count > local_count {
            has_newer = true;
        }
    }
    // Also check devices only in local — if local has devices remote doesn't,
    // remote is not strictly newer.
    for device in local.keys() {
        if !remote.contains_key(device) {
            return false;
        }
    }
    has_newer
}

/// Resolve a conflict using last-write-wins strategy.
/// Uses `last_synced_at` as a tiebreaker.
pub fn resolve_lww(local: &SyncEntityState, remote: &SyncEntityState) -> Resolution {
    let local_ts = local.last_synced_at.as_deref().unwrap_or("");
    let remote_ts = remote.last_synced_at.as_deref().unwrap_or("");

    // If remote is newer or equal, use remote. Otherwise keep local.
    if remote_ts >= local_ts {
        Resolution::UseRemote
    } else {
        Resolution::UseLocal
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn vv(entries: &[(&str, u64)]) -> VersionVector {
        entries
            .iter()
            .map(|(k, v)| (k.to_string(), *v))
            .collect()
    }

    #[test]
    fn no_conflict_same_version() {
        let local = vv(&[("device1", 1)]);
        let remote = vv(&[("device1", 1)]);
        assert!(!has_conflict(&local, &remote));
    }

    #[test]
    fn no_conflict_remote_newer() {
        let local = vv(&[("device1", 1)]);
        let remote = vv(&[("device1", 2)]);
        assert!(!has_conflict(&local, &remote));
        assert!(is_remote_newer(&local, &remote));
    }

    #[test]
    fn conflict_concurrent_edits() {
        let local = vv(&[("device1", 2), ("device2", 1)]);
        let remote = vv(&[("device1", 1), ("device2", 2)]);
        assert!(has_conflict(&local, &remote));
        assert!(!is_remote_newer(&local, &remote));
    }

    #[test]
    fn conflict_local_has_new_device() {
        let local = vv(&[("device1", 1), ("device3", 1)]);
        let remote = vv(&[("device1", 2)]);
        assert!(has_conflict(&local, &remote));
        assert!(!is_remote_newer(&local, &remote));
    }
}
