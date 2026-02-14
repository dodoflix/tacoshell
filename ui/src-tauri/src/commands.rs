//! Tauri commands for IPC with the frontend

use crate::state::{ActiveSession, AppState};
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use tacoshell_core::{AuthMethod, Protocol, Secret, SecretKind, Server, ServerSecret};
use tacoshell_ssh::{PtyConfig, SshChannel, SshSession};
use tauri::{AppHandle, Emitter, State};
use uuid::Uuid;

/// Error response for commands
#[derive(Debug, Serialize)]
pub struct CommandError {
    pub message: String,
}

impl From<tacoshell_core::Error> for CommandError {
    fn from(err: tacoshell_core::Error) -> Self {
        Self {
            message: err.to_string(),
        }
    }
}

type CommandResult<T> = Result<T, CommandError>;

// ============================================================================
// Server Commands
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerResponse {
    pub id: String,
    pub name: String,
    pub host: String,
    pub port: u16,
    pub username: String,
    pub protocol: String,
    pub tags: Vec<String>,
}

impl From<Server> for ServerResponse {
    fn from(s: Server) -> Self {
        Self {
            id: s.id.to_string(),
            name: s.name,
            host: s.host,
            port: s.port,
            username: s.username,
            protocol: format!("{:?}", s.protocol).to_lowercase(),
            tags: s.tags,
        }
    }
}

#[tauri::command]
pub fn get_servers(state: State<AppState>) -> CommandResult<Vec<ServerResponse>> {
    let servers = state.db.servers().list().map_err(CommandError::from)?;
    Ok(servers.into_iter().map(ServerResponse::from).collect())
}

#[derive(Debug, Deserialize)]
pub struct AddServerRequest {
    pub name: String,
    pub host: String,
    pub port: u16,
    pub username: String,
    pub protocol: Option<String>,
    pub tags: Option<Vec<String>>,
}

#[tauri::command]
pub fn add_server(state: State<AppState>, request: AddServerRequest) -> CommandResult<ServerResponse> {
    // Input validation
    if request.name.trim().is_empty() {
        return Err(CommandError { message: "Server name cannot be empty".to_string() });
    }
    if request.name.len() > 255 {
        return Err(CommandError { message: "Server name too long (max 255 characters)".to_string() });
    }
    if request.host.trim().is_empty() {
        return Err(CommandError { message: "Host cannot be empty".to_string() });
    }
    if request.port == 0 {
        return Err(CommandError { message: "Port must be between 1 and 65535".to_string() });
    }
    if request.username.trim().is_empty() {
        return Err(CommandError { message: "Username cannot be empty".to_string() });
    }

    let mut server = Server::new(
        request.name.trim().to_string(),
        request.host.trim().to_string(),
        request.port,
        request.username.trim().to_string(),
    );

    if let Some(protocol) = request.protocol {
        server.protocol = match protocol.to_lowercase().as_str() {
            "sftp" => Protocol::Sftp,
            "ftp" => Protocol::Ftp,
            _ => Protocol::Ssh,
        };
    }

    if let Some(tags) = request.tags {
        server.tags = tags;
    }

    state.db.servers().store(&server).map_err(CommandError::from)?;
    Ok(ServerResponse::from(server))
}

#[tauri::command]
pub fn update_server(state: State<AppState>, request: ServerResponse) -> CommandResult<()> {
    let id = Uuid::parse_str(&request.id)
        .map_err(|e| CommandError { message: format!("Invalid UUID: {}", e) })?;

    let mut server = state.db.servers().get(id)
        .map_err(CommandError::from)?
        .ok_or_else(|| CommandError { message: "Server not found".to_string() })?;

    server.name = request.name;
    server.host = request.host;
    server.port = request.port;
    server.username = request.username;
    server.tags = request.tags;

    state.db.servers().update(&server).map_err(CommandError::from)?;
    Ok(())
}

#[tauri::command]
pub fn delete_server(state: State<AppState>, id: String) -> CommandResult<()> {
    let uuid = Uuid::parse_str(&id)
        .map_err(|e| CommandError { message: format!("Invalid UUID: {}", e) })?;
    state.db.servers().delete(uuid).map_err(CommandError::from)?;
    Ok(())
}

// ============================================================================
// Secret Commands
// ============================================================================

#[derive(Debug, Serialize)]
pub struct SecretResponse {
    pub id: String,
    pub name: String,
    pub kind: String,
}

impl From<Secret> for SecretResponse {
    fn from(s: Secret) -> Self {
        Self {
            id: s.id.to_string(),
            name: s.name,
            kind: format!("{:?}", s.kind).to_lowercase(),
        }
    }
}

#[tauri::command]
pub fn get_secrets(state: State<AppState>) -> CommandResult<Vec<SecretResponse>> {
    let secrets = state.db.secrets().list().map_err(CommandError::from)?;
    Ok(secrets.into_iter().map(SecretResponse::from).collect())
}

#[derive(Debug, Deserialize)]
pub struct AddSecretRequest {
    pub name: String,
    pub kind: String,
    pub value: String,
}

