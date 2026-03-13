use std::path::PathBuf;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::storage::StorageError;

// ---------------------------------------------------------------------------
// Shared types
// ---------------------------------------------------------------------------

/// A snapshot of the vault as last fetched from GitHub.
///
/// Stores the raw vault.json bytes (already containing individually-encrypted
/// items) and the GitHub blob SHA needed for the next optimistic-lock PUT.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    /// Raw `vault.json` bytes — JSON with encrypted items, NOT additionally encrypted.
    pub vault_bytes: Vec<u8>,
    /// GitHub blob SHA for the version in `vault_bytes`.
    pub sha: String,
    /// Wall-clock time when this entry was written.
    pub cached_at: DateTime<Utc>,
}

// ---------------------------------------------------------------------------
// Trait
// ---------------------------------------------------------------------------

/// Persistent local cache for the vault snapshot.
///
/// Allows the app to start in offline mode and avoids redundant GitHub fetches
/// when the SHA has not changed.
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait Cache: Send + Sync {
    /// Load the cached vault snapshot.
    ///
    /// Returns `None` if no cache exists yet (first run or after `clear`).
    async fn load(&self) -> Result<Option<CacheEntry>, StorageError>;

    /// Persist a vault snapshot, overwriting any previous entry.
    async fn store(&self, entry: &CacheEntry) -> Result<(), StorageError>;

    /// Remove the cached snapshot (e.g., on sign-out).
    async fn clear(&self) -> Result<(), StorageError>;
}

// ---------------------------------------------------------------------------
// File-based implementation
// ---------------------------------------------------------------------------

/// File-based cache that stores a single JSON file at `{base_dir}/vault_cache.json`.
///
/// Writes are atomic: the file is written to a `.tmp` sibling first, then
/// renamed, so a crash mid-write never leaves a corrupt file on disk.
pub struct FileCache {
    base_dir: PathBuf,
}

impl FileCache {
    const CACHE_FILE: &'static str = "vault_cache.json";
    const TEMP_FILE: &'static str = "vault_cache.json.tmp";

    /// Production constructor — resolves the platform-appropriate data directory.
    ///
    /// | Platform | Path |
    /// |----------|------|
    /// | macOS    | `~/Library/Application Support/tacoshell/cache/` |
    /// | Windows  | `%APPDATA%\tacoshell\cache\` |
    /// | Linux    | `~/.local/share/tacoshell/cache/` |
    pub fn new() -> Result<Self, StorageError> {
        let dir = dirs::data_dir()
            .ok_or_else(|| StorageError::Cache("cannot determine platform data directory".into()))?
            .join("tacoshell")
            .join("cache");
        Ok(FileCache { base_dir: dir })
    }

    /// Test constructor — caller supplies the directory.
    #[cfg(test)]
    pub fn with_base_dir(base_dir: impl Into<PathBuf>) -> Self {
        FileCache {
            base_dir: base_dir.into(),
        }
    }

    fn cache_path(&self) -> PathBuf {
        self.base_dir.join(Self::CACHE_FILE)
    }

    fn temp_path(&self) -> PathBuf {
        self.base_dir.join(Self::TEMP_FILE)
    }
}

#[async_trait]
impl Cache for FileCache {
    async fn load(&self) -> Result<Option<CacheEntry>, StorageError> {
        let path = self.cache_path();
        if !path.exists() {
            return Ok(None);
        }
        let bytes = std::fs::read(&path)?;
        let entry: CacheEntry = serde_json::from_slice(&bytes)?;
        Ok(Some(entry))
    }

    async fn store(&self, entry: &CacheEntry) -> Result<(), StorageError> {
        std::fs::create_dir_all(&self.base_dir)?;
        let json = serde_json::to_vec(entry)?;
        // Atomic write: write to .tmp then rename.
        let tmp = self.temp_path();
        std::fs::write(&tmp, &json)?;
        std::fs::rename(&tmp, self.cache_path())?;
        Ok(())
    }

    async fn clear(&self) -> Result<(), StorageError> {
        let path = self.cache_path();
        if path.exists() {
            std::fs::remove_file(path)?;
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn make_cache() -> (FileCache, TempDir) {
        let dir = TempDir::new().unwrap();
        let cache = FileCache::with_base_dir(dir.path());
        (cache, dir)
    }

    fn sample_entry() -> CacheEntry {
        CacheEntry {
            vault_bytes: br#"{"schema_version":"1","items":[]}"#.to_vec(),
            sha: "sha-abc123".to_string(),
            cached_at: Utc::now(),
        }
    }

    // --- load ---

    #[tokio::test]
    async fn load_returns_none_when_no_cache_file_exists() {
        let (cache, _dir) = make_cache();
        let result = cache.load().await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn load_returns_entry_after_store() {
        let (cache, _dir) = make_cache();
        let entry = sample_entry();
        cache.store(&entry).await.unwrap();
        let loaded = cache
            .load()
            .await
            .unwrap()
            .expect("expected Some(CacheEntry)");
        assert_eq!(loaded.sha, entry.sha);
        assert_eq!(loaded.vault_bytes, entry.vault_bytes);
    }

    #[tokio::test]
    async fn load_returns_none_after_clear() {
        let (cache, _dir) = make_cache();
        cache.store(&sample_entry()).await.unwrap();
        cache.clear().await.unwrap();
        assert!(cache.load().await.unwrap().is_none());
    }

    // --- store ---

    #[tokio::test]
    async fn store_creates_parent_directories_if_missing() {
        let dir = TempDir::new().unwrap();
        // Use a nested subdirectory that does not exist yet.
        let nested = dir.path().join("a").join("b").join("c");
        let cache = FileCache::with_base_dir(&nested);
        // Should succeed even though the directory tree doesn't exist.
        cache.store(&sample_entry()).await.unwrap();
        assert!(cache.cache_path().exists());
    }

    #[tokio::test]
    async fn store_overwrites_previous_entry() {
        let (cache, _dir) = make_cache();
        let first = CacheEntry {
            sha: "sha-first".to_string(),
            ..sample_entry()
        };
        let second = CacheEntry {
            sha: "sha-second".to_string(),
            ..sample_entry()
        };
        cache.store(&first).await.unwrap();
        cache.store(&second).await.unwrap();
        let loaded = cache.load().await.unwrap().unwrap();
        assert_eq!(loaded.sha, "sha-second");
    }

    // --- clear ---

    #[tokio::test]
    async fn clear_is_idempotent_when_no_cache_exists() {
        let (cache, _dir) = make_cache();
        // Must not error when there is nothing to clear.
        cache.clear().await.unwrap();
        cache.clear().await.unwrap();
    }

    #[tokio::test]
    async fn clear_removes_cache_file() {
        let (cache, _dir) = make_cache();
        cache.store(&sample_entry()).await.unwrap();
        assert!(cache.cache_path().exists());
        cache.clear().await.unwrap();
        assert!(!cache.cache_path().exists());
    }

    // --- round-trip ---

    #[tokio::test]
    async fn vault_bytes_survive_store_and_load_round_trip() {
        let (cache, _dir) = make_cache();
        let raw = b"arbitrary binary \x00\x01\x02 vault bytes";
        let entry = CacheEntry {
            vault_bytes: raw.to_vec(),
            sha: "sha-rt".to_string(),
            cached_at: Utc::now(),
        };
        cache.store(&entry).await.unwrap();
        let loaded = cache.load().await.unwrap().unwrap();
        assert_eq!(loaded.vault_bytes, raw);
        assert_eq!(loaded.sha, "sha-rt");
    }
}
