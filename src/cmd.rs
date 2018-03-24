use std::ffi;
use std::fmt;
use std::io::Write;
use std::io;
use std::process;
use std::str;

use failure;

/// Extend `Command` with helpers for running the current crate's binaries.
pub trait CommandCargoExt {
    /// Create a `Command` to run the crate's main binary.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// extern crate assert_cli;
    /// use std::process::Command;
    /// use assert_cli::cmd::*;
    ///
    /// Command::main_binary()
    ///     .output()
    ///     .unwrap();
    /// ```
    fn main_binary() -> Self;

    /// Create a `Command` Run a specific binary of the current crate.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// extern crate assert_cli;
    /// use std::process::Command;
    /// use assert_cli::cmd::*;
    ///
    /// Command::cargo_binary("assert_fixture")
    ///     .output()
    ///     .unwrap();
    /// ```
    fn cargo_binary<S: AsRef<ffi::OsStr>>(name: S) -> Self;
}

impl CommandCargoExt for process::Command {
    fn main_binary() -> Self {
        let mut cmd = process::Command::new("carg");
        cmd.arg("run").arg("--quit").arg("--");
        cmd
    }

    fn cargo_binary<S: AsRef<ffi::OsStr>>(name: S) -> Self {
        let mut cmd = process::Command::new("carg");
        cmd.arg("run")
            .arg("--quit")
            .arg("--bin")
            .arg(name.as_ref())
            .arg("--");
        cmd
    }
}

/// Extend `Command` with a helper to pass a buffer to `stdin`
pub trait CommandStdInExt {
    /// Write `buffer` to `stdin` when the command is run.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// extern crate assert_cli;
    /// use std::process::Command;
    /// use assert_cli::cmd::*;
    ///
    /// Command::new("cat")
    ///     .with_stdin("42")
    ///     .unwrap();
    /// ```
    fn with_stdin<S>(self, buffer: S) -> StdInCommand
    where
        S: Into<Vec<u8>>;
}

impl CommandStdInExt for process::Command {
    fn with_stdin<S>(self, buffer: S) -> StdInCommand
    where
        S: Into<Vec<u8>>,
    {
        StdInCommand {
            cmd: self,
            stdin: buffer.into(),
        }
    }
}

/// `std::process::Command` with a `stdin` buffer.
pub struct StdInCommand {
    cmd: process::Command,
    stdin: Vec<u8>,
}

impl StdInCommand {
    /// Executes the command as a child process, waiting for it to finish and collecting all of its
    /// output.
    ///
    /// By default, stdout and stderr are captured (and used to provide the resulting output).
    /// Stdin is not inherited from the parent and any attempt by the child process to read from
    /// the stdin stream will result in the stream immediately closing.
    ///
    /// *(mirrors `std::process::Command::output`**
    pub fn output(&mut self) -> io::Result<process::Output> {
        self.spawn()?.wait_with_output()
    }

    /// Executes the command as a child process, returning a handle to it.
    ///
    /// By default, stdin, stdout and stderr are inherited from the parent.
    ///
    /// *(mirrors `std::process::Command::spawn`**
    fn spawn(&mut self) -> io::Result<process::Child> {
        // stdout/stderr should only be piped for `output` according to `process::Command::new`.
        self.cmd.stdin(process::Stdio::piped());
        self.cmd.stdout(process::Stdio::piped());
        self.cmd.stderr(process::Stdio::piped());

        let mut spawned = self.cmd.spawn()?;

        spawned
            .stdin
            .as_mut()
            .expect("Couldn't get mut ref to command stdin")
            .write_all(&self.stdin)?;
        Ok(spawned)
    }
}

/// `std::process::Output` represented as a `Result`.
pub type OutputResult = Result<process::Output, OutputError>;

/// Extends `std::process::Output` with methods to to convert it to an `OutputResult`.
pub trait OutputOkExt
where
    Self: ::std::marker::Sized,
{
    /// Convert an `std::process::Output` into an `OutputResult`.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// extern crate assert_cli;
    /// use std::process::Command;
    /// use assert_cli::cmd::*;
    ///
    /// Command::new("echo")
    ///     .args(&["42"])
    ///     .output()
    ///     .ok()
    ///     .unwrap();
    /// ```
    fn ok(self) -> OutputResult;

    /// Unwrap a `std::process::Output` but with a prettier message than `.ok().unwrap()`.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// extern crate assert_cli;
    /// use std::process::Command;
    /// use assert_cli::cmd::*;
    ///
    /// Command::new("echo")
    ///     .args(&["42"])
    ///     .output()
    ///     .unwrap();
    /// ```
    fn unwrap(self) {
        if let Err(err) = self.ok() {
            panic!("{}", err);
        }
    }
}

