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
