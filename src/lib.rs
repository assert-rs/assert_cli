extern crate difference;
#[macro_use] extern crate error_chain;

use std::process::Command;

use difference::Changeset;

mod errors;
use errors::*;

mod diff;

/// Assertions for a specific command
#[derive(Debug)]
pub struct Assert {
    cmd: Vec<String>,
    expect_success: bool,
    expect_exit_code: Option<i32>,
    expect_output: Option<String>,
    fuzzy: bool,
}

impl std::default::Default for Assert {
    /// Construct an assert using `cargo run --` as command.
    fn default() -> Self {
        Assert {
            cmd: vec!["cargo", "run", "--"].into_iter().map(String::from).collect(),
            expect_success: true,
            expect_exit_code: None,
            expect_output: None,
            fuzzy: false,
        }
    }
}

impl Assert {
    /// Use the crate's main binary as command
    pub fn main_binary() -> Self {
        Assert::default()
    }

    /// Use custom command
    pub fn command(cmd: &[&str]) -> Self {
        Assert {
            cmd: cmd.into_iter().cloned().map(String::from).collect(),
            ..Self::default()
        }
    }

    /// Small helper to make chains more readable
    pub fn and(self) -> Self {
        self
    }

    /// Expect the command to be executed successfully
    pub fn succeeds(mut self) -> Self {
        self.expect_success = true;
        self
    }

    /// Expect the command to fail
    pub fn fails(mut self) -> Self {
        self.expect_success = false;
        self
    }

    /// Expect the command to fail and return a specific error code
    pub fn fails_with(mut self, expect_exit_code: i32) -> Self {
        self.expect_success = false;
        self.expect_exit_code = Some(expect_exit_code);
        self
    }

    /// Expect the command's output to contain `output`
    pub fn prints<O: Into<String>>(mut self, output: O) -> Self {
        self.expect_output = Some(output.into());
        self.fuzzy = true;
        self
    }

    /// Expect the command to output exactly this `output`
    pub fn prints_exactly<O: Into<String>>(mut self, output: O) -> Self {
        self.expect_output = Some(output.into());
        self.fuzzy = false;
        self
    }

    /// Execute the command and check the assertions
    pub fn execute(self) -> Result<()> {
        let ref cmd = self.cmd[0];
        let args: Vec<_> = self.cmd.iter().skip(1).collect();
        let mut command = Command::new(cmd);
        let command = command.args(&args);
        let output = command.output()?;

        if self.expect_success != output.status.success() {
            bail!("Command {:?} {} but expected it to {}",
                self.cmd,
                if output.status.success() { "succeeded" } else { "failed" },
                if self.expect_success { "succeed" } else { "fail" },
            );
        }

        if self.expect_exit_code.is_some() &&
            self.expect_exit_code != output.status.code() {
            bail!("Command {:?} exited with code {:?} but expected it to be {:?}",
                self.cmd,
                output.status.code(),
                self.expect_exit_code,
            );
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        match (self.expect_output, self.fuzzy) {
            (Some(ref expected_output), true) if !stdout.contains(expected_output) => {
                bail!("Expected output to contain\n{}\n\
                    but could not find it in\n{}",
                    expected_output,
                    stdout,
                )
            },
            (Some(ref expected_output), false) => {
                let differences = Changeset::new(expected_output.trim(), &stdout.trim(), "\n");
                if differences.distance > 0 {
                    bail!("Output was not as expected:\n{}",
                        diff::render(differences)?,
                    );
                }
            },
            _ => {},
        }

        Ok(())
    }
}
