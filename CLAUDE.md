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

## Implementation Order

Features should be implemented in milestone order as defined in `docs/design/overview.md`:

1. **Foundation** — server binary scaffolding, Tauri client shell, pubkey auth (challenge-response), basic text channels
2. **Core UX** — DMs, roles, permissions, server settings, invite system
3. **Voice & Video** — WebRTC voice channels, video, screen share (resolve SFU strategy first — see `docs/design/architecture.md` open questions)
4. **Federation** — cross-server identity, inter-server messaging
5. **Managed Hosting** — one-click deploy, billing, support tooling

Before starting any feature, check the relevant design doc for open questions that must be resolved first. Do not implement features that depend on unresolved architectural decisions without first documenting the decision in the design doc.

## Implementing Features

Follow this process for each feature:

1. **Read the design doc** for that milestone area before writing any code. Identify and resolve open questions.
2. **Start with the shared types** (`shared/` crate) — define the data model and wire protocol types that both server and client will use.
3. **Implement the server side** — REST endpoints in `server/`, WebSocket events, storage trait methods.
4. **Implement the Tauri core** — any privileged operations (signing, key access, file I/O) in the Tauri Rust core.
5. **Implement the React UI** — connect to the Tauri IPC and server WebSocket/REST APIs.
6. **Write tests** at each layer before moving to the next. Run `cargo clippy -- -D warnings` and `pnpm lint` before considering a feature complete.

### Skills Available

The following Claude Code skills are available and should be used where appropriate:

- **`/feature`** — create a GitHub issue for a new feature.
- **`/implement`** — implement a feature from its GitHub issue.
- **`/verify`** — verify a feature's implementation against its GitHub issue.

## Git Worktrees

When working on features or verifying pull requests, always use `git worktree` to avoid interfering with other active agents or the main development branch. This ensures each task has its own isolated environment.

- **Implementation:** Create a new worktree for the feature branch: `git worktree add ../feature-<number>-<slug> -b feature/<number>-<slug> main`.
- **Verification:** Create a new worktree to review a PR: `git worktree add ../verify-<number> <branch-name>`.
- **Cleanup:** Always remove the worktree when finished: `git worktree remove ../<dir-name>`.
