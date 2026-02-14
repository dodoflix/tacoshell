//! SFTP client implementation

use ssh2::Sftp;
use std::path::Path;
use tacoshell_core::{Error, FileEntry, Result};
use tracing::debug;

/// SFTP client wrapper
pub struct SftpClient {
    sftp: Sftp,
}

impl SftpClient {
    /// Create a new SFTP client from an SSH session's SFTP subsystem
    pub fn new(sftp: Sftp) -> Self {
        Self { sftp }
    }

    /// List directory contents
    pub fn list_dir(&self, path: &str) -> Result<Vec<FileEntry>> {
        debug!("Listing directory: {}", path);

        let entries = self
            .sftp
            .readdir(Path::new(path))
            .map_err(|e| Error::Transfer(format!("Failed to list directory: {}", e)))?;

        let file_entries = entries
            .into_iter()
            .map(|(path, stat)| FileEntry {
                name: path.file_name().unwrap_or_default().to_string_lossy().to_string(),
                path: path.to_string_lossy().to_string(),
                is_dir: stat.is_dir(),
                size: stat.size.unwrap_or(0),
                modified: stat.mtime.map(|t| {
                    chrono::DateTime::from_timestamp(t as i64, 0)
                        .unwrap_or_default()
                        .with_timezone(&chrono::Utc)
                }),
                permissions: stat.perm,
            })
            .collect();

        Ok(file_entries)
    }

    /// Download a file from remote to local
    pub fn download(&self, remote_path: &str, local_path: &str) -> Result<()> {
        debug!("Downloading {} to {}", remote_path, local_path);

        let mut remote_file = self
            .sftp
            .open(Path::new(remote_path))
            .map_err(|e| Error::Transfer(format!("Failed to open remote file: {}", e)))?;

        let mut local_file = std::fs::File::create(local_path)
            .map_err(|e| Error::Transfer(format!("Failed to create local file: {}", e)))?;

        std::io::copy(&mut remote_file, &mut local_file)
            .map_err(|e| Error::Transfer(format!("Failed to copy file: {}", e)))?;

        Ok(())
    }

    /// Upload a file from local to remote
    pub fn upload(&self, local_path: &str, remote_path: &str) -> Result<()> {
        debug!("Uploading {} to {}", local_path, remote_path);

        let mut local_file = std::fs::File::open(local_path)
            .map_err(|e| Error::Transfer(format!("Failed to open local file: {}", e)))?;

        let mut remote_file = self
            .sftp
            .create(Path::new(remote_path))
            .map_err(|e| Error::Transfer(format!("Failed to create remote file: {}", e)))?;

        std::io::copy(&mut local_file, &mut remote_file)
            .map_err(|e| Error::Transfer(format!("Failed to copy file: {}", e)))?;

        Ok(())
    }

    /// Create a directory
    pub fn mkdir(&self, path: &str) -> Result<()> {
        debug!("Creating directory: {}", path);

        self.sftp
            .mkdir(Path::new(path), 0o755)
            .map_err(|e| Error::Transfer(format!("Failed to create directory: {}", e)))
    }

    /// Remove a file
    pub fn remove_file(&self, path: &str) -> Result<()> {
        debug!("Removing file: {}", path);

        self.sftp
            .unlink(Path::new(path))
            .map_err(|e| Error::Transfer(format!("Failed to remove file: {}", e)))
    }

    /// Remove a directory
    pub fn remove_dir(&self, path: &str) -> Result<()> {
        debug!("Removing directory: {}", path);

        self.sftp
            .rmdir(Path::new(path))
            .map_err(|e| Error::Transfer(format!("Failed to remove directory: {}", e)))
    }

    /// Get file/directory stats
    pub fn stat(&self, path: &str) -> Result<FileEntry> {
        let stat = self
            .sftp
            .stat(Path::new(path))
            .map_err(|e| Error::Transfer(format!("Failed to stat: {}", e)))?;

        Ok(FileEntry {
            name: Path::new(path)
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string(),
            path: path.to_string(),
            is_dir: stat.is_dir(),
            size: stat.size.unwrap_or(0),
            modified: stat.mtime.map(|t| {
                chrono::DateTime::from_timestamp(t as i64, 0)
                    .unwrap_or_default()
                    .with_timezone(&chrono::Utc)
            }),
            permissions: stat.perm,
        })
    }
}

