# Feature: Project Scaffolding

## Overview
Set up the foundational project structure: a Cargo workspace containing the server crate and a shared types crate, plus a Tauri v2 + React + TypeScript client application. This establishes the build tooling, directory layout, and dependency baseline that every subsequent feature builds on.

## Background
The design docs specify a Rust server (axum, tokio, sqlx), a Tauri v2 client with a React + TypeScript frontend styled with Tailwind CSS + Catppuccin, and shared type definitions used by both server and client Rust code. See [Architecture](../design/architecture.md) for the component diagram and [Overview](../design/overview.md) for the technology choices.

## Requirements
- [x] Cargo workspace at the repo root with members: `server`, `shared`
- [x] `server` crate compiles and runs a minimal axum HTTP server that responds to `GET /health` with `200 OK`
- [x] `shared` crate compiles and is a dependency of `server`
- [x] Tauri v2 application in `client/` with a React + TypeScript frontend bootstrapped via `create-tauri-app` or equivalent
- [x] Frontend uses pnpm as the package manager
- [x] Tailwind CSS is configured in the frontend
- [x] The Tauri Rust core (`client/src-tauri/`) depends on the `shared` crate
- [x] `cargo build` from the workspace root builds both `server` and `shared`
- [x] `pnpm dev` from `client/` launches the Tauri dev environment (requires desktop environment — manual verification)
- [x] CI-ready: `cargo clippy -- -D warnings` and `cargo test` pass with zero warnings on a clean build
- [x] `pnpm lint` and `pnpm test` pass in the client

## Design

### API / Interface Changes
A single health-check endpoint on the server:
- `GET /health` -> `200 OK` (plain text or JSON `{"status": "ok"}`)

A single Tauri IPC command to verify the bridge works:
- `ping` command returns `"pong"` from the Rust core to the React frontend

### Data Model Changes
None. No database or storage at this stage.

### Component Changes

**New files/directories:**

```
Cargo.toml                      # workspace root
server/
  Cargo.toml                    # axum, tokio, shared dependency
  src/
    main.rs                     # axum server with /health route
shared/
  Cargo.toml                    # serde, common types
  src/
    lib.rs                      # placeholder module
client/
  package.json                  # pnpm, React, TypeScript, Tailwind
  pnpm-lock.yaml
  tsconfig.json
  vite.config.ts
  tailwind.config.ts
  postcss.config.js
  src/
    main.tsx                    # React entry point
    App.tsx                     # Root component with ping test
    index.css                   # Tailwind directives
  src-tauri/
    Cargo.toml                  # tauri, shared dependency
    tauri.conf.json
    src/
      main.rs                   # Tauri setup, registers ping command
      lib.rs                    # IPC command definitions
```

## Task List
- [x] Create workspace `Cargo.toml` with members `server`, `shared`, and `client/src-tauri`
- [x] Create `shared` crate with `serde` and `serde_json` dependencies and a placeholder `lib.rs`
- [x] Create `server` crate with `axum`, `tokio`, and `shared` dependencies; implement `main.rs` with a `/health` endpoint
- [x] Initialize Tauri v2 + React + TypeScript app in `client/` using pnpm
- [x] Add `shared` as a path dependency in `client/src-tauri/Cargo.toml`
- [x] Configure Tailwind CSS in the frontend (`@tailwindcss/vite` plugin, `@import "tailwindcss"` in App.css)
- [x] Implement a `ping` IPC command in the Tauri Rust core and call it from `App.tsx` to verify the bridge
- [x] Add ESLint configuration for the frontend (`pnpm lint`)
- [x] Add Vitest configuration for the frontend (`pnpm test`)
- [x] Verify `cargo build`, `cargo test`, `cargo clippy -- -D warnings` all pass from workspace root
- [x] Verify `pnpm lint`, `pnpm test` all work from `client/`

## Test List
- [x] Unit test: `GET /health` returns 200 with expected body (axum test helpers)
- [x] Unit test: `shared` crate compiles and a trivial round-trip serde test passes
- [x] Unit test: Tauri `ping` command returns `"pong"`
- [x] Frontend test: `App.tsx` renders without errors (Vitest + React Testing Library)
- [x] Manual: `cargo build` from repo root succeeds
- [ ] Manual: `pnpm dev` from `client/` opens the Tauri window with the React app loaded (requires desktop environment — **not verified in this session**)
- [x] Manual: `cargo clippy -- -D warnings` produces no warnings

## Open Questions
- ~~Should the Tauri app's `src-tauri` Cargo project be a workspace member?~~ **Resolved:** Added as a workspace member (`client/src-tauri`). `cargo test` and `cargo clippy` cover all three crates from the root. Tauri's build pipeline works correctly with this setup.
- Should we add a `.github/workflows/ci.yml` as part of scaffolding, or defer CI setup to a separate task? (Deferred)
