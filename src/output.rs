use self::errors::*;
pub use self::errors::{Error, ErrorKind};
use diff;
use difference::Changeset;
use std::process::Output;

// TODO(dsprenkels) Should implement Debug. Because of the type of `pred`, we will have
// to do this by hand.
pub struct OutputAssertion {
    pub pred: Box<Fn(&str) -> bool>,
    pub kind: OutputKind,
}

impl OutputAssertion {
    pub fn execute(&self, output: &Output, cmd: &[String]) -> super::errors::Result<()> {
        // TODO(dsprenkels) There is currently no error reporting. I still have to think
        // of a solution that nicely handles all the predefined errors (`OutputDoesntContain`
        // etc.) and also handles UserErrors. I may even consider using Any.
        let observed = String::from_utf8_lossy(self.kind.select(output));

        let result = match (self.pred)(&observed) {
            true => Ok(()),
            false => Err(ErrorKind::Unspecified(observed.into()).into()),
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
            Unspecified(got: String) {
                description("Unspecified error")
                display("output=```{}```", got)
            }
        }
    }
}
