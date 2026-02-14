//! SSH session management

use ssh2::Session;
use std::net::TcpStream;
use std::path::Path;
use std::sync::Arc;
use tacoshell_core::{AuthMethod, Error, Result, Server};
use tracing::{debug, info};

/// Represents an active SSH session
pub struct SshSession {
    session: Arc<Session>,
    server: Server,
}

impl SshSession {
    /// Connect to a server with the given authentication method
    pub fn connect(server: &Server, auth: AuthMethod) -> Result<Self> {
        let addr = server.address();
        info!("Connecting to {}", addr);

        // Establish TCP connection
        let tcp = TcpStream::connect(&addr)
            .map_err(|e| Error::Connection(format!("Failed to connect to {}: {}", addr, e)))?;

        // Set non-blocking for async compatibility
        tcp.set_nonblocking(false)
            .map_err(|e| Error::Connection(format!("Failed to set blocking mode: {}", e)))?;

        // Create SSH session
        let mut session = Session::new()
            .map_err(|e| Error::Session(format!("Failed to create session: {}", e)))?;

        session.set_tcp_stream(tcp);
        session
            .handshake()
            .map_err(|e| Error::Connection(format!("SSH handshake failed: {}", e)))?;

        debug!("Handshake complete, authenticating...");

        // Authenticate
        match auth {
            AuthMethod::Password(password) => {
                session
                    .userauth_password(&server.username, &password)
                    .map_err(|e| Error::Authentication(format!("Password auth failed: {}", e)))?;
            }
            AuthMethod::PrivateKey { key, passphrase } => {
                // Try to parse as a path first, otherwise treat as key content
                let key_path = Path::new(&key);
                if key_path.exists() {
                    session
                        .userauth_pubkey_file(
                            &server.username,
                            None,
                            key_path,
                            passphrase.as_deref(),
                        )
                        .map_err(|e| Error::Authentication(format!("Key auth failed: {}", e)))?;
                } else {
                    // Key content provided directly - use secure temp file with restricted permissions
                    // Note: ssh2-rs doesn't have userauth_pubkey_memory, so we use a secure temp approach
                    let temp_dir = std::env::temp_dir();
                    let temp_key_path = temp_dir.join(format!(".tacoshell_key_{}", uuid::Uuid::new_v4()));

                    // Write key with restricted permissions
                    #[cfg(unix)]
                    {
                        use std::os::unix::fs::OpenOptionsExt;
                        std::fs::OpenOptions::new()
                            .write(true)
                            .create_new(true)
                            .mode(0o600)
                            .open(&temp_key_path)
                            .and_then(|mut f| std::io::Write::write_all(&mut f, key.as_bytes()))
                            .map_err(|e| Error::Authentication(format!("Failed to write temp key: {}", e)))?;
                    }
                    #[cfg(windows)]
                    {
                        std::fs::write(&temp_key_path, &key)
                            .map_err(|e| Error::Authentication(format!("Failed to write temp key: {}", e)))?;
                    }

                    let result = session
                        .userauth_pubkey_file(
                            &server.username,
                            None,
                            &temp_key_path,
                            passphrase.as_deref(),
                        );

                    // Securely clean up temp file - overwrite before delete
                    let key_len = key.len();
                    let _ = std::fs::write(&temp_key_path, vec![0u8; key_len]);
                    let _ = std::fs::remove_file(&temp_key_path);

                    result.map_err(|e| Error::Authentication(format!("Key auth failed: {}", e)))?;
                }
            }
            AuthMethod::Agent => {
                let mut agent = session
                    .agent()
                    .map_err(|e| Error::Authentication(format!("Failed to get SSH agent: {}", e)))?;

                agent
                    .connect()
                    .map_err(|e| Error::Authentication(format!("Failed to connect to agent: {}", e)))?;

                agent
                    .list_identities()
                    .map_err(|e| Error::Authentication(format!("Failed to list identities: {}", e)))?;

                let mut authenticated = false;
                for identity in agent.identities().unwrap() {
                    if agent.userauth(&server.username, &identity).is_ok() {
                        authenticated = true;
                        break;
                    }
                }

                if !authenticated {
                    return Err(Error::Authentication("No valid identity found in agent".into()));
                }
            }
        }

        if !session.authenticated() {
            return Err(Error::Authentication("Authentication failed".into()));
        }

        info!("Successfully connected to {}", addr);

        Ok(Self {
            session: Arc::new(session),
            server: server.clone(),
        })
    }

    /// Check if the session is still authenticated
    pub fn is_authenticated(&self) -> bool {
        self.session.authenticated()
    }

    /// Get the server this session is connected to
    pub fn server(&self) -> &Server {
        &self.server
    }

    /// Get a reference to the underlying SSH session
    pub fn inner(&self) -> &Session {
        &self.session
    }

    /// Set the session blocking mode
    pub fn set_blocking(&self, blocking: bool) {
        self.session.set_blocking(blocking);
    }

    /// Set the session timeout in milliseconds (0 = no timeout)
    pub fn set_timeout(&self, timeout_ms: u32) {
        self.session.set_timeout(timeout_ms);
    }

    /// Open a new channel for executing commands or starting a shell
    pub fn open_channel(&self) -> Result<ssh2::Channel> {
        self.session
            .channel_session()
            .map_err(|e| Error::Session(format!("Failed to open channel: {}", e)))
    }

    /// Execute a single command and return the output
    pub fn exec(&self, command: &str) -> Result<String> {
        let mut channel = self.open_channel()?;

        channel
            .exec(command)
            .map_err(|e| Error::Session(format!("Failed to execute command: {}", e)))?;

        let mut output = String::new();
        use std::io::Read;
        channel
            .read_to_string(&mut output)
            .map_err(|e| Error::Session(format!("Failed to read output: {}", e)))?;

        channel
            .wait_close()
            .map_err(|e| Error::Session(format!("Failed to close channel: {}", e)))?;

        Ok(output)
    }

    /// Open an SFTP subsystem
    pub fn sftp(&self) -> Result<ssh2::Sftp> {
        self.session
            .sftp()
            .map_err(|e| Error::Session(format!("Failed to open SFTP: {}", e)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_address() {
        let server = Server::new(
            "Test".to_string(),
            "192.168.1.1".to_string(),
            22,
            "root".to_string(),
        );
        assert_eq!(server.address(), "192.168.1.1:22");
    }
}

