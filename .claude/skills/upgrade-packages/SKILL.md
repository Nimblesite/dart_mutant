---
name: upgrade-packages
description: Upgrade all Rust crate dependencies in Cargo.toml to their latest versions. Use when the user says "upgrade packages", "update dependencies", "bump versions", or "upgrade deps".
argument-hint: "[--check-only] [--major] [crate-name]"
---
<!-- agent-pmo:3140e31 -->

# Upgrade Packages

Upgrade all Rust crate dependencies to their latest compatible (or latest major, if `--major`) versions.

## Arguments

- `--check-only` — List outdated crates without upgrading. Stop after Step 2.
- `--major` — Include major version bumps (breaking changes). Without this flag, stay within semver-compatible ranges.
- Any other argument is treated as a specific crate name to upgrade (instead of all crates).

## Step 1 — Verify the manifest

Inspect the repo root for `Cargo.toml`. This project is a single Rust crate (not a workspace). All dependencies live under `[dependencies]` and `[dev-dependencies]` in the root `Cargo.toml`.

If `Cargo.toml` is missing, stop and tell the user.

## Step 2 — List outdated crates

Run `cargo outdated` to list what's outdated BEFORE upgrading anything.

```bash
cargo outdated        # install: cargo install cargo-outdated
cargo update --dry-run
```

**Read the docs:** https://doc.rust-lang.org/cargo/commands/cargo-update.html

If `--check-only` was passed, **stop here** and report the outdated list.

## Step 3 — Read the official upgrade docs

**Before running any upgrade command, you MUST fetch and read** https://doc.rust-lang.org/cargo/commands/cargo-update.html to confirm the correct flags and behavior for the installed `cargo` version. Do not guess from memory.

## Step 4 — Upgrade crates

Run the upgrade. If a specific crate name was given as an argument, upgrade only that crate.

```bash
cargo update                          # semver-compatible updates
# --major flag:
cargo update --breaking               # major version bumps (cargo 1.84+)
# specific crate:
cargo update -p <crate-name>
```

For major version bumps that `cargo update --breaking` cannot handle (or on older cargo), edit the version string in `Cargo.toml` directly, then run `cargo update`.

## Step 5 — Verify the upgrade

After upgrading, run the project's build and test suite to confirm nothing broke:

```bash
make ci
```

If tests fail:
1. Read the failure output carefully
2. Check the changelog / migration guide for the upgraded crate (fetch the release notes URL if available)
3. Fix breaking changes in the code
4. Re-run `make ci`
5. If stuck after 3 attempts on the same failure, report it to the user with the error details and the crate that caused it

## Step 6 — Report

Provide a summary:

- Crates upgraded (old version -> new version)
- Crates skipped (and why, e.g., major version bump without `--major` flag)
- Build/test result after upgrade
- Any breaking changes that were fixed
- Any crates that could not be upgraded (with error details)

## Rules

- **Always list outdated crates first** before upgrading anything
- **Always read the official docs** before running upgrade commands
- **Always run `make ci` after upgrading** to catch breakage immediately
- **Never remove crates** unless they were explicitly deprecated and replaced
- **Never downgrade crates** unless rolling back a broken upgrade
- **Never modify `Cargo.lock` manually** — let `cargo` regenerate it
- **Commit nothing** — leave changes in the working tree for the user to review

## Success criteria

- All outdated crates upgraded to latest compatible (or latest major if `--major`)
- `make ci` passes
- User has a clear summary of what changed
