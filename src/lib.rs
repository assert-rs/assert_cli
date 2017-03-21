//! # Test CLI Applications
//!
//! This crate's goal is to provide you some very easy tools to test your CLI
//! applications. It can currently execute child processes and validate their
//! exit status as well as stdout and stderr output against your assertions.
//!
//! Include the crate like
//!
//! ```rust
//! #[macro_use] // <-- import the convenience macro (optional)
//! extern crate assert_cli;
//! # fn main() { }
//! ```
//!
//! ## Basic Examples
//!
//! Here's a trivial example:
//!
//! ```rust
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
//!     .prints_exactly("1337")
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
//! ## Assert CLI Crates
//!
//! If you are testing a Rust binary crate, you can start with
//! `Assert::main_binary()` to use `cargo run` as command. Or, if you want to
//! run a specific binary (if you have more than one), use
//! `Assert::cargo_binary`.
//!
//! ## `assert_cmd!` Macro
//!
//! Alternatively, you can use the `assert_cmd!` macro to construct the command more conveniently:
//!
//! ```rust
//! # #[macro_use] extern crate assert_cli;
//! # fn main() {
//! assert_cmd!(echo 42).succeeds().prints("42").unwrap();
//! # }
//! ```
//!
//! Don't forget to import the crate with `#[macro_use]`. ;-)
//!
//! ## Don't Panic!
//!
//! If you don't want it to panic when the assertions are not met, simply call
//! `.execute` instead of `.unwrap` to get a `Result`:
//!
//! ```rust
//! # #[macro_use] extern crate assert_cli;
//! # fn main() {
//! let x = assert_cmd!(echo 1337).prints_exactly("42").execute();
//! assert!(x.is_err());
//! # }
//! ```

#![deny(warnings, missing_docs)]

extern crate difference;
#[macro_use] extern crate error_chain;

use std::process::{Command, Output};
use std::fmt;

use difference::Changeset;

mod errors;
use errors::*;

mod diff;

/// Assertions for a specific command.
#[derive(Debug)]
pub struct Assert {
    cmd: Vec<String>,
    expect_success: bool,
    expect_exit_code: Option<i32>,
    expect_stdout: Option<OutputAssertion>,
    expect_stderr: Option<OutputAssertion>,
}

#[derive(Debug)]
struct OutputAssertion {
    expect: String,
    fuzzy: bool,
}

#[derive(Debug, Copy, Clone)]
enum OutputType {
    StdOut,
    StdErr,
}

impl OutputType {
    fn select<'a>(&self, o: &'a Output) -> &'a [u8] {
        match *self {
            OutputType::StdOut => &o.stdout,
            OutputType::StdErr => &o.stderr,
        }
    }
}

impl fmt::Display for OutputType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            OutputType::StdOut => write!(f, "stdout"),
            OutputType::StdErr => write!(f, "stderr"),
        }
    }
}

impl std::default::Default for Assert {
    /// Construct an assert using `cargo run --` as command.
    ///
    /// Defaults to asserting _successful_ execution.
    fn default() -> Self {
        Assert {
            cmd: vec!["cargo", "run", "--"]
                .into_iter().map(String::from).collect(),
            expect_success: true,
            expect_exit_code: None,
            expect_stdout: None,
            expect_stderr: None,
        }
    }
}

impl Assert {
    /// Run the crate's main binary.
    ///
    /// Defaults to asserting _successful_ execution.
    pub fn main_binary() -> Self {
        Assert::default()
    }

    /// Run a specific binary of the current crate.
    ///
    /// Defaults to asserting _successful_ execution.
    pub fn cargo_binary(name: &str) -> Self {
        Assert {
            cmd: vec!["cargo", "run", "--bin", name, "--"]
                .into_iter().map(String::from).collect(),
            ..Self::default()
        }
    }

    /// Run a custom command.
    ///
    /// Defaults to asserting _successful_ execution.
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

    /// Add arguments to the command.
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

    /// Small helper to make chains more readable.
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

    /// Expect the command to be executed successfully.
    ///
    /// Note: This is already set by default, so you only need this for explicitness.
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

    /// Expect the command to fail.
    ///
    /// Note: This does not include shell failures like `command not found`. I.e. the
    ///       command must _run_ and fail for this assertion to pass.
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

    /// Expect the command to fail and return a specific error code.
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

    /// Expect the command's output to contain `output`.
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
        self.expect_stdout = Some(OutputAssertion {
            expect: output.into(),
            fuzzy: true,
        });
        self
    }

    /// Expect the command to output exactly this `output`.
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
        self.expect_stdout = Some(OutputAssertion {
            expect: output.into(),
            fuzzy: false,
        });
        self
    }

    /// Expect the command's stderr output to contain `output`.
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
        self.expect_stderr = Some(OutputAssertion {
            expect: output.into(),
            fuzzy: true,
        });
        self
    }

    /// Expect the command to output exactly this `output` to stderr.
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
        self.expect_stderr = Some(OutputAssertion {
            expect: output.into(),
            fuzzy: false,
        });
        self
    }

    /// Execute the command and check the assertions.
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

        self.assert_output(OutputType::StdOut, &output)?;
        self.assert_output(OutputType::StdErr, &output)?;

        Ok(())
    }

    /// Perform the appropriate output assertion.
    fn assert_output(&self, output_type: OutputType, output: &Output) -> Result<()> {
        let observed = String::from_utf8_lossy(output_type.select(output));
        match *self.expect_output(output_type) {
            Some(OutputAssertion {
                expect: ref expected_output,
                fuzzy: true,
            }) if !observed.contains(expected_output) => {
                bail!(ErrorKind::OutputMismatch(
                    output_type.to_string(),
                    self.cmd.clone(),
                    expected_output.clone(),
                    observed.into(),
                ));
            },
            Some(OutputAssertion {
                expect: ref expected_output,
                fuzzy: false,
            }) => {
                let differences = Changeset::new(expected_output.trim(), observed.trim(), "\n");
                if differences.distance > 0 {
                    let nice_diff = diff::render(&differences)?;
                    bail!(ErrorKind::ExactOutputMismatch(
                        output_type.to_string(),
                        self.cmd.clone(),
                        nice_diff
                    ));
                }
            },
            _ => {},
        }
        Ok(())
    }

    /// Return a reference to the appropriate output assertion.
    fn expect_output(&self, output_type: OutputType) -> &Option<OutputAssertion> {
        match output_type {
            OutputType::StdOut => &self.expect_stdout,
            OutputType::StdErr => &self.expect_stderr,
        }
    }

    /// Execute the command, check the assertions, and panic when they fail.
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
            panic!("{}", err);
        }
    }
}

/// Easily construct an `Assert` with a custom command.
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
/// ...,  you would call it like this:
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
