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
        StatusMismatch(cmd: Vec<OsString>, expected: bool, out: String, err: String) {
            description("Wrong status")
            display(
                "{}: (command `{}` expected to {})\nstatus={}\nstdout=```{}```\nstderr=```{}```",
                ERROR_PREFIX,
                format_cmd(cmd),
                expected = if *expected { "succeed" } else { "fail" },
                got = if *expected { "failed" } else { "succeeded" },
                out = out,
                err = err,
            )
        }
        ExitCodeMismatch(
            cmd: Vec<OsString>,
            expected: Option<i32>,
            got: Option<i32>,
            out: String,
            err: String
        ) {
            description("Wrong exit code")
            display(
                "{prefix}: (exit code of `{cmd}` expected to be `{expected:?}`)\n\
                exit code=`{code:?}`\n\
                stdout=```{stdout}```\n\
                stderr=```{stderr}```",
                prefix=ERROR_PREFIX,
                cmd=format_cmd(cmd),
                expected=expected,
                code=got,
                stdout=out,
                stderr=err,
            )
        }
        OutputMismatch(cmd: Vec<OsString>, output_err: output::Error, kind: output::OutputKind) {
            description("Output was not as expected")
            display(
                "{}: `{}` {:?} mismatch: {}",
                ERROR_PREFIX, format_cmd(cmd), kind, output_err,
            )
        }

    }
}
