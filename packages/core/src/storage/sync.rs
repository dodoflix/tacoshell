use std::collections::HashMap;

use chrono::Utc;
use tracing::{instrument, warn};

use crate::crypto::vault::{EncryptedItem, VaultFile};
use crate::storage::{
    cache::{Cache, CacheEntry},
    github::GitHubStorage,
    StorageError,
};

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// Where the loaded vault data came from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoadSource {
    /// Data was fetched live from GitHub.
    GitHub,
    /// Data was loaded from the local cache (offline mode).
    Cache,
}

/// Result of `SyncEngine::load`.
#[derive(Debug)]
pub struct LoadResult {
    /// The parsed vault file.
    pub vault: VaultFile,
    /// The GitHub blob SHA associated with this vault version.
    pub sha: String,
    /// Where the data came from.
    pub source: LoadSource,
}

/// A vault item that could not be automatically resolved during merge.
///
/// Both `local` and `remote` share the same `id` and the same `updated_at`
/// timestamp but have different ciphertext, meaning two devices made
/// conflicting changes at the exact same instant.
#[derive(Debug, Clone)]
pub struct ConflictItem {
    pub id: String,
    pub local: EncryptedItem,
    pub remote: EncryptedItem,
}

/// The result of merging two vault snapshots.
#[derive(Debug)]
pub struct MergeResult {
    /// The merged vault, ready to push back to GitHub.
    ///
    /// When there are conflicts, the local version of the conflicted item is
    /// kept as a provisional winner until the user resolves it in the UI.
    pub merged: VaultFile,
    /// Items that could not be automatically resolved (same id, same timestamp,
    /// different ciphertext). These must be surfaced to the user.
    pub conflicts: Vec<ConflictItem>,
}

// ---------------------------------------------------------------------------
// SyncEngine
// ---------------------------------------------------------------------------

/// Orchestrates vault fetch, push, offline fallback, and conflict resolution.
///
/// Generic over `G: GitHubStorage` and `C: Cache` so the engine is fully
/// testable with mockall mocks — no network required.
pub struct SyncEngine<G, C>
where
    G: GitHubStorage,
    C: Cache,
{
    github: G,
    cache: C,
    /// GitHub username of the authenticated user — the vault repo owner.
    owner: String,
}

impl<G: GitHubStorage, C: Cache> SyncEngine<G, C> {
    pub fn new(github: G, cache: C, owner: impl Into<String>) -> Self {
        SyncEngine {
            github,
            cache,
            owner: owner.into(),
        }
    }

    /// Load the vault.
    ///
    /// 1. Fetch `vault.json` from GitHub.
    /// 2. On success: update the local cache, return the vault with `source = GitHub`.
    /// 3. On network/offline error: fall back to the local cache, `source = Cache`.
    /// 4. On a hard error (auth, parse, etc.): propagate the error.
    #[instrument(skip(self), fields(owner = %self.owner))]
    pub async fn load(&self) -> Result<LoadResult, StorageError> {
        match self.github.read_file(&self.owner, "vault.json").await {
            Ok(Some(file)) => {
                let json_str = std::str::from_utf8(&file.content)
                    .map_err(|e| StorageError::GitHub(format!("vault.json is not UTF-8: {e}")))?;
                let vault = VaultFile::from_json(json_str)?;

                // Update the local cache with the freshly-fetched version.
                self.cache
                    .store(&CacheEntry {
                        vault_bytes: file.content.clone(),
                        sha: file.sha.clone(),
                        cached_at: Utc::now(),
                    })
                    .await?;

                Ok(LoadResult {
                    vault,
                    sha: file.sha,
                    source: LoadSource::GitHub,
                })
            }
            Ok(None) => Err(StorageError::RepoNotFound),
            Err(e) if is_offline_error(&e) => {
                warn!("GitHub unreachable ({e}), falling back to local cache");
                self.load_from_cache().await
            }
            Err(e) => Err(e),
        }
    }

