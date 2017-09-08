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
//!     .prints("42")
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
//! ## `assert_cmd!` Macro
//!
//! Alternatively, you can use the `assert_cmd!` macro to construct the command more conveniently,
//! but please carefully read the limitations below, or this may seriously go wrong.
//!
//! ```rust
//! # #[macro_use] extern crate assert_cli;
//! # fn main() {
//! assert_cmd!(echo "42").prints("42").unwrap();
//! # }
//! ```
//!
//! **Tips**
//!
//! - Don't forget to import the crate with `#[macro_use]`. ;-)
//! - Enclose arguments in the `assert_cmd!` macro in quotes `"`,
//!   if there are special characters, which the macro doesn't accept, e.g.
//!   `assert_cmd!(cat "foo.txt")`.
//!
//! ## Exit Status
//!
//! All assertion default to checking that the command exited with success.
//!
//! However, when you expect a command to fail, you can express it like this:
//!
//! ```rust
//! # #[macro_use] extern crate assert_cli;
//! # fn main() {
//! assert_cmd!(cat "non-existing-file")
//!     .fails()
//!     .and()
//!     .prints_error("non-existing-file")
//!     .unwrap();
//! # }
//! ```
//!
//! Some notes on this:
//!
//! - Use `fails_with` to assert a specific exit status.
//! - There is also a `succeeds` method, but this is already the implicit default
//!   and can usually be omitted.
//! - We can inspect the output of **stderr** with `prints_error` and `prints_error_exactly`.
//! - The `and` method has no effect, other than to make everything more readable.
//!   Feel free to use it. :-)
//!
//! ## Assert CLI Crates
//!
//! If you are testing a Rust binary crate, you can start with
//! `Assert::main_binary()` to use `cargo run` as command. Or, if you want to
//! run a specific binary (if you have more than one), use
//! `Assert::cargo_binary`.
//!
//! ## Don't Panic!
//!
//! If you don't want it to panic when the assertions are not met, simply call
//! `.execute` instead of `.unwrap` to get a `Result`:
//!
//! ```rust
//! # #[macro_use] extern crate assert_cli;
//! # fn main() {
//! let x = assert_cmd!(echo "1337").prints_exactly("42").execute();
//! assert!(x.is_err());
//! # }
//! ```

#![deny(missing_docs)]

extern crate difference;
#[macro_use] extern crate error_chain;
extern crate rustc_serialize;

use std::process::Command;
use std::path::PathBuf;

mod errors;
use errors::*;

#[macro_use] mod macros;
pub use macros::flatten_escaped_string;

mod output;
use output::{OutputAssertion, StdErr, StdOut};

mod diff;

/// Assertions for a specific command.
#[derive(Debug)]
pub struct Assert {
    cmd: Vec<String>,
    current_dir: Option<PathBuf>,
    expect_success: Option<bool>,
    expect_exit_code: Option<i32>,
    expect_stdout: Option<OutputAssertion<StdOut>>,
    expect_stderr: Option<OutputAssertion<StdErr>>,
}

