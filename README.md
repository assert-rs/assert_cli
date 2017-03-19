# Assert CLI

> Test CLI Applications.

Currently, this crate only includes basic functionality to check the output of a child process
is as expected.

[![Build Status](https://travis-ci.org/killercup/assert_cli.svg)](https://travis-ci.org/killercup/assert_cli) [![Coverage Status](https://coveralls.io/repos/killercup/assert_cli/badge.svg?branch=master&service=github)](https://coveralls.io/github/killercup/assert_cli?branch=master)

**[Documentation](http://killercup.github.io/assert_cli/)**

## Install

Just add it to your `Cargo.toml`:

```toml
[dependencies]
assert_cli = "0.4"
```

## Example

Here's a trivial example:

```rust
extern crate assert_cli;

fn main() {
    assert_cli::Assert::command(&["echo", "42"]).prints("42").unwrap();
}
```

Or if you'd rather use the macro:

```rust
#[macro_use] extern crate assert_cli;

fn main() {
    assert_cmd!(echo 42).succeeds().and().prints("42").unwrap();
}
```

And here is one that will fail (which also shows `execute` which returns a
`Result` and can be used instead of `unwrap`):

```rust
#[macro_use] extern crate assert_cli;

fn main() {
    let test = assert_cmd!(grep amet Cargo.toml)
        .fails_with(1)
        .execute();
    assert!(test.is_err());
}
```

If you want to check for the program's output, you can use `print` or
`print_exactly`:

```rust,should_panic="Assert CLI failure"
#[macro_use] extern crate assert_cli;

fn main() {
    assert_cmd!("wc" "README.md")
        .prints_exactly("1337 README.md")
        .unwrap();
}
```

this will show a nice, colorful diff in your terminal, like this:

```diff
-1337
+92
```

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
