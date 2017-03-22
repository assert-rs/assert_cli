use std::fmt;
use std::process::Output;

use difference::Changeset;

use errors::*;
use diff;

#[derive(Debug, Clone)]
pub struct OutputAssertion<T> {
    pub expect: String,
    pub fuzzy: bool,
    pub kind: T,
}

impl<T: OutputType> OutputAssertion<T> {
    fn matches_fuzzy(&self, got: &str) -> Result<()> {
        if !got.contains(&self.expect) {
            bail!(ErrorKind::OutputMismatch(
                self.kind.to_string(),
                vec!["Foo".to_string()],
                self.expect.clone(),
                got.into(),
            ));
        }

        Ok(())
    }

    fn matches_exact(&self, got: &str) -> Result<()> {
        let differences = Changeset::new(self.expect.trim(), got.trim(), "\n");

        if differences.distance > 0 {
            let nice_diff = diff::render(&differences)?;
            bail!(ErrorKind::ExactOutputMismatch(
                self.kind.to_string(),
                vec!["Foo".to_string()],
                nice_diff
            ));
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
