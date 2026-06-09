use crate::db::schema::MIGRATIONS;
use crate::errors::AppResult;
use rusqlite::Connection;

/// Initialize the migrations tracking table and apply any pending migrations.
pub fn run_migrations(conn: &mut Connection) -> AppResult<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS _migrations (
            name TEXT PRIMARY KEY,
            applied_at TEXT NOT NULL
        );",
    )?;

    let applied: Vec<String> = {
        let mut stmt = conn.prepare("SELECT name FROM _migrations")?;
        let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;
        rows.collect::<Result<Vec<_>, _>>()?
    };

    for (name, sql) in MIGRATIONS {
        if applied.iter().any(|n| n == name) {
            continue;
        }

        log::info!("Applying migration: {name}");
        let tx = conn.transaction()?;
        tx.execute_batch(sql)?;
        tx.execute(
            "INSERT INTO _migrations (name, applied_at) VALUES (?1, ?2)",
            (name, chrono::Utc::now().to_rfc3339()),
        )?;
        tx.commit()?;
    }

    Ok(())
}
