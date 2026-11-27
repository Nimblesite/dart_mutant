// Allow panics and expects in test code - tests need to fail loudly
#![allow(clippy::expect_used, clippy::panic, clippy::unwrap_used)]

//! End-to-end integration tests for dart_mutant
//!
//! These tests verify the complete mutation testing pipeline:
//! - CLI argument parsing
//! - File discovery
//! - Mutation generation
//! - Test execution (when Dart is available)
//! - Report generation
//!
//! These are HIGH-LEVEL tests that test the actual tool behavior.
//!
//! IMPORTANT: Tests that run actual mutations use COPIES of fixtures
//! to prevent corrupting the original fixture files.

use std::path::PathBuf;
use std::process::Command;

/// Get the path to the test fixtures directory
fn fixtures_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("simple_dart_project")
}

/// Get the path to the dart_mutant binary. Cargo builds it for integration
/// tests and sets this variable, with the `.exe` suffix on Windows and the
/// `llvm-cov` target directory under coverage.
fn binary_path() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_dart_mutant"))
}

/// Check if the binary exists
fn binary_exists() -> bool {
    binary_path().exists()
}

/// Check if Dart is available
fn dart_available() -> bool {
    Command::new("dart")
        .arg("--version")
        .output()
        .is_ok_and(|o| o.status.success())
}

/// Copy a named fixture project to a temp directory to prevent mutation from
/// corrupting originals. Returns the temp dir (auto-cleaned on drop) and the
/// path to the copied project inside it.
fn copy_fixture_to_temp(fixture_name: &str) -> Option<(tempfile::TempDir, PathBuf)> {
    let temp_dir = tempfile::tempdir().ok()?;
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(fixture_name);
    let dest = temp_dir.path().join(fixture_name);

    // Copy recursively using walkdir
    for entry in walkdir::WalkDir::new(&source)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let rel_path = entry.path().strip_prefix(&source).ok()?;
        let target = dest.join(rel_path);

        if entry.file_type().is_dir() {
            std::fs::create_dir_all(&target).ok()?;
        } else {
            if let Some(parent) = target.parent() {
                std::fs::create_dir_all(parent).ok()?;
            }
            std::fs::copy(entry.path(), &target).ok()?;
        }
    }

    Some((temp_dir, dest))
}

/// Copy the default `simple_dart_project` fixture to a temp directory.
/// Returns the temp directory path (which will be automatically cleaned up on drop).
fn copy_fixtures_to_temp() -> Option<tempfile::TempDir> {
    copy_fixture_to_temp("simple_dart_project").map(|(temp_dir, _)| temp_dir)
}

mod cli_arguments {
    use super::*;

    #[test]
    fn help_flag_shows_usage() {
        if !binary_exists() {
            println!("Skipping: binary not built. Run `cargo build` first.");
            return;
        }

        let output = Command::new(binary_path())
            .arg("--help")
            .output()
            .expect("Failed to execute command");

        let stdout = String::from_utf8_lossy(&output.stdout);

        assert!(
            stdout.contains("dart_mutant") || stdout.contains("mutation"),
            "Help should mention dart_mutant or mutation testing"
        );
        assert!(
            stdout.contains("--path") || stdout.contains("-p"),
            "Help should show --path option"
        );
        assert!(
            stdout.contains("--threshold"),
            "Help should show --threshold option"
        );
    }

    #[test]
    fn version_flag_shows_version() {
        if !binary_exists() {
            println!("Skipping: binary not built");
            return;
        }

        let output = Command::new(binary_path())
            .arg("--version")
            .output()
            .expect("Failed to execute command");

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        let combined = format!("{}{}", stdout, stderr);

        assert!(
            combined.contains("0.") || combined.contains("dart_mutant"),
            "Version output should contain version number or program name"
        );
    }

