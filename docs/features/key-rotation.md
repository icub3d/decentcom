# Feature: Key Rotation

## Overview
Key rotation allows a user to migrate their identity from an old Ed25519 key pair to a new one. The user signs a rotation declaration with the old key, proving they control the current identity, and the server migrates all associated data (membership, roles, messages) to the new public key. The old key is added to a revocation list so it cannot be used for authentication.

## Background
The identity design doc (`docs/design/identity.md`) describes the rotation approach: the user signs a `{new_pubkey, timestamp, reason}` declaration with the old key, servers accept the declaration and migrate the identity, and the old key is revoked. The doc raises an open question about a time-delay grace period for rotation. Key rotation is the hardest problem in the identity model because it requires server cooperation and cannot be done if the old private key is lost (that scenario requires creating a new identity entirely).

Depends on: `identity` (feature #4), `auth` (feature #5), `device-keys` (feature #15), all Phase 1 and Phase 2 features.

## Requirements
- [ ] Users can initiate key rotation from the client, generating a new key pair and signing a rotation declaration with the old key
- [ ] The rotation declaration contains: old public key, new public key, timestamp, and reason
- [ ] The declaration is signed by the old private key
- [ ] The server verifies the declaration signature and migrates the user's identity to the new public key
- [ ] All server-side records referencing the old public key are updated to the new public key
- [ ] The old public key is added to a per-server revocation list
- [ ] Authentication attempts with a revoked key are rejected with a clear error message indicating the key has been rotated
- [ ] Existing sessions for the old key are invalidated
- [ ] Device sub-keys delegated by the old master key are revoked (user must re-delegate with the new master key)
- [ ] The rotation declaration is stored for audit purposes
- [ ] The client stores the new key pair in the OS keychain, replacing the old one

## Design

### API / Interface Changes

**REST endpoints:**

| Method | Path | Description |
|---|---|---|
| POST | `/api/v1/identity/rotate` | Submit a signed rotation declaration |
| GET | `/api/v1/identity/revocations` | List revoked public keys for this server |

**POST `/api/v1/identity/rotate` request body:**

```json
{
  "old_pubkey": "<base58>",
  "new_pubkey": "<base58>",
  "timestamp": "2026-04-10T12:00:00Z",
  "reason": "routine rotation",
  "signature": "<base64 — old_privkey signs (old_pubkey || new_pubkey || timestamp || reason)>"
}
```

**Response (success):**

```json
{
  "status": "rotated",
  "new_session_token": "<token for the new key>"
}
```

**Tauri IPC commands:**

| Command | Parameters | Returns | Description |
|---|---|---|---|
| `key_rotate` | `reason: string` | `Result<RotationResult, Error>` | Generate new key pair, sign rotation declaration with old key, submit to all connected servers |
| `key_rotation_status` | none | `Result<RotationHistory, Error>` | List past rotations (stored locally) |

### Data Model Changes

**New table: `revoked_keys`**

| Column | Type | Description |
|---|---|---|
| `id` | INTEGER/BIGSERIAL | Primary key |
| `old_pubkey` | BLOB/BYTEA | The revoked public key |
| `new_pubkey` | BLOB/BYTEA | The replacement public key |
| `declaration` | BLOB/BYTEA | The full signed rotation declaration |
| `rotated_at` | TIMESTAMP | When the rotation was processed |

**Modified `users` table:** The `pubkey` column is updated in place. The old pubkey is only preserved in the `revoked_keys` table.

**Modified `device_keys` table:** All device delegation certificates for the old master key are deleted (they were signed by the old key and are no longer valid).

### Component Changes

**Server (`server/`):**

- `server/src/routes/identity.rs` — new route module with the `rotate` and `revocations` endpoints
- `server/src/models/revocation.rs` — new model: `RevokedKey` struct, rotation declaration parsing and signature verification
- `server/src/storage/trait.rs` — extend `UserStore` trait with `rotate_key()` and `is_key_revoked()` methods
- `server/src/storage/sqlite/users.rs` — implement `rotate_key()`: update `users.pubkey`, insert into `revoked_keys`, delete device keys, invalidate sessions
- `server/src/storage/postgres/users.rs` — same implementation for PostgreSQL (if postgres feature is already built)
- `server/src/auth.rs` — check revocation list during authentication; return specific error for revoked keys
- `server/src/gateway/handler.rs` — disconnect active WebSocket sessions for the old key after rotation

**Client Tauri core (`client/src-tauri/src/`):**

- `client/src-tauri/src/commands/key_rotation.rs` — Tauri IPC command handler: generate new key pair, construct and sign the rotation declaration, call the server endpoint, update the local keychain
- `client/src-tauri/src/crypto/rotation.rs` — rotation declaration construction and signing logic
- `client/src-tauri/src/keystore.rs` — extend with `rotate_key()` method that generates new key, stores it, and archives the old key locally

**Client React (`client/src/`):**

- `client/src/components/settings/KeyRotation.tsx` — new component: rotation UI with reason input, confirmation dialog, progress indicator, success/error feedback
- `client/src/components/settings/SecuritySettings.tsx` — extend to include key rotation section

## Task List

### Phase A: Server Rotation Handling
- [ ] Create `revoked_keys` table in SQLite and PostgreSQL migration files
- [ ] Create `server/src/models/revocation.rs` — `RotationDeclaration` struct with signature verification
- [ ] Extend `UserStore` trait with `rotate_key(old_pubkey, new_pubkey, declaration)` and `is_key_revoked(pubkey)` methods
- [ ] Implement `rotate_key()` for SQLite: update `users.pubkey`, insert `revoked_keys`, delete `device_keys` for old master, invalidate sessions
- [ ] Implement `is_key_revoked()` for SQLite
- [ ] Create `server/src/routes/identity.rs` with the `rotate` endpoint: validate declaration, verify signature, call `rotate_key()`
- [ ] Add `revocations` endpoint to list revoked keys
- [ ] Modify authentication flow in `server/src/auth.rs` to check revocation list and return a specific error for revoked keys
- [ ] Disconnect active WebSocket sessions for the old key after successful rotation

### Phase B: Client Rotation Logic
- [ ] Create `client/src-tauri/src/crypto/rotation.rs` — build rotation declaration bytes and sign with old key
- [ ] Create `client/src-tauri/src/commands/key_rotation.rs` — `key_rotate` command: generate new key pair, sign declaration, submit to server, update local keychain on success
- [ ] Extend keystore with `rotate_key()`: generate new key pair, store new key, keep a local record of the old public key for reference
- [ ] Handle multi-server rotation: iterate over all servers the user is connected to and submit the rotation declaration to each

### Phase C: Client UI
- [ ] Create `client/src/components/settings/KeyRotation.tsx` — rotation form with reason field, warning about device key revocation, confirmation dialog
- [ ] Add progress indicator showing rotation status per server
- [ ] Display error if rotation fails on any server (partial rotation state)
- [ ] Add key rotation section to `client/src/components/settings/SecuritySettings.tsx`
- [ ] After rotation, prompt user to re-export their key backup with the new key

## Test List
- [ ] Unit test: rotation declaration signature verification accepts a valid declaration
- [ ] Unit test: rotation declaration signature verification rejects a tampered declaration
- [ ] Unit test: rotation declaration signature verification rejects a declaration signed by a different key
- [ ] Unit test: `rotate_key()` updates the user's pubkey in the database
- [ ] Unit test: `rotate_key()` inserts a record into `revoked_keys`
- [ ] Unit test: `rotate_key()` deletes device keys associated with the old master key
- [ ] Unit test: `is_key_revoked()` returns true for a revoked key and false for an active key
- [ ] Unit test: authentication with a revoked key returns the expected revocation error
- [ ] Integration test: full rotation flow — client generates declaration, server processes it, old key is rejected, new key authenticates successfully
- [ ] Integration test: active WebSocket session is disconnected after rotation
- [ ] Integration test: device sub-keys for the old master key no longer authenticate after rotation
- [ ] Manual test: rotate key, verify all servers accept the new key, verify old key is rejected everywhere

## Open Questions
- **Grace period:** Should there be a configurable delay (e.g., 24 hours) before the old key is fully revoked, during which the old key can cancel the rotation? This protects against an attacker who gained temporary access. The identity design doc raises this question.
- **Partial rotation failure:** If rotation succeeds on some servers but fails on others, the user has different identities on different servers. How should the client handle this? Retry logic? Manual resolution UI?
- **Message re-signing:** Messages signed by the old key are still valid and verifiable. Should they be re-signed with the new key, or is it acceptable that old messages reference the old pubkey? Re-signing is expensive and may not be necessary if the revocation record provides the link.
- **Federation implications:** When federation is implemented, key rotation declarations may need to propagate across servers. This is out of scope now but should be considered in the declaration format.
