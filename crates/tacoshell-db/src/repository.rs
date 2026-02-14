//! Repository implementations for database access

use rusqlite::{params, Connection, Row};
use std::sync::{Arc, Mutex};
use tacoshell_core::{Error, Protocol, Result, Secret, SecretKind, Server, ServerSecret};
use uuid::Uuid;

/// Repository for secret operations
pub struct SecretRepository {
    conn: Arc<Mutex<Connection>>,
}

impl SecretRepository {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    /// Store a new secret
    pub fn store(&self, secret: &Secret) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO secrets (id, name, kind, encrypted_value, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                secret.id.to_string(),
                secret.name,
                serde_json::to_string(&secret.kind).unwrap().trim_matches('"'),
                secret.encrypted_value,
                secret.created_at.to_rfc3339(),
                secret.updated_at.to_rfc3339(),
            ],
        )
        .map_err(|e| Error::Database(e.to_string()))?;
        Ok(())
    }

    /// Get a secret by ID
    pub fn get(&self, id: Uuid) -> Result<Option<Secret>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare("SELECT id, name, kind, encrypted_value, created_at, updated_at FROM secrets WHERE id = ?1")
            .map_err(|e| Error::Database(e.to_string()))?;

        let result = stmt
            .query_row([id.to_string()], |row| Self::row_to_secret(row))
            .optional()
            .map_err(|e| Error::Database(e.to_string()))?;

        Ok(result)
    }

    /// List all secrets
    pub fn list(&self) -> Result<Vec<Secret>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare("SELECT id, name, kind, encrypted_value, created_at, updated_at FROM secrets ORDER BY name")
            .map_err(|e| Error::Database(e.to_string()))?;

        let secrets = stmt
            .query_map([], |row| Self::row_to_secret(row))
            .map_err(|e| Error::Database(e.to_string()))?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| Error::Database(e.to_string()))?;

        Ok(secrets)
    }

    /// Delete a secret by ID
    pub fn delete(&self, id: Uuid) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM secrets WHERE id = ?1", [id.to_string()])
            .map_err(|e| Error::Database(e.to_string()))?;
        Ok(())
    }

    /// Update an existing secret
    pub fn update(&self, secret: &Secret) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE secrets SET name = ?2, kind = ?3, encrypted_value = ?4, updated_at = ?5 WHERE id = ?1",
            params![
                secret.id.to_string(),
                secret.name,
                serde_json::to_string(&secret.kind).unwrap().trim_matches('"'),
                secret.encrypted_value,
                chrono::Utc::now().to_rfc3339(),
            ],
        )
        .map_err(|e| Error::Database(e.to_string()))?;
        Ok(())
    }

    fn row_to_secret(row: &Row) -> rusqlite::Result<Secret> {
        let id_str: String = row.get(0)?;
        let kind_str: String = row.get(2)?;
        let created_str: String = row.get(4)?;
        let updated_str: String = row.get(5)?;

        Ok(Secret {
            id: Uuid::parse_str(&id_str).unwrap(),
            name: row.get(1)?,
            kind: match kind_str.as_str() {
                "password" => SecretKind::Password,
                "private_key" => SecretKind::PrivateKey,
                "token" => SecretKind::Token,
                "kubeconfig" => SecretKind::Kubeconfig,
                _ => SecretKind::Password,
            },
            encrypted_value: row.get(3)?,
            created_at: chrono::DateTime::parse_from_rfc3339(&created_str)
                .unwrap()
                .with_timezone(&chrono::Utc),
            updated_at: chrono::DateTime::parse_from_rfc3339(&updated_str)
                .unwrap()
                .with_timezone(&chrono::Utc),
        })
    }
}

/// Repository for server operations
pub struct ServerRepository {
    conn: Arc<Mutex<Connection>>,
}