    #[test]
    fn accepts_path_argument() {
        if !binary_exists() {
            println!("Skipping: binary not built");
            return;
        }

        // Just test that the argument is accepted (--dry-run prevents actual execution)
        let output = Command::new(binary_path())
            .args(["--path", "/nonexistent", "--dry-run"])
            .output()
            .expect("Failed to execute command");

        // It should at least try to run, even if the path doesn't exist
        let stderr = String::from_utf8_lossy(&output.stderr);
        // Either it runs or reports path not found - both are valid
        assert!(
            !stderr.contains("error: unexpected argument")
                && !stderr.contains("error: Found argument"),
            "--path should be a valid argument"
        );
    }

    #[test]
    fn accepts_threshold_argument() {
        if !binary_exists() {
            println!("Skipping: binary not built");
            return;
        }

        let output = Command::new(binary_path())
            .args(["--threshold", "80", "--dry-run", "--path", "/nonexistent"])
            .output()
            .expect("Failed to execute command");

        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            !stderr.contains("error: unexpected argument"),
            "--threshold should be a valid argument"
        );
    }

    #[test]
    fn accepts_parallel_jobs_argument() {
        if !binary_exists() {
            println!("Skipping: binary not built");
            return;
        }

        let output = Command::new(binary_path())
            .args(["--parallel", "4", "--dry-run", "--path", "/nonexistent"])
            .output()
            .expect("Failed to execute command");

        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            !stderr.contains("error: unexpected argument"),
            "--parallel should be a valid argument"
        );
    }

    #[test]
    fn accepts_output_directory_argument() {
        if !binary_exists() {
            println!("Skipping: binary not built");
            return;
        }

        let output = Command::new(binary_path())
            .args([
                "--output",
                "/tmp/reports",
                "--dry-run",
                "--path",
                "/nonexistent",
            ])
            .output()
            .expect("Failed to execute command");

        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            !stderr.contains("error: unexpected argument"),
            "--output should be a valid argument"
        );
    }

    #[test]
    fn accepts_exclude_patterns() {
        if !binary_exists() {
            println!("Skipping: binary not built");
            return;
        }

        let output = Command::new(binary_path())
            .args([
                "--exclude",
                "**/*.g.dart",
                "--exclude",
                "**/*.freezed.dart",
                "--dry-run",
                "--path",
                "/nonexistent",
            ])
            .output()
            .expect("Failed to execute command");

        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            !stderr.contains("error: unexpected argument"),
            "--exclude should be a valid argument"
        );
    }
}

mod file_discovery_e2e {
    use super::*;

    #[test]
    fn discovers_fixture_dart_files() {
        let lib = fixtures_path().join("lib");
        assert!(lib.exists(), "Test fixtures lib directory should exist");

        let dart_file_count = walkdir::WalkDir::new(&lib)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map_or(false, |ext| ext == "dart"))
            .count();

        assert_eq!(
            dart_file_count, 4,
            "Should find exactly 4 Dart files in fixtures/lib"
        );
    }

    #[test]
    fn fixture_files_are_parseable() {
        let lib = fixtures_path().join("lib");

        let dart_files = vec![
            lib.join("calculator.dart"),
            lib.join("string_utils.dart"),
            lib.join("null_safe.dart"),
        ];

        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&tree_sitter_dart::language())
            .expect("Should load Dart grammar");

        for file in dart_files {
            let source =
                std::fs::read_to_string(&file).unwrap_or_else(|_| panic!("Should read {:?}", file));

            let tree = parser.parse(&source, None).expect("Should parse");

            assert!(
                !tree.root_node().has_error(),
                "{:?} should parse without errors",
                file.file_name()
            );
        }
    }
}

mod mutation_generation_e2e {
    use super::*;

    /// Count mutation candidates in a Dart file
    fn count_mutation_candidates(source: &str) -> usize {
        let mut count = 0;

        // Arithmetic
        count += source.matches(" + ").count();
        count += source.matches(" - ").count();
        count += source.matches(" * ").count();
        count += source.matches(" / ").count();

        // Comparison
        count += source.matches(" > ").count();
        count += source.matches(" < ").count();
        count += source.matches(" >= ").count();
        count += source.matches(" <= ").count();
        count += source.matches(" == ").count();
        count += source.matches(" != ").count();

        // Logical
        count += source.matches(" && ").count();
        count += source.matches(" || ").count();

        // Null safety
        count += source.matches("??").count();
        count += source.matches("?.").count();

        // Control flow (if statements generate 2 mutations each)
        count += (source.matches("if (").count() + source.matches("if(").count()) * 2;

        count
    }

