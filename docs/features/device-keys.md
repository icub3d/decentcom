# Feature: Device Sub-Keys

## Overview
Device sub-keys allow a user to authenticate from multiple devices without sharing their master private key. Each device generates its own Ed25519 key pair and receives a delegation certificate signed by the master key. Servers verify the delegation chain to accept device-key-signed operations. Devices can be listed and individually revoked without affecting the user's identity.

## Background
The identity design doc (`docs/design/identity.md`, "Multiple Devices" section) defines the device key model. A device key is authorized by a delegation certificate signed by the master key:

```
master_privkey signs {
  device_pubkey,
  device_name,
  issued_at,
  expires_at (optional),
  permissions (e.g. "no key management")
}
```

The server accepts operations signed by a device key if it can verify the delegation chain back to the trusted master public key. This means the master key can stay offline after initial setup, and a compromised device key has limited blast radius.

Phase 1 identity (#4) and auth (#5) established master key generation and challenge-response authentication. This feature extends that to support device keys alongside master keys.

## Requirements
- [ ] A device generates its own Ed25519 key pair on first launch (stored in the OS keychain via Tauri)
- [ ] The master key signs a delegation certificate for the device key, including device name, issued_at, optional expires_at, and permission scope
- [ ] The delegation certificate is a well-defined, canonical binary format (to ensure deterministic signing)
- [ ] The client can submit the delegation certificate to any server the user is a member of
- [ ] The server stores active delegation certificates per user and validates them
- [ ] Authentication can proceed using either the master key or a device key (with its delegation certificate)
- [ ] When authenticating with a device key, the server verifies: (a) the device key signed the challenge, (b) the delegation certificate is signed by the master key, (c) the certificate has not expired, (d) the device key has not been revoked
- [ ] The user can list all active device keys from any device that holds the master key
- [ ] The user can revoke a device key (requires master key signature)
- [ ] Revoked device keys are immediately rejected on subsequent auth attempts
- [ ] Device key permissions can restrict operations (e.g., a device key that cannot manage other device keys)

## Design

### API / Interface Changes

**REST endpoints** (all under `/api/v1/`):

| Method | Path | Description | Required Permission |
|--------|------|-------------|---------------------|
| POST | `/devices` | Register a device delegation certificate | Authenticated (master key) |
| GET | `/devices` | List active device keys for the authenticated user | Authenticated |
| DELETE | `/devices/:device_pubkey` | Revoke a device key | Authenticated (master key) |

**POST `/devices` request body:**
```json
{
  "delegation_certificate": {
    "device_pubkey": "<base58>",
    "device_name": "Laptop",
    "issued_at": "2026-04-10T00:00:00Z",
    "expires_at": "2027-04-10T00:00:00Z",
    "permissions": ["sign_messages", "sign_auth"],
    "master_pubkey": "<base58>",
    "signature": "<base64 of master key signature over canonical cert bytes>"
  }
}
```

**Modified auth flow:** The `POST /auth/verify` endpoint accepts an additional optional field:

```json
{
  "pubkey": "<device_pubkey>",
  "signature": "<device signature of challenge>",
  "delegation_certificate": { ... }
}
```

If `delegation_certificate` is present, the server verifies the chain instead of requiring the pubkey to match a known master key directly.

**Gateway events:**
- `DEVICE_ADD` — new device registered (sent to the user's other sessions)
- `DEVICE_REVOKE` — device revoked (sent to all sessions; revoked device's session is terminated)

**Tauri IPC commands:**
- `generate_device_key` — generate a new Ed25519 key pair for this device and store in keychain
- `create_delegation_certificate` — sign a delegation cert with the master key (requires master key to be available on this device)
- `sign_with_device_key` — sign a challenge with the device's key
- `get_device_pubkey` — return this device's public key

### Data Model Changes

**New table:**

```sql
CREATE TABLE device_keys (
    device_pubkey   TEXT NOT NULL,
    user_id         TEXT NOT NULL REFERENCES users(id),
    device_name     TEXT NOT NULL,
    certificate     BLOB NOT NULL,  -- full serialized delegation certificate
    master_signature BLOB NOT NULL, -- master key signature over certificate
    issued_at       TEXT NOT NULL,
    expires_at      TEXT,           -- nullable
    revoked_at      TEXT,           -- nullable; set on revocation
    PRIMARY KEY (device_pubkey, user_id)
);
```

### Component Changes

**Server (`server/`):**

- `server/src/models/device_key.rs` — DeviceKey, DelegationCertificate structs; canonical serialization format for signing
- `server/src/services/delegation.rs` — Delegation certificate verification logic: parse cert, verify master signature, check expiry, check revocation
- `server/src/store/device_key_store.rs` — `DeviceKeyStore` trait
- `server/src/store/sqlite/device_key_store.rs` — SQLite implementation
- `server/src/routes/devices.rs` — REST handlers for device registration, listing, revocation
- Modify `server/src/routes/auth.rs` — Extend challenge-response to accept device key + delegation certificate
- Modify `server/src/middleware/auth.rs` — Session tokens issued to device keys carry the user's identity (master pubkey) as the principal
- Modify `server/src/gateway/mod.rs` — On device revocation, terminate any active sessions for that device key

**Shared types (`shared/`):**

- `shared/src/delegation.rs` — DelegationCertificate struct and canonical serialization shared between server and client Rust code

**Client (Tauri core, `client/src-tauri/`):**

- `client/src-tauri/src/commands/device_key.rs` — Tauri IPC commands for device key generation, delegation cert creation, device signing
- Modify `client/src-tauri/src/commands/auth.rs` — Support auth flow with device key: sign challenge with device key, attach delegation certificate

**Client (React, `client/src/`):**

- `client/src/api/devices.ts` — API client functions for device endpoints
- `client/src/components/settings/DeviceList.tsx` — List of registered devices with revoke button
- `client/src/components/settings/AddDevice.tsx` — Flow for registering the current device (generate key, create cert, submit to server)

**Database migrations:**

- `server/migrations/NNNN_create_device_keys.sql`

## Task List

### Shared
- [ ] Define DelegationCertificate struct with canonical binary serialization (deterministic field order, no padding) in `shared/src/delegation.rs`
- [ ] Implement serialization and deserialization for the certificate format
- [ ] Add tests for round-trip serialization

### Server
- [ ] Define DeviceKey model struct
- [ ] Add `DeviceKeyStore` trait to the storage trait hierarchy
- [ ] Write SQLite migration to create `device_keys` table
- [ ] Implement `DeviceKeyStore` for the SQLite backend
- [ ] Implement delegation certificate verification service: parse cert, verify master key signature over canonical bytes, check expiry, check revocation status
- [ ] Implement POST `/devices` handler: validate cert signature, store delegation
- [ ] Implement GET `/devices` handler: list active (non-revoked, non-expired) device keys for the authenticated user
- [ ] Implement DELETE `/devices/:device_pubkey` handler: mark as revoked (requires master key auth)
- [ ] Extend POST `/auth/challenge` to accept device pubkeys (challenge issued per pubkey, works for any key)
- [ ] Extend POST `/auth/verify` to accept device key signature + delegation certificate; verify chain before issuing session token
- [ ] Ensure session tokens issued via device key carry the master pubkey as the user identity
- [ ] Broadcast DEVICE_ADD and DEVICE_REVOKE events via gateway (sent to user's sessions via `send_to_user`)

### Client (Tauri core)
- [ ] Implement `generate_device_key` IPC command: generate Ed25519 key pair, store in OS keychain
- [ ] Implement `create_delegation_certificate` IPC command: construct cert struct, sign with master key, return signed cert
- [ ] Implement `sign_with_device_key` IPC command: sign arbitrary bytes with the device key
- [ ] Implement `get_device_pubkey` IPC command
- [ ] Extend auth flow to detect whether master key or device key is available and use the appropriate auth path

### Client (React)
- [ ] Add devices API client functions
- [ ] Build AddDevice flow: generate device key, create delegation cert (if master key available on this device), submit cert to server
- [ ] Build DeviceList in user settings showing all registered devices
- [ ] Add revoke button per device (calls DELETE /devices/:device_pubkey)
- [ ] Handle DEVICE_ADD and DEVICE_REVOKE gateway events

## Test List
- [ ] Unit: canonical serialization of DelegationCertificate is deterministic
- [ ] Unit: delegation certificate verification succeeds with valid master signature
- [ ] Unit: delegation certificate verification fails with wrong master signature
- [ ] Unit: expired delegation certificate is rejected
- [ ] Unit: revoked device key is rejected during auth
- [ ] Integration: register device key via POST /devices, verify it appears in GET /devices
- [ ] Integration: authenticate with device key + delegation certificate, receive valid session token
- [ ] Integration: revoke device key, verify subsequent auth attempts fail
- [ ] Integration: session token from device key auth identifies the user by master pubkey
- [ ] Integration: DEVICE_ADD event is sent to other sessions on registration
- [ ] Integration: DEVICE_REVOKE event terminates the revoked device's session
- [ ] Manual: add device flow works end-to-end in the Tauri client
- [ ] Manual: device list shows all devices with names and can revoke

## Open Questions
- Should device keys be able to authorize new device keys (transitive delegation), or only the master key can issue device keys? The design doc flags this as an open question. Recommendation: start with master-key-only delegation for simplicity and security.
- What is the canonical serialization format for delegation certificates? Options: CBOR, MessagePack, or a custom deterministic binary encoding. CBOR with deterministic encoding (RFC 7049 Section 3.9) is a reasonable choice.
- How should the "add device" flow work when the master key is on a different physical device? A QR code or one-time pairing code shown on the master device could be scanned by the new device. This may be complex enough to defer to a follow-up feature.
- Should device key permissions be a fixed set of flags or an extensible list? A fixed set is simpler and sufficient for now.
