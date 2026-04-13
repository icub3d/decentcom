# Feature: Authentication

## Overview
Implement the Ed25519 challenge-response authentication protocol between the client and server. The client sends its public key, the server returns a nonce challenge, the Tauri core signs the nonce, and the server verifies the signature and issues a session token. On first authentication, the server automatically registers the user. This replaces traditional username/password auth with cryptographic identity verification.

## Background
The authentication flow is defined in the architecture doc ([architecture.md](../design/architecture.md)) and the identity doc ([identity.md](../design/identity.md)). The server never stores passwords or secrets -- only public keys. Session tokens are short-lived. The storage layer (feature #3) provides `UserStore` and `SessionStore` for persistence. The identity feature (#4) provides the client-side key pair and signing capability.

## Requirements
- [x] `POST /api/v1/auth/challenge` accepts a public key and returns a nonce challenge
- [x] Challenges are stored server-side with a short TTL (e.g., 5 minutes) and are single-use
- [x] `POST /api/v1/auth/verify` accepts a public key and signature, verifies the signature against the stored challenge, and returns a session token
- [x] If the public key is not yet registered, the server creates a new user record automatically on successful verification
- [x] Session tokens are opaque random strings (not JWTs) stored in the `sessions` table
- [x] Session tokens have a configurable expiry (default: 24 hours)
- [x] Authenticated endpoints validate the session token via an `Authorization: Bearer <token>` header
- [x] An axum middleware (extractor) validates session tokens and injects the authenticated user into request handlers
- [x] Invalid or expired tokens return `401 Unauthorized`
- [ ] The client calls the auth flow automatically on connecting to a server, using the Tauri core to sign the challenge
- [x] Challenge nonces are cryptographically random (at least 32 bytes)

## Design

### API / Interface Changes

**New REST endpoints:**

#### `POST /api/v1/auth/challenge`
Request:
```json
{
  "pubkey": "<base58-encoded Ed25519 public key>"
}
```
Response (200):
```json
{
  "challenge": "<base64-encoded random nonce>"
}
```
Errors:
- `400 Bad Request` -- invalid public key format

#### `POST /api/v1/auth/verify`
Request:
```json
{
  "pubkey": "<base58-encoded Ed25519 public key>",
  "signature": "<base64-encoded Ed25519 signature of the challenge>"
}
```
Response (200):
```json
{
  "token": "<opaque session token>",
  "user_id": "<user ID>",
  "expires_at": "<ISO 8601 timestamp>"
}
```
Errors:
- `400 Bad Request` -- missing fields or invalid format
- `401 Unauthorized` -- signature verification failed or challenge expired/not found

### Data Model Changes

**New table (added to existing migration or a new migration):**
```sql
CREATE TABLE challenges (
    id         TEXT PRIMARY KEY,
    pubkey     TEXT NOT NULL,
    nonce      BLOB NOT NULL,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    expires_at TEXT NOT NULL
);

CREATE INDEX idx_challenges_pubkey ON challenges(pubkey);
CREATE INDEX idx_challenges_expires ON challenges(expires_at);
```

Alternatively, challenges can be stored in-memory (a `DashMap` or `tokio::sync::RwLock<HashMap>`) since they are short-lived. An in-memory approach is simpler and avoids database writes for every auth attempt.

### Component Changes

**New files:**
```
server/src/auth/
  mod.rs                    # Module exports
  handlers.rs               # Axum handlers for /auth/challenge and /auth/verify
  middleware.rs              # Session token validation extractor (AuthUser)
  challenge.rs              # Challenge store (in-memory with TTL cleanup)
  session.rs                # Session token generation and validation logic
shared/src/auth.rs          # Request/response types shared between server and client
client/src/api/auth.ts      # Client-side auth API calls
client/src/hooks/useAuth.ts # React hook for authentication state
```

**Modified files:**
```
server/Cargo.toml           # Add: ed25519-dalek, base58, base64, rand, dashmap
server/src/main.rs          # Mount auth routes, initialize challenge store
server/src/storage/traits.rs # Possibly no changes if using in-memory challenge store
client/src-tauri/src/lib.rs # Add auth-related IPC commands if needed
client/src/App.tsx          # Integrate auth flow into server connection
```

**Auth middleware extractor:**
```rust
// server/src/auth/middleware.rs
pub struct AuthUser {
    pub user_id: String,
    pub pubkey: String,
}

// Implements axum::extract::FromRequestParts
// Extracts and validates the Bearer token from the Authorization header
// Looks up the session in SessionStore
// Returns 401 if missing, invalid, or expired
```

**Client auth flow (in `client/src/api/auth.ts`):**
1. Call `POST /api/v1/auth/challenge` with the public key
2. Receive the challenge nonce
3. Call the Tauri `sign` IPC command with the nonce
4. Call `POST /api/v1/auth/verify` with the public key and signature
5. Store the returned session token in memory for subsequent API calls
6. Attach the token as `Authorization: Bearer <token>` to all subsequent requests

## Task List

### Server-side
- [x] Add `ed25519-dalek`, `bs58`, `base64`, `rand`, `dashmap` to `server/Cargo.toml`
- [x] Define shared auth request/response types in `shared/src/auth.rs` (`ChallengeRequest`, `ChallengeResponse`, `VerifyRequest`, `VerifyResponse`)
- [x] Implement in-memory challenge store in `server/src/auth/challenge.rs` with TTL-based expiry and background cleanup task
- [x] Implement `POST /api/v1/auth/challenge` handler: validate pubkey format, generate 32-byte random nonce, store challenge, return nonce
- [x] Implement `POST /api/v1/auth/verify` handler: look up challenge by pubkey, verify Ed25519 signature, consume challenge (single-use), create or fetch user, create session, return token
- [x] Implement session token generation (session token is generated by SessionStore using double-ULID scheme from the storage feature)
- [x] Implement `AuthUser` extractor in `server/src/auth/middleware.rs`
- [x] Mount auth routes under `/api/v1/auth/` in the router
- [x] Add a protected test endpoint (`GET /api/v1/auth/me`) that returns the authenticated user's info

### Client-side
- [x] Create `client/src/api/auth.ts` with functions for the challenge-response flow
- [x] Create `useAuth` hook that manages authentication state (unauthenticated, authenticating, authenticated)
- [ ] Wire the auth flow into the server connection process: after entering a server address, run the challenge-response automatically (deferred to client-shell feature)
- [x] Store the session token in React state via useAuth hook
- [ ] Handle token expiry: detect 401 responses and re-run the auth flow (deferred to client-shell feature)

## Test List
- [x] Unit test (server): challenge store creates and retrieves challenges
- [x] Unit test (server): expired challenges are not returned (purge_expired_removes_stale_entries)
- [x] Unit test (server): challenges are single-use (second retrieval fails)
- [x] Unit test (server): valid Ed25519 signature is accepted by verify handler
- [x] Unit test (server): invalid signature is rejected with 401
- [x] Unit test (server): verify with unknown challenge returns 401
- [x] Unit test (server): first-time pubkey creates a new user record
- [x] Unit test (server): returning pubkey does not create a duplicate user (first_time_pubkey_creates_user)
- [x] Unit test (server): `AuthUser` extractor returns user for valid token
- [x] Unit test (server): `AuthUser` extractor returns 401 for missing/invalid/expired token
- [x] Unit test (server): `GET /api/v1/auth/me` returns current user when authenticated
- [x] Integration test: full auth flow -- challenge, sign, verify, access protected endpoint
- [x] Unit test (React): `useAuth` hook transitions through authentication states correctly
- [ ] Manual: connect to a running server from the client, observe successful authentication
- [ ] Manual: stop server, restart, verify client re-authenticates automatically

## Implementation Notes

- Challenges are stored in an in-memory `DashMap` (not the database). The `ChallengeStore` is `Arc`-backed and held directly in `AppState`.
- Session token generation reuses the double-ULID scheme from the storage feature's `SessionStore`; no separate `session.rs` file was needed.
- The `AuthUser` extractor does ISO string comparison for expiry checking (lexicographic string ordering works correctly for ISO 8601 timestamps). 
- The `GET /api/v1/auth/me` endpoint omits pubkey lookup for now — it only returns `user_id`. Pubkey will be added when the profiles feature is implemented.
- Client-side token expiry detection and automatic re-auth are deferred to the client-shell feature where the HTTP client layer is established.
- `server.src.config.auth.session_ttl_seconds` was added so session expiry is configurable with a 24-hour default.
- The current client identity signing IPC signs UTF-8 strings, so the server verifies signatures against the base64 challenge string that is returned by `/auth/challenge`.

## Open Questions
- Should challenges be stored in-memory (simpler, lost on restart) or in the database (persists across restarts but adds write load)? In-memory is recommended since challenges are short-lived and a server restart just means clients re-authenticate.
- The architecture doc mentions JWTs as an alternative to opaque tokens. Opaque tokens are simpler and more revocable. Should we leave the door open for JWTs later, or commit to opaque tokens? Starting with opaque tokens and the `SessionStore` trait makes it easy to swap later.
- Should the server enforce a rate limit on `/auth/challenge` to prevent nonce-flooding? This is a good idea but may be better addressed by a general rate-limiting middleware in a later feature.
- Should the auto-registration on first auth be configurable? In `allowlist` membership mode, the server should reject unknown public keys rather than auto-registering them. The membership mode check should gate auto-registration.
