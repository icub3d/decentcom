//! Identity management: BIP39 mnemonics, Ed25519 key derivation and signing.

use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use bip39::{Language, Mnemonic};
use ed25519_dalek::{Signer, SigningKey};

use crate::error::{Error, Result};

/// A decentcom identity: an Ed25519 keypair derived from a BIP39 mnemonic or
/// raw 32-byte seed. The pubkey is encoded as base58 on the wire.
#[derive(Clone)]
pub struct Identity {
    signing_key: SigningKey,
    pubkey_b58: String,
}

impl Identity {
    /// Generate a fresh 24-word BIP39 mnemonic and derive a key from it.
    pub fn generate() -> Result<(Self, Mnemonic)> {
        let mnemonic = Mnemonic::generate_in(Language::English, 24)
            .map_err(|e| Error::Mnemonic(format!("{e:?}")))?;
        let identity = Self::from_mnemonic(&mnemonic)?;
        Ok((identity, mnemonic))
    }

    /// Parse a BIP39 phrase (space-separated words) and derive a key.
    pub fn from_phrase(phrase: &str) -> Result<Self> {
        let mnemonic = Mnemonic::parse_in(Language::English, phrase)
            .map_err(|e| Error::Mnemonic(format!("{e:?}")))?;
        Self::from_mnemonic(&mnemonic)
    }

    /// Derive a key from a parsed BIP39 mnemonic.
    pub fn from_mnemonic(mnemonic: &Mnemonic) -> Result<Self> {
        let seed = mnemonic.to_seed("");
        let seed_bytes: [u8; 32] = seed[..32]
            .try_into()
            .map_err(|_| Error::Identity("invalid seed length".into()))?;
        Ok(Self::from_seed(&seed_bytes))
    }

    /// Build an identity directly from a 32-byte Ed25519 seed.
    pub fn from_seed(seed: &[u8; 32]) -> Self {
        let signing_key = SigningKey::from_bytes(seed);
        let pubkey_b58 = bs58::encode(signing_key.verifying_key().as_bytes()).into_string();
        Self {
            signing_key,
            pubkey_b58,
        }
    }

    /// Base58-encoded public key, as used on the wire.
    pub fn pubkey(&self) -> &str {
        &self.pubkey_b58
    }

    /// Sign `data` and return the base64-encoded signature (the form the
    /// server expects in `VerifyRequest`).
    pub fn sign(&self, data: &[u8]) -> String {
        let sig = self.signing_key.sign(data);
        BASE64.encode(sig.to_bytes())
    }

    /// Raw signing key — exposed for callers that need to sign non-auth
    /// payloads with the same key material.
    pub fn signing_key(&self) -> &SigningKey {
        &self.signing_key
    }
}

impl std::fmt::Debug for Identity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Identity")
            .field("pubkey", &self.pubkey_b58)
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::Verifier;

    const KNOWN_PHRASE: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art";

    #[test]
    fn phrase_derivation_is_deterministic() {
        let a = Identity::from_phrase(KNOWN_PHRASE).unwrap();
        let b = Identity::from_phrase(KNOWN_PHRASE).unwrap();
        assert_eq!(a.pubkey(), b.pubkey());
        assert!(!a.pubkey().is_empty());
    }

    #[test]
    fn seed_derivation_matches_known_output() {
        let id = Identity::from_seed(&[7u8; 32]);
        let expected = bs58::encode(
            SigningKey::from_bytes(&[7u8; 32])
                .verifying_key()
                .as_bytes(),
        )
        .into_string();
        assert_eq!(id.pubkey(), expected);
    }

    #[test]
    fn sign_produces_valid_base64_signature() {
        let id = Identity::from_seed(&[1u8; 32]);
        let sig_b64 = id.sign(b"challenge");
        let bytes = BASE64.decode(sig_b64).unwrap();
        let sig = ed25519_dalek::Signature::from_slice(&bytes).unwrap();
        id.signing_key()
            .verifying_key()
            .verify(b"challenge", &sig)
            .unwrap();
    }

    #[test]
    fn invalid_phrase_rejected() {
        let err = Identity::from_phrase("not a valid mnemonic at all").unwrap_err();
        assert!(matches!(err, Error::Mnemonic(_)));
    }

    #[test]
    fn generate_produces_24_word_phrase() {
        let (id, mnemonic) = Identity::generate().unwrap();
        assert_eq!(mnemonic.to_string().split(' ').count(), 24);
        assert!(!id.pubkey().is_empty());
    }
}
