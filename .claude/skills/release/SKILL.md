---
name: release
description: Create a new claudini release by tagging and pushing
allowed-tools: Bash
argument-hint: <version>
---

# Create a release

Create a new release for claudini. The version argument should be a semver string (e.g. `0.2.0`) without the `v` prefix.

## Steps

1. Read the current version from `Cargo.toml`
2. Update the version in `Cargo.toml` to `$ARGUMENTS`
3. Run `cargo build` to verify the project compiles
4. Commit the version bump with message `Bump version to $ARGUMENTS`
5. Create a git tag `v$ARGUMENTS`
6. Push the commit and tag to origin: `git push origin main && git push origin v$ARGUMENTS`
7. Report that the release workflow has been triggered and link to `https://github.com/kimrgrey/claudini/actions`
