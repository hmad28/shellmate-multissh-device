# ERD Plan - Entity Relationship Diagram
## ShellMate — Database Schema (v1.0 Production)

**Version:** 2.3
**Last Updated:** 2026-06-11

---

## 1. Overview

ShellMate uses SQLite for local data storage. The database schema is designed for:
- **Security:** Credentials stored encrypted, never in plaintext
- **Performance:** Indexed queries for fast host search
- **Flexibility:** JSON fields for extensible data (tags, settings)
- **Integrity:** Foreign key constraints, check constraints

---

## 2. Entity Relationship Diagram

```
┌─────────────────────────────────────────────────────────────────┐
│                         ERD Diagram                             │
└─────────────────────────────────────────────────────────────────┘

┌──────────────────────┐       ┌──────────────────────┐
│       groups         │       │      credentials     │
├──────────────────────┤       ├──────────────────────┤
│ id (PK)              │       │ id (PK)              │
│ name                 │       │ type                 │
│ color                │       │ encrypted_data       │
│ parent_id (FK)       │       │ nonce                │
│ sort_order           │       │ created_at           │
└──────────┬───────────┘       │ updated_at           │
           │                   └──────────┬───────────┘
           │ 1:n                          │ 1:n
           │                              │
┌──────────▼──────────────────────────────▼───────────┐
│                     hosts                           │
├─────────────────────────────────────────────────────┤
│ id (PK)                                             │
│ label                                               │
│ hostname                                            │
│ port                                                │
│ username                                            │
│ auth_type                                           │
│ credential_id (FK) ─────────────────────────────────┘
│ group_id (FK)
│ tags (JSON)
│ notes
│ created_at
│ updated_at
└──────────────────────┬──────────────────────────────┘
                       │ 1:n
                       │
┌──────────────────────▼──────────────────────────────┐
│                  port_forwards                       │
├─────────────────────────────────────────────────────┤
│ id (PK)                                             │
│ host_id (FK)                                        │
│ type (local/remote)                                 │
│ local_port                                          │
│ remote_host                                         │
│ remote_port                                         │
│ enabled                                             │
└─────────────────────────────────────────────────────┘

┌──────────────────────┐       ┌──────────────────────┐
│      snippets        │       │      settings        │
├──────────────────────┤       ├──────────────────────┤
│ id (PK)              │       │ key (PK)             │
│ title                │       │ value                │
│ command              │       └──────────────────────┘
│ description          │
│ tags (JSON)          │
│ created_at           │
│ updated_at           │
└──────────────────────┘
```

---

## 3. Detailed Table Schemas

### 3.1 groups
Stores host groups for organization.

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| `id` | TEXT | PRIMARY KEY | UUID v4 |
| `name` | TEXT | NOT NULL | Group name |
| `color` | TEXT | NULLABLE | HEX color code (e.g., "#3b82f6") |
| `parent_id` | TEXT | NULLABLE, FK → groups.id | Parent group for nesting |
| `sort_order` | INTEGER | DEFAULT 0 | Display order |

**Indexes:**
```sql
CREATE INDEX idx_groups_parent_id ON groups(parent_id);
CREATE INDEX idx_groups_sort_order ON groups(sort_order);
```

**Sample Data:**
```sql
INSERT INTO groups (id, name, color, sort_order) VALUES
('g1', 'Production', '#ef4444', 1),
('g2', 'Staging', '#f59e0b', 2),
('g3', 'Development', '#22c55e', 3);
```

---

### 3.2 credentials
Stores encrypted credentials (passwords, private keys).

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| `id` | TEXT | PRIMARY KEY | UUID v4 |
| `type` | TEXT | NOT NULL, CHECK IN ('password', 'private_key') | Credential type |
| `encrypted_data` | BLOB | NOT NULL | AES-256-GCM encrypted data |
| `nonce` | BLOB | NOT NULL | GCM nonce (12 bytes) |
| `created_at` | TEXT | NOT NULL | ISO 8601 timestamp |
| `updated_at` | TEXT | NOT NULL | ISO 8601 timestamp |

**Indexes:**
```sql
CREATE INDEX idx_credentials_type ON credentials(type);
```

