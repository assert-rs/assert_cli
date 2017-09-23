use std::process::Output;

use difference::Changeset;

use self::errors::*;
pub use self::errors::{Error, ErrorKind};
use diff;

#[derive(Debug, Clone)]
pub struct OutputAssertion {
    pub expect: String,
    pub fuzzy: bool,
    pub expected_result: bool,
    pub kind: OutputKind,
}

impl OutputAssertion {
    fn matches_fuzzy(&self, got: &str) -> Result<()> {
        let result = got.contains(&self.expect);
        if result != self.expected_result {
            if self.expected_result {
                bail!(ErrorKind::OutputDoesntContain(self.expect.clone(), got.into()));
            } else {
                bail!(ErrorKind::OutputContains(self.expect.clone(), got.into()));
            }
        }

        Ok(())
    }

    fn matches_exact(&self, got: &str) -> Result<()> {
        let differences = Changeset::new(self.expect.trim(), got.trim(), "\n");
        let result = differences.distance == 0;

        if result != self.expected_result {
            if self.expected_result {
                let nice_diff = diff::render(&differences)?;
                bail!(ErrorKind::OutputDoesntMatch(nice_diff));
            } else {
                bail!(ErrorKind::OutputMatches(got.to_owned()));
            }
        }

        Ok(())
    }

    pub fn execute(&self, output: &Output, cmd: &[String]) -> super::errors::Result<()> {
        let observed = String::from_utf8_lossy(self.kind.select(output));

        let result = if self.fuzzy {
            self.matches_fuzzy(&observed)
        } else {
            self.matches_exact(&observed)
        };
        result.map_err(|e| self.kind.map_err(e, cmd))
    }
}

#[derive(Debug, Clone, Copy)]
pub enum OutputKind {
    StdOut,
    StdErr,
}

impl OutputKind {
    pub fn select<'a>(self, o: &'a Output) -> &'a [u8] {
        match self {
            OutputKind::StdOut => &o.stdout,
            OutputKind::StdErr => &o.stderr,
        }
    }

    pub fn map_err(self, e: Error, cmd: &[String]) -> super::errors::Error {
        match self {
            OutputKind::StdOut => super::errors::ErrorKind::StdoutMismatch(cmd.to_vec(), e).into(),
            OutputKind::StdErr => super::errors::ErrorKind::StderrMismatch(cmd.to_vec(), e).into(),
        }
    }
}

mod errors {
    error_chain! {
        foreign_links {
            Fmt(::std::fmt::Error);
        }
        errors {
            OutputDoesntContain(expected: String, got: String) {
                description("Output was not as expected")
                display("expected to contain {:?}, got {:?}", expected, got)
            }
            OutputContains(expected: String, got: String) {
                description("Output was not as expected")
                display("expected to not contain {:?}, got {:?}", expected, got)
            }
            OutputDoesntMatch(diff: String) {
                description("Output was not as expected")
                display("{}", diff)
            }
            OutputMatches(got: String) {
                description("Output was not as expected")
                display("{}", got)
            }
        }
    }
}
