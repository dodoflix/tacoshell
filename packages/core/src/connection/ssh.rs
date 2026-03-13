//! SSH adapter — implements `ConnectionAdapter` + `TerminalAdapter` using `russh`.
//!
//! # Authentication methods
//! - **Password** — plain username/password, tunnelled inside the SSH encrypted channel.
//! - **PublicKey** — OpenSSH-format private key (Ed25519 or RSA) stored in the vault.
//! - **Agent** — delegates signing to a local SSH agent via `SSH_AUTH_SOCK` (Unix/macOS only).
//!
//! # Host key verification (TOFU)
//! On first connection the server's key fingerprint is stored in `HostKeyStore`.
//! On subsequent connections the stored fingerprint is compared; a mismatch
//! returns `ConnectionError::HostKeyMismatch`.
//!
//! # Keepalive
//! Controlled by `SshSettings::keepalive_secs` on the profile. Defaults to 30 s.

use std::collections::HashMap;
use std::fmt;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use russh::client;
use russh::keys::key;
use secrecy::{ExposeSecret, SecretString};
use tokio::sync::{mpsc, Mutex};
use tracing::{debug, warn};
use zeroize::Zeroizing;

use crate::connection::{ConnectionAdapter, ConnectionError, ExecResult, TerminalAdapter};
use crate::profile::types::{ConnectionProfile, HostKeyPolicy, Protocol, SshSettings};

// ---------------------------------------------------------------------------
// SshCredential
// ---------------------------------------------------------------------------

/// Resolved authentication credential passed to the SSH adapter at connect time.
///
/// The profile's `credential_id` is resolved by the caller (typically the
/// `ProfileManager`), and the actual secret material is passed here.
pub enum SshCredential {
    /// Username/password authentication.
    Password(SecretString),
    /// Public key authentication using an OpenSSH PEM private key.
    PublicKey(Zeroizing<String>),
    /// Delegate signing to the local SSH agent (`SSH_AUTH_SOCK`).
    Agent,
}

impl fmt::Debug for SshCredential {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SshCredential::Password(_) => f.write_str("SshCredential::Password([redacted])"),
            SshCredential::PublicKey(_) => f.write_str("SshCredential::PublicKey([redacted])"),
            SshCredential::Agent => f.write_str("SshCredential::Agent"),
        }
    }
}

// ---------------------------------------------------------------------------
// HostKeyStore — TOFU implementation
// ---------------------------------------------------------------------------

/// In-memory store of known SSH host key fingerprints (Trust-On-First-Use).
///
/// Keys are indexed by `"host:port"`. The first time a host is seen, its
/// fingerprint is accepted and recorded. On subsequent connections the stored
/// fingerprint must match.
#[derive(Debug, Default)]
pub struct HostKeyStore {
    known: HashMap<String, String>,
}

impl HostKeyStore {
    /// Create an empty store.
    pub fn new() -> Self {
        HostKeyStore {
            known: HashMap::new(),
        }
    }

    /// Check `fingerprint` against the stored value for `host:port`.
    ///
    /// Returns `true` when:
    /// - This is the first connection to `host:port` (fingerprint is stored and accepted), or
    /// - The fingerprint matches the previously stored value.
    ///
    /// Returns `false` when the fingerprint **differs** from the stored value
    /// (a potential MITM attack).
    pub fn check_or_store(&mut self, host: &str, port: u16, fingerprint: &str) -> bool {
        let key = format!("{}:{}", host, port);
        match self.known.get(&key) {
            None => {
                debug!("TOFU: storing new host key for {}", key);
                self.known.insert(key, fingerprint.to_string());
                true
            }
            Some(known_fp) => {
                let matches = known_fp == fingerprint;
                if !matches {
                    warn!(
                        "TOFU: host key mismatch for {} (stored={}, got={})",
                        key, known_fp, fingerprint
                    );
                }
                matches
            }
        }
    }

    /// Remove the stored fingerprint for `host:port`.
    ///
    /// Useful in tests and for "forget this host" functionality.
    pub fn forget(&mut self, host: &str, port: u16) {
        let key = format!("{}:{}", host, port);
        self.known.remove(&key);
    }