    /// Push a locally-modified vault to GitHub with optimistic locking.
    ///
    /// 1. Serialize and PUT to GitHub using `current_sha` as the concurrency token.
    /// 2. On 200: update the cache, return the new SHA.
    /// 3. On 422 (SHA mismatch): fetch the remote, merge LWW, retry the PUT once.
    ///
    /// Returns the new GitHub blob SHA.
    #[instrument(skip(self, local), fields(owner = %self.owner, current_sha = %current_sha))]
    pub async fn push(&self, local: &VaultFile, current_sha: &str) -> Result<String, StorageError> {
        let json_bytes = local.to_json()?.into_bytes();

        match self
            .github
            .write_file(
                &self.owner,
                "vault.json",
                &json_bytes,
                current_sha,
                "tacoshell: sync vault",
            )
            .await
        {
            Ok(new_sha) => {
                self.cache
                    .store(&CacheEntry {
                        vault_bytes: json_bytes,
                        sha: new_sha.clone(),
                        cached_at: Utc::now(),
                    })
                    .await?;
                Ok(new_sha)
            }
            Err(StorageError::ShaMismatch) => self.push_after_conflict(local).await,
            Err(e) => Err(e),
        }
    }

    /// First-time vault creation: uploads an empty vault to GitHub.
    ///
    /// Returns the initial blob SHA.
    #[instrument(skip(self), fields(owner = %self.owner))]
    pub async fn init_vault(&self) -> Result<String, StorageError> {
        let empty = VaultFile::new();
        let json_bytes = empty.to_json()?.into_bytes();
        let sha = self
            .github
            .create_file(
                &self.owner,
                "vault.json",
                &json_bytes,
                "tacoshell: init vault",
            )
            .await?;
        self.cache
            .store(&CacheEntry {
                vault_bytes: json_bytes,
                sha: sha.clone(),
                cached_at: Utc::now(),
            })
            .await?;
        Ok(sha)
    }

    /// Merge `local` and `remote` vault files using per-item last-write-wins.
    ///
    /// Rules (from `STORAGE.md §3.3`):
    /// - `local.updated_at > remote.updated_at` → keep local
    /// - `remote.updated_at > local.updated_at` → keep remote
    /// - equal timestamps, same ciphertext → deduplicated (no conflict)
    /// - equal timestamps, different ciphertext → conflict: keep local provisionally
    /// - item only in local → include (local add)
    /// - item only in remote → include (do NOT propagate local deletes)
    pub fn merge_vaults(local: &VaultFile, remote: &VaultFile) -> MergeResult {
        let local_map: HashMap<&str, &EncryptedItem> =
            local.items.iter().map(|i| (i.id.as_str(), i)).collect();
        let remote_map: HashMap<&str, &EncryptedItem> =
            remote.items.iter().map(|i| (i.id.as_str(), i)).collect();

        let mut merged_items: Vec<EncryptedItem> = Vec::new();
        let mut conflicts: Vec<ConflictItem> = Vec::new();

        // Iterate all unique IDs across both sides.
        let all_ids: std::collections::HashSet<&str> =
            local_map.keys().chain(remote_map.keys()).copied().collect();

        for id in all_ids {
            match (local_map.get(id), remote_map.get(id)) {
                (Some(l), None) => {
                    // Present locally, absent remotely → include (local add).
                    merged_items.push((*l).clone());
                }
                (None, Some(r)) => {
                    // Absent locally, present remotely → include.
                    // This also covers the "local delete" case: the remote version
                    // is restored per §3.3 ("don't propagate local deletes").
                    merged_items.push((*r).clone());
                }
                (Some(l), Some(r)) => {
                    if l.updated_at > r.updated_at {
                        merged_items.push((*l).clone());
                    } else if r.updated_at > l.updated_at {
                        merged_items.push((*r).clone());
                    } else {
                        // Equal timestamps.
                        if l.ciphertext == r.ciphertext {
                            // Identical content — no conflict.
                            merged_items.push((*l).clone());
                        } else {
                            // True conflict: keep local provisionally, surface to UI.
                            warn!("vault merge conflict on item {id} — keeping local version");
                            conflicts.push(ConflictItem {
                                id: id.to_string(),
                                local: (*l).clone(),
                                remote: (*r).clone(),
                            });
                            merged_items.push((*l).clone());
                        }
                    }
                }
                (None, None) => unreachable!("id came from one of the maps"),
            }
        }

        let mut merged = VaultFile::new();
        merged.items = merged_items;
        MergeResult { merged, conflicts }
    }

