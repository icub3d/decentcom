---
name: feature
description: Create a feature document in docs/features/ that describes a new feature's design, scope, and acceptance criteria. Use before implementing a feature.
disable-model-invocation: true
allowed-tools: Read Grep Glob Write Edit Bash(ls *)
argument-hint: [feature-name]
---

# Create Feature Document

Create a feature document for: **$ARGUMENTS**

## Process

1. **Understand the feature.** Read the design documents in `docs/design/` to understand how this feature fits into the overall architecture. Ask clarifying questions if the scope is ambiguous.

2. **Research the codebase.** Look at existing code to understand what already exists, what can be reused, and where the feature would live.

3. **Write the feature document.** Create `docs/features/$0.md` using the structure below.

4. **Present the document** to the user for review. Do not start implementation — that is a separate step.

## Feature Document Structure

```markdown
# Feature: <Title>

## Overview
One-paragraph description of what this feature does and why it matters.

## Background
Context from the design docs or codebase that motivates this feature. Link to relevant design documents.

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

- Keep the document focused and actionable. This is a blueprint for implementation, not a design essay.
- Reference specific files and modules when describing where changes go.
- The implementation plan should be ordered so each step builds on the last.
- If the feature spans both server and client, organize the plan so one side can be built and tested before the other (typically server first).
- Flag anything that conflicts with or requires changes to the design docs in `docs/design/`.
