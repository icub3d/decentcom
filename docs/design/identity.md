# User Identity

## Model

Every decentcom user is identified by an **Ed25519 public key**. This key is the user's persistent, portable identity across all servers. A username and avatar are metadata that the user can set per-server, but the underlying identity is always the key pair.

Why Ed25519:
- Small key and signature sizes (32-byte public key, 64-byte signature)
- Fast signing and verification
- Widely audited, used in SSH, Signal, TLS 1.3
- Available in Rust via `ring` or `ed25519-dalek`

## Key Generation

The first time the client is launched, it generates an Ed25519 key pair. This is the **master key pair**.

```
master_privkey  →  master_pubkey  (= user identity)
```

The master public key is the canonical user ID. It can be displayed as a human-readable fingerprint (e.g. Base58 or Bech32 encoding) for users to share and verify out-of-band.

### Key Storage

The private key is stored by the Tauri core:
- **macOS:** Keychain
- **Linux:** `libsecret` / GNOME Keyring, or an encrypted file if unavailable
- **Windows:** Windows Credential Manager

The private key is **never** written to disk in plaintext and **never** exposed to the React frontend layer.

## Authentication

Authentication is a challenge-response protocol:

1. Client sends its public key to the server.
2. Server generates a random nonce (challenge) and returns it.
3. Tauri core signs the nonce with the master private key.
4. Client sends the signature to the server.
5. Server verifies the signature against the stored public key.
6. On success, the server issues a session token for that connection.

The server stores only the public key. There is no password hash, no secret, nothing useful to steal from the server's user table.

## Multiple Devices

Each device has its own Ed25519 key pair (the **device key**). The device key is authorized by signing a delegation certificate with the master key:

```
master_privkey signs {
  device_pubkey,
  device_name,
  issued_at,
  expires_at (optional),
  permissions (e.g. "no key management")
}
```

The server accepts messages signed by a device key if it can verify the delegation chain back to the trusted master key. The server stores both the master public key and the set of active device delegation certificates.

This means:
- The master key can stay offline (hardware key, paper backup) after initial setup.
- Individual devices can be revoked without changing the user's identity.
- A compromised device key has limited blast radius.

**Open question:** Should device keys be able to authorize new device keys (transitive delegation), or only the master key can issue device keys?

## Key Recovery

Lost private key = lost identity, unless recovery mechanisms are in place. Options (not mutually exclusive):

### 1. Recovery Seed Phrase
At key generation time, the user is presented with a 24-word BIP39-style mnemonic that encodes the master private key (or a seed to derive it). The user writes it down and stores it securely.

This is the "bitcoin wallet" model. Simple, robust, requires no infrastructure. UX must strongly encourage users to write this down before proceeding.

### 2. Encrypted Key Backup
The master key is encrypted with a strong passphrase and backed up to a location the user chooses:
- Local file
- Cloud storage (iCloud, Google Drive, etc.) — user's own account
- A decentcom managed backup service (optional, for hosted users)

The backup is useless without the passphrase, so the passphrase becomes the recovery credential.

### 3. Social Recovery (Shamir Secret Sharing)
The master private key is split into N shares using Shamir's Secret Sharing. The user distributes shares to trusted contacts. Recovery requires any M-of-N shares.

This removes single points of failure and avoids reliance on any one person or service. It is more complex to implement and explain to users.

**Recommendation:** Ship with seed phrase (Option 1) and encrypted key backup (Option 2) at launch. Social recovery is a compelling future feature but requires significant UX investment.

## Key Rotation

If a master key is compromised (not just lost), the user needs to rotate to a new key. This is the hardest problem in the model:

- A new key pair has no history and no trust on any server.
- The old key (if not revoked) can still be used by an attacker.

Rotation approach:
1. User signs a "rotation declaration" with the old key: `{new_pubkey, timestamp, reason}`.
2. Servers that trust the old key can accept this declaration and migrate the identity.
3. The old key is added to a per-server revocation list.

This requires servers to implement rotation handling. It cannot be done if the user has lost the old private key (rotation requires proving you previously controlled the identity). This is why recovery mechanisms above are important.

**Open question:** Should there be a time-delay on key rotation (e.g. 24-hour grace period where the old key can cancel the rotation) to protect against attackers who gained temporary key access?

## Server-Side Identity Records

What a server stores per user:

```
users {
  id              -- internal row ID
  pubkey          -- Ed25519 public key (bytes or base58)
  display_name    -- user-chosen display name for this server
  avatar_hash     -- hash of avatar image (image stored separately)
  joined_at
  device_keys[]   -- active device delegation certificates
}
```

No email. No phone number. No password. The pubkey is the only required identity field.

**Open question:** Should we support optional, verified email for account recovery notifications? (e.g. "a key rotation was requested for your account"). This adds a small centralization element but may significantly help average users.
