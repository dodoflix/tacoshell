//! Tauri commands for IPC with the frontend

use crate::state::{ActiveSession, AppState, SshInput};
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use tacoshell_core::{AuthMethod, Protocol, Secret, SecretKind, Server, ServerSecret};
use tacoshell_ssh::{PtyConfig, SshChannel, SshSession};
use tauri::{AppHandle, Emitter, State};
use tokio::sync::mpsc;
use uuid::Uuid;

/// Error response for commands
#[derive(Debug, Serialize)]
pub struct CommandError {
    pub message: String,
    pub code: Option<String>,
}

impl From<tacoshell_core::Error> for CommandError {
    fn from(err: tacoshell_core::Error) -> Self {
        let code = match &err {
            tacoshell_core::Error::Connection(_) => Some("CONNECTION_ERROR".to_string()),
            tacoshell_core::Error::Authentication(_) => Some("AUTH_FAILED".to_string()),
            tacoshell_core::Error::Session(_) => Some("SESSION_ERROR".to_string()),
            tacoshell_core::Error::Database(_) => Some("DATABASE_ERROR".to_string()),
            tacoshell_core::Error::Secret(_) => Some("SECRET_ERROR".to_string()),
            _ => None,
        };

        Self {
            message: err.to_string(),
            code,
        }
    }
}

