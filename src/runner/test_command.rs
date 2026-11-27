//! The test command run against the unmutated baseline and every mutant.
//!
//! Implements [RUNNER-TEST-COMMAND]: `--test-command` decides what runs, so a
//! Flutter package can use `flutter test` where the default `dart test` cannot
//! load `flutter_test`.

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::process::Stdio;
use tokio::process::Command;
use tracing::{debug, info, trace};

/// A parsed `--test-command`: the program to spawn and its arguments.
#[derive(Debug, Clone)]
pub struct TestCommand {
    /// The program as the user wrote it, for messages.
    name: String,
    /// The program resolved through `PATH`, or `name` when it has a path.
    program: PathBuf,
    /// The arguments passed to `program`.
    args: Vec<String>,
}

impl TestCommand {
    /// Parse a `--test-command` value and resolve its program.
    ///
    /// Whitespace separates words. Single or double quotes group a word that
    /// contains spaces. Backslashes are literal, so Windows paths need no
    /// escaping.
    pub fn parse(command: &str) -> Result<Self> {
        let mut words = split_words(command)?.into_iter();
        let name = words.next().context("--test-command is empty")?;
        let program = resolve_program(&name);
        let args: Vec<String> = words.collect();
        info!(program = %program.display(), args = args.len(), "using test command");
        debug!(?args, "test command arguments");
        Ok(Self {
            name,
            program,
            args,
        })
    }

    /// Run the command in `project_path` and return (exit code, stdout, stderr).
    pub async fn run(&self, project_path: &Path) -> Result<(i32, String, String)> {
        trace!(project = %project_path.display(), program = %self.name, "spawning test command");
        let output = Command::new(&self.program)
            .args(&self.args)
            .current_dir(project_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .with_context(|| format!("Failed to run test command `{}`", self.name))?;

        let exit_code = output.status.code().unwrap_or(-1);
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        Ok((exit_code, stdout, stderr))
    }
}

/// Resolve a bare program name through `PATH` (and `PATHEXT` on Windows, so
/// `flutter` finds `flutter.bat`). A name with a path separator is used as is.
fn resolve_program(name: &str) -> PathBuf {
    if name.contains(std::path::is_separator) {
        return PathBuf::from(name);
    }
    which::which(name).unwrap_or_else(|error| {
        debug!(program = name, %error, "program not found on PATH; spawning by name");
        PathBuf::from(name)
    })
}

/// Split a command line into words. Quotes group words; nothing is escaped.
fn split_words(command: &str) -> Result<Vec<String>> {
    let mut words = Vec::new();
    let mut word: Option<String> = None;
    let mut quote: Option<char> = None;
    for c in command.chars() {
        match quote {
            Some(open) if c == open => quote = None,
            Some(_) => word.get_or_insert_with(String::new).push(c),
            None if c == '"' || c == '\'' => {
                quote = Some(c);
                word.get_or_insert_with(String::new);
            }
            None if c.is_whitespace() => words.extend(word.take()),
            None => word.get_or_insert_with(String::new).push(c),
        }
    }
    anyhow::ensure!(
        quote.is_none(),
        "--test-command has an unclosed quote: {command}"
    );
    words.extend(word);
    Ok(words)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn quoted_words_keep_their_spaces() {
        let words = split_words(r#"flutter test --plain-name "adds two numbers""#).unwrap();

        assert_eq!(
            words,
            ["flutter", "test", "--plain-name", "adds two numbers"]
        );
    }

    #[test]
    fn backslashes_in_windows_paths_stay_literal() {
        let words = split_words(r"C:\fvm\versions\3.50.1\bin\flutter.bat test").unwrap();

        assert_eq!(words, [r"C:\fvm\versions\3.50.1\bin\flutter.bat", "test"]);
    }

    #[test]
    fn unclosed_quote_is_rejected() {
        assert!(split_words(r#"flutter test --name "half"#).is_err());
    }

    #[test]
    fn blank_command_is_rejected() {
        assert!(TestCommand::parse("   ").is_err());
    }

    #[test]
    fn program_with_a_path_is_spawned_as_written() {
        let command = TestCommand::parse("tool/run_tests.sh --fast").unwrap();

        assert_eq!(command.program, PathBuf::from("tool/run_tests.sh"));
        assert_eq!(command.args, ["--fast"]);
    }
}