impl ServerRepository {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    /// Store a new server
    pub fn store(&self, server: &Server) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO servers (id, name, host, port, username, protocol, tags, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                server.id.to_string(),
                server.name,
                server.host,
                server.port,
                server.username,
                serde_json::to_string(&server.protocol).unwrap().trim_matches('"'),
                serde_json::to_string(&server.tags).unwrap(),
                server.created_at.to_rfc3339(),
                server.updated_at.to_rfc3339(),
            ],
        )
        .map_err(|e| Error::Database(e.to_string()))?;
        Ok(())
    }

    /// Get a server by ID
    pub fn get(&self, id: Uuid) -> Result<Option<Server>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare("SELECT id, name, host, port, username, protocol, tags, created_at, updated_at FROM servers WHERE id = ?1")
            .map_err(|e| Error::Database(e.to_string()))?;

        let result = stmt
            .query_row([id.to_string()], |row| Self::row_to_server(row))
            .optional()
            .map_err(|e| Error::Database(e.to_string()))?;

        Ok(result)
    }

    /// List all servers
    pub fn list(&self) -> Result<Vec<Server>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare("SELECT id, name, host, port, username, protocol, tags, created_at, updated_at FROM servers ORDER BY name")
            .map_err(|e| Error::Database(e.to_string()))?;

        let servers = stmt
            .query_map([], |row| Self::row_to_server(row))
            .map_err(|e| Error::Database(e.to_string()))?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| Error::Database(e.to_string()))?;

        Ok(servers)
    }

    /// Delete a server by ID
    pub fn delete(&self, id: Uuid) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM servers WHERE id = ?1", [id.to_string()])
            .map_err(|e| Error::Database(e.to_string()))?;
        Ok(())
    }

    /// Update an existing server
    pub fn update(&self, server: &Server) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE servers SET name = ?2, host = ?3, port = ?4, username = ?5, protocol = ?6, tags = ?7, updated_at = ?8 WHERE id = ?1",
            params![
                server.id.to_string(),
                server.name,
                server.host,
                server.port,
                server.username,
                serde_json::to_string(&server.protocol).unwrap().trim_matches('"'),
                serde_json::to_string(&server.tags).unwrap(),
                chrono::Utc::now().to_rfc3339(),
            ],
        )
        .map_err(|e| Error::Database(e.to_string()))?;
        Ok(())
    }

    /// Link a secret to a server
    pub fn link_secret(&self, link: &ServerSecret) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO server_secrets (server_id, secret_id, priority) VALUES (?1, ?2, ?3)",
            params![
                link.server_id.to_string(),
                link.secret_id.to_string(),
                link.priority,
            ],
        )
        .map_err(|e| Error::Database(e.to_string()))?;
        Ok(())
    }

    /// Unlink a secret from a server
    pub fn unlink_secret(&self, server_id: Uuid, secret_id: Uuid) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "DELETE FROM server_secrets WHERE server_id = ?1 AND secret_id = ?2",
            params![server_id.to_string(), secret_id.to_string()],
        )
        .map_err(|e| Error::Database(e.to_string()))?;
        Ok(())
    }

    /// Get secrets linked to a server, ordered by priority
    pub fn get_secrets(&self, server_id: Uuid) -> Result<Vec<Secret>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare(
                "SELECT s.id, s.name, s.kind, s.encrypted_value, s.created_at, s.updated_at
                 FROM secrets s
                 JOIN server_secrets ss ON s.id = ss.secret_id
                 WHERE ss.server_id = ?1
                 ORDER BY ss.priority ASC",
            )
            .map_err(|e| Error::Database(e.to_string()))?;

        let secrets = stmt
            .query_map([server_id.to_string()], |row| SecretRepository::row_to_secret(row))
            .map_err(|e| Error::Database(e.to_string()))?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| Error::Database(e.to_string()))?;

        Ok(secrets)
    }

    fn row_to_server(row: &Row) -> rusqlite::Result<Server> {
        let id_str: String = row.get(0)?;
        let protocol_str: String = row.get(5)?;
        let tags_str: String = row.get(6)?;
        let created_str: String = row.get(7)?;
        let updated_str: String = row.get(8)?;

        Ok(Server {
            id: Uuid::parse_str(&id_str).unwrap(),
            name: row.get(1)?,
            host: row.get(2)?,
            port: row.get(3)?,
            username: row.get(4)?,
            protocol: match protocol_str.as_str() {
                "ssh" => Protocol::Ssh,
                "sftp" => Protocol::Sftp,
                "ftp" => Protocol::Ftp,
                _ => Protocol::Ssh,
            },
            tags: serde_json::from_str(&tags_str).unwrap_or_default(),
            created_at: chrono::DateTime::parse_from_rfc3339(&created_str)
                .unwrap()
                .with_timezone(&chrono::Utc),
            updated_at: chrono::DateTime::parse_from_rfc3339(&updated_str)
                .unwrap()
                .with_timezone(&chrono::Utc),
        })
    }
}

// Add optional() method to rusqlite Result
trait OptionalExt<T> {
    fn optional(self) -> std::result::Result<Option<T>, rusqlite::Error>;
}

impl<T> OptionalExt<T> for std::result::Result<T, rusqlite::Error> {
    fn optional(self) -> std::result::Result<Option<T>, rusqlite::Error> {
        match self {
            Ok(value) => Ok(Some(value)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Database;

    #[test]
    fn test_server_crud() {
        let db = Database::in_memory().unwrap();
        let repo = db.servers();

        let server = Server::new(
            "Test Server".to_string(),
            "192.168.1.1".to_string(),
            22,
            "admin".to_string(),
        );

        // Create
        repo.store(&server).unwrap();

        // Read
        let retrieved = repo.get(server.id).unwrap().unwrap();
        assert_eq!(retrieved.name, "Test Server");

        // List
        let all = repo.list().unwrap();
        assert_eq!(all.len(), 1);

        // Delete
        repo.delete(server.id).unwrap();
        assert!(repo.get(server.id).unwrap().is_none());
    }

    #[test]
    fn test_server_secret_linking() {
        let db = Database::in_memory().unwrap();

        let secret = Secret::new(
            "My Password".to_string(),
            SecretKind::Password,
            b"encrypted_data".to_vec(),
        );
        db.secrets().store(&secret).unwrap();

        let server = Server::new(
            "Test Server".to_string(),
            "192.168.1.1".to_string(),
            22,
            "admin".to_string(),
        );
        db.servers().store(&server).unwrap();

        // Link
        let link = ServerSecret::new(server.id, secret.id, 0);
        db.servers().link_secret(&link).unwrap();

        // Get linked secrets
        let secrets = db.servers().get_secrets(server.id).unwrap();
        assert_eq!(secrets.len(), 1);
        assert_eq!(secrets[0].name, "My Password");

        // Unlink
        db.servers().unlink_secret(server.id, secret.id).unwrap();
        let secrets = db.servers().get_secrets(server.id).unwrap();
        assert!(secrets.is_empty());
    }
}