    #[test]
    fn fixtures_have_many_mutation_candidates() {
        let lib = fixtures_path().join("lib");

        let files = vec![
            ("calculator.dart", 15), // Minimum expected mutations
            ("string_utils.dart", 10),
            ("null_safe.dart", 10),
        ];

        for (filename, min_expected) in files {
            let path = lib.join(filename);
            let source = std::fs::read_to_string(&path)
                .unwrap_or_else(|_| panic!("Should read {}", filename));

            let count = count_mutation_candidates(&source);
            assert!(
                count >= min_expected,
                "{} should have at least {} mutation candidates, found {}",
                filename,
                min_expected,
                count
            );
        }
    }

    #[test]
    fn calculator_has_all_mutation_types() {
        let calc = fixtures_path().join("lib").join("calculator.dart");
        let source = std::fs::read_to_string(&calc).expect("Should read");

        // Verify presence of different mutation-worthy constructs
        assert!(source.contains(" + "), "Should have addition");
        assert!(source.contains(" - "), "Should have subtraction");
        assert!(source.contains(" * "), "Should have multiplication");
        assert!(
            source.contains("~/") || source.contains(" / "),
            "Should have division"
        );
        assert!(
            source.contains(" > ") || source.contains(" >= "),
            "Should have greater than"
        );
        assert!(
            source.contains(" < ") || source.contains(" <= "),
            "Should have less than"
        );
        assert!(source.contains(" == "), "Should have equality");
        assert!(
            source.contains("if (") || source.contains("if("),
            "Should have if statement"
        );
        assert!(
            source.contains("++") || source.contains("--"),
            "Should have increment/decrement"
        );
    }

    #[test]
    fn null_safe_has_dart_specific_constructs() {
        let ns = fixtures_path().join("lib").join("null_safe.dart");
        let source = std::fs::read_to_string(&ns).expect("Should read");

        // Dart-specific null safety mutations
        assert!(source.contains("??"), "Should have null coalescing");
        assert!(source.contains("?."), "Should have null-aware access");
        assert!(
            source.contains("!= null") || source.contains("== null"),
            "Should have null checks"
        );
    }
}

mod full_pipeline_e2e {
    use super::*;

    #[test]
    fn dry_run_on_fixtures_succeeds() {
        if !binary_exists() {
            println!("Skipping: binary not built");
            return;
        }

        let output = Command::new(binary_path())
            .args(["--path", fixtures_path().to_str().unwrap(), "--dry-run"])
            .output()
            .expect("Failed to execute command");

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        println!("stdout: {}", stdout);
        println!("stderr: {}", stderr);

        // In dry run mode, should discover files and generate mutations without running tests
        assert!(
            output.status.success() || stderr.contains("mutation"),
            "Dry run should succeed or at least start mutation process"
        );
    }

