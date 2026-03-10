# Release Process

This document outlines the steps for releasing new versions of `fp-library` and `fp-macros`.

## Prerequisites

- Ensure you have the latest code: `git pull`
- Ensure the working directory is clean: `git status`
- Ensure all tests pass: `cargo test`

## Merging to Main

All changes should be merged into `main` via Pull Requests before a release is cut.

- **Squash and Merge**: This is the preferred method to keep the `main` branch history clean and linear.
- **CI Checks**: All CI checks (tests, clippy, formatting, cargo-deny) must pass before merging. See `.github/workflows/ci.yml`.
- **Conventional Commits**: Encouraged for PR titles/squash commits to simplify changelog generation (e.g., `feat:`, `fix:`, `chore:`, `refactor:`, `docs:`, etc.).

## Release Steps

### 1. Determine Version Bump

Follow [Semantic Versioning](https://semver.org/), with the following policy for pre-1.0.0 releases:

- **Major** (X.0.0): Reserved for when the API is declared stable.
- **Minor** (0.X.0): Incompatible API changes or significant new functionality.
- **Patch** (0.0.X): Backwards-compatible bug fixes or minor additions.

### 2. Update Changelogs

Update `fp-library/CHANGELOG.md` and `fp-macros/CHANGELOG.md` (if applicable).

#### Determining changelog content

For each package being released, determine what has changed since the last release:

1.  **Find the latest tag for the package:**

    ```bash
    # For fp-library
    git tag --list 'fp-library-v*' --sort=-version:refname | head -1

    # For fp-macros
    git tag --list 'fp-macros-v*' --sort=-version:refname | head -1
    ```

2.  **Review the diff between the tag and the current state:**

    ```bash
    # View commits since the last release
    git log fp-library-vX.Y.Z..HEAD -- fp-library/

    # View the full diff for detailed inspection
    git diff fp-library-vX.Y.Z..HEAD -- fp-library/
    ```

    Replace the tag with the appropriate one for `fp-macros` when reviewing that package.

3.  **Categorize the changes** into the appropriate changelog headers:

    - **Added** - New features, new modules, new type class implementations.
    - **Changed** - Modifications to existing APIs, behavior changes, refactors that affect the public interface.
    - **Removed** - Removed features, deprecated items that have been deleted.
    - **Fixed** - Bug fixes.

    Use the commit messages (especially conventional commit prefixes like `feat:`, `fix:`, `refactor:`) as a guide, but always verify against the actual diff to ensure nothing is missed or mischaracterized.

#### Writing the changelog entry

1.  Rename the `[Unreleased]` section to the new version number and date (e.g., `[0.3.0] - 2026-01-16`).
2.  List all notable changes under the appropriate headers (Added, Changed, Removed, Fixed).
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

### 6. Push and Publish

Push the commit and tags to trigger the automated release workflow:

```bash
git push origin main
git push origin --tags
```

The release workflow (`.github/workflows/release.yml`) will automatically:

1.  Run the full test/clippy/doc validation suite.
2.  Publish `fp-macros` to crates.io (if an `fp-macros-v*` tag was pushed).
3.  Publish `fp-library` to crates.io (if an `fp-library-v*` tag was pushed).
4.  Create a GitHub Release with auto-generated notes.

**Order matters when both crates are released together:** Push the `fp-macros` tag first and wait for its publish job to succeed before pushing the `fp-library` tag, since `fp-library` depends on `fp-macros`.

> **Note:** The release workflow requires a `CARGO_REGISTRY_TOKEN` secret configured in the repository settings.

#### Manual publishing (fallback)

If the automated workflow fails or you need to publish manually:

```bash
# Publish fp-macros first (if updated)
cargo publish -p fp-macros
# Wait for crates.io to index it
cargo publish -p fp-library
```

## Post-Release

- Verify the release workflow succeeded in the GitHub Actions tab.
- Verify the new version is available on [crates.io](https://crates.io/crates/fp-library).
- Verify documentation is updated on [docs.rs](https://docs.rs/fp-library).
