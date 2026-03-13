use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;
use zeroize::Zeroizing;

use crate::crypto::cipher::{self, CipherError, EncryptedEnvelope, NONCE_LEN, TAG_LEN};

pub const SCHEMA_VERSION: &str = "1";
pub const TACOSHELL_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Error)]
pub enum VaultError {
    #[error("cipher error: {0}")]
    Cipher(#[from] CipherError),
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("base64 decode error: {0}")]
    Base64(#[from] base64::DecodeError),
    #[error("nonce has wrong length (expected {NONCE_LEN} bytes)")]
    InvalidNonceLength,
    #[error("tag has wrong length (expected {TAG_LEN} bytes)")]
    InvalidTagLength,
}

/// A single encrypted vault item as stored in vault.json.
///
/// All sensitive fields are base64-encoded binary. Plaintext never appears here.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedItem {
    pub id: String,
    /// Base64-encoded 12-byte random nonce.
    pub nonce: String,
    /// Base64-encoded AES-GCM ciphertext.
    pub ciphertext: String,
    /// Base64-encoded 16-byte GCM authentication tag.
    pub tag: String,
    pub version: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl EncryptedItem {
    /// Encrypts `plaintext` and wraps it in a new `EncryptedItem` with a fresh UUID.
    pub fn encrypt(key: &[u8; 32], plaintext: &[u8]) -> Result<Self, VaultError> {
        let id = Uuid::new_v4().to_string();
        Self::encrypt_with_id(key, &id, plaintext)
    }

    /// Encrypts `plaintext` using the given `id`.
    ///
    /// Use this when re-encrypting an existing item so that the ID remains stable
    /// and the `created_at` timestamp can be preserved by the caller.
    pub fn encrypt_with_id(key: &[u8; 32], id: &str, plaintext: &[u8]) -> Result<Self, VaultError> {
        let aad = make_aad(id, SCHEMA_VERSION);
        let envelope = cipher::encrypt(key, plaintext, &aad)?;
        let now = Utc::now();

        Ok(EncryptedItem {
            id: id.to_string(),
            nonce: BASE64.encode(envelope.nonce),
            ciphertext: BASE64.encode(&envelope.ciphertext),
            tag: BASE64.encode(envelope.tag),
            version: SCHEMA_VERSION.to_string(),
            created_at: now,
            updated_at: now,
        })
    }

    /// Decrypts this item and returns the plaintext in a `Zeroizing` wrapper.
    pub fn decrypt(&self, key: &[u8; 32]) -> Result<Zeroizing<Vec<u8>>, VaultError> {
        let nonce_bytes = BASE64.decode(&self.nonce)?;
        let ciphertext = BASE64.decode(&self.ciphertext)?;
        let tag_bytes = BASE64.decode(&self.tag)?;

        let nonce: [u8; NONCE_LEN] = nonce_bytes
            .try_into()
            .map_err(|_| VaultError::InvalidNonceLength)?;

        let tag: [u8; TAG_LEN] = tag_bytes
            .try_into()
            .map_err(|_| VaultError::InvalidTagLength)?;

        let envelope = EncryptedEnvelope {
            nonce,
            ciphertext,
            tag,
        };

        // AAD must match what was used during encryption.
        let aad = make_aad(&self.id, &self.version);
        Ok(cipher::decrypt(key, &envelope, &aad)?)
    }
}

/// The `vault.json` file stored in the private GitHub repository.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultFile {
    pub schema_version: String,
    pub items: Vec<EncryptedItem>,
}

impl VaultFile {
    pub fn new() -> Self {
        VaultFile {
            schema_version: SCHEMA_VERSION.to_string(),
            items: Vec::new(),
        }
    }

    pub fn to_json(&self) -> Result<String, VaultError> {
        Ok(serde_json::to_string_pretty(self)?)
    }

    pub fn from_json(s: &str) -> Result<Self, VaultError> {
        Ok(serde_json::from_str(s)?)
    }

