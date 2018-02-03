extern crate assert_cli;

#[test]
fn main_binary() {
    assert_cli::Assert::main_binary()
        .with_env(assert_cli::Environment::inherit().insert("stdout", "42"))
        .stdout(assert_cli::Output::is("42"))
        .stderr(assert_cli::Output::is(""))
        .unwrap();
}

#[test]
fn cargo_binary() {
    assert_cli::Assert::cargo_binary("assert_fixture")
        .with_env(assert_cli::Environment::inherit().insert("stdout", "42"))
        .stdout(assert_cli::Output::is("42"))
        .stderr(assert_cli::Output::is(""))
        .unwrap();
}
