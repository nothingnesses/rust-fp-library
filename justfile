# Load the Nix development environment via direnv for all recipes.
set shell := ["bash", "-c"]
direnv_prefix := "direnv allow && eval \"$(direnv export bash)\" &&"

# List available recipes.
default:
    @just --list

# Format code (rustfmt).
fmt *args:
    {{direnv_prefix}} cargo fmt {{args}}

# Run clippy.
clippy *args:
    {{direnv_prefix}} cargo clippy {{args}}

# Check documentation (must produce zero warnings).
doc *args:
    {{direnv_prefix}} cargo doc {{args}}

# Build the workspace.
build *args:
    {{direnv_prefix}} cargo build {{args}}

# Check without building.
check *args:
    {{direnv_prefix}} cargo check {{args}}

# Run benchmarks.
bench *args:
    {{direnv_prefix}} cargo bench {{args}}

# Run any cargo subcommand (except test; use `just test` for that).
cargo *args:
    #!/usr/bin/env bash
    if [ "$1" = "test" ]; then
        echo "ERROR: Use 'just test' instead of 'just cargo test'." >&2
        exit 1
    fi
    {{direnv_prefix}} cargo "$@"

# Run tests with output caching. Re-runs only when source files have changed.
test *args:
    #!/usr/bin/env bash
    set -euo pipefail
    mkdir -p .claude/test-cache
    LATEST=$(find fp-library/src fp-macros/src tests -name '*.rs' -printf '%T@\n' 2>/dev/null | sort -rn | head -1; find . -maxdepth 2 -name 'Cargo.toml' -printf '%T@\n' | sort -rn | head -1)
    CACHED=$(cat .claude/test-cache/source-timestamp.txt 2>/dev/null || echo "0")
    ARGS="{{ args }}"
    if [ "$LATEST" = "$CACHED" ] && [ -z "$ARGS" ]; then
        echo "=== CACHED TEST OUTPUT (no source changes) ==="
        cat .claude/test-cache/test-output.txt
    else
        echo "=== Running tests ==="
        {{direnv_prefix}} cargo test --workspace --all-features $ARGS 2>&1 | tee .claude/test-cache/test-output.txt
        if [ -z "$ARGS" ]; then
            echo "$LATEST" > .claude/test-cache/source-timestamp.txt
        fi
    fi

# Force re-run tests (ignores cache).
test-force *args:
    #!/usr/bin/env bash
    set -euo pipefail
    rm -f .claude/test-cache/source-timestamp.txt
    just test {{ args }}

# Run a subset of tests (always runs, does not update cache timestamp).
test-subset +args:
    #!/usr/bin/env bash
    set -euo pipefail
    mkdir -p .claude/test-cache
    {{direnv_prefix}} cargo test {{ args }} 2>&1 | tee .claude/test-cache/test-output.txt

# Verify: fmt, clippy, doc, then test (in order).
verify:
    just fmt --all
    just clippy --workspace --all-features
    just doc --workspace --all-features --no-deps
    just test
