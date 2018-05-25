extern crate assert_cli;

#[test]
fn main_binary() {
    assert_cli::Assert::main_binary()
        .with_env(assert_cli::Environment::inherit().insert("stdout", "42"))
        .stdout()
        .is("42")
        .stderr()
        .is("")
        .unwrap();
}

#[test]
fn cargo_binary() {
    assert_cli::Assert::cargo_binary("bin_fixture")
        .with_env(assert_cli::Environment::inherit().insert("stdout", "42"))
        .stdout()
        .is("42")
        .stderr()
        .is("")
        .unwrap();
}

#[test]
fn cargo_example() {
    assert_cli::Assert::example("example_fixture")
        .unwrap()
        .with_env(assert_cli::Environment::inherit().insert("stdout", "42"))
        .stdout()
        .is("42")
        .stderr()
        .is("")
        .unwrap();
}
