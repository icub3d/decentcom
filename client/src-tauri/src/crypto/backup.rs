//! Encrypted key backup: encrypt/decrypt Ed25519 seeds for portable recovery.
//!
//! File format (137 bytes total):
//! ```text
//! Offset  Size  Field
//! 0       4     Magic bytes: "DCKB"
//! 4       1     Format version (1)
//! 5       32    Ed25519 public key (cleartext)
//! 37      4     Argon2id memory cost (KiB, LE u32)
//! 41      4     Argon2id time cost (iterations, LE u32)
//! 45      4     Argon2id parallelism (LE u32)
//! 49      16    Argon2id salt
//! 65      24    XChaCha20-Poly1305 nonce
//! 89      48    Ciphertext (32-byte seed + 16-byte Poly1305 tag)
//! ```

use argon2::Argon2;
use chacha20poly1305::{
    aead::{Aead, KeyInit},
    XChaCha20Poly1305, XNonce,
};
use ed25519_dalek::SigningKey;
use rand::RngCore;
use zeroize::Zeroizing;

const MAGIC: &[u8; 4] = b"DCKB";
const FORMAT_VERSION: u8 = 1;
const EXPECTED_FILE_SIZE: usize = 137;

// Argon2id defaults — strong but reasonable on modern hardware.
const DEFAULT_MEMORY_KIB: u32 = 64 * 1024; // 64 MiB (3.4× OWASP minimum; old files decrypt fine via header params)
const DEFAULT_TIME_COST: u32 = 3;
const DEFAULT_PARALLELISM: u32 = 4;

const SALT_LEN: usize = 16;
const NONCE_LEN: usize = 24;
const SEED_LEN: usize = 32;

#[derive(Debug, thiserror::Error)]
pub enum BackupError {
    #[error("Invalid backup file: {0}")]
    InvalidFormat(String),
    #[error("Decryption failed: wrong passphrase or corrupted file")]
    DecryptionFailed,
    #[error("KDF error: {0}")]
    Kdf(String),
    #[error("Public key mismatch after decryption")]
    PubkeyMismatch,
}

/// Encrypt a 32-byte Ed25519 seed into the DCKB backup format.
pub fn encrypt_key(seed: &[u8; SEED_LEN], passphrase: &str) -> Result<Vec<u8>, BackupError> {
    // Derive the public key from the seed for the cleartext header.
    let signing_key = SigningKey::from_bytes(seed);
    let pubkey_bytes = signing_key.verifying_key().to_bytes();

    // Generate random salt and nonce.
    let mut salt = [0u8; SALT_LEN];
    let mut nonce_bytes = [0u8; NONCE_LEN];
    let mut rng = rand::rng();
    rng.fill_bytes(&mut salt);
    rng.fill_bytes(&mut nonce_bytes);

    // Derive encryption key with Argon2id.
    let enc_key = derive_key(passphrase, &salt)?;

    // Encrypt the seed.
    let cipher = XChaCha20Poly1305::new_from_slice(enc_key.as_ref())
        .map_err(|e| BackupError::Kdf(e.to_string()))?;
    let nonce = XNonce::from_slice(&nonce_bytes);
    let ciphertext = cipher
        .encrypt(nonce, seed.as_slice())
        .map_err(|_| BackupError::DecryptionFailed)?;

    // Assemble the file.
    let mut out = Vec::with_capacity(EXPECTED_FILE_SIZE);
    out.extend_from_slice(MAGIC);
    out.push(FORMAT_VERSION);
    out.extend_from_slice(&pubkey_bytes);
    out.extend_from_slice(&DEFAULT_MEMORY_KIB.to_le_bytes());
    out.extend_from_slice(&DEFAULT_TIME_COST.to_le_bytes());
    out.extend_from_slice(&DEFAULT_PARALLELISM.to_le_bytes());
    out.extend_from_slice(&salt);
    out.extend_from_slice(&nonce_bytes);
    out.extend_from_slice(&ciphertext);

    debug_assert_eq!(out.len(), EXPECTED_FILE_SIZE);
    Ok(out)
}

