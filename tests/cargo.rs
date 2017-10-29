extern crate assert_cli;
use assert_cli::prelude::*;

#[test]
fn main_binary() {
    Assert::main_binary()
        .with_env(Environment::inherit().insert("stdout", "42"))
        .stdout(is("42"))
        .stderr(is(""))
        .unwrap();
}

#[test]
fn cargo_binary() {
    Assert::cargo_binary("assert_fixture")
        .with_env(Environment::inherit().insert("stdout", "42"))
        .stdout(is("42"))
        .stderr(is(""))
        .unwrap();
}
