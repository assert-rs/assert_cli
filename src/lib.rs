//! # Test CLI Applications
//!
//! Currently, this crate only includes basic functionality to check the output of a child process
//! is as expected.
//!
//! ## Example
//!
//! Here's a trivial example:
//!
//! ```rust
//! extern crate assert_cli;
//! assert_cli::assert_cli_output("echo", &["42"], "42").unwrap();
//! ```
//!
//! And here is one that will fail:
//!
//! ```rust,should_panic
//! extern crate assert_cli;
//! assert_cli::assert_cli_output("echo", &["42"], "1337").unwrap();
//! ```
//!
//! this will show a nice, colorful diff in your terminal, like this:
//!
//! ```diff
//! -1337
//! +42
//! ```

#![cfg_attr(feature = "dev", feature(plugin))]
#![cfg_attr(feature = "dev", plugin(clippy))]

#![deny(missing_docs)]

extern crate ansi_term;
extern crate difference;

use std::process::{Command, Output};
use std::error::Error;
use std::ffi::OsStr;

mod cli_error;
mod diff;

use cli_error::CliError;

/// Assert a CLI call returns the expected output.
///
/// To test that
///
/// ```sh
/// ls -n1 src/
/// ```
///
/// returns
///
/// ```plain
/// cli_error.rs
/// diff.rs
/// lib.rs
/// ```
///
/// you would call it like this:
///
/// ```rust,no_run
/// # extern crate assert_cli;
/// assert_cli::assert_cli_output("ls", &["-n1", "src/"], "cli_error.rs\ndiff.rs\nlib.rs");
/// ```
pub fn assert_cli_output<S>(cmd: &str, args: &[S], expected_output: &str) -> Result<(), Box<Error>>
    where S: AsRef<OsStr>
{
    let call: Result<Output, Box<Error>> = Command::new(cmd)
                                               .args(args)
                                               .output()
                                               .map_err(From::from);

    call.and_then(|output| {
            if !output.status.success() {
                return Err(From::from(CliError::NoSuccess(output)));
            }

            let stdout = String::from_utf8_lossy(&output.stdout);
            let (distance, changes) = difference::diff(expected_output.trim(),
                                                       &stdout.trim(),
                                                       "\n");
            if distance > 0 {
                return Err(From::from(CliError::OutputMissmatch(changes)));
            }

            Ok(())
        })
        .map_err(From::from)
}
