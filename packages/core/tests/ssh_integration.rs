//! SSH adapter integration tests.
//!
//! These tests require Docker and run against a real OpenSSH server in a
//! container.  They are gated behind the `integration` feature flag:
//!
//! ```sh
//! cargo test --package tacoshell-core --features integration
//! ```

#![cfg(feature = "integration")]

use std::time::Duration;

use secrecy::SecretString;
use testcontainers::{
    core::{IntoContainerPort, WaitFor},
    runners::AsyncRunner,
    GenericImage, ImageExt,
};

use tacoshell_core::connection::ssh::{ConnectionAdapter, SshAdapter, TerminalAdapter};
use tacoshell_core::connection::Credential;
use tacoshell_core::profile::types::ConnectionProfile;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Docker image used for all SSH integration tests.
///
/// `linuxserver/openssh-server` supports environment-variable-based setup:
/// - `PASSWORD_ACCESS=true`   — enable password authentication
/// - `USER_NAME=<name>`       — create this Unix user
/// - `USER_PASSWORD=<pass>`   — password for that user
/// - `PUBLIC_KEY=<pub_key>`   — inject an authorized key
const SSHD_IMAGE: &str = "lscr.io/linuxserver/openssh-server";
/// Pinned to a specific digest for deterministic, non-breaking test runs.
/// Update this tag when intentionally upgrading the test image.
const SSHD_TAG: &str = "version-9.6_p1-r1-ls175";
const SSHD_PORT: u16 = 2222;

const TEST_USER: &str = "testuser";
const TEST_PASSWORD: &str = "correcthorsebatterystaple";

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Start an SSH server container that accepts password authentication.
async fn start_sshd_password() -> (
    testcontainers::ContainerAsync<GenericImage>,
    ConnectionProfile,
) {
    let container = GenericImage::new(SSHD_IMAGE, SSHD_TAG)
        .with_exposed_port(SSHD_PORT.tcp())
        .with_env_var("PASSWORD_ACCESS", "true")
        .with_env_var("USER_NAME", TEST_USER)
        .with_env_var("USER_PASSWORD", TEST_PASSWORD)
        // Disable strict host-key checking inside the container itself.
        .with_env_var("SUDO_ACCESS", "false")
        .with_wait_for(WaitFor::message_on_stderr("Server listening on"))
        .start()
        .await
        .expect("failed to start openssh-server container");

    let host = container
        .get_host()
        .await
        .expect("container host")
        .to_string();
    let port = container
        .get_host_port_ipv4(SSHD_PORT)
        .await
        .expect("container port");

    let profile = ConnectionProfile::new_ssh("Integration Test", host, port, TEST_USER);
    (container, profile)
}

/// Start an SSH server container that accepts a specific Ed25519 public key.
async fn start_sshd_pubkey(
    pub_key_openssh: &str,
) -> (
    testcontainers::ContainerAsync<GenericImage>,
    ConnectionProfile,
) {
    let container = GenericImage::new(SSHD_IMAGE, SSHD_TAG)
        .with_exposed_port(SSHD_PORT.tcp())
        .with_env_var("USER_NAME", TEST_USER)
        .with_env_var("PUBLIC_KEY", pub_key_openssh)
        .with_wait_for(WaitFor::message_on_stderr("Server listening on"))
        .start()
        .await
        .expect("failed to start openssh-server container");

    let host = container
        .get_host()
        .await
        .expect("container host")
        .to_string();
    let port = container
        .get_host_port_ipv4(SSHD_PORT)
        .await
        .expect("container port");

    let profile = ConnectionProfile::new_ssh("Integration Test", host, port, TEST_USER);
    (container, profile)
}

// ---------------------------------------------------------------------------
// Password authentication
// ---------------------------------------------------------------------------

#[tokio::test]
async fn ssh_password_auth_connect_succeeds() {
    let (_container, profile) = start_sshd_password().await;

    let credential = Credential::Password(SecretString::new(TEST_PASSWORD.to_owned()));

    let mut adapter = SshAdapter::connect(&profile, credential)
        .await
        .expect("should connect with correct password");

    assert!(
        adapter.is_alive(),
        "adapter should report alive after connect"
    );
    adapter.disconnect().await.expect("clean disconnect");
}