/// Decrypt a DCKB backup file, returning the 32-byte Ed25519 seed.
pub fn decrypt_key(data: &[u8], passphrase: &str) -> Result<Zeroizing<[u8; SEED_LEN]>, BackupError> {
    let parsed = parse_header(data)?;

    let enc_key = derive_key_with_params(
        passphrase,
        &parsed.salt,
        parsed.memory_kib,
        parsed.time_cost,
        parsed.parallelism,
    )?;

    let cipher = XChaCha20Poly1305::new_from_slice(enc_key.as_ref())
        .map_err(|e| BackupError::Kdf(e.to_string()))?;
    let nonce = XNonce::from_slice(&parsed.nonce);
    let plaintext = cipher
        .decrypt(nonce, parsed.ciphertext)
        .map_err(|_| BackupError::DecryptionFailed)?;

    let seed: [u8; SEED_LEN] = plaintext
        .try_into()
        .map_err(|_| BackupError::InvalidFormat("unexpected plaintext length".into()))?;

    // Verify the decrypted seed produces the same public key stored in the header.
    let signing_key = SigningKey::from_bytes(&seed);
    if signing_key.verifying_key().to_bytes() != parsed.pubkey {
        return Err(BackupError::PubkeyMismatch);
    }

    Ok(Zeroizing::new(seed))
}

/// Read the cleartext public key from a backup file without decrypting.
pub fn read_backup_pubkey(data: &[u8]) -> Result<[u8; 32], BackupError> {
    if data.len() < 37 {
        return Err(BackupError::InvalidFormat("file too short".into()));
    }
    if &data[0..4] != MAGIC {
        return Err(BackupError::InvalidFormat("bad magic bytes".into()));
    }
    let mut pubkey = [0u8; 32];
    pubkey.copy_from_slice(&data[5..37]);
    Ok(pubkey)
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

struct ParsedBackup<'a> {
    pubkey: [u8; 32],
    memory_kib: u32,
    time_cost: u32,
    parallelism: u32,
    salt: [u8; SALT_LEN],
    nonce: [u8; NONCE_LEN],
    ciphertext: &'a [u8],
}

fn parse_header(data: &[u8]) -> Result<ParsedBackup<'_>, BackupError> {
    if data.len() < EXPECTED_FILE_SIZE {
        return Err(BackupError::InvalidFormat(format!(
            "expected {} bytes, got {}",
            EXPECTED_FILE_SIZE,
            data.len()
        )));
    }
    if &data[0..4] != MAGIC {
        return Err(BackupError::InvalidFormat("bad magic bytes".into()));
    }
    if data[4] != FORMAT_VERSION {
        return Err(BackupError::InvalidFormat(format!(
            "unsupported version {}",
            data[4]
        )));
    }

    let mut pubkey = [0u8; 32];
    pubkey.copy_from_slice(&data[5..37]);

    let memory_kib = u32::from_le_bytes(data[37..41].try_into().unwrap());
    let time_cost = u32::from_le_bytes(data[41..45].try_into().unwrap());
    let parallelism = u32::from_le_bytes(data[45..49].try_into().unwrap());

    let mut salt = [0u8; SALT_LEN];
    salt.copy_from_slice(&data[49..65]);

    let mut nonce = [0u8; NONCE_LEN];
    nonce.copy_from_slice(&data[65..89]);

    let ciphertext = &data[89..EXPECTED_FILE_SIZE];

    Ok(ParsedBackup {
        pubkey,
        memory_kib,
        time_cost,
        parallelism,
        salt,
        nonce,
        ciphertext,
    })
}

fn derive_key(passphrase: &str, salt: &[u8; SALT_LEN]) -> Result<Zeroizing<[u8; 32]>, BackupError> {
    derive_key_with_params(
        passphrase,
        salt,
        DEFAULT_MEMORY_KIB,
        DEFAULT_TIME_COST,
        DEFAULT_PARALLELISM,
    )
}

