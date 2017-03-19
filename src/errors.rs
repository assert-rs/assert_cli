error_chain! {
    foreign_links {
        Io(::std::io::Error);
        Fmt(::std::fmt::Error);
    }
    errors {
        StatusMismatch(cmd: Vec<String>, expected: bool) {
            description("Wrong status")
            display(
                "Command {:?} {got} but expected it to {expected}",
                cmd.join(" "),
                got = if *expected { "failed" } else { "succeed" },
                expected = if *expected { "succeed" } else { "failed" },
            )
        }
        ExitCodeMismatch(cmd: Vec<String>, expected: Option<i32>, got: Option<i32>) {
            description("Wrong exit code")
            display(
                "Command {:?} exited with code {:?} but expected it to be {:?}",
                cmd.join(" "), got, expected,
            )
        }
        OutputMismatch(expected: String, got: String) {
            description("Output was not as expected")
            display(
                "Expected output to contain\n{}\nbut could not find it in\n{}",
                expected,
                got,
            )
        }
        ExactOutputMismatch(diff: String) {
            description("Output was not as expected")
            display("{}", diff)
        }
        ErrorOutputMismatch(expected: String, got: String) {
            description("Stderr output was not as expected")
            display(
                "Expected stderr output to contain\n{}\nbut could not find it in\n{}",
                expected,
                got,
            )
        }
        ExactErrorOutputMismatch(diff: String) {
            description("Stderr output was not as expected")
            display("{}", diff)
        }
    }
}
