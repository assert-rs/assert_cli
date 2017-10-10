use serde_json;
use std::borrow::Cow;

/// Easily construct an `Assert` with a custom command.
///
/// Make sure to include the crate as `#[macro_use] extern crate assert_cli;` if
/// you want to use this macro.
///
/// # Examples
///
/// To test that our very complex cli applications succeeds and prints some
/// text to stdout that contains
///
/// ```plain
/// No errors whatsoever
/// ```
///
/// ...,  you would call it like this:
///
/// ```rust
/// #[macro_use] extern crate assert_cli;
/// # fn main() {
/// assert_cmd!(echo "Launch sequence initiated.\nNo errors whatsoever!\n")
///     .succeeds()
///     .stdout().contains("No errors whatsoever")
///     .unwrap();
/// # }
/// ```
///
/// The macro will try to convert its arguments as strings, but is limited by
/// Rust's default tokenizer, e.g., you always need to quote CLI arguments
/// like `"--verbose"`.
#[macro_export]
macro_rules! assert_cmd {
    ($($x:tt)+) => {{
        $(__assert_single_token_expression!(@CHECK $x);)*

        $crate::Assert::command(
            &[$(
                $crate::flatten_escaped_string(stringify!($x)).as_ref()
            ),*]
        )
    }}
}

/// Deserialize a JSON-encoded `String`.
///
/// # Panics
///
/// If `x` can not be decoded as `String`.
#[doc(hidden)]
fn deserialize_json_string(x: &str) -> String {
    serde_json::from_str(x).expect(&format!("Unable to deserialize `{:?}` as string.", x))
}

/// Deserialize a JSON-encoded `String`.
///
/// # Panics
///
/// If `x` can not be decoded as `String`.
#[doc(hidden)]
pub fn flatten_escaped_string(x: &str) -> Cow<str> {
    if x.starts_with('"') && x.ends_with('"') {
        Cow::Owned(deserialize_json_string(x))
    } else {
        Cow::Borrowed(x)
    }
}

/// Inspect a single token and decide if it is safe to `stringify!`, without loosing
/// information about whitespaces, to address [issue 22].
///
/// [issue 22]: https://github.com/killercup/assert_cli/issues/22
///
/// Call like `__assert_single_token_expression!(@CHECK x)`, where `x` can be any token to check.
///
/// This macro will only accept single tokens, which parse as expressions, e.g.
/// - strings "foo", r#"foo"
/// - idents `foo`, `foo42`
/// - numbers `42`
/// - chars `'a'`
///
/// Delimited token trees `{...}` and the like are rejected. Everything thats not an expression
/// will also be rejected.
#[doc(hidden)]
#[macro_export]
macro_rules! __assert_single_token_expression {
    // deny `{...}`
    (@CHECK {$( $x:tt )*}) => { assert_cmd!(@DENY {$( $x )*}) };
    // deny `(...)`
    (@CHECK ($( $x:tt )*)) => { assert_cmd!(@DENY {$( $x )*}) };
    // deny `[...]`
    (@CHECK [$( $x:tt )*]) => { assert_cmd!(@DENY {$( $x )*}) };
    // only allow tokens that parse as expression
    (@CHECK $x:expr) => { };
    // little helper
    (@DENY) => { };
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn flatten_unquoted() {
        assert_eq!(flatten_escaped_string("hello world"), "hello world");
    }

    #[test]
    fn flatten_quoted() {
        assert_eq!(flatten_escaped_string(r#""hello world""#), "hello world");
    }

    #[test]
    fn flatten_escaped() {
        assert_eq!(
            flatten_escaped_string(r#""hello world \u0042 A""#),
            "hello world B A"
        );
    }
}
