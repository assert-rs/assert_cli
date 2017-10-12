# Assert CLI

> **Test CLI Applications** - This crate checks the output of a child process is as expected.

[![Build Status](https://travis-ci.org/killercup/assert_cli.svg)](https://travis-ci.org/killercup/assert_cli) [![Documentation](https://img.shields.io/badge/docs-master-blue.svg)][Documentation]

## Install

Just add it to your `Cargo.toml`:

```toml
[dependencies]
assert_cli = "0.5"
```

## Example

Here's a trivial example:

```rust
extern crate assert_cli;

fn main() {
    assert_cli::Assert::command(&["echo", "42"]).stdout().contains("42").unwrap();
}
```

Or if you'd rather use the macro, to save you some writing:

```rust
#[macro_use] extern crate assert_cli;

fn main() {
    assert_cmd!(echo "42").stdout().contains("42").unwrap();
}
```

And here is one that will fail (which also shows `execute` which returns a
`Result` and can be used instead of `unwrap`):

```rust
#[macro_use] extern crate assert_cli;

fn main() {
    let test = assert_cmd!(ls "foo-bar-foo")
        .fails()
        .and()
        .stderr().contains("foo-bar-foo")
        .execute();
    assert!(test.is_ok());
}
```

If you want to match the program's output _exactly_, you can use
`stdout().is`:

```rust,should_panic
#[macro_use] extern crate assert_cli;

fn main() {
    assert_cmd!(wc "README.md")
        .stdout().is("1337 README.md")
        .unwrap();
}
```

... which has the benefit to show a nice, colorful diff in your terminal,
like this:

```diff
-1337
+92
```

**Tip**: Enclose arguments in the `assert_cmd!` macro in quotes `"`,
         if there are special characters, which the macro doesn't accept, e.g.
         `assert_cmd!(cat "foo.txt")`.

Assert Cli use [Environment][Environment] underneath to deal with environment variables.

More detailed information is available in the [documentation]. :-)

## License

Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the Apache-2.0
license, shall be dual licensed as above, without any additional terms or
conditions.

[Documentation]: https://docs.rs/assert_cli
[Environment]: https://github.com/Freyskeyd/environment