**Security Notes:**
- `encrypted_data` contains the actual password or private key, encrypted with the vault key
- `nonce` is required for GCM decryption
- Never store plaintext credentials

---

### 3.3 hosts
Stores SSH host configurations.

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| `id` | TEXT | PRIMARY KEY | UUID v4 |
| `label` | TEXT | NOT NULL | Display name |
| `hostname` | TEXT | NOT NULL | IP or domain |
| `port` | INTEGER | NOT NULL, DEFAULT 22 | SSH port |
| `username` | TEXT | NOT NULL | SSH username |
| `auth_type` | TEXT | NOT NULL, CHECK IN ('password', 'key', 'key_passphrase') | Authentication method |
| `credential_id` | TEXT | NOT NULL, FK → credentials.id | Reference to credential |
| `group_id` | TEXT | NULLABLE, FK → groups.id | Reference to group |
| `tags` | TEXT | NULLABLE | JSON array of tags |
| `notes` | TEXT | NULLABLE | Free-form notes |
| `created_at` | TEXT | NOT NULL | ISO 8601 timestamp |
| `updated_at` | TEXT | NOT NULL | ISO 8601 timestamp |

**Indexes:**
```sql
CREATE INDEX idx_hosts_group_id ON hosts(group_id);
CREATE INDEX idx_hosts_credential_id ON hosts(credential_id);
CREATE INDEX idx_hosts_hostname ON hosts(hostname);
CREATE INDEX idx_hosts_label ON hosts(label);
```

**Sample Data:**
```sql
INSERT INTO hosts (id, label, hostname, port, username, auth_type, credential_id, group_id, tags) VALUES
('h1', 'Production Web', '192.168.1.100', 22, 'root', 'key', 'c1', 'g1', '["web", "nginx"]'),
('h2', 'Staging DB', '192.168.2.50', 22, 'admin', 'password', 'c2', 'g2', '["db", "mysql"]');
```

---

### 3.4 snippets
Stores frequently used commands.

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| `id` | TEXT | PRIMARY KEY | UUID v4 |
| `title` | TEXT | NOT NULL | Snippet name |
| `command` | TEXT | NOT NULL | Command template |
| `description` | TEXT | NULLABLE | Description |
| `tags` | TEXT | NULLABLE | JSON array of tags |
| `created_at` | TEXT | NOT NULL | ISO 8601 timestamp |
| `updated_at` | TEXT | NOT NULL | ISO 8601 timestamp |

**Indexes:**
```sql
CREATE INDEX idx_snippets_title ON snippets(title);
```

**Sample Data:**
```sql
INSERT INTO snippets (id, title, command, description, tags) VALUES
('s1', 'Check disk usage', 'df -h', 'Check disk space usage', '["system", "disk"]'),
('s2', 'List files', 'ls -la', 'List all files with details', '["files"]');
```

**Template Variables:**
- `{{username}}` - Current SSH username
- `{{host}}` - Current host hostname
- `{{hostname}}` - Same as host
- `{{port}}` - SSH port

---

### 3.5 port_forwards
Stores SSH port forwarding rules.

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| `id` | TEXT | PRIMARY KEY | UUID v4 |
| `host_id` | TEXT | NOT NULL, FK → hosts.id | Reference to host |
| `type` | TEXT | NOT NULL, CHECK IN ('local', 'remote') | Forwarding type |
| `local_port` | INTEGER | NOT NULL | Local port number |
| `remote_host` | TEXT | NOT NULL | Remote host address |
| `remote_port` | INTEGER | NOT NULL | Remote port number |
| `enabled` | INTEGER | NOT NULL, DEFAULT 1 | 0=disabled, 1=enabled |

**Indexes:**
```sql
CREATE INDEX idx_port_forwards_host_id ON port_forwards(host_id);
CREATE INDEX idx_port_forwards_local_port ON port_forwards(local_port);
```

**Sample Data:**
```sql
INSERT INTO port_forwards (id, host_id, type, local_port, remote_host, remote_port, enabled) VALUES
('pf1', 'h1', 'local', 3306, 'localhost', 3306, 1),
('pf2', 'h1', 'remote', 8080, 'localhost', 8080, 1);
```

---

