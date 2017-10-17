use self::errors::*;
pub use self::errors::{Error, ErrorKind};
use diff;
use difference::Changeset;
use std::fmt;
use std::process::Output;
use std::rc::Rc;
use std::result::Result as StdResult;

#[derive(Clone)]
pub struct OutputAssertion {
    pub test: Rc<Fn(&str) -> Result<()>>,
    pub kind: OutputKind,
}

impl OutputAssertion {
    pub fn execute(&self, output: &Output, cmd: &[String]) -> super::errors::Result<()> {
        let observed = String::from_utf8_lossy(self.kind.select(output));
        let result = (self.test)(&observed);
        result.map_err(|e| super::errors::ErrorKind::OutputMismatch(cmd.to_vec(), e, self.kind))?;
        Ok(())
    }
}

impl fmt::Debug for OutputAssertion {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
               "OutputAssertion {{ test: [user supplied closure], kind: {:?} }}",
               self.kind)
    }
}

pub fn matches_fuzzy(got: &str, expect: &str, expected_result: bool) -> Result<()> {
    let result = got.contains(expect);
    if result != expected_result {
        if expected_result {
            bail!(ErrorKind::OutputDoesntContain(expect.into(), got.into()));
        } else {
            bail!(ErrorKind::OutputContains(expect.into(), got.into()));
        }
    }

    Ok(())
}

pub fn matches_exact(got: &str, expect: &str, expected_result: bool) -> Result<()> {
    let differences = Changeset::new(expect.trim(), got.trim(), "\n");
    let result = differences.distance == 0;

    if result != expected_result {
        if expected_result {
            let nice_diff = diff::render(&differences)?;
            bail!(ErrorKind::OutputDoesntMatch(expect.to_owned(), got.to_owned(), nice_diff));
        } else {
            bail!(ErrorKind::OutputMatches(got.to_owned()));
        }
    }

    Ok(())
}

pub fn matches_pred(got: &str, pred: &Fn(&str) -> bool) -> Result<()> {
    match pred(got) {
        true => Ok(()),
        false => bail!(ErrorKind::PredicateFails(got.to_owned(), None)),
    }
}

pub fn matches_pred_ok(got: &str, pred_ok: &Fn(&str) -> StdResult<(), String>) -> Result<()> {
    match pred_ok(got) {
        Ok(()) => Ok(()),
        Err(s) => bail!(ErrorKind::PredicateFails(got.to_owned(), Some(s.to_owned()))),
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
            PredicateFails(got: String, err_str: Option<String>) {
                description("User-supplied predicate failed")
                display("Error string: {:?}\noutput=```{}```", err_str, got)
            }
        }
    }
}
