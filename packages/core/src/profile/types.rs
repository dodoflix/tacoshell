use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

pub type ProfileId = String;

// ---------------------------------------------------------------------------
// Enumerations shared across types
// ---------------------------------------------------------------------------

/// Top-level protocol discriminant.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Protocol {
    Ssh,
    Sftp,
    Ftp,
    Kubernetes,
}

/// SSH host key verification policy.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HostKeyPolicy {
    /// Trust on first connection, reject changes (TOFU). Recommended default.
    #[default]
    StrictFirstConnect,
    /// Accept any host key without verification. User must explicitly opt in.
    AcceptAll,
}

/// FTP connection security mode.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FtpMode {
    /// Plain FTP on port 21 (no encryption — not recommended).
    Plain,
    /// FTPS with explicit TLS upgrade (STARTTLS on port 21).
    ExplicitTls,
    /// FTPS with implicit TLS (always-on TLS on port 990).
    ImplicitTls,
}

/// SSH key algorithm.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SshKeyType {
    Ed25519,
    Rsa,
    Ecdsa,
}

/// Kubernetes authentication method.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum KubeAuth {
    /// Bearer token (e.g., service account token).
    Token { token: String },
    /// Client certificate + private key (PEM-encoded).
    ClientCert { cert: String, key: String },
    /// External credential plugin (e.g., `aws eks get-token`).
    ExecCredential { command: String, args: Vec<String> },
}

// ---------------------------------------------------------------------------
// Per-protocol sub-settings embedded inside ConnectionProfile
// ---------------------------------------------------------------------------

/// SSH-specific settings that supplement `ConnectionProfile`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SshSettings {
    pub host_key_policy: HostKeyPolicy,
    /// SSH keepalive interval in seconds. `None` disables keepalives.
    pub keepalive_secs: Option<u64>,
    /// Ordered list of jump host profile IDs (ProxyJump chain).
    pub jump_host_ids: Vec<ProfileId>,
}

impl Default for SshSettings {
    fn default() -> Self {
        SshSettings {
            host_key_policy: HostKeyPolicy::default(),
            keepalive_secs: Some(30),
            jump_host_ids: Vec::new(),
        }
    }
}

/// FTP-specific settings that supplement `ConnectionProfile`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FtpSettings {
    pub mode: FtpMode,
    /// Use passive (PASV) mode. `true` by default (works through most firewalls).
    pub passive: bool,
}

impl Default for FtpSettings {
    fn default() -> Self {
        FtpSettings {
            mode: FtpMode::ExplicitTls,
            passive: true,
        }
    }
}

// ---------------------------------------------------------------------------
// Core vault item types
// ---------------------------------------------------------------------------

/// A connection profile — the top-level unit of user data.
///
/// Each profile represents one saved connection (SSH, SFTP, FTP, or Kubernetes).
/// Authentication credentials are stored separately and referenced by
/// `credential_id` to allow credential reuse across profiles.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConnectionProfile {
    pub id: ProfileId,
    /// Human-readable label shown in the sidebar (e.g., "Production Web Server").
    pub display_name: String,
    pub protocol: Protocol,
    pub host: String,
    pub port: u16,
    pub username: String,
    /// ID of the `Password` or `SshKey` vault item used for authentication.
    /// `None` if the protocol does not require credentials or uses agent auth.
    pub credential_id: Option<ProfileId>,
    /// Present and non-None for SSH and SFTP protocols.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ssh: Option<SshSettings>,
    /// Present and non-None for FTP protocols.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ftp: Option<FtpSettings>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl ConnectionProfile {
    /// Construct a minimal SSH profile with default settings.
    pub fn new_ssh(
        display_name: impl Into<String>,
        host: impl Into<String>,
        port: u16,
        username: impl Into<String>,
    ) -> Self {
        let now = Utc::now();
        ConnectionProfile {
            id: uuid::Uuid::new_v4().to_string(),
            display_name: display_name.into(),
            protocol: Protocol::Ssh,
            host: host.into(),
            port,
            username: username.into(),
            credential_id: None,
            ssh: Some(SshSettings::default()),
            ftp: None,
            created_at: now,
            updated_at: now,
        }
    }
}