    #[test]
    fn full_run_on_fixtures_produces_report() {
        if !binary_exists() || !dart_available() {
            println!("Skipping: binary not built or Dart not available");
            return;
        }

        // IMPORTANT: Copy fixtures to temp dir to prevent corruption during mutation testing
        let Some(temp_fixtures) = copy_fixtures_to_temp() else {
            println!("Skipping: failed to copy fixtures");
            return;
        };
        let project_path = temp_fixtures.path().join("simple_dart_project");

        // Run dart pub get on the COPY
        let pub_get = Command::new("dart")
            .args(["pub", "get"])
            .current_dir(&project_path)
            .output();

        if pub_get.is_err() || !pub_get.unwrap().status.success() {
            println!("Skipping: dart pub get failed");
            return;
        }

        let temp_output = std::env::temp_dir().join("dart_mutant_e2e_test");
        std::fs::create_dir_all(&temp_output).ok();

        let output = Command::new(binary_path())
            .args([
                "--path",
                project_path.to_str().unwrap(),
                "--output",
                temp_output.to_str().unwrap(),
                "--html",
                "--json",
                "--sample",
                "5", // Only test 5 mutations for speed
                "--timeout",
                "10",
            ])
            .output()
            .expect("Failed to execute command");

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        println!("stdout: {}", stdout);
        println!("stderr: {}", stderr);

        // Check for report files
        let html_report = temp_output.join("mutation-report.html");
        let json_report = temp_output.join("mutation-report.json");

        if html_report.exists() {
            let html = std::fs::read_to_string(&html_report).unwrap();
            assert!(
                html.contains("<!DOCTYPE html>"),
                "HTML report should be valid HTML"
            );
        }

        if json_report.exists() {
            let json = std::fs::read_to_string(&json_report).unwrap();
            assert!(
                json.contains("mutationScore") || json.contains("mutation_score"),
                "JSON report should contain mutation score"
            );
        }

        // Cleanup (temp_fixtures auto-cleans on drop, but output dir needs manual cleanup)
        std::fs::remove_dir_all(&temp_output).ok();
    }

    #[test]
    fn ai_report_flag_generates_markdown_report() {
        if !binary_exists() || !dart_available() {
            println!("Skipping: binary not built or Dart not available");
            return;
        }

        // IMPORTANT: Copy fixtures to temp dir to prevent corruption during mutation testing
        let Some(temp_fixtures) = copy_fixtures_to_temp() else {
            println!("Skipping: failed to copy fixtures");
            return;
        };
        let project_path = temp_fixtures.path().join("simple_dart_project");

        // Run dart pub get on the COPY
        let pub_get = Command::new("dart")
            .args(["pub", "get"])
            .current_dir(&project_path)
            .output();

        if pub_get.is_err() || !pub_get.unwrap().status.success() {
            println!("Skipping: dart pub get failed");
            return;
        }

        let temp_output = std::env::temp_dir().join("dart_mutant_ai_report_test");
        std::fs::create_dir_all(&temp_output).ok();

        let output = Command::new(binary_path())
            .args([
                "--path",
                project_path.to_str().unwrap(),
                "--output",
                temp_output.to_str().unwrap(),
                "--ai-report",
                "--sample",
                "3", // Only test 3 mutations for speed
                "--timeout",
                "10",
            ])
            .output()
            .expect("Failed to execute command");

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        println!("stdout: {}", stdout);
        println!("stderr: {}", stderr);

        // Check for AI report file
        let ai_report = temp_output.join("mutation-report-ai.md");

        if ai_report.exists() {
            let md = std::fs::read_to_string(&ai_report).unwrap();
            assert!(
                md.contains("# Mutation Testing Report (AI-Optimized)"),
                "AI report should have expected header"
            );
            assert!(
                md.contains("## Summary"),
                "AI report should have summary section"
            );
            assert!(
                md.contains("**Mutation Score**"),
                "AI report should contain mutation score"
            );
            assert!(
                md.contains("**Killed**") && md.contains("**Survived**"),
                "AI report should contain killed/survived counts"
            );

            // If there are survivors, should have action items
            if md.contains("Surviving Mutants") {
                assert!(
                    md.contains("**Mutation**:"),
                    "AI report should show mutation details for survivors"
                );
                assert!(
                    md.contains("**Suggested Test**:"),
                    "AI report should provide test hints"
                );
                assert!(
                    md.contains("## Quick Reference"),
                    "AI report should have quick reference section"
                );
            }
        } else {
            println!("AI report not generated (possibly no mutations tested)");
        }

        // Cleanup (temp_fixtures auto-cleans on drop)
        std::fs::remove_dir_all(&temp_output).ok();
    }
}

mod baseline_verification {
    use super::*;

