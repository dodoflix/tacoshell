use thiserror::Error;
use zeroize::Zeroizing;

use crate::crypto::vault::{EncryptedItem, VaultError, VaultFile};
use crate::profile::types::{
    ConnectionProfile, KubeConfigItem, Password, ProfileId, SshKey, VaultPayload,
};

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

#[derive(Debug, Error)]
pub enum ProfileError {
    #[error("profile not found: {id}")]
    NotFound { id: String },

    #[error("type mismatch: expected {expected}, found {found}")]
    WrongType {
        expected: &'static str,
        found: &'static str,
    },

    #[error("vault error: {0}")]
    Vault(#[from] VaultError),

    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("UTF-8 error in vault payload: {0}")]
    Utf8(#[from] std::str::Utf8Error),
}

// ---------------------------------------------------------------------------
// ProfileManager
// ---------------------------------------------------------------------------

/// CRUD interface for typed profile and credential items in the vault.
///
/// `ProfileManager` owns a `VaultFile` and the AES-256-GCM master key. Every
/// item is individually encrypted with the master key before being stored in
/// the vault, and decrypted on read. The manager tracks in-memory state only —
/// callers are responsible for persisting the vault via `SyncEngine`.
pub struct ProfileManager {
    vault: VaultFile,
    /// Master key — zeroized when this struct is dropped.
    master_key: Zeroizing<[u8; 32]>,
}

impl ProfileManager {
    /// Create a manager wrapping an existing (possibly empty) vault.
    pub fn new(vault: VaultFile, master_key: [u8; 32]) -> Self {
        ProfileManager {
            vault,
            master_key: Zeroizing::new(master_key),
        }
    }

    /// Borrow the underlying `VaultFile` for persistence (e.g., via `SyncEngine`).
    pub fn vault(&self) -> &VaultFile {
        &self.vault
    }

    // -----------------------------------------------------------------------
    // Generic CRUD
    // -----------------------------------------------------------------------

    /// Encrypt `payload` and add it to the vault.
    ///
    /// Returns the item ID (taken from the payload itself, not generated here).
    pub fn add(&mut self, payload: VaultPayload) -> Result<ProfileId, ProfileError> {
        let id = payload.id().to_string();
        let json = serde_json::to_vec(&payload)?;
        let item = EncryptedItem::encrypt_with_id(&self.master_key, &id, &json)?;
        self.vault.add_item(item);
        Ok(id)
    }

    /// Decrypt and deserialize the vault item with the given ID.
    pub fn get(&self, id: &str) -> Result<VaultPayload, ProfileError> {
        let item = self
            .vault
            .get_item(id)
            .ok_or_else(|| ProfileError::NotFound { id: id.to_string() })?;
        self.decrypt_item(item)
    }

    /// Update an existing vault item in-place, preserving its `created_at` timestamp.
    ///
    /// The `payload`'s `id()` must match an existing item. Returns `NotFound`
    /// if no item with that ID exists.
    pub fn update(&mut self, payload: VaultPayload) -> Result<(), ProfileError> {
        let id = payload.id().to_string();

        // Preserve the original creation time before removing.
        let original_created_at = self
            .vault
            .get_item(&id)
            .ok_or_else(|| ProfileError::NotFound { id: id.clone() })?
            .created_at;

        self.vault.remove_item(&id);

        let json = serde_json::to_vec(&payload)?;
        let mut item = EncryptedItem::encrypt_with_id(&self.master_key, &id, &json)?;
        item.created_at = original_created_at;

        self.vault.add_item(item);
        Ok(())
    }

    /// Remove the item with the given ID.
    ///
    /// Returns `true` if an item was removed, `false` if it didn't exist.
    pub fn delete(&mut self, id: &str) -> bool {
        self.vault.remove_item(id)
    }

