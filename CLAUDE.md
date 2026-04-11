<!-- agent-pmo:3140e31 -->
# dart_mutant — Agent Instructions

⚠️ CRITICAL: **Reduce token usage.** Check file size before loading. Write less. Delete fluff and dead/duplicate code/docs. ⚠️

> Read this entire file before writing any code.
> These rules are NON-NEGOTIABLE. Violations will be rejected in review.

## Project Overview

`dart_mutant` is a Rust-based mutation testing tool for Dart. It uses tree-sitter
for AST-based mutations, generates syntactically valid mutants, runs `dart test`
against each mutant in parallel, and produces HTML (dark theme) and JSON
(Stryker-compatible) reports so you can see which mutations escaped your test suite.

**Primary language:** Rust (edition 2021)
**Repo type:** CLI tool
**Build command:** `make ci`
**Test command:** `make test`
**Lint command:** `make lint`

## Too Many Cooks (Multi-Agent Coordination)

If the TMC server is available:
1. Register immediately: descriptive name, intent, files you will touch
2. Before editing any file: lock it via TMC
3. Broadcast your plan before starting work
4. Check messages every few minutes
5. Release locks immediately when done
6. Never edit a locked file — wait or find another approach

## Hard Rules — Universal (no exceptions)

- **DO NOT use git commands.** No `git add`, `git commit`, `git push`, `git checkout`, `git merge`, `git rebase`, or any other git command. CI and GitHub Actions handle git.
- **ZERO DUPLICATION.** Before writing any code, search the codebase for existing implementations. Move code, don't copy it.
- **NO THROWING / PANICKING.** Return `Result<T, E>` or `Option<T>`. Panics are only for unrecoverable bugs — and they must never reach production code paths.
- **NO REGEX on structured data.** Never parse JSON, YAML, TOML, code, or any structured format with regex. Use `serde_json`, `toml`, or tree-sitter.
- **NO PLACEHOLDERS.** If something isn't implemented, leave a loud compilation error (`todo!()`). Never write code that silently does nothing.
- **Functions < 20 lines.** Refactor aggressively. If a function exceeds 20 lines, split it.
- **Files < 500 lines.** If a file exceeds 500 lines, extract modules.
- **100% test coverage is the goal.** Never delete or skip tests. Never remove assertions.
- **`make test` is FAIL-FAST.** Stops at first failing test. Never `--no-fail-fast`. Saves CI minutes.
- **`make test` ALWAYS computes coverage AND enforces it.** Threshold lives in `coverage-thresholds.json` — NOT env vars, NOT GitHub repo variables. Below threshold = pipeline fails. Ratchet only.
- **Prefer E2E/integration tests.** Unit tests are acceptable only for isolating problems.
- **Heavy logging everywhere.** Use `tracing` (already in `Cargo.toml`). See Logging Standards.
- **No suppressing linter warnings.** Fix the code, not the linter.
- **Pure functions** over statements.
- **Every spec section MUST have a unique, hierarchical, non-numeric ID.** Format: `[GROUP-TOPIC]` or `[GROUP-TOPIC-DETAIL]` (e.g., `[PARSER-DART-AST]`, `[RUNNER-TIMEOUT]`). The first word is the **group** — all sections in the same group MUST be adjacent in the spec's TOC. NEVER use sequential numbers like `[SPEC-001]`. All code, tests, and design docs that implement or relate to a spec section MUST reference its ID in a comment (e.g., `// Implements [RUNNER-TIMEOUT]`).

## Logging Standards

- **Use `tracing`.** Never use `println!`, `eprintln!`, `dbg!`, or `print!` for diagnostics. The `tracing` crate is already a dependency; pair it with `tracing-subscriber` for output.
- **Log at entry/exit of all significant operations.** Use levels: `error`, `warn`, `info`, `debug`, `trace`.
- **Logging must be throughout the app.** Every non-trivial operation (parsing, mutation application, test run, report generation) should log. Silent failures are forbidden.
- **NEVER log personal data.** No names, emails, addresses, phone numbers, IPs, or any PII.
- **NEVER log secrets.** No API keys, tokens, passwords, connection strings. If you need to confirm an AI-provider key is loaded, log `"API key: present"`.
- **Structured fields over string interpolation.** Use `tracing::info!(file = %path, mutations = count, "discovered mutations")`, not `info!("Found {} mutations in {}", count, path)`.

| Language | Library | Notes |
|----------|---------|-------|
| Rust | `tracing` | With `tracing-subscriber` for output |

## Hard Rules — Rust

- No `unwrap()` — use `?` or explicit `match`
- No `expect()` in production code (tests may use it)
- No `panic!()`, `todo!()`, `unimplemented!()`, `unreachable!()` in production code
- No `unsafe {}` blocks (`unsafe_code = "deny"` in `Cargo.toml`)
- No `#[allow(clippy::...)]` attributes without a documented justification
- All public items must have doc comments (`///`) — `missing_docs = "deny"`
- Use `thiserror` for error types; `anyhow::Result` is acceptable in `main.rs`/CLI code
- All Clippy lints run at `deny` level: `all`, `pedantic`, `nursery`, `cargo`
- Prefer iterator chains (`map`, `filter`, `and_then`, `collect`) over imperative loops
- Use `?` for error propagation — no manual `match err { ... return Err(...) }` ladders