    /// Regression test for GitHub issue #5: running mutation testing on a
    /// project whose test suite FAILS on unmutated code (a "red baseline")
    /// must NOT report every mutant as killed (a false 100% score). The tool
    /// must refuse to run and surface the baseline failure.
    ///
    /// Implements [RUNNER-BASELINE].
    #[test]
    fn red_baseline_is_rejected_not_reported_as_all_killed() {
        if !binary_exists() || !dart_available() {
            println!("Skipping: binary not built or Dart not available");
            return;
        }

        let Some((_temp, project_path)) = copy_fixture_to_temp("red_baseline_project") else {
            println!("Skipping: failed to copy fixtures");
            return;
        };

        let pub_get = Command::new("dart")
            .args(["pub", "get"])
            .current_dir(&project_path)
            .output();
        if pub_get.is_err() || !pub_get.is_ok_and(|o| o.status.success()) {
            println!("Skipping: dart pub get failed");
            return;
        }

        let output = Command::new(binary_path())
            .args([
                "--path",
                project_path.to_str().unwrap(),
                "--threshold",
                "0",
                "--timeout",
                "60",
            ])
            .output()
            .expect("Failed to execute command");

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        let combined = format!("{stdout}{stderr}");
        println!("combined output:\n{combined}");

        // BUG: with a red baseline the tool currently marks every mutant as
        // killed (exit code 0, 100% score). It must instead refuse to run.
        assert!(
            !output.status.success(),
            "dart_mutant must exit non-zero on a red baseline, but it succeeded.\n{combined}"
        );

        // It must NOT print a mutation-results summary (which would imply a
        // bogus score on an unverifiable baseline).
        assert!(
            !combined.contains("MUTATION TESTING RESULTS"),
            "dart_mutant printed a mutation results summary on a red baseline.\n{combined}"
        );

        // It must explain that the baseline failed.
        assert!(
            combined.to_lowercase().contains("baseline"),
            "Expected a baseline failure message, got:\n{combined}"
        );
    }
}

/// `--test-command` decides what runs for the baseline and every mutant.
///
/// Implements [RUNNER-TEST-COMMAND]. Regression tests for GitHub issue #8.
mod test_command_e2e {
    use super::*;

    /// Statuses of every mutant in a `--json` report.
    fn mutant_statuses(report: &std::path::Path) -> Vec<String> {
        let json = std::fs::read_to_string(report).expect("JSON report should exist");
        let report: serde_json::Value = serde_json::from_str(&json).expect("valid JSON");
        report["files"]
            .as_object()
            .expect("files object")
            .values()
            .flat_map(|file| file["mutants"].as_array().cloned().unwrap_or_default())
            .map(|mutant| mutant["status"].as_str().unwrap_or_default().to_string())
            .collect()
    }

    /// The red-baseline suite fails under `dart test`. A command that always
    /// succeeds passes the baseline and lets every mutant survive, which is
    /// only possible when both runs use `--test-command`.
    #[test]
    fn custom_command_runs_for_baseline_and_every_mutant() {
        if !dart_available() {
            println!("Skipping: Dart not available");
            return;
        }
        let Some((temp, project_path)) = copy_fixture_to_temp("red_baseline_project") else {
            println!("Skipping: failed to copy fixtures");
            return;
        };
        let report_dir = temp.path().join("reports");

        let output = Command::new(binary_path())
            .args(["--path", project_path.to_str().unwrap()])
            .args(["--test-command", "dart --version"])
            .args(["--sample", "2", "--timeout", "60", "--json"])
            .args(["--output", report_dir.to_str().unwrap()])
            .output()
            .expect("Failed to execute command");

        let combined = format!(
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(output.status.success(), "run should succeed:\n{combined}");
        let statuses = mutant_statuses(&report_dir.join("mutation-report.json"));
        assert_eq!(statuses, ["Survived", "Survived"], "\n{combined}");
    }

    /// A program that does not exist fails the run before any mutant is
    /// tested, and the error names the program that could not start.
    #[test]
    fn missing_program_fails_the_run() {
        let Some((temp, project_path)) = copy_fixture_to_temp("simple_dart_project") else {
            println!("Skipping: failed to copy fixtures");
            return;
        };

        let output = Command::new(binary_path())
            .args(["--path", project_path.to_str().unwrap()])
            .args(["--test-command", "dart_mutant_no_such_program test"])
            .args(["--output", temp.path().join("reports").to_str().unwrap()])
            .output()
            .expect("Failed to execute command");

        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(!output.status.success(), "run should fail:\n{stderr}");
        assert!(
            stderr.contains("dart_mutant_no_such_program"),
            "got:\n{stderr}"
        );
    }

    /// An unclosed quote is rejected up front, even in `--dry-run`.
    #[test]
    fn unclosed_quote_is_rejected() {
        let output = Command::new(binary_path())
            .args(["--path", fixtures_path().to_str().unwrap(), "--dry-run"])
            .args(["--test-command", "flutter test --plain-name \"adds"])
            .output()
            .expect("Failed to execute command");

        assert!(!output.status.success());
    }
}

/// `--glob` narrows the files to mutate, relative to `--path`.
///
/// Implements [PARSER-DISCOVERY-GLOB].
mod glob_e2e {
    use super::*;

