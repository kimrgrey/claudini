---
name: merge-pull-request
description: Wait for CI checks to pass on a PR, then merge it and switch back to main.
---

# Merge Pull Request

Wait for CI and merge a PR.

## Steps

1. **Wait for CI** with `gh pr checks <number> --watch`
   - **Always use `--watch` flag** to wait for checks to complete — never poll with `gh api`
2. **Merge** with `gh pr merge <number> --merge`
3. **Switch to main** and pull latest: `git checkout main && git pull origin main`

## Rules

- If CI fails, investigate and fix before merging — do not use `--admin` to bypass checks without asking
- Follow all Git & PR conventions from CLAUDE.md
- Never force push
