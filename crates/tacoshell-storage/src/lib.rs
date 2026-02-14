//! # Tacoshell Storage
//!
//! JSON-based storage layer for storing servers, secrets, and their relationships.

pub mod repository;

use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use std::fs;
use serde::{Deserialize, Serialize};
use tacoshell_core::{Result, Error, Secret, Server, ServerSecret};

pub use repository::{SecretRepository, ServerRepository};

/// The root data structure stored in JSON
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct StorageData {
    pub servers: Vec<Server>,
    pub secrets: Vec<Secret>,
    pub server_secrets: Vec<ServerSecret>,
}

/// JSON storage handler
#[derive(Clone)]
pub struct Storage {
    path: PathBuf,
    data: Arc<RwLock<StorageData>>,
}

impl Storage {
    /// Open or create a storage at the given path
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        let data = if path.exists() {
            let content = fs::read_to_string(&path)
                .map_err(|e| Error::Database(format!("Failed to read storage file: {}", e)))?;
            serde_json::from_str(&content)
                .map_err(|e| Error::Database(format!("Failed to parse storage file: {}", e)))?
        } else {
            StorageData::default()
        };

        Ok(Self {
            path,
            data: Arc::new(RwLock::new(data)),
        })
    }

    /// Create an in-memory storage (useful for testing)
    pub fn in_memory() -> Self {
        Self {
            path: PathBuf::from(":memory:"),
            data: Arc::new(RwLock::new(StorageData::default())),
        }
    }

    /// Save the current data to the JSON file
    pub fn save(&self) -> Result<()> {
        if self.path == PathBuf::from(":memory:") {
            return Ok(());
        }

        let data = self.data.read().unwrap();
        let content = serde_json::to_string_pretty(&*data)
            .map_err(|e| Error::Database(format!("Failed to serialize storage: {}", e)))?;
        
        // Use a temporary file for atomic write
        let tmp_path = self.path.with_extension("tmp");
        fs::write(&tmp_path, content)
            .map_err(|e| Error::Database(format!("Failed to write to temp file: {}", e)))?;
        
        fs::rename(tmp_path, &self.path)
            .map_err(|e| Error::Database(format!("Failed to replace storage file: {}", e)))?;

        Ok(())
    }

    /// Get a secret repository
    pub fn secrets(&self) -> SecretRepository {
        SecretRepository::new(self.clone())
    }

    /// Get a server repository
    pub fn servers(&self) -> ServerRepository {
        ServerRepository::new(self.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_in_memory_storage() {
        let _storage = Storage::in_memory();
    }
}
