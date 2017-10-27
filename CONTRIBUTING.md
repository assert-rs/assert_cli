Thanks for wanting to contribute!

Feel free to create issues or make pull requests, we'll try to quickly review them.

If you need to reach out to us, find a relevant [issue](https://github.com/killercup/assert_cli/issues) or open a new one.

# Issues

Some helpful pieces of information when reporting issues
* assert_cli version
* rust version
* OS and version

# Pull Requests

## Project Ideas

If you're looking for things to do check out the [open issues](https://github.com/killercup`/assert_cli/issues).
Or take a grep through [all TODO comments](https://github.com/killercup/assert_cli/search?q=TODO) in the code and feel free to help us out there!

## Best Practices

We appreciate your help as-is.  We'd love to help you through the process for contributing.  We have some suggestions to help make things go more smoothly.

Before spending too much time on a PR, consider opening an issue so we can make sure we're aligned on how the problem should be solved.

🌈 **Here's a checklist for the perfect pull request:**
- [ ] Make sure existing tests still work by running `cargo test` locally.
- [ ] Add new tests for any new feature or regression tests for bugfixes.
- [ ] Install [Clippy](https://github.com/Manishearth/rust-clippy) and run `rustup run nightly cargo clippy` to catch common mistakes (will be checked by Travis)

# Releasing

When we're ready to release, a project owner should do the following
- Determine what the next version is, according to semver
- Update the version in `Cargo.toml` and in the `README.md` and commit
- Run `git tag v<X>.<Y>.<Z>`
- Push all of this to `master`
- Create a github release
  - Identify what fixes, features, and breaking changes are in the release.
- Run `cargo publish` (run `cargo login` first if needed)