#[tauri::command]
pub fn add_secret(state: State<AppState>, request: AddSecretRequest) -> CommandResult<SecretResponse> {
    let kind = match request.kind.to_lowercase().as_str() {
        "private_key" | "privatekey" => SecretKind::PrivateKey,
        "token" => SecretKind::Token,
        "kubeconfig" => SecretKind::Kubeconfig,
        _ => SecretKind::Password,
    };

    let encrypted_value = state.encryption.encrypt_string(&request.value)
        .map_err(CommandError::from)?;

    let secret = Secret::new(request.name, kind, encrypted_value);
    state.db.secrets().store(&secret).map_err(CommandError::from)?;

    Ok(SecretResponse::from(secret))
}

#[tauri::command]
pub fn delete_secret(state: State<AppState>, id: String) -> CommandResult<()> {
    let uuid = Uuid::parse_str(&id)
        .map_err(|e| CommandError { message: format!("Invalid UUID: {}", e) })?;
    state.db.secrets().delete(uuid).map_err(CommandError::from)?;
    Ok(())
}

#[tauri::command(rename_all = "snake_case")]
pub fn link_secret_to_server(
    state: State<AppState>,
    server_id: String,
    secret_id: String,
    priority: Option<i32>,
) -> CommandResult<()> {
    let server_uuid = Uuid::parse_str(&server_id)
        .map_err(|e| CommandError { message: format!("Invalid server UUID: {}", e) })?;
    let secret_uuid = Uuid::parse_str(&secret_id)
        .map_err(|e| CommandError { message: format!("Invalid secret UUID: {}", e) })?;

    let link = ServerSecret::new(server_uuid, secret_uuid, priority.unwrap_or(0));
    state.db.servers().link_secret(&link).map_err(CommandError::from)?;
    Ok(())
}

#[tauri::command(rename_all = "snake_case")]
pub fn unlink_secret_from_server(
    state: State<AppState>,
    server_id: String,
    secret_id: String,
) -> CommandResult<()> {
    let server_uuid = Uuid::parse_str(&server_id)
        .map_err(|e| CommandError { message: format!("Invalid server UUID: {}", e) })?;
    let secret_uuid = Uuid::parse_str(&secret_id)
        .map_err(|e| CommandError { message: format!("Invalid secret UUID: {}", e) })?;

    state.db.servers().unlink_secret(server_uuid, secret_uuid).map_err(CommandError::from)?;
    Ok(())
}

// ============================================================================
// SSH Session Commands
// ============================================================================

#[derive(Debug, Serialize)]
pub struct SessionResponse {
    pub session_id: String,
    pub server_id: String,
    pub connected: bool,
}

#[derive(Debug, Deserialize)]
pub struct ConnectRequest {
    pub server_id: String,
    /// Optional direct password (if not using stored secret)
    pub password: Option<String>,
    /// Optional direct private key (if not using stored secret)
    pub private_key: Option<String>,
    /// Passphrase for private key
    pub passphrase: Option<String>,
}

/// SSH output event payload
#[derive(Clone, Serialize)]
pub struct SshOutputEvent {
    pub session_id: String,
    pub data: String,
    pub eof: bool,
}

