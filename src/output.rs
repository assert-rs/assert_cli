use std::fmt;
use std::process;
use std::rc;

use difference::Changeset;

use diff;
use self::errors::*;
pub use self::errors::{Error, ErrorKind};


#[derive(Debug, Clone, PartialEq, Eq)]
struct IsPredicate {
    pub expect: String,
    pub expected_result: bool,
}

impl IsPredicate {
    pub fn verify_str(&self, got: &str) -> Result<()> {
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
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ContainsPredicate {
    pub expect: String,
    pub expected_result: bool,
}

impl ContainsPredicate {
    pub fn verify_str(&self, got: &str) -> Result<()> {
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
}

#[derive(Clone)]
struct FnPredicate {
    pub pred: rc::Rc<Fn(&str) -> bool>,
    pub msg: String,
}

impl FnPredicate {
    pub fn verify_str(&self, got: &str) -> Result<()> {
        let pred = &self.pred;
        if ! pred(got) {
            bail!(ErrorKind::PredicateFailed(got.into(), self.msg.clone()));
        }

        Ok(())
    }
}

impl fmt::Debug for FnPredicate {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.msg)
    }
}

#[derive(Debug, Clone)]
enum StrPredicate {
    Is(IsPredicate),
    Contains(ContainsPredicate),
	Fn(FnPredicate),
}

impl StrPredicate {
    pub fn verify_str(&self, got: &str) -> Result<()> {
        match *self {
            StrPredicate::Is(ref pred) => pred.verify_str(got),
            StrPredicate::Contains(ref pred) => pred.verify_str(got),
            StrPredicate::Fn(ref pred) => pred.verify_str(got),
        }
    }
}

/// Assertions for command output.
#[derive(Debug, Clone)]
pub struct Output {
    pred: StrPredicate,
}

impl Output {
    /// Expect the command's output to **contain** `output`.
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
    /// ```
    pub fn contains<O: Into<String>>(output: O) -> Self {
        let pred = ContainsPredicate {
            expect: output.into(),
            expected_result: true,
        };
        Self::new(StrPredicate::Contains(pred))
    }

    /// Expect the command to output **exactly** this `output`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// extern crate assert_cli;
    ///
    /// assert_cli::Assert::command(&["echo"])
    ///     .with_args(&["42"])
    ///     .stdout().is("42")
    ///     .unwrap();
    /// ```
    pub fn is<O: Into<String>>(output: O) -> Self {
        let pred = IsPredicate {
            expect: output.into(),
            expected_result: true,
        };
        Self::new(StrPredicate::Is(pred))
    }

    /// Expect the command's output to not **contain** `output`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// extern crate assert_cli;
    ///
    /// assert_cli::Assert::command(&["echo"])
    ///     .with_args(&["42"])
    ///     .stdout().doesnt_contain("73")
    ///     .unwrap();
    /// ```
    pub fn doesnt_contain<O: Into<String>>(output: O) -> Self {
        let pred = ContainsPredicate {
            expect: output.into(),
            expected_result: false,
        };
        Self::new(StrPredicate::Contains(pred))
    }

    /// Expect the command to output to not be **exactly** this `output`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// extern crate assert_cli;
    ///
    /// assert_cli::Assert::command(&["echo"])
    ///     .with_args(&["42"])
    ///     .stdout().isnt("73")
    ///     .unwrap();
    /// ```
    pub fn isnt<O: Into<String>>(output: O) -> Self {
        let pred = IsPredicate {
            expect: output.into(),
            expected_result: false,
        };
        Self::new(StrPredicate::Is(pred))
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
    pub fn satisfies<F, M>(pred: F, msg: M) -> Self
        where F: 'static + Fn(&str) -> bool,
              M: Into<String>
    {
        let pred = FnPredicate {
            pred: rc::Rc::new(pred),
            msg: msg.into(),
        };
        Self::new(StrPredicate::Fn(pred))
    }

    fn new(pred: StrPredicate) -> Self {
        Self { pred }
    }

    pub(crate) fn verify_str(&self, got: &str) -> Result<()> {
        self.pred.verify_str(got)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum OutputKind {
    StdOut,
    StdErr,
}

impl OutputKind {
    pub fn select(self, o: &process::Output) -> &[u8] {
        match self {
            OutputKind::StdOut => &o.stdout,
            OutputKind::StdErr => &o.stderr,
        }
    }
}

#[derive(Debug, Clone)]
pub struct OutputPredicate {
    kind: OutputKind,
    pred: Output,
}

impl OutputPredicate {
    pub fn new(kind: OutputKind, pred: Output) -> Self {
        Self {
            kind,
            pred,
        }
    }

    pub(crate) fn verify_str(&self, got: &str) -> Result<()> {
        let kind = self.kind;
        self.pred
            .verify_str(got)
            .chain_err(|| ErrorKind::OutputMismatch(kind))
    }

    pub(crate) fn verify_output(&self, got: &process::Output) -> Result<()> {
        let got = String::from_utf8_lossy(self.kind.select(got));
        self.verify_str(&got)
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
            PredicateFailed(got: String, msg: String) {
                description("Output predicate failed")
                display("{}\noutput=```{}```", msg, got)
            }
            OutputMismatch(kind: super::OutputKind) {
                description("Output was not as expected")
                display(
                    "Unexpected {:?}",
                    kind
                )
            }
        }
    }
}
