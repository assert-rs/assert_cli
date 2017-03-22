# Assert CLI

> **Test CLI Applications** - This crate checks the output of a child process is as expected.

[![Build Status](https://travis-ci.org/killercup/assert_cli.svg)](https://travis-ci.org/killercup/assert_cli) [![Documentation](https://img.shields.io/badge/docs-master-blue.svg)][Documentation]

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
    let test = assert_cmd!(grep amet "Cargo.toml")
        .fails_with(1)
        .execute();
    assert!(test.is_ok());
}
```

If you want to match the program's output _exactly_, you can use
`prints_exactly`:

```rust,should_panic
#[macro_use] extern crate assert_cli;

fn main() {
    assert_cmd!("wc" "README.md")
        .prints_exactly("1337 README.md")
        .unwrap();
}
```

... which has the benefit to show a nice, colorful diff in your terminal,
like this:

```diff
-1337
+92
```

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

[Documentation]: http://killercup.github.io/assert_cli/
