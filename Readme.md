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
assert_cli = "0.1.0"
```

## Example

Here's a trivial example:

```rust
extern crate assert_cli;
assert_cli::assert_cli_output("echo", &["42"], "42").unwrap();
```

Or if you'd rather use the macro:

```rust,ignore
#[macro_use] extern crate assert_cli;
assert_cli!("echo", &["42"] => Success, "42").unwrap();
assert_cli!("black-box", &["--special"] => Error 42, "error no 42\n").unwrap()
```

And here is one that will fail:

```rust,should_panic
extern crate assert_cli;
assert_cli::assert_cli_output("echo", &["42"], "1337").unwrap();
```

this will show a nice, colorful diff in your terminal, like this:

```diff
-1337
+42
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
