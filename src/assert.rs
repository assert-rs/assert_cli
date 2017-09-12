use std::default;
use std::process::Command;
use std::path::PathBuf;
use std::vec::Vec;

use errors::*;
use output::{OutputAssertion, StdErr, StdOut};

/// Assertions for a specific command.
#[derive(Debug)]
pub struct Assert {
    cmd: Vec<String>,
    current_dir: Option<PathBuf>,
    expect_success: Option<bool>,
    expect_exit_code: Option<i32>,
    expect_stdout: Vec<OutputAssertion<StdOut>>,
    expect_stderr: Vec<OutputAssertion<StdErr>>,
}

impl default::Default for Assert {
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
            expect_stdout: vec![],
            expect_stderr: vec![],
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
        self.expect_stdout.push(OutputAssertion {
            expect: output.into(),
            fuzzy: true,
            expected_result: true,
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
        self.expect_stdout.push(OutputAssertion {
            expect: output.into(),
            fuzzy: false,
            expected_result: true,
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
        self.expect_stderr.push(OutputAssertion {
            expect: output.into(),
            fuzzy: true,
            expected_result: true,
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
        self.expect_stderr.push(OutputAssertion {
            expect: output.into(),
            fuzzy: false,
            expected_result: true,
            kind: StdErr,
        });
        self
    }

    /// Expect the command's output to not **contain** `output`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// extern crate assert_cli;
    ///
    /// assert_cli::Assert::command(&["echo", "42"])
    ///     .doesnt_print("73")
    ///     .execute()
    ///     .unwrap();
    /// ```
    pub fn doesnt_print<O: Into<String>>(mut self, output: O) -> Self {
        self.expect_stdout.push(OutputAssertion {
            expect: output.into(),
            fuzzy: true,
            expected_result: false,
            kind: StdOut,
        });
        self
    }

    /// Expect the command to output to not be **exactly** this `output`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// extern crate assert_cli;
    ///
    /// assert_cli::Assert::command(&["echo", "42"])
    ///     .doesnt_print_exactly("73")
    ///     .execute()
    ///     .unwrap();
    /// ```
    pub fn doesnt_print_exactly<O: Into<String>>(mut self, output: O) -> Self {
        self.expect_stdout.push(OutputAssertion {
            expect: output.into(),
            fuzzy: false,
            expected_result: false,
            kind: StdOut,
        });
        self
    }

    /// Expect the command's stderr output to not **contain** `output`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// extern crate assert_cli;
    ///
    /// assert_cli::Assert::command(&["cat", "non-existing-file"])
    ///     .fails()
    ///     .and()
    ///     .doesnt_print_error("content")
    ///     .execute()
    ///     .unwrap();
    /// ```
    pub fn doesnt_print_error<O: Into<String>>(mut self, output: O) -> Self {
        self.expect_stderr.push(OutputAssertion {
            expect: output.into(),
            fuzzy: true,
            expected_result: false,
            kind: StdErr,
        });
        self
    }

    /// Expect the command to output to not be **exactly** this `output` to stderr.
    ///
    /// # Examples
    ///
    /// ```rust
    /// extern crate assert_cli;
    ///
    /// assert_cli::Assert::command(&["cat", "non-existing-file"])
    ///     .fails_with(1)
    ///     .and()
    ///     .doesnt_print_error_exactly("content")
    ///     .execute()
    ///     .unwrap();
    /// ```
    pub fn doesnt_print_error_exactly<O: Into<String>>(mut self, output: O) -> Self {
        self.expect_stderr.push(OutputAssertion {
            expect: output.into(),
            fuzzy: false,
            expected_result: false,
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
            None => command,
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

        self.expect_stdout
            .iter()
            .map(|a| a.execute(&output).map_err(|e| ErrorKind::StdoutMismatch(self.cmd.clone(), e).into()))
            .collect::<Result<Vec<()>>>()?;
        self.expect_stderr
            .iter()
            .map(|a| a.execute(&output).map_err(|e| ErrorKind::StderrMismatch(self.cmd.clone(), e).into()))
            .collect::<Result<Vec<()>>>()?;

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
