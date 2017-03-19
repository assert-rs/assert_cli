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
    fuzzy: bool,
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
            fuzzy: false,
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
    pub fn command(cmd: &[&str]) -> Self {
        Assert {
            cmd: cmd.into_iter().cloned().map(String::from).collect(),
            ..Self::default()
        }
    }

    /// Small helper to make chains more readable
    pub fn and(self) -> Self {
        self
    }

    /// Expect the command to be executed successfully
    pub fn succeeds(mut self) -> Self {
        self.expect_success = true;
        self
    }

    /// Expect the command to fail
    pub fn fails(mut self) -> Self {
        self.expect_success = false;
        self
    }

    /// Expect the command to fail and return a specific error code
    pub fn fails_with(mut self, expect_exit_code: i32) -> Self {
        self.expect_success = false;
        self.expect_exit_code = Some(expect_exit_code);
        self
    }

    /// Expect the command's output to contain `output`
    pub fn prints<O: Into<String>>(mut self, output: O) -> Self {
        self.expect_output = Some(output.into());
        self.fuzzy = true;
        self
    }

    /// Expect the command to output exactly this `output`
    pub fn prints_exactly<O: Into<String>>(mut self, output: O) -> Self {
        self.expect_output = Some(output.into());
        self.fuzzy = false;
        self
    }

    /// Execute the command and check the assertions
    pub fn execute(self) -> Result<()> {
        let ref cmd = self.cmd[0];
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
        match (self.expect_output, self.fuzzy) {
            (Some(ref expected_output), true) if !stdout.contains(expected_output) => {
                bail!(ErrorKind::OutputMismatch(
                    expected_output.clone(),
                    stdout.into(),
                ));
            },
            (Some(ref expected_output), false) => {
                let differences = Changeset::new(expected_output.trim(), &stdout.trim(), "\n");
                if differences.distance > 0 {
                    let nice_diff = diff::render(&differences)?;
                    bail!(ErrorKind::ExactOutputMismatch(nice_diff));
                }
            },
            _ => {},
        }

        Ok(())
    }

    /// Execute the command, check the assertions, and panic when they fail
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
#[macro_export]
macro_rules! assert_cmd {
    ($($x:tt)+) => {{
        $crate::Assert::command(
            &[$(stringify!($x)),*]
        )
    }}
}