    /// Returns the stored fingerprint for `host:port`, if any.
    pub fn get(&self, host: &str, port: u16) -> Option<&str> {
        let key = format!("{}:{}", host, port);
        self.known.get(&key).map(|s| s.as_str())
    }
}

// ---------------------------------------------------------------------------
// Internal: reason why check_server_key returned false
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum KeyRejection {
    Mismatch,
}

// ---------------------------------------------------------------------------
// SshClientHandler — russh::client::Handler implementation
// ---------------------------------------------------------------------------

struct SshClientHandler {
    host: String,
    port: u16,
    policy: HostKeyPolicy,
    store: Arc<std::sync::Mutex<HostKeyStore>>,
    rejection: Arc<std::sync::Mutex<Option<KeyRejection>>>,
}

#[async_trait]
impl client::Handler for SshClientHandler {
    type Error = russh::Error;

    async fn check_server_key(
        &mut self,
        server_public_key: &key::PublicKey,
    ) -> Result<bool, Self::Error> {
        match self.policy {
            HostKeyPolicy::AcceptAll => {
                debug!("host key accepted (AcceptAll policy) for {}:{}", self.host, self.port);
                Ok(true)
            }
            HostKeyPolicy::StrictFirstConnect => {
                let fingerprint = server_public_key.fingerprint();
                let accepted = self
                    .store
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .check_or_store(&self.host, self.port, &fingerprint);

                if !accepted {
                    if let Ok(mut r) = self.rejection.lock() {
                        r.replace(KeyRejection::Mismatch);
                    }
                }
                Ok(accepted)
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Internal channel commands (forwarded from TerminalAdapter methods)
// ---------------------------------------------------------------------------

enum ChannelCommand {
    SendData(Vec<u8>),
    Resize { cols: u16, rows: u16 },
    Close,
}

// ---------------------------------------------------------------------------
// SshAdapter
// ---------------------------------------------------------------------------

/// SSH connection adapter.
///
/// Created via [`SshAdapter::connect_with_credential`]. Implements both
/// [`ConnectionAdapter`] and [`TerminalAdapter`].
pub struct SshAdapter {
    /// The underlying russh session handle, shared for async multi-use.
    handle: Arc<Mutex<client::Handle<SshClientHandler>>>,
    /// Sends commands to the background channel-driver task.
    cmd_tx: mpsc::UnboundedSender<ChannelCommand>,
    /// One-shot receiver for terminal output. Taken on first `output_stream()` call.
    output_rx: std::sync::Mutex<Option<mpsc::Receiver<Vec<u8>>>>,
    /// Original profile (host, port, username, settings).
    profile: ConnectionProfile,
    /// Stored credentials for `reconnect()`.
    credential: SshCredential,
    /// Liveness flag updated by the background task.
    alive: Arc<AtomicBool>,
    /// Shared TOFU host key store — survives reconnects.
    key_store: Arc<std::sync::Mutex<HostKeyStore>>,
}

impl fmt::Debug for SshAdapter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SshAdapter")
            .field("host", &self.profile.host)
            .field("port", &self.profile.port)
            .field("alive", &self.alive.load(Ordering::Relaxed))
            .finish()
    }
}

impl SshAdapter {
    /// Connect with explicit credentials and a host key store.
    ///
    /// This is the primary entry point used by the application. The
    /// `ConnectionAdapter::connect` trait method provides a simplified entry
    /// point for agent-based auth only.
    pub async fn connect_with_credential(
        profile: &ConnectionProfile,
        credential: SshCredential,
        key_store: Arc<std::sync::Mutex<HostKeyStore>>,
    ) -> Result<Self, ConnectionError> {
        let ssh_settings: &SshSettings = profile
            .ssh
            .as_ref()
            .ok_or_else(|| ConnectionError::Protocol("profile missing SSH settings".to_string()))?;

        let keepalive = ssh_settings
            .keepalive_secs
            .map(Duration::from_secs);

        let config = Arc::new(client::Config {
            keepalive_interval: keepalive,
            keepalive_max: 3,
            ..Default::default()
        });

        let rejection = Arc::new(std::sync::Mutex::new(None::<KeyRejection>));
        let handler = SshClientHandler {
            host: profile.host.clone(),
            port: profile.port,
            policy: ssh_settings.host_key_policy.clone(),
            store: Arc::clone(&key_store),
            rejection: Arc::clone(&rejection),
        };

        let addr = (profile.host.as_str(), profile.port);
        let mut session = client::connect(config, addr, handler)
            .await
            .map_err(|e| {
                // Check if we explicitly rejected the key (TOFU mismatch).
                if let Ok(guard) = rejection.lock() {
                    if guard.as_ref() == Some(&KeyRejection::Mismatch) {
                        return ConnectionError::HostKeyMismatch {
                            host: profile.host.clone(),
                        };
                    }
                }
                ConnectionError::from(e)
            })?;

        // Authenticate
        let authenticated = Self::authenticate(&mut session, &profile.username, &credential)
            .await?;

        if !authenticated {
            return Err(ConnectionError::AuthFailed {
                reason: "server rejected credentials".to_string(),
            });
        }

        // Open the primary interactive shell channel
        let mut channel = session
            .channel_open_session()
            .await
            .map_err(ConnectionError::from)?;

        channel
            .request_pty(
                false,
                "xterm-256color",
                80,
                24,
                0,
                0,
                &[],
            )
            .await
            .map_err(ConnectionError::from)?;

        channel
            .request_shell(false)
            .await
            .map_err(ConnectionError::from)?;

        // Spin up the channel driver task
        let alive = Arc::new(AtomicBool::new(true));
        let (cmd_tx, mut cmd_rx) = mpsc::unbounded_channel::<ChannelCommand>();
        let (output_tx, output_rx) = mpsc::channel::<Vec<u8>>(256);
        let alive_task = Arc::clone(&alive);

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    biased;

                    // Commands from the adapter (user input / resize / close)
                    cmd = cmd_rx.recv() => {
                        match cmd {
                            Some(ChannelCommand::SendData(data)) => {
                                if channel.data(&data[..]).await.is_err() {
                                    break;
                                }
                            }
                            Some(ChannelCommand::Resize { cols, rows }) => {
                                let _ = channel
                                    .window_change(cols as u32, rows as u32, 0, 0)
                                    .await;
                            }
                            Some(ChannelCommand::Close) | None => {
                                let _ = channel.eof().await;
                                break;
                            }
                        }
                    }

                    // Data arriving from the SSH server
                    msg = channel.wait() => {
                        match msg {
                            Some(russh::ChannelMsg::Data { ref data }) => {
                                if output_tx.send(data.to_vec()).await.is_err() {
                                    break;
                                }
                            }
                            Some(russh::ChannelMsg::ExtendedData { ref data, .. }) => {
                                // stderr — forward through the same stream
                                if output_tx.send(data.to_vec()).await.is_err() {
                                    break;
                                }
                            }
                            None
                            | Some(russh::ChannelMsg::Eof)
                            | Some(russh::ChannelMsg::ExitStatus { .. })
                            | Some(russh::ChannelMsg::Close) => {
                                break;
                            }
                            _ => {}
                        }
                    }
                }
            }
            alive_task.store(false, Ordering::Relaxed);
            debug!("SSH channel driver task exited");
        });

        Ok(SshAdapter {
            handle: Arc::new(Mutex::new(session)),
            cmd_tx,
            output_rx: std::sync::Mutex::new(Some(output_rx)),
            profile: profile.clone(),
            credential,
            alive,
            key_store,
        })
    }

    // -----------------------------------------------------------------------
    // Private: perform authentication on the session
    // -----------------------------------------------------------------------

    async fn authenticate(
        session: &mut client::Handle<SshClientHandler>,
        username: &str,
        credential: &SshCredential,
    ) -> Result<bool, ConnectionError> {
        match credential {
            SshCredential::Password(secret) => session
                .authenticate_password(username, secret.expose_secret())
                .await
                .map_err(ConnectionError::from),

            SshCredential::PublicKey(pem) => {
                let key_pair =
                    russh::keys::decode_secret_key(pem.as_str(), None).map_err(|e| {
                        ConnectionError::AuthFailed {
                            reason: format!("failed to decode private key: {e}"),
                        }
                    })?;
                session
                    .authenticate_publickey(username, Arc::new(key_pair))
                    .await
                    .map_err(ConnectionError::from)
            }

            SshCredential::Agent => {
                Self::authenticate_with_agent(session, username).await
            }
        }
    }

    /// Try each identity offered by the local SSH agent in turn.
    ///
    /// Returns `Ok(false)` (not an error) when no agent is available or no
    /// identity matches — the caller treats this as an auth failure.
    #[cfg(unix)]
    async fn authenticate_with_agent(
        session: &mut client::Handle<SshClientHandler>,
        username: &str,
    ) -> Result<bool, ConnectionError> {
        use russh::keys::agent::client::AgentClient;

        let mut agent = match AgentClient::connect_env().await {
            Ok(a) => a,
            Err(_) => {
                return Err(ConnectionError::AuthFailed {
                    reason: "SSH_AUTH_SOCK not set or agent not reachable".to_string(),
                });
            }
        };

        let identities = agent.request_identities().await.map_err(|e| {
            ConnectionError::AuthFailed {
                reason: format!("agent identity request failed: {e}"),
            }
        })?;

        for public_key in identities {
            let (new_agent, result) =
                session.authenticate_future(username, public_key, agent).await;
            agent = new_agent;
            match result {
                Ok(true) => return Ok(true),
                Ok(false) => continue,
                Err(e) => {
                    return Err(ConnectionError::AuthFailed {
                        reason: format!("agent auth error: {e}"),
                    })
                }
            }
        }

        Ok(false)
    }

    #[cfg(not(unix))]
    async fn authenticate_with_agent(
        _session: &mut client::Handle<SshClientHandler>,
        _username: &str,
    ) -> Result<bool, ConnectionError> {
        Err(ConnectionError::NotSupported)
    }
}

