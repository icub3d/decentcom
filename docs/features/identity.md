# Feature: Identity & Key Generation

## Overview
Implement Ed25519 master key pair generation in the Tauri client, derive it from a BIP39 seed phrase for recoverability, store the private key securely via the OS keychain, and expose the public key to the React frontend in a human-readable display format. This is the foundation of user identity in decentcom -- the public key is the user's portable identity across all servers.

## Background
The identity design doc ([identity.md](../design/identity.md)) specifies Ed25519 key pairs as the identity primitive. The master public key is the canonical user ID. The private key is stored by the Tauri core using OS-level secure storage and never exposed to the React layer. A BIP39 seed phrase is presented at generation time for recovery. The architecture doc ([architecture.md](../design/architecture.md)) confirms the signing boundary: all cryptographic operations happen in Rust, the React app only sends data to be signed and receives signatures.

## Requirements
- [ ] On first launch, the client generates an Ed25519 master key pair
- [ ] The key pair is derived from a BIP39-compatible 24-word mnemonic seed phrase
- [ ] The seed phrase is displayed to the user with a strong prompt to write it down before proceeding
- [ ] The private key is stored in the OS keychain (macOS Keychain, Linux libsecret/GNOME Keyring, Windows Credential Manager) via Tauri's secure storage plugin
- [ ] If the OS keychain is unavailable, the key is stored in an encrypted file with a user-provided passphrase (deferred — keychain failure surfaces as an error for now)
- [ ] The private key is never sent to the React frontend
- [ ] The public key is available to the React frontend via a Tauri IPC command
- [ ] The public key is displayed in a human-readable format (Base58 encoding)
- [ ] A Tauri IPC command allows the React app to request signing of arbitrary data; the Tauri core signs it and returns the signature
- [ ] The user can import an existing identity by entering a 24-word seed phrase
- [ ] On subsequent launches, the client loads the existing key from secure storage without prompting for the seed phrase

## Design

### API / Interface Changes

**Tauri IPC commands (Rust -> React bridge):**

| Command | Input | Output | Description |
|---|---|---|---|
| `has_identity` | none | `bool` | Check if a key pair exists in secure storage |
| `generate_identity` | none | `{ pubkey: string, seed_phrase: string[] }` | Generate new key pair, store private key, return public key and seed phrase |
| `import_identity` | `{ seed_phrase: string[] }` | `{ pubkey: string }` | Derive key pair from seed phrase, store private key, return public key |
| `get_public_key` | none | `{ pubkey: string }` | Return the stored public key in Base58 format |
| `sign` | `{ data: string }` | `{ signature: string }` | Sign data with the private key, return Base64-encoded signature |

### Data Model Changes
None on the server side. The client stores the private key in the OS keychain, not in a database.

**Keychain entries:**
- Key: `decentcom_master_privkey`
- Value: the 32-byte Ed25519 private key seed (encrypted by the OS keychain)

### Component Changes

**New files:**
```
client/src-tauri/src/identity.rs    # Key generation, BIP39 derivation, signing, storage
client/src/hooks/useIdentity.ts     # React hook wrapping identity IPC commands
client/src/pages/Setup.tsx          # First-launch setup flow: generate or import key
client/src/pages/SeedPhrase.tsx     # Seed phrase display and confirmation screen
```

**Modified files:**
```
client/src-tauri/Cargo.toml         # Add: ed25519-dalek, bip39, tauri-plugin-stronghold (or keyring), rand, base58
client/src-tauri/src/main.rs        # Register identity IPC commands
client/src-tauri/src/lib.rs         # Export identity module
client/src/App.tsx                  # Route to setup flow if no identity exists
```

**Key derivation flow:**
1. Generate 256 bits of entropy using a CSPRNG
2. Encode as a 24-word BIP39 mnemonic
3. Derive a 32-byte seed from the mnemonic using BIP39's PBKDF2 (with an empty passphrase, or a user passphrase for extra protection)
4. Use the first 32 bytes of the derived seed as the Ed25519 signing key seed
5. Compute the public key from the signing key
6. Store the signing key in the OS keychain
7. Return the public key and mnemonic to the caller

## Task List

### Server-side (none for this feature)

