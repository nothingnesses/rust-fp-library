# Load the Nix development environment via direnv for all recipes.
# In CI, recipes are invoked via `nix develop --command just`, so direnv
# is not available. Set SKIP_DIRENV=1 to bypass the prefix.
set shell := ["bash", "-c"]
skip_direnv := env_var_or_default("SKIP_DIRENV", "")
direnv_prefix := if skip_direnv != "" { "" } else { "direnv allow && eval \"$(direnv export bash)\" &&" }

# List available recipes.
default:
    @just --list

# Format all files (Rust, Nix, Markdown, YAML, TOML) via treefmt.
fmt:
    cd devenv && nix fmt

# Run clippy (warnings are errors).
clippy *args:
    {{direnv_prefix}} cargo clippy {{ if args == "" { "--workspace --all-targets --all-features" } else { args } }} -- -D warnings

# Check documentation (warnings are errors) and reject emoji/unicode.
doc *args:
    #!/usr/bin/env bash
    set -euo pipefail
    {{direnv_prefix}} true
    if grep -rn '[✅❌⚠⚡←→↔≥≤≠✓✗✘✔✖──━┃┏┓┗┛┣┫┳┻╋═║►▶◀◁▲△▼▽●○■□★☆♠♣♥♦]' fp-library/src/ fp-macros/src/ --include='*.rs' docs/ fp-library/docs/ --include='*.md' 2>/dev/null; then
        echo "ERROR: Found emoji or unicode characters in source or documentation files. Use ASCII equivalents." >&2
        exit 1
    fi
    lychee --offline --no-progress "README.md" "fp-library/docs/**/*.md" "docs/**/*.md"
    RUSTDOCFLAGS="-D warnings" cargo doc {{ if args == "" { "--workspace --all-features --no-deps" } else { args } }}

# Build the workspace.
build *args:
    {{direnv_prefix}} cargo build {{ if args == "" { "--workspace --all-targets --all-features" } else { args } }}

# Run benchmarks. Use regex dots for spaces in benchmark names, e.g.:
#   just bench -p fp-library --bench benchmarks -- "CatList.Left-Assoc"
bench *args:
    {{direnv_prefix}} cargo bench {{ if args == "" { "--workspace --all-targets --all-features" } else { args } }}

# Check without building.
check *args:
    {{direnv_prefix}} cargo check {{ if args == "" { "--workspace --all-targets --all-features" } else { args } }}

# Run any cargo subcommand (except test; use `just test` for that).
cargo *args:
    #!/usr/bin/env bash
    set -- {{args}}
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
    mkdir -p .cache/test-output
    ARGS="{{ args }}"
    CONTENT_HASH=$(git ls-files -z | xargs -0 md5sum 2>/dev/null | md5sum | cut -c1-32 || true)
    CACHE_KEY=$(echo "${ARGS}:${CONTENT_HASH}" | md5sum | cut -c1-12)
    OUTPUT_FILE=".cache/test-output/test-output-${CACHE_KEY}.txt"
    if [ -s "$OUTPUT_FILE" ]; then
        echo "=== CACHED TEST OUTPUT (no source changes) ==="
        (trap '' PIPE; cat "$OUTPUT_FILE")
    else
        echo "=== Running tests ==="
        TEMP_FILE="${OUTPUT_FILE}.tmp"
        rm -f "$TEMP_FILE"
        trap 'rm -f "$TEMP_FILE"' INT TERM HUP
        RC=0
        {{direnv_prefix}} cargo test {{ if args == "" { "--workspace --all-features" } else { args } }} > "$TEMP_FILE" 2>&1 || RC=$?
        if [ ! -s "$TEMP_FILE" ]; then
            rm -f "$TEMP_FILE"
            exit "${RC:-1}"
        fi
        mv "$TEMP_FILE" "$OUTPUT_FILE"
        (trap '' PIPE; cat "$OUTPUT_FILE")
        exit "$RC"
    fi

# Remove build artifacts and test cache.
clean:
    {{direnv_prefix}} cargo clean
    rm -rf .cache/test-output/

# Check licenses and advisories with cargo-deny.
deny:
    {{direnv_prefix}} cargo deny check

# Verify: fmt, check, clippy, deny, doc, then test (in order).
verify:
    just fmt
    just check
    just clippy
    just deny
    just doc
    just test
