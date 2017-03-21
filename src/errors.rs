static ERROR_PREFIX: &'static str = "CLI assertion failed";

error_chain! {
    foreign_links {
        Io(::std::io::Error);
        Fmt(::std::fmt::Error);
    }
    errors {
        StatusMismatch(cmd: Vec<String>, expected: bool) {
            description("Wrong status")
            display(
                "{}: `(command `{}` expected to {})` (command {})",
                ERROR_PREFIX,
                cmd.join(" "),
                expected = if *expected { "succeed" } else { "fail" },
                got = if *expected { "failed" } else { "succeeded" },
            )
        }
        ExitCodeMismatch(cmd: Vec<String>, expected: Option<i32>, got: Option<i32>) {
            description("Wrong exit code")
            display(
                "{}: `(exit code of `{}` expected to be `{:?}`)` (exit code was: `{:?}`)",
                ERROR_PREFIX,
                cmd.join(" "),
                expected,
                got,
            )
        }
        OutputMismatch(output_name: String, cmd: Vec<String>, expected: String, got: String) {
            description("Output was not as expected")
            display(
                "{}: `({} of `{}` expected to contain `{:?}`)` (output was: `{:?}`)",
                ERROR_PREFIX,
                output_name,
                cmd.join(" "),
                expected,
                got,
            )
        }
        ExactOutputMismatch(output_name: String, cmd: Vec<String>, diff: String) {
            description("Output was not as expected")
            display(
                "{}: `({} of `{}` was not as expected)`\n{}\n",
                ERROR_PREFIX,
                output_name,
                cmd.join(" "),
                diff.trim()
            )
        }
    }
}
