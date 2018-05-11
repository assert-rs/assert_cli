# Assert CLI

> **Test CLI Applications** - This crate checks the output of a child process is as expected.

[![Build Status](https://travis-ci.org/assert-rs/assert_cli.svg)][Travis]
[![Documentation](https://img.shields.io/badge/docs-master-blue.svg)][Documentation]
![License](https://img.shields.io/crates/l/assert_cli.svg)
[![crates.io](https://img.shields.io/crates/v/assert_cli.svg)][Crates.io]

## Install

For your tests, add it to your `Cargo.toml`:

```toml
[dev-dependencies]
assert_cli = "0.6"
```

## Example

Here's a trivial example:

```rust,ignore
extern crate assert_cli;

fn main() {
    assert_cli::Assert::main_binary().unwrap();
}
```

And here is one that will fail (and demonstrates running arbitrary commands):

```rust
extern crate assert_cli;

fn main() {
    assert_cli::Assert::command(&["ls", "foo-bar-foo"])
        .fails()
        .and()
        .stderr().contains("foo-bar-foo")
        .unwrap();
}
```

If you want to match the program's output _exactly_, you can use
`stdout().is` (and shows the macro form of `command`):

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

## Relevant crates

Other crates that might be useful in testing command line programs.
* [dir-diff][dir-diff] for testing file side-effects.
* [tempfile][tempfile] for scratchpad directories.
* [duct][duct] for orchestrating multiple processes.

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

[Travis]: https://travis-ci.org/assert-rs/assert_cli
[Crates.io]: https://crates.io/crates/assert_cli
[Documentation]: https://docs.rs/assert_cli
[Environment]: https://github.com/Freyskeyd/environment
[dir-diff]: https://crates.io/crates/dir-diff
[tempfile]: https://crates.io/crates/tempfile
[duct]: https://crates.io/crates/duct
