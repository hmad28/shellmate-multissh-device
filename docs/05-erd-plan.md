# ERD Plan - Entity Relationship Diagram
## ShellMate - Database Schema

**Version:** 1.1
**Last Updated:** 2026-06-09

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

*This document provides the complete database schema and ERD plan for ShellMate.*
