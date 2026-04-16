---
name: feature
description: Create a GitHub issue for a new feature that describes its design, scope, and acceptance criteria. Use before implementing a feature.
disable-model-invocation: true
allowed-tools: Read Grep Glob Bash(ls * gh issue * gh label *)
argument-hint: [feature-name]
---

# Create Feature Issue

Create a GitHub issue for the feature: **$ARGUMENTS**

## Process

1. **Understand the feature.** Read the design documents in `docs/design/` to understand how this feature fits into the overall architecture. Ask clarifying questions if the scope is ambiguous.

2. **Research the codebase in a worktree.** To avoid interfering with other agent work, create a new git worktree for research if you plan to run tests or make experimental changes:
   ```
   git fetch origin
   git worktree add ../research-<feature-name> origin/main
   cd ../research-<feature-name>
   ```
   Look at existing code to understand what already exists, what can be reused, and where the feature would live.

3. **Check for an existing issue.** Run:
   ```
   gh issue list --label feature --search "$ARGUMENTS" --limit 10
   ```
   If an issue already exists for this feature, update it instead of creating a new one.

4. **Determine the phase label.** Based on `docs/features/FEATURES.md` or the feature's dependencies, pick the right phase:
   - `phase:1-foundation`
   - `phase:2-core-ux`
   - `phase:3-media-voice`
   - `phase:4-operations`

5. **Draft the issue body** using the structure below.

6. **Create the issue:**
   ```
   gh issue create --title "Feature: <Title>" \
     --label "feature,<phase-label>,status:planned" \
     --body "<body>"
   ```

7. **Present the issue URL** to the user for review. Do not start implementation — that is a separate step.

8. **Cleanup.** Do NOT automatically remove the research worktree. Leave it for local testing unless the user explicitly instructs you to clean it up. If you need to remove it later, use:
   ```
   cd <original-project-dir>
   git worktree remove ../research-<feature-name>
   ```

## Issue Body Structure

```markdown
# Feature: <Title>

## Overview
One-paragraph description of what this feature does and why it matters.

## Background
Context from the design docs or codebase that motivates this feature.

## Requirements
Concrete, testable requirements. Use a checklist:
- [ ] Requirement 1
- [ ] Requirement 2

## Design

### API / Interface Changes
New or modified endpoints, IPC commands, or public interfaces.

### Data Model Changes
New tables, columns, or schema changes.

### Component Changes
Which files/modules are created or modified. Be specific about where code lives.

## Task List
Ordered checklist of implementation tasks. Each task should be small enough to be a single commit. Group into phases if the feature is large.
- [ ] Task 1
- [ ] Task 2

## Test List
Checklist of tests that verify the feature works. Include unit tests, integration tests, and manual verification steps. Each entry should be concrete and checkable.
- [ ] Test 1
- [ ] Test 2

## Open Questions
Anything unresolved that needs a decision before or during implementation.
```

## Guidelines

- Keep the issue focused and actionable. This is a blueprint for implementation, not a design essay.
- Reference specific files and modules when describing where changes go.
- The implementation plan should be ordered so each step builds on the last.
- If the feature spans both server and client, organize the plan so one side can be built and tested before the other (typically server first).
- Flag anything that conflicts with or requires changes to the design docs in `docs/design/`.
