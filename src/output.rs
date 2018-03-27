use std::fmt;
use std::process;
use std::rc;

use difference::Changeset;
use failure;

use diff;
use errors::*;

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
    pub fn verify(&self, got: &[u8]) -> Result<(), failure::Error> {
        match self.expect {
            Content::Str(ref expect) => {
                self.verify_str(expect, String::from_utf8_lossy(got).as_ref())
            }
            Content::Bytes(ref expect) => self.verify_bytes(expect, got),
        }
    }

    fn verify_bytes(&self, expect: &[u8], got: &[u8]) -> Result<(), failure::Error> {
        let result = expect == got;

        if result != self.expected_result {
            if self.expected_result {
                bail!(BytesDoesntMatch::new(expect.to_owned(), got.to_owned()));
            } else {
                bail!(BytesMatches::new(got.to_owned()));
            }
        }

        Ok(())
    }

    fn verify_str(&self, expect: &str, got: &str) -> Result<(), failure::Error> {
        let differences = Changeset::new(expect.trim(), got.trim(), "\n");
        let result = differences.distance == 0;

        if result != self.expected_result {
            if self.expected_result {
                let nice_diff = diff::render(&differences)?;
                bail!(StrDoesntMatch::new(
                    expect.to_owned(),
                    got.to_owned(),
                    nice_diff
                ));
            } else {
                bail!(StrMatches::new(got.to_owned()));
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
    pub fn verify(&self, got: &[u8]) -> Result<(), failure::Error> {
        match self.expect {
            Content::Str(ref expect) => {
                self.verify_str(expect, String::from_utf8_lossy(got).as_ref())
            }
            Content::Bytes(ref expect) => self.verify_bytes(expect, got),
        }
    }

    pub fn verify_bytes(&self, expect: &[u8], got: &[u8]) -> Result<(), failure::Error> {
        let result = find_subsequence(got, expect).is_some();
        if result != self.expected_result {
            if self.expected_result {
                bail!(BytesDoesntContain::new(expect.to_owned(), got.to_owned()));
            } else {
                bail!(BytesContains::new(expect.to_owned(), got.to_owned()));
            }
        }

        Ok(())
    }

    pub fn verify_str(&self, expect: &str, got: &str) -> Result<(), failure::Error> {
        let result = got.contains(expect);
        if result != self.expected_result {
            if self.expected_result {
                bail!(StrDoesntContain::new(expect.to_owned(), got.to_owned()));
            } else {
                bail!(StrContains::new(expect.to_owned(), got.to_owned()));
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
    pub fn verify(&self, got: &[u8]) -> Result<(), failure::Error> {
        let got = String::from_utf8_lossy(got);
        let pred = &self.pred;
        if !pred(&got) {
            bail!(PredicateFailed::new(self.msg.clone(), got.into_owned()));
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
    pub fn verify(&self, got: &[u8]) -> Result<(), failure::Error> {
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

    pub(crate) fn verify(&self, got: &[u8]) -> Result<(), failure::Error> {
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

impl fmt::Display for OutputKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            OutputKind::StdOut => write!(f, "stdout"),
            OutputKind::StdErr => write!(f, "stderr"),
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

    pub(crate) fn verify(&self, got: &process::Output) -> Result<(), OutputError> {
        let got = self.kind.select(got);
        self.pred.verify(got).chain(OutputError::new(self.kind))?;
        Ok(())
    }
}

#[derive(Fail, Debug)]
pub struct StrDoesntContain {
    needle: String,
    haystack: String,
}

impl StrDoesntContain {
    pub fn new(needle: String, haystack: String) -> Self {
        Self { needle, haystack }
    }
}

impl fmt::Display for StrDoesntContain {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Expected to contain needle.\n")?;
        write!(f, "needle=```{}```\n", self.needle)?;
        write!(f, "haystack=```{}```", self.haystack)
    }
}

#[derive(Fail, Debug)]
pub struct BytesDoesntContain {
    needle: Vec<u8>,
    haystack: Vec<u8>,
}

impl BytesDoesntContain {
    pub fn new(needle: Vec<u8>, haystack: Vec<u8>) -> Self {
        Self { needle, haystack }
    }
}

impl fmt::Display for BytesDoesntContain {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Expected to contain needle.\n")?;
        write!(f, "needle=```{:?}```\n", self.needle)?;
        write!(f, "haystack=```{:?}```", self.haystack)
    }
}

#[derive(Fail, Debug)]
pub struct StrContains {
    needle: String,
    haystack: String,
}

impl StrContains {
    pub fn new(needle: String, haystack: String) -> Self {
        Self { needle, haystack }
    }
}

impl fmt::Display for StrContains {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Expected to not contain needle.\n")?;
        write!(f, "needle=```{}```\n", self.needle)?;
        write!(f, "haystack=```{}```", self.haystack)
    }
}

#[derive(Fail, Debug)]
pub struct BytesContains {
    needle: Vec<u8>,
    haystack: Vec<u8>,
}

impl BytesContains {
    pub fn new(needle: Vec<u8>, haystack: Vec<u8>) -> Self {
        Self { needle, haystack }
    }
}

impl fmt::Display for BytesContains {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Expected to not contain needle.\n")?;
        write!(f, "needle=```{:?}```\n", self.needle)?;
        write!(f, "haystack=```{:?}```", self.haystack)
    }
}

#[derive(Fail, Debug)]
pub struct StrDoesntMatch {
    expected: String,
    got: String,
    diff: String,
}

impl StrDoesntMatch {
    pub fn new(expected: String, got: String, diff: String) -> Self {
        Self {
            expected,
            got,
            diff,
        }
    }
}

impl fmt::Display for StrDoesntMatch {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Didn't match.\n")?;
        write!(f, "diff=\n``{}```", self.diff)
    }
}

#[derive(Fail, Debug)]
pub struct BytesDoesntMatch {
    expected: Vec<u8>,
    got: Vec<u8>,
}

impl BytesDoesntMatch {
    pub fn new(expected: Vec<u8>, got: Vec<u8>) -> Self {
        Self { expected, got }
    }
}

impl fmt::Display for BytesDoesntMatch {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Didn't match.\n")?;
        write!(f, "expected=```{:?}```\n", self.expected)?;
        write!(f, "got=```{:?}```", self.got)
    }
}

#[derive(Fail, Debug)]
pub struct StrMatches {
    output: String,
}

impl StrMatches {
    pub fn new(output: String) -> Self {
        Self { output }
    }
}

impl fmt::Display for StrMatches {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Expected to not match.")?;
        write!(f, "output=```{}```", self.output)
    }
}

#[derive(Fail, Debug)]
pub struct BytesMatches {
    output: Vec<u8>,
}

impl BytesMatches {
    pub fn new(output: Vec<u8>) -> Self {
        Self { output }
    }
}

impl fmt::Display for BytesMatches {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Expected to not match.")?;
        write!(f, "output=```{:?}```", self.output)
    }
}

#[derive(Fail, Debug)]
pub struct PredicateFailed {
    msg: String,
    got: String,
}

impl PredicateFailed {
    pub fn new(msg: String, got: String) -> Self {
        Self { msg, got }
    }
}

impl fmt::Display for PredicateFailed {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Predicate failed: {}", self.msg)?;
        write!(f, "got=```{}```\n", self.got)
    }
}

#[derive(Debug)]
pub struct OutputError {
    kind: OutputKind,
    cause: Option<failure::Error>,
}

impl OutputError {
    pub fn new(kind: OutputKind) -> Self {
        Self { kind, cause: None }
    }
}

impl failure::Fail for OutputError {
    fn cause(&self) -> Option<&failure::Fail> {
        self.cause.as_ref().map(failure::Error::cause)
    }

    fn backtrace(&self) -> Option<&failure::Backtrace> {
        None
    }
}

impl ChainFail for OutputError {
    fn chain<E>(mut self, error: E) -> Self
    where
        E: Into<failure::Error>,
    {
        self.cause = Some(error.into());
        self
    }
}

impl fmt::Display for OutputError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "Unexpected {}", self.kind)
    }
}

impl<T> ResultChainExt<T> for Result<T, OutputError> {
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
