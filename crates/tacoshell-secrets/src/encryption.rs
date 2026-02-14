//! Encryption utilities using age

use std::io::{Read, Write};
use tacoshell_core::{Error, Result};

/// Handles encryption and decryption of secrets using age
pub struct SecretEncryption {
    passphrase: String,
}

impl SecretEncryption {
    /// Create a new encryption handler with the given passphrase
    pub fn new(passphrase: impl Into<String>) -> Self {
        Self {
            passphrase: passphrase.into(),
        }
    }

    /// Encrypt plaintext data
    pub fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>> {
        let encryptor = age::Encryptor::with_user_passphrase(age::secrecy::Secret::new(
            self.passphrase.clone(),
        ));

        let mut encrypted = vec![];
        let mut writer = encryptor
            .wrap_output(&mut encrypted)
            .map_err(|e| Error::Secret(format!("Failed to create encryptor: {}", e)))?;

        writer
            .write_all(plaintext)
            .map_err(|e| Error::Secret(format!("Failed to encrypt: {}", e)))?;

        writer
            .finish()
            .map_err(|e| Error::Secret(format!("Failed to finish encryption: {}", e)))?;

        Ok(encrypted)
    }

    /// Decrypt encrypted data
    pub fn decrypt(&self, ciphertext: &[u8]) -> Result<Vec<u8>> {
        let decryptor = match age::Decryptor::new(ciphertext)
            .map_err(|e| Error::Secret(format!("Failed to create decryptor: {}", e)))?
        {
            age::Decryptor::Passphrase(d) => d,
            _ => return Err(Error::Secret("Unexpected decryptor type".into())),
        };

        let mut decrypted = vec![];
        let mut reader = decryptor
            .decrypt(&age::secrecy::Secret::new(self.passphrase.clone()), None)
            .map_err(|e| Error::Secret(format!("Failed to decrypt: {}", e)))?;

        reader
            .read_to_end(&mut decrypted)
            .map_err(|e| Error::Secret(format!("Failed to read decrypted data: {}", e)))?;

        Ok(decrypted)
    }

    /// Encrypt a string and return the encrypted bytes
    pub fn encrypt_string(&self, plaintext: &str) -> Result<Vec<u8>> {
        self.encrypt(plaintext.as_bytes())
    }

    /// Decrypt bytes and return as a string
    pub fn decrypt_string(&self, ciphertext: &[u8]) -> Result<String> {
        let decrypted = self.decrypt(ciphertext)?;
        String::from_utf8(decrypted)
            .map_err(|e| Error::Secret(format!("Invalid UTF-8 in decrypted data: {}", e)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt() {
        let encryption = SecretEncryption::new("test-passphrase");
        let plaintext = b"my secret password";

        let encrypted = encryption.encrypt(plaintext).unwrap();
        assert_ne!(encrypted, plaintext);

        let decrypted = encryption.decrypt(&encrypted).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_encrypt_decrypt_string() {
        let encryption = SecretEncryption::new("test-passphrase");
        let plaintext = "my secret password";

        let encrypted = encryption.encrypt_string(plaintext).unwrap();
        let decrypted = encryption.decrypt_string(&encrypted).unwrap();

        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_wrong_passphrase_fails() {
        let encryption = SecretEncryption::new("correct-passphrase");
        let encrypted = encryption.encrypt(b"secret").unwrap();

        let wrong_encryption = SecretEncryption::new("wrong-passphrase");
        assert!(wrong_encryption.decrypt(&encrypted).is_err());
    }
}

