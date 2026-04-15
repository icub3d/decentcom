use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use bip39::{Language, Mnemonic};
use ed25519_dalek::{Signer, SigningKey};
use keyring::Entry;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use zeroize::Zeroizing;

const SERVICE_NAME: &str = "decentcom";
/// Keyring entry storing a JSON array of pubkey strings.
const ACCOUNTS_INDEX: &str = "accounts_index";
/// Prefix for per-account seed entries: "seed_{pubkey}".
fn seed_entry_name(pubkey: &str) -> String {
    format!("seed_{pubkey}")
}

/// In-memory active account pubkey. Defaults to first account if not set.
static ACTIVE_ACCOUNT: Mutex<Option<String>> = Mutex::new(None);

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

#[derive(Serialize, Deserialize, Clone)]
pub struct AccountInfo {
    pub pubkey: String,
    pub active: bool,
}

#[derive(Serialize, Deserialize)]
pub struct PublicKeyInfo {
    pub pubkey: String,
}

#[derive(Serialize, Deserialize)]
pub struct SignatureInfo {
    pub signature: String,
}

// ---------------------------------------------------------------------------
// Accounts index helpers
// ---------------------------------------------------------------------------

fn load_accounts_index() -> Result<Vec<String>, IdentityError> {
    let entry = Entry::new(SERVICE_NAME, ACCOUNTS_INDEX)?;
    match entry.get_password() {
        Ok(json) => {
            let accounts: Vec<String> =
                serde_json::from_str(&json).unwrap_or_default();
            Ok(accounts)
        }
        Err(keyring::Error::NoEntry) => Ok(Vec::new()),
        Err(e) => Err(IdentityError::Keyring(e.to_string())),
    }
}

fn save_accounts_index(accounts: &[String]) -> Result<(), IdentityError> {
    let entry = Entry::new(SERVICE_NAME, ACCOUNTS_INDEX)?;
    let json = serde_json::to_string(accounts)
        .map_err(|e| IdentityError::Crypto(e.to_string()))?;
    entry.set_password(&json)?;
    Ok(())
}

fn get_active_pubkey() -> Result<String, IdentityError> {
    let accounts = load_accounts_index()?;
    if accounts.is_empty() {
        return Err(IdentityError::Crypto("no accounts found".into()));
    }
    let lock = ACTIVE_ACCOUNT.lock().unwrap();
    if let Some(ref pk) = *lock {
        if accounts.contains(pk) {
            return Ok(pk.clone());
        }
    }
    Ok(accounts[0].clone())
}

// ---------------------------------------------------------------------------
// Tauri commands
// ---------------------------------------------------------------------------

#[tauri::command]
pub fn has_identity() -> bool {
    load_accounts_index()
        .map(|a| !a.is_empty())
        .unwrap_or(false)
}

#[tauri::command]
pub fn list_accounts() -> Result<Vec<AccountInfo>, String> {
    inner_list_accounts().map_err(|e| e.to_string())
}

fn inner_list_accounts() -> Result<Vec<AccountInfo>, IdentityError> {
    let accounts = load_accounts_index()?;
    let active = get_active_pubkey().ok();
    Ok(accounts
        .into_iter()
        .map(|pk| AccountInfo {
            active: active.as_deref() == Some(&pk),
            pubkey: pk,
        })
        .collect())
}

#[tauri::command]
pub fn set_active_account(pubkey: String) -> Result<(), String> {
    inner_set_active_account(pubkey).map_err(|e| e.to_string())
}

fn inner_set_active_account(pubkey: String) -> Result<(), IdentityError> {
    let accounts = load_accounts_index()?;
    if !accounts.contains(&pubkey) {
        return Err(IdentityError::Crypto("account not found".into()));
    }
    let mut lock = ACTIVE_ACCOUNT.lock().unwrap();
    *lock = Some(pubkey);
    Ok(())
}

#[tauri::command]
pub fn delete_account(pubkey: String) -> Result<(), String> {
    inner_delete_account(pubkey).map_err(|e| e.to_string())
}

