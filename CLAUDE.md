# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

decentcom is a self-hostable, decentralized Discord alternative. Users are identified by Ed25519 public keys (no passwords). Each server instance is independently operated. See `docs/design/` for architecture and design decisions.

## Stack

- **Backend:** Rust — `server/` (axum, tokio, sqlx)
- **Frontend:** Tauri v2 + React + TypeScript — `client/`
- **Styling:** Tailwind CSS + Catppuccin themes (Mocha default)
- **Auth:** Ed25519 challenge-response; private keys never leave the Tauri core process
- **Realtime:** WebSockets
- **Voice/Video:** WebRTC

## Commands

```
# Backend (run from repo root)
cargo build                  # build all workspace crates
cargo test                   # run all server + shared + Tauri core tests
cargo test <test_name>       # run a single test
cargo clippy -- -D warnings  # lint (zero warnings enforced)

# Frontend (run from client/)
pnpm install                 # install deps
pnpm dev                     # start Tauri dev (hot reload)
pnpm build                   # production build
pnpm lint                    # ESLint
pnpm test                    # Vitest (run once)
pnpm test -- --watch         # Vitest watch mode
```

## Architecture Notes

- The Tauri core (Rust) holds the private key and does all signing. The React app never touches private key material — it sends data to be signed via Tauri IPC and receives signatures back.
- The storage layer is abstracted behind a Rust trait. SQLite + local disk is the default; PostgreSQL + S3-compatible object storage is the scale-out option.
- Authentication is a two-step challenge-response: client sends pubkey → server returns nonce → client signs nonce → server issues session token.
- Each community ("server" in Discord terms) runs on a single decentcom instance. One instance = one community.

## Design Documents

Before making significant architectural decisions, check `docs/design/` — several open questions are documented there and should be resolved before implementation:

- `docs/design/overview.md` — vision, non-goals, milestone plan
- `docs/design/architecture.md` — component diagram, auth flow, open questions
- `docs/design/identity.md` — key generation, device sub-keys, recovery options
- `docs/design/server-model.md` — membership modes, feature flags, permissions
- `docs/design/storage.md` — backend options, media storage, retention