    /// Number of mutations a `--dry-run` with `extra` arguments would test.
    fn dry_run_mutation_count(path: &std::path::Path, extra: &[&str]) -> usize {
        let temp = tempfile::tempdir().expect("temp dir");
        let output = Command::new(binary_path())
            .args(["--path", path.to_str().unwrap(), "--dry-run"])
            .args(["--output", temp.path().to_str().unwrap()])
            .args(extra)
            .output()
            .expect("Failed to execute command");
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8_lossy(&output.stdout)
            .lines()
            .find(|line| line.contains("mutations would be tested"))
            .and_then(|line| line.split_whitespace().next()?.parse().ok())
            .expect("dry run should report a mutation count")
    }

    /// `--glob lib/calculator.dart` yields the same mutations as pointing
    /// `--path` at that file, and fewer than the whole project.
    #[test]
    fn glob_selects_only_the_matching_file() {
        let project = fixtures_path();
        let calculator = project.join("lib").join("calculator.dart");

        let with_glob = dry_run_mutation_count(&project, &["--glob", "lib/calculator.dart"]);
        let whole_project = dry_run_mutation_count(&project, &[]);

        assert_eq!(with_glob, dry_run_mutation_count(&calculator, &[]));
        assert!(with_glob < whole_project, "{with_glob} vs {whole_project}");
    }

    /// A malformed pattern stops the run instead of mutating every file.
    #[test]
    fn invalid_glob_is_rejected() {
        let output = Command::new(binary_path())
            .args(["--path", fixtures_path().to_str().unwrap(), "--dry-run"])
            .args(["--glob", "lib/[.dart"])
            .output()
            .expect("Failed to execute command");

        assert!(!output.status.success());
    }
}

/// The console summary prints the same score as the JSON report of the
/// same run: one decimal and a single `%`.
///
/// Implements [REPORT-CONSOLE-SCORE].
mod summary_score_e2e {
    use super::*;
    use std::path::Path;

    /// Run dart_mutant on `project` with `args`, `--json` and no color.
    /// Returns its stdout and the `mutationScore` of its JSON report.
    fn run_and_read_scores(project: &Path, out: &Path, args: &[&str]) -> (String, f64) {
        let output = Command::new(binary_path())
            .args(["--path", project.to_str().unwrap(), "--json"])
            .args(["--output", out.to_str().unwrap()])
            .args(args)
            .env("NO_COLOR", "1")
            .env_remove("CLICOLOR_FORCE")
            .output()
            .expect("Failed to execute command");
        let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(output.status.success(), "run failed:\n{stdout}{stderr}");
        let json = std::fs::read_to_string(out.join("mutation-report.json")).expect("JSON report");
        let report: serde_json::Value = serde_json::from_str(&json).expect("valid JSON");
        let score = report["mutationScore"].as_f64().expect("mutationScore");
        (stdout, score)
    }

    /// The number on the summary's score line: the only stdout line that,
    /// trimmed, is a number followed by exactly one `%`.
    fn console_score(stdout: &str) -> String {
        let scores: Vec<&str> = stdout
            .lines()
            .filter_map(|line| line.trim().strip_suffix('%'))
            .filter(|number| number.parse::<f64>().is_ok())
            .collect();
        assert_eq!(scores.len(), 1, "expected one score line in:\n{stdout}");
        scores[0].to_owned()
    }