### 3.6 settings
Stores application settings as key-value pairs.

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| `key` | TEXT | PRIMARY KEY | Setting name |
| `value` | TEXT | NOT NULL | Setting value (JSON) |

**Sample Data:**
```sql
INSERT INTO settings (key, value) VALUES
('theme', '"dark"'),
('font_family', '"JetBrains Mono"'),
('font_size', '14'),
('cursor_style', '"block"'),
('cursor_blink', 'true'),
('scrollback_lines', '5000'),
('auto_lock_timeout', '15'),
('keepalive_interval', '60'),
('vault_salt', '"base64-encoded-salt"'),
('vault_key_hash', '"argon2-hash-of-master-password"');
```

---

## 4. Relationships

### 4.1 One-to-Many Relationships

| Parent | Child | Foreign Key | On Delete |
|--------|-------|-------------|-----------|
| groups | hosts | group_id | SET NULL |
| groups | groups | parent_id | CASCADE |
| credentials | hosts | credential_id | RESTRICT |
| hosts | port_forwards | host_id | CASCADE |

### 4.2 Relationship Diagram
```
groups (1) ──────── (n) hosts
   │                      │
   │ (self-referencing)   │
   │                      │
   └──── (1) ─────── (n) port_forwards
         
credentials (1) ──── (n) hosts
```

---

## 5. Migration Strategy

### 5.1 Migration Files
```
migrations/
├── 001_initial_schema.sql
├── 002_add_indexes.sql
├── 003_add_tags_index.sql
└── ...
```

### 5.2 Migration Runner
```rust
pub fn run_migrations(conn: &Connection) -> Result<(), rusqlite::Error> {
    // Create migrations table if not exists
    conn.execute(
        "CREATE TABLE IF NOT EXISTS migrations (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            applied_at TEXT NOT NULL
        )",
        [],
    )?;
    
    // Get applied migrations
    let applied: Vec<String> = conn
        .prepare("SELECT name FROM migrations")?
        .query_map([], |row| row.get(0))?
        .filter_map(|r| r.ok())
        .collect();
    
    // Apply new migrations
    let migrations = vec![
        ("001_initial_schema", include_str!("../migrations/001_initial_schema.sql")),
        ("002_add_indexes", include_str!("../migrations/002_add_indexes.sql")),
    ];
    
    for (name, sql) in migrations {
        if !applied.contains(&name.to_string()) {
            conn.execute_batch(sql)?;
            conn.execute(
                "INSERT INTO migrations (name, applied_at) VALUES (?1, ?2)",
                params![name, chrono::Utc::now().to_rfc3339()],
            )?;
        }
    }
    
    Ok(())
}
```

---

## 6. Query Examples

### 6.1 Get All Hosts with Groups
```sql
SELECT 
    h.*,
    g.name as group_name,
    g.color as group_color
FROM hosts h
LEFT JOIN groups g ON h.group_id = g.id
ORDER BY g.sort_order, h.label;
```

### 6.2 Search Hosts
```sql
SELECT * FROM hosts
WHERE label LIKE '%' || :query || '%'
   OR hostname LIKE '%' || :query || '%'
   OR tags LIKE '%' || :query || '%'
ORDER BY label;
```

### 6.3 Get Host with Credential
```sql
SELECT 
    h.*,
    c.type as credential_type,
    c.encrypted_data,
    c.nonce
FROM hosts h
JOIN credentials c ON h.credential_id = c.id
WHERE h.id = :host_id;
```

### 6.4 Get Port Forwards for Host
```sql
SELECT * FROM port_forwards
WHERE host_id = :host_id
AND enabled = 1;
```

### 6.5 Get Snippets by Tag
```sql
SELECT * FROM snippets
WHERE tags LIKE '%' || :tag || '%'
ORDER BY title;
```

---

## 7. Security Considerations

### 7.1 Credential Encryption
- All credentials encrypted with AES-256-GCM
- Vault key derived from master password via Argon2id
- Unique nonce per encryption operation
- Credentials zeroized from memory after use

### 7.2 Database Security
- Database file stored in app-specific directory
- No plaintext credentials in database
- Foreign key constraints enforced
- SQL injection prevented via parameterized queries

### 7.3 Backup Considerations
- Database can be backed up by copying the file
- Credentials remain encrypted in backup
- Master password required to restore

