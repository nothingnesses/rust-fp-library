#!/usr/bin/env bash
# Wrapper that loads the Nix dev environment via direnv before invoking cargo.
# Usage: .claude/direnv-cargo.sh <subcommand> [args...]
#
# "cargo test" is blocked here to enforce use of the test output caching
# wrapper defined in CLAUDE.md. Use the caching one-liner instead.

if [ "$1" = "test" ]; then
  echo "ERROR: Do not use .claude/direnv-cargo.sh test directly." >&2
  echo "Use the test output caching wrapper from CLAUDE.md instead." >&2
  exit 1
fi

direnv allow && eval "$(direnv export bash)" && cargo "$@"
