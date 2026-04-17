---
name: verify
description: Verify a feature's pull request by reviewing the code changes against the GitHub issue requirements, running tests, and optionally merging. Use when asked to "verify" or "review" a feature (e.g., /verify auth, /verify 14, /verify #42).
---

# Verify Feature PR

Verify the implementation of: **$ARGUMENTS**

## Process

1. **Find the PR.** Locate the pull request for this feature. If given a PR number, use it directly. Otherwise, search by feature name or issue number:
   ```
   GH_PAGER= GH_PROMPT_DISABLED=1 gh pr list --search "$ARGUMENTS" --limit 5 --json number,title,url,headRefName
   ```
   Or if given an issue number, find the PR that closes it:
   ```
   GH_PAGER= GH_PROMPT_DISABLED=1 gh pr list --search "closes #<number>" --limit 5 --json number,title,url,headRefName
   ```
   Read the PR body:
   ```
   GH_PAGER= GH_PROMPT_DISABLED=1 gh pr view <pr-number> --json number,title,body,url,headRefName,baseRefName,additions,deletions,changedFiles
   ```

2. **Find the linked issue.** Extract the issue number from the PR body (look for `Closes #<number>`). Read the full issue:
   ```
   GH_PAGER= GH_PROMPT_DISABLED=1 gh issue view <number> --json number,title,body,url
   ```
   Read the requirements, task list, and test list from the issue body.

3. **Review the PR diff using a worktree.** To avoid interfering with other agent work, create a new git worktree to review the PR:
   ```
   git fetch origin
   git worktree add ../verify-<pr-number> origin/<branch-name>
   cd ../verify-<pr-number>
   git log --oneline origin/main..HEAD
   ```
   Perform all verification work (testing, linting) within this worktree.
   Then review the diff:
   ```
   git --no-pager diff origin/main...HEAD --stat
   ```
   Read changed files to verify they match the issue requirements. Focus on:
   - Do the code changes implement what the issue specifies?
   - Are there any missing requirements from the task list?
   - Are there obvious bugs, security issues, or style violations?
   - Are new tests included for the new functionality?

4. **Run tests and lints.** Verify everything passes on the feature branch:
   ```
   cargo test
   cargo clippy -- -D warnings
   cd client && pnpm test && pnpm lint
   ```

5. **Cross-check the task list.** Compare the issue's task list and test list against the actual code changes:
   - For each `[x]` item, verify the code actually implements it.
   - For each `[ ]` item, check if it's actually done but not checked off, or truly missing.
   - Flag any discrepancies.

6. **Report.** Provide a verification summary:
   - ✅ / ❌ status for each requirement
   - Test results (pass/fail counts)
   - Lint results
   - Any concerns or issues found
   - The PR URL for manual review/merge

   End with:
   ```
   PR ready for review: <PR URL>
   ```
   Then ask: "Would you like me to merge this PR, or would you prefer to review and merge it manually?"

7. **Merge (if requested).** If the user asks to merge:
   ```
   GH_PAGER= GH_PROMPT_DISABLED=1 gh pr merge <pr-number> --squash --delete-branch
   ```

8. **Cleanup.** Do NOT automatically remove the worktree. Leave it for local testing unless the user explicitly instructs you to clean it up. If you need to remove it later, use:
   ```
   cd <original-project-dir>
   git worktree remove ../verify-<pr-number>
   ```

## Guidelines

- **Read the issue first.** The issue is the source of truth for what should be implemented.
- **Review the code, not just the tests.** Tests passing doesn't mean the implementation is correct.
- **Be specific about problems.** If something is wrong, point to the exact file and line.
- **Don't fix code.** This skill is for verification only. If issues are found, report them — don't fix them. The implementer should address them.
- **Use non-interactive gh.** Always run `gh` commands with `GH_PAGER=` and `GH_PROMPT_DISABLED=1`, and prefer `--json` for reads.

## Project Context Reference

- **Backend**: Rust (`server/`, `shared/`). `axum` for API, `sqlx` for SQLite.
- **Frontend**: React + TypeScript (`client/`). `Tauri v2` for the shell.
- **Styling**: Tailwind CSS + Catppuccin themes.
- **Identity**: Ed25519 keys, BIP39 seed phrases.
- **Realtime**: WebSocket gateway.
- **Issues**: All tasks are tracked via GitHub issues using `type:*` and `area:*` labels.


## Commands

- `cargo test`: Run all backend tests.
- `pnpm test`: Run all frontend tests (in `client/`).
- `cargo clippy -- -D warnings`: Lint backend.
- `pnpm lint`: Lint frontend.
- `gh pr view <number>`: Read PR details.
- `gh pr diff <number>`: View PR diff.
- `gh pr merge <number> --squash --delete-branch`: Squash-merge and clean up.
- `gh issue view <number>`: Read issue body.

