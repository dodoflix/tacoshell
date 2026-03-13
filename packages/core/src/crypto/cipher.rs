use aes_gcm::{
    aead::{Aead, KeyInit, Payload},
    Aes256Gcm, Nonce,
};
use ring::rand::{SecureRandom, SystemRandom};
use thiserror::Error;
use zeroize::Zeroizing;

pub const NONCE_LEN: usize = 12; // 96-bit nonce
pub const TAG_LEN: usize = 16; // 128-bit GCM authentication tag

#[derive(Debug, Error)]
pub enum CipherError {
    #[error("encryption failed")]
    Encryption,
    #[error("decryption failed — ciphertext may be corrupt or tampered")]
    Decryption,
    #[error("random number generation failed")]
    Rng,
}

/// An AEAD-encrypted data envelope: nonce, ciphertext, and authentication tag stored separately.
#[derive(Debug, Clone)]
pub struct EncryptedEnvelope {
    pub nonce: [u8; NONCE_LEN],
    pub ciphertext: Vec<u8>,
    pub tag: [u8; TAG_LEN],
}

/// Encrypts `plaintext` with AES-256-GCM using a fresh random nonce.
///
/// `aad` is bound to the ciphertext via GCM authentication (use `b""` for no AAD).
/// Pass the vault item ID + schema version as AAD to prevent cross-item ciphertext swaps.
pub fn encrypt(
    key: &[u8; 32],
    plaintext: &[u8],
    aad: &[u8],
) -> Result<EncryptedEnvelope, CipherError> {
    let rng = SystemRandom::new();
    let mut nonce_bytes = [0u8; NONCE_LEN];
    rng.fill(&mut nonce_bytes).map_err(|_| CipherError::Rng)?;

    let cipher = Aes256Gcm::new_from_slice(key).map_err(|_| CipherError::Encryption)?;
    let nonce = Nonce::from_slice(&nonce_bytes);

    // aes-gcm appends the 16-byte tag to the ciphertext: output = ciphertext || tag
    let ciphertext_with_tag = cipher
        .encrypt(
            nonce,
            Payload {
                msg: plaintext,
                aad,
            },
        )
        .map_err(|_| CipherError::Encryption)?;

    // Split ciphertext and tag
    let tag_start = ciphertext_with_tag.len() - TAG_LEN;
    let mut tag = [0u8; TAG_LEN];
    tag.copy_from_slice(&ciphertext_with_tag[tag_start..]);
    let ciphertext = ciphertext_with_tag[..tag_start].to_vec();

    Ok(EncryptedEnvelope {
        nonce: nonce_bytes,
        ciphertext,
        tag,
    })
}

/// Decrypts an `EncryptedEnvelope` with AES-256-GCM.
///
/// Returns the plaintext in a `Zeroizing` wrapper. Returns `CipherError::Decryption` if the
/// authentication tag is invalid (tampered data, wrong key, or wrong AAD).
pub fn decrypt(
    key: &[u8; 32],
    envelope: &EncryptedEnvelope,
    aad: &[u8],
) -> Result<Zeroizing<Vec<u8>>, CipherError> {
    let cipher = Aes256Gcm::new_from_slice(key).map_err(|_| CipherError::Decryption)?;
    let nonce = Nonce::from_slice(&envelope.nonce);

    // Reassemble ciphertext || tag for the Aead::decrypt interface
    let mut ct_with_tag = envelope.ciphertext.clone();
    ct_with_tag.extend_from_slice(&envelope.tag);

    let plaintext = cipher
        .decrypt(
            nonce,
            Payload {
                msg: &ct_with_tag,
                aad,
            },
        )
        .map_err(|_| CipherError::Decryption)?;

    Ok(Zeroizing::new(plaintext))
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_KEY: [u8; 32] = [0x42u8; 32];

    #[test]
    fn encrypt_produces_output_same_length_as_plaintext() {
        let envelope = encrypt(&TEST_KEY, b"hello world", b"").unwrap();
        assert_eq!(envelope.ciphertext.len(), b"hello world".len());
    }

    #[test]
    fn encrypt_uses_random_nonce_each_time() {
        let a = encrypt(&TEST_KEY, b"hello", b"").unwrap();
        let b = encrypt(&TEST_KEY, b"hello", b"").unwrap();
        assert_ne!(a.nonce, b.nonce);
    }

    #[test]
    fn encrypt_produces_different_ciphertext_each_time() {
        let a = encrypt(&TEST_KEY, b"hello", b"").unwrap();
        let b = encrypt(&TEST_KEY, b"hello", b"").unwrap();
        assert_ne!(a.ciphertext, b.ciphertext);
    }

    #[test]
    fn decrypt_round_trips_correctly() {
        let plaintext = b"the quick brown fox";
        let envelope = encrypt(&TEST_KEY, plaintext, b"").unwrap();
        let decrypted = decrypt(&TEST_KEY, &envelope, b"").unwrap();
        assert_eq!(decrypted.as_slice(), plaintext);
    }

    #[test]
    fn decrypt_round_trips_with_aad() {
        let plaintext = b"secret data";
        let aad = b"item-id-123v1";
        let envelope = encrypt(&TEST_KEY, plaintext, aad).unwrap();
        let decrypted = decrypt(&TEST_KEY, &envelope, aad).unwrap();
        assert_eq!(decrypted.as_slice(), plaintext);
    }

    #[test]
    fn decrypt_fails_with_wrong_key() {
        let envelope = encrypt(&TEST_KEY, b"data", b"aad").unwrap();
        let wrong_key = [0x01u8; 32];
        let result = decrypt(&wrong_key, &envelope, b"aad");
        assert!(result.is_err());
    }

    #[test]
    fn decrypt_fails_with_wrong_aad() {
        let envelope = encrypt(&TEST_KEY, b"data", b"correct-aad").unwrap();
        let result = decrypt(&TEST_KEY, &envelope, b"wrong-aad");
        assert!(result.is_err());
    }

    #[test]
    fn decrypt_fails_when_ciphertext_is_tampered() {
        let mut envelope = encrypt(&TEST_KEY, b"data", b"").unwrap();
        envelope.ciphertext[0] ^= 0xff;
        let result = decrypt(&TEST_KEY, &envelope, b"");
        assert!(result.is_err());
    }

    #[test]
    fn decrypt_fails_when_tag_is_tampered() {
        let mut envelope = encrypt(&TEST_KEY, b"data", b"").unwrap();
        envelope.tag[0] ^= 0xff;
        let result = decrypt(&TEST_KEY, &envelope, b"");
        assert!(result.is_err());
    }

    #[test]
    fn decrypt_fails_when_nonce_is_tampered() {
        let mut envelope = encrypt(&TEST_KEY, b"data", b"").unwrap();
        envelope.nonce[0] ^= 0xff;
        let result = decrypt(&TEST_KEY, &envelope, b"");
        assert!(result.is_err());
    }

    #[test]
    fn encrypt_empty_plaintext_succeeds() {
        let envelope = encrypt(&TEST_KEY, b"", b"").unwrap();
        let decrypted = decrypt(&TEST_KEY, &envelope, b"").unwrap();
        assert_eq!(decrypted.as_slice(), b"");
    }

    #[test]
    fn tag_is_16_bytes() {
        let envelope = encrypt(&TEST_KEY, b"data", b"").unwrap();
        assert_eq!(envelope.tag.len(), TAG_LEN);
    }

    #[test]
    fn nonce_is_12_bytes() {
        let envelope = encrypt(&TEST_KEY, b"data", b"").unwrap();
        assert_eq!(envelope.nonce.len(), NONCE_LEN);
    }
}
