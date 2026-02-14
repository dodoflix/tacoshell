//! Repository implementations for storage access

use crate::Storage;
use tacoshell_core::{Result, Secret, Server, ServerSecret};
use uuid::Uuid;

#[cfg(test)]
use tacoshell_core::SecretKind;

/// Repository for secret operations
pub struct SecretRepository {
    storage: Storage,
}

impl SecretRepository {
    pub fn new(storage: Storage) -> Self {
        Self { storage }
    }

    /// Store a new secret
    pub fn store(&self, secret: &Secret) -> Result<()> {
        {
            let mut data = self.storage.data.write().unwrap();
            data.secrets.push(secret.clone());
        }
        self.storage.save()
    }

    /// Get a secret by ID
    pub fn get(&self, id: Uuid) -> Result<Option<Secret>> {
        let data = self.storage.data.read().unwrap();
        Ok(data.secrets.iter().find(|s| s.id == id).cloned())
    }

    /// List all secrets
    pub fn list(&self) -> Result<Vec<Secret>> {
        let data = self.storage.data.read().unwrap();
        let mut secrets = data.secrets.clone();
        secrets.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(secrets)
    }

    /// Delete a secret by ID
    pub fn delete(&self, id: Uuid) -> Result<()> {
        {
            let mut data = self.storage.data.write().unwrap();
            data.secrets.retain(|s| s.id != id);
            // Also remove links
            data.server_secrets.retain(|ss| ss.secret_id != id);
        }
        self.storage.save()
    }

    /// Update an existing secret
    pub fn update(&self, secret: &Secret) -> Result<()> {
        {
            let mut data = self.storage.data.write().unwrap();
            if let Some(s) = data.secrets.iter_mut().find(|s| s.id == secret.id) {
                *s = secret.clone();
                s.updated_at = chrono::Utc::now();
            }
        }
        self.storage.save()
    }
}

/// Repository for server operations
pub struct ServerRepository {
    storage: Storage,
}

impl ServerRepository {
    pub fn new(storage: Storage) -> Self {
        Self { storage }
    }

    /// Store a new server
    pub fn store(&self, server: &Server) -> Result<()> {
        {
            let mut data = self.storage.data.write().unwrap();
            data.servers.push(server.clone());
        }
        self.storage.save()
    }

    /// Get a server by ID
    pub fn get(&self, id: Uuid) -> Result<Option<Server>> {
        let data = self.storage.data.read().unwrap();
        Ok(data.servers.iter().find(|s| s.id == id).cloned())
    }

    /// List all servers
    pub fn list(&self) -> Result<Vec<Server>> {
        let data = self.storage.data.read().unwrap();
        let mut servers = data.servers.clone();
        servers.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(servers)
    }

    /// Delete a server by ID
    pub fn delete(&self, id: Uuid) -> Result<()> {
        {
            let mut data = self.storage.data.write().unwrap();
            data.servers.retain(|s| s.id != id);
            // Also remove links
            data.server_secrets.retain(|ss| ss.server_id != id);
        }
        self.storage.save()
    }

    /// Update an existing server
    pub fn update(&self, server: &Server) -> Result<()> {
        {
            let mut data = self.storage.data.write().unwrap();
            if let Some(s) = data.servers.iter_mut().find(|s| s.id == server.id) {
                *s = server.clone();
                s.updated_at = chrono::Utc::now();
            }
        }
        self.storage.save()
    }

    /// Link a secret to a server
    pub fn link_secret(&self, link: &ServerSecret) -> Result<()> {
        {
            let mut data = self.storage.data.write().unwrap();
            // Remove existing link if any
            data.server_secrets.retain(|ss| ss.server_id != link.server_id || ss.secret_id != link.secret_id);
            data.server_secrets.push(link.clone());
        }
        self.storage.save()
    }

    /// Unlink a secret from a server
    pub fn unlink_secret(&self, server_id: Uuid, secret_id: Uuid) -> Result<()> {
        {
            let mut data = self.storage.data.write().unwrap();
            data.server_secrets.retain(|ss| ss.server_id != server_id || ss.secret_id != secret_id);
        }
        self.storage.save()
    }

    /// Get secrets linked to a server, ordered by priority
    pub fn get_secrets(&self, server_id: Uuid) -> Result<Vec<Secret>> {
        let data = self.storage.data.read().unwrap();
        
        let mut linked_secrets: Vec<(i32, Uuid)> = data.server_secrets.iter()
            .filter(|ss| ss.server_id == server_id)
            .map(|ss| (ss.priority, ss.secret_id))
            .collect();
            
        linked_secrets.sort_by_key(|(priority, _)| *priority);
        
        let mut results = Vec::new();
        for (_, secret_id) in linked_secrets {
            if let Some(secret) = data.secrets.iter().find(|s| s.id == secret_id) {
                results.push(secret.clone());
            }
        }
        
        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Storage;

    #[test]
    fn test_server_crud() {
        let storage = Storage::in_memory();
        let repo = storage.servers();

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
        let storage = Storage::in_memory();

        let secret = Secret::new(
            "My Password".to_string(),
            SecretKind::Password,
            b"encrypted_data".to_vec(),
            Some("admin".to_string()),
        );
        storage.secrets().store(&secret).unwrap();

        let server = Server::new(
            "Test Server".to_string(),
            "192.168.1.1".to_string(),
            22,
            "admin".to_string(),
        );
        storage.servers().store(&server).unwrap();

        // Link
        let link = ServerSecret::new(server.id, secret.id, 0);
        storage.servers().link_secret(&link).unwrap();

        // Get linked secrets
        let secrets = storage.servers().get_secrets(server.id).unwrap();
        assert_eq!(secrets.len(), 1);
        assert_eq!(secrets[0].name, "My Password");

        // Unlink
        storage.servers().unlink_secret(server.id, secret.id).unwrap();
        let secrets = storage.servers().get_secrets(server.id).unwrap();
        assert!(secrets.is_empty());
    }
}

