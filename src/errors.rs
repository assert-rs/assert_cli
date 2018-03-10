use std::ffi;
use std::fmt;
use std::result;

use failure;

pub type Result<T> = result::Result<T, failure::Error>;

fn format_cmd(cmd: &[ffi::OsString]) -> String {
    let result: Vec<String> = cmd.iter()
        .map(|s| s.to_string_lossy().into_owned())
        .collect();
    result.join(" ")
}

#[derive(Fail, Debug)]
pub struct SpawnError {
    cmd: Vec<ffi::OsString>,
}

impl SpawnError {
    pub fn new(cmd: Vec<ffi::OsString>) -> Self {
        Self { cmd }
    }
}

impl fmt::Display for SpawnError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Failed to run `{}`", format_cmd(&self.cmd))
    }
}

#[derive(Fail, Debug)]
pub struct AssertionError {
    cmd: Vec<ffi::OsString>,
}

impl AssertionError {
    pub fn new(cmd: Vec<ffi::OsString>) -> Self {
        Self { cmd }
    }
}

impl fmt::Display for AssertionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Assertion failed for `{}`", format_cmd(&self.cmd))
    }
}

#[derive(Fail, Debug)]
pub struct StatusError {
    unexpected: bool,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
}

impl StatusError {
    pub fn new(unexpected: bool, stdout: Vec<u8>, stderr: Vec<u8>) -> Self {
        Self {
            unexpected,
            stdout,
            stderr,
        }
    }
}

impl fmt::Display for StatusError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let out = String::from_utf8_lossy(&self.stdout);
        let err = String::from_utf8_lossy(&self.stderr);
        writeln!(
            f,
            "Unexpected {} return status",
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

#[derive(Fail, Debug)]
pub struct ExitCodeError {
    expected: Option<i32>,
    got: Option<i32>,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
}

impl ExitCodeError {
    pub fn new(expected: Option<i32>, got: Option<i32>, stdout: Vec<u8>, stderr: Vec<u8>) -> Self {
        Self {
            expected,
            got,
            stdout,
            stderr,
        }
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
