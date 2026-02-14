//! Kubernetes data models

use serde::{Deserialize, Serialize};

/// Kubernetes cluster configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterConfig {
    pub name: String,
    pub context: String,
    pub kubeconfig_path: Option<String>,
}

/// Simplified pod information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PodInfo {
    pub name: String,
    pub namespace: String,
    pub status: String,
    pub ready: bool,
    pub containers: Vec<ContainerInfo>,
}

/// Container information within a pod
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerInfo {
    pub name: String,
    pub image: String,
    pub ready: bool,
    pub restart_count: i32,
}

/// Namespace information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NamespaceInfo {
    pub name: String,
    pub status: String,
}

