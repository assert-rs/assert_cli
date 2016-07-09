#[macro_use]
extern crate assert_cli;

fn test_helper(exit_code: i32, output: &str) -> Vec<String> {
    vec!["-c".into(),
         format!(r#"function test_helper() {{ >&2 echo $1; return {}; }}; test_helper "{}""#,
                 exit_code,
                 output)]
}

#[test]
fn assert_success() {
    assert_cli!("true", &[""] => Success).unwrap();
    assert_cli!("echo", &["42"] => Success, "42").unwrap();
    assert!(assert_cli!("echo", &["1"] => Success, "42").is_err());
}

#[test]
fn assert_failure() {
    assert_cli!("bash", &test_helper(66, "sorry, my bad") => Error).unwrap();
    assert_cli!("bash", &test_helper(66, "sorry, my bad") => Error, "sorry, my bad").unwrap();
    assert_cli!("bash", &test_helper(42, "error no 42") => Error 42).unwrap();
    assert_cli!("bash", &test_helper(42, "error no 42") => Error 42, "error no 42").unwrap();

    assert!(assert_cli!("echo", &["good"] => Error, "").is_err());
    assert!(assert_cli!("echo", &["good"] => Error 11, "").is_err());
}
