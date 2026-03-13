//! SSH protocol adapter.
//!
//! Implements [`ConnectionAdapter`] and [`TerminalAdapter`] using the
//! `russh` crate (pure-Rust SSH-2 implementation).
//!
//! # Authentication
//!
//! Pass a [`Credential`] to [`SshAdapter::connect`]:
//!
//! - [`Credential::Password`] — username/password
//! - [`Credential::PublicKey`] — OpenSSH PEM private key (Ed25519 or RSA)
//! - [`Credential::SshAgent`] — delegate to the SSH agent at `SSH_AUTH_SOCK`
//!
//! # Host key verification (TOFU)
//!
//! With [`HostKeyPolicy::StrictFirstConnect`] (the default), the server's
//! public key is accepted on the first connection and stored in memory.
//! Subsequent connections that present a *different* key are rejected with
//! [`ConnectionError::HostKeyMismatch`].
//!
//! With [`HostKeyPolicy::AcceptAll`], every key is accepted without
//! comparison (useful for testing; not recommended in production).
//!
//! # Keepalive
//!
//! [`SshSettings::keepalive_secs`] maps directly to `russh::client::Config
//! ::keepalive_interval`. The `russh` session loop sends SSH keepalive
//! messages automatically.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use secrecy::ExposeSecret;
use tokio::sync::mpsc;

use crate::profile::types::{ConnectionProfile, HostKeyPolicy, Protocol};
// Brings `public_key_bytes()` into scope on `PublicKey` for TOFU comparisons.
use russh::keys::PublicKeyBase64 as _;

use super::{ConnectionError, Credential, ExecResult};

// Re-export ConnectionAdapter and TerminalAdapter so callers only need this module.
pub use super::{ConnectionAdapter, TerminalAdapter};

// ---------------------------------------------------------------------------
// Error conversions
// ---------------------------------------------------------------------------

impl From<russh::Error> for ConnectionError {
    fn from(err: russh::Error) -> Self {
        match err {
            russh::Error::IO(io) => ConnectionError::Io(io),
            other => ConnectionError::Protocol(other.to_string()),
        }
    }
}

impl From<russh::keys::Error> for ConnectionError {
    fn from(err: russh::keys::Error) -> Self {
        ConnectionError::Protocol(format!("SSH key error: {err}"))
    }
}

// ---------------------------------------------------------------------------
// Internal shell-task command
// ---------------------------------------------------------------------------

/// Commands sent from `SshAdapter` to the background shell task.
enum ShellCmd {
    /// Raw bytes to write to the remote PTY stdin.
    Data(Vec<u8>),
    /// PTY window-size change.
    Resize { cols: u16, rows: u16 },
    /// Graceful close.
    Close,
}

// ---------------------------------------------------------------------------
// SshClientHandler — implements russh::client::Handler
// ---------------------------------------------------------------------------

/// russh client handler.
///
/// Responsible for host-key verification (TOFU / AcceptAll). Lives only for
/// the duration of the connection setup; after that, `SshAdapter` drives the
/// session through the [`russh::client::Handle`].
struct SshClientHandler {
    host: String,
    port: u16,
    host_key_policy: HostKeyPolicy,
    /// Shared map of `"host:port"` → wire-encoded public-key bytes.
    ///
    /// Keyed by the full `host:port` pair so that two SSH daemons on the same
    /// host but different ports are treated as independent servers.
    /// Written on first connect; compared on subsequent connects within the
    /// same [`SshAdapter`] lifetime (or across `reconnect()` calls, which
    /// reuse this [`Arc`]).
    known_keys: Arc<tokio::sync::Mutex<HashMap<String, Vec<u8>>>>,
}