impl std::default::Default for Assert {
    /// Construct an assert using `cargo run --` as command.
    ///
    /// Defaults to asserting _successful_ execution.
    fn default() -> Self {
        Assert {
            cmd: vec!["cargo", "run", "--"]
                .into_iter().map(String::from).collect(),
            current_dir: None,
            expect_success: Some(true),
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
    ///     .prints("42")
    ///     .unwrap();
    /// ```
    pub fn with_args(mut self, args: &[&str]) -> Self {
        self.cmd.extend(args.into_iter().cloned().map(String::from));
        self
    }

    /// Sets the working directory for the command.
    ///
    /// # Examples
    ///
    /// ```rust
    /// extern crate assert_cli;
    ///
    /// assert_cli::Assert::command(&["wc", "lib.rs"])
    ///     .current_dir(std::path::Path::new("src"))
    ///     .prints("lib.rs")
    ///     .execute()
    ///     .unwrap();
    /// ```
    pub fn current_dir<P: Into<PathBuf>>(mut self, dir: P) -> Self {
        self.current_dir = Some(dir.into());
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
    ///     .prints("42")
    ///     .unwrap();
    /// ```
    pub fn and(self) -> Self {
        self
    }

    /// Expect the command to be executed successfully.
    ///
    /// # Examples
    ///
    /// ```rust
    /// extern crate assert_cli;
    ///
    /// assert_cli::Assert::command(&["echo", "42"])
    ///     .unwrap();
    /// ```
    pub fn succeeds(mut self) -> Self {
        self.expect_exit_code = None;
        self.expect_success = Some(true);
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
    /// assert_cli::Assert::command(&["cat", "non-existing-file"])
    ///     .fails()
    ///     .and()
    ///     .prints_error("non-existing-file")
    ///     .unwrap();
    /// ```
    pub fn fails(mut self) -> Self {
        self.expect_success = Some(false);
        self
    }

    /// Expect the command to fail and return a specific error code.
    ///
    /// # Examples
    ///
    /// ```rust
    /// extern crate assert_cli;
    ///
    /// assert_cli::Assert::command(&["cat", "non-existing-file"])
    ///     .fails_with(1)
    ///     .and()
    ///     .prints_error_exactly("cat: non-existing-file: No such file or directory")
    ///     .unwrap();
    /// ```
    pub fn fails_with(mut self, expect_exit_code: i32) -> Self {
        self.expect_success = Some(false);
        self.expect_exit_code = Some(expect_exit_code);
        self
    }

    /// Expect the command's output to **contain** `output`.
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
            kind: StdOut,
        });
        self
    }

    /// Expect the command to output **exactly** this `output`.
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
            kind: StdOut,
        });
        self
    }

    /// Expect the command's stderr output to **contain** `output`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// extern crate assert_cli;
    ///
    /// assert_cli::Assert::command(&["cat", "non-existing-file"])
    ///     .fails()
    ///     .and()
    ///     .prints_error("non-existing-file")
    ///     .unwrap();
    /// ```
    pub fn prints_error<O: Into<String>>(mut self, output: O) -> Self {
        self.expect_stderr = Some(OutputAssertion {
            expect: output.into(),
            fuzzy: true,
            kind: StdErr,
        });
        self
    }

    /// Expect the command to output **exactly** this `output` to stderr.
    ///
    /// # Examples
    ///
    /// ```rust
    /// extern crate assert_cli;
    ///
    /// assert_cli::Assert::command(&["cat", "non-existing-file"])
    ///     .fails_with(1)
    ///     .and()
    ///     .prints_error_exactly("cat: non-existing-file: No such file or directory")
    ///     .unwrap();
    /// ```
    pub fn prints_error_exactly<O: Into<String>>(mut self, output: O) -> Self {
        self.expect_stderr = Some(OutputAssertion {
            expect: output.into(),
            fuzzy: false,
            kind: StdErr,
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
    ///     .prints("42")
    ///     .execute();
    /// assert!(test.is_ok());
    /// ```
    pub fn execute(self) -> Result<()> {
        let cmd = &self.cmd[0];
        let args: Vec<_> = self.cmd.iter().skip(1).collect();
        let mut command = Command::new(cmd);
        let command = command.args(&args);
        let command = match self.current_dir {
            Some(ref dir) => command.current_dir(dir),
            None => command
        };
        let output = command.output()?;


        if let Some(expect_success) = self.expect_success {
            if expect_success != output.status.success() {
                bail!(ErrorKind::StatusMismatch(
                    self.cmd.clone(),
                    expect_success,
                ));
            }
        }

        if self.expect_exit_code.is_some() &&
            self.expect_exit_code != output.status.code() {
            bail!(ErrorKind::ExitCodeMismatch(
                self.cmd.clone(),
                self.expect_exit_code,
                output.status.code(),
            ));
        }

        if let Some(ref ouput_assertion) = self.expect_stdout {
            ouput_assertion.execute(&output)
                .map_err(|e| ErrorKind::StdoutMismatch(self.cmd.clone(), e))?;
        }

        if let Some(ref ouput_assertion) = self.expect_stderr {
            ouput_assertion.execute(&output)
                .map_err(|e| ErrorKind::StderrMismatch(self.cmd.clone(), e))?;
        }

        Ok(())
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
