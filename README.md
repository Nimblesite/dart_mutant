# dart_mutant

Mutation testing for Dart. AST-based, parallel, written in Rust.

Code coverage tells you what code *runs*. Mutation testing tells you what code
your tests actually *verify*. dart_mutant injects small bugs (mutants) into
your code and runs `dart test` to see which ones your tests catch.

## Install

### macOS (Apple Silicon) and Linux (x64) — Homebrew

```bash
brew install nimblesite/tap/dart_mutant
```

Works on macOS (Apple Silicon) and any Linux distro with
[Homebrew on Linux](https://docs.brew.sh/Homebrew-on-Linux) installed.

### Windows (x64) — Scoop

```powershell
scoop bucket add nimblesite https://github.com/Nimblesite/scoop-bucket
scoop install dart_mutant
```

### Other platforms — from source

Requires [Rust](https://rustup.rs/) 1.75+:

```bash
git clone https://github.com/Nimblesite/dart_mutant
cd dart_mutant
cargo build --release
export PATH="$PATH:$(pwd)/target/release"
```

## Usage

```bash
cd your_dart_project
dart_mutant
```

Open `./mutation-reports/mutation-report.html` to see what your tests missed.

## Common options

```bash
dart_mutant -j 8                          # parallel jobs (default: CPU count)
dart_mutant --sample 50                   # quick feedback on a subset
dart_mutant --threshold 80                # fail if score < 80
dart_mutant --incremental --base-ref main # only test changed files
```

Full reference: `dart_mutant --help`.

## Reports

| Flag | Output | Use case |
|---|---|---|
| `--html` *(default)* | Interactive HTML dashboard | Human review |
| `--json` | Stryker-compatible JSON | CI dashboards |
| `--junit` | JUnit XML | CI test results |
| `--ai-report` | LLM-optimized markdown | Paste into Claude/ChatGPT |

The AI report groups surviving mutants by file with line numbers and test
hints, ready for an LLM to write the missing tests.

## Mutation operators

| Category | Examples |
|---|---|
| Arithmetic | `+` ↔ `-`, `*` ↔ `/`, `++` ↔ `--` |
| Comparison | `>` ↔ `>=`, `<` ↔ `<=`, `==` ↔ `!=` |
| Logical | `&&` ↔ `\|\|`, `!` removal |
| Null safety | `??` removal, `?.` → `.` |
| Control flow | `if(x)` → `if(true)` / `if(false)` |
| Literals | `true` ↔ `false`, `"str"` → `""` |

See [docs/operators](https://dartmutant.dev/docs/operators/) for the full list.

## Results

- **Killed** — a test failed when the mutation was applied (good)
- **Survived** — every test still passed (a gap in your suite)
- **Timeout** — the mutation caused the test to hang (counted as killed)

## License

MIT. Copyright (c) 2026 Nimblesite. See [LICENSE](LICENSE).

---

Maintained by [Nimblesite](https://nimblesite.co). Docs at
[dartmutant.dev](https://dartmutant.dev).