/// An SSH private key stored in the vault.
///
/// The private key material is stored encrypted inside a vault item. The
/// `public_key` is included so the UI can display key fingerprints without
/// decrypting the vault.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SshKey {
    pub id: ProfileId,
    pub display_name: String,
    /// OpenSSH-format private key (PEM). Sensitive — stored encrypted.
    pub private_key_pem: String,
    /// OpenSSH-format public key. Not sensitive.
    pub public_key: String,
    pub key_type: SshKeyType,
    /// Profile IDs that reference this key via `credential_id`.
    pub associated_profile_ids: Vec<ProfileId>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl SshKey {
    pub fn new(
        display_name: impl Into<String>,
        private_key_pem: impl Into<String>,
        public_key: impl Into<String>,
        key_type: SshKeyType,
    ) -> Self {
        let now = Utc::now();
        SshKey {
            id: uuid::Uuid::new_v4().to_string(),
            display_name: display_name.into(),
            private_key_pem: private_key_pem.into(),
            public_key: public_key.into(),
            key_type,
            associated_profile_ids: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }
}

/// A stored username/password credential.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Password {
    pub id: ProfileId,
    pub display_name: String,
    pub username: String,
    /// Plaintext password. Sensitive — stored encrypted in the vault.
    pub password: String,
    /// Profile IDs that reference this credential via `credential_id`.
    pub associated_profile_ids: Vec<ProfileId>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Password {
    pub fn new(
        display_name: impl Into<String>,
        username: impl Into<String>,
        password: impl Into<String>,
    ) -> Self {
        let now = Utc::now();
        Password {
            id: uuid::Uuid::new_v4().to_string(),
            display_name: display_name.into(),
            username: username.into(),
            password: password.into(),
            associated_profile_ids: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }
}

/// A Kubernetes cluster configuration extracted from a kubeconfig file.
///
/// Rather than storing the entire kubeconfig, Tacoshell stores only the
/// cluster/context/auth triples needed for a specific connection.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KubeConfigItem {
    pub id: ProfileId,
    pub display_name: String,
    pub cluster_name: String,
    /// Kubernetes API server URL (e.g., `https://api.example.com`).
    pub server: String,
    /// CA certificate in base64 PEM format. `None` if the CA is trusted by the OS.
    pub ca_cert: Option<String>,
    pub auth: KubeAuth,
    /// Default namespace for `kubectl`-style operations.
    pub default_namespace: Option<String>,
    /// Profile IDs that reference this kubeconfig.
    pub associated_profile_ids: Vec<ProfileId>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl KubeConfigItem {
    pub fn new(
        display_name: impl Into<String>,
        cluster_name: impl Into<String>,
        server: impl Into<String>,
        auth: KubeAuth,
    ) -> Self {
        let now = Utc::now();
        KubeConfigItem {
            id: uuid::Uuid::new_v4().to_string(),
            display_name: display_name.into(),
            cluster_name: cluster_name.into(),
            server: server.into(),
            ca_cert: None,
            auth,
            default_namespace: None,
            associated_profile_ids: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }
}

// ---------------------------------------------------------------------------
// Vault payload wrapper
// ---------------------------------------------------------------------------

/// Type-tagged wrapper for all vault item payloads.
///
/// This is what gets JSON-serialized and then AES-256-GCM encrypted before
/// being stored as an `EncryptedItem` in the vault. The `"type"` field in the
/// JSON enables the correct deserialization path on load.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum VaultPayload {
    ConnectionProfile(ConnectionProfile),
    SshKey(SshKey),
    Password(Password),
    KubeConfig(KubeConfigItem),
}

impl VaultPayload {
    pub fn type_name(&self) -> &'static str {
        match self {
            VaultPayload::ConnectionProfile(_) => "connection_profile",
            VaultPayload::SshKey(_) => "ssh_key",
            VaultPayload::Password(_) => "password",
            VaultPayload::KubeConfig(_) => "kube_config",
        }
    }

    pub fn id(&self) -> &str {
        match self {
            VaultPayload::ConnectionProfile(p) => &p.id,
            VaultPayload::SshKey(k) => &k.id,
            VaultPayload::Password(p) => &p.id,
            VaultPayload::KubeConfig(k) => &k.id,
        }
    }
}

impl From<ConnectionProfile> for VaultPayload {
    fn from(p: ConnectionProfile) -> Self {
        VaultPayload::ConnectionProfile(p)
    }
}

impl From<SshKey> for VaultPayload {
    fn from(k: SshKey) -> Self {
        VaultPayload::SshKey(k)
    }
}

impl From<Password> for VaultPayload {
    fn from(p: Password) -> Self {
        VaultPayload::Password(p)
    }
}

impl From<KubeConfigItem> for VaultPayload {
    fn from(k: KubeConfigItem) -> Self {
        VaultPayload::KubeConfig(k)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn connection_profile_new_ssh_defaults() {
        let p = ConnectionProfile::new_ssh("My Server", "example.com", 22, "alice");
        assert_eq!(p.protocol, Protocol::Ssh);
        assert_eq!(p.port, 22);
        assert_eq!(p.host, "example.com");
        assert!(p.ssh.is_some());
        assert!(p.ftp.is_none());
        assert!(p.credential_id.is_none());
    }

    #[test]
    fn ssh_key_new_sets_fields() {
        let k = SshKey::new(
            "My Key",
            "-----BEGIN OPENSSH PRIVATE KEY-----",
            "ssh-ed25519 AAAA",
            SshKeyType::Ed25519,
        );
        assert_eq!(k.key_type, SshKeyType::Ed25519);
        assert!(k.associated_profile_ids.is_empty());
    }

    #[test]
    fn password_new_sets_fields() {
        let p = Password::new("My Cred", "alice", "s3cr3t");
        assert_eq!(p.username, "alice");
        assert_eq!(p.password, "s3cr3t");
    }

    #[test]
    fn kube_config_item_new_sets_fields() {
        let k = KubeConfigItem::new(
            "Prod Cluster",
            "prod",
            "https://k8s.example.com",
            KubeAuth::Token {
                token: "tok".into(),
            },
        );
        assert_eq!(k.cluster_name, "prod");
        assert!(k.ca_cert.is_none());
        assert!(k.default_namespace.is_none());
    }

    #[test]
    fn vault_payload_type_name_is_stable() {
        let profile = ConnectionProfile::new_ssh("Test", "host", 22, "user");
        let payload = VaultPayload::from(profile);
        assert_eq!(payload.type_name(), "connection_profile");
    }

    #[test]
    fn vault_payload_id_matches_inner_id() {
        let profile = ConnectionProfile::new_ssh("Test", "host", 22, "user");
        let id = profile.id.clone();
        let payload = VaultPayload::ConnectionProfile(profile);
        assert_eq!(payload.id(), id);
    }

    #[test]
    fn vault_payload_serialization_includes_type_tag() {
        let profile = ConnectionProfile::new_ssh("Test", "host", 22, "user");
        let payload = VaultPayload::ConnectionProfile(profile);
        let json = serde_json::to_string(&payload).unwrap();
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(value["type"], "connection_profile");
    }

    #[test]
    fn vault_payload_round_trips_through_json() {
        let original = Password::new("Work VPN", "bob", "hunter2");
        let payload = VaultPayload::Password(original.clone());
        let json = serde_json::to_string(&payload).unwrap();
        let restored: VaultPayload = serde_json::from_str(&json).unwrap();
        assert_eq!(restored, payload);
    }

    #[test]
    fn kube_auth_variants_round_trip() {
        let variants = vec![
            KubeAuth::Token {
                token: "tok".into(),
            },
            KubeAuth::ClientCert {
                cert: "cert".into(),
                key: "key".into(),
            },
            KubeAuth::ExecCredential {
                command: "aws".into(),
                args: vec!["eks".into()],
            },
        ];
        for auth in variants {
            let json = serde_json::to_string(&auth).unwrap();
            let restored: KubeAuth = serde_json::from_str(&json).unwrap();
            assert_eq!(restored, auth);
        }
    }
}
