use serde::{Deserialize, Serialize};
use tacoshell_core::crypto::kdf;
use tacoshell_core::crypto::vault::{EncryptedItem, VaultFile};
use tacoshell_core::storage::cache::FileCache;
use tacoshell_core::storage::github::GitHubClient;
use tacoshell_core::storage::sync::SyncEngine;

#[derive(Debug, Serialize, Deserialize)]
pub struct VaultItem {
    pub id: String,
    pub r#type: String,
    pub payload: serde_json::Value,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoadVaultResult {
    pub items: Vec<VaultItem>,
    pub sha: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SaveVaultResult {
    pub sha: String,
}

/// Derive master key from passphrase + GitHub user ID as salt.
fn derive_master_key(passphrase: &str, github_user_id: &str) -> Result<[u8; 32], String> {
    let key =
        kdf::derive_master_key(passphrase.as_bytes(), github_user_id).map_err(|e| e.to_string())?;
    Ok(*key)
}

#[tauri::command]
pub async fn create_vault(
    token: String,
    passphrase: String,
    github_user_id: String,
) -> Result<(), String> {
    let master_key = derive_master_key(&passphrase, &github_user_id)?;
    let github = GitHubClient::new(&token).map_err(|e| e.to_string())?;
    let cache = FileCache::new().map_err(|e| e.to_string())?;
    let engine = SyncEngine::new(github, cache, &github_user_id);
    let _ = master_key; // key is derived but not used during init (vault is empty)
    engine
        .init_vault()
        .await
        .map(|_| ())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn load_vault(
    token: String,
    passphrase: String,
    github_user_id: String,
) -> Result<LoadVaultResult, String> {
    let master_key = derive_master_key(&passphrase, &github_user_id)?;
    let github = GitHubClient::new(&token).map_err(|e| e.to_string())?;
    let cache = FileCache::new().map_err(|e| e.to_string())?;
    let engine = SyncEngine::new(github, cache, &github_user_id);
    let result = engine.load().await.map_err(|e| e.to_string())?;

    // Decrypt each item and convert to frontend VaultItem format.
    let items = result
        .vault
        .items
        .iter()
        .filter_map(|encrypted_item| {
            let payload_bytes = encrypted_item.decrypt(&master_key).ok()?;
            let payload: serde_json::Value = serde_json::from_slice(&payload_bytes).ok()?;
            let type_str = payload["type"].as_str()?.to_string();
            Some(VaultItem {
                id: encrypted_item.id.clone(),
                r#type: type_str,
                payload: payload.clone(),
                created_at: encrypted_item.created_at.to_rfc3339(),
                updated_at: encrypted_item.updated_at.to_rfc3339(),
            })
        })
        .collect();

    Ok(LoadVaultResult {
        items,
        sha: result.sha,
    })
}

#[tauri::command]
pub async fn save_vault(
    token: String,
    passphrase: String,
    github_user_id: String,
    items: Vec<VaultItem>,
    current_sha: String,
) -> Result<SaveVaultResult, String> {
    let master_key = derive_master_key(&passphrase, &github_user_id)?;
    let github = GitHubClient::new(&token).map_err(|e| e.to_string())?;
    let cache = FileCache::new().map_err(|e| e.to_string())?;
    let engine = SyncEngine::new(github, cache, &github_user_id);

    // Build VaultFile from decrypted frontend items.
    let mut vault = VaultFile::new();
    for item in &items {
        let payload_bytes = serde_json::to_vec(&item.payload).map_err(|e| e.to_string())?;
        let encrypted = EncryptedItem::encrypt_with_id(&master_key, &item.id, &payload_bytes)
            .map_err(|e| e.to_string())?;
        vault.items.push(encrypted);
    }

    let new_sha = engine
        .push(&vault, &current_sha)
        .await
        .map_err(|e| e.to_string())?;

    Ok(SaveVaultResult { sha: new_sha })
}