    pub fn add_item(&mut self, item: EncryptedItem) {
        self.items.push(item);
    }

    /// Removes the item with the given ID. Returns `true` if an item was removed.
    pub fn remove_item(&mut self, id: &str) -> bool {
        let before = self.items.len();
        self.items.retain(|i| i.id != id);
        self.items.len() < before
    }

    pub fn get_item(&self, id: &str) -> Option<&EncryptedItem> {
        self.items.iter().find(|i| i.id == id)
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

impl Default for VaultFile {
    fn default() -> Self {
        Self::new()
    }
}

/// The `meta.json` file (unencrypted) stored alongside `vault.json`.
///
/// Contains only non-sensitive metadata. No vault contents appear here.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaFile {
    pub schema_version: String,
    pub created_at: DateTime<Utc>,
    pub last_sync: DateTime<Utc>,
    pub tacoshell_version: String,
}

impl MetaFile {
    pub fn new(tacoshell_version: &str) -> Self {
        let now = Utc::now();
        MetaFile {
            schema_version: SCHEMA_VERSION.to_string(),
            created_at: now,
            last_sync: now,
            tacoshell_version: tacoshell_version.to_string(),
        }
    }

    pub fn to_json(&self) -> Result<String, VaultError> {
        Ok(serde_json::to_string_pretty(self)?)
    }

