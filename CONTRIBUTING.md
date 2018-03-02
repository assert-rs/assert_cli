Thanks for wanting to contribute!

Feel free to create issues or make pull requests, we'll try to quickly review them.

If you need to reach out to us, find a relevant [issue][Issues] or open a new one.

# Pull Requests

## Project Ideas

If you're looking for things to do check out the [open issues][Issues].
Or take a grep through [all TODO comments][TODO] in the code and feel free to help us out there!

## Best Practices

We appreciate your help as-is. We'd love to help you through the process for contributing. We have some suggestions to help make things go more smoothly.

Before spending too much time on a PR, consider opening an issue so we can make sure we're aligned on how the problem should be solved.

# Releasing

When we're ready to release, a project owner should do the following
- Determine what the next version is, according to semver
- Update the version in `Cargo.toml` and in the `README.md` and commit
- Run `git tag v<X>.<Y>.<Z>`
- Push all of this to `master`
- Create a github release
  - Identify what fixes, features, and breaking changes are in the release.
- Run `cargo publish` (run `cargo login` first if needed)

[Issues]: https://github.com/assert-rs/assert_cli/issues
[TODO]: https://github.com/assert-rs/assert_cli/search?q=TODO
