---
name: create-pull-request
description: Create a new branch, commit all changes, push, and open a PR.
---

# Create Pull Request

Commit, push, and open a PR.

## Context

Current branch: !`git branch --show-current`
Working tree status: !`git status --short`

## Steps

1. **Create a new branch** from the current branch
   - Pick a short, readable name describing the changes
2. **Stage and commit** all changes
   - Stage specific files — never use `git add -A` or `git add .`
   - Write a concise commit message (1-2 sentences) focusing on the "why"
3. **Push** the branch with `-u origin <branch>`
4. **Open a PR** with `gh pr create`
   - Title under 70 characters
   - Short summary in the body
   - Assign to `@me`
5. **Stop here** — return the PR URL to the user

## Rules

- **Do not merge unless the user explicitly asks to merge** — if asked, use the `merge-pull-request` skill
- Follow all Git & PR conventions from CLAUDE.md
- Never force push
- Never add Claude as a co-author
- Always assign the PR to its author