fn inner_delete_account(pubkey: String) -> Result<(), IdentityError> {
    let mut accounts = load_accounts_index()?;
    if !accounts.contains(&pubkey) {
        return Err(IdentityError::Crypto("account not found".into()));
    }

    // Remove the seed from keyring.
    let entry = Entry::new(SERVICE_NAME, &seed_entry_name(&pubkey))?;
    match entry.delete_credential() {
        Ok(()) | Err(keyring::Error::NoEntry) => {}
        Err(e) => return Err(IdentityError::Keyring(e.to_string())),
    }

    accounts.retain(|pk| pk != &pubkey);
    save_accounts_index(&accounts)?;

    // If the deleted account was active, clear the selection.
    let mut lock = ACTIVE_ACCOUNT.lock().unwrap();
    if lock.as_deref() == Some(&pubkey) {
        *lock = None;
    }

    Ok(())
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
/// seed in the OS keychain under a per-account entry, add the pubkey to the
/// accounts index, and set it as the active account.
fn derive_and_store(mnemonic: &Mnemonic) -> Result<(String, SigningKey), IdentityError> {
    let seed = Zeroizing::new(mnemonic.to_seed(""));
    let seed_bytes: Zeroizing<[u8; 32]> = Zeroizing::new(
        seed[..32]
            .try_into()
            .map_err(|_| IdentityError::Crypto("Invalid seed length".into()))?,
    );
    let signing_key = SigningKey::from_bytes(&seed_bytes);
    let pubkey = bs58::encode(signing_key.verifying_key().as_bytes()).into_string();

    // Store seed under per-account keyring entry.
    let hex_seed = Zeroizing::new(hex::encode(seed_bytes.as_slice()));
    let entry = Entry::new(SERVICE_NAME, &seed_entry_name(&pubkey))?;
    entry.set_password(&hex_seed)?;

    // Add to accounts index (idempotent).
    let mut accounts = load_accounts_index()?;
    if !accounts.contains(&pubkey) {
        accounts.push(pubkey.clone());
        save_accounts_index(&accounts)?;
    }

    // Set as active.
    let mut lock = ACTIVE_ACCOUNT.lock().unwrap();
    *lock = Some(pubkey.clone());

    Ok((pubkey, signing_key))
}

#[tauri::command]
pub fn get_public_key() -> Result<PublicKeyInfo, String> {
    inner_get_public_key().map_err(|e| e.to_string())
}

fn inner_get_public_key() -> Result<PublicKeyInfo, IdentityError> {
    let pubkey = get_active_pubkey()?;
    // Validate we can actually load the key.
    let _ = get_signing_key_for(&pubkey)?;
    Ok(PublicKeyInfo { pubkey })
}

#[tauri::command]
pub fn sign(data: String) -> Result<SignatureInfo, String> {
    inner_sign(data).map_err(|e| e.to_string())
}

fn inner_sign(data: String) -> Result<SignatureInfo, IdentityError> {
    let pubkey = get_active_pubkey()?;
    let signing_key = get_signing_key_for(&pubkey)?;
    let signature = signing_key.sign(data.as_bytes());
    Ok(SignatureInfo {
        signature: BASE64.encode(signature.to_bytes()),
    })
}

fn get_signing_key_for(pubkey: &str) -> Result<SigningKey, IdentityError> {
    let entry = Entry::new(SERVICE_NAME, &seed_entry_name(pubkey))?;
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

    #[test]
    fn test_accounts_index_roundtrip() {
        let accounts = vec!["pk1".to_string(), "pk2".to_string()];
        let json = serde_json::to_string(&accounts).unwrap();
        let parsed: Vec<String> = serde_json::from_str(&json).unwrap();
        assert_eq!(accounts, parsed);
    }

    #[test]
    fn test_seed_entry_name() {
        assert_eq!(seed_entry_name("abc123"), "seed_abc123");
    }
}
