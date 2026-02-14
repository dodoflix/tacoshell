//! Application state management

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tacoshell_core::Result;
use tacoshell_db::Database;
use tacoshell_secrets::SecretEncryption;
use tacoshell_ssh::{SshChannel, SshSession};
use uuid::Uuid;

/// Active SSH session with its channel
pub struct ActiveSession {
    pub session: SshSession,
    pub channel: Option<SshChannel>,
}

/// Application state shared across all Tauri commands
pub struct AppState {
    /// Database connection
    pub db: Database,
    /// Secret encryption handler
    pub encryption: SecretEncryption,
    /// Active SSH sessions by session ID
    pub sessions: Arc<Mutex<HashMap<Uuid, ActiveSession>>>,
}

impl AppState {
    /// Create a new application state
    pub fn new() -> Result<Self> {
        // Get data directory
        let data_dir = dirs::data_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join("tacoshell");

        std::fs::create_dir_all(&data_dir)
            .map_err(|e| tacoshell_core::Error::Config(format!("Failed to create data dir: {}", e)))?;

        let db_path = data_dir.join("tacoshell.db");
        tracing::info!("Opening database at {:?}", db_path);

        let db = Database::open(&db_path)?;

        // TODO: Get passphrase from user or keyring
        // For now, use a default passphrase (this should be changed in production)
        let encryption = SecretEncryption::new("tacoshell-default-key");

        Ok(Self {
            db,
            encryption,
            sessions: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    /// Add an active session
    pub fn add_session(&self, id: Uuid, session: ActiveSession) {
        let mut sessions = self.sessions.lock().unwrap();
        sessions.insert(id, session);
    }

    /// Remove an active session
    pub fn remove_session(&self, id: &Uuid) -> Option<ActiveSession> {
        let mut sessions = self.sessions.lock().unwrap();
        sessions.remove(id)
    }

    /// Get mutable access to a session
    pub fn with_session<F, R>(&self, id: &Uuid, f: F) -> Option<R>
    where
        F: FnOnce(&mut ActiveSession) -> R,
    {
        let mut sessions = self.sessions.lock().unwrap();
        sessions.get_mut(id).map(f)
    }
}