    pub fn from_json(s: &str) -> Result<Self, VaultError> {
        Ok(serde_json::from_str(s)?)
    }
}

/// Constructs the Additional Authenticated Data (AAD) for a vault item.
///
/// AAD = item_id bytes || schema_version bytes.
/// This binds the ciphertext to a specific item ID and schema version,
/// preventing an attacker from transplanting ciphertext between items.
fn make_aad(item_id: &str, schema_version: &str) -> Vec<u8> {
    let mut aad = item_id.as_bytes().to_vec();
    aad.extend_from_slice(schema_version.as_bytes());
    aad
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    const TEST_KEY: [u8; 32] = [0x55u8; 32];

    // --- EncryptedItem tests ---

    #[test]
    fn encrypted_item_round_trips() {
        let plaintext = b"ssh://user@host:22";
        let item = EncryptedItem::encrypt(&TEST_KEY, plaintext).unwrap();
        let decrypted = item.decrypt(&TEST_KEY).unwrap();
        assert_eq!(decrypted.as_slice(), plaintext);
    }

    #[test]
    fn encrypt_with_same_plaintext_produces_different_ciphertext() {
        let a = EncryptedItem::encrypt(&TEST_KEY, b"data").unwrap();
        let b = EncryptedItem::encrypt(&TEST_KEY, b"data").unwrap();
        // Different IDs → different AAD, plus random nonce, so both nonce and ciphertext differ.
        assert_ne!(a.nonce, b.nonce);
        assert_ne!(a.id, b.id);
    }

    #[test]
    fn encrypted_item_fields_are_valid_base64() {
        let item = EncryptedItem::encrypt(&TEST_KEY, b"payload").unwrap();
        assert!(BASE64.decode(&item.nonce).is_ok());
        assert!(BASE64.decode(&item.ciphertext).is_ok());
        assert!(BASE64.decode(&item.tag).is_ok());
    }

    #[test]
    fn decrypt_fails_with_wrong_key() {
        let item = EncryptedItem::encrypt(&TEST_KEY, b"secret").unwrap();
        let wrong_key = [0x01u8; 32];
        assert!(item.decrypt(&wrong_key).is_err());
    }

    #[test]
    fn encrypt_with_id_preserves_id() {
        let item = EncryptedItem::encrypt_with_id(&TEST_KEY, "my-stable-id", b"data").unwrap();
        assert_eq!(item.id, "my-stable-id");
    }

    #[test]
    fn decrypt_fails_when_id_is_tampered_after_encryption() {
        let mut item = EncryptedItem::encrypt_with_id(&TEST_KEY, "original-id", b"data").unwrap();
        item.id = "tampered-id".to_string(); // AAD mismatch on decrypt
        assert!(item.decrypt(&TEST_KEY).is_err());
    }

    // --- VaultFile tests ---

    #[test]
    fn vault_file_new_is_empty() {
        let vault = VaultFile::new();
        assert!(vault.is_empty());
        assert_eq!(vault.schema_version, SCHEMA_VERSION);
    }

    #[test]
    fn vault_file_default_equals_new() {
        let vault: VaultFile = Default::default();
        assert!(vault.is_empty());
        assert_eq!(vault.schema_version, SCHEMA_VERSION);
    }

    #[test]
    fn vault_file_add_then_get_item() {
        let mut vault = VaultFile::new();
        let item = EncryptedItem::encrypt(&TEST_KEY, b"data").unwrap();
        let id = item.id.clone();
        vault.add_item(item);
        assert!(vault.get_item(&id).is_some());
    }

    #[test]
    fn vault_file_remove_item_returns_true_when_found() {
        let mut vault = VaultFile::new();
        let item = EncryptedItem::encrypt(&TEST_KEY, b"data").unwrap();
        let id = item.id.clone();
        vault.add_item(item);
        assert!(vault.remove_item(&id));
        assert!(vault.get_item(&id).is_none());
    }

    #[test]
    fn vault_file_remove_item_returns_false_when_not_found() {
        let mut vault = VaultFile::new();
        assert!(!vault.remove_item("does-not-exist"));
    }

    #[test]
    fn vault_file_json_round_trip() {
        let mut vault = VaultFile::new();
        vault.add_item(EncryptedItem::encrypt(&TEST_KEY, b"item1").unwrap());
        vault.add_item(EncryptedItem::encrypt(&TEST_KEY, b"item2").unwrap());

        let json = vault.to_json().unwrap();
        let restored = VaultFile::from_json(&json).unwrap();

        assert_eq!(restored.items.len(), 2);
        assert_eq!(restored.schema_version, SCHEMA_VERSION);
    }

    #[test]
    fn vault_file_round_trip_decrypts_correctly() {
        let plaintext = b"my ssh password";
        let mut vault = VaultFile::new();
        let item = EncryptedItem::encrypt(&TEST_KEY, plaintext).unwrap();
        let id = item.id.clone();
        vault.add_item(item);

        let json = vault.to_json().unwrap();
        let restored = VaultFile::from_json(&json).unwrap();

        let decrypted = restored.get_item(&id).unwrap().decrypt(&TEST_KEY).unwrap();
        assert_eq!(decrypted.as_slice(), plaintext);
    }

    #[test]
    fn vault_file_json_is_valid_json() {
        let vault = VaultFile::new();
        let json = vault.to_json().unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["schema_version"], SCHEMA_VERSION);
    }

    // --- MetaFile tests ---

    #[test]
    fn meta_file_new_sets_schema_version() {
        let meta = MetaFile::new("0.1.0");
        assert_eq!(meta.schema_version, SCHEMA_VERSION);
        assert_eq!(meta.tacoshell_version, "0.1.0");
    }

    #[test]
    fn meta_file_json_round_trip() {
        let meta = MetaFile::new("0.1.0");
        let json = meta.to_json().unwrap();
        let restored = MetaFile::from_json(&json).unwrap();
        assert_eq!(restored.schema_version, meta.schema_version);
        assert_eq!(restored.tacoshell_version, meta.tacoshell_version);
    }

    #[test]
    fn meta_file_from_json_parses_example_from_spec() {
        let json = r#"{
            "schema_version": "1",
            "created_at": "2026-03-13T00:00:00Z",
            "last_sync": "2026-03-13T00:00:00Z",
            "tacoshell_version": "0.1.0"
        }"#;
        let meta = MetaFile::from_json(json).unwrap();
        assert_eq!(meta.schema_version, "1");
    }
}
