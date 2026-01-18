# Release Process

This document outlines the steps for releasing new versions of `fp-library` and `fp-macros`.

## Prerequisites

- Ensure you have the latest code: `git pull`
- Ensure the working directory is clean: `git status`
- Ensure all tests pass: `cargo test`

## Merging to Main

All changes should be merged into `main` via Pull Requests before a release is cut.

- **Squash and Merge**: This is the preferred method to keep the `main` branch history clean and linear.
- **CI Checks**: Ensure all Continuous Integration checks (tests, clippy, formatting) pass before merging.
- **Conventional Commits**: Encouraged for PR titles/squash commits to simplify changelog generation (e.g., `feat:`, `fix:`, `chore:`, `refactor:`, `docs:`, etc.).

## Release Steps

### 1. Determine Version Bump

Follow [Semantic Versioning](https://semver.org/), with the following policy for pre-1.0.0 releases:

- **Major** (X.0.0): Reserved for when the API is declared stable.
- **Minor** (0.X.0): Incompatible API changes or significant new functionality.
- **Patch** (0.0.X): Backwards-compatible bug fixes or minor additions.

### 2. Update Changelogs

Update `fp-library/CHANGELOG.md` and `fp-macros/CHANGELOG.md` (if applicable):

1.  Rename the `[Unreleased]` section to the new version number and date (e.g., `[0.3.0] - 2026-01-16`).
2.  Ensure all notable changes are listed under appropriate headers (Added, Changed, Removed, Fixed).
3.  Create a new empty `[Unreleased]` section at the top.

### 3. Update Cargo.toml

#### fp-macros (if changed)

1.  Update `version` in `fp-macros/Cargo.toml` and in `fp-macros/README.md`.

#### fp-library

1.  Update `version` in `fp-library/Cargo.toml` and in `README.md`.
2.  If `fp-macros` was updated, ensure the `fp-macros` dependency in `fp-library/Cargo.toml` matches the new version.

### 4. Verification

Run the full suite of checks to ensure the release is stable:

```bash
# Check compilation
cargo check

# Run all tests (unit, integration, and doc)
cargo test

# Run linter
cargo clippy

# Verify documentation builds and looks correct
cargo doc --open
```

### 5. Commit and Tag

1.  Stage the changes:

    ```bash
    git add fp-library/Cargo.toml fp-library/CHANGELOG.md
    # Add fp-macros files if changed
    git add fp-macros/Cargo.toml fp-macros/CHANGELOG.md
    ```

2.  Commit with a release message:

    ```bash
    git commit -m "chore: release fp-library vX.Y.Z / fp-macros vA.B.C"
    ```

3.  Tag the release(s):
    Since crates are versioned independently, create a tag for each crate being released:

    ```bash
    # If releasing fp-library
    git tag fp-library-vX.Y.Z

    # If releasing fp-macros
    git tag fp-macros-vA.B.C
    ```

### 6. Publish

**Order matters:** If `fp-macros` was updated, it must be published first because `fp-library` depends on it.

1.  **Publish fp-macros** (if updated):

    ```bash
    cargo publish -p fp-macros
    ```

    _Wait a few moments for crates.io to index the new version._

2.  **Publish fp-library**:

    ```bash
    cargo publish -p fp-library
    ```

3.  **Push to Remote**:
    ```bash
    git push origin main
    git push origin --tags
    ```

## Post-Release

- Verify the new version is available on [crates.io](https://crates.io/crates/fp-library).
- Verify documentation is updated on [docs.rs](https://docs.rs/fp-library).
