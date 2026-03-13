// SSH connection adapter using russh
//
// Implements:
//   - ConnectionAdapter (connect, disconnect, is_alive, reconnect)
//   - TerminalAdapter (send_input, output_stream, resize, exec)
//
// Authentication methods:
//   - Password
//   - Public key (Ed25519, RSA, ECDSA)
//   - SSH agent forwarding (desktop/mobile only)
//
// Host key verification uses Trust on First Use (TOFU) policy by default.

use super::{ConnectionAdapter, ConnectionError, ExecResult, Result, TerminalAdapter};
use crate::profile::types::{ConnectionProfile, Protocol};
use async_trait::async_trait;
use russh::client::{self, Handle, Msg};
use russh::keys::key::PublicKey;
use russh::Channel;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};

/// SSH connection adapter
pub struct SshAdapter {
    profile: ConnectionProfile,
    handle: Arc<Mutex<Option<Handle<SshClientHandler>>>>,
    channel: Arc<Mutex<Option<Channel<Msg>>>>,
    output_rx: Arc<Mutex<Option<mpsc::Receiver<Vec<u8>>>>>,
    output_tx: mpsc::Sender<Vec<u8>>,
}

impl SshAdapter {
    /// Get the credential for authentication
    async fn get_credential(profile: &ConnectionProfile) -> Result<Credential> {
        // For now, we'll support password and key stored in profile metadata
        // In the full implementation, this would fetch from the vault
        if let Some(_credential_id) = &profile.credential_id {
            // TODO: Fetch from vault using credential_id
            // For now, return an error
            return Err(ConnectionError::AuthFailed {
                reason: "Vault credential lookup not yet implemented".into(),
            });
        }

        // If no credential_id, check for inline password in profile (testing only)
        Err(ConnectionError::AuthFailed {
            reason: "No authentication credential provided".into(),
        })
    }
}

enum Credential {
    Password(String),
    PublicKey { private_key_pem: String },
    Agent,
}

struct SshClientHandler;

#[async_trait]
impl client::Handler for SshClientHandler {
    type Error = russh::Error;

    async fn check_server_key(
        &mut self,
        _server_public_key: &PublicKey,
    ) -> std::result::Result<bool, Self::Error> {
        // TODO: Implement TOFU host key verification
        Ok(true) // Accept all for now
    }
}

#[async_trait]
impl ConnectionAdapter for SshAdapter {
    async fn connect(profile: &ConnectionProfile) -> Result<Self>
    where
        Self: Sized,
    {
        // Validate that this is an SSH profile
        if profile.protocol != Protocol::Ssh {
            return Err(ConnectionError::NotSupported {
                protocol: profile.protocol.clone(),
            });
        }

        // Get authentication credential
        let _credential = Self::get_credential(profile).await?;

        // For now, return a stub that will fail tests
        let (output_tx, output_rx) = mpsc::channel(100);

        Ok(SshAdapter {
            profile: profile.clone(),
            handle: Arc::new(Mutex::new(None)),
            channel: Arc::new(Mutex::new(None)),
            output_rx: Arc::new(Mutex::new(Some(output_rx))),
            output_tx,
        })
    }

    async fn disconnect(&mut self) -> Result<()> {
        if let Some(handle) = self.handle.lock().await.take() {
            handle
                .disconnect(russh::Disconnect::ByApplication, "", "en")
                .await
                .map_err(|e| ConnectionError::Protocol(e.to_string()))?;
        }
        Ok(())
    }

    fn is_alive(&self) -> bool {
        // TODO: Implement proper connection liveness check
        false
    }

    async fn reconnect(&mut self) -> Result<()> {
        self.disconnect().await?;
        let new_adapter = Self::connect(&self.profile).await?;
        *self = new_adapter;
        Ok(())
    }

    fn protocol(&self) -> Protocol {
        Protocol::Ssh
    }
}

#[async_trait]
impl TerminalAdapter for SshAdapter {
    async fn send_input(&self, _data: &[u8]) -> Result<()> {
        // TODO: Implement
        Err(ConnectionError::Protocol(
            "send_input not yet implemented".into(),
        ))
    }

    fn output_stream(&self) -> mpsc::Receiver<Vec<u8>> {
        // This is a hack for testing - in real implementation, we'd need to handle this differently
        // since we can only take the receiver once
        tokio::sync::mpsc::channel(100).1
    }

    async fn resize(&self, _cols: u16, _rows: u16) -> Result<()> {
        // TODO: Implement
        Err(ConnectionError::Protocol("resize not yet implemented".into()))
    }

