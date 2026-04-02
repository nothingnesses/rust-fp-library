#!/usr/bin/env bash
# Times each max-depth/size benchmark individually with a 35-second timeout.
# Verifies that no individual benchmark exceeds 30 seconds of wall time when
# run with Criterion's --sample-size 10.
#
# Pre-compiles benchmarks first to exclude compilation time from measurements.
#
# Usage: bash scripts/bench_timing.sh

set -euo pipefail

cd "$(git rev-parse --show-toplevel)"
eval "$(direnv export bash 2>/dev/null || true)"

echo "Pre-compiling benchmarks..."
cargo bench -p fp-library --bench benchmarks --no-run 2>/dev/null
echo ""

BENCHMARKS=(
  # Lazy evaluation (shallow max: depth 100)
  "Thunk Map Chain/depth/100$"
  "Thunk Bind Chain/depth/100$"
  "RcLazy ref_map Chain/depth/100$"
  "ArcLazy ref_map Chain/depth/100$"

  # Lazy evaluation (deep max: depth 10000)
  "Trampoline Bind Chain/depth/10000$"
  "Trampoline Map Chain/depth/10000$"
  "Free Left-Assoc Bind/depth/10000$"
  "Free Right-Assoc Bind/depth/10000$"
  "Free Evaluate/depth/10000$"

  # Coyoneda (max depth: 100)
  "Coyoneda/Direct/100$"
  "Coyoneda/Coyoneda/100$"
  "Coyoneda/CoyonedaExplicit/100$"
  "Coyoneda/RcCoyoneda/100$"
  "Coyoneda/ArcCoyoneda/100$"
  "Coyoneda Repeated Lower/RcCoyoneda/100$"
  "Coyoneda Repeated Lower/ArcCoyoneda/100$"
  "Coyoneda Clone Map/RcCoyoneda/100$"
  "Coyoneda Clone Map/ArcCoyoneda/100$"

  # CatList structural (max size: 5000)
  "CatList Cons/CatList/5000$"
  "CatList Cons/Vec/5000$"
  "CatList Snoc/CatList/5000$"
  "CatList Append/CatList/5000$"
  "CatList Uncons/CatList/5000$"
  "CatList Left-Assoc Append/CatList/5000$"
  "CatList Iteration/CatList/5000$"
  "CatList Nested Uncons/CatList .nested./5000$"

  # CatList type class ops (max size: 5000)
  "CatList Fold Map/CatList .fp./5000$"
  "CatList Fold Left/CatList .fp./5000$"
  "CatList Traverse/CatList .fp./5000$"
  "CatList Filter/CatList .fp./5000$"
  "CatList Compact/CatList .fp./5000$"

  # Vec parallel (max size: 100000)
  "Vec Par Map/par_map/100000$"
  "Vec Par Map/map .sequential./100000$"
  "Vec Par Fold Map/par_fold_map/100000$"
  "Vec Par Fold Map/fold_map .sequential./100000$"
  "Vec Par Filter Map/par_filter_map/100000$"
  "Vec Par Filter Map/filter_map .sequential./100000$"
  "Vec Par Compact/par_compact/100000$"
  "Vec Par Compact/compact .sequential./100000$"
)

PASS=0
FAIL=0
TIMEOUT=0

for bench in "${BENCHMARKS[@]}"; do
  start=$(date +%s%3N)
  if timeout 35 cargo bench -p fp-library --bench benchmarks -- "$bench" --sample-size 10 > /tmp/bench_out.txt 2>&1; then
    end=$(date +%s%3N)
    elapsed=$((end - start))
    per_iter=$(grep "time:" /tmp/bench_out.txt | head -1 | sed 's/.*\[//' | sed 's/\].*//' | awk '{print $1, $2}')
    if [ "$elapsed" -gt 30000 ]; then
      echo "SLOW  ${elapsed}ms  (${per_iter})  $bench"
      FAIL=$((FAIL + 1))
    else
      echo "OK    ${elapsed}ms  (${per_iter})  $bench"
      PASS=$((PASS + 1))
    fi
  else
    end=$(date +%s%3N)
    elapsed=$((end - start))
    if [ "$elapsed" -ge 34000 ]; then
      echo "TIMEOUT  ${elapsed}ms  $bench"
      TIMEOUT=$((TIMEOUT + 1))
    else
      echo "FAIL  ${elapsed}ms  $bench"
      FAIL=$((FAIL + 1))
    fi
  fi
done

echo ""
echo "Summary: $PASS OK, $FAIL slow/failed, $TIMEOUT timed out (of ${#BENCHMARKS[@]} total)"
