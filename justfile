# Load the Nix development environment via direnv for all recipes.
# In CI, recipes are invoked via `nix develop --command just`, so direnv
# is not available. Set SKIP_DIRENV=1 to bypass the prefix.
set shell := ["bash", "-c"]
skip_direnv := env_var_or_default("SKIP_DIRENV", "")
direnv_prefix := if skip_direnv != "" { "" } else { "{ direnv status | grep -q '^Found RC allowed 0$' || { echo \"ERROR: direnv environment is not approved. Review .envrc and Nix flake changes, then run 'just allow-env'.\" >&2; exit 1; }; __direnv_export=\"$(direnv export bash)\" && eval \"$__direnv_export\"; } &&" }

# List available recipes.
default:
    @just --list

# Approve the direnv environment after reviewing `.envrc` and Nix flake changes.
allow-env:
    direnv allow

# Format all files (Rust, Nix, Markdown, YAML, TOML) via treefmt.
fmt:
    {{ direnv_prefix }} cd devenv && nix fmt

# Run clippy (warnings are errors).
[positional-arguments]
clippy *args:
    #!/usr/bin/env bash
    set -euo pipefail
    if [ "$#" -eq 0 ]; then
        set -- --workspace --all-targets --all-features
    fi
    {{ direnv_prefix }} cargo clippy "$@" -- -D warnings

# Check documentation (warnings are errors) and reject any non-ASCII characters.
[positional-arguments]
doc *args:
    #!/usr/bin/env bash
    set -euo pipefail
    {{ direnv_prefix }} true
    # ASCII-only allow-list: reject any byte outside the printable ASCII
    # range. Catches em-dashes, en-dashes, smart quotes, non-breaking
    # spaces, emoji, math symbols, accented letters, CJK characters, and
    # anything else non-ASCII without per-character maintenance.
    matches=$(rg -nP '[^[:ascii:]]' fp-library/src/ fp-macros/src/ -g '*.rs' docs/ fp-library/docs/ -g '*.md' || true)
    if [[ -n "$matches" ]]; then
        echo "ERROR: Non-ASCII characters found in source or documentation files. Use ASCII equivalents (e.g., '->' not the unicode arrow, ',' or ';' not em-dash, '\"' not smart quotes)." >&2
        echo "" >&2
        echo "Offending lines:" >&2
        echo "$matches" >&2
        exit 1
    fi
    lychee --offline --no-progress "README.md" "fp-library/docs/**/*.md" "docs/**/*.md"
    if [ "$#" -eq 0 ]; then
        set -- --workspace --all-features --no-deps
    fi
    RUSTDOCFLAGS="-D warnings" cargo doc "$@"

# Build the workspace.
[positional-arguments]
build *args:
    #!/usr/bin/env bash
    set -euo pipefail
    if [ "$#" -eq 0 ]; then
        set -- --workspace --all-targets --all-features
    fi
    {{ direnv_prefix }} cargo build "$@"

# Run benchmarks. Use regex dots for spaces in benchmark names, e.g.:
# just bench -p fp-library --bench benchmarks -- "CatList.Left-Assoc"
[positional-arguments]
bench *args:
    #!/usr/bin/env bash
    set -euo pipefail
    if [ "$#" -eq 0 ]; then
        set -- --workspace --all-targets --all-features
    fi
    {{ direnv_prefix }} cargo bench "$@"

# Check without building.
[positional-arguments]
check *args:
    #!/usr/bin/env bash
    set -euo pipefail
    if [ "$#" -eq 0 ]; then
        set -- --workspace --all-targets --all-features
    fi
    {{ direnv_prefix }} cargo check "$@"

# Run any cargo subcommand (except test; use `just test` for that).
[positional-arguments]
cargo *args:
    #!/usr/bin/env bash
    set -euo pipefail
    if [ "$#" -eq 0 ]; then
        echo "ERROR: cargo subcommand required." >&2
        exit 2
    fi
    if [ "$1" = "test" ]; then
        echo "ERROR: Use 'just test' instead of 'just cargo test'." >&2
        exit 1
    fi
    {{ direnv_prefix }} cargo "$@"