### Client-side (Tauri core)
- [x] Add `ed25519-dalek`, `bip39`, `rand`, `bs58`, and a secure storage plugin (`tauri-plugin-stronghold` or `keyring` crate) to `client/src-tauri/Cargo.toml`
- [x] Implement key generation from BIP39 mnemonic in `client/src-tauri/src/identity.rs`
- [x] Implement key import from existing mnemonic
- [x] Implement secure storage: save and load private key from OS keychain
- [ ] Implement fallback encrypted file storage if keychain is unavailable (deferred)
- [x] Implement `has_identity` IPC command
- [x] Implement `generate_identity` IPC command
- [x] Implement `import_identity` IPC command
- [x] Implement `get_public_key` IPC command (returns Base58-encoded public key)
- [x] Implement `sign` IPC command (signs arbitrary bytes, returns Base64 signature)
- [x] Register all IPC commands in `client/src-tauri/src/lib.rs`

### Client-side (React)
- [x] Create `useIdentity` hook that wraps `has_identity`, `get_public_key`, `generate_identity`, `import_identity`
- [x] Create Setup page that checks `has_identity` and offers "Create New Identity" or "Import Existing"
- [x] Create SeedPhrase page: displays the 24 words in a numbered grid, requires user confirmation before proceeding
- [x] Create import flow: text input for 24 words, validates the mnemonic, derives and stores the key
- [x] Update `App.tsx` to route to Setup on first launch, then to the main app after identity is established

## Test List
- [x] Unit test (Rust): generate key pair from mnemonic, verify the same mnemonic always produces the same key pair
- [x] Unit test (Rust): sign data and verify signature with the public key
- [x] Unit test (Rust): import mnemonic produces the expected public key
- [x] Unit test (Rust): invalid mnemonic (wrong words, wrong count) returns an error
- [x] Unit test (Rust): Base58 encoding of public key is deterministic and decodable
- [x] Unit test (React): `useIdentity` hook returns expected states (tested implicitly by `App.test.tsx`)
- [x] Unit test (React): Setup page renders create and import options (tested in `App.test.tsx`)
- [x] Unit test (React): SeedPhrase page displays 24 words (tested in `SeedPhrase.test.tsx`)
- [x] Integration test: full flow -- generate identity, retrieve public key, sign data, verify signature (tested via Rust unit tests in `identity.rs`)
- [x] Manual: first launch shows setup flow, generate key, see seed phrase, proceed to main app
- [x] Manual: close and reopen app, identity is loaded from keychain without setup flow
- [x] Manual: import a seed phrase from a previous generation, verify same public key is produced

## Implementation Notes

- Used `keyring` v3 crate (native OS APIs) rather than `tauri-plugin-stronghold`. Chose `async-secret-service` + `tokio` + `crypto-openssl` features for Linux libsecret compatibility.
- BIP39 uses standard PBKDF2 derivation (`mnemonic.to_seed("")`) for compatibility with other BIP39 tools. First 32 bytes of the 64-byte seed are used as the Ed25519 signing key seed.
- The private key seed is stored as hex in the OS keychain; the `Zeroizing` wrapper ensures the in-memory copy is cleared on drop.
- The encrypted file fallback is deferred — keychain unavailability will surface as a user-visible error until that is implemented.
- `bip39` v2 requires the `rand` feature to expose `Mnemonic::generate`.
- Seed phrase confirmation is a single "I have written it down" button (no word re-entry quiz). The quiz variant is left as a future UX improvement.

## Open Questions
- Should we use `tauri-plugin-stronghold` (Tauri's built-in encrypted vault) or the `keyring` crate (direct OS keychain access)? Stronghold is more portable but adds a Tauri-specific dependency. The `keyring` crate uses native OS APIs directly.
- Should the seed phrase confirmation require the user to re-enter specific words (e.g., "enter word #3, #12, #19") before allowing them to proceed? This is common in crypto wallets and strongly encourages users to actually write the phrase down.
- The BIP39 spec uses PBKDF2 with 2048 rounds to derive the seed. Should we use this standard derivation, or simply use the entropy bytes directly as the Ed25519 seed? Using standard BIP39 derivation means the seed phrase is compatible with other tools; using raw entropy is simpler.
