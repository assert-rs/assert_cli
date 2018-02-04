extern crate regex;

use self::errors::*;
pub use self::errors::{Error, ErrorKind};
use diff;
use difference::Changeset;
use std::fmt;
use std::process;
use std::rc;


#[derive(Clone, PartialEq, Eq)]
pub enum Content {
    Str(String),
    Bytes(Vec<u8>),
}

impl fmt::Debug for Content {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Content::Str(ref data) => write!(f, "{}", data),
            Content::Bytes(ref data) => write!(f, "{:?}", data),
        }
    }
}

impl<'a> From<&'a str> for Content {
    fn from(data: &'a str) -> Self {
        Content::Str(data.into())
    }
}

impl<'a> From<&'a [u8]> for Content {
    fn from(data: &'a [u8]) -> Self {
        Content::Bytes(data.into())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct IsPredicate {
    pub expect: Content,
    pub expected_result: bool,
}

impl IsPredicate {
    pub fn verify(&self, got: &[u8]) -> Result<()> {
        match self.expect {
            Content::Str(ref expect) => {
                self.verify_str(expect, String::from_utf8_lossy(got).as_ref())
            }
            Content::Bytes(ref expect) => self.verify_bytes(expect, got),
        }
    }

    fn verify_bytes(&self, expect: &[u8], got: &[u8]) -> Result<()> {
        let result = expect == got;

        if result != self.expected_result {
            if self.expected_result {
                bail!(ErrorKind::BytesDoesntMatch(
                    expect.to_owned(),
                    got.to_owned(),
                ));
            } else {
                bail!(ErrorKind::BytesMatches(got.to_owned()));
            }
        }
        Ok(())
    }

    fn verify_str(&self, expect: &str, got: &str) -> Result<()> {
        let differences = Changeset::new(expect.trim(), got.trim(), "\n");
        let result = differences.distance == 0;

        if result != self.expected_result {
            if self.expected_result {
                let nice_diff = diff::render(&differences)?;
                bail!(ErrorKind::StrDoesntMatch(
                    expect.to_owned(),
                    got.to_owned(),
                    nice_diff
                ));
            } else {
                bail!(ErrorKind::StrMatches(got.to_owned()));
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ContainsPredicate {
    pub expect: Content,
    pub expected_result: bool,
}

fn find_subsequence(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

#[test]
fn test_find_subsequence() {
    assert_eq!(find_subsequence(b"qwertyuiop", b"tyu"), Some(4));
    assert_eq!(find_subsequence(b"qwertyuiop", b"asd"), None);
}

impl ContainsPredicate {
    pub fn verify(&self, got: &[u8]) -> Result<()> {
        match self.expect {
            Content::Str(ref expect) => {
                self.verify_str(expect, String::from_utf8_lossy(got).as_ref())
            }
            Content::Bytes(ref expect) => self.verify_bytes(expect, got),
        }
    }

    pub fn verify_bytes(&self, expect: &[u8], got: &[u8]) -> Result<()> {
        let result = find_subsequence(got, expect).is_some();
        if result != self.expected_result {
            if self.expected_result {
                bail!(ErrorKind::BytesDoesntContain(
                    expect.to_owned(),
                    got.to_owned()
                ));
            } else {
                bail!(ErrorKind::BytesContains(expect.to_owned(), got.to_owned()));
            }
        }

        Ok(())
    }

    pub fn verify_str(&self, expect: &str, got: &str) -> Result<()> {
        let result = got.contains(expect);
        if result != self.expected_result {
            if self.expected_result {
                bail!(ErrorKind::StrDoesntContain(
                    expect.to_owned(),
                    got.to_owned()
                ));
            } else {
                bail!(ErrorKind::StrContains(expect.to_owned(), got.to_owned()));
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
    pub fn verify(&self, got: &[u8]) -> Result<()> {
        let got = String::from_utf8_lossy(got);
        let pred = &self.pred;
        if !pred(&got) {
            let err: Error = ErrorKind::PredicateFailed(got.into_owned(), self.msg.clone()).into();
            bail!(err);
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
enum ContentPredicate {
    Is(IsPredicate),
    Contains(ContainsPredicate),
    Fn(FnPredicate),
}

impl ContentPredicate {
    pub fn verify(&self, got: &[u8]) -> Result<()> {
        match *self {
            ContentPredicate::Is(ref pred) => pred.verify(got),
            ContentPredicate::Contains(ref pred) => pred.verify(got),
            ContentPredicate::Fn(ref pred) => pred.verify(got),
        }
    }
}

/// Assertions for command output.
#[derive(Debug, Clone)]
pub struct Output {
    pred: ContentPredicate,
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
    pub fn contains<O: Into<Content>>(output: O) -> Self {
        let pred = ContainsPredicate {
            expect: output.into(),
            expected_result: true,
        };
        Self::new(ContentPredicate::Contains(pred))
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
    pub fn is<O: Into<Content>>(output: O) -> Self {
        let pred = IsPredicate {
            expect: output.into(),
            expected_result: true,
        };
        Self::new(ContentPredicate::Is(pred))
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
    pub fn doesnt_contain<O: Into<Content>>(output: O) -> Self {
        let pred = ContainsPredicate {
            expect: output.into(),
            expected_result: false,
        };
        Self::new(ContentPredicate::Contains(pred))
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
    pub fn isnt<O: Into<Content>>(output: O) -> Self {
        let pred = IsPredicate {
            expect: output.into(),
            expected_result: false,
        };
        Self::new(ContentPredicate::Is(pred))
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
    where
        F: 'static + Fn(&str) -> bool,
        M: Into<String>,
    {
        let pred = FnPredicate {
            pred: rc::Rc::new(pred),
            msg: msg.into(),
        };
        Self::new(ContentPredicate::Fn(pred))
    }

    fn new(pred: ContentPredicate) -> Self {
        Self { pred }
    }

    pub(crate) fn verify(&self, got: &[u8]) -> Result<()> {
        self.pred.verify(got)
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
        Self { kind, pred }
    }

    pub(crate) fn verify(&self, got: &process::Output) -> Result<()> {
        let got = self.kind.select(got);
        self.pred
            .verify(got)
            .chain_err(|| ErrorKind::OutputMismatch(self.kind))
    }
}

mod errors {
    error_chain! {
        foreign_links {
            Fmt(::std::fmt::Error);
        }
        errors {
            StrDoesntContain(expected: String, got: String) {
                description("Output was not as expected")
                display("expected to contain {:?}\noutput=```{}```", expected, got)
            }
            BytesDoesntContain(expected: Vec<u8>, got: Vec<u8>) {
                description("Output was not as expected")
                display("expected to contain {:?}\noutput=```{:?}```", expected, got)
            }
            StrContains(expected: String, got: String) {
                description("Output was not as expected")
                display("expected to not contain {:?}\noutput=```{}```", expected, got)
            }
            BytesContains(expected: Vec<u8>, got: Vec<u8>) {
                description("Output was not as expected")
                display("expected to not contain {:?}\noutput=```{:?}```", expected, got)
            }
            StrDoesntMatch(expected: String, got: String, diff: String) {
                description("Output was not as expected")
                display("diff:\n{}", diff)
            }
            BytesDoesntMatch(expected: Vec<u8>, got: Vec<u8>) {
                description("Output was not as expected")
                display("expected=```{:?}```\noutput=```{:?}```", expected, got)
            }
            StrMatches(got: String) {
                description("Output was not as expected")
                display("expected to not match\noutput=```{}```", got)
            }
            BytesMatches(got: Vec<u8>) {
                description("Output was not as expected")
                display("expected to not match\noutput=```{:?}```", got)
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
