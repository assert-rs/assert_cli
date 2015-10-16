use std::error::Error;
use std::fmt;

use diff::render as render_diff;
use std::process::Output;
use difference::Difference;

pub enum CliError {
    WrongExitCode(Output),
    OutputMissmatch(Vec<Difference>),
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl fmt::Debug for CliError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            CliError::WrongExitCode(ref output) => write!(f,
                       "Unexpected error code {code:?} with this stderr:\n{stderr}",
                       code = output.status.code(),
                       stderr = String::from_utf8_lossy(&output.stderr)),
            CliError::OutputMissmatch(ref diff) => {
                let diff = match render_diff(&diff) {
                    Ok(diff) => diff,
                    Err(_) => return Err(fmt::Error),
                };
                write!(f, "Output was not as expected:\n{}", diff)
            }
        }
    }
}

impl Error for CliError {
    fn description(&self) -> &str {
        match *self {
            CliError::WrongExitCode(_) => "Command return unexpected error code.",
            CliError::OutputMissmatch(_) => "Command output was not as expected.",
        }
    }
}
