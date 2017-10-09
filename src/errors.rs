const ERROR_PREFIX: &'static str = "CLI assertion failed";

error_chain! {
    foreign_links {
        Io(::std::io::Error);
        Fmt(::std::fmt::Error);
    }
    errors {
        StatusMismatch(cmd: Vec<String>, expected: bool, out: String, err: String) {
            description("Wrong status")
            display(
                "{}: (command `{}` expected to {})\nstatus={}\nstdout=```{}```\nstderr=```{}```",
                ERROR_PREFIX,
                cmd.join(" "),
                expected = if *expected { "succeed" } else { "fail" },
                got = if *expected { "failed" } else { "succeeded" },
                out = out,
                err = err,
            )
        }
        ExitCodeMismatch(
            cmd: Vec<String>,
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
                cmd=cmd.join(" "),
                expected=expected,
                code=got,
                stdout=out,
                stderr=err,
            )
        }
        OutputMismatch(cmd: Vec<String>, output_err: ::output::Error, kind: ::output::OutputKind) {
            description("Output was not as expected")
            display(
                "{}: `{}` {:?} mismatch: {}",
                ERROR_PREFIX, cmd.join(" "), kind, output_err,
            )
        }

    }
}
