//! # Test CLI Applications
//!
//! This crate's goal is to provide you some very easy tools to test your CLI
//! applications. It can currently execute child processes and validate their
//! exit status as well as stdout and stderr output against your assertions.
//!
//! Include the crate like
//!
//! ```rust
//! #[macro_use] // <-- import the convenience macro (optional)
//! extern crate assert_cli;
//! # fn main() { }
//! ```
//!
//! ## Basic Examples
//!
//! Here's a trivial example:
//!
//! ```rust
//! assert_cli::Assert::command(&["echo", "42"])
//!     .prints("42")
//!     .unwrap();
//! ```
//!
//! And here is one that will fail:
//!
//! ```rust,should_panic
//! assert_cli::Assert::command(&["echo", "42"])
//!     .prints_exactly("1337")
//!     .unwrap();
//! ```
//!
//! this will show a nice, colorful diff in your terminal, like this:
//!
//! ```diff
//! -1337
//! +42
//! ```
//!
//! ## `assert_cmd!` Macro
//!
//! Alternatively, you can use the `assert_cmd!` macro to construct the command more conveniently,
//! but please carefully read the limitations below, or this may seriously go wrong.
//!
//! ```rust
//! # #[macro_use] extern crate assert_cli;
//! # fn main() {
//! assert_cmd!(echo "42").prints("42").unwrap();
//! # }
//! ```
//!
//! **Tips**
//!
//! - Don't forget to import the crate with `#[macro_use]`. ;-)
//! - Enclose arguments in the `assert_cmd!` macro in quotes `"`,
//!   if there are special characters, which the macro doesn't accept, e.g.
//!   `assert_cmd!(cat "foo.txt")`.
//!
//! ## Exit Status
//!
//! All assertion default to checking that the command exited with success.
//!
//! However, when you expect a command to fail, you can express it like this:
//!
//! ```rust
//! # #[macro_use] extern crate assert_cli;
//! # fn main() {
//! assert_cmd!(cat "non-existing-file")
//!     .fails()
//!     .and()
//!     .prints_error("non-existing-file")
//!     .unwrap();
//! # }
//! ```
//!
//! Some notes on this:
//!
//! - Use `fails_with` to assert a specific exit status.
//! - There is also a `succeeds` method, but this is already the implicit default
//!   and can usually be omitted.
//! - We can inspect the output of **stderr** with `prints_error` and `prints_error_exactly`.
//! - The `and` method has no effect, other than to make everything more readable.
//!   Feel free to use it. :-)
//!
//! ## Assert CLI Crates
//!
//! If you are testing a Rust binary crate, you can start with
//! `Assert::main_binary()` to use `cargo run` as command. Or, if you want to
//! run a specific binary (if you have more than one), use
//! `Assert::cargo_binary`.
//!
//! ## Don't Panic!
//!
//! If you don't want it to panic when the assertions are not met, simply call
//! `.execute` instead of `.unwrap` to get a `Result`:
//!
//! ```rust
//! # #[macro_use] extern crate assert_cli;
//! # fn main() {
//! let x = assert_cmd!(echo "1337").prints_exactly("42").execute();
//! assert!(x.is_err());
//! # }
//! ```

#![deny(missing_docs)]

extern crate difference;
#[macro_use] extern crate error_chain;
extern crate serde_json;

mod errors;

#[macro_use] mod macros;
pub use macros::flatten_escaped_string;

mod output;

mod diff;

mod assert;
pub use assert::Assert;
