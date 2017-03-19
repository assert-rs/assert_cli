//! # Test CLI Applications
//!
//! This crate's goal is to provide you some very easy tools to test your CLI
//! applications. It can currently execute child processes and validate their
//! exit status as well as stdout output against your assertions.
//!
//! ## Examples
//!
//! Here's a trivial example:
//!
//! ```rust extern crate assert_cli;
//!
//! assert_cli::Assert::command(&["echo", "42"])
//!     .succeeds()
//!     .and().prints("42")
//!     .unwrap();
//! ```
//!
//! And here is one that will fail:
//!
//! ```rust,should_panic
//! assert_cli::Assert::command(&["echo", "42"])
//!     .prints("1337")
//!     .unwrap();
//! ```
//!
//! this will show a nice, colorful diff in your terminal, like this:
//!
//! ```diff
//! -1337
//! +42
//! ```
//!
//! If you are testing a Rust binary crate, you can start with
//! `Assert::main_binary()` to use `cargo run` as command. Or, if you want to
//! run a specific binary (if you have more than one), use
//! `Assert::cargo_binary`.
//!
//! Alternatively, you can use the `assert_cmd!` macro to construct the command:
//!
//! ```rust
//! #[macro_use] extern crate assert_cli;
//!
//! # fn main() {
//! assert_cmd!(echo 42).succeeds().prints("42").unwrap();
//! # }
//! ```
//!
//! (Make sure to include the crate as `#[macro_use] extern crate assert_cli;`!)
//!
//! If you don't want it to panic when the assertions are not met, simply call
//! `.execute` instead of `.unwrap` to get a `Result`:
//!
//! ```rust
//! #[macro_use] extern crate assert_cli;
//!
//! # fn main() {
//! let x = assert_cmd!(echo 1337).prints_exactly("42").execute();
//! assert!(x.is_err());
//! # }
//! ```

#![deny(warnings, missing_docs)]

extern crate difference;
#[macro_use] extern crate error_chain;

use std::process::Command;

use difference::Changeset;

mod errors;
use errors::*;

mod diff;

/// Assertions for a specific command
#[derive(Debug)]
pub struct Assert {
    cmd: Vec<String>,
    expect_success: bool,
    expect_exit_code: Option<i32>,
    expect_output: Option<String>,
    fuzzy_output: bool,
    expect_error_output: Option<String>,
    fuzzy_error_output: bool,
}

impl std::default::Default for Assert {
    /// Construct an assert using `cargo run --` as command.
    fn default() -> Self {
        Assert {
            cmd: vec!["cargo", "run", "--"]
                .into_iter().map(String::from).collect(),
            expect_success: true,
            expect_exit_code: None,
            expect_output: None,
            fuzzy_output: false,
            expect_error_output: None,
            fuzzy_error_output: false,
        }
    }
}

impl Assert {
    /// Use the crate's main binary as command
    pub fn main_binary() -> Self {
        Assert::default()
    }

    /// Use the crate's main binary as command
    pub fn cargo_binary(name: &str) -> Self {
        Assert {
            cmd: vec!["cargo", "run", "--bin", name, "--"]
                .into_iter().map(String::from).collect(),
            ..Self::default()
        }
    }

    /// Use custom command
    ///
    /// # Examples
    ///
    /// ```rust
    /// extern crate assert_cli;
    ///
    /// assert_cli::Assert::command(&["echo", "1337"])
    ///     .succeeds()
    ///     .unwrap();
    /// ```
    pub fn command(cmd: &[&str]) -> Self {
        Assert {
            cmd: cmd.into_iter().cloned().map(String::from).collect(),
            ..Self::default()
        }
    }

    /// Add arguments to the command
    ///
    /// # Examples
    ///
    /// ```rust
    /// extern crate assert_cli;
    ///
    /// assert_cli::Assert::command(&["echo"])
    ///     .with_args(&["42"])
    ///     .succeeds()
    ///     .prints("42")
    ///     .unwrap();
    /// ```
    pub fn with_args(mut self, args: &[&str]) -> Self {
        self.cmd.extend(args.into_iter().cloned().map(String::from));
        self
    }

    /// Small helper to make chains more readable
    ///
    /// # Examples
    ///
    /// ```rust
    /// extern crate assert_cli;
    ///
    /// assert_cli::Assert::command(&["echo", "42"])
    ///     .succeeds().and().prints("42")
    ///     .unwrap();
    /// ```
    pub fn and(self) -> Self {
        self
    }

    /// Expect the command to be executed successfully
    ///
    /// # Examples
    ///
    /// ```rust
    /// extern crate assert_cli;
    ///
    /// assert_cli::Assert::command(&["echo", "42"])
    ///     .succeeds()
    ///     .unwrap();
    /// ```
    pub fn succeeds(mut self) -> Self {
        self.expect_success = true;
        self
    }

    /// Expect the command to fail
    ///
    /// # Examples
    ///
    /// ```rust
    /// extern crate assert_cli;
    ///
    /// assert_cli::Assert::command(&["cat", "non-exisiting-file"])
    ///     .fails()
    ///     .unwrap();
    /// ```
    pub fn fails(mut self) -> Self {
        self.expect_success = false;
        self
    }

    /// Expect the command to fail and return a specific error code
    ///
    /// # Examples
    ///
    /// ```rust
    /// extern crate assert_cli;
    ///
    /// assert_cli::Assert::command(&["cat", "non-exisiting-file"])
    ///     .fails_with(1)
    ///     .unwrap();
    /// ```
    pub fn fails_with(mut self, expect_exit_code: i32) -> Self {
        self.expect_success = false;
        self.expect_exit_code = Some(expect_exit_code);
        self
    }