    // -----------------------------------------------------------------------
    // Private helpers
    // -----------------------------------------------------------------------

    async fn load_from_cache(&self) -> Result<LoadResult, StorageError> {
        match self.cache.load().await? {
            Some(entry) => {
                let json_str = std::str::from_utf8(&entry.vault_bytes).map_err(|e| {
                    StorageError::Cache(format!("cached vault.json is not UTF-8: {e}"))
                })?;
                let vault = VaultFile::from_json(json_str)?;
                Ok(LoadResult {
                    vault,
                    sha: entry.sha,
                    source: LoadSource::Cache,
                })
            }
            None => Err(StorageError::Offline(
                "no local cache available — connect to the internet to load your vault".into(),
            )),
        }
    }

    /// Handle a 422 SHA mismatch: fetch remote, merge, retry PUT.
    async fn push_after_conflict(&self, local: &VaultFile) -> Result<String, StorageError> {
        let remote_file = self
            .github
            .read_file(&self.owner, "vault.json")
            .await?
            .ok_or(StorageError::RepoNotFound)?;

        let remote_json = std::str::from_utf8(&remote_file.content)
            .map_err(|e| StorageError::GitHub(format!("remote vault.json is not UTF-8: {e}")))?;
        let remote_vault = VaultFile::from_json(remote_json)?;

        let MergeResult { merged, conflicts } = Self::merge_vaults(local, &remote_vault);
        if !conflicts.is_empty() {
            warn!(
                "vault merge after SHA mismatch produced {} conflict(s) — keeping local versions",
                conflicts.len()
            );
        }

        let merged_bytes = merged.to_json()?.into_bytes();
        let new_sha = self
            .github
            .write_file(
                &self.owner,
                "vault.json",
                &merged_bytes,
                &remote_file.sha,
                "tacoshell: sync vault (conflict resolved)",
            )
            .await?;

        self.cache
            .store(&CacheEntry {
                vault_bytes: merged_bytes,
                sha: new_sha.clone(),
                cached_at: Utc::now(),
            })
            .await?;

        Ok(new_sha)
    }
}