// ---------------------------------------------------------------------------
// ConnectionAdapter impl
// ---------------------------------------------------------------------------

#[async_trait]
impl ConnectionAdapter for SshAdapter {
    /// Connect using only the profile (agent auth or no-credential).
    ///
    /// For password and public key authentication, use
    /// [`SshAdapter::connect_with_credential`] instead.
    async fn connect(profile: &ConnectionProfile) -> Result<Self, ConnectionError> {
        SshAdapter::connect_with_credential(
            profile,
            SshCredential::Agent,
            Arc::new(std::sync::Mutex::new(HostKeyStore::new())),
        )
        .await
    }

    async fn disconnect(&mut self) -> Result<(), ConnectionError> {
        // Signal the channel driver to stop
        let _ = self.cmd_tx.send(ChannelCommand::Close);
        // Send SSH disconnect
        self.handle
            .lock()
            .await
            .disconnect(russh::Disconnect::ByApplication, "", "English")
            .await
            .map_err(ConnectionError::from)
    }

    fn is_alive(&self) -> bool {
        self.alive.load(Ordering::Relaxed)
    }

    async fn reconnect(&mut self) -> Result<(), ConnectionError> {
        let new = SshAdapter::connect_with_credential(
            &self.profile,
            // We cannot move out of self.credential directly, so we re-construct
            // a shallow credential reference. For password/key, we clone the inner value.
            match &self.credential {
                SshCredential::Password(s) => {
                    SshCredential::Password(SecretString::new(s.expose_secret().into()))
                }
                SshCredential::PublicKey(pem) => {
                    SshCredential::PublicKey(Zeroizing::new(pem.as_str().to_string()))
                }
                SshCredential::Agent => SshCredential::Agent,
            },
            Arc::clone(&self.key_store),
        )
        .await?;

        *self = new;
        Ok(())
    }

