//! Encrypted key backup: encrypt Ed25519 seeds into the DCKB v2 format.
//!
//! This is a copy of the backup logic from `client/src-tauri/src/crypto/backup.rs`,
//! kept in sync manually. Only `encrypt_key` is needed here.

use argon2::Argon2;
use chacha20poly1305::{
    aead::{Aead, KeyInit},
    XChaCha20Poly1305, XNonce,
};
use ed25519_dalek::SigningKey;
use rand::RngCore;
use zeroize::Zeroizing;

const MAGIC: &[u8; 4] = b"DCKB";
const FORMAT_VERSION_V2: u8 = 2;

const DEFAULT_MEMORY_KIB: u32 = 64 * 1024;
const DEFAULT_TIME_COST: u32 = 3;
const DEFAULT_PARALLELISM: u32 = 4;

const SALT_LEN: usize = 16;
const NONCE_LEN: usize = 24;
const SEED_LEN: usize = 32;
const KEY_SECTION_SIZE: usize = 137;
const V2_MIN_SIZE: usize = KEY_SECTION_SIZE + 4;

#[derive(Debug, thiserror::Error)]
pub enum BackupError {
    #[error("KDF error: {0}")]
    Kdf(String),
    #[error("Encryption failed")]
    EncryptionFailed,
}

/// Encrypt a 32-byte Ed25519 seed into the DCKB v2 backup format.
pub fn encrypt_key(
    seed: &[u8; SEED_LEN],
    passphrase: &str,
    metadata: Option<&str>,
) -> Result<Vec<u8>, BackupError> {
    let signing_key = SigningKey::from_bytes(seed);
    let pubkey_bytes = signing_key.verifying_key().to_bytes();

    let mut salt = [0u8; SALT_LEN];
    let mut nonce_bytes = [0u8; NONCE_LEN];
    let mut rng = rand::rng();
    rng.fill_bytes(&mut salt);
    rng.fill_bytes(&mut nonce_bytes);

    let enc_key = derive_key(passphrase, &salt)?;

    let cipher = XChaCha20Poly1305::new_from_slice(enc_key.as_ref())
        .map_err(|e| BackupError::Kdf(e.to_string()))?;
    let nonce = XNonce::from_slice(&nonce_bytes);
    let ciphertext = cipher
        .encrypt(nonce, seed.as_slice())
        .map_err(|_| BackupError::EncryptionFailed)?;

    let mut out = Vec::with_capacity(V2_MIN_SIZE);
    out.extend_from_slice(MAGIC);
    out.push(FORMAT_VERSION_V2);
    out.extend_from_slice(&pubkey_bytes);
    out.extend_from_slice(&DEFAULT_MEMORY_KIB.to_le_bytes());
    out.extend_from_slice(&DEFAULT_TIME_COST.to_le_bytes());
    out.extend_from_slice(&DEFAULT_PARALLELISM.to_le_bytes());
    out.extend_from_slice(&salt);
    out.extend_from_slice(&nonce_bytes);
    out.extend_from_slice(&ciphertext);
    // Now at byte 137.

    match metadata {
        None => {
            out.extend_from_slice(&0u32.to_le_bytes());
        }
        Some(meta) => {
            let mut meta_nonce_bytes = [0u8; NONCE_LEN];
            rng.fill_bytes(&mut meta_nonce_bytes);

            let meta_cipher = XChaCha20Poly1305::new_from_slice(enc_key.as_ref())
                .map_err(|e| BackupError::Kdf(e.to_string()))?;
            let meta_nonce = XNonce::from_slice(&meta_nonce_bytes);
            let meta_ct = meta_cipher
                .encrypt(meta_nonce, meta.as_bytes())
                .map_err(|_| BackupError::EncryptionFailed)?;

            let meta_ct_len = meta_ct.len() as u32;
            out.extend_from_slice(&meta_ct_len.to_le_bytes());
            out.extend_from_slice(&meta_nonce_bytes);
            out.extend_from_slice(&meta_ct);
        }
    }

    Ok(out)
}

fn derive_key(
    passphrase: &str,
    salt: &[u8; SALT_LEN],
) -> Result<Zeroizing<[u8; 32]>, BackupError> {
    let params =
        argon2::Params::new(DEFAULT_MEMORY_KIB, DEFAULT_TIME_COST, DEFAULT_PARALLELISM, Some(32))
            .map_err(|e| BackupError::Kdf(e.to_string()))?;
    let argon2 = Argon2::new(argon2::Algorithm::Argon2id, argon2::Version::V0x13, params);

    let mut key = Zeroizing::new([0u8; 32]);
    argon2
        .hash_password_into(passphrase.as_bytes(), salt, key.as_mut())
        .map_err(|e| BackupError::Kdf(e.to_string()))?;
    Ok(key)
}
