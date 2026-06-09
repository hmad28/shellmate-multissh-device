/// SQL DDL for the initial schema.
///
/// Each migration is a single string applied in order. Future migrations should
/// be added to the array; the migration runner records applied versions in the
/// `_migrations` table.
pub const MIGRATIONS: &[(&str, &str)] = &[(
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
)];