    fn protocol(&self) -> Protocol {
        Protocol::Ssh
    }
}

// ---------------------------------------------------------------------------
// Private helpers
// ---------------------------------------------------------------------------

/// Returns an already-closed `mpsc::Receiver`. Used as a fallback when the
/// real receiver has already been taken or the mutex is poisoned.
fn closed_receiver() -> mpsc::Receiver<Vec<u8>> {
    let (_, rx) = mpsc::channel(1);
    rx
}

// ---------------------------------------------------------------------------
// TerminalAdapter impl
// ---------------------------------------------------------------------------

#[async_trait]
impl TerminalAdapter for SshAdapter {
    async fn send_input(&self, data: &[u8]) -> Result<(), ConnectionError> {
        self.cmd_tx
            .send(ChannelCommand::SendData(data.to_vec()))
            .map_err(|_| ConnectionError::Protocol("channel driver task has exited".to_string()))
    }

    fn output_stream(&self) -> mpsc::Receiver<Vec<u8>> {
        let mut guard = match self.output_rx.lock() {
            Ok(g) => g,
            Err(_) => return closed_receiver(),
        };
        guard.take().unwrap_or_else(closed_receiver)
    }

    async fn resize(&self, cols: u16, rows: u16) -> Result<(), ConnectionError> {
        self.cmd_tx
            .send(ChannelCommand::Resize { cols, rows })
            .map_err(|_| ConnectionError::Protocol("channel driver task has exited".to_string()))
    }

