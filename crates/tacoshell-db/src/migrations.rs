//! Database migrations

use rusqlite::Connection;
use tacoshell_core::Result;
use tracing::info;

/// Run all database migrations
pub fn run_all(conn: &Connection) -> Result<()> {
    info!("Running database migrations");

    // Create migrations tracking table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS _migrations (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL UNIQUE,
            applied_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )
    .map_err(|e| tacoshell_core::Error::Database(e.to_string()))?;

    // Run each migration if not already applied
    let migrations: Vec<(&str, &str)> = vec![
        ("001_initial_schema", include_str!("../migrations/001_initial_schema.sql")),
    ];

    for (name, sql) in migrations {
        if !is_migration_applied(conn, name)? {
            info!("Applying migration: {}", name);
            conn.execute_batch(sql)
                .map_err(|e| tacoshell_core::Error::Database(format!("Migration {} failed: {}", name, e)))?;
            mark_migration_applied(conn, name)?;
        }
    }

    info!("Migrations complete");
    Ok(())
}

fn is_migration_applied(conn: &Connection, name: &str) -> Result<bool> {
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM _migrations WHERE name = ?1",
            [name],
            |row| row.get(0),
        )
        .map_err(|e| tacoshell_core::Error::Database(e.to_string()))?;
    Ok(count > 0)
}

fn mark_migration_applied(conn: &Connection, name: &str) -> Result<()> {
    conn.execute("INSERT INTO _migrations (name) VALUES (?1)", [name])
        .map_err(|e| tacoshell_core::Error::Database(e.to_string()))?;
    Ok(())
}