#[tauri::command]
pub fn connect_ssh(
    app: AppHandle,
    state: State<AppState>,
    request: ConnectRequest,
) -> CommandResult<SessionResponse> {
    let server_uuid = Uuid::parse_str(&request.server_id)
        .map_err(|e| CommandError { message: format!("Invalid server UUID: {}", e) })?;

    let server = state.db.servers().get(server_uuid)
        .map_err(CommandError::from)?
        .ok_or_else(|| CommandError { message: "Server not found".to_string() })?;

    // Determine authentication method
    let auth = if let Some(password) = request.password {
        AuthMethod::Password(password)
    } else if let Some(key) = request.private_key {
        AuthMethod::PrivateKey {
            key,
            passphrase: request.passphrase,
        }
    } else {
        // Try to get secrets linked to this server
        let secrets = state.db.servers().get_secrets(server_uuid)
            .map_err(CommandError::from)?;

        if let Some(secret) = secrets.first() {
            let decrypted = state.encryption.decrypt(&secret.encrypted_value)
                .map_err(CommandError::from)?;
            let value = String::from_utf8(decrypted)
                .map_err(|e| CommandError { message: format!("Invalid secret encoding: {}", e) })?;

            match secret.kind {
                SecretKind::Password => AuthMethod::Password(value),
                SecretKind::PrivateKey => AuthMethod::PrivateKey {
                    key: value,
                    passphrase: None,
                },
                _ => return Err(CommandError { message: "Unsupported secret type for SSH".to_string() }),
            }
        } else {
            // Try SSH agent as fallback
            AuthMethod::Agent
        }
    };

    // Connect
    let ssh_session = SshSession::connect(&server, auth)
        .map_err(CommandError::from)?;

    // Open channel with PTY
    let channel = ssh_session.open_channel()
        .map_err(CommandError::from)?;

    let mut ssh_channel = SshChannel::new(channel);
    ssh_channel.request_pty(&PtyConfig::default())
        .map_err(CommandError::from)?;
    ssh_channel.shell()
        .map_err(CommandError::from)?;

    // Keep in blocking mode but set a short timeout for reads
    // This allows us to periodically check the stop flag
    ssh_session.set_timeout(100); // 100ms timeout

    let session_id = Uuid::new_v4();
    let session_id_str = session_id.to_string();
    let stop_flag = Arc::new(AtomicBool::new(false));

    // Store the session
    state.add_session(session_id, ActiveSession {
        session: ssh_session,
        channel: Some(ssh_channel),
        stop_flag: stop_flag.clone(),
    });

    // Spawn background thread to read output and emit events
    let app_handle = app.clone();
    let session_id_for_thread = session_id_str.clone();
    let sessions = state.sessions.clone();

    thread::spawn(move || {
        let mut buf = vec![0u8; 8192];

        loop {
            // Check if we should stop
            if stop_flag.load(Ordering::Relaxed) {
                break;
            }

            // Try to read from the channel - hold lock briefly
            let read_result = {
                let mut sessions_guard = sessions.lock().unwrap();
                if let Some(session) = sessions_guard.get_mut(&session_id) {
                    if let Some(channel) = &mut session.channel {
                        // Blocking read with timeout
                        match channel.read(&mut buf) {
                            Ok(n) if n > 0 => Some((buf[..n].to_vec(), channel.eof())),
                            Ok(_) => None, // Timeout or no data
                            Err(_) => None, // Timeout or error
                        }
                    } else {
                        None
                    }
                } else {
                    // Session was removed
                    break;
                }
            };
            // Lock is released here

            if let Some((data, eof)) = read_result {
                let output = String::from_utf8_lossy(&data).to_string();
                if !output.is_empty() {
                    let _ = app_handle.emit("ssh-output", SshOutputEvent {
                        session_id: session_id_for_thread.clone(),
                        data: output,
                        eof,
                    });
                }

                if eof {
                    break;
                }
            }

            // Small sleep to prevent busy-waiting on timeout
            thread::sleep(Duration::from_millis(10));
        }
    });

    Ok(SessionResponse {
        session_id: session_id_str,
        server_id: request.server_id,
        connected: true,
    })
}

#[tauri::command(rename_all = "snake_case")]
pub fn disconnect_ssh(state: State<AppState>, session_id: String) -> CommandResult<()> {
    let uuid = Uuid::parse_str(&session_id)
        .map_err(|e| CommandError { message: format!("Invalid session UUID: {}", e) })?;

    if let Some(mut session) = state.remove_session(&uuid) {
        // Signal the background thread to stop
        session.stop_flag.store(true, Ordering::Relaxed);

        if let Some(mut channel) = session.channel.take() {
            let _ = channel.close();
        }
    }

    Ok(())
}

/// Send input to an SSH session (output is received via ssh-output events)
#[tauri::command(rename_all = "snake_case")]
pub fn send_ssh_input(
    state: State<AppState>,
    session_id: String,
    input: String,
) -> CommandResult<()> {
    if input.is_empty() {
        return Ok(());
    }

    tracing::debug!("send_ssh_input called: session={}, input_len={}", session_id, input.len());

    let uuid = Uuid::parse_str(&session_id)
        .map_err(|e| CommandError { message: format!("Invalid session UUID: {}", e) })?;

    let result = state.with_session(&uuid, |active_session| {
        if let Some(channel) = &mut active_session.channel {
            tracing::debug!("Writing {} bytes to channel", input.len());

            // Use write_all for reliable writes (session is in blocking mode with timeout)
            channel.write_all(input.as_bytes())?;
            let _ = channel.flush();

            tracing::debug!("Write completed successfully");
            Ok(())
        } else {
            tracing::error!("No active channel for session {}", session_id);
            Err(tacoshell_core::Error::Session("No active channel".to_string()))
        }
    });

    match &result {
        Some(Ok(_)) => tracing::debug!("Input sent successfully"),
        Some(Err(e)) => tracing::error!("Error sending input: {}", e),
        None => tracing::error!("Session not found: {}", session_id),
    }

    result
        .ok_or_else(|| CommandError { message: "Session not found".to_string() })?
        .map_err(CommandError::from)
}

#[tauri::command(rename_all = "snake_case")]
pub fn resize_terminal(
    state: State<AppState>,
    session_id: String,
    cols: u32,
    rows: u32,
) -> CommandResult<()> {
    let uuid = Uuid::parse_str(&session_id)
        .map_err(|e| CommandError { message: format!("Invalid session UUID: {}", e) })?;

    let result = state.with_session(&uuid, |session| {
        if let Some(channel) = &mut session.channel {
            channel.resize(cols, rows)
        } else {
            Err(tacoshell_core::Error::Session("No active channel".to_string()))
        }
    });

    result
        .ok_or_else(|| CommandError { message: "Session not found".to_string() })?
        .map_err(CommandError::from)
}




