use output;
use std::ffi::OsString;

const ERROR_PREFIX: &'static str = "CLI assertion failed";

fn format_cmd(cmd: &[OsString]) -> String {
    let result: Vec<String> = cmd.iter()
        .map(|s| s.to_string_lossy().into_owned())
        .collect();
    result.join(" ")
}

error_chain! {
    links {
        Output(output::Error, output::ErrorKind);
    }
    foreign_links {
        Io(::std::io::Error);
        Fmt(::std::fmt::Error);
    }
    errors {
        SpawnFailed(cmd: Vec<OsString>) {
            description("Spawn failed")
            display(
                "{}: (command `{}` failed to run)",
                ERROR_PREFIX,
                format_cmd(cmd),
            )
        }
        AssertionFailed(cmd: Vec<OsString>) {
            description("Assertion failed")
            display(
                "{}: (command `{}` failed)",
                ERROR_PREFIX,
                format_cmd(cmd),
            )
        }
        StatusMismatch(expected: bool, out: String, err: String) {
            description("Wrong status")
            display(
                "Expected to {}\nstatus={}\nstdout=```{}```\nstderr=```{}```",
                expected = if *expected { "succeed" } else { "fail" },
                got = if *expected { "failed" } else { "succeeded" },
                out = out,
                err = err,
            )
        }
        ExitCodeMismatch(
            expected: Option<i32>,
            got: Option<i32>,
            out: String,
            err: String
        ) {
            description("Wrong exit code")
            display(
                "Expected exit code to be `{expected:?}`)\n\
                exit code=`{code:?}`\n\
                stdout=```{stdout}```\n\
                stderr=```{stderr}```",
                expected=expected,
                code=got,
                stdout=out,
                stderr=err,
            )
        }
    }
}
