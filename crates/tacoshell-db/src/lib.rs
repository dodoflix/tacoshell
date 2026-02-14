//! # Tacoshell Database
//!
//! SQLite database layer for storing servers, secrets, and their relationships.

pub mod migrations;
pub mod repository;

use rusqlite::Connection;
use std::path::Path;
use std::sync::{Arc, Mutex};
use tacoshell_core::Result;

pub use repository::{SecretRepository, ServerRepository};

/// Database connection wrapper
#[derive(Clone)]
pub struct Database {
    conn: Arc<Mutex<Connection>>,
}

impl Database {
    /// Open or create a database at the given path
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let conn = Connection::open(path)
            .map_err(|e| tacoshell_core::Error::Database(e.to_string()))?;

        let db = Self {
            conn: Arc::new(Mutex::new(conn)),
        };

        db.run_migrations()?;
        Ok(db)
    }

    /// Create an in-memory database (useful for testing)
    pub fn in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()
            .map_err(|e| tacoshell_core::Error::Database(e.to_string()))?;

        let db = Self {
            conn: Arc::new(Mutex::new(conn)),
        };

        db.run_migrations()?;
        Ok(db)
    }

    /// Run all database migrations
    fn run_migrations(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        migrations::run_all(&conn)
    }

    /// Get a secret repository
    pub fn secrets(&self) -> SecretRepository {
        SecretRepository::new(self.conn.clone())
    }

    /// Get a server repository
    pub fn servers(&self) -> ServerRepository {
        ServerRepository::new(self.conn.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_in_memory_db() {
        let db = Database::in_memory().expect("Failed to create in-memory database");
        assert!(db.conn.lock().is_ok());
    }
}