impl OutputOkExt for process::Output {
    /// Convert an `std::process::Output` into an `OutputResult`.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// extern crate assert_cli;
    /// use std::process::Command;
    /// use assert_cli::cmd::*;
    ///
    /// Command::new("echo")
    ///     .args(&["42"])
    ///     .output()
    ///     .ok()
    ///     .unwrap();
    /// ```
    fn ok(self) -> OutputResult {
        if self.status.success() {
            Ok(self)
        } else {
            let error = OutputError::new(self);
            Err(error)
        }
    }
}

impl<'c> OutputOkExt for &'c mut process::Command {
    /// Convert an `std::process::Command` into an `OutputResult`.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// extern crate assert_cli;
    /// use std::process::Command;
    /// use assert_cli::cmd::*;
    ///
    /// Command::new("echo")
    ///     .args(&["42"])
    ///     .ok()
    ///     .unwrap();
    /// ```
    fn ok(self) -> OutputResult {
        let output = self.output().map_err(|e| OutputError::with_cause(e))?;
        if output.status.success() {
            Ok(output)
        } else {
            let error = OutputError::new(output).set_cmd(format!("{:?}", self));
            Err(error)
        }
    }
}

impl<'c> OutputOkExt for &'c mut StdInCommand {
    /// Convert an `std::process::Command` into an `OutputResult`.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// extern crate assert_cli;
    /// use std::process::Command;
    /// use assert_cli::cmd::*;
    ///
    /// Command::new("cat")
    ///     .with_stdin("42")
    ///     .ok()
    ///     .unwrap();
    /// ```
    fn ok(self) -> OutputResult {
        let output = self.output().map_err(|e| OutputError::with_cause(e))?;
        if output.status.success() {
            Ok(output)
        } else {
            let error = OutputError::new(output)
                .set_cmd(format!("{:?}", self.cmd))
                .set_stdin(self.stdin.clone());
            Err(error)
        }
    }
}

#[derive(Fail, Debug)]
struct Output {
    output: process::Output,
}

impl fmt::Display for Output {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(code) = self.output.status.code() {
            writeln!(f, "code={}", code)?;
        } else {
            writeln!(f, "code=<interrupted>")?;
        }
        if let Ok(stdout) = str::from_utf8(&self.output.stdout) {
            writeln!(f, "stdout=```{}```", stdout)?;
        } else {
            writeln!(f, "stdout=```{:?}```", self.output.stdout)?;
        }
        if let Ok(stderr) = str::from_utf8(&self.output.stderr) {
            writeln!(f, "stderr=```{}```", stderr)?;
        } else {
            writeln!(f, "stderr=```{:?}```", self.output.stderr)?;
        }

        Ok(())
    }
}

#[derive(Debug)]
enum OutputCause {
    Expected(Output),
    Unexpected(failure::Error),
}

impl fmt::Display for OutputCause {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            OutputCause::Expected(ref e) => write!(f, "{}", e),
            OutputCause::Unexpected(ref e) => write!(f, "{}", e),
        }
    }
}

/// `std::process::Output` as a `Fail`.
#[derive(Fail, Debug)]
pub struct OutputError {
    cmd: Option<String>,
    stdin: Option<Vec<u8>>,
    cause: OutputCause,
}

impl OutputError {
    /// Convert `std::process::Output` into a `Fail`.
    pub fn new(output: process::Output) -> Self {
        Self {
            cmd: None,
            stdin: None,
            cause: OutputCause::Expected(Output { output }),
        }
    }

    /// For errors that happen in creating a `std::process::Output`.
    pub fn with_cause<E>(cause: E) -> Self
    where
        E: Into<failure::Error>,
    {
        Self {
            cmd: None,
            stdin: None,
            cause: OutputCause::Unexpected(cause.into()),
        }
    }

    /// Add the command line for additional context.
    pub fn set_cmd(mut self, cmd: String) -> Self {
        self.cmd = Some(cmd);
        self
    }

    /// Add the `stdn` for additional context.
    pub fn set_stdin(mut self, stdin: Vec<u8>) -> Self {
        self.stdin = Some(stdin);
        self
    }

    /// Access the contained `std::process::Output`.
    pub fn as_output(&self) -> Option<&process::Output> {
        match self.cause {
            OutputCause::Expected(ref e) => Some(&e.output),
            OutputCause::Unexpected(_) => None,
        }
    }
}

impl fmt::Display for OutputError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(ref cmd) = self.cmd {
            writeln!(f, "command=`{}`", cmd)?;
        }
        if let Some(ref stdin) = self.stdin {
            if let Ok(stdin) = str::from_utf8(&stdin) {
                writeln!(f, "stdin=```{}```", stdin)?;
            } else {
                writeln!(f, "stdin=```{:?}```", stdin)?;
            }
        }
        write!(f, "{}", self.cause)
    }
}
