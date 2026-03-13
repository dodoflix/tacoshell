use argon2::{Algorithm, Argon2, Params, Version};
use ring::digest;
use thiserror::Error;
use zeroize::Zeroizing;

const ARGON2_M_COST: u32 = 65536; // 64 MB
const ARGON2_T_COST: u32 = 3;
const ARGON2_P_COST: u32 = 1;
pub const KEY_LEN: usize = 32;

#[derive(Debug, Error)]
pub enum KdfError {
    #[error("key derivation failed: {0}")]
    Derivation(#[from] argon2::Error),
}

/// Derives a 256-bit Master Key from a passphrase and GitHub user ID.
///
/// Salt = SHA-256(github_user_id) — deterministic, never stored on disk.
/// Parameters: Argon2id, m=64 MB, t=3 iterations, p=1 lane.
pub fn derive_master_key(
    passphrase: &[u8],
    github_user_id: &str,
) -> Result<Zeroizing<[u8; KEY_LEN]>, KdfError> {
    // Salt is deterministically derived — no storage needed, still unique per user.
    let salt_digest = digest::digest(&digest::SHA256, github_user_id.as_bytes());
    let salt = salt_digest.as_ref(); // 32 bytes

    let params = Params::new(ARGON2_M_COST, ARGON2_T_COST, ARGON2_P_COST, Some(KEY_LEN))?;
    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);

    let mut key = Zeroizing::new([0u8; KEY_LEN]);
    argon2.hash_password_into(passphrase, salt, key.as_mut())?;

    Ok(key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derive_master_key_returns_32_bytes() {
        let key = derive_master_key(b"mysecretpassphrase", "12345678").unwrap();
        assert_eq!(key.len(), 32);
    }

    #[test]
    fn derive_master_key_is_deterministic() {
        let key1 = derive_master_key(b"passphrase", "user123").unwrap();
        let key2 = derive_master_key(b"passphrase", "user123").unwrap();
        assert_eq!(key1.as_ref(), key2.as_ref());
    }

    #[test]
    fn different_passphrases_produce_different_keys() {
        let key1 = derive_master_key(b"passphrase_a", "user123").unwrap();
        let key2 = derive_master_key(b"passphrase_b", "user123").unwrap();
        assert_ne!(key1.as_ref(), key2.as_ref());
    }

    #[test]
    fn different_user_ids_produce_different_keys() {
        let key1 = derive_master_key(b"passphrase", "user_aaa").unwrap();
        let key2 = derive_master_key(b"passphrase", "user_bbb").unwrap();
        assert_ne!(key1.as_ref(), key2.as_ref());
    }

    #[test]
    fn empty_passphrase_is_accepted() {
        let result = derive_master_key(b"", "user123");
        assert!(result.is_ok());
    }

    #[test]
    fn output_is_non_zero_for_valid_input() {
        let key = derive_master_key(b"correct horse battery staple", "1234567").unwrap();
        // All-zero output would indicate a bug in the KDF.
        assert!(key.iter().any(|&b| b != 0));
    }

    #[test]
    fn key_length_is_exactly_32_bytes() {
        let key = derive_master_key(b"test", "uid").unwrap();
        assert_eq!(key.as_ref().len(), 32);
    }
}
