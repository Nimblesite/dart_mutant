---
name: ci-prep
description: Prepares the current branch for CI by running the exact same steps locally and fixing issues. If CI is already failing, fetches the GH Actions logs first to diagnose. Use before pushing, when CI is red, or when the user says "fix ci".
argument-hint: "[--failing] [optional job name to focus on]"
---
<!-- agent-pmo:5547fd2 -->

# CI Prep

Prepare the current state for CI. If CI is already failing, fetch and analyze the logs first.

## Arguments

- `--failing` — Indicates a GitHub Actions run is already failing. When present, you MUST execute **Step 1** before doing anything else.
- Any other argument is treated as a job name to focus on (but all failures are still reported).

If `--failing` is NOT passed, skip directly to **Step 2**.

## Step 1 — Fetch failed CI logs (only when `--failing`)

You MUST do this before any other work.

```bash
BRANCH=$(git branch --show-current)
PR_JSON=$(gh pr list --head "$BRANCH" --state open --json number,title,url --limit 1)
```

If the JSON array is empty, **stop immediately**:
> No open PR found for branch `$BRANCH`. Create a PR first.

Otherwise fetch the logs:

```bash
PR_NUMBER=$(echo "$PR_JSON" | jq -r '.[0].number')
gh pr checks "$PR_NUMBER"
RUN_ID=$(gh run list --branch "$BRANCH" --limit 1 --json databaseId --jq '.[0].databaseId')
gh run view "$RUN_ID"
gh run view "$RUN_ID" --log-failed
```

Read **every line** of `--log-failed` output. For each failure note the exact file, line, and error message. If a job name argument was provided, prioritize that job but still report all failures.

## Step 2 — Analyze the CI workflow

1. Find the CI workflow file at `.github/workflows/ci.yml`.
2. Read the workflow file completely. Parse every job and every step.
3. Extract the ordered list of commands CI actually runs. For this repo the `ci` job runs: `make lint`, `make test`, `make coverage-check`, `make build` (plus `cargo install cargo-llvm-cov` and `dart-lang/setup-dart` for the Dart SDK that integration tests need).
4. Note environment variables like `COVERAGE_THRESHOLD_RUST` and any matrix/conditional steps.

**Do NOT assume** — re-read the workflow each time, because it may have been updated.

## Step 3 — Run each CI step locally, in order

Work through failures in this priority order:

1. **Formatting** — `cargo fmt --all` to clear noise, then `make fmt-check`
2. **Compilation errors** — must compile before lint/test (`cargo check --all-targets`)
3. **Lint violations** — `cargo clippy --release --all-targets -- -D warnings`
4. **Runtime / test failures** — fix source code to satisfy the test (`make test`)

For each command extracted from the CI workflow:

1. Run the command exactly as CI would run it (adjusting only for local environment differences like not needing `actions/checkout`).
2. If the step fails, **stop and fix the issues** before continuing to the next step.
3. After fixing, re-run the same step to confirm it passes.
4. Move to the next step only after the current one succeeds.

### Hard constraints

- **NEVER modify test files** — fix the source code, not the tests
- **NEVER add suppressions** (`#[allow(clippy::...)]`, `#[allow(dead_code)]`, etc.)
- **NEVER delete or ignore failing tests** (no new `#[ignore]`)
- **NEVER remove assertions**
- **NEVER use `unwrap()` / `expect()` in production code** to silence errors — use `?`

If stuck on the same failure after 5 attempts, ask the user for help.

## Step 4 — Report

- List every step that was run and its result (pass/fail/fixed).
- If any step could not be fixed, report what failed and why.
- Confirm whether the branch is ready to push.

## Step 5 — Commit/Push (only when `--failing`)

Once all CI steps pass locally:

1. Commit, but DO NOT MARK THE COMMIT WITH YOU AS AN AUTHOR!!! Only the user authors the commit!
2. Push
3. Monitor until completion or failure
4. Upon failure, go back to Step 1

## Rules

- **Always read the CI workflow first.** Never assume what commands CI runs.
- Do not push if any step fails (unless `--failing` and all steps now pass)
- Fix issues found in each step before moving to the next
- Never skip steps or suppress errors
- If the CI workflow has multiple jobs, run all of them (respecting dependency order)
- Skip steps that are CI-infrastructure-only (checkout, `dtolnay/rust-toolchain`, `Swatinem/rust-cache`, `dart-lang/setup-dart`, `cargo install cargo-llvm-cov`, artifact uploads) — focus on the actual `make` targets

## Success criteria

- Every `make` target that CI runs has been executed locally and passed
- All fixes are applied to the working tree
- The CI passes successfully (if you are correcting an existing failure)
