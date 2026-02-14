//! SSH channel wrapper for PTY and shell operations

use ssh2::Channel;
use std::io::{Read, Write};
use tacoshell_core::{Error, Result};
use tracing::debug;

/// PTY modes and dimensions
#[derive(Debug, Clone)]
pub struct PtyConfig {
    pub term: String,
    pub cols: u32,
    pub rows: u32,
    pub width: u32,
    pub height: u32,
}

impl Default for PtyConfig {
    fn default() -> Self {
        Self {
            term: "xterm-256color".to_string(),
            cols: 80,
            rows: 24,
            width: 0,
            height: 0,
        }
    }
}

/// Wrapper around SSH channel with PTY support
pub struct SshChannel {
    channel: Channel,
}

impl SshChannel {
    /// Create a new channel wrapper
    pub fn new(channel: Channel) -> Self {
        Self { channel }
    }

    /// Request a PTY with the given configuration
    pub fn request_pty(&mut self, config: &PtyConfig) -> Result<()> {
        debug!("Requesting PTY: {} {}x{}", config.term, config.cols, config.rows);

        self.channel
            .request_pty(
                &config.term,
                None,
                Some((config.cols, config.rows, config.width, config.height)),
            )
            .map_err(|e| Error::Session(format!("Failed to request PTY: {}", e)))?;

        Ok(())
    }

    /// Start an interactive shell
    pub fn shell(&mut self) -> Result<()> {
        debug!("Starting shell");
        self.channel
            .shell()
            .map_err(|e| Error::Session(format!("Failed to start shell: {}", e)))
    }

    /// Resize the PTY
    pub fn resize(&mut self, cols: u32, rows: u32) -> Result<()> {
        debug!("Resizing PTY to {}x{}", cols, rows);
        self.channel
            .request_pty_size(cols, rows, None, None)
            .map_err(|e| Error::Session(format!("Failed to resize PTY: {}", e)))
    }

    /// Write data to the channel
    pub fn write(&mut self, data: &[u8]) -> Result<usize> {
        self.channel
            .write(data)
            .map_err(|e| Error::Session(format!("Failed to write to channel: {}", e)))
    }

    /// Write all data to the channel, retrying on WouldBlock
    pub fn write_all(&mut self, data: &[u8]) -> Result<()> {
        use std::io::Write;
        self.channel
            .write_all(data)
            .map_err(|e| Error::Session(format!("Failed to write to channel: {}", e)))
    }

    /// Flush the channel
    pub fn flush(&mut self) -> Result<()> {
        use std::io::Write;
        self.channel
            .flush()
            .map_err(|e| Error::Session(format!("Failed to flush channel: {}", e)))
    }

    /// Read data from the channel (non-blocking style)
    pub fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        self.channel
            .read(buf)
            .map_err(|e| Error::Session(format!("Failed to read from channel: {}", e)))
    }

    /// Check if the channel has reached EOF
    pub fn eof(&self) -> bool {
        self.channel.eof()
    }

    /// Send EOF to the channel
    pub fn send_eof(&mut self) -> Result<()> {
        self.channel
            .send_eof()
            .map_err(|e| Error::Session(format!("Failed to send EOF: {}", e)))
    }

    /// Close the channel
    pub fn close(&mut self) -> Result<()> {
        self.channel
            .close()
            .map_err(|e| Error::Session(format!("Failed to close channel: {}", e)))?;
        self.channel
            .wait_close()
            .map_err(|e| Error::Session(format!("Failed to wait for close: {}", e)))
    }

    /// Get the exit status of the channel
    pub fn exit_status(&self) -> Result<i32> {
        self.channel
            .exit_status()
            .map_err(|e| Error::Session(format!("Failed to get exit status: {}", e)))
    }
}