---

## 8. Performance Optimization

### 8.1 Indexes
- Primary keys (automatic)
- Foreign keys for JOINs
- Search fields (label, hostname)
- JSON fields (tags) for LIKE queries

### 8.2 Query Optimization
- Use parameterized queries
- Avoid SELECT * in production code
- Use LIMIT for pagination
- Cache frequently accessed data in memory

### 8.3 Connection Pooling
- SQLite uses file-level locking
- Single connection sufficient for most cases
- Use WAL mode for concurrent reads

---

## Appendix A: v1.0 Schema Additions

The following tables are added in later phases. Migrations registered incrementally in `db/schema.rs`.

### A.1 `themes` (Phase 4)

User-defined custom themes.

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| `id` | TEXT | PRIMARY KEY | UUID v4 |
| `name` | TEXT | NOT NULL | Display name |
| `base` | TEXT | NOT NULL CHECK (base IN ('dark', 'light')) | Base style |
| `definition` | TEXT | NOT NULL | JSON ThemeDefinition (see 03-frontend-plan §12.4) |
| `is_builtin` | INTEGER | NOT NULL DEFAULT 0 | 1 for shipped themes |
| `created_at` | TEXT | NOT NULL | |
| `updated_at` | TEXT | NOT NULL | |

### A.2 `known_hosts` (Phase 6)

SSH host key fingerprints for TOFU verification.

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| `id` | TEXT | PRIMARY KEY | UUID v4 |
| `hostname` | TEXT | NOT NULL | Server hostname/IP |
| `port` | INTEGER | NOT NULL | SSH port |
| `key_type` | TEXT | NOT NULL | e.g., `ssh-ed25519`, `ssh-rsa` |
| `fingerprint` | TEXT | NOT NULL | SHA256 fingerprint |
| `public_key_blob` | BLOB | NOT NULL | Raw public key for comparison |
| `first_seen` | TEXT | NOT NULL | |
| `last_seen` | TEXT | NOT NULL | |

Index: `(hostname, port)` UNIQUE.

### A.3 `sync_state` (Phase 9)

Per-entity sync metadata.

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| `entity_type` | TEXT | NOT NULL | `host`, `group`, `snippet`, `setting`, `theme` |
| `entity_id` | TEXT | NOT NULL | FK to source entity |
| `version_vector` | TEXT | NOT NULL | JSON map device_id → counter |
| `last_synced_at` | TEXT | NULLABLE | NULL if never synced |
| `pending_change` | INTEGER | NOT NULL DEFAULT 0 | 1 if local change waiting upload |
| `remote_object_id` | TEXT | NULLABLE | Opaque cloud object ID |

Primary key: `(entity_type, entity_id)`.

### A.4 `sync_config` (Phase 9)

Sync backend configuration (encrypted via vault).

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| `id` | TEXT | PRIMARY KEY | Always `'default'` for v1.0 (single backend per device) |
| `backend_type` | TEXT | NOT NULL | `icloud`, `gdrive`, `dropbox`, `s3`, `webdav`, `http` |
| `encrypted_credentials` | BLOB | NOT NULL | Vault-encrypted JSON config |
| `nonce` | BLOB | NOT NULL | |
| `enabled` | INTEGER | NOT NULL DEFAULT 1 | Pause flag |
| `last_sync_at` | TEXT | NULLABLE | |
| `created_at` | TEXT | NOT NULL | |
| `updated_at` | TEXT | NOT NULL | |

### A.5 `audit_events` (Phase 13)

Hash-chained audit log.

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| `id` | TEXT | PRIMARY KEY | UUID v4 |
| `host_id` | TEXT | NULLABLE FK → hosts(id) | NULL for non-host events |
| `event_type` | TEXT | NOT NULL | `session_start`, `session_end`, `sftp_transfer`, `command_sent`, `vault_lock`, `vault_unlock`, etc. |
| `encrypted_payload` | BLOB | NOT NULL | Vault-encrypted event details |
| `nonce` | BLOB | NOT NULL | |
| `prev_hash` | TEXT | NOT NULL | SHA256 of previous event row's canonical bytes |
| `created_at` | TEXT | NOT NULL | |

Index: `(host_id, created_at)`, `(event_type)`.