#[tokio::test]
async fn ssh_password_auth_wrong_password_fails() {
    let (_container, profile) = start_sshd_password().await;

    let credential = Credential::Password(SecretString::new("wrongpassword".to_owned()));

    let result = SshAdapter::connect(&profile, credential).await;
    assert!(result.is_err(), "connect with wrong password should fail");
    match result.unwrap_err() {
        tacoshell_core::connection::ConnectionError::AuthFailed { .. } => {}
        other => panic!("expected AuthFailed, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// Public key authentication
// ---------------------------------------------------------------------------

#[tokio::test]
async fn ssh_pubkey_auth_ed25519_succeeds() {
    use russh::keys::{key::KeyPair, PublicKeyBase64};

    let key_pair = KeyPair::generate_ed25519().expect("ed25519 keygen");
    let pub_key_base64 = key_pair.public_key_base64();
    let pub_key_openssh = format!("ssh-ed25519 {pub_key_base64}");

    let (_container, profile) = start_sshd_pubkey(&pub_key_openssh).await;

    // Encode the private key as OpenSSH PEM.
    let mut pem_buf = Vec::new();
    russh::keys::write_openssh_key_pair_to_writer(&key_pair, None, &mut pem_buf)
        .expect("write private key");
    let pem = String::from_utf8(pem_buf).expect("pem utf8");

    let credential = Credential::PublicKey {
        private_key_pem: SecretString::new(pem.into()),
        passphrase: None,
    };

    let mut adapter = SshAdapter::connect(&profile, credential)
        .await
        .expect("should connect with ed25519 key");

    assert!(adapter.is_alive());
    adapter.disconnect().await.expect("clean disconnect");
}

#[tokio::test]
async fn ssh_pubkey_auth_rsa_succeeds() {
    use russh::keys::{
        key::{KeyPair, SignatureHash},
        PublicKeyBase64,
    };

    let key_pair = KeyPair::generate_rsa(2048, SignatureHash::SHA2_256).expect("rsa keygen");
    let pub_key_base64 = key_pair.public_key_base64();
    let pub_key_openssh = format!("ssh-rsa {pub_key_base64}");

    let (_container, profile) = start_sshd_pubkey(&pub_key_openssh).await;

    let mut pem_buf = Vec::new();
    russh::keys::write_openssh_key_pair_to_writer(&key_pair, None, &mut pem_buf)
        .expect("write private key");
    let pem = String::from_utf8(pem_buf).expect("pem utf8");

    let credential = Credential::PublicKey {
        private_key_pem: SecretString::new(pem.into()),
        passphrase: None,
    };

    let mut adapter = SshAdapter::connect(&profile, credential)
        .await
        .expect("should connect with rsa key");

    assert!(adapter.is_alive());
    adapter.disconnect().await.expect("clean disconnect");
}

// ---------------------------------------------------------------------------
// TOFU host-key verification
// ---------------------------------------------------------------------------

#[tokio::test]
async fn ssh_tofu_first_connect_stores_host_key() {
    let (_container, profile) = start_sshd_password().await;
    let credential = Credential::Password(SecretString::new(TEST_PASSWORD.to_owned()));

    // First connect — should succeed and store the host key.
    let mut adapter = SshAdapter::connect(&profile, credential.clone())
        .await
        .expect("first connect should succeed");

    assert!(adapter.is_alive());
    adapter.disconnect().await.ok();
}

#[tokio::test]
async fn ssh_tofu_reconnect_with_same_host_key_succeeds() {
    let (_container, profile) = start_sshd_password().await;
    let credential = Credential::Password(SecretString::new(TEST_PASSWORD.to_owned()));

    // First connect — stores the host key in the adapter's known_keys store.
    let mut adapter = SshAdapter::connect(&profile, credential)
        .await
        .expect("first connect");

    assert!(adapter.is_alive());

    // reconnect() reuses the same known_keys store, so the host key must be
    // verified and must match — this is what validates TOFU persistence.
    adapter
        .reconnect()
        .await
        .expect("reconnect with same host key should succeed");

    assert!(
        adapter.is_alive(),
        "adapter should be alive after reconnect"
    );
    adapter.disconnect().await.ok();
}

// ---------------------------------------------------------------------------
// exec
// ---------------------------------------------------------------------------

#[tokio::test]
async fn ssh_exec_returns_command_output() {
    let (_container, profile) = start_sshd_password().await;
    let credential = Credential::Password(SecretString::new(TEST_PASSWORD.to_owned()));

    let adapter = SshAdapter::connect(&profile, credential)
        .await
        .expect("connect");

    let result = adapter.exec("echo hello").await.expect("exec");
    let stdout = result.stdout_str();
    assert!(
        stdout.trim() == "hello",
        "expected stdout 'hello', got {stdout:?}"
    );
    assert_eq!(result.exit_code, Some(0));
}

#[tokio::test]
async fn ssh_exec_nonzero_exit_code() {
    let (_container, profile) = start_sshd_password().await;
    let credential = Credential::Password(SecretString::new(TEST_PASSWORD.to_owned()));

    let adapter = SshAdapter::connect(&profile, credential)
        .await
        .expect("connect");

    let result = adapter.exec("exit 42").await.expect("exec");
    assert_eq!(result.exit_code, Some(42));
}

#[tokio::test]
async fn ssh_exec_captures_stderr() {
    let (_container, profile) = start_sshd_password().await;
    let credential = Credential::Password(SecretString::new(TEST_PASSWORD.to_owned()));

    let adapter = SshAdapter::connect(&profile, credential)
        .await
        .expect("connect");

    let result = adapter.exec("echo err >&2").await.expect("exec");
    let stderr = result.stderr_str();
    assert!(
        stderr.trim() == "err",
        "expected stderr 'err', got {stderr:?}"
    );
}

// ---------------------------------------------------------------------------
// send_input / output_stream
// ---------------------------------------------------------------------------

#[tokio::test]
async fn ssh_input_output_stream_round_trip() {
    let (_container, profile) = start_sshd_password().await;
    let credential = Credential::Password(SecretString::new(TEST_PASSWORD.to_owned()));

    let mut adapter = SshAdapter::connect(&profile, credential)
        .await
        .expect("connect");

    let mut rx = adapter.output_stream().expect("output_stream");

    // Give the shell a moment to send its prompt.
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Send a command via send_input and wait for output.
    adapter
        .send_input(b"echo roundtrip\n")
        .await
        .expect("send_input");

    // Collect output until we see the expected string (with a timeout).
    let deadline = tokio::time::Instant::now() + Duration::from_secs(5);
    let mut collected = String::new();
    loop {
        if tokio::time::Instant::now() > deadline {
            panic!("timed out waiting for 'roundtrip' in output; got: {collected:?}");
        }
        tokio::select! {
            data = rx.recv() => {
                match data {
                    Some(bytes) => {
                        collected.push_str(&String::from_utf8_lossy(&bytes));
                        if collected.contains("roundtrip") {
                            break;
                        }
                    }
                    None => panic!("output channel closed before receiving expected output"),
                }
            }
            _ = tokio::time::sleep(Duration::from_millis(50)) => {}
        }
    }
    assert!(collected.contains("roundtrip"));
}

#[tokio::test]
async fn ssh_output_stream_returns_none_on_second_call() {
    let (_container, profile) = start_sshd_password().await;
    let credential = Credential::Password(SecretString::new(TEST_PASSWORD.to_owned()));

    let mut adapter = SshAdapter::connect(&profile, credential)
        .await
        .expect("connect");

    let first = adapter.output_stream();
    let second = adapter.output_stream();

    assert!(first.is_some(), "first call should return Some");
    assert!(second.is_none(), "second call should return None");
}

// ---------------------------------------------------------------------------
// resize
// ---------------------------------------------------------------------------

#[tokio::test]
async fn ssh_resize_does_not_error() {
    let (_container, profile) = start_sshd_password().await;
    let credential = Credential::Password(SecretString::new(TEST_PASSWORD.to_owned()));

    let adapter = SshAdapter::connect(&profile, credential)
        .await
        .expect("connect");

    adapter
        .resize(120, 40)
        .await
        .expect("resize should not error on an active connection");
}

// ---------------------------------------------------------------------------
// is_alive / disconnect
// ---------------------------------------------------------------------------

#[tokio::test]
async fn ssh_is_alive_false_after_disconnect() {
    let (_container, profile) = start_sshd_password().await;
    let credential = Credential::Password(SecretString::new(TEST_PASSWORD.to_owned()));

    let mut adapter = SshAdapter::connect(&profile, credential)
        .await
        .expect("connect");

    assert!(adapter.is_alive());
    adapter.disconnect().await.expect("disconnect");

    // After disconnect the background task sets alive = false.
    // Give it a moment to propagate.
    tokio::time::sleep(Duration::from_millis(50)).await;
    assert!(
        !adapter.is_alive(),
        "is_alive should be false after disconnect"
    );
}

// ---------------------------------------------------------------------------
// Broken-pipe / channel-cleanup
// ---------------------------------------------------------------------------

/// Dropping the output receiver while the shell is running should cause the
/// background task to close the SSH channel promptly and mark the adapter dead.
/// Before the fix this passed if the server happened to send data quickly, but
/// the channel was left open on the server side (no explicit `ch.close()`).
#[tokio::test]
async fn ssh_output_pipe_break_sets_alive_false() {
    let (_container, profile) = start_sshd_password().await;
    let credential = Credential::Password(SecretString::new(TEST_PASSWORD.to_owned()));

    let mut adapter = SshAdapter::connect(&profile, credential)
        .await
        .expect("connect");

    // Take the output receiver then immediately drop it — this breaks the
    // output pipe. The server will send a shell prompt within milliseconds, so
    // the background task will attempt output_tx.send(), detect the broken
    // pipe, call ch.close(), and mark alive = false.
    let rx = adapter.output_stream().expect("output_stream");
    drop(rx);

    // Allow up to 500 ms for the background task to detect the broken pipe.
    let deadline = tokio::time::Instant::now() + Duration::from_millis(500);
    while adapter.is_alive() && tokio::time::Instant::now() < deadline {
        tokio::time::sleep(Duration::from_millis(20)).await;
    }

    assert!(
        !adapter.is_alive(),
        "adapter must be dead after output receiver is dropped and server sends data"
    );
}

/// After the output pipe breaks, send_input must return an error (not panic or
/// block) and is_alive() must be false.
#[tokio::test]
async fn ssh_send_input_after_pipe_break_returns_error_and_alive_false() {
    let (_container, profile) = start_sshd_password().await;
    let credential = Credential::Password(SecretString::new(TEST_PASSWORD.to_owned()));

    let mut adapter = SshAdapter::connect(&profile, credential)
        .await
        .expect("connect");

    // Drop output receiver to trigger background task shutdown.
    drop(adapter.output_stream());

    // Wait for the background task to exit.
    let deadline = tokio::time::Instant::now() + Duration::from_millis(500);
    while adapter.is_alive() && tokio::time::Instant::now() < deadline {
        tokio::time::sleep(Duration::from_millis(20)).await;
    }

    // send_input must fail cleanly.
    let result = adapter.send_input(b"echo test\n").await;
    assert!(
        result.is_err(),
        "send_input must fail when channel is closed"
    );

    // alive must be false at the point send_input fails.
    assert!(
        !adapter.is_alive(),
        "alive must be false when send_input fails"
    );
}

/// After the output pipe breaks, resize must also return an error and alive
/// must be false — matching the same contract as send_input.
#[tokio::test]
async fn ssh_resize_after_pipe_break_returns_error_and_alive_false() {
    let (_container, profile) = start_sshd_password().await;
    let credential = Credential::Password(SecretString::new(TEST_PASSWORD.to_owned()));

    let mut adapter = SshAdapter::connect(&profile, credential)
        .await
        .expect("connect");

    drop(adapter.output_stream());

    let deadline = tokio::time::Instant::now() + Duration::from_millis(500);
    while adapter.is_alive() && tokio::time::Instant::now() < deadline {
        tokio::time::sleep(Duration::from_millis(20)).await;
    }

    let result = adapter.resize(80, 24).await;
    assert!(result.is_err(), "resize must fail when channel is closed");
    assert!(!adapter.is_alive(), "alive must be false when resize fails");
}

// ---------------------------------------------------------------------------
// keepalive
// ---------------------------------------------------------------------------

#[tokio::test]
async fn ssh_keepalive_configured_does_not_drop_idle_connection() {
    let (_container, mut profile) = start_sshd_password().await;

    // Enable a short keepalive interval.
    if let Some(ref mut ssh) = profile.ssh {
        ssh.keepalive_secs = Some(1);
    }

    let credential = Credential::Password(SecretString::new(TEST_PASSWORD.to_owned()));

    let adapter = SshAdapter::connect(&profile, credential)
        .await
        .expect("connect with keepalive");

    // Wait for 3 keepalive cycles; the connection should remain alive.
    tokio::time::sleep(Duration::from_secs(4)).await;

    assert!(
        adapter.is_alive(),
        "connection should stay alive under keepalive"
    );
}