    /// Decrypt and return all vault items.
    pub fn list_all(&self) -> Result<Vec<VaultPayload>, ProfileError> {
        self.vault
            .items
            .iter()
            .map(|item| self.decrypt_item(item))
            .collect()
    }

    // -----------------------------------------------------------------------
    // Typed helpers — ConnectionProfile
    // -----------------------------------------------------------------------

    pub fn add_profile(&mut self, profile: ConnectionProfile) -> Result<ProfileId, ProfileError> {
        self.add(VaultPayload::ConnectionProfile(profile))
    }

    pub fn get_profile(&self, id: &str) -> Result<ConnectionProfile, ProfileError> {
        match self.get(id)? {
            VaultPayload::ConnectionProfile(p) => Ok(p),
            other => Err(ProfileError::WrongType {
                expected: "connection_profile",
                found: other.type_name(),
            }),
        }
    }

    pub fn update_profile(&mut self, profile: ConnectionProfile) -> Result<(), ProfileError> {
        self.update(VaultPayload::ConnectionProfile(profile))
    }

    pub fn list_profiles(&self) -> Result<Vec<ConnectionProfile>, ProfileError> {
        self.list_all().map(|items| {
            items
                .into_iter()
                .filter_map(|p| match p {
                    VaultPayload::ConnectionProfile(profile) => Some(profile),
                    _ => None,
                })
                .collect()
        })
    }

    // -----------------------------------------------------------------------
    // Typed helpers — SshKey
    // -----------------------------------------------------------------------

    pub fn add_ssh_key(&mut self, key: SshKey) -> Result<ProfileId, ProfileError> {
        self.add(VaultPayload::SshKey(key))
    }

    pub fn get_ssh_key(&self, id: &str) -> Result<SshKey, ProfileError> {
        match self.get(id)? {
            VaultPayload::SshKey(k) => Ok(k),
            other => Err(ProfileError::WrongType {
                expected: "ssh_key",
                found: other.type_name(),
            }),
        }
    }

    pub fn update_ssh_key(&mut self, key: SshKey) -> Result<(), ProfileError> {
        self.update(VaultPayload::SshKey(key))
    }

    pub fn list_ssh_keys(&self) -> Result<Vec<SshKey>, ProfileError> {
        self.list_all().map(|items| {
            items
                .into_iter()
                .filter_map(|p| match p {
                    VaultPayload::SshKey(k) => Some(k),
                    _ => None,
                })
                .collect()
        })
    }

    // -----------------------------------------------------------------------
    // Typed helpers — Password
    // -----------------------------------------------------------------------

    pub fn add_password(&mut self, password: Password) -> Result<ProfileId, ProfileError> {
        self.add(VaultPayload::Password(password))
    }

    pub fn get_password(&self, id: &str) -> Result<Password, ProfileError> {
        match self.get(id)? {
            VaultPayload::Password(p) => Ok(p),
            other => Err(ProfileError::WrongType {
                expected: "password",
                found: other.type_name(),
            }),
        }
    }

    pub fn update_password(&mut self, password: Password) -> Result<(), ProfileError> {
        self.update(VaultPayload::Password(password))
    }

    pub fn list_passwords(&self) -> Result<Vec<Password>, ProfileError> {
        self.list_all().map(|items| {
            items
                .into_iter()
                .filter_map(|p| match p {
                    VaultPayload::Password(pw) => Some(pw),
                    _ => None,
                })
                .collect()
        })
    }

    // -----------------------------------------------------------------------
    // Typed helpers — KubeConfigItem
    // -----------------------------------------------------------------------

    pub fn add_kube_config(&mut self, kube: KubeConfigItem) -> Result<ProfileId, ProfileError> {
        self.add(VaultPayload::KubeConfig(kube))
    }

    pub fn get_kube_config(&self, id: &str) -> Result<KubeConfigItem, ProfileError> {
        match self.get(id)? {
            VaultPayload::KubeConfig(k) => Ok(k),
            other => Err(ProfileError::WrongType {
                expected: "kube_config",
                found: other.type_name(),
            }),
        }
    }

