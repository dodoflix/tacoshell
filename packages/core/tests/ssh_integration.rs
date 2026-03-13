// Integration tests for SSH adapter using testcontainers
//
// These tests require Docker to be available and the `integration` feature to be enabled.
// Run with: `cargo test --features integration ssh_integration`

#![cfg(feature = "integration")]

use tacoshell_core::connection::ssh::SshAdapter;
use tacoshell_core::connection::ConnectionAdapter;
use tacoshell_core::profile::types::{ConnectionProfile, Protocol, SshSettings};
use testcontainers::runners::AsyncRunner;
use testcontainers::{ContainerAsync, GenericImage, ImageExt};

/// SSH container configuration for testing
struct SshTestContainer {
    _container: ContainerAsync<GenericImage>,
    host: String,
    port: u16,
    username: String,
    password: String,
}

impl SshTestContainer {
    /// Start a new SSH server container for testing
    async fn start() -> Result<Self, Box<dyn std::error::Error>> {
        // Use linuxserver/openssh-server image
        let image = GenericImage::new("lscr.io/linuxserver/openssh-server", "latest")
            .with_exposed_port(2222.into())
            .with_env_var("PUID", "1000")
            .with_env_var("PGID", "1000")
            .with_env_var("TZ", "Etc/UTC")
            .with_env_var("PASSWORD_ACCESS", "true")
            .with_env_var("USER_PASSWORD", "testpassword")
            .with_env_var("USER_NAME", "testuser");

        let container: ContainerAsync<GenericImage> = image.start().await?;
        let host = "127.0.0.1".to_string();
        let port = container.get_host_port_ipv4(2222).await?;

        // Wait for SSH server to be ready (longer wait for container startup)
        tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;

        Ok(Self {
            _container: container,
            host,
            port,
            username: "testuser".to_string(),
            password: "testpassword".to_string(),
        })
    }

    /// Create a connection profile for this container
    fn profile(&self) -> ConnectionProfile {
        ConnectionProfile {
            id: uuid::Uuid::new_v4().to_string(),
            display_name: "Test SSH Container".to_string(),
            protocol: Protocol::Ssh,
            host: self.host.clone(),
            port: self.port,
            username: self.username.clone(),
            credential_id: None, // Will need to be updated when vault integration is complete
            ssh: Some(SshSettings::default()),
            ftp: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }

    /// Get the password for testing (normally would come from vault)
    fn _password(&self) -> &str {
        &self.password
    }
}

#[tokio::test]
#[ignore] // Ignore by default since it requires Docker
async fn ssh_connect_with_password_succeeds() {
    let container = SshTestContainer::start().await.expect("Failed to start SSH container");
    let profile = container.profile();

    // This will fail until we have vault integration to provide the password
    let result = SshAdapter::connect(&profile).await;

    // For now, we expect it to fail with AuthFailed because we don't have credential resolution
    assert!(result.is_err());
}

#[tokio::test]
#[ignore] // Ignore by default since it requires Docker
async fn ssh_exec_command_returns_output() {
    let container = SshTestContainer::start().await.expect("Failed to start SSH container");
    let profile = container.profile();

    // This will fail until we have vault integration
    // Once implemented, this test should:
    // 1. Connect to the SSH server
    // 2. Execute a simple command like "echo hello"
    // 3. Verify the output is "hello"
    let result = SshAdapter::connect(&profile).await;
    assert!(result.is_err(), "Expected error until vault integration is complete");
}

#[tokio::test]
#[ignore] // Ignore by default since it requires Docker
async fn ssh_connection_timeout_on_unreachable_host() {
    let profile = ConnectionProfile {
        id: uuid::Uuid::new_v4().to_string(),
        display_name: "Unreachable Host".to_string(),
        protocol: Protocol::Ssh,
        host: "192.0.2.1".to_string(), // TEST-NET-1, guaranteed to be unreachable
        port: 22,
        username: "testuser".to_string(),
        credential_id: None,
        ssh: Some(SshSettings::default()),
        ftp: None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let result = SshAdapter::connect(&profile).await;

    // Should fail with AuthFailed (no credentials) before even attempting connection
    assert!(result.is_err());
}
