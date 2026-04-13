# Feature: Key Recovery

## Overview
Key recovery allows users to export their master Ed25519 private key as an encrypted backup and import it on a new device. The backup is encrypted with a user-chosen passphrase using a strong KDF, producing a portable file that is useless without the passphrase. This is the primary mechanism for users to recover their identity if they lose access to their device.

## Background
The identity design doc (`docs/design/identity.md`) describes two recommended recovery mechanisms for launch: seed phrase (BIP39 mnemonic, implemented in the `identity` feature #4) and encrypted key backup. This feature implements the encrypted key backup (Option 2 from the design doc). The master private key is stored in the OS keychain by the Tauri core and never exposed to the React frontend. The export and import operations happen entirely within the Tauri Rust process.

Depends on: `identity` (feature #4), all Phase 1 and Phase 2 features.

## Requirements
- [ ] Users can export their master private key as an encrypted file
- [ ] The export is encrypted with a user-provided passphrase using Argon2id KDF + XChaCha20-Poly1305 AEAD
- [ ] The exported file is a self-contained binary format with a version header, KDF parameters, nonce, and ciphertext
- [ ] Users can import an encrypted key backup on a new device by providing the file and passphrase
- [ ] Import replaces any existing key on the device (with confirmation)
- [ ] The passphrase is validated for minimum strength (at least 12 characters)
- [ ] The export file includes the master public key in cleartext (for identification without decryption)
- [ ] Export and import operations happen entirely in the Tauri core process — the passphrase is passed via IPC but the private key never reaches the React layer
- [ ] The file format is versioned to allow future changes to encryption parameters

## Design

### API / Interface Changes

No server-side API changes. Key recovery is entirely a client-side operation.

**Tauri IPC commands:**

| Command | Parameters | Returns | Description |
|---|---|---|---|
| `key_export` | `passphrase: string, path: string` | `Result<(), Error>` | Encrypt the master key and write to the specified file path |
| `key_import` | `passphrase: string, path: string` | `Result<PublicKeyInfo, Error>` | Read and decrypt a key backup file, store the key in the OS keychain |
| `key_export_validate_passphrase` | `passphrase: string` | `Result<PassphraseStrength, Error>` | Check passphrase strength before export |

**Exported file format (binary):**

```
Offset  Size  Field
0       4     Magic bytes: "DCKB" (DecentCom Key Backup)
4       1     Format version (1)
5       32    Master public key (Ed25519, cleartext — for identification)
37      4     Argon2id memory cost (KiB, little-endian u32)
41      4     Argon2id time cost (iterations, little-endian u32)
45      4     Argon2id parallelism (little-endian u32)
49      16    Argon2id salt (random)
65      24    XChaCha20-Poly1305 nonce (random)
89      48    Ciphertext (32-byte private key + 16-byte Poly1305 tag)
```

Total file size: 137 bytes.

### Data Model Changes

No server-side data model changes.

### Component Changes

**Client Tauri core (`client/src-tauri/src/`):**

- `client/src-tauri/src/crypto/backup.rs` — new module: key backup encryption and decryption
  - `encrypt_key(privkey, passphrase) -> Vec<u8>` — KDF + AEAD encryption, produces the backup file bytes
  - `decrypt_key(file_bytes, passphrase) -> Result<Ed25519PrivateKey, Error>` — parse, decrypt, verify
  - `read_backup_pubkey(file_bytes) -> Result<Ed25519PublicKey, Error>` — read the cleartext pubkey without decrypting
- `client/src-tauri/src/crypto/passphrase.rs` — new module: passphrase strength validation (length, entropy estimation)
- `client/src-tauri/src/commands/key_recovery.rs` — Tauri IPC command handlers for export/import
- `client/src-tauri/src/keystore.rs` — extend existing keystore module with `replace_key()` method for import

**Client React (`client/src/`):**

- `client/src/components/settings/KeyExport.tsx` — new component: export UI with passphrase input, strength indicator, file save dialog
- `client/src/components/settings/KeyImport.tsx` — new component: import UI with file picker, passphrase input, confirmation dialog
- `client/src/components/settings/SecuritySettings.tsx` — extend settings page to include key export/import sections

**Dependencies:**

- `argon2` crate — Argon2id KDF
- `chacha20poly1305` crate — XChaCha20-Poly1305 AEAD
- Both are likely already available via `ring` or can be added as standalone crates

## Task List

### Phase A: Core Encryption Module
- [ ] Add `argon2` and `chacha20poly1305` crate dependencies to `client/src-tauri/Cargo.toml`
- [ ] Create `client/src-tauri/src/crypto/backup.rs` with `encrypt_key()` function: generate salt and nonce, derive key with Argon2id, encrypt with XChaCha20-Poly1305, assemble the file format
- [ ] Implement `decrypt_key()` in the same module: parse the file format, derive key with Argon2id, decrypt and verify
- [ ] Implement `read_backup_pubkey()` for reading the cleartext public key from a backup file
- [ ] Create `client/src-tauri/src/crypto/passphrase.rs` with passphrase strength validation

### Phase B: Tauri IPC Commands
- [ ] Create `client/src-tauri/src/commands/key_recovery.rs` with `key_export` command: read key from keystore, encrypt, write to file
- [ ] Add `key_import` command: read file, decrypt, confirm replacement, store in keystore
- [ ] Add `key_export_validate_passphrase` command
- [ ] Extend `client/src-tauri/src/keystore.rs` with `replace_key()` that updates the OS keychain entry
- [ ] Register new commands in the Tauri command handler

### Phase C: Client UI
- [ ] Create `client/src/components/settings/KeyExport.tsx` — passphrase input with strength meter, file save dialog via Tauri's dialog API, success/error feedback
- [ ] Create `client/src/components/settings/KeyImport.tsx` — file picker, passphrase input, display the backup's public key for confirmation, import button with confirmation dialog
- [ ] Add key export/import sections to `client/src/components/settings/SecuritySettings.tsx`
- [ ] Add warning text explaining that losing both the device and the backup passphrase means permanent identity loss

## Test List
- [ ] Unit test: `encrypt_key` followed by `decrypt_key` round-trips successfully with the correct passphrase
- [ ] Unit test: `decrypt_key` with wrong passphrase returns a decryption error
- [ ] Unit test: `decrypt_key` with truncated file returns a parse error
- [ ] Unit test: `decrypt_key` with corrupted ciphertext returns an authentication error
- [ ] Unit test: `read_backup_pubkey` returns the correct public key without needing the passphrase
- [ ] Unit test: file format version byte is correctly set to 1
- [ ] Unit test: passphrase strength validation rejects passphrases shorter than 12 characters
- [ ] Unit test: Argon2id parameters in the file match the expected defaults
- [ ] Integration test: full export-import cycle via Tauri IPC commands — export on one keystore, import on a fresh keystore, verify the imported key matches
- [ ] Manual test: export a key, copy the file to a different machine, import it, and verify authentication works with the recovered identity

## Open Questions
- **Argon2id parameters:** What memory/time/parallelism values? Suggested: 256 MiB memory, 3 iterations, 4 parallelism. These should be strong but not so slow that the UX suffers on low-end hardware. Should they be configurable?
- **Cloud backup integration:** The design doc mentions iCloud/Google Drive as backup destinations. Should this feature include native cloud storage integration, or just produce a file that the user manually copies? Native integration is more convenient but adds significant platform-specific code.
- **Multiple key backups:** Should the export include device sub-keys in addition to the master key? The design doc keeps them separate, but importing a master key without device keys means re-authorizing all devices.
- **Backup reminder:** Should the client periodically remind users who haven't exported their key to do so? This is a UX consideration that could significantly improve key recovery rates.
