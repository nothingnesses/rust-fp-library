# Load the Nix development environment via direnv for all recipes.
set shell := ["bash", "-c"]
direnv_prefix := "direnv allow && eval \"$(direnv export bash)\" &&"

# List available recipes.
default:
    @just --list

# Format all files (Rust, Nix, Markdown, YAML, TOML) via treefmt.
fmt:
    cd devenv && nix fmt

# Run clippy (warnings are errors).
clippy *args:
    {{direnv_prefix}} cargo clippy {{args}} -- -D warnings

# Check documentation (warnings are errors) and reject emoji/unicode.
doc *args:
    #!/usr/bin/env bash
    set -euo pipefail
    if grep -rn '[✅❌⚠⚡←→↔≥≤≠✓✗✘✔✖──━┃┏┓┗┛┣┫┳┻╋═║►▶◀◁▲△▼▽●○■□★☆♠♣♥♦]' fp-library/src/ fp-macros/src/ --include='*.rs' 2>/dev/null; then
        echo "ERROR: Found emoji or unicode characters in source files. Use ASCII equivalents." >&2
        exit 1
    fi
    RUSTDOCFLAGS="-D warnings" {{direnv_prefix}} cargo doc {{args}}

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
# Each unique set of arguments gets its own independent cache.
test *args:
    #!/usr/bin/env bash
    set -euo pipefail
    mkdir -p .claude/test-cache
    ARGS="{{ args }}"
    CACHE_KEY=$(echo "$ARGS" | md5sum | cut -c1-12)
    OUTPUT_FILE=".claude/test-cache/test-output-${CACHE_KEY}.txt"
    TIMESTAMP_FILE=".claude/test-cache/source-timestamp-${CACHE_KEY}.txt"
    LATEST=$(find fp-library/src fp-macros/src tests -name '*.rs' -printf '%T@\n' 2>/dev/null | sort -rn | head -1; find . -maxdepth 2 -name 'Cargo.toml' -printf '%T@\n' | sort -rn | head -1)
    CACHED=$(cat "$TIMESTAMP_FILE" 2>/dev/null || echo "0")
    if [ "$LATEST" = "$CACHED" ]; then
        echo "=== CACHED TEST OUTPUT (no source changes) ==="
        cat "$OUTPUT_FILE"
    else
        echo "=== Running tests ==="
        {{direnv_prefix}} cargo test --workspace --all-features $ARGS 2>&1 | tee "$OUTPUT_FILE"
        echo "$LATEST" > "$TIMESTAMP_FILE"
    fi

# Verify: fmt, clippy, doc, then test (in order).
verify:
    just fmt
    just clippy --workspace --all-features
    just doc --workspace --all-features --no-deps
    just test