/// Returns `true` for errors that indicate a transient network problem, where
/// falling back to the local cache is the appropriate response.
fn is_offline_error(e: &StorageError) -> bool {
    match e {
        StorageError::Offline(_) => true,
        StorageError::GitHub(msg) => {
            let lower = msg.to_lowercase();
            lower.contains("timeout")
                || lower.contains("connection")
                || lower.contains("network")
                || lower.contains("dns")
        }
        _ => false,
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::vault::EncryptedItem;
    use crate::storage::cache::MockCache;
    use crate::storage::github::{FileContent, MockGitHubStorage};
    use chrono::{DateTime, Duration, TimeZone};
    use pretty_assertions::assert_eq;

    const OWNER: &str = "test-owner";
    const SHA1: &str = "sha-v1";
    const SHA2: &str = "sha-v2";

    fn empty_vault_bytes() -> Vec<u8> {
        VaultFile::new().to_json().unwrap().into_bytes()
    }

    fn make_engine(
        github: MockGitHubStorage,
        cache: MockCache,
    ) -> SyncEngine<MockGitHubStorage, MockCache> {
        SyncEngine::new(github, cache, OWNER)
    }

    fn vault_with_item(id: &str, updated_at: chrono::DateTime<Utc>) -> VaultFile {
        let key = [0x42u8; 32];
        let mut item = EncryptedItem::encrypt_with_id(&key, id, b"payload").unwrap();
        item.updated_at = updated_at;
        let mut v = VaultFile::new();
        v.items.push(item);
        v
    }

    // --- load: GitHub path ---

    #[tokio::test]
    async fn load_fetches_from_github_caches_and_returns_vault() {
        let bytes = empty_vault_bytes();
        let bytes_clone = bytes.clone();

        let mut github = MockGitHubStorage::new();
        github
            .expect_read_file()
            .withf(|o, p| o == OWNER && p == "vault.json")
            .once()
            .returning(move |_, _| {
                Ok(Some(FileContent {
                    content: bytes_clone.clone(),
                    sha: SHA1.to_string(),
                }))
            });

        let mut cache = MockCache::new();
        cache.expect_store().once().returning(|_| Ok(()));

        let engine = make_engine(github, cache);
        let result = engine.load().await.unwrap();

        assert_eq!(result.source, LoadSource::GitHub);
        assert_eq!(result.sha, SHA1);
        assert!(result.vault.is_empty());
    }

    #[tokio::test]
    async fn load_returns_repo_not_found_when_file_absent() {
        let mut github = MockGitHubStorage::new();
        github.expect_read_file().once().returning(|_, _| Ok(None));

        let cache = MockCache::new();
        let engine = make_engine(github, cache);
        let err = engine.load().await.unwrap_err();
        assert!(matches!(err, StorageError::RepoNotFound));
    }

    #[tokio::test]
    async fn load_falls_back_to_cache_on_offline_error() {
        let bytes = empty_vault_bytes();
        let bytes_clone = bytes.clone();

        let mut github = MockGitHubStorage::new();
        github
            .expect_read_file()
            .once()
            .returning(|_, _| Err(StorageError::Offline("no route to host".into())));

        let mut cache = MockCache::new();
        cache.expect_load().once().returning(move || {
            Ok(Some(CacheEntry {
                vault_bytes: bytes_clone.clone(),
                sha: SHA1.to_string(),
                cached_at: Utc::now(),
            }))
        });

        let engine = make_engine(github, cache);
        let result = engine.load().await.unwrap();

        assert_eq!(result.source, LoadSource::Cache);
        assert_eq!(result.sha, SHA1);
    }

    #[tokio::test]
    async fn load_returns_offline_error_when_github_down_and_no_cache() {
        let mut github = MockGitHubStorage::new();
        github
            .expect_read_file()
            .once()
            .returning(|_, _| Err(StorageError::Offline("unreachable".into())));

        let mut cache = MockCache::new();
        cache.expect_load().once().returning(|| Ok(None));

        let engine = make_engine(github, cache);
        let err = engine.load().await.unwrap_err();
        assert!(matches!(err, StorageError::Offline(_)));
    }

    // --- push: success path ---

    #[tokio::test]
    async fn push_succeeds_updates_cache_and_returns_new_sha() {
        let mut github = MockGitHubStorage::new();
        github
            .expect_write_file()
            .once()
            .returning(|_, _, _, _, _| Ok(SHA2.to_string()));

        let mut cache = MockCache::new();
        cache.expect_store().once().returning(|_| Ok(()));

        let engine = make_engine(github, cache);
        let vault = VaultFile::new();
        let new_sha = engine.push(&vault, SHA1).await.unwrap();
        assert_eq!(new_sha, SHA2);
    }

    // --- push: SHA mismatch / conflict resolution ---

    #[tokio::test]
    async fn push_retries_with_merged_vault_after_sha_mismatch() {
        let remote_bytes = empty_vault_bytes();
        let remote_clone = remote_bytes.clone();

        let mut github = MockGitHubStorage::new();
        // First PUT → 422
        github
            .expect_write_file()
            .once()
            .returning(|_, _, _, _, _| Err(StorageError::ShaMismatch));
        // Fetch remote
        github.expect_read_file().once().returning(move |_, _| {
            Ok(Some(FileContent {
                content: remote_clone.clone(),
                sha: "remote-sha".to_string(),
            }))
        });
        // Retry PUT with remote SHA
        github
            .expect_write_file()
            .once()
            .returning(|_, _, _, sha, _| {
                assert_eq!(sha, "remote-sha");
                Ok("merged-sha".to_string())
            });

        let mut cache = MockCache::new();
        cache.expect_store().once().returning(|_| Ok(()));

        let engine = make_engine(github, cache);
        let local = VaultFile::new();
        let new_sha = engine.push(&local, SHA1).await.unwrap();
        assert_eq!(new_sha, "merged-sha");
    }

    // --- merge_vaults: pure logic ---

    fn base_ts() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap()
    }

    #[test]
    fn merge_local_wins_when_local_is_newer() {
        let ts = base_ts();
        let local = vault_with_item("item-1", ts + Duration::seconds(10));
        let remote = vault_with_item("item-1", ts);
        let result = SyncEngine::<MockGitHubStorage, MockCache>::merge_vaults(&local, &remote);
        assert!(result.conflicts.is_empty());
        assert_eq!(result.merged.items.len(), 1);
        assert_eq!(result.merged.items[0].updated_at, local.items[0].updated_at);
    }

    #[test]
    fn merge_remote_wins_when_remote_is_newer() {
        let ts = base_ts();
        let local = vault_with_item("item-1", ts);
        let remote = vault_with_item("item-1", ts + Duration::seconds(5));
        let result = SyncEngine::<MockGitHubStorage, MockCache>::merge_vaults(&local, &remote);
        assert!(result.conflicts.is_empty());
        assert_eq!(
            result.merged.items[0].updated_at,
            remote.items[0].updated_at
        );
    }

    #[test]
    fn merge_keeps_local_only_item() {
        let ts = base_ts();
        let local = vault_with_item("local-only", ts);
        let remote = VaultFile::new();
        let result = SyncEngine::<MockGitHubStorage, MockCache>::merge_vaults(&local, &remote);
        assert_eq!(result.merged.items.len(), 1);
        assert_eq!(result.merged.items[0].id, "local-only");
    }

    #[test]
    fn merge_restores_remote_item_absent_from_local() {
        // Simulates "local delete": item was removed locally but still present remotely.
        // Per STORAGE.md §3.3, the remote version must be restored.
        let ts = base_ts();
        let local = VaultFile::new();
        let remote = vault_with_item("remote-only", ts);
        let result = SyncEngine::<MockGitHubStorage, MockCache>::merge_vaults(&local, &remote);
        assert_eq!(result.merged.items.len(), 1);
        assert_eq!(result.merged.items[0].id, "remote-only");
    }

    #[test]
    fn merge_no_conflict_when_same_timestamp_and_same_ciphertext() {
        let ts = base_ts();
        // Build two identical items (same id, same encrypted payload).
        let key = [0x42u8; 32];
        let item = EncryptedItem::encrypt_with_id(&key, "shared-id", b"data").unwrap();
        // Force the same timestamp.
        let mut item_l = item.clone();
        item_l.updated_at = ts;
        let mut item_r = item.clone();
        item_r.updated_at = ts;
        // Make ciphertext identical by copying.
        item_r.ciphertext = item_l.ciphertext.clone();
        item_r.nonce = item_l.nonce.clone();
        item_r.tag = item_l.tag.clone();

        let mut local = VaultFile::new();
        local.items.push(item_l);
        let mut remote = VaultFile::new();
        remote.items.push(item_r);

        let result = SyncEngine::<MockGitHubStorage, MockCache>::merge_vaults(&local, &remote);
        assert!(result.conflicts.is_empty());
        assert_eq!(result.merged.items.len(), 1);
    }

    #[test]
    fn merge_flags_conflict_when_same_timestamp_different_ciphertext() {
        let ts = base_ts();
        let key_a = [0x01u8; 32];
        let key_b = [0x02u8; 32];

        let mut item_l = EncryptedItem::encrypt_with_id(&key_a, "conflict-id", b"local").unwrap();
        item_l.updated_at = ts;
        let mut item_r = EncryptedItem::encrypt_with_id(&key_b, "conflict-id", b"remote").unwrap();
        item_r.updated_at = ts;

        let mut local = VaultFile::new();
        local.items.push(item_l.clone());
        let mut remote = VaultFile::new();
        remote.items.push(item_r);

        let result = SyncEngine::<MockGitHubStorage, MockCache>::merge_vaults(&local, &remote);
        assert_eq!(result.conflicts.len(), 1);
        assert_eq!(result.conflicts[0].id, "conflict-id");
        // Local is the provisional winner.
        assert_eq!(result.merged.items.len(), 1);
        assert_eq!(result.merged.items[0].ciphertext, item_l.ciphertext);
    }

    #[test]
    fn merge_combines_disjoint_local_and_remote_items() {
        let ts = base_ts();
        let local = vault_with_item("item-a", ts);
        let remote = vault_with_item("item-b", ts);
        let result = SyncEngine::<MockGitHubStorage, MockCache>::merge_vaults(&local, &remote);
        assert_eq!(result.merged.items.len(), 2);
        let ids: std::collections::HashSet<&str> =
            result.merged.items.iter().map(|i| i.id.as_str()).collect();
        assert!(ids.contains("item-a"));
        assert!(ids.contains("item-b"));
    }

    // --- integration: full create → modify → sync → reload ---

    #[tokio::test]
    async fn full_cycle_create_modify_push_reload() {
        let empty_bytes = empty_vault_bytes();
        let empty_bytes_c1 = empty_bytes.clone();
        let empty_bytes_c2 = empty_bytes.clone();

        // Phase 1: init_vault → create_file → sha1
        let mut github = MockGitHubStorage::new();
        github
            .expect_create_file()
            .once()
            .returning(move |_, _, _, _| Ok(SHA1.to_string()));

        // Phase 2: load → read_file → sha1
        let bytes_for_load = empty_bytes_c1.clone();
        github.expect_read_file().once().returning(move |_, _| {
            Ok(Some(FileContent {
                content: bytes_for_load.clone(),
                sha: SHA1.to_string(),
            }))
        });

        // Phase 3: push → write_file → sha2
        github
            .expect_write_file()
            .once()
            .returning(|_, _, _, _, _| Ok(SHA2.to_string()));

        // Phase 4: reload → read_file again → sha2
        let key = [0x42u8; 32];
        let item = EncryptedItem::encrypt(&key, b"secret").unwrap();
        let item_id = item.id.clone();
        let mut updated_vault = VaultFile::new();
        updated_vault.items.push(item);
        let updated_bytes = updated_vault.to_json().unwrap().into_bytes();
        let updated_bytes_c = updated_bytes.clone();
        github.expect_read_file().once().returning(move |_, _| {
            Ok(Some(FileContent {
                content: updated_bytes_c.clone(),
                sha: SHA2.to_string(),
            }))
        });

        let _ = empty_bytes_c2;
        let _ = updated_bytes;
        let mut cache = MockCache::new();
        cache.expect_store().times(4).returning(|_| Ok(()));

        let engine = SyncEngine::new(github, cache, OWNER);

        // 1. Init
        let sha1 = engine.init_vault().await.unwrap();
        assert_eq!(sha1, SHA1);

        // 2. Load
        let loaded = engine.load().await.unwrap();
        assert_eq!(loaded.sha, SHA1);
        assert!(loaded.vault.is_empty());

        // 3. Push modified vault
        let new_sha = engine.push(&updated_vault, SHA1).await.unwrap();
        assert_eq!(new_sha, SHA2);

        // 4. Reload and verify item survived
        let reloaded = engine.load().await.unwrap();
        assert_eq!(reloaded.sha, SHA2);
        assert_eq!(reloaded.vault.items.len(), 1);
        assert_eq!(reloaded.vault.items[0].id, item_id);
    }
}
