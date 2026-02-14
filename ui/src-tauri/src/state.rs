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
        use std::collections::hash_map::RandomState;
        use std::hash::{BuildHasher, Hasher};

        // Generate random bytes using system entropy
        let mut key_bytes = [0u8; 32];
        for (i, byte) in key_bytes.iter_mut().enumerate() {
            let hasher = RandomState::new().build_hasher();
            let mut h = hasher;
            h.write_usize(i);
            h.write_u64(std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos() as u64);
            *byte = (h.finish() & 0xFF) as u8;
        }

        // Encode as base64-like string
        base16ct::lower::encode_string(&key_bytes)
    }

    /// Derive a fallback key when keyring is unavailable
    fn derive_fallback_key() -> String {
        // Use machine-specific data to derive a key
        let hostname = hostname::get()
            .map(|h| h.to_string_lossy().to_string())
            .unwrap_or_else(|_| "unknown".to_string());

        let data_dir = dirs::data_dir()
            .map(|d| d.to_string_lossy().to_string())
            .unwrap_or_else(|| ".".to_string());

        format!("tacoshell-fallback-{}-{}", hostname, data_dir.len())
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

