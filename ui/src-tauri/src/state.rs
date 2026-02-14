//! Application state management

use std::collections::HashMap;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};
use tacoshell_core::Result;
use tacoshell_db::Database;
use tacoshell_secrets::SecretEncryption;
use tacoshell_ssh::{SshChannel, SshSession};
use uuid::Uuid;

const KEYRING_SERVICE: &str = "tacoshell";
const KEYRING_USER: &str = "master-key";

/// Active SSH session with its channel
pub struct ActiveSession {
    pub session: SshSession,
    pub channel: Option<SshChannel>,
    /// Flag to stop the background reader thread
    pub stop_flag: Arc<AtomicBool>,
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

        // Get or create master key from OS keyring
        let master_key = Self::get_or_create_master_key()?;
        let encryption = SecretEncryption::new(master_key);

        Ok(Self {
            db,
            encryption,
            sessions: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    /// Get or create a master key using the OS keyring
    fn get_or_create_master_key() -> Result<String> {
        let entry = keyring::Entry::new(KEYRING_SERVICE, KEYRING_USER)
            .map_err(|e| tacoshell_core::Error::Config(format!("Failed to access keyring: {}", e)))?;

        // Try to get existing key
        match entry.get_password() {
            Ok(key) => {
                tracing::info!("Retrieved master key from OS keyring");
                Ok(key)
            }
            Err(keyring::Error::NoEntry) => {
                // Generate new master key
                let new_key = Self::generate_master_key();
                tracing::info!("Generated new master key, storing in OS keyring");

                entry.set_password(&new_key)
                    .map_err(|e| tacoshell_core::Error::Config(format!("Failed to store master key: {}", e)))?;

                Ok(new_key)
            }
            Err(e) => {
                // Fallback to a derived key if keyring is unavailable
                tracing::warn!("Keyring unavailable ({}), using fallback key derivation", e);
                Ok(Self::derive_fallback_key())
            }
        }
    }

    /// Generate a cryptographically secure random master key
    fn generate_master_key() -> String {
        use rand::RngCore;

        // Generate 32 cryptographically secure random bytes using OS entropy
        let mut key_bytes = [0u8; 32];
        rand::rngs::OsRng.fill_bytes(&mut key_bytes);

        // Encode as hex string (64 characters)
        base16ct::lower::encode_string(&key_bytes)
    }

    /// Derive a fallback key when keyring is unavailable
    ///
    /// WARNING: This fallback is less secure than the OS keyring.
    /// The key is derived from machine-specific data but should be considered
    /// a temporary measure. Users should resolve keyring access issues.
    fn derive_fallback_key() -> String {
        use rand::RngCore;
        use std::io::Write;

        // Try to use a file-based key as fallback
        let data_dir = dirs::data_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join("tacoshell");
        let key_file = data_dir.join(".master_key");

        // Try to read existing fallback key
        if let Ok(existing_key) = std::fs::read_to_string(&key_file) {
            let key = existing_key.trim().to_string();
            if key.len() == 64 {
                tracing::warn!("Using file-based fallback key - OS keyring unavailable");
                return key;
            }
        }

        // Generate new fallback key
        let mut key_bytes = [0u8; 32];
        rand::rngs::OsRng.fill_bytes(&mut key_bytes);
        let key = base16ct::lower::encode_string(&key_bytes);

        // Try to save it (ignore errors - we'll regenerate next time)
        if let Ok(mut file) = std::fs::File::create(&key_file) {
            let _ = file.write_all(key.as_bytes());
            tracing::warn!("Created file-based fallback key at {:?} - OS keyring unavailable", key_file);
        }

        key
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

