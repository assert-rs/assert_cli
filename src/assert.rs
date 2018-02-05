extern crate regex;

use environment::Environment;
use error_chain::ChainedError;
use errors::*;
use output::{Content, Output, OutputKind, OutputPredicate};
use std::default;
use std::ffi::{OsStr, OsString};
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::vec::Vec;

/// Assertions for a specific command.
#[derive(Debug)]
#[must_use]
pub struct Assert {
    cmd: Vec<OsString>,
    env: Environment,
    current_dir: Option<PathBuf>,
    expect_success: Option<bool>,
    expect_exit_code: Option<i32>,
    expect_output: Vec<OutputPredicate>,
    stdin_contents: Option<Vec<u8>>,
}

impl default::Default for Assert {
    /// Construct an assert using `cargo run --` as command.
    ///
    /// Defaults to asserting _successful_ execution.
    fn default() -> Self {
        Assert {
            cmd: vec!["cargo", "run", "--quiet", "--"]
                .into_iter()
                .map(OsString::from)
                .collect(),
            env: Environment::inherit(),
            current_dir: None,
            expect_success: Some(true),
            expect_exit_code: None,
            expect_output: vec![],
            stdin_contents: None,
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
    pub fn cargo_binary<S: AsRef<OsStr>>(name: S) -> Self {
        Assert {
            cmd: vec![
                OsStr::new("cargo"),
                OsStr::new("run"),
                OsStr::new("--quiet"),
                OsStr::new("--bin"),
                name.as_ref(),
                OsStr::new("--"),
            ].into_iter()
                .map(OsString::from)
                .collect(),
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
    pub fn command<S: AsRef<OsStr>>(cmd: &[S]) -> Self {
        Assert {
            cmd: cmd.into_iter().map(OsString::from).collect(),
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
    ///     .stdout().contains("42")
    ///     .unwrap();
    ///
    /// ```
    pub fn with_args<S: AsRef<OsStr>>(mut self, args: &[S]) -> Self {
        self.cmd.extend(args.into_iter().map(OsString::from));
        self
    }

    /// Add stdin to the command.
    ///
    /// # Examples
    ///
    /// ```rust
    /// extern crate assert_cli;
    ///
    /// assert_cli::Assert::command(&["cat"])
    ///     .stdin("42")
    ///     .stdout().contains("42")
    ///     .unwrap();
    /// ```
    pub fn stdin<S: Into<Vec<u8>>>(mut self, contents: S) -> Self {
        self.stdin_contents = Some(contents.into());
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
    ///     .stdout().contains("lib.rs")
    ///     .execute()
    ///     .unwrap();
    /// ```
    pub fn current_dir<P: Into<PathBuf>>(mut self, dir: P) -> Self {
        self.current_dir = Some(dir.into());
        self
    }

    /// Sets environments variables for the command.
    ///
    /// # Examples
    ///
    /// ```rust
    /// extern crate assert_cli;
    ///
    /// assert_cli::Assert::command(&["printenv"])
    ///     .with_env(&[("TEST_ENV", "OK")])
    ///     .stdout().contains("TEST_ENV=OK")
    ///     .execute()
    ///     .unwrap();
    ///
    /// let env = assert_cli::Environment::empty()
    ///     .insert("FOO", "BAR");
    ///
    /// assert_cli::Assert::command(&["printenv"])
    ///     .with_env(&env)
    ///     .stdout().is("FOO=BAR")
    ///     .execute()
    ///     .unwrap();
    ///
    /// ::std::env::set_var("BAZ", "BAR");
    ///
    /// assert_cli::Assert::command(&["printenv"])
    ///     .stdout().contains("BAZ=BAR")
    ///     .execute()
    ///     .unwrap();
    /// ```
    pub fn with_env<E: Into<Environment>>(mut self, env: E) -> Self {
        self.env = env.into();

        self
    }

    /// Small helper to make chains more readable.
    ///
    /// # Examples
    ///
    /// ```rust
    /// extern crate assert_cli;
    ///
    /// assert_cli::Assert::command(&["cat", "non-existing-file"])
    ///     .fails()
    ///     .and()
    ///     .stderr().contains("non-existing-file")
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
    ///     .succeeds()
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
    ///     .stderr().contains("non-existing-file")
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
    ///     .stderr().is("cat: non-existing-file: No such file or directory")
    ///     .unwrap();
    /// ```
    pub fn fails_with(mut self, expect_exit_code: i32) -> Self {
        self.expect_success = Some(false);
        self.expect_exit_code = Some(expect_exit_code);
        self
    }

    /// Do not care whether the command exits successfully or if it fails.
    ///
    /// This function removes any assertions that were already set, including
    /// any expected exit code that was set with [`fails_with`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// extern crate assert_cli;
    ///
    /// assert_cli::Assert::command(&["cat", "non-existing-file"])
    ///     .ignore_status()
    ///     .and()
    ///     .stderr().is("cat: non-existing-file: No such file or directory")
    ///     .unwrap();
    /// ```
    ///
    /// [`fails_with`]: #method.fails_with
    pub fn ignore_status(mut self) -> Self {
        self.expect_exit_code = None;
        self.expect_success = None;
        self
    }

    /// Create an assertion for stdout's contents
    ///
    /// # Examples
    ///
    /// ```rust
    /// extern crate assert_cli;
    ///
    /// assert_cli::Assert::command(&["echo", "42"])
    ///     .stdout().contains("42")
    ///     .unwrap();
    /// ```
    pub fn stdout(self) -> OutputAssertionBuilder {
        OutputAssertionBuilder {
            assertion: self,
            kind: OutputKind::StdOut,
        }
    }

    /// Create an assertion for stdout's contents
    ///
    /// # Examples
    ///
    /// ```rust
    /// extern crate assert_cli;
    ///
    /// assert_cli::Assert::command(&["cat", "non-existing-file"])
    ///     .fails_with(1)
    ///     .and()
    ///     .stderr().is("cat: non-existing-file: No such file or directory")
    ///     .unwrap();
    /// ```
    pub fn stderr(self) -> OutputAssertionBuilder {
        OutputAssertionBuilder {
            assertion: self,
            kind: OutputKind::StdErr,
        }
    }

    /// Execute the command and check the assertions.
    ///
    /// # Examples
    ///
    /// ```rust
    /// extern crate assert_cli;
    ///
    /// let test = assert_cli::Assert::command(&["echo", "42"])
    ///     .stdout().contains("42")
    ///     .execute();
    /// assert!(test.is_ok());
    /// ```
    pub fn execute(self) -> Result<()> {
        let bin = &self.cmd[0];

        let args: Vec<_> = self.cmd.iter().skip(1).collect();
        let mut command = Command::new(bin);
        let command = command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .env_clear()
            .envs(self.env.clone().compile())
            .args(&args);

        let command = match self.current_dir {
            Some(ref dir) => command.current_dir(dir),
            None => command,
        };

        let mut spawned = command.spawn().chain_err(
            || ErrorKind::SpawnFailed(self.cmd.clone()),
        )?;

        if let Some(ref contents) = self.stdin_contents {
            spawned
                .stdin
                .as_mut()
                .expect("Couldn't get mut ref to command stdin")
                .write_all(contents)?;
        }
        let output = spawned.wait_with_output()?;

        if let Some(expect_success) = self.expect_success {
            if expect_success != output.status.success() {
                let out = String::from_utf8_lossy(&output.stdout).to_string();
                let err = String::from_utf8_lossy(&output.stderr).to_string();
                let err: Error = ErrorKind::StatusMismatch(expect_success, out, err).into();
                bail!(err.chain_err(
                    || ErrorKind::AssertionFailed(self.cmd.clone()),
                ));
            }
        }

        if self.expect_exit_code.is_some() && self.expect_exit_code != output.status.code() {
            let out = String::from_utf8_lossy(&output.stdout).to_string();
            let err = String::from_utf8_lossy(&output.stderr).to_string();
            let err: Error =
                ErrorKind::ExitCodeMismatch(self.expect_exit_code, output.status.code(), out, err)
                    .into();
            bail!(err.chain_err(
                || ErrorKind::AssertionFailed(self.cmd.clone()),
            ));
        }

        self.expect_output
            .iter()
            .map(|a| {
                a.verify(&output).chain_err(|| {
                    ErrorKind::AssertionFailed(self.cmd.clone())
                })
            })
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
            panic!("{}", err.display_chain());
        }
    }
}

/// Assertions for command output.
#[derive(Debug)]
#[must_use]
pub struct OutputAssertionBuilder {
    assertion: Assert,
    kind: OutputKind,
}

impl OutputAssertionBuilder {
    /// Expect the command's output to **contain** `output`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// extern crate assert_cli;
    ///
    /// assert_cli::Assert::command(&["echo", "42"])
    ///     .stdout().contains("42")
    ///     .unwrap();
    /// ```
    pub fn contains<O: Into<Content>>(mut self, output: O) -> Assert {
        let pred = OutputPredicate::new(self.kind, Output::contains(output));
        self.assertion.expect_output.push(pred);
        self.assertion
    }

    /// Expect the command to output **exactly** this `output`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// extern crate assert_cli;
    ///
    /// assert_cli::Assert::command(&["echo", "42"])
    ///     .stdout().is("42")
    ///     .unwrap();
    /// ```
    pub fn is<O: Into<Content>>(mut self, output: O) -> Assert {
        let pred = OutputPredicate::new(self.kind, Output::is(output));
        self.assertion.expect_output.push(pred);
        self.assertion
    }

    /// Expect the command to match **however many times** this `output`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// extern crate assert_cli;
    /// assert_cli::Assert::command(&["echo", "42"])
    ///     .stdout().matches("[0-9]{2}")
    ///     .unwrap();
    /// ```
    pub fn matches(mut self, output: String) -> Assert {
        let pred = OutputPredicate::new(self.kind, Output::matches(output));
        self.assertion.expect_output.push(pred);
        self.assertion
    }

    /// Expect the command to match `nmatches` times this `output`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// extern crate assert_cli;
    /// assert_cli::Assert::command(&["echo", "42"])
    ///     .stdout().matches_ntimes("[0-9]{1}", 2)
    ///     .unwrap();
    /// ```
    pub fn matches_ntimes(mut self, output: String, nmatches: u32) -> Assert {
        let pred = OutputPredicate::new(self.kind, Output::matches_ntimes(output, nmatches));
        self.assertion.expect_output.push(pred);
        self.assertion
    }

    /// Expect the command's output to not **contain** `output`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// extern crate assert_cli;
    ///
    /// assert_cli::Assert::command(&["echo", "42"])
    ///     .stdout().doesnt_contain("73")
    ///     .unwrap();
    /// ```
    pub fn doesnt_contain<O: Into<Content>>(mut self, output: O) -> Assert {
        let pred = OutputPredicate::new(self.kind, Output::doesnt_contain(output));
        self.assertion.expect_output.push(pred);
        self.assertion
    }

    /// Expect the command to output to not be **exactly** this `output`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// extern crate assert_cli;
    ///
    /// assert_cli::Assert::command(&["echo", "42"])
    ///     .stdout().isnt("73")
    ///     .unwrap();
    /// ```
    pub fn isnt<O: Into<Content>>(mut self, output: O) -> Assert {
        let pred = OutputPredicate::new(self.kind, Output::isnt(output));
        self.assertion.expect_output.push(pred);
        self.assertion
    }

    /// Expect the command output to satisfy the given predicate.
    ///
    /// # Examples
    ///
    /// ```rust
    /// extern crate assert_cli;
    ///
    /// assert_cli::Assert::command(&["echo", "-n", "42"])
    ///     .stdout().satisfies(|x| x.len() == 2, "bad length")
    ///     .unwrap();
    /// ```
    pub fn satisfies<F, M>(mut self, pred: F, msg: M) -> Assert
    where
        F: 'static + Fn(&str) -> bool,
        M: Into<String>,
    {
        let pred = OutputPredicate::new(self.kind, Output::satisfies(pred, msg));
        self.assertion.expect_output.push(pred);
        self.assertion
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::ffi::OsString;

    fn command() -> Assert {
        Assert::command(&["printenv"])
    }

    #[test]
    fn take_ownership() {
        let x = Environment::inherit();

        command()
            .with_env(x.clone())
            .with_env(&x)
            .with_env(x)
            .unwrap();
    }

    #[test]
    fn in_place_mod() {
        let y = Environment::empty();

        let y = y.insert("key", "value");

        assert_eq!(
            y.compile(),
            vec![(OsString::from("key"), OsString::from("value"))]
        );
    }

    #[test]
    fn in_place_mod2() {
        let x = Environment::inherit();

        command()
            .with_env(&x.insert("key", "value").insert("key", "vv"))
            .stdout()
            .contains("key=vv")
            .execute()
            .unwrap();
        // Granted, `insert` moved `x`, so we can no longer reference it, even
        // though only a reference was passed to `with_env`
    }

    #[test]
    fn in_place_mod3() {
        // In-place modification while allowing later accesses to the `Environment`
        let y = Environment::empty();

        assert_eq!(
            y.clone().insert("key", "value").compile(),
            vec![(OsString::from("key"), OsString::from("value"))]
        );

        command()
            .with_env(y)
            .stdout()
            .doesnt_contain("key=value")
            .execute()
            .unwrap();
    }

    #[test]
    fn empty_env() {
        // In-place modification while allowing later accesses to the `Environment`
        let y = Environment::empty();

        assert!(command().with_env(y).stdout().is("").execute().is_ok());
    }
    #[test]
    fn take_vec() {
        let v = vec![("bar".to_string(), "baz".to_string())];

        command()
            .with_env(&vec![("bar", "baz")])
            .stdout()
            .contains("bar=baz")
            .execute()
            .unwrap();

        command()
            .with_env(&v)
            .stdout()
            .contains("bar=baz")
            .execute()
            .unwrap();

        command()
            .with_env(&vec![("bar", "baz")])
            .stdout()
            .isnt("")
            .execute()
            .unwrap();
    }

    #[test]
    fn take_slice_of_strs() {
        command()
            .with_env(&[("bar", "BAZ")])
            .stdout()
            .contains("bar=BAZ")
            .execute()
            .unwrap();

        command()
            .with_env(&[("bar", "BAZ")][..])
            .stdout()
            .contains("bar=BAZ")
            .execute()
            .unwrap();

        command()
            .with_env([("bar", "BAZ")].as_ref())
            .stdout()
            .contains("bar=BAZ")
            .execute()
            .unwrap();
    }

    #[test]
    fn take_slice_of_strings() {
        // same deal as above

        command()
            .with_env(&[("bar".to_string(), "BAZ".to_string())])
            .stdout()
            .contains("bar=BAZ")
            .execute()
            .unwrap();

        command()
            .with_env(&[("bar".to_string(), "BAZ".to_string())][..])
            .stdout()
            .contains("bar=BAZ")
            .execute()
            .unwrap();
    }

    #[test]
    fn take_slice() {
        command()
            .with_env(&[("hey", "ho")])
            .stdout()
            .contains("hey=ho")
            .execute()
            .unwrap();

        command()
            .with_env(&[("hey", "ho".to_string())])
            .stdout()
            .contains("hey=ho")
            .execute()
            .unwrap();
    }

    #[test]
    fn take_string_i32() {
        command()
            .with_env(&[("bar", 3 as i32)])
            .stdout()
            .contains("bar=3")
            .execute()
            .unwrap();
    }
}
