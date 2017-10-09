use self::errors::*;
pub use self::errors::{Error, ErrorKind};
use diff;
use difference::Changeset;
use std::process::Output;

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
                bail!(ErrorKind::OutputDoesntContain(
                    self.expect.clone(),
                    got.into()
                ));
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
                bail!(ErrorKind::OutputDoesntMatch(
                    self.expect.clone(),
                    got.to_owned(),
                    nice_diff
                ));
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
        result.map_err(|e| {
            super::errors::ErrorKind::OutputMismatch(cmd.to_vec(), e, self.kind)
        })?;

        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
pub enum OutputKind {
    StdOut,
    StdErr,
}

impl OutputKind {
    pub fn select(self, o: &Output) -> &[u8] {
        match self {
            OutputKind::StdOut => &o.stdout,
            OutputKind::StdErr => &o.stderr,
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
                display("expected to contain {:?}\noutput=```{}```", expected, got)
            }
            OutputContains(expected: String, got: String) {
                description("Output was not as expected")
                display("expected to not contain {:?}\noutput=```{}```", expected, got)
            }
            OutputDoesntMatch(expected: String, got: String, diff: String) {
                description("Output was not as expected")
                display("diff:\n{}", diff)
            }
            OutputMatches(got: String) {
                description("Output was not as expected")
                display("expected to not match\noutput=```{}```", got)
            }
        }
    }
}