# Run tests with output caching. Re-runs only when source files have changed.
# Each unique set of arguments gets its own independent cache.
[positional-arguments]
test *args:
    #!/usr/bin/env bash
    set -euo pipefail
    mkdir -p .cache/test-output
    ARGS=$(printf '%q ' "$@")
    CONTENT_HASH=$(git ls-files -z | xargs -0 md5sum 2>/dev/null | md5sum | cut -c1-32 || true)
    CACHE_KEY=$(echo "${ARGS}:${CONTENT_HASH}" | md5sum | cut -c1-12)
    OUTPUT_FILE=".cache/test-output/test-output-${CACHE_KEY}.txt"
    STATUS_FILE=".cache/test-output/test-output-${CACHE_KEY}.status"
    if [ -s "$OUTPUT_FILE" ] && [ -s "$STATUS_FILE" ]; then
        echo "=== CACHED TEST OUTPUT (no source changes) ==="
        (trap '' PIPE; cat "$OUTPUT_FILE")
        exit "$(cat "$STATUS_FILE")"
    else
        echo "=== Running tests ==="
        TEMP_FILE="${OUTPUT_FILE}.tmp"
        TEMP_STATUS_FILE="${STATUS_FILE}.tmp"
        rm -f "$TEMP_FILE"
        rm -f "$TEMP_STATUS_FILE"
        trap 'rm -f "$TEMP_FILE" "$TEMP_STATUS_FILE"' INT TERM HUP
        RC=0
        if [ "$#" -eq 0 ]; then
            set -- --workspace --all-features
        fi
        {{ direnv_prefix }} cargo test "$@" > "$TEMP_FILE" 2>&1 || RC=$?
        if [ ! -s "$TEMP_FILE" ]; then
            rm -f "$TEMP_FILE"
            rm -f "$TEMP_STATUS_FILE"
            exit "${RC:-1}"
        fi
        printf '%s\n' "$RC" > "$TEMP_STATUS_FILE"
        mv "$TEMP_FILE" "$OUTPUT_FILE"
        mv "$TEMP_STATUS_FILE" "$STATUS_FILE"
        (trap '' PIPE; cat "$OUTPUT_FILE")
        exit "$RC"
    fi

# Remove build artifacts and test cache.
clean:
    {{ direnv_prefix }} cargo clean
    rm -rf .cache/test-output/

# Check licenses and advisories with cargo-deny.
deny:
    {{ direnv_prefix }} cargo deny check

# Run an allowed just recipe and filter its output with a caller-provided rg regex.
[positional-arguments]
filtered recipe filter *args:
    #!/usr/bin/env bash
    set -euo pipefail

    recipe="$1"
    filter="$2"
    shift 2

    if [ -z "$filter" ]; then
        echo "ERROR: filtered requires a non-empty rg regex." >&2
        exit 2
    fi

    case "$recipe" in
        check|clippy|deny|doc|fmt|test|verify) ;;
        *)
            echo "ERROR: unsupported filtered recipe: $recipe" >&2
            exit 2
            ;;
    esac

    for arg in "$@"; do
        case "$arg" in
            *$'\n'*|*$'\r'*|*[\;\&\|\\\<\>\`\$\'\"\(\)\{\}]*)
                echo "ERROR: unsafe filtered recipe argument: $arg" >&2
                exit 2
                ;;
        esac
    done

    output=$(mktemp -t just-filtered.XXXXXX)
    trap 'rm -f "$output"' EXIT

    set +e
    just --one "$recipe" "$@" > "$output" 2>&1
    recipe_status=$?
    set -e

    rg_status=0
    rg -n -m 300 -- "$filter" "$output" || rg_status=$?
    if [ "$rg_status" -eq 2 ]; then
        exit 2
    fi

    if [ "$rg_status" -ne 0 ] && [ "$recipe_status" -ne 0 ]; then
        echo "=== no filter matches; last 80 lines ===" >&2
        tail -n 80 "$output" >&2
    fi

    exit "$recipe_status"

# Verify: fmt, check, clippy, deny, doc, then test (in order).
verify:
    just fmt
    just check
    just clippy
    just deny
    just doc
    just test
