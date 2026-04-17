# decentcom

Decentralized Communication — open-source, self-hostable community software where your identity is a cryptographic key pair you own and no central authority controls your server.

"Decent" in the name is intentional: a decent (good) way to communicate, built on decentralized infrastructure.

## Philosophy

- **No central authority.** Servers are fully self-managed. There is no decentcom.io account required to run or join a server.
- **Users own their identity.** Authentication is built on public key cryptography. Servers store only your public key — never a password or credential.
- **Server operators are in control.** Each server chooses its own policies: open or invite-only, which features are enabled, how data is stored, and what content is allowed.
- **Open source first.** The software is MIT-licensed. A managed hosting service will eventually be offered as a revenue model, but the core will always be free and open.

## Tech Stack

| Layer | Technology |
|---|---|
| Backend | Rust (axum, tokio, sqlx) |
| Desktop client | Tauri v2 + React + TypeScript |
| Styling | Tailwind CSS + Catppuccin themes (Mocha default) |
| Auth | Ed25519 public key cryptography (no passwords) |
| Realtime | WebSockets |
| Voice / Video | WebRTC |

## Design Documents

Architecture and feature decisions are documented in [`docs/design/`](docs/design/):

- [Overview & Vision](docs/design/overview.md) — goals, non-goals, comparison with alternatives
- [Architecture](docs/design/architecture.md) — system components and data flow
- [User Identity](docs/design/identity.md) — public key auth, multi-device, key recovery
- [Server Model](docs/design/server-model.md) — self-hosting, federation, server configuration
- [Storage Backends](docs/design/storage.md) — SQLite, PostgreSQL, cloud, Kubernetes

## Prerequisites

To develop decentcom you will need:

- [Rust](https://rustup.rs/) (stable toolchain)
- [Node.js](https://nodejs.org/) 20+ and `pnpm`
- [Tauri prerequisites](https://tauri.app/start/prerequisites/) for your platform (system WebView, build tools)

## Project Structure

```
decentcom/
├── server/          # Rust backend (axum HTTP + WebSocket server)
├── client/          # Tauri + React frontend
│   ├── src/         # React app (TypeScript)
│   ├── src-tauri/   # Tauri host (Rust)
│   └── public/
├── docs/
│   └── design/      # Architecture and design documents
└── README.md
```

## Development

The fastest way to develop both the client and server is using [Overmind](https://github.com/DarthSim/overmind). We have a `Procfile` configured to run three different test servers simultaneously:

- **Open Server** (port 8081): Open membership mode.
- **Private Server** (port 8082): Invite-only mode.
- **Strict Server** (port 8083): Allowlist mode, restricted features.
- **Client**: The Tauri dev server with hot reload.

To start everything:
```bash
overmind start
```

### Tips
- **Inspect one server**: `overmind connect private`
- **Restart one node**: `overmind restart open`
- **Stop everything**: Press `Ctrl+C` in the overmind session.

## License

MIT — see [LICENSE](LICENSE).