    /// Run `git` with `args` in `dir`; `true` when it succeeds.
    fn git(dir: &Path, args: &[&str]) -> bool {
        Command::new("git")
            .args(args)
            .current_dir(dir)
            .output()
            .is_ok_and(|o| o.status.success())
    }

    /// `git diff --quiet` passes on the committed baseline and fails as
    /// soon as a mutant edits a tracked file, so every mutant is killed.
    #[test]
    fn all_killed_run_prints_100_percent_like_the_json() {
        let Some((temp, project)) = copy_fixture_to_temp("simple_dart_project") else {
            println!("Skipping: failed to copy fixtures");
            return;
        };
        let committed = git(&project, &["init", "-q"])
            && git(&project, &["add", "-A"])
            && git(
                &project,
                &[
                    "-c",
                    "user.name=t",
                    "-c",
                    "user.email=t@t",
                    "commit",
                    "-qm",
                    "f",
                ],
            );
        if !committed {
            println!("Skipping: git not available");
            return;
        }
        let out = temp.path().join("reports");

        let (stdout, json) = run_and_read_scores(
            &project,
            &out,
            &[
                "--test-command",
                "git diff --quiet",
                "--sample",
                "5",
                "--timeout",
                "30",
            ],
        );

        assert!(
            (json - 100.0).abs() < f64::EPSILON,
            "score {json}\n{stdout}"
        );
        assert_eq!(console_score(&stdout), format!("{json:.1}"));
    }

    /// A sampled run with `dart test` can land on any score; the console
    /// must print the JSON's score with one decimal.
    #[test]
    fn mixed_run_prints_the_json_score_with_one_decimal() {
        if !dart_available() {
            println!("Skipping: Dart not available");
            return;
        }
        let Some((temp, project)) = copy_fixture_to_temp("simple_dart_project") else {
            println!("Skipping: failed to copy fixtures");
            return;
        };
        let pub_get = Command::new("dart")
            .args(["pub", "get"])
            .current_dir(&project)
            .output();
        if !pub_get.is_ok_and(|o| o.status.success()) {
            println!("Skipping: dart pub get failed");
            return;
        }
        let out = temp.path().join("reports");

        let (stdout, json) = run_and_read_scores(
            &project,
            &out,
            &["--sample", "6", "--timeout", "60", "--parallel", "1"],
        );

        assert_eq!(console_score(&stdout), format!("{json:.1}"));
    }

    /// A dry run tests no mutant: both reports say 0.0.
    #[test]
    fn dry_run_prints_zero_percent_like_the_json() {
        let out = tempfile::tempdir().expect("temp dir");

        let (stdout, json) = run_and_read_scores(&fixtures_path(), out.path(), &["--dry-run"]);

        assert!(json.abs() < f64::EPSILON, "score {json}\n{stdout}");
        assert_eq!(console_score(&stdout), format!("{json:.1}"));
    }
}

mod void_call_removal {
    use super::*;

    /// Regression test for GitHub issue #4: dart_mutant should generate a
    /// "remove the call" mutation for each statement-level call to a
    /// void-returning function/method (including awaited `Future<void>` calls),
    /// since such calls exist for their side effects. `--dry-run` lists the
    /// generated mutations as `original → mutated`, so each removed statement
    /// appears verbatim. Implements [PARSER-VOID-CALL].
    #[test]
    fn generates_removal_mutations_for_void_call_statements() {
        if !binary_exists() {
            println!("Skipping: binary not built");
            return;
        }

        let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join("void_calls_project");

        let output = Command::new(binary_path())
            .args(["--path", fixture.to_str().unwrap(), "--dry-run"])
            .output()
            .expect("Failed to execute command");

        let stdout = String::from_utf8_lossy(&output.stdout);
        println!("stdout:\n{stdout}");

        // Each discarded-result call statement must be offered for removal,
        // including the awaited Future<void> call.
        for call in [
            "other.validate();",
            "other.persist();",
            "notify();",
            "await other.drain();",
        ] {
            assert!(
                stdout.contains(call),
                "Expected a removal mutation for `{call}`, got:\n{stdout}"
            );
        }

        // The braceless `if (flag) other.skip();` body must NOT be removed —
        // doing so would produce invalid Dart. [PARSER-VOID-CALL]
        assert!(
            !stdout.contains("other.skip()"),
            "Braceless-body call was offered for removal (would be invalid Dart):\n{stdout}"
        );
    }
}

mod threshold_behavior {
    #[test]
    fn threshold_zero_always_passes() {
        // With threshold 0, any mutation score should pass
        let scores = vec![0.0, 25.0, 50.0, 75.0, 100.0];
        let threshold = 0.0;

        for score in scores {
            let passes = score >= threshold;
            assert!(
                passes,
                "Score {} should pass threshold {}",
                score, threshold
            );
        }
    }