    async fn exec(&self, command: &str) -> Result<ExecResult, ConnectionError> {
        let mut channel = self
            .handle
            .lock()
            .await
            .channel_open_session()
            .await
            .map_err(ConnectionError::from)?;

        channel
            .exec(false, command)
            .await
            .map_err(ConnectionError::from)?;

        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let mut exit_code: u32 = 0;

        loop {
            match channel.wait().await {
                Some(russh::ChannelMsg::Data { ref data }) => {
                    stdout.extend_from_slice(data);
                }
                Some(russh::ChannelMsg::ExtendedData { ref data, .. }) => {
                    stderr.extend_from_slice(data);
                }
                Some(russh::ChannelMsg::ExitStatus { exit_status }) => {
                    exit_code = exit_status;
                }
                None | Some(russh::ChannelMsg::Eof) | Some(russh::ChannelMsg::Close) => {
                    break;
                }
                _ => {}
            }
        }

        Ok(ExecResult {
            exit_code,
            stdout,
            stderr,
        })
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::profile::types::{HostKeyPolicy, SshSettings};

    // -----------------------------------------------------------------------
    // HostKeyStore — TOFU logic
    // -----------------------------------------------------------------------

    #[test]
    fn first_connection_is_accepted_and_stored() {
        let mut store = HostKeyStore::new();
        assert!(
            store.check_or_store("example.com", 22, "SHA256:abc123"),
            "first connection should be accepted"
        );
    }

    #[test]
    fn second_connection_with_same_key_is_accepted() {
        let mut store = HostKeyStore::new();
        store.check_or_store("example.com", 22, "SHA256:abc123");
        assert!(
            store.check_or_store("example.com", 22, "SHA256:abc123"),
            "reconnection with same key should be accepted"
        );
    }

    #[test]
    fn second_connection_with_different_key_is_rejected() {
        let mut store = HostKeyStore::new();
        store.check_or_store("example.com", 22, "SHA256:abc123");
        assert!(
            !store.check_or_store("example.com", 22, "SHA256:DIFFERENT"),
            "reconnection with a different key must be rejected (TOFU)"
        );
    }

    #[test]
    fn different_ports_are_treated_as_different_hosts() {
        let mut store = HostKeyStore::new();
        store.check_or_store("example.com", 22, "SHA256:key-a");
        // Port 2222 is a completely independent entry
        assert!(
            store.check_or_store("example.com", 2222, "SHA256:key-b"),
            "different port should be treated as a separate host entry"
        );
    }

    #[test]
    fn forget_clears_stored_key() {
        let mut store = HostKeyStore::new();
        store.check_or_store("example.com", 22, "SHA256:abc123");
        store.forget("example.com", 22);
        // After forget, next connection is treated as first again
        assert!(
            store.check_or_store("example.com", 22, "SHA256:NEWKEY"),
            "after forget, any key should be accepted as if first time"
        );
    }

    #[test]
    fn get_returns_stored_fingerprint() {
        let mut store = HostKeyStore::new();
        assert!(store.get("example.com", 22).is_none());
        store.check_or_store("example.com", 22, "SHA256:abc123");
        assert_eq!(store.get("example.com", 22), Some("SHA256:abc123"));
    }

    #[test]
    fn multiple_hosts_tracked_independently() {
        let mut store = HostKeyStore::new();
        store.check_or_store("host-a.example.com", 22, "SHA256:key-a");
        store.check_or_store("host-b.example.com", 22, "SHA256:key-b");
        // Each host only accepts its own key
        assert!(!store.check_or_store("host-a.example.com", 22, "SHA256:key-b"));
        assert!(!store.check_or_store("host-b.example.com", 22, "SHA256:key-a"));
    }

    // -----------------------------------------------------------------------
    // ConnectionError — display messages
    // -----------------------------------------------------------------------

    #[test]
    fn connection_error_refused_message() {
        let e = ConnectionError::Refused {
            host: "example.com".to_string(),
            port: 22,
        };
        let msg = e.to_string();
        assert!(msg.contains("example.com"));
        assert!(msg.contains("22"));
    }

    #[test]
    fn connection_error_auth_failed_message() {
        let e = ConnectionError::AuthFailed {
            reason: "bad password".to_string(),
        };
        assert!(e.to_string().contains("bad password"));
    }

    #[test]
    fn connection_error_host_key_mismatch_message() {
        let e = ConnectionError::HostKeyMismatch {
            host: "evil.example.com".to_string(),
        };
        assert!(e.to_string().contains("evil.example.com"));
    }

    #[test]
    fn connection_error_timeout_message() {
        let e = ConnectionError::Timeout {
            timeout: Duration::from_secs(30),
        };
        assert!(e.to_string().contains("30s"));
    }

    // -----------------------------------------------------------------------
    // SshCredential — Debug does not expose secrets
    // -----------------------------------------------------------------------

    #[test]
    fn ssh_credential_password_debug_does_not_expose_secret() {
        let cred =
            SshCredential::Password(SecretString::new("supersecretpassword".to_string()));
        let debug_str = format!("{cred:?}");
        assert!(
            !debug_str.contains("supersecretpassword"),
            "password must not appear in Debug output"
        );
        assert!(debug_str.contains("redacted"));
    }

    #[test]
    fn ssh_credential_public_key_debug_does_not_expose_pem() {
        let cred = SshCredential::PublicKey(Zeroizing::new("-----BEGIN OPENSSH PRIVATE KEY-----\nfakekey\n-----END OPENSSH PRIVATE KEY-----".to_string()));
        let debug_str = format!("{cred:?}");
        assert!(
            !debug_str.contains("BEGIN OPENSSH"),
            "private key PEM must not appear in Debug output"
        );
        assert!(debug_str.contains("redacted"));
    }

    // -----------------------------------------------------------------------
    // Protocol
    // -----------------------------------------------------------------------

    #[test]
    fn ssh_settings_default_has_tofu_policy() {
        let settings = SshSettings::default();
        assert_eq!(settings.host_key_policy, HostKeyPolicy::StrictFirstConnect);
    }

    #[test]
    fn ssh_settings_default_keepalive_is_30s() {
        let settings = SshSettings::default();
        assert_eq!(settings.keepalive_secs, Some(30));
    }

    // -----------------------------------------------------------------------
    // Integration tests — require `--features integration` and Docker
    // -----------------------------------------------------------------------

    /// Integration test: connect with password auth, execute a command, disconnect.
    #[cfg(feature = "integration")]
    #[tokio::test]
    async fn password_auth_connect_exec_disconnect() {
        use testcontainers::{clients::Cli, images::generic::GenericImage};

        let docker = Cli::default();
        let ssh_image = GenericImage::new("linuxserver/openssh-server", "latest")
            .with_env_var("PASSWORD_ACCESS", "true")
            .with_env_var("USER_PASSWORD", "testpass")
            .with_env_var("USER_NAME", "testuser")
            .with_exposed_port(2222);
        let container = docker.run(ssh_image);
        let port = container.get_host_port_ipv4(2222);

        let profile = crate::profile::types::ConnectionProfile::new_ssh(
            "Integration Test",
            "127.0.0.1",
            port,
            "testuser",
        );

        let mut adapter = SshAdapter::connect_with_credential(
            &profile,
            SshCredential::Password(SecretString::new("testpass".into())),
            Arc::new(std::sync::Mutex::new(HostKeyStore::new())),
        )
        .await
        .expect("should connect with password");

        assert!(adapter.is_alive());
        assert_eq!(adapter.protocol(), Protocol::Ssh);

        let result = adapter.exec("echo hello-from-ssh").await.unwrap();
        assert_eq!(result.exit_code, 0);
        assert!(String::from_utf8_lossy(&result.stdout).contains("hello-from-ssh"));

        adapter.disconnect().await.expect("should disconnect cleanly");
    }

    /// Integration test: TOFU — first connection stores key, mismatch on second server.
    #[cfg(feature = "integration")]
    #[tokio::test]
    async fn tofu_stores_key_on_first_connect_and_rejects_mismatch() {
        use testcontainers::{clients::Cli, images::generic::GenericImage};

        let docker = Cli::default();
        let ssh_image = GenericImage::new("linuxserver/openssh-server", "latest")
            .with_env_var("PASSWORD_ACCESS", "true")
            .with_env_var("USER_PASSWORD", "testpass")
            .with_env_var("USER_NAME", "testuser")
            .with_exposed_port(2222);
        let container = docker.run(ssh_image);
        let port = container.get_host_port_ipv4(2222);

        let store = Arc::new(std::sync::Mutex::new(HostKeyStore::new()));
        let profile = crate::profile::types::ConnectionProfile::new_ssh(
            "TOFU test",
            "127.0.0.1",
            port,
            "testuser",
        );
        let cred = || SshCredential::Password(SecretString::new("testpass".into()));

        // First connection — should succeed (key stored via TOFU)
        let mut adapter =
            SshAdapter::connect_with_credential(&profile, cred(), Arc::clone(&store))
                .await
                .expect("first connection should succeed");
        adapter.disconnect().await.ok();

        // Tamper with the store — inject a fake fingerprint to simulate MITM
        {
            let mut s = store.lock().unwrap();
            s.forget("127.0.0.1", port);
            s.check_or_store("127.0.0.1", port, "SHA256:fakefakefake");
        }

        // Second connection — should fail with HostKeyMismatch
        let result =
            SshAdapter::connect_with_credential(&profile, cred(), Arc::clone(&store)).await;
        assert!(
            matches!(result, Err(ConnectionError::HostKeyMismatch { .. })),
            "expected HostKeyMismatch, got: {result:?}"
        );
    }

    /// Integration test: public key (Ed25519) authentication.
    #[cfg(feature = "integration")]
    #[tokio::test]
    async fn public_key_ed25519_auth_succeeds() {
        use testcontainers::{clients::Cli, images::generic::GenericImage};

        // Generate a throwaway Ed25519 key pair for the test
        let key_pair = russh::keys::key::KeyPair::generate_ed25519()
            .expect("should generate ed25519 key pair");
        let public_key = key_pair.clone_public_key().unwrap();
        let public_key_b64 = russh::keys::write_public_key_base64(&public_key).unwrap();
        let pem = russh::keys::encode_pkcs8_pem(&key_pair).unwrap();

        let docker = Cli::default();
        let ssh_image = GenericImage::new("linuxserver/openssh-server", "latest")
            .with_env_var("PUBLIC_KEY", format!("ssh-ed25519 {}", public_key_b64))
            .with_env_var("USER_NAME", "testuser")
            .with_exposed_port(2222);
        let container = docker.run(ssh_image);
        let port = container.get_host_port_ipv4(2222);

        let profile = crate::profile::types::ConnectionProfile::new_ssh(
            "Ed25519 Test",
            "127.0.0.1",
            port,
            "testuser",
        );

        let mut adapter = SshAdapter::connect_with_credential(
            &profile,
            SshCredential::PublicKey(Zeroizing::new(pem)),
            Arc::new(std::sync::Mutex::new(HostKeyStore::new())),
        )
        .await
        .expect("should connect with ed25519 key");

        assert!(adapter.is_alive());
        adapter.disconnect().await.ok();
    }

    /// Integration test: wrong password returns AuthFailed.
    #[cfg(feature = "integration")]
    #[tokio::test]
    async fn wrong_password_returns_auth_failed() {
        use testcontainers::{clients::Cli, images::generic::GenericImage};

        let docker = Cli::default();
        let ssh_image = GenericImage::new("linuxserver/openssh-server", "latest")
            .with_env_var("PASSWORD_ACCESS", "true")
            .with_env_var("USER_PASSWORD", "correctpass")
            .with_env_var("USER_NAME", "testuser")
            .with_exposed_port(2222);
        let container = docker.run(ssh_image);
        let port = container.get_host_port_ipv4(2222);

        let profile = crate::profile::types::ConnectionProfile::new_ssh(
            "Bad auth test",
            "127.0.0.1",
            port,
            "testuser",
        );

        let result = SshAdapter::connect_with_credential(
            &profile,
            SshCredential::Password(SecretString::new("wrongpass".into())),
            Arc::new(std::sync::Mutex::new(HostKeyStore::new())),
        )
        .await;

        assert!(
            matches!(result, Err(ConnectionError::AuthFailed { .. })),
            "expected AuthFailed, got: {result:?}"
        );
    }

    /// Integration test: send terminal input, receive echoed output.
    #[cfg(feature = "integration")]
    #[tokio::test]
    async fn send_input_echoes_in_output_stream() {
        use testcontainers::{clients::Cli, images::generic::GenericImage};
        use tokio::time::timeout;

        let docker = Cli::default();
        let ssh_image = GenericImage::new("linuxserver/openssh-server", "latest")
            .with_env_var("PASSWORD_ACCESS", "true")
            .with_env_var("USER_PASSWORD", "testpass")
            .with_env_var("USER_NAME", "testuser")
            .with_exposed_port(2222);
        let container = docker.run(ssh_image);
        let port = container.get_host_port_ipv4(2222);

        let profile = crate::profile::types::ConnectionProfile::new_ssh(
            "Echo test",
            "127.0.0.1",
            port,
            "testuser",
        );

        let adapter = SshAdapter::connect_with_credential(
            &profile,
            SshCredential::Password(SecretString::new("testpass".into())),
            Arc::new(std::sync::Mutex::new(HostKeyStore::new())),
        )
        .await
        .expect("should connect");

        let mut rx = adapter.output_stream();

        // Wait for the shell prompt to appear
        let _ = timeout(Duration::from_secs(5), rx.recv()).await;

        // Send a command
        adapter.send_input(b"echo integration-test-marker\n").await.unwrap();

        // Collect output for up to 5 seconds
        let mut collected = Vec::new();
        let deadline = std::time::Instant::now() + Duration::from_secs(5);
        loop {
            let remaining = deadline.saturating_duration_since(std::time::Instant::now());
            if remaining.is_zero() {
                break;
            }
            match timeout(remaining, rx.recv()).await {
                Ok(Some(chunk)) => {
                    collected.extend_from_slice(&chunk);
                    if String::from_utf8_lossy(&collected)
                        .contains("integration-test-marker")
                    {
                        break;
                    }
                }
                _ => break,
            }
        }

        let output = String::from_utf8_lossy(&collected);
        assert!(
            output.contains("integration-test-marker"),
            "expected echo output, got: {output:?}"
        );
    }

    /// Integration test: resize sends window-change without error.
    #[cfg(feature = "integration")]
    #[tokio::test]
    async fn resize_succeeds_on_live_connection() {
        use testcontainers::{clients::Cli, images::generic::GenericImage};

        let docker = Cli::default();
        let ssh_image = GenericImage::new("linuxserver/openssh-server", "latest")
            .with_env_var("PASSWORD_ACCESS", "true")
            .with_env_var("USER_PASSWORD", "testpass")
            .with_env_var("USER_NAME", "testuser")
            .with_exposed_port(2222);
        let container = docker.run(ssh_image);
        let port = container.get_host_port_ipv4(2222);

        let profile = crate::profile::types::ConnectionProfile::new_ssh(
            "Resize test",
            "127.0.0.1",
            port,
            "testuser",
        );

        let adapter = SshAdapter::connect_with_credential(
            &profile,
            SshCredential::Password(SecretString::new("testpass".into())),
            Arc::new(std::sync::Mutex::new(HostKeyStore::new())),
        )
        .await
        .expect("should connect");

        adapter.resize(120, 40).await.expect("resize should succeed");
    }
}
