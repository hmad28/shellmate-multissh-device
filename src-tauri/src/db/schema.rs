/// SQL DDL for the initial schema.
///
/// Each migration is a single string applied in order. Future migrations should
/// be added to the array; the migration runner records applied versions in the
/// `_migrations` table.
pub const MIGRATIONS: &[(&str, &str)] = &[
    (
        "001_initial_schema",
        r#"
    CREATE TABLE IF NOT EXISTS groups (
        id TEXT PRIMARY KEY,
        name TEXT NOT NULL,
        color TEXT,
        parent_id TEXT REFERENCES groups(id),
        sort_order INTEGER NOT NULL DEFAULT 0
    );

    CREATE INDEX IF NOT EXISTS idx_groups_parent_id ON groups(parent_id);
    CREATE INDEX IF NOT EXISTS idx_groups_sort_order ON groups(sort_order);

    CREATE TABLE IF NOT EXISTS credentials (
        id TEXT PRIMARY KEY,
        type TEXT NOT NULL CHECK (type IN ('password', 'private_key')),
        encrypted_data BLOB NOT NULL,
        nonce BLOB NOT NULL,
        created_at TEXT NOT NULL,
        updated_at TEXT NOT NULL
    );

    CREATE INDEX IF NOT EXISTS idx_credentials_type ON credentials(type);

    CREATE TABLE IF NOT EXISTS hosts (
        id TEXT PRIMARY KEY,
        label TEXT NOT NULL,
        hostname TEXT NOT NULL,
        port INTEGER NOT NULL DEFAULT 22 CHECK (port BETWEEN 1 AND 65535),
        username TEXT NOT NULL,
        auth_type TEXT NOT NULL CHECK (auth_type IN ('password', 'key', 'key_passphrase')),
        credential_id TEXT NOT NULL REFERENCES credentials(id),
        group_id TEXT REFERENCES groups(id),
        tags TEXT,
        notes TEXT,
        created_at TEXT NOT NULL,
        updated_at TEXT NOT NULL
    );

    CREATE INDEX IF NOT EXISTS idx_hosts_group_id ON hosts(group_id);
    CREATE INDEX IF NOT EXISTS idx_hosts_credential_id ON hosts(credential_id);
    CREATE INDEX IF NOT EXISTS idx_hosts_hostname ON hosts(hostname);
    CREATE INDEX IF NOT EXISTS idx_hosts_label ON hosts(label);

    CREATE TABLE IF NOT EXISTS snippets (
        id TEXT PRIMARY KEY,
        title TEXT NOT NULL,
        command TEXT NOT NULL,
        description TEXT,
        tags TEXT,
        created_at TEXT NOT NULL,
        updated_at TEXT NOT NULL
    );

    CREATE INDEX IF NOT EXISTS idx_snippets_title ON snippets(title);

    CREATE TABLE IF NOT EXISTS port_forwards (
        id TEXT PRIMARY KEY,
        host_id TEXT NOT NULL REFERENCES hosts(id) ON DELETE CASCADE,
        type TEXT NOT NULL CHECK (type IN ('local', 'remote')),
        local_port INTEGER NOT NULL CHECK (local_port BETWEEN 1 AND 65535),
        remote_host TEXT NOT NULL,
        remote_port INTEGER NOT NULL CHECK (remote_port BETWEEN 1 AND 65535),
        enabled INTEGER NOT NULL DEFAULT 1
    );

    CREATE INDEX IF NOT EXISTS idx_port_forwards_host_id ON port_forwards(host_id);

    CREATE TABLE IF NOT EXISTS settings (
        key TEXT PRIMARY KEY,
        value TEXT NOT NULL
    );
    "#,
    ),
    (
        "002_themes",
        r#"
    CREATE TABLE IF NOT EXISTS themes (
        id TEXT PRIMARY KEY,
        name TEXT NOT NULL,
        base TEXT NOT NULL CHECK (base IN ('dark', 'light')),
        definition TEXT NOT NULL,
        is_builtin INTEGER NOT NULL DEFAULT 0,
        created_at TEXT NOT NULL,
        updated_at TEXT NOT NULL
    );

    CREATE INDEX IF NOT EXISTS idx_themes_is_builtin ON themes(is_builtin);
    "#,
    ),
    (
        "003_known_hosts",
        r#"
    CREATE TABLE IF NOT EXISTS known_hosts (
        id TEXT PRIMARY KEY,
        hostname TEXT NOT NULL,
        port INTEGER NOT NULL DEFAULT 22,
        key_type TEXT NOT NULL,
        fingerprint TEXT NOT NULL,
        public_key_blob BLOB NOT NULL,
        trusted INTEGER NOT NULL DEFAULT 1,
        created_at TEXT NOT NULL,
        updated_at TEXT NOT NULL
    );

    CREATE UNIQUE INDEX IF NOT EXISTS idx_known_hosts_hostname_port ON known_hosts(hostname, port);
    CREATE INDEX IF NOT EXISTS idx_known_hosts_fingerprint ON known_hosts(fingerprint);
    "#,
    ),
    (
        "004_command_history",
        r#"
    CREATE TABLE IF NOT EXISTS command_history (
        id TEXT PRIMARY KEY,
        session_id TEXT NOT NULL,
        command TEXT NOT NULL,
        exit_code INTEGER,
        working_dir TEXT,
        executed_at TEXT NOT NULL
    );

    CREATE INDEX IF NOT EXISTS idx_command_history_session_id ON command_history(session_id);
    CREATE INDEX IF NOT EXISTS idx_command_history_executed_at ON command_history(executed_at);
    CREATE INDEX IF NOT EXISTS idx_command_history_command ON command_history(command);
    "#,
    ),
    (
        "005_biometric_state",
        r#"
    CREATE TABLE IF NOT EXISTS biometric_state (
        id TEXT PRIMARY KEY DEFAULT 'default',
        enabled INTEGER NOT NULL DEFAULT 0,
        wrapped_master_key BLOB,
        device_secret_nonce BLOB,
        os_handle TEXT,
        enrolled_at TEXT
    );
    "#,
    ),
    (
        "006_sync",
        r#"
    CREATE TABLE IF NOT EXISTS sync_config (
        id TEXT PRIMARY KEY DEFAULT 'default',
        backend_type TEXT NOT NULL CHECK (backend_type IN ('s3', 'webdav', 'http', 'gdrive', 'dropbox', 'icloud')),
        endpoint_url TEXT NOT NULL,
        encrypted_credentials BLOB,
        credentials_nonce BLOB,
        enabled INTEGER NOT NULL DEFAULT 1,
        last_sync_at TEXT,
        created_at TEXT NOT NULL,
        updated_at TEXT NOT NULL
    );

    CREATE TABLE IF NOT EXISTS sync_state (
        entity_type TEXT NOT NULL,
        entity_id TEXT NOT NULL,
        version_vector TEXT NOT NULL DEFAULT '{}',
        last_synced_at TEXT,
        pending_change INTEGER NOT NULL DEFAULT 0,
        remote_object_id TEXT,
        PRIMARY KEY (entity_type, entity_id)
    );

    CREATE INDEX IF NOT EXISTS idx_sync_state_pending ON sync_state(pending_change) WHERE pending_change = 1;
    CREATE INDEX IF NOT EXISTS idx_sync_state_remote ON sync_state(remote_object_id);
    "#,
    ),
    (
        "007_team_vault",
        r#"
    CREATE TABLE IF NOT EXISTS team (
        id TEXT PRIMARY KEY,
        name TEXT NOT NULL,
        team_master_key_wrapped BLOB NOT NULL,
        team_master_key_nonce BLOB NOT NULL,
        created_at TEXT NOT NULL
    );

    CREATE TABLE IF NOT EXISTS team_members (
        id TEXT PRIMARY KEY,
        team_id TEXT NOT NULL REFERENCES team(id) ON DELETE CASCADE,
        member_pubkey TEXT NOT NULL,
        member_label TEXT NOT NULL,
        wrapped_team_key BLOB NOT NULL,
        added_at TEXT NOT NULL,
        revoked_at TEXT
    );

    CREATE INDEX IF NOT EXISTS idx_team_members_team ON team_members(team_id);

    CREATE TABLE IF NOT EXISTS team_shares (
        id TEXT PRIMARY KEY,
        team_id TEXT NOT NULL REFERENCES team(id) ON DELETE CASCADE,
        host_id TEXT NOT NULL REFERENCES hosts(id) ON DELETE CASCADE,
        permission TEXT NOT NULL CHECK (permission IN ('read', 'edit')),
        shared_at TEXT NOT NULL
    );

    CREATE INDEX IF NOT EXISTS idx_team_shares_team ON team_shares(team_id);
    CREATE INDEX IF NOT EXISTS idx_team_shares_host ON team_shares(host_id);

    CREATE TABLE IF NOT EXISTS team_member_secrets (
        member_id TEXT PRIMARY KEY REFERENCES team_members(id) ON DELETE CASCADE,
        encrypted_secret BLOB NOT NULL,
        secret_nonce BLOB NOT NULL
    );
    "#,
    ),
    (
        "008_plugins",
        r#"
    CREATE TABLE IF NOT EXISTS plugins (
        id TEXT PRIMARY KEY,
        name TEXT NOT NULL,
        version TEXT NOT NULL,
        author TEXT NOT NULL,
        description TEXT,
        wasm_path TEXT NOT NULL,
        manifest_json TEXT NOT NULL,
        enabled INTEGER NOT NULL DEFAULT 1,
        installed_at TEXT NOT NULL,
        updated_at TEXT NOT NULL
    );

    CREATE TABLE IF NOT EXISTS plugin_capabilities (
        plugin_id TEXT NOT NULL REFERENCES plugins(id) ON DELETE CASCADE,
        capability TEXT NOT NULL,
        granted INTEGER NOT NULL DEFAULT 0,
        config TEXT,
        PRIMARY KEY (plugin_id, capability)
    );

    CREATE INDEX IF NOT EXISTS idx_plugins_enabled ON plugins(enabled);
    CREATE INDEX IF NOT EXISTS idx_plugin_capabilities_plugin ON plugin_capabilities(plugin_id);
    "#,
    ),
    (
        "009_audit_log",
        r#"
    CREATE TABLE IF NOT EXISTS audit_events (
        id TEXT PRIMARY KEY,
        host_id TEXT REFERENCES hosts(id) ON DELETE SET NULL,
        event_type TEXT NOT NULL,
        encrypted_payload BLOB NOT NULL,
        nonce BLOB NOT NULL,
        prev_hash TEXT NOT NULL,
        created_at TEXT NOT NULL
    );

    CREATE INDEX IF NOT EXISTS idx_audit_events_host_created ON audit_events(host_id, created_at);
    CREATE INDEX IF NOT EXISTS idx_audit_events_type ON audit_events(event_type);
    CREATE INDEX IF NOT EXISTS idx_audit_events_created ON audit_events(created_at);

    CREATE TABLE IF NOT EXISTS audit_settings (
        host_id TEXT PRIMARY KEY REFERENCES hosts(id) ON DELETE CASCADE,
        audit_enabled INTEGER NOT NULL DEFAULT 0,
        command_history_enabled INTEGER NOT NULL DEFAULT 0,
        redaction_patterns TEXT,
        retention_days INTEGER NOT NULL DEFAULT 90
    );
    "#,
    ),
    (
        "010_session_recording",
        r#"
    CREATE TABLE IF NOT EXISTS session_recordings (
        id TEXT PRIMARY KEY,
        session_id TEXT NOT NULL,
        host_id TEXT REFERENCES hosts(id) ON DELETE SET NULL,
        host_label TEXT NOT NULL,
        started_at TEXT NOT NULL,
        ended_at TEXT,
        duration_secs INTEGER,
        event_count INTEGER NOT NULL DEFAULT 0
    );

    CREATE TABLE IF NOT EXISTS session_events (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        recording_id TEXT NOT NULL REFERENCES session_recordings(id) ON DELETE CASCADE,
        timestamp_ms INTEGER NOT NULL,
        event_type TEXT NOT NULL,
        data TEXT NOT NULL
    );

    CREATE INDEX IF NOT EXISTS idx_session_recordings_started ON session_recordings(started_at DESC);
    CREATE INDEX IF NOT EXISTS idx_session_events_recording ON session_events(recording_id, timestamp_ms);
    "#,
    ),
    (
        "011_ssh_keys",
        r#"
    CREATE TABLE IF NOT EXISTS ssh_keys (
        id TEXT PRIMARY KEY,
        name TEXT NOT NULL,
        key_type TEXT NOT NULL CHECK (key_type IN ('ed25519', 'rsa', 'ecdsa')),
        fingerprint TEXT NOT NULL,
        public_key TEXT NOT NULL,
        encrypted_private_key BLOB NOT NULL,
        private_key_nonce BLOB NOT NULL,
        has_passphrase INTEGER NOT NULL DEFAULT 0,
        created_at TEXT NOT NULL
    );

    CREATE INDEX IF NOT EXISTS idx_ssh_keys_fingerprint ON ssh_keys(fingerprint);
    CREATE INDEX IF NOT EXISTS idx_ssh_keys_name ON ssh_keys(name);
    "#,
    ),
    (
        "012_paired_devices",
        r#"
    CREATE TABLE IF NOT EXISTS paired_devices (
        id TEXT PRIMARY KEY,
        device_name TEXT NOT NULL,
        token_hash TEXT NOT NULL,
        bound_ip TEXT NOT NULL,
        revoked_at TEXT,
        paired_at TEXT NOT NULL,
        last_seen_at TEXT NOT NULL
    );

    CREATE UNIQUE INDEX IF NOT EXISTS idx_paired_devices_token_hash ON paired_devices(token_hash);
    CREATE INDEX IF NOT EXISTS idx_paired_devices_revoked ON paired_devices(revoked_at);
    "#,
    ),
];
