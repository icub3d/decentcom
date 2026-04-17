//! Encrypted key backup: encrypt/decrypt Ed25519 seeds for portable recovery.
//!
//! ## Version 1 format (137 bytes, read-only — legacy)
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
//!
//! ## Version 2 format (variable length — new writes)
//! Bytes 0-136 are identical to v1 (with version byte = 2).
//! ```text
//! 137     4     Metadata ciphertext length N (LE u32; 0 = no metadata)
//! --- only present when N > 0 ---
//! 141     24    Metadata XChaCha20-Poly1305 nonce
//! 165     N     Encrypted metadata (UTF-8 JSON + 16-byte Poly1305 tag)
//! ```
//! Metadata is encrypted with the same Argon2id-derived key, fresh nonce.

use argon2::Argon2;
use chacha20poly1305::{
    aead::{Aead, KeyInit},
    XChaCha20Poly1305, XNonce,
};
use ed25519_dalek::SigningKey;
use rand::RngCore;
use zeroize::Zeroizing;

const MAGIC: &[u8; 4] = b"DCKB";
const FORMAT_VERSION_V1: u8 = 1;
const FORMAT_VERSION_V2: u8 = 2;

// Argon2id defaults — strong but reasonable on modern hardware.
const DEFAULT_MEMORY_KIB: u32 = 64 * 1024; // 64 MiB
const DEFAULT_TIME_COST: u32 = 3;
const DEFAULT_PARALLELISM: u32 = 4;

const SALT_LEN: usize = 16;
const NONCE_LEN: usize = 24;
const SEED_LEN: usize = 32;

/// Size of the fixed key section shared by v1 and v2.
const KEY_SECTION_SIZE: usize = 137;
/// Minimum size of a v2 file (key section + 4-byte metadata length).
const V2_MIN_SIZE: usize = KEY_SECTION_SIZE + 4;

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

/// Encrypt a 32-byte Ed25519 seed into the DCKB v2 backup format.
/// Pass `metadata` as a JSON string to include optional encrypted app state.
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
        .map_err(|_| BackupError::DecryptionFailed)?;

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
                .map_err(|_| BackupError::DecryptionFailed)?;

            let meta_ct_len = meta_ct.len() as u32;
            out.extend_from_slice(&meta_ct_len.to_le_bytes());
            out.extend_from_slice(&meta_nonce_bytes);
            out.extend_from_slice(&meta_ct);
        }
    }

    Ok(out)
}

/// Decrypt a DCKB backup file (v1 or v2).
/// Returns `(seed, metadata_json)` where `metadata_json` is `None` for v1
/// files or v2 files without embedded metadata.
pub fn decrypt_key(
    data: &[u8],
    passphrase: &str,
) -> Result<(Zeroizing<[u8; SEED_LEN]>, Option<String>), BackupError> {
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

    let signing_key = SigningKey::from_bytes(&seed);
    if signing_key.verifying_key().to_bytes() != parsed.pubkey {
        return Err(BackupError::PubkeyMismatch);
    }

    if parsed.version != FORMAT_VERSION_V2 {
        return Ok((Zeroizing::new(seed), None));
    }

    // V2: read optional metadata.
    if data.len() < V2_MIN_SIZE {
        return Ok((Zeroizing::new(seed), None));
    }
    let meta_ct_len =
        u32::from_le_bytes(data[KEY_SECTION_SIZE..V2_MIN_SIZE].try_into().unwrap()) as usize;
    if meta_ct_len == 0 {
        return Ok((Zeroizing::new(seed), None));
    }

    let meta_nonce_start = V2_MIN_SIZE;
    let meta_nonce_end = meta_nonce_start + NONCE_LEN;
    let meta_ct_start = meta_nonce_end;
    let meta_ct_end = meta_ct_start + meta_ct_len;

    if data.len() < meta_ct_end {
        return Err(BackupError::InvalidFormat(
            "v2 file truncated in metadata section".into(),
        ));
    }

    let meta_nonce = XNonce::from_slice(&data[meta_nonce_start..meta_nonce_end]);
    let meta_ciphertext = &data[meta_ct_start..meta_ct_end];

    let meta_cipher = XChaCha20Poly1305::new_from_slice(enc_key.as_ref())
        .map_err(|e| BackupError::Kdf(e.to_string()))?;
    let meta_plaintext = meta_cipher
        .decrypt(meta_nonce, meta_ciphertext)
        .map_err(|_| BackupError::DecryptionFailed)?;

    let meta_json = String::from_utf8(meta_plaintext)
        .map_err(|_| BackupError::InvalidFormat("metadata is not valid UTF-8".into()))?;

    Ok((Zeroizing::new(seed), Some(meta_json)))
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
    version: u8,
    pubkey: [u8; 32],
    memory_kib: u32,
    time_cost: u32,
    parallelism: u32,
    salt: [u8; SALT_LEN],
    nonce: [u8; NONCE_LEN],
    ciphertext: &'a [u8],
}

