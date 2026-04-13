---
name: verify
description: Verify the implementation status of features in the decentcom project against their design documents and requirements. Use when asked to "verify" or "check" a specific feature (e.g., /verify auth).
---

# Decentcom Verifier

This skill automates the verification of features in the decentcom codebase. It ensures that implementation matches design, tests pass, and documentation is up to date.

## Verification Workflow

When triggered to verify a feature (e.g., "auth", "identity", "gateway"):

1.  **Locate Feature Doc**: Find the relevant markdown file in `docs/features/<feature>.md`. If multiple match or none match, ask for clarification.
2.  **Audit Implementation**:
    *   Read the "Requirements" and "Task List" in the feature doc.
    *   Verify that the expected files exist (check `server/`, `client/`, `shared/`).
    *   Surgically read key files to confirm logic matches the design (e.g., endpoint paths, storage traits, component structure).
3.  **Run Tests**:
    *   Identify relevant tests (Rust unit/integration tests, Vitest frontend tests).
    *   Run Rust tests: `cargo test -p <package>` or `cargo test <test_name>`.
    *   Run Frontend tests: `pnpm test` in the `client/` directory.
    *   **Mandatory**: Empirical verification. If a bug fix is part of the verification, ensure a reproduction test case exists.
4.  **Update Documentation**:
    *   If the code is complete but the task/test lists in the doc are not marked `[x]`, update them.
    *   Note any deviations from the original design in the "Implementation Notes" section.
5.  **Final Report**:
    *   Provide a concise summary of the verification results.
    *   If everything is complete and verified, propose a `git commit` command with a descriptive message (follow project conventions).
    *   Do NOT commit unless explicitly asked to "commit" in the initial request or after the report.

## Project Context Reference

*   **Backend**: Rust (`server/`, `shared/`). `axum` for API, `sqlx` for SQLite.
*   **Frontend**: React + TypeScript (`client/`). `Tauri v2` for the shell.
*   **Styling**: Tailwind CSS + Catppuccin themes.
*   **Identity**: Ed25519 keys, BIP39 seed phrases.
*   **Realtime**: WebSocket gateway.

## Commands

*   `cargo test`: Run all backend tests.
*   `pnpm test`: Run all frontend tests (in `client/`).
*   `cargo clippy -- -D warnings`: Lint backend.
*   `pnpm lint`: Lint frontend.

