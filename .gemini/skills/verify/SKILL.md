---
name: verify
description: Verify the implementation status of features in the decentcom project against their GitHub issue requirements. Use when asked to "verify" or "check" a specific feature (e.g., /verify auth).
---

# Decentcom Verifier

This skill automates the verification of features in the decentcom codebase. It ensures that implementation matches the GitHub issue spec, tests pass, and the issue is up to date.

## Verification Workflow

When triggered to verify a feature (e.g., "auth", "identity", "gateway"):

1.  **Find the Issue**: Search for the feature's GitHub issue:
    ```
    gh issue list --label feature --search "<feature>" --limit 5 --state all
    ```
    Read the full issue body to get the requirements, task list, and test list:
    ```
    gh issue view <number>
    ```
    If multiple match or none match, ask for clarification.

2.  **Audit Implementation**:
    *   Read the "Requirements" and "Task List" sections from the issue body.
    *   Verify that the expected files exist (check `server/`, `client/`, `shared/`).
    *   Surgically read key files to confirm logic matches the design (e.g., endpoint paths, storage traits, component structure).

3.  **Run Tests**:
    *   Identify relevant tests (Rust unit/integration tests, Vitest frontend tests).
    *   Run Rust tests: `cargo test -p <package>` or `cargo test <test_name>`.
    *   Run Frontend tests: `pnpm test` in the `client/` directory.
    *   **Mandatory**: Empirical verification. If a bug fix is part of the verification, ensure a reproduction test case exists.

4.  **Update the Issue**:
    *   If code is complete but tasks/tests in the issue are not checked `[x]`, update the issue body:
        ```
        gh issue edit <number> --body "<updated body with completed items checked>"
        ```
    *   Note any deviations from the original design by adding an "Implementation Notes" section to the issue body.
    *   If fully complete, update the status label and close the issue:
        ```
        gh issue edit <number> --remove-label "status:partial" --add-label "status:complete"
        gh issue close <number> --reason completed
        ```

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
*   **Issues**: All feature issues are on GitHub with `feature` label. Use `gh issue list --label feature --state all` to browse them.

## Commands

*   `cargo test`: Run all backend tests.
*   `pnpm test`: Run all frontend tests (in `client/`).
*   `cargo clippy -- -D warnings`: Lint backend.
*   `pnpm lint`: Lint frontend.
*   `gh issue view <number>`: Read issue body (contains requirements, task list, test list).
*   `gh issue edit <number> --body "<body>"`: Update issue body (to check off tasks).
*   `gh issue list --label feature --state all`: List all feature issues.

