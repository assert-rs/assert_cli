# Assert CLI

> Test CLI Applications.

Currently, this crate only includes basic functionality to check the output of a child process
is as expected.

[![Build Status](https://travis-ci.org/killercup/assert_cli.svg)](https://travis-ci.org/killercup/assert_cli) [![Coverage Status](https://coveralls.io/repos/killercup/assert_cli/badge.svg?branch=master&service=github)](https://coveralls.io/github/killercup/assert_cli?branch=master)

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

MIT
