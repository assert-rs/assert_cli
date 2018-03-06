use std::fmt;
use std::result;

use failure;

pub type Result<T> = result::Result<T, failure::Error>;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Fail)]
pub enum AssertionKind {
    #[fail(display = "Spawn failed.")] Spawn,
    #[fail(display = "Status mismatch.")] StatusMismatch,
    #[fail(display = "Exit code mismatch.")] ExitCodeMismatch,
    #[fail(display = "Output mismatch.")] OutputMismatch,
}

#[derive(Debug)]
pub struct AssertionError {
    inner: failure::Context<AssertionKind>,
}

impl AssertionError {
    pub fn new(kind: AssertionKind) -> Self {
        Self { inner: kind.into() }
    }

    pub fn kind(&self) -> AssertionKind {
        *self.inner.get_context()
    }
}

impl failure::Fail for AssertionError {
    fn cause(&self) -> Option<&failure::Fail> {
        self.inner.cause()
    }

    fn backtrace(&self) -> Option<&failure::Backtrace> {
        self.inner.backtrace()
    }
}

impl fmt::Display for AssertionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "CLI Assertion Error: {}", self.inner)
    }
}

impl From<AssertionKind> for AssertionError {
    fn from(kind: AssertionKind) -> AssertionError {
        AssertionError {
            inner: failure::Context::new(kind),
        }
    }
}

impl From<failure::Context<AssertionKind>> for AssertionError {
    fn from(inner: failure::Context<AssertionKind>) -> AssertionError {
        AssertionError { inner: inner }
    }
}

#[derive(Debug)]
pub struct KeyValueDisplay<D>
where
    D: fmt::Display + Send + Sync + 'static,
{
    key: &'static str,
    context: D,
}

impl<D> KeyValueDisplay<D>
where
    D: fmt::Display + Send + Sync + 'static,
{
    pub fn new(key: &'static str, context: D) -> Self {
        Self { key, context }
    }

    pub fn key(&self) -> &str {
        self.key
    }

    pub fn context(&self) -> &D {
        &self.context
    }
}

impl<D> fmt::Display for KeyValueDisplay<D>
where
    D: fmt::Display + Send + Sync + 'static,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}={}", self.key, self.context)
    }
}

#[derive(Debug)]
pub struct DebugDisplay<D>
where
    D: fmt::Debug + Send + Sync + 'static,
{
    context: D,
}

impl<D> DebugDisplay<D>
where
    D: fmt::Debug + Send + Sync + 'static,
{
    pub fn new(context: D) -> Self {
        Self { context }
    }

    pub fn context(&self) -> &D {
        &self.context
    }
}

impl<D> fmt::Display for DebugDisplay<D>
where
    D: fmt::Debug + Send + Sync + 'static,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.context)
    }
}

#[derive(Debug)]
pub struct QuotedDisplay<D>
where
    D: fmt::Display + Send + Sync + 'static,
{
    context: D,
}

impl<D> QuotedDisplay<D>
where
    D: fmt::Display + Send + Sync + 'static,
{
    pub fn new(context: D) -> Self {
        Self { context }
    }

    pub fn context(&self) -> &D {
        &self.context
    }
}

impl<D> fmt::Display for QuotedDisplay<D>
where
    D: fmt::Display + Send + Sync + 'static,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "```{}```", self.context)
    }
}