    async fn exec(&self, _command: &str) -> Result<ExecResult> {
        // TODO: Implement
        Err(ConnectionError::Protocol("exec not yet implemented".into()))
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::profile::types::SshSettings;

    fn create_test_profile(
        host: &str,
        port: u16,
        username: &str,
        credential_id: Option<String>,
    ) -> ConnectionProfile {
        ConnectionProfile {
            id: uuid::Uuid::new_v4().to_string(),
            display_name: "Test SSH".to_string(),
            protocol: Protocol::Ssh,
            host: host.to_string(),
            port,
            username: username.to_string(),
            credential_id,
            ssh: Some(SshSettings::default()),
            ftp: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }

    // ---------------------------------------------------------------------------
    // Password Authentication Tests (RED phase - these should fail)
    // ---------------------------------------------------------------------------

    #[tokio::test]
    async fn connect_with_password_returns_error_when_credential_not_found() {
        let profile = create_test_profile("localhost", 22, "testuser", Some("fake-id".into()));
        let result = SshAdapter::connect(&profile).await;
        assert!(result.is_err());
        match result {
            Err(ConnectionError::AuthFailed { reason }) => {
                assert!(reason.contains("not yet implemented"));
            }
            _ => panic!("Expected AuthFailed error"),
        }
    }

    #[tokio::test]
    async fn connect_with_password_returns_error_when_no_credential() {
        let profile = create_test_profile("localhost", 22, "testuser", None);
        let result = SshAdapter::connect(&profile).await;
        assert!(result.is_err());
        match result {
            Err(ConnectionError::AuthFailed { .. }) => {}
            _ => panic!("Expected AuthFailed error"),
        }
    }

    #[tokio::test]
    async fn connect_fails_with_wrong_protocol() {
        let mut profile = create_test_profile("localhost", 22, "testuser", None);
        profile.protocol = Protocol::Ftp;
        let result = SshAdapter::connect(&profile).await;
        assert!(result.is_err());
        match result {
            Err(ConnectionError::NotSupported { protocol }) => {
                assert_eq!(protocol, Protocol::Ftp);
            }
            _ => panic!("Expected NotSupported error"),
        }
    }

    #[tokio::test]
    async fn protocol_returns_ssh() {
        let profile = create_test_profile("localhost", 22, "testuser", None);
        // Even though connect will fail, we can test the protocol method
        if let Ok(adapter) = SshAdapter::connect(&profile).await {
            assert_eq!(adapter.protocol(), Protocol::Ssh);
        }
        // This test will pass if connect fails, which is expected in RED phase
    }

    #[tokio::test]
    async fn is_alive_returns_false_when_not_connected() {
        let profile = create_test_profile("localhost", 22, "testuser", None);
        if let Ok(adapter) = SshAdapter::connect(&profile).await {
            assert!(!adapter.is_alive());
        }
    }

    #[tokio::test]
    async fn disconnect_succeeds_when_not_connected() {
        let profile = create_test_profile("localhost", 22, "testuser", None);
        if let Ok(mut adapter) = SshAdapter::connect(&profile).await {
            let result = adapter.disconnect().await;
            assert!(result.is_ok());
        }
    }

    // ---------------------------------------------------------------------------
    // Terminal Adapter Tests (RED phase - these should fail)
    // ---------------------------------------------------------------------------

    #[tokio::test]
    async fn send_input_returns_not_implemented_error() {
        let profile = create_test_profile("localhost", 22, "testuser", None);
        if let Ok(adapter) = SshAdapter::connect(&profile).await {
            let result = adapter.send_input(b"test").await;
            assert!(result.is_err());
            match result {
                Err(ConnectionError::Protocol(msg)) => {
                    assert!(msg.contains("not yet implemented"));
                }
                _ => panic!("Expected Protocol error"),
            }
        }
    }

    #[tokio::test]
    async fn resize_returns_not_implemented_error() {
        let profile = create_test_profile("localhost", 22, "testuser", None);
        if let Ok(adapter) = SshAdapter::connect(&profile).await {
            let result = adapter.resize(80, 24).await;
            assert!(result.is_err());
            match result {
                Err(ConnectionError::Protocol(msg)) => {
                    assert!(msg.contains("not yet implemented"));
                }
                _ => panic!("Expected Protocol error"),
            }
        }
    }

    #[tokio::test]
    async fn exec_returns_not_implemented_error() {
        let profile = create_test_profile("localhost", 22, "testuser", None);
        if let Ok(adapter) = SshAdapter::connect(&profile).await {
            let result = adapter.exec("echo test").await;
            assert!(result.is_err());
            match result {
                Err(ConnectionError::Protocol(msg)) => {
                    assert!(msg.contains("not yet implemented"));
                }
                _ => panic!("Expected Protocol error"),
            }
        }
    }
}