## Testing Rules

- **Never delete a failing test.** Fix the code or fix the test expectation — never delete.
- **Never skip a test** (`#[ignore]`) without a ticket number and expiry date in the skip reason.
- **Assertions must be specific.** `assert!(true)` without a real condition is illegal.
- **No try/catch swallowing.** No test that catches an error and then asserts success.
- **Tests must be deterministic.** No sleep, no timing dependencies, no random state.
- **E2E tests: black-box only.** Only interact via the `dart_mutant` CLI binary. Never call internal library functions from E2E tests.
- **Coarse integration tests only.** Per the project's philosophy: tests in `tests/` must actually TEST TESTS — they verify mutation testing catches weak Dart test suites, using `tests/fixtures/simple_dart_project/` as a real fixture.

## Build Commands (cross-platform via GNU Make)

All `make` targets work on Linux, macOS, and Windows. The Makefile uses OS detection
to select portable commands. There are exactly 7 targets — no others.

```bash
make build   # cargo build --release
make test    # fail-fast tests + coverage + threshold enforcement (ONLY test entry point)
make lint    # cargo fmt --check + cargo clippy (read-only, no formatting)
make fmt     # cargo fmt --all (format in-place)
make clean   # cargo clean + remove lcov.info
make ci      # lint + test + build (full CI simulation)
make setup   # install cargo-llvm-cov + rust components
```

**`make test` is fail-fast AND enforces the coverage threshold from `coverage-thresholds.json`.**
It exits non-zero on any test failure or coverage shortfall. Coverage thresholds live in
`coverage-thresholds.json` at the repo root — NOT env vars, NOT GitHub repo variables.

**`make fmt`** formats code in-place. **`make lint`** runs linters (read-only). **`make test`**
runs tests with coverage. Three separate, non-overlapping targets.

## Repo Structure

```
dart_mutant/
├── .github/
│   ├── workflows/
│   │   ├── ci.yml            # lint → test → coverage → build
│   │   ├── release.yml       # workflow_dispatch → build + homebrew update
│   │   └── deploy-pages.yml  # website deploy (release-only)
│   └── pull_request_template.md
├── .claude/skills/           # Claude Code skills (this repo's primary agent)
├── docs/
│   ├── specs/                # Specs (behavior / requirements)
│   └── plans/                # Plans (TODO checklists, how-to-achieve docs)
├── src/
│   ├── main.rs               # Entry point, CLI orchestration, progress display
│   ├── cli/mod.rs            # Clap argument definitions (Args, AiProvider)
│   ├── parser/mod.rs         # Tree-sitter Dart parsing, mutation discovery
│   ├── mutation/             # Mutation types + 40+ operators + sampling
│   │   ├── mod.rs
│   │   └── operators.rs
│   ├── runner/mod.rs         # Parallel test execution, file mutation/restore
│   ├── report/mod.rs         # HTML (dark theme) and JSON (Stryker-compatible)
│   └── ai/mod.rs             # AI-powered mutation suggestions (Anthropic/OpenAI/Ollama)
├── tests/
│   ├── integration_parser.rs
│   ├── integration_mutation.rs
│   ├── integration_runner.rs
│   ├── integration_report.rs
│   ├── integration_e2e.rs
│   └── fixtures/simple_dart_project/
├── website/                  # Eleventy-based project website
├── Cargo.toml                # Rust manifest + workspace-wide lints
├── rustfmt.toml
├── Makefile
├── CLAUDE.md                 # <-- you are here (canonical instructions)
└── AGENTS.md                 # pointer → CLAUDE.md
```

### Data flow

1. `parser::discover_dart_files` finds `.dart` files, excluding generated code.
2. `parser::parse_and_find_mutations` uses tree-sitter to walk the AST and find mutation points.
3. `runner::run_mutation_tests` applies each mutation, runs `dart test`, restores the file.
4. `report::generate_*_report` creates HTML / JSON output with per-file breakdown.

### Key types

- `Mutation` — describes a single code mutation (location, original, replacement, operator).
- `MutantTestResult` — result after running tests (`Killed`/`Survived`/`Timeout`/`Error`).
- `MutationResult` — aggregate stats (`mutation_score`, `killed`, `survived` counts).

## CI Notes

- CI runs on Ubuntu; full integration tests require the Dart SDK (installed via `dart-lang/setup-dart@v1`).
- Coverage is collected by `cargo-llvm-cov`. Threshold is enforced by `make test` via `coverage-thresholds.json` (currently 85%). No GitHub repo variables for coverage.
- The release workflow publishes a Homebrew formula to `Nimblesite/homebrew-tap`.

## Website (SEO + AI-aware)

If you edit content under `website/`, optimise for both traditional SEO and
AI-powered search:

- [Top ways to ensure your content performs well in Google's AI experiences on Search](https://developers.google.com/search/blog/2025/05/succeeding-in-ai-search)
- [Search Engine Optimization (SEO) Starter Guide](https://developers.google.com/search/docs/fundamentals/seo-starter-guide)

## Agent Reference Docs (Claude Code)

- CLAUDE.md memory format: https://code.claude.com/docs/en/memory#claude-md-files
- Claude Code skills: https://platform.claude.com/docs/en/agents-and-tools/agent-skills/overview
