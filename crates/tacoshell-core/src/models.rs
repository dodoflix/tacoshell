//! Core data models for Tacoshell

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Type of secret stored in the system
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SecretKind {
    /// Plain text password
    Password,
    /// SSH private key (PEM or OpenSSH format)
    PrivateKey,
    /// API token or bearer token
    Token,
    /// Kubernetes kubeconfig
    Kubeconfig,
}

/// A secret credential that can be applied to servers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Secret {
    pub id: Uuid,
    pub name: String,
    pub kind: SecretKind,
    /// Optional username (used for Password kind)
    pub username: Option<String>,
    /// Encrypted value - decrypted only when needed
    #[serde(skip_serializing)]
    pub encrypted_value: Vec<u8>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Secret {
    pub fn new(name: String, kind: SecretKind, encrypted_value: Vec<u8>, username: Option<String>) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name,
            kind,
            username,
            encrypted_value,
            created_at: now,
            updated_at: now,
        }
    }
}

/// Protocol type for server connections
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum Protocol {
    #[default]
    Ssh,
    Sftp,
    Ftp,
}

/// A server/host configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Server {
    pub id: Uuid,
    pub name: String,
    pub host: String,
    pub port: u16,
    pub username: String,
    pub protocol: Protocol,
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Server {
    pub fn new(name: String, host: String, port: u16, username: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name,
            host,
            port,
            username,
            protocol: Protocol::default(),
            tags: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }

    /// Returns the connection address as "host:port"
    pub fn address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

/// Junction model linking servers to secrets (many-to-many)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerSecret {
    pub server_id: Uuid,
    pub secret_id: Uuid,
    /// Priority for fallback ordering (lower = higher priority)
    pub priority: i32,
}

impl ServerSecret {
    pub fn new(server_id: Uuid, secret_id: Uuid, priority: i32) -> Self {
        Self {
            server_id,
            secret_id,
            priority,
        }
    }
}

/// Connection status for sessions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ConnectionStatus {
    Disconnected,
    Connecting,
    Connected,
    Error(String),
}

/// Represents an active session (SSH, SFTP, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {
    pub id: Uuid,
    pub server_id: Uuid,
    pub protocol: Protocol,
    pub status: ConnectionStatus,
    pub started_at: DateTime<Utc>,
}

impl SessionInfo {
    pub fn new(server_id: Uuid, protocol: Protocol) -> Self {
        Self {
            id: Uuid::new_v4(),
            server_id,
            protocol,
            status: ConnectionStatus::Disconnected,
            started_at: Utc::now(),
        }
    }
}