    /// Expect the command's output to contain `output`
    ///
    /// # Examples
    ///
    /// ```rust
    /// extern crate assert_cli;
    ///
    /// assert_cli::Assert::command(&["echo", "42"])
    ///     .prints("42")
    ///     .unwrap();
    /// ```
    pub fn prints<O: Into<String>>(mut self, output: O) -> Self {
        self.expect_output = Some(output.into());
        self.fuzzy_output = true;
        self
    }

    /// Expect the command to output exactly this `output`
    ///
    /// # Examples
    ///
    /// ```rust
    /// extern crate assert_cli;
    ///
    /// assert_cli::Assert::command(&["echo", "42"])
    ///     .prints_exactly("42")
    ///     .unwrap();
    /// ```
    pub fn prints_exactly<O: Into<String>>(mut self, output: O) -> Self {
        self.expect_output = Some(output.into());
        self.fuzzy_output = false;
        self
    }

    /// Expect the command's stderr output to contain `output`
    ///
    /// # Examples
    ///
    /// ```rust
    /// extern crate assert_cli;
    ///
    /// assert_cli::Assert::command(&["cat", "non-exisiting-file"])
    ///     .fails()
    ///     .prints_error("non-exisiting-file")
    ///     .unwrap();
    /// ```
    pub fn prints_error<O: Into<String>>(mut self, output: O) -> Self {
        self.expect_error_output = Some(output.into());
        self.fuzzy_error_output = true;
        self
    }

    /// Expect the command to output exactly this `output` to stderr
    ///
    /// # Examples
    ///
    /// ```rust
    /// extern crate assert_cli;
    ///
    /// assert_cli::Assert::command(&["cat", "non-exisiting-file"])
    ///     .fails()
    ///     .prints_error_exactly("cat: non-exisiting-file: No such file or directory")
    ///     .unwrap();
    /// ```
    pub fn prints_error_exactly<O: Into<String>>(mut self, output: O) -> Self {
        self.expect_error_output = Some(output.into());
        self.fuzzy_error_output = false;
        self
    }

    /// Execute the command and check the assertions
    ///
    /// # Examples
    ///
    /// ```rust
    /// extern crate assert_cli;
    ///
    /// let test = assert_cli::Assert::command(&["echo", "42"])
    ///     .succeeds()
    ///     .execute();
    /// assert!(test.is_ok());
    /// ```
    pub fn execute(self) -> Result<()> {
        let cmd = &self.cmd[0];
        let args: Vec<_> = self.cmd.iter().skip(1).collect();
        let mut command = Command::new(cmd);
        let command = command.args(&args);
        let output = command.output()?;


        if self.expect_success != output.status.success() {
            bail!(ErrorKind::StatusMismatch(
                self.cmd.clone(),
                self.expect_success.clone(),
            ));
        }

        if self.expect_exit_code.is_some() &&
            self.expect_exit_code != output.status.code() {
            bail!(ErrorKind::ExitCodeMismatch(
                self.cmd.clone(),
                self.expect_exit_code,
                output.status.code(),
            ));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        match (self.expect_output, self.fuzzy_output) {
            (Some(ref expected_output), true) if !stdout.contains(expected_output) => {
                bail!(ErrorKind::OutputMismatch(
                    expected_output.clone(),
                    stdout.into(),
                ));
            },
            (Some(ref expected_output), false) => {
                let differences = Changeset::new(expected_output.trim(), stdout.trim(), "\n");
                if differences.distance > 0 {
                    let nice_diff = diff::render(&differences)?;
                    bail!(ErrorKind::ExactOutputMismatch(nice_diff));
                }
            },
            _ => {},
        }

        let stderr = String::from_utf8_lossy(&output.stderr);
        match (self.expect_error_output, self.fuzzy_error_output) {
            (Some(ref expected_output), true) if !stderr.contains(expected_output) => {
                bail!(ErrorKind::ErrorOutputMismatch(
                    expected_output.clone(),
                    stderr.into(),
                ));
            },
            (Some(ref expected_output), false) => {
                let differences = Changeset::new(expected_output.trim(), stderr.trim(), "\n");
                if differences.distance > 0 {
                    let nice_diff = diff::render(&differences)?;
                    bail!(ErrorKind::ExactErrorOutputMismatch(nice_diff));
                }
            },
            _ => {},
        }

        Ok(())
    }

    /// Execute the command, check the assertions, and panic when they fail
    ///
    /// # Examples
    ///
    /// ```rust,should_panic="Assert CLI failure"
    /// extern crate assert_cli;
    ///
    /// assert_cli::Assert::command(&["echo", "42"])
    ///     .fails()
    ///     .unwrap(); // panics
    /// ```
    pub fn unwrap(self) {
        if let Err(err) = self.execute() {
            panic!("Assert CLI failure:\n{}", err);
        }
    }
}

/// Easily construct an `Assert` with a custom command
///
/// Make sure to include the crate as `#[macro_use] extern crate assert_cli;` if
/// you want to use this macro.
///
/// # Examples
///
/// To test that our very complex cli applications succeeds and prints some
/// text to stdout that contains
///
/// ```plain
/// No errors whatsoever
/// ```
///
/// you would call it like this:
///
/// ```rust
/// #[macro_use] extern crate assert_cli;
/// # fn main() {
/// assert_cmd!(echo "Launch sequence initiated.\nNo errors whatsoever!\n")
///     .succeeds()
///     .prints("No errors whatsoever")
///     .unwrap();
/// # }
/// ```
///
/// The macro will try to convert its arguments as strings, but is limited by
/// Rust's default tokenizer, e.g., you always need to quote CLI arguments
/// like `"--verbose"`.
#[macro_export]
macro_rules! assert_cmd {
    ($($x:tt)+) => {{
        $crate::Assert::command(
            &[$(stringify!($x)),*]
        )
    }}
}