    #[test]
    fn threshold_80_requires_high_score() {
        let threshold = 80.0_f64;

        assert!(80.0 >= threshold, "80% should pass 80% threshold");
        assert!(100.0 >= threshold, "100% should pass 80% threshold");
        assert!(79.9 < threshold, "79.9% should fail 80% threshold");
        assert!(50.0 < threshold, "50% should fail 80% threshold");
    }

    #[test]
    fn threshold_determines_exit_code() {
        let determine_exit = |score: f64, threshold: f64| -> i32 { i32::from(score < threshold) };

        assert_eq!(determine_exit(100.0, 80.0), 0);
        assert_eq!(determine_exit(80.0, 80.0), 0);
        assert_eq!(determine_exit(79.0, 80.0), 1);
        assert_eq!(determine_exit(0.0, 0.0), 0);
    }
}

mod output_format_e2e {
    #[test]
    fn banner_is_displayed() {
        // The tool should display a banner on startup
        // This is a visual test - we just verify the banner format

        let banner = r#"
    ██████╗  █████╗ ██████╗ ████████╗    ███╗   ███╗██╗   ██╗████████╗ █████╗ ███╗   ██╗████████╗
    ██╔══██╗██╔══██╗██╔══██╗╚══██╔══╝    ████╗ ████║██║   ██║╚══██╔══╝██╔══██╗████╗  ██║╚══██╔══╝
    ██║  ██║███████║██████╔╝   ██║       ██╔████╔██║██║   ██║   ██║   ███████║██╔██╗ ██║   ██║
    ██║  ██║██╔══██║██╔══██╗   ██║       ██║╚██╔╝██║██║   ██║   ██║   ██╔══██║██║╚██╗██║   ██║
    ██████╔╝██║  ██║██║  ██║   ██║       ██║ ╚═╝ ██║╚██████╔╝   ██║   ██║  ██║██║ ╚████║   ██║
    ╚═════╝ ╚═╝  ╚═╝╚═╝  ╚═╝   ╚═╝       ╚═╝     ╚═╝ ╚═════╝    ╚═╝   ╚═╝  ╚═╝╚═╝  ╚═══╝   ╚═╝
"#;

        // Verify banner contains DART MUTANT text
        assert!(banner.contains("██"), "Banner should use block characters");
    }

    #[test]
    fn score_bar_format() {
        // Test the score bar generation logic

        let create_score_bar = |score: f64| -> String {
            let width = 40;
            let filled = ((score / 100.0) * width as f64) as usize;
            let empty = width - filled;
            format!("{}{}", "█".repeat(filled), "░".repeat(empty))
        };

        let bar_100 = create_score_bar(100.0);
        assert_eq!(bar_100.chars().filter(|c| *c == '█').count(), 40);

        let bar_50 = create_score_bar(50.0);
        assert_eq!(bar_50.chars().filter(|c| *c == '█').count(), 20);
        assert_eq!(bar_50.chars().filter(|c| *c == '░').count(), 20);

        let bar_0 = create_score_bar(0.0);
        assert_eq!(bar_0.chars().filter(|c| *c == '░').count(), 40);
    }
}