impl SshClientHandler {
    fn host_key(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

#[async_trait]
impl russh::client::Handler for SshClientHandler {
    type Error = ConnectionError;

    async fn check_server_key(
        &mut self,
        server_public_key: &russh::keys::key::PublicKey,
    ) -> Result<bool, Self::Error> {
        match self.host_key_policy {
            HostKeyPolicy::AcceptAll => Ok(true),
            HostKeyPolicy::StrictFirstConnect => {
                let key_bytes = server_public_key.public_key_bytes();
                let host_key = self.host_key();
                let mut known = self.known_keys.lock().await;
                match known.get(&host_key) {
                    None => {
                        // Trust on first use — store the key.
                        known.insert(host_key, key_bytes);
                        Ok(true)
                    }
                    Some(stored) if stored == &key_bytes => Ok(true),
                    Some(_) => Err(ConnectionError::HostKeyMismatch {
                        host: self.host.clone(),
                    }),
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// SshAdapter
// ---------------------------------------------------------------------------

/// SSH session adapter.
///
/// An `SshAdapter` corresponds to one active SSH connection with one
/// interactive PTY channel. Additional one-shot commands run in their own
/// transient channels via [`TerminalAdapter::exec`].
pub struct SshAdapter {
    /// Handle to the underlying russh session, used to open new channels.
    handle: tokio::sync::Mutex<russh::client::Handle<SshClientHandler>>,
    /// Sends commands to the background shell task.
    shell_tx: mpsc::Sender<ShellCmd>,
    /// Output bytes streamed from the interactive shell. Taken once via
    /// [`TerminalAdapter::output_stream`].
    output_rx: Option<mpsc::Receiver<Vec<u8>>>,
    /// Set to `false` by the background task when the shell channel closes.
    alive: Arc<AtomicBool>,
    /// The profile used to establish this connection (needed for reconnect).
    profile: ConnectionProfile,
    /// Credential used to authenticate (needed for reconnect).
    credential: Credential,
    /// Persists host-key knowledge across reconnects.
    known_keys: Arc<tokio::sync::Mutex<HashMap<String, Vec<u8>>>>,
}

// ---------------------------------------------------------------------------
// Private helpers
// ---------------------------------------------------------------------------

/// Build a [`russh::client::Config`] from the connection profile.
fn build_russh_config(profile: &ConnectionProfile) -> Arc<russh::client::Config> {
    let keepalive_interval = profile
        .ssh
        .as_ref()
        .and_then(|s| s.keepalive_secs)
        .map(Duration::from_secs);

    Arc::new(russh::client::Config {
        keepalive_interval,
        keepalive_max: 3,
        ..Default::default()
    })
}

/// Authenticate the session using the provided credential.
///
/// Returns `Ok(())` on success or a typed [`ConnectionError`] on failure.
async fn authenticate(
    handle: &mut russh::client::Handle<SshClientHandler>,
    username: &str,
    credential: &Credential,
) -> Result<(), ConnectionError> {
    let ok = match credential {
        Credential::Password(secret) => {
            handle
                .authenticate_password(username, secret.expose_secret().as_str())
                .await
                .map_err(ConnectionError::from)?
        }
        Credential::PublicKey {
            private_key_pem,
            passphrase,
        } => {
            let pem = private_key_pem.expose_secret().as_str();
            let pass = passphrase.as_ref().map(|p| p.expose_secret().as_str());
            let key_pair =
                russh::keys::decode_secret_key(pem, pass).map_err(ConnectionError::from)?;
            handle
                .authenticate_publickey(username, Arc::new(key_pair))
                .await
                .map_err(ConnectionError::from)?
        }
        Credential::SshAgent => {
            // Connect to the local SSH agent and authenticate via the first
            // available identity.
            let mut agent = russh::keys::agent::client::AgentClient::connect_env()
                .await
                .map_err(|e| ConnectionError::Protocol(format!("SSH agent error: {e}")))?;

            let identities = agent
                .request_identities()
                .await
                .map_err(|e| ConnectionError::Protocol(format!("SSH agent identities: {e}")))?;

            let Some(pub_key) = identities.into_iter().next() else {
                return Err(ConnectionError::AuthFailed {
                    reason: "SSH agent has no identities".to_owned(),
                });
            };

            let (_, result) = handle
                .authenticate_future(username, pub_key, agent)
                .await;

            result.map_err(|e| ConnectionError::Protocol(format!("SSH agent auth: {e}")))?
        }
    };

    if ok {
        Ok(())
    } else {
        Err(ConnectionError::AuthFailed {
            reason: "server rejected the credential".to_owned(),
        })
    }
}

/// Inner connect helper shared by [`ConnectionAdapter::connect`] and
/// [`ConnectionAdapter::reconnect`].
async fn connect_inner(
    profile: ConnectionProfile,
    credential: Credential,
    known_keys: Arc<tokio::sync::Mutex<HashMap<String, Vec<u8>>>>,
) -> Result<SshAdapter, ConnectionError> {
    let config = build_russh_config(&profile);

    let host_key_policy = profile
        .ssh
        .as_ref()
        .map(|s| s.host_key_policy.clone())
        .unwrap_or_default();

    let handler = SshClientHandler {
        host: profile.host.clone(),
        port: profile.port,
        host_key_policy,
        known_keys: Arc::clone(&known_keys),
    };

    let addr = format!("{}:{}", profile.host, profile.port);
    let mut handle = russh::client::connect(config, addr.as_str(), handler)
        .await
        .map_err(|e| match e {
            ConnectionError::Io(ref io)
                if io.kind() == std::io::ErrorKind::ConnectionRefused =>
            {
                ConnectionError::Refused {
                    host: profile.host.clone(),
                    port: profile.port,
                }
            }
            other => other,
        })?;

    authenticate(&mut handle, &profile.username, &credential).await?;

    // Open the interactive shell channel.
    let channel = handle.channel_open_session().await?;
    channel
        .request_pty(
            false,
            "xterm-256color",
            80,
            24,
            0,
            0,
            &[], // terminal modes
        )
        .await?;
    channel.request_shell(false).await?;

    // Spawn the background task that owns the shell channel.
    let alive = Arc::new(AtomicBool::new(true));
    let (shell_tx, mut shell_rx) = mpsc::channel::<ShellCmd>(64);
    let (output_tx, output_rx) = mpsc::channel::<Vec<u8>>(256);
    let alive_bg = Arc::clone(&alive);

    tokio::spawn(async move {
        let mut ch = channel;
        loop {
            tokio::select! {
                msg = ch.wait() => {
                    match msg {
                        Some(russh::ChannelMsg::Data { data }) => {
                            if output_tx.send(data.to_vec()).await.is_err() {
                                break;
                            }
                        }
                        Some(russh::ChannelMsg::ExitStatus { .. })
                        | Some(russh::ChannelMsg::Eof)
                        | Some(russh::ChannelMsg::Close)
                        | None => break,
                        _ => {}
                    }
                }
                cmd = shell_rx.recv() => {
                    match cmd {
                        Some(ShellCmd::Data(bytes)) => {
                            if ch.data(std::io::Cursor::new(bytes)).await.is_err() {
                                // Write failure means the channel is broken.
                                break;
                            }
                        }
                        Some(ShellCmd::Resize { cols, rows }) => {
                            if ch.window_change(cols as u32, rows as u32, 0, 0).await.is_err() {
                                break;
                            }
                        }
                        Some(ShellCmd::Close) | None => {
                            let _ = ch.close().await;
                            break;
                        }
                    }
                }
            }
        }
        alive_bg.store(false, Ordering::Relaxed);
    });

    Ok(SshAdapter {
        handle: tokio::sync::Mutex::new(handle),
        shell_tx,
        output_rx: Some(output_rx),
        alive,
        profile,
        credential,
        known_keys,
    })
}

// ---------------------------------------------------------------------------
// ConnectionAdapter impl
// ---------------------------------------------------------------------------

#[async_trait]
impl ConnectionAdapter for SshAdapter {
    async fn connect(
        profile: &ConnectionProfile,
        credential: Credential,
    ) -> Result<Self, ConnectionError> {
        let known_keys = Arc::new(tokio::sync::Mutex::new(HashMap::new()));
        connect_inner(profile.clone(), credential, known_keys).await
    }

    async fn disconnect(&mut self) -> Result<(), ConnectionError> {
        // Ask the background task to close the shell channel.
        let _ = self.shell_tx.send(ShellCmd::Close).await;
        // Send the SSH disconnect message.
        let handle = self.handle.lock().await;
        let _ = handle
            .disconnect(russh::Disconnect::ByApplication, "", "en-US")
            .await;
        self.alive.store(false, Ordering::Relaxed);
        Ok(())
    }

    fn is_alive(&self) -> bool {
        self.alive.load(Ordering::Relaxed)
    }

    async fn reconnect(&mut self) -> Result<(), ConnectionError> {
        let profile = self.profile.clone();
        let credential = self.credential.clone();
        let known_keys = Arc::clone(&self.known_keys);
        let _ = self.disconnect().await;
        let new = connect_inner(profile, credential, known_keys).await?;
        *self = new;
        Ok(())
    }

    fn protocol(&self) -> Protocol {
        Protocol::Ssh
    }
}

// ---------------------------------------------------------------------------
// TerminalAdapter impl
// ---------------------------------------------------------------------------

#[async_trait]
impl TerminalAdapter for SshAdapter {
    async fn send_input(&self, data: &[u8]) -> Result<(), ConnectionError> {
        self.shell_tx
            .send(ShellCmd::Data(data.to_vec()))
            .await
            .map_err(|_| ConnectionError::Protocol("shell channel closed".to_owned()))
    }

    fn output_stream(&mut self) -> Option<mpsc::Receiver<Vec<u8>>> {
        self.output_rx.take()
    }

    async fn resize(&self, cols: u16, rows: u16) -> Result<(), ConnectionError> {
        self.shell_tx
            .send(ShellCmd::Resize { cols, rows })
            .await
            .map_err(|_| ConnectionError::Protocol("shell channel closed".to_owned()))
    }

    async fn exec(&self, command: &str) -> Result<ExecResult, ConnectionError> {
        let channel = {
            let handle = self.handle.lock().await;
            handle.channel_open_session().await?
        };

        channel.exec(true, command).await?;

        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let mut exit_code = None;
        let mut ch = channel;

        // Collect output until the channel is fully closed.
        // We handle `Eof` and `Close` separately: `Eof` signals no more data
        // from the server but the exit status / close message may still follow;
        // `Close` (or `None`) means the channel is gone and we stop.
        let mut got_eof = false;
        loop {
            match ch.wait().await {
                Some(russh::ChannelMsg::Data { data }) => {
                    stdout.extend_from_slice(&data);
                }
                Some(russh::ChannelMsg::ExtendedData { data, ext: 1 }) => {
                    stderr.extend_from_slice(&data);
                }
                Some(russh::ChannelMsg::ExitStatus { exit_status }) => {
                    exit_code = Some(exit_status);
                }
                Some(russh::ChannelMsg::Eof) => {
                    got_eof = true;
                    // Exit status often arrives after Eof. Keep reading until
                    // we get it or the channel closes.
                    if exit_code.is_some() {
                        break;
                    }
                }
                Some(russh::ChannelMsg::Close) | None => break,
                _ => {}
            }
            // If the server sent Eof and we already have an exit status,
            // there is nothing more to collect.
            if got_eof && exit_code.is_some() {
                break;
            }
        }

        Ok(ExecResult {
            stdout,
            stderr,
            exit_code,
        })
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::Arc;

    use russh::keys::key::KeyPair;
    use russh::keys::PublicKeyBase64 as _;

    use crate::profile::types::{ConnectionProfile, HostKeyPolicy};

    use super::{ConnectionError, SshClientHandler};

    // -----------------------------------------------------------------------
    // Helper: build a handler for testing the TOFU logic directly.
    // -----------------------------------------------------------------------

    fn make_handler(
        host: &str,
        port: u16,
        policy: HostKeyPolicy,
        known_keys: Arc<tokio::sync::Mutex<HashMap<String, Vec<u8>>>>,
    ) -> SshClientHandler {
        SshClientHandler {
            host: host.to_owned(),
            port,
            host_key_policy: policy,
            known_keys,
        }
    }

    fn fresh_known_keys() -> Arc<tokio::sync::Mutex<HashMap<String, Vec<u8>>>> {
        Arc::new(tokio::sync::Mutex::new(HashMap::new()))
    }

    // -----------------------------------------------------------------------
    // TOFU tests
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn host_key_tofu_first_connect_stores_key() {
        use russh::client::Handler as _;

        let known = fresh_known_keys();
        let mut handler = make_handler("host.example.com", 22, HostKeyPolicy::StrictFirstConnect, Arc::clone(&known));

        let key_pair = KeyPair::generate_ed25519().expect("ed25519 keygen failed");
        let pub_key = key_pair.clone_public_key().expect("clone public key failed");

        let accepted = handler.check_server_key(&pub_key).await.unwrap();
        assert!(accepted, "first connect should be accepted (TOFU)");

        let stored = known.lock().await;
        assert!(
            stored.contains_key("host.example.com:22"),
            "key should be stored keyed by host:port after first connect"
        );
    }

    #[tokio::test]
    async fn host_key_tofu_same_key_accepted_on_second_connect() {
        use russh::client::Handler as _;

        let known = fresh_known_keys();

        let key_pair = KeyPair::generate_ed25519().expect("ed25519 keygen failed");
        let pub_key = key_pair.clone_public_key().expect("clone public key failed");

        // First connect — stores the key.
        let mut h1 = make_handler("host.example.com", 22, HostKeyPolicy::StrictFirstConnect, Arc::clone(&known));
        h1.check_server_key(&pub_key).await.unwrap();

        // Second connect with the SAME key — must be accepted.
        let mut h2 = make_handler("host.example.com", 22, HostKeyPolicy::StrictFirstConnect, Arc::clone(&known));
        let accepted = h2.check_server_key(&pub_key).await.unwrap();
        assert!(accepted, "same key should be accepted on subsequent connects");
    }

    #[tokio::test]
    async fn host_key_tofu_changed_key_rejected() {
        use russh::client::Handler as _;

        let known = fresh_known_keys();

        let key1 = KeyPair::generate_ed25519().expect("keygen");
        let pub1 = key1.clone_public_key().expect("pub1");

        // First connect — stores key1.
        let mut h1 = make_handler("host.example.com", 22, HostKeyPolicy::StrictFirstConnect, Arc::clone(&known));
        h1.check_server_key(&pub1).await.unwrap();

        // Second connect with a DIFFERENT key — must be rejected.
        let key2 = KeyPair::generate_ed25519().expect("keygen");
        let pub2 = key2.clone_public_key().expect("pub2");

        let mut h2 = make_handler("host.example.com", 22, HostKeyPolicy::StrictFirstConnect, Arc::clone(&known));
        let result = h2.check_server_key(&pub2).await;

        match result {
            Err(ConnectionError::HostKeyMismatch { host }) => {
                assert_eq!(host, "host.example.com");
            }
            other => panic!("expected HostKeyMismatch, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn host_key_accept_all_skips_verification() {
        use russh::client::Handler as _;

        let known = fresh_known_keys();

        let key1 = KeyPair::generate_ed25519().expect("keygen");
        let pub1 = key1.clone_public_key().expect("pub1");

        // Simulate a "stored" key for a different fingerprint.
        {
            let key2 = KeyPair::generate_ed25519().expect("keygen");
            let pub2 = key2.clone_public_key().expect("pub2");
            known.lock().await.insert(
                "host.example.com:22".to_owned(),
                pub2.public_key_bytes(),
            );
        }

        // AcceptAll should ignore the mismatch.
        let mut h = make_handler("host.example.com", 22, HostKeyPolicy::AcceptAll, known);
        let accepted = h.check_server_key(&pub1).await.unwrap();
        assert!(accepted, "AcceptAll must accept any key");
    }

    #[tokio::test]
    async fn host_key_tofu_different_hosts_are_independent() {
        use russh::client::Handler as _;

        let known = fresh_known_keys();

        let key_a = KeyPair::generate_ed25519().expect("keygen");
        let pub_a = key_a.clone_public_key().expect("pub_a");

        let key_b = KeyPair::generate_ed25519().expect("keygen");
        let pub_b = key_b.clone_public_key().expect("pub_b");

        // Connect to host-a.
        let mut ha = make_handler("a.example.com", 22, HostKeyPolicy::StrictFirstConnect, Arc::clone(&known));
        ha.check_server_key(&pub_a).await.unwrap();

        // Connect to host-b (different host, different key) — should succeed.
        let mut hb = make_handler("b.example.com", 22, HostKeyPolicy::StrictFirstConnect, Arc::clone(&known));
        let accepted = hb.check_server_key(&pub_b).await.unwrap();
        assert!(accepted, "different host should be treated independently");
    }

    #[tokio::test]
    async fn host_key_tofu_same_host_different_ports_are_independent() {
        use russh::client::Handler as _;

        let known = fresh_known_keys();

        let key_22 = KeyPair::generate_ed25519().expect("keygen");
        let pub_22 = key_22.clone_public_key().expect("pub_22");

        let key_2222 = KeyPair::generate_ed25519().expect("keygen");
        let pub_2222 = key_2222.clone_public_key().expect("pub_2222");

        // Connect on port 22 — stores key_22.
        let mut h22 = make_handler("host.example.com", 22, HostKeyPolicy::StrictFirstConnect, Arc::clone(&known));
        h22.check_server_key(&pub_22).await.unwrap();

        // Connect on port 2222 with a DIFFERENT key — must succeed (different server).
        let mut h2222 = make_handler("host.example.com", 2222, HostKeyPolicy::StrictFirstConnect, Arc::clone(&known));
        let accepted = h2222.check_server_key(&pub_2222).await.unwrap();
        assert!(accepted, "same host on different port must be treated as a different server");
    }

    // -----------------------------------------------------------------------
    // ExecResult helpers
    // -----------------------------------------------------------------------

    #[test]
    fn exec_result_stdout_str_decodes_utf8() {
        use super::ExecResult;

        let r = ExecResult {
            stdout: b"hello\n".to_vec(),
            stderr: vec![],
            exit_code: Some(0),
        };
        assert_eq!(r.stdout_str(), "hello\n");
    }

    #[test]
    fn exec_result_stderr_str_decodes_utf8() {
        use super::ExecResult;

        let r = ExecResult {
            stdout: vec![],
            stderr: b"error\n".to_vec(),
            exit_code: Some(1),
        };
        assert_eq!(r.stderr_str(), "error\n");
    }

    // -----------------------------------------------------------------------
    // Credential debug redaction
    // -----------------------------------------------------------------------

    #[test]
    fn credential_debug_does_not_leak_secrets() {
        use secrecy::SecretString;

        use super::Credential;

        let password_cred = Credential::Password(SecretString::new("hunter2".to_owned()));
        let debug = format!("{password_cred:?}");
        assert!(!debug.contains("hunter2"), "password must not appear in debug output");

        let key_cred = Credential::PublicKey {
            private_key_pem: SecretString::new("-----BEGIN OPENSSH".to_owned()),
            passphrase: None,
        };
        let debug = format!("{key_cred:?}");
        assert!(!debug.contains("BEGIN OPENSSH"), "key material must not appear in debug output");
    }

    // -----------------------------------------------------------------------
    // ConnectionProfile helper
    // -----------------------------------------------------------------------

    #[test]
    fn new_ssh_profile_has_strict_host_key_policy_by_default() {
        let p = ConnectionProfile::new_ssh("Test", "host.example.com", 22, "alice");
        let ssh_settings = p.ssh.as_ref().expect("ssh settings present");
        assert_eq!(ssh_settings.host_key_policy, HostKeyPolicy::StrictFirstConnect);
    }

    #[test]
    fn new_ssh_profile_has_keepalive_enabled_by_default() {
        let p = ConnectionProfile::new_ssh("Test", "host.example.com", 22, "alice");
        let ssh_settings = p.ssh.as_ref().expect("ssh settings present");
        assert!(ssh_settings.keepalive_secs.is_some(), "keepalive should be enabled by default");
    }
}