### A.6 `audit_settings` (Phase 13)

Per-host audit opt-in.

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| `host_id` | TEXT | PRIMARY KEY FK → hosts(id) | |
| `audit_enabled` | INTEGER | NOT NULL DEFAULT 0 | Master toggle |
| `command_history_enabled` | INTEGER | NOT NULL DEFAULT 0 | Higher sensitivity, separate opt-in |
| `redaction_patterns` | TEXT | NULLABLE | JSON array of regex strings |
| `retention_days` | INTEGER | NOT NULL DEFAULT 90 | 0 = forever |

### A.7 `team` + `team_members` + `team_shares` (Phase 11)

Team vault.

```sql
CREATE TABLE team (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  team_master_key_wrapped BLOB NOT NULL,  -- wrapped with current user's key
  team_master_key_nonce BLOB NOT NULL,
  created_at TEXT NOT NULL
);

CREATE TABLE team_members (
  id TEXT PRIMARY KEY,
  team_id TEXT NOT NULL REFERENCES team(id) ON DELETE CASCADE,
  member_pubkey TEXT NOT NULL,            -- X25519 public key (base64)
  member_label TEXT NOT NULL,
  wrapped_team_key BLOB NOT NULL,         -- team master key wrapped with member_pubkey
  added_at TEXT NOT NULL,
  revoked_at TEXT NULLABLE
);

CREATE TABLE team_shares (
  id TEXT PRIMARY KEY,
  team_id TEXT NOT NULL REFERENCES team(id) ON DELETE CASCADE,
  host_id TEXT NOT NULL REFERENCES hosts(id) ON DELETE CASCADE,
  permission TEXT NOT NULL CHECK (permission IN ('read', 'edit')),
  shared_at TEXT NOT NULL
);

CREATE INDEX idx_team_members_team ON team_members(team_id);
CREATE INDEX idx_team_shares_team ON team_shares(team_id);
CREATE INDEX idx_team_shares_host ON team_shares(host_id);
```

### A.8 `plugins` + `plugin_capabilities` (Phase 12)

Plugin registry.

```sql
CREATE TABLE plugins (
  id TEXT PRIMARY KEY,                    -- plugin id from manifest (e.g., com.example.theme)
  name TEXT NOT NULL,
  version TEXT NOT NULL,
  author TEXT NOT NULL,
  api_version TEXT NOT NULL,
  manifest_path TEXT NOT NULL,            -- on-disk path to plugin dir
  signature_pubkey TEXT NOT NULL,
  enabled INTEGER NOT NULL DEFAULT 1,
  installed_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE TABLE plugin_capabilities (
  plugin_id TEXT NOT NULL REFERENCES plugins(id) ON DELETE CASCADE,
  capability TEXT NOT NULL,               -- e.g., 'network', 'filesystem', 'secrets'
  granted INTEGER NOT NULL DEFAULT 0,
  config TEXT,                            -- JSON: e.g., allow-listed network hosts
  PRIMARY KEY (plugin_id, capability)
);
```

### A.9 `biometric_state` (Phase 8)

Per-device biometric enablement (NOT synced — device-specific).

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| `id` | TEXT | PRIMARY KEY | Always `'default'` |
| `enabled` | INTEGER | NOT NULL DEFAULT 0 | |
| `wrapped_master_key` | BLOB | NULLABLE | Master key wrapped by OS biometric-protected key |
| `os_keychain_handle` | TEXT | NULLABLE | OS-specific reference (Keychain item ID etc.) |
| `enrolled_at` | TEXT | NULLABLE | |

---

## Appendix B: Update to ERD Diagram (v1.0)

The complete v1.0 ERD adds the following relationships beyond Phase 1-2:

```
hosts ──┬── audit_events (1:n)
        ├── audit_settings (1:1)
        ├── team_shares (n:m through team)
        └── port_forwards (1:n)

team ──┬── team_members (1:n)
       └── team_shares (1:n)

plugins ── plugin_capabilities (1:n)

(sync_state and sync_config are standalone — not foreign-key linked, they reference
 entities by composite (entity_type, entity_id))

themes, known_hosts, biometric_state — standalone tables.
```

---

*This document provides the complete database schema and ERD plan for ShellMate.*
