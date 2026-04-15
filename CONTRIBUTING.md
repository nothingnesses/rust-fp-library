# Contributing

We welcome contributions! Please feel free to look through [Issues](https://github.com/nothingnesses/rust-fp-library/issues) and submit [Pull Requests](https://github.com/nothingnesses/rust-fp-library/compare).

## Important

- When creating a PR, please ensure a corresponding issue has first been opened. The issue should detail the motivation for the changes, expected behaviours and, if the issue relates to bugs, it should include a [minimal reproducible example](https://en.wikipedia.org/wiki/Minimal_reproducible_example).
- When adding new features, please include corresponding tests to ensure these are well-tested and behave as expected.
- Please ensure all PRs pass `just verify` before submission.

## Development Environment

This project uses [Nix](https://nixos.org/) to manage the development environment.

1.  Install [Nix Package Manager](https://nixos.org/download/).
2.  Install [nix-direnv](https://github.com/nix-community/nix-direnv) (recommended) for automatic environment loading.

To set up the environment:

```sh
# If using direnv
direnv allow

# Or manually enter the shell
nix develop
```

This will provide a shell with the correct Rust version and dependencies.

### Rebuilding the Dev Shell

nix-direnv caches the dev shell for performance. If `devenv/flake.nix` or
`devenv/flake.lock` changes (e.g., after pulling new commits that update
dependencies or add tools), the cache may be stale and new tools will be
missing from your PATH.

**Signs the cache is stale:**

- A tool listed in `devenv/flake.nix` is not found (e.g., `lychee: command not found`).
- `direnv export` output says "Using cached dev shell" after you know the flake changed.
- A `just` recipe fails with a missing command even though the tool is in the flake.

**To rebuild:**

```sh
# Remove the cached shell and reload
rm -f .direnv/flake-profile* .direnv/nix-direnv.*
direnv allow
```

If using `nix develop` directly instead of direnv, exit and re-enter the shell.

## Building and Testing

All commands are run via [just](https://github.com/casey/just) recipes defined in the project's `justfile`. Never run `cargo` directly; the `justfile` handles Nix environment setup automatically.

```sh
just fmt     # Format all files (Rust, Nix, Markdown, YAML, TOML)
just clippy  # Run clippy
just test    # Run all tests (cached; only re-runs when content changes)
just doc     # Build docs (must produce zero warnings)
just deny    # Check licenses and advisories with cargo-deny
just verify  # Run fmt, check, clippy, deny, doc, test in order
just clean   # Remove build artifacts and test cache
```

Run `just --list` to see all available recipes.

## Snapshot Tests

The HM type signature generation system uses [insta](https://insta.rs/) snapshot tests to guard against regressions. If you change the signature rendering code in `fp-macros`, snapshots may need updating.

```sh
# Run snapshot tests to see if any changed
just test -p fp-macros --lib -- snapshot

# Review and accept/reject changed snapshots interactively
cargo insta review

# Accept all changed snapshots without review
cargo insta accept
```

You can also review snapshots manually by running the tests with `INSTA_UPDATE=new` and inspecting the `.snap.new` files in `fp-macros/src/documentation/snapshots/`.

## Compile-Fail Tests (trybuild)

Both `fp-macros` and `fp-library` use [trybuild](https://github.com/dtolnay/trybuild)
for compile-fail tests. These are `.rs` files in `tests/ui/` that must fail to compile,
with expected compiler output stored in matching `.stderr` golden files.

If your changes alter error messages (e.g., changing a proc macro's error output,
renaming types, or moving modules), the `.stderr` files need updating.

**Signs a trybuild test is stale:**

- Test output shows a diff between expected and actual compiler stderr.
- The diff contains changed file paths, line numbers, or error wording.

**To update:**

```sh
# Overwrite all stale .stderr files with current compiler output
TRYBUILD=overwrite just test

# Or update only a specific crate's tests
TRYBUILD=overwrite just test -p fp-macros
TRYBUILD=overwrite just test -p fp-library
```

Always review the updated `.stderr` files before committing to ensure the new
error messages are correct and intentional.

## Project Structure

See the [Project Structure](fp-library/docs/project-structure.md) documentation.

## Release Process

For maintainers, the release process is documented in [release-process.md](fp-library/docs/release-process.md).

## Benchmarking

This project uses [Criterion.rs](https://github.com/criterion-rs/criterion.rs) for benchmarking to ensure zero-cost abstractions and detect performance regressions.

```sh
just bench -p fp-library                               # To run all benchmarks
just bench -p fp-library --bench benchmarks -- --list  # To list available benchmarks
just bench -p fp-library --bench benchmarks -- Vec     # To run a specific benchmark (e.g., `Vec`)
```

Benchmark reports are generated in `target/criterion/report/index.html`.