pub type CommandResult<T> = Result<T, CommandError>;

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
        return Err(CommandError { message: "Server name cannot be empty".to_string(), code: None });
    }
    if request.name.len() > 255 {
        return Err(CommandError { message: "Server name too long (max 255 characters)".to_string(), code: None });
    }
    if request.host.trim().is_empty() {
        return Err(CommandError { message: "Host cannot be empty".to_string(), code: None });
    }
    if request.port == 0 {
        return Err(CommandError { message: "Port must be between 1 and 65535".to_string(), code: None });
    }
    if request.username.trim().is_empty() {
        return Err(CommandError { message: "Username cannot be empty".to_string(), code: None });
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
        .map_err(|e| CommandError { message: format!("Invalid UUID: {}", e), code: Some("INVALID_UUID".into()) })?;

    let mut server = state.db.servers().get(id)
        .map_err(CommandError::from)?
        .ok_or_else(|| CommandError { message: "Server not found".to_string(), code: Some("NOT_FOUND".into()) })?;

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
        .map_err(|e| CommandError { message: format!("Invalid UUID: {}", e), code: Some("INVALID_UUID".into()) })?;
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
        .map_err(|e| CommandError { message: format!("Invalid UUID: {}", e), code: Some("INVALID_UUID".into()) })?;
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
        .map_err(|e| CommandError { message: format!("Invalid server UUID: {}", e), code: Some("INVALID_UUID".into()) })?;
    let secret_uuid = Uuid::parse_str(&secret_id)
        .map_err(|e| CommandError { message: format!("Invalid secret UUID: {}", e), code: Some("INVALID_UUID".into()) })?;

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
        .map_err(|e| CommandError { message: format!("Invalid server UUID: {}", e), code: Some("INVALID_UUID".into()) })?;
    let secret_uuid = Uuid::parse_str(&secret_id)
        .map_err(|e| CommandError { message: format!("Invalid secret UUID: {}", e), code: Some("INVALID_UUID".into()) })?;

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
        .map_err(|e| CommandError { message: format!("Invalid server UUID: {}", e), code: Some("INVALID_UUID".into()) })?;

    let server = state.db.servers().get(server_uuid)
        .map_err(CommandError::from)?
        .ok_or_else(|| CommandError { message: "Server not found".to_string(), code: Some("NOT_FOUND".into()) })?;

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
                .map_err(|e| CommandError { message: format!("Invalid secret encoding: {}", e), code: Some("DECODE_ERROR".into()) })?;

            match secret.kind {
                SecretKind::Password => AuthMethod::Password(value),
                SecretKind::PrivateKey => AuthMethod::PrivateKey {
                    key: value,
                    passphrase: None,
                },
                _ => return Err(CommandError { message: "Unsupported secret type for SSH".to_string(), code: Some("UNSUPPORTED_SECRET".into()) }),
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
    ssh_session.set_timeout(10); // 10ms timeout

    let session_id = Uuid::new_v4();
    let session_id_str = session_id.to_string();
    let stop_flag = Arc::new(AtomicBool::new(false));
    let (input_tx, mut input_rx) = mpsc::unbounded_channel::<SshInput>();

    // Store the session
    state.add_session(session_id, ActiveSession {
        input_tx,
        stop_flag: stop_flag.clone(),
    });

    // Spawn background thread to read output and emit events
    let app_handle = app.clone();
    let session_id_for_thread = session_id_str.clone();

    thread::spawn(move || {
        let mut buf = vec![0u8; 8192];
        let mut ssh_channel = ssh_channel;
        let _ssh_session = ssh_session; // Keep session alive

        loop {
            // Check if we should stop
            if stop_flag.load(Ordering::Relaxed) {
                break;
            }

            // Handle all pending inputs immediately for better responsiveness
            let mut inputs_processed = 0;
            while let Ok(input) = input_rx.try_recv() {
                match input {
                    SshInput::Data(data) => {
                        let _ = ssh_channel.write_all(data.as_bytes());
                        let _ = ssh_channel.flush();
                    }
                    SshInput::Resize { cols, rows } => {
                        let _ = ssh_channel.resize(cols, rows);
                    }
                    SshInput::Disconnect => {
                        stop_flag.store(true, Ordering::Relaxed);
                        break;
                    }
                }
                inputs_processed += 1;
                // Don't stay in input loop forever if there's a flood
                if inputs_processed > 50 {
                    break;
                }
            }

            if stop_flag.load(Ordering::Relaxed) {
                break;
            }

            // Blocking read with timeout
            match ssh_channel.read(&mut buf) {
                Ok(n) if n > 0 => {
                    let output = String::from_utf8_lossy(&buf[..n]).to_string();
                    if !output.is_empty() {
                        let _ = app_handle.emit("ssh-output", SshOutputEvent {
                            session_id: session_id_for_thread.clone(),
                            data: output,
                            eof: ssh_channel.eof(),
                        });
                    }

                    if ssh_channel.eof() {
                        break;
                    }
                }
                _ => {
                    // Timeout, no data, or error
                    // Small sleep to prevent busy-waiting if no data and no input
                    thread::sleep(Duration::from_millis(5));
                }
            }
        }

        // Cleanup
        let _ = ssh_channel.close();
        let _ = app_handle.emit("ssh-output", SshOutputEvent {
            session_id: session_id_for_thread,
            data: "".to_string(),
            eof: true,
        });
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
        .map_err(|e| CommandError { message: format!("Invalid session UUID: {}", e), code: Some("INVALID_UUID".into()) })?;

    if let Some(session) = state.remove_session(&uuid) {
        // Signal the background thread to stop
        let _ = session.input_tx.send(SshInput::Disconnect);
        session.stop_flag.store(true, Ordering::Relaxed);
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
        .map_err(|e| CommandError { message: format!("Invalid session UUID: {}", e), code: Some("INVALID_UUID".into()) })?;

    state.with_session(&uuid, |active_session| {
        let _ = active_session.input_tx.send(SshInput::Data(input));
    }).ok_or_else(|| CommandError { message: "Session not found".to_string(), code: Some("NOT_FOUND".into()) })?;

    Ok(())
}

#[tauri::command(rename_all = "snake_case")]
pub fn resize_terminal(
    state: State<AppState>,
    session_id: String,
    cols: u32,
    rows: u32,
) -> CommandResult<()> {
    let uuid = Uuid::parse_str(&session_id)
        .map_err(|e| CommandError { message: format!("Invalid session UUID: {}", e), code: Some("INVALID_UUID".into()) })?;

    state.with_session(&uuid, |session| {
        let _ = session.input_tx.send(SshInput::Resize { cols, rows });
    }).ok_or_else(|| CommandError { message: "Session not found".to_string(), code: Some("NOT_FOUND".into()) })?;

    Ok(())
}