fn parse_header(data: &[u8]) -> Result<ParsedBackup<'_>, BackupError> {
    if data.len() < KEY_SECTION_SIZE {
        return Err(BackupError::InvalidFormat(format!(
            "expected at least {} bytes, got {}",
            KEY_SECTION_SIZE,
            data.len()
        )));
    }
    if &data[0..4] != MAGIC {
        return Err(BackupError::InvalidFormat("bad magic bytes".into()));
    }
    let version = data[4];
    if version != FORMAT_VERSION_V1 && version != FORMAT_VERSION_V2 {
        return Err(BackupError::InvalidFormat(format!(
            "unsupported version {}",
            version
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

    let ciphertext = &data[89..KEY_SECTION_SIZE];

    Ok(ParsedBackup {
        version,
        pubkey,
        memory_kib,
        time_cost,
        parallelism,
        salt,
        nonce,
        ciphertext,
    })
}

fn derive_key(
    passphrase: &str,
    salt: &[u8; SALT_LEN],
) -> Result<Zeroizing<[u8; 32]>, BackupError> {
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

    fn test_encrypt(seed: &[u8; 32], passphrase: &str) -> Vec<u8> {
        encrypt_key(seed, passphrase, None).unwrap()
    }

    #[test]
    fn roundtrip_encrypt_decrypt() {
        let seed = [42u8; 32];
        let passphrase = "test-passphrase-long-enough";
        let backup = test_encrypt(&seed, passphrase);
        let (recovered, meta) = decrypt_key(&backup, passphrase).unwrap();
        assert_eq!(*recovered, seed);
        assert!(meta.is_none());
    }

    #[test]
    fn roundtrip_with_metadata() {
        let seed = [42u8; 32];
        let passphrase = "test-passphrase-long-enough";
        let meta_in = r#"{"version":1,"servers":{},"theme":"mocha"}"#;
        let backup = encrypt_key(&seed, passphrase, Some(meta_in)).unwrap();
        let (recovered, meta_out) = decrypt_key(&backup, passphrase).unwrap();
        assert_eq!(*recovered, seed);
        assert_eq!(meta_out.as_deref(), Some(meta_in));
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
    fn format_version_is_two() {
        let seed = [42u8; 32];
        let backup = test_encrypt(&seed, "test-passphrase-long-enough");
        assert_eq!(backup[4], FORMAT_VERSION_V2);
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
    fn v2_no_metadata_min_size() {
        let seed = [42u8; 32];
        let backup = test_encrypt(&seed, "test-passphrase-long-enough");
        assert_eq!(backup.len(), V2_MIN_SIZE);
    }

    #[test]
    fn v2_with_metadata_size() {
        let seed = [42u8; 32];
        let meta = r#"{"version":1}"#;
        let backup = encrypt_key(&seed, "test-passphrase-long-enough", Some(meta)).unwrap();
        // V2_MIN_SIZE + NONCE_LEN + plaintext_len + 16-byte poly1305 tag
        let expected = V2_MIN_SIZE + NONCE_LEN + meta.len() + 16;
        assert_eq!(backup.len(), expected);
    }

    #[test]
    fn bad_magic_bytes_rejected() {
        let mut data = vec![0u8; KEY_SECTION_SIZE];
        data[0..4].copy_from_slice(b"NOPE");
        let result = decrypt_key(&data, "anything");
        assert!(matches!(result, Err(BackupError::InvalidFormat(_))));
    }

    #[test]
    fn v1_file_decrypts_without_metadata() {
        // Construct a minimal v1-format file manually and verify it decrypts.
        let seed = [42u8; 32];
        let passphrase = "test-passphrase-long-enough";
        // Encrypt as v2 (no metadata), then patch byte 4 to version 1.
        let mut backup = test_encrypt(&seed, passphrase);
        backup[4] = FORMAT_VERSION_V1;
        // Trim to exactly KEY_SECTION_SIZE to simulate a real v1 file.
        backup.truncate(KEY_SECTION_SIZE);
        let (recovered, meta) = decrypt_key(&backup, passphrase).unwrap();
        assert_eq!(*recovered, seed);
        assert!(meta.is_none());
    }

    #[test]
    fn wrong_metadata_passphrase_fails() {
        let seed = [42u8; 32];
        let meta = r#"{"version":1}"#;
        let backup = encrypt_key(&seed, "correct-passphrase!!", Some(meta)).unwrap();
        let result = decrypt_key(&backup, "wrong-passphrase!!!");
        assert!(matches!(result, Err(BackupError::DecryptionFailed)));
    }
}
