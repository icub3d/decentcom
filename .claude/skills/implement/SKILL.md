---
name: implement
description: Implement a feature from an existing feature document in docs/features/. Reads the feature doc, follows its implementation plan, and checks off requirements as they are completed.
disable-model-invocation: true
allowed-tools: Read Grep Glob Write Edit Bash(cargo * pnpm * npx * ls * mkdir *)
argument-hint: [feature-name]
---

# Implement Feature

Implement the feature described in: **docs/features/$0.md**

## Process

1. **Read the feature document.** Read `docs/features/$0.md` in full. Understand all requirements, the design, the task list, and the test list.

2. **Read the design docs.** Check any design documents referenced by the feature doc for additional context.

3. **Check current state.** Look at the codebase to understand what exists. Verify that the task list in the feature doc still makes sense given the current state of the code. If something has changed, note it and adapt.

4. **Work through the task list.** Complete each task in order. For each task:
   - Write the code.
   - Run relevant tests and checks (`cargo test`, `cargo clippy -- -D warnings`, `pnpm test`, `pnpm lint`) to make sure nothing is broken.
   - Check off the task in the feature doc.

5. **Work through the test list.** After tasks are complete, verify each test in the test list:
   - Run or write the specified test.
   - Confirm it passes.
   - Check off the test in the feature doc.

6. **Update the feature document.** Mark all completed tasks and tests. If you make design decisions not covered by the doc, add them to the document.

7. **Final verification.** After all items are complete:
   - Run the full test suite for affected areas.
   - Run linting/clippy with no warnings.
   - Confirm all tasks and tests in the feature doc are checked off.

8. **Report.** Summarize what was built, any deviations from the plan, and anything left unresolved.

## Guidelines

- **Follow the plan.** The feature document is the source of truth. If you need to deviate, explain why and update the doc.
- **Small steps.** Each task should result in working, testable code. Don't build everything and test at the end.
- **Server before client.** If the feature spans both, implement and verify the server side first, then the client.
- **Don't gold-plate.** Implement what the feature doc specifies. Don't add features, refactors, or improvements beyond scope.
- **Test as you go.** Run tests after each meaningful change, not just at the end.
