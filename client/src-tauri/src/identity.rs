use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use bip39::{Language, Mnemonic};
use ed25519_dalek::{Signer, SigningKey};
use keyring::Entry;
use serde::{Deserialize, Serialize};
use zeroize::Zeroizing;

const SERVICE_NAME: &str = "decentcom";
const ACCOUNT_NAME: &str = "master_privkey_seed";

#[derive(Debug, thiserror::Error)]
pub enum IdentityError {
    #[error("Keyring error: {0}")]
    Keyring(String),
    #[error("Mnemonic error: {0}")]
    Mnemonic(String),
    #[error("Crypto error: {0}")]
    Crypto(String),
}

impl From<keyring::Error> for IdentityError {
    fn from(err: keyring::Error) -> Self {
        Self::Keyring(err.to_string())
    }
}

#[derive(Serialize, Deserialize)]
pub struct IdentityInfo {
    pub pubkey: String,
    pub seed_phrase: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct PublicKeyInfo {
    pub pubkey: String,
}

#[derive(Serialize, Deserialize)]
pub struct SignatureInfo {
    pub signature: String,
}

#[tauri::command]
pub fn has_identity() -> bool {
    let entry = match Entry::new(SERVICE_NAME, ACCOUNT_NAME) {
        Ok(entry) => entry,
        Err(_) => return false,
    };
    entry.get_password().is_ok()
}

#[tauri::command]
pub fn generate_identity() -> Result<IdentityInfo, String> {
    inner_generate_identity().map_err(|e| e.to_string())
}

fn inner_generate_identity() -> Result<IdentityInfo, IdentityError> {
    let mnemonic = Mnemonic::generate_in(Language::English, 24)
        .map_err(|e| IdentityError::Mnemonic(format!("{:?}", e)))?;
    let seed_phrase: Vec<String> = mnemonic.to_string().split(' ').map(String::from).collect();

    let (pubkey, _) = derive_and_store(&mnemonic)?;
    Ok(IdentityInfo { pubkey, seed_phrase })
}

#[tauri::command]
pub fn import_identity(seed_phrase: Vec<String>) -> Result<PublicKeyInfo, String> {
    inner_import_identity(seed_phrase).map_err(|e| e.to_string())
}

fn inner_import_identity(seed_phrase: Vec<String>) -> Result<PublicKeyInfo, IdentityError> {
    let phrase = seed_phrase.join(" ");
    let mnemonic = Mnemonic::parse_in(Language::English, &phrase)
        .map_err(|e| IdentityError::Mnemonic(format!("{:?}", e)))?;
    let (pubkey, _) = derive_and_store(&mnemonic)?;
    Ok(PublicKeyInfo { pubkey })
}

/// Derive the Ed25519 signing key from a BIP39 mnemonic, store its 32-byte
/// seed in the OS keychain (hex-encoded), and return the Base58 public key.
/// All intermediate private material is held in `Zeroizing` buffers that
/// are scrubbed on drop.
fn derive_and_store(mnemonic: &Mnemonic) -> Result<(String, SigningKey), IdentityError> {
    let seed = Zeroizing::new(mnemonic.to_seed(""));
    let seed_bytes: Zeroizing<[u8; 32]> = Zeroizing::new(
        seed[..32]
            .try_into()
            .map_err(|_| IdentityError::Crypto("Invalid seed length".into()))?,
    );
    let signing_key = SigningKey::from_bytes(&seed_bytes);
    let pubkey = bs58::encode(signing_key.verifying_key().as_bytes()).into_string();

    let hex_seed = Zeroizing::new(hex::encode(seed_bytes.as_slice()));
    let entry = Entry::new(SERVICE_NAME, ACCOUNT_NAME)?;
    entry.set_password(&hex_seed)?;

    Ok((pubkey, signing_key))
}

#[tauri::command]
pub fn get_public_key() -> Result<PublicKeyInfo, String> {
    inner_get_public_key().map_err(|e| e.to_string())
}

fn inner_get_public_key() -> Result<PublicKeyInfo, IdentityError> {
    let signing_key = get_signing_key()?;
    let pubkey = bs58::encode(signing_key.verifying_key().as_bytes()).into_string();
    Ok(PublicKeyInfo { pubkey })
}

#[tauri::command]
pub fn sign(data: String) -> Result<SignatureInfo, String> {
    inner_sign(data).map_err(|e| e.to_string())
}

fn inner_sign(data: String) -> Result<SignatureInfo, IdentityError> {
    let signing_key = get_signing_key()?;
    let signature = signing_key.sign(data.as_bytes());
    Ok(SignatureInfo {
        signature: BASE64.encode(signature.to_bytes()),
    })
}

fn get_signing_key() -> Result<SigningKey, IdentityError> {
    let entry = Entry::new(SERVICE_NAME, ACCOUNT_NAME)?;
    let hex_seed = Zeroizing::new(entry.get_password()?);
    let seed_bytes: Zeroizing<Vec<u8>> = Zeroizing::new(
        hex::decode(hex_seed.as_str())
            .map_err(|_| IdentityError::Crypto("Invalid hex in keychain".into()))?,
    );
    let seed_array: Zeroizing<[u8; 32]> = Zeroizing::new(
        seed_bytes
            .as_slice()
            .try_into()
            .map_err(|_| IdentityError::Crypto("Invalid seed length".into()))?,
    );
    Ok(SigningKey::from_bytes(&seed_array))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::Verifier;

    #[test]
    fn test_mnemonic_derivation() {
        let mnemonic_phrase = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art";
        let seed_phrase: Vec<String> = mnemonic_phrase.split(' ').map(String::from).collect();
        
        let res = inner_import_identity(seed_phrase).unwrap();
        // The public key for this mnemonic should be deterministic
        assert!(!res.pubkey.is_empty());
        let pubkey1 = res.pubkey;
        
        let seed_phrase2: Vec<String> = mnemonic_phrase.split(' ').map(String::from).collect();
        let res2 = inner_import_identity(seed_phrase2).unwrap();
        assert_eq!(pubkey1, res2.pubkey);
    }

    #[test]
    fn test_sign_verify() {
        let seed_bytes = [0u8; 32];
        let signing_key = SigningKey::from_bytes(&seed_bytes);
        let data = "test data";
        let signature = signing_key.sign(data.as_bytes());
        
        signing_key.verifying_key().verify(data.as_bytes(), &signature).expect("Verification failed");
    }

    #[test]
    fn test_invalid_mnemonic() {
        let seed_phrase = vec!["invalid".to_string(); 24];
        let res = inner_import_identity(seed_phrase);
        assert!(res.is_err());
    }
}
