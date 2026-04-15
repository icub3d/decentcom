---
name: implement
description: Implement a feature from its GitHub issue. Reads the issue, follows its task list, and checks off requirements as they are completed.
disable-model-invocation: true
allowed-tools: Read Grep Glob Write Edit Bash(cargo * pnpm * npx * ls * mkdir * gh issue *)
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

2. **Read the design docs.** Check any design documents in `docs/design/` referenced by the issue for additional context.

3. **Check current state.** Look at the codebase to understand what exists. Verify that the task list in the issue still makes sense given the current state of the code. If something has changed, note it and adapt.

4. **Work through the task list.** Complete each task in order. For each task:
   - Write the code.
   - Run relevant tests and checks (`cargo test`, `cargo clippy -- -D warnings`, `pnpm test`, `pnpm lint`) to make sure nothing is broken.
   - Check off the task in the issue by editing the issue body:
     ```
       GH_PAGER= GH_PROMPT_DISABLED=1 gh issue edit <number> --body "<updated body with [x] checked>"
     ```

5. **Work through the test list.** After tasks are complete, verify each test in the test list:
   - Run or write the specified test.
   - Confirm it passes.
   - Check off the test in the issue.

6. **Update the issue.** Mark all completed tasks and tests. If you make design decisions not covered by the issue, add them to the issue body as an "Implementation Notes" section. When fully complete, update the `status:partial` or `status:planned` label to `status:complete`:
   ```
   GH_PAGER= GH_PROMPT_DISABLED=1 gh issue edit <number> --remove-label "status:planned" --add-label "status:complete"
   # or
   GH_PAGER= GH_PROMPT_DISABLED=1 gh issue edit <number> --remove-label "status:partial" --add-label "status:complete"
   ```
   Then close the issue:
   ```
   GH_PAGER= GH_PROMPT_DISABLED=1 gh issue close <number> --reason completed
   ```

7. **Final verification.** After all items are complete:
   - Run the full test suite for affected areas.
   - Run linting/clippy with no warnings.
   - Confirm all tasks and tests in the issue are checked off.

8. **Report.** Summarize what was built, any deviations from the plan, and anything left unresolved.

## Guidelines

- **Follow the plan.** The issue is the source of truth. If you need to deviate, explain why and update the issue.
- **Small steps.** Each task should result in working, testable code. Don't build everything and test at the end.
- **Server before client.** If the feature spans both, implement and verify the server side first, then the client.
- **Don't gold-plate.** Implement what the issue specifies. Don't add features, refactors, or improvements beyond scope.
- **Test as you go.** Run tests after each meaningful change, not just at the end.
- **Use non-interactive gh.** Always run `gh issue` commands with `GH_PAGER=` and `GH_PROMPT_DISABLED=1`, and prefer `--json` for reads.