fn derive_key_with_params(
    passphrase: &str,
    salt: &[u8; SALT_LEN],
    memory_kib: u32,
    time_cost: u32,
    parallelism: u32,
) -> Result<Zeroizing<[u8; 32]>, BackupError> {
    let params = argon2::Params::new(memory_kib, time_cost, parallelism, Some(32))
        .map_err(|e| BackupError::Kdf(e.to_string()))?;
    let argon2 = Argon2::new(argon2::Algorithm::Argon2id, argon2::Version::V0x13, params);

    let mut key = Zeroizing::new([0u8; 32]);
    argon2
        .hash_password_into(passphrase.as_bytes(), salt, key.as_mut())
        .map_err(|e| BackupError::Kdf(e.to_string()))?;
    Ok(key)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Use fast KDF params in tests to avoid slow CI. We test the real
    // params via the `argon2id_params_in_file` test that checks the header.
    fn test_encrypt(seed: &[u8; 32], passphrase: &str) -> Vec<u8> {
        // Encrypt normally then verify the header params match defaults.
        // For speed, we'll just use the real function — the default params
        // are used in the file format. Tests that call decrypt_key will
        // read those params from the file header and use them.
        encrypt_key(seed, passphrase).unwrap()
    }

    #[test]
    fn roundtrip_encrypt_decrypt() {
        let seed = [42u8; 32];
        let passphrase = "test-passphrase-long-enough";
        let backup = test_encrypt(&seed, passphrase);
        let recovered = decrypt_key(&backup, passphrase).unwrap();
        assert_eq!(*recovered, seed);
    }

    #[test]
    fn wrong_passphrase_fails() {
        let seed = [42u8; 32];
        let backup = test_encrypt(&seed, "correct-passphrase!!");
        let result = decrypt_key(&backup, "wrong-passphrase!!!");
        assert!(matches!(result, Err(BackupError::DecryptionFailed)));
    }

    #[test]
    fn truncated_file_fails() {
        let seed = [42u8; 32];
        let backup = test_encrypt(&seed, "test-passphrase-long-enough");
        let result = decrypt_key(&backup[..50], "test-passphrase-long-enough");
        assert!(matches!(result, Err(BackupError::InvalidFormat(_))));
    }

    #[test]
    fn corrupted_ciphertext_fails() {
        let seed = [42u8; 32];
        let passphrase = "test-passphrase-long-enough";
        let mut backup = test_encrypt(&seed, passphrase);
        // Flip a bit in the ciphertext region.
        backup[100] ^= 0xFF;
        let result = decrypt_key(&backup, passphrase);
        assert!(matches!(result, Err(BackupError::DecryptionFailed)));
    }

    #[test]
    fn read_pubkey_without_decrypting() {
        let seed = [42u8; 32];
        let signing_key = SigningKey::from_bytes(&seed);
        let expected_pubkey = signing_key.verifying_key().to_bytes();

        let backup = test_encrypt(&seed, "test-passphrase-long-enough");
        let pubkey = read_backup_pubkey(&backup).unwrap();
        assert_eq!(pubkey, expected_pubkey);
    }

    #[test]
    fn format_version_is_one() {
        let seed = [42u8; 32];
        let backup = test_encrypt(&seed, "test-passphrase-long-enough");
        assert_eq!(backup[4], 1);
    }

    #[test]
    fn argon2id_params_in_file() {
        let seed = [42u8; 32];
        let backup = test_encrypt(&seed, "test-passphrase-long-enough");
        let memory = u32::from_le_bytes(backup[37..41].try_into().unwrap());
        let time = u32::from_le_bytes(backup[41..45].try_into().unwrap());
        let par = u32::from_le_bytes(backup[45..49].try_into().unwrap());
        assert_eq!(memory, DEFAULT_MEMORY_KIB);
        assert_eq!(time, DEFAULT_TIME_COST);
        assert_eq!(par, DEFAULT_PARALLELISM);
    }

    #[test]
    fn file_size_is_correct() {
        let seed = [42u8; 32];
        let backup = test_encrypt(&seed, "test-passphrase-long-enough");
        assert_eq!(backup.len(), EXPECTED_FILE_SIZE);
    }

    #[test]
    fn bad_magic_bytes_rejected() {
        let mut data = vec![0u8; EXPECTED_FILE_SIZE];
        data[0..4].copy_from_slice(b"NOPE");
        let result = decrypt_key(&data, "anything");
        assert!(matches!(result, Err(BackupError::InvalidFormat(_))));
    }
}