    pub fn update_kube_config(&mut self, kube: KubeConfigItem) -> Result<(), ProfileError> {
        self.update(VaultPayload::KubeConfig(kube))
    }

    pub fn list_kube_configs(&self) -> Result<Vec<KubeConfigItem>, ProfileError> {
        self.list_all().map(|items| {
            items
                .into_iter()
                .filter_map(|p| match p {
                    VaultPayload::KubeConfig(k) => Some(k),
                    _ => None,
                })
                .collect()
        })
    }

    // -----------------------------------------------------------------------
    // Private helpers
    // -----------------------------------------------------------------------

    fn decrypt_item(&self, item: &EncryptedItem) -> Result<VaultPayload, ProfileError> {
        let plaintext = item.decrypt(&self.master_key)?;
        let payload: VaultPayload = serde_json::from_slice(&plaintext)?;
        Ok(payload)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::profile::types::{KubeAuth, Protocol, SshKeyType};
    use pretty_assertions::assert_eq;

    const KEY: [u8; 32] = [0xABu8; 32];

    fn make_manager() -> ProfileManager {
        ProfileManager::new(VaultFile::new(), KEY)
    }

    // --- ConnectionProfile CRUD ---

    #[test]
    fn add_and_get_connection_profile() {
        let mut mgr = make_manager();
        let profile = ConnectionProfile::new_ssh("Dev Box", "dev.example.com", 22, "alice");
        let id = mgr.add_profile(profile.clone()).unwrap();
        let got = mgr.get_profile(&id).unwrap();
        assert_eq!(got, profile);
    }

    #[test]
    fn update_profile_changes_fields_and_preserves_created_at() {
        let mut mgr = make_manager();
        let mut profile = ConnectionProfile::new_ssh("Old Name", "host.example.com", 22, "alice");
        let id = mgr.add_profile(profile.clone()).unwrap();
        let original_created_at = profile.created_at;

        profile.display_name = "New Name".to_string();
        profile.port = 2222;
        profile.updated_at = chrono::Utc::now();
        mgr.update_profile(profile.clone()).unwrap();

        let got = mgr.get_profile(&id).unwrap();
        assert_eq!(got.display_name, "New Name");
        assert_eq!(got.port, 2222);
        // created_at must be preserved even though the item was re-encrypted.
        assert_eq!(got.created_at, original_created_at);
    }

    #[test]
    fn delete_profile_removes_it_from_vault() {
        let mut mgr = make_manager();
        let profile = ConnectionProfile::new_ssh("Temp", "host", 22, "user");
        let id = mgr.add_profile(profile).unwrap();

        assert!(mgr.delete(&id));
        assert!(matches!(
            mgr.get_profile(&id).unwrap_err(),
            ProfileError::NotFound { .. }
        ));
        // Second delete returns false.
        assert!(!mgr.delete(&id));
    }

    #[test]
    fn get_nonexistent_profile_returns_not_found() {
        let mgr = make_manager();
        let err = mgr.get_profile("no-such-id").unwrap_err();
        assert!(matches!(err, ProfileError::NotFound { .. }));
    }

    #[test]
    fn list_profiles_returns_only_profiles() {
        let mut mgr = make_manager();
        mgr.add_profile(ConnectionProfile::new_ssh("A", "a.com", 22, "u"))
            .unwrap();
        mgr.add_profile(ConnectionProfile::new_ssh("B", "b.com", 22, "u"))
            .unwrap();
        // Adding an SSH key should not appear in list_profiles.
        mgr.add_ssh_key(SshKey::new("Key", "pem", "pub", SshKeyType::Ed25519))
            .unwrap();

        let profiles = mgr.list_profiles().unwrap();
        assert_eq!(profiles.len(), 2);
        assert!(profiles.iter().all(|p| p.protocol == Protocol::Ssh));
    }

    // --- SshKey CRUD ---

    #[test]
    fn add_and_get_ssh_key() {
        let mut mgr = make_manager();
        let key = SshKey::new(
            "My Key",
            "-----BEGIN OPENSSH PRIVATE KEY-----\nfake\n",
            "ssh-ed25519 AAAA",
            SshKeyType::Ed25519,
        );
        let id = mgr.add_ssh_key(key.clone()).unwrap();
        let got = mgr.get_ssh_key(&id).unwrap();
        assert_eq!(got, key);
    }

    #[test]
    fn update_ssh_key_changes_display_name() {
        let mut mgr = make_manager();
        let mut key = SshKey::new("Old", "pem", "pub", SshKeyType::Rsa);
        let id = mgr.add_ssh_key(key.clone()).unwrap();
        key.display_name = "New".to_string();
        mgr.update_ssh_key(key.clone()).unwrap();
        assert_eq!(mgr.get_ssh_key(&id).unwrap().display_name, "New");
    }

    #[test]
    fn list_ssh_keys_returns_only_keys() {
        let mut mgr = make_manager();
        mgr.add_ssh_key(SshKey::new("K1", "pem", "pub", SshKeyType::Ed25519))
            .unwrap();
        mgr.add_profile(ConnectionProfile::new_ssh("P", "h", 22, "u"))
            .unwrap();
        assert_eq!(mgr.list_ssh_keys().unwrap().len(), 1);
    }

    // --- Password CRUD ---

    #[test]
    fn add_and_get_password() {
        let mut mgr = make_manager();
        let pw = Password::new("My VPN", "alice", "s3cr3t!");
        let id = mgr.add_password(pw.clone()).unwrap();
        let got = mgr.get_password(&id).unwrap();
        assert_eq!(got, pw);
        assert_eq!(got.password, "s3cr3t!");
    }

    #[test]
    fn update_password_changes_password_value() {
        let mut mgr = make_manager();
        let mut pw = Password::new("Cred", "bob", "old-pass");
        let id = mgr.add_password(pw.clone()).unwrap();
        pw.password = "new-pass".to_string();
        mgr.update_password(pw.clone()).unwrap();
        assert_eq!(mgr.get_password(&id).unwrap().password, "new-pass");
    }

    #[test]
    fn list_passwords_returns_only_passwords() {
        let mut mgr = make_manager();
        mgr.add_password(Password::new("A", "u", "p")).unwrap();
        mgr.add_password(Password::new("B", "u", "p")).unwrap();
        mgr.add_profile(ConnectionProfile::new_ssh("P", "h", 22, "u"))
            .unwrap();
        assert_eq!(mgr.list_passwords().unwrap().len(), 2);
    }

    // --- KubeConfigItem CRUD ---

    #[test]
    fn add_and_get_kube_config() {
        let mut mgr = make_manager();
        let kube = KubeConfigItem::new(
            "Prod",
            "prod-cluster",
            "https://k8s.example.com",
            KubeAuth::Token {
                token: "tok".into(),
            },
        );
        let id = mgr.add_kube_config(kube.clone()).unwrap();
        let got = mgr.get_kube_config(&id).unwrap();
        assert_eq!(got, kube);
    }

    #[test]
    fn update_kube_config_changes_server() {
        let mut mgr = make_manager();
        let mut kube = KubeConfigItem::new(
            "Dev",
            "dev",
            "https://old.k8s.example.com",
            KubeAuth::Token { token: "t".into() },
        );
        let id = mgr.add_kube_config(kube.clone()).unwrap();
        kube.server = "https://new.k8s.example.com".to_string();
        mgr.update_kube_config(kube.clone()).unwrap();
        assert_eq!(
            mgr.get_kube_config(&id).unwrap().server,
            "https://new.k8s.example.com"
        );
    }

    // --- Type isolation ---

    #[test]
    fn get_wrong_type_returns_wrong_type_error() {
        let mut mgr = make_manager();
        let pw = Password::new("Cred", "user", "pass");
        let id = mgr.add_password(pw).unwrap();
        // Requesting as ConnectionProfile should fail with WrongType.
        let err = mgr.get_profile(&id).unwrap_err();
        assert!(
            matches!(
                err,
                ProfileError::WrongType {
                    expected: "connection_profile",
                    ..
                }
            ),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn wrong_key_causes_decrypt_error() {
        let mut mgr = make_manager();
        let profile = ConnectionProfile::new_ssh("Test", "host", 22, "user");
        let id = mgr.add_profile(profile).unwrap();

        // Create a manager with a different key over the same vault.
        let different_key = [0xFFu8; 32];
        let mgr2 = ProfileManager::new(mgr.vault().clone(), different_key);
        assert!(mgr2.get_profile(&id).is_err());
    }

    // --- list_all ---

    #[test]
    fn list_all_returns_mixed_types() {
        let mut mgr = make_manager();
        mgr.add_profile(ConnectionProfile::new_ssh("P", "h", 22, "u"))
            .unwrap();
        mgr.add_ssh_key(SshKey::new("K", "pem", "pub", SshKeyType::Ed25519))
            .unwrap();
        mgr.add_password(Password::new("C", "u", "p")).unwrap();
        assert_eq!(mgr.list_all().unwrap().len(), 3);
    }

    // --- Round-trip: create → serialize vault → reload → compare ---

    #[test]
    fn create_save_reload_compare() {
        // Step 1: create and populate a manager.
        let mut mgr = make_manager();

        let profile = ConnectionProfile::new_ssh("My Server", "10.0.0.1", 2222, "root");
        let ssh_key = SshKey::new(
            "Deploy Key",
            "-----BEGIN OPENSSH PRIVATE KEY-----\nfakekey\n",
            "ssh-ed25519 AAAA fake",
            SshKeyType::Ed25519,
        );
        let password = Password::new("FTP Cred", "ftpuser", "ftppass");
        let kube = KubeConfigItem::new(
            "Staging K8s",
            "staging",
            "https://staging.k8s.example.com",
            KubeAuth::ExecCredential {
                command: "aws".into(),
                args: vec![
                    "eks".into(),
                    "get-token".into(),
                    "--cluster-name".into(),
                    "staging".into(),
                ],
            },
        );

        let profile_id = mgr.add_profile(profile.clone()).unwrap();
        let key_id = mgr.add_ssh_key(ssh_key.clone()).unwrap();
        let pw_id = mgr.add_password(password.clone()).unwrap();
        let kube_id = mgr.add_kube_config(kube.clone()).unwrap();

        // Step 2: "save" by serializing the vault to JSON.
        let vault_json = mgr.vault().to_json().unwrap();

        // Step 3: "reload" by parsing the JSON into a new manager.
        let reloaded_vault = VaultFile::from_json(&vault_json).unwrap();
        let mgr2 = ProfileManager::new(reloaded_vault, KEY);

        // Step 4: verify each item survived the round-trip.
        assert_eq!(mgr2.get_profile(&profile_id).unwrap(), profile);
        assert_eq!(mgr2.get_ssh_key(&key_id).unwrap(), ssh_key);
        assert_eq!(mgr2.get_password(&pw_id).unwrap(), password);
        assert_eq!(mgr2.get_kube_config(&kube_id).unwrap(), kube);
    }

    // --- Vault isolation ---

    #[test]
    fn vault_contains_no_plaintext_after_add() {
        let mut mgr = make_manager();
        let pw = Password::new("Secret Cred", "user", "my-super-secret-password");
        mgr.add_password(pw).unwrap();
        // The serialized vault must not contain the plaintext password.
        let json = mgr.vault().to_json().unwrap();
        assert!(!json.contains("my-super-secret-password"));
    }
}
