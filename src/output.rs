extern crate regex;

use self::errors::*;
pub use self::errors::{Error, ErrorKind};
use diff;
use difference::Changeset;
use std::ffi::OsString;
use std::process::Output;

#[derive(Debug, Clone)]
pub enum ExpectType {
    STRING(String),
    REGEX(regex::Regex, u32),
    //perdicate?
}

#[derive(Debug, Clone)]
pub struct OutputAssertion {
    pub expect: ExpectType,
    pub fuzzy: bool,
    pub expected_result: bool,
    pub kind: OutputKind,
}

impl OutputAssertion {
    fn matches_fuzzy(&self, got: &str) -> Result<()> {
        match self.expect {
            ExpectType::STRING(ref self_str) => {
                let result = got.contains(self_str);
                if result != self.expected_result {
                    if self.expected_result {
                        bail!(ErrorKind::OutputDoesntContain(self_str.clone(), got.into()));
                    } else {
                        bail!(ErrorKind::OutputContains(self_str.clone(), got.into()));
                    }
                }
            }
            ExpectType::REGEX(ref self_regex, _) => {
                let result = self_regex.is_match(got);
                if result != self.expected_result {
                    bail!(ErrorKind::OutputDoesntMatchRegex(
                        String::from(self_regex.as_str()),
                        got.into(),
                    ));
                }
            }
        }
        Ok(())
    }

    fn matches_exact(&self, got: &str) -> Result<()> {
        match self.expect {
            ExpectType::STRING(ref self_str) => {
                let differences = Changeset::new(self_str.trim(), got.trim(), "\n");
                let result = differences.distance == 0;

                if result != self.expected_result {
                    if self.expected_result {
                        let nice_diff = diff::render(&differences)?;
                        bail!(ErrorKind::OutputDoesntMatch(
                            self_str.clone(),
                            got.to_owned(),
                            nice_diff,
                        ));
                    } else {
                        bail!(ErrorKind::OutputMatches(got.to_owned()));
                    }
                }
            }
            ExpectType::REGEX(ref self_regex, nmatches) => {
                let regex_matches = self_regex.captures_iter(got).count();
                if regex_matches != (nmatches as usize) {
                    bail!(ErrorKind::OutputDoesntMatchRegexExactTimes(
                        String::from(self_regex.as_str()),
                        got.into(),
                        nmatches,
                        regex_matches,
                    ));
                }
            }
        }
        Ok(())
    }

    pub fn execute(&self, output: &Output, cmd: &[OsString]) -> super::errors::Result<()> {
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
            OutputDoesntMatchRegex(regex: String, got: String) {
                description("Regex did not match")
                display("expected {} to match\noutput=```{}```", regex, got)
            }
            OutputDoesntMatchRegexExactTimes(regex: String, got: String, expected_times: u32, got_times: usize) {
                description("Regex did not match exact number of times")
                display("expected {} to match {} {} times\noutput=```{}```", regex, got, expected_times, got_times)
            }
        }
    }
}
