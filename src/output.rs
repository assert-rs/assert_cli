use std::fmt;
use std::process::Output;

use difference::Changeset;

use self::errors::*;
pub use self::errors::{Error, ErrorKind};
use diff;

#[derive(Debug, Clone)]
pub struct OutputAssertion<T> {
    pub expect: String,
    pub fuzzy: bool,
    pub expected_result: bool,
    pub kind: T,
}

impl<T: OutputType> OutputAssertion<T> {
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

    pub fn execute(&self, output: &Output) -> Result<()> {
        let observed = String::from_utf8_lossy(self.kind.select(output));

        if self.fuzzy {
            self.matches_fuzzy(&observed)
        } else {
            self.matches_exact(&observed)
        }
    }
}


pub trait OutputType: fmt::Display {
    fn select<'a>(&self, o: &'a Output) -> &'a [u8];
}


#[derive(Debug, Clone, Copy)]
pub struct StdOut;

impl fmt::Display for StdOut {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "stdout")
    }
}

impl OutputType for StdOut {
    fn select<'a>(&self, o: &'a Output) -> &'a [u8] {
        &o.stdout
    }
}


#[derive(Debug, Clone, Copy)]
pub struct StdErr;

impl fmt::Display for StdErr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "stderr")
    }
}

impl OutputType for StdErr {
    fn select<'a>(&self, o: &'a Output) -> &'a [u8] {
        &o.stderr
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
