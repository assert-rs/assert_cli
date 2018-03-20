use std::ffi;
use std::fmt;
use std::io;

use failure;

fn format_cmd(cmd: &[ffi::OsString]) -> String {
    let result: Vec<String> = cmd.iter()
        .map(|s| s.to_string_lossy().into_owned())
        .collect();
    result.join(" ")
}

pub trait ChainFail {
    fn chain<E>(self, cause: E) -> Self
    where
        E: Into<failure::Error>;
}

pub trait ResultChainExt<T> {
    fn chain<C>(self, chainable: C) -> Result<T, C>
    where
        C: ChainFail;

    fn chain_with<F, C>(self, chainable: F) -> Result<T, C>
    where
        F: FnOnce() -> C,
        C: ChainFail;
}

impl<T> ResultChainExt<T> for Result<T, failure::Error> {
    fn chain<C>(self, chainable: C) -> Result<T, C>
    where
        C: ChainFail,
    {
        self.map_err(|e| chainable.chain(e))
    }

    fn chain_with<F, C>(self, chainable: F) -> Result<T, C>
    where
        F: FnOnce() -> C,
        C: ChainFail,
    {
        self.map_err(|e| chainable().chain(e))
    }
}

impl<T> ResultChainExt<T> for Result<T, io::Error> {
    fn chain<C>(self, chainable: C) -> Result<T, C>
    where
        C: ChainFail,
    {
        self.map_err(|e| chainable.chain(e))
    }

    fn chain_with<F, C>(self, chainable: F) -> Result<T, C>
    where
        F: FnOnce() -> C,
        C: ChainFail,
    {
        self.map_err(|e| chainable().chain(e))
    }
}

/// Failure when processing assertions.
#[derive(Debug)]
pub struct AssertionError {
    cmd: Vec<ffi::OsString>,
    cause: Option<failure::Error>,
}

impl AssertionError {
    pub(crate) fn new(cmd: Vec<ffi::OsString>) -> Self {
        Self { cmd, cause: None }
    }
}

impl failure::Fail for AssertionError {
    fn cause(&self) -> Option<&failure::Fail> {
        self.cause.as_ref().map(failure::Error::cause)
    }

    fn backtrace(&self) -> Option<&failure::Backtrace> {
        None
    }
}

impl ChainFail for AssertionError {
    fn chain<E>(mut self, error: E) -> Self
    where
        E: Into<failure::Error>,
    {
        self.cause = Some(error.into());
        self
    }
}

impl fmt::Display for AssertionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Assertion failed for `{}`", format_cmd(&self.cmd))
    }
}

#[derive(Debug)]
pub struct StatusError {
    unexpected: bool,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
    cause: Option<failure::Error>,
}

impl StatusError {
    pub fn new(unexpected: bool, stdout: Vec<u8>, stderr: Vec<u8>) -> Self {
        Self {
            unexpected,
            stdout,
            stderr,
            cause: None,
        }
    }
}

impl failure::Fail for StatusError {
    fn cause(&self) -> Option<&failure::Fail> {
        self.cause.as_ref().map(failure::Error::cause)
    }

    fn backtrace(&self) -> Option<&failure::Backtrace> {
        None
    }
}

impl ChainFail for StatusError {
    fn chain<E>(mut self, error: E) -> Self
    where
        E: Into<failure::Error>,
    {
        self.cause = Some(error.into());
        self
    }
}

impl fmt::Display for StatusError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let out = String::from_utf8_lossy(&self.stdout);
        let err = String::from_utf8_lossy(&self.stderr);
        writeln!(
            f,
            "Unexpected return status: {}",
            if self.unexpected {
                "success"
            } else {
                "failure"
            }
        )?;
        writeln!(f, "stdout=```{}```", out)?;
        write!(f, "stderr=```{}```", err)
    }
}

#[derive(Debug)]
pub struct ExitCodeError {
    expected: Option<i32>,
    got: Option<i32>,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
    cause: Option<failure::Error>,
}

impl ExitCodeError {
    pub fn new(expected: Option<i32>, got: Option<i32>, stdout: Vec<u8>, stderr: Vec<u8>) -> Self {
        Self {
            expected,
            got,
            stdout,
            stderr,
            cause: None,
        }
    }
}

impl failure::Fail for ExitCodeError {
    fn cause(&self) -> Option<&failure::Fail> {
        self.cause.as_ref().map(failure::Error::cause)
    }

    fn backtrace(&self) -> Option<&failure::Backtrace> {
        None
    }
}

impl ChainFail for ExitCodeError {
    fn chain<E>(mut self, error: E) -> Self
    where
        E: Into<failure::Error>,
    {
        self.cause = Some(error.into());
        self
    }
}

impl fmt::Display for ExitCodeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let out = String::from_utf8_lossy(&self.stdout);
        let err = String::from_utf8_lossy(&self.stderr);
        writeln!(f, "expected={:?}", self.expected)?;
        writeln!(f, "got={:?}", self.got)?;
        writeln!(f, "stdout=```{}```", out)?;
        write!(f, "stderr=```{}```", err)
    }
}
