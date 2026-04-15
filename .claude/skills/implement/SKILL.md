---
name: implement
description: Implement a feature from its GitHub issue. Creates a feature branch, follows the task list, and opens a pull request when done.
disable-model-invocation: true
allowed-tools: Read Grep Glob Write Edit Bash(cargo * pnpm * npx * ls * mkdir * git checkout * git switch * git branch * git add * git commit * git push * gh issue * gh pr *)
argument-hint: [feature-name or issue number]
---

# Implement Feature

Implement the feature: **$ARGUMENTS**

## Process

1. **Find the issue.** Locate the GitHub issue for this feature:
   ```
   GH_PAGER= GH_PROMPT_DISABLED=1 gh issue list --label feature --search "$ARGUMENTS" --limit 5 --json number,title,url
   ```
   Or if given an issue number directly:
   ```
   GH_PAGER= GH_PROMPT_DISABLED=1 gh issue view <number> --json number,title,body,url
   ```
   Read the issue body in full — it contains the requirements, design, task list, and test list.

2. **Create a feature branch.** Branch from `main` using the naming convention `feature/<issue-number>-<short-slug>`:
   ```
   git switch main
   git pull --ff-only
   git switch -c feature/<number>-<short-slug>
   ```
   For example, issue #14 "User Profiles" → `feature/14-user-profiles`.

3. **Read the design docs.** Check any design documents in `docs/design/` referenced by the issue for additional context.

4. **Check current state.** Look at the codebase to understand what exists. Verify that the task list in the issue still makes sense given the current state of the code. If something has changed, note it and adapt.

5. **Work through the task list.** Complete each task in order. For each task:
   - Write the code.
   - Run relevant tests and checks (`cargo test`, `cargo clippy -- -D warnings`, `pnpm test`, `pnpm lint`) to make sure nothing is broken.
   - Commit the task with a descriptive message referencing the issue:
     ```
     git add -A
     git commit -m "<descriptive message> (#<number>)"
     ```
   - Check off the task in the issue by editing the issue body:
     ```
       GH_PAGER= GH_PROMPT_DISABLED=1 gh issue edit <number> --body "<updated body with [x] checked>"
     ```

6. **Work through the test list.** After tasks are complete, verify each test in the test list:
   - Run or write the specified test.
   - Confirm it passes.
   - Check off the test in the issue.

7. **Final verification.** After all items are complete:
   - Run the full test suite for affected areas.
   - Run linting/clippy with no warnings.
   - Confirm all tasks and tests in the issue are checked off.

8. **Push and open a pull request.** Push the feature branch and create a PR:
   ```
   git push -u origin feature/<number>-<short-slug>
   GH_PAGER= GH_PROMPT_DISABLED=1 gh pr create \
     --title "feat: <short title> (#<number>)" \
     --body "Closes #<number>

   ## Summary
   <brief description of what was implemented>

   ## Changes
   <bulleted list of major changes>

   ## Testing
   - All existing tests pass
   - <list new tests added>
   - cargo clippy -- -D warnings: clean
   - pnpm lint: clean" \
     --base main
   ```

9. **Update the issue.** Mark the status label to `status:complete`:
   ```
   GH_PAGER= GH_PROMPT_DISABLED=1 gh issue edit <number> --remove-label "status:planned" --add-label "status:complete"
   # or
   GH_PAGER= GH_PROMPT_DISABLED=1 gh issue edit <number> --remove-label "status:partial" --add-label "status:complete"
   ```
   Do **not** close the issue — it will be closed automatically when the PR merges via the `Closes #<number>` reference.

10. **Report.** Summarize what was built, any deviations from the plan, and provide the PR URL for review.

## Guidelines

- **Follow the plan.** The issue is the source of truth. If you need to deviate, explain why and update the issue.
- **Small steps.** Each task should result in working, testable code. Don't build everything and test at the end.
- **Commit often.** Each logical unit of work (task or group of related tasks) should be a separate commit.
- **Server before client.** If the feature spans both, implement and verify the server side first, then the client.
- **Don't gold-plate.** Implement what the issue specifies. Don't add features, refactors, or improvements beyond scope.
- **Test as you go.** Run tests after each meaningful change, not just at the end.
- **Use non-interactive gh.** Always run `gh` commands with `GH_PAGER=` and `GH_PROMPT_DISABLED=1`, and prefer `--json` for reads.
