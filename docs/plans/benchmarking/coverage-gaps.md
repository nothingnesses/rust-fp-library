# Benchmark Coverage Gaps

Audit of benchmark coverage as of 2026-04-01. Organized by priority.

## Current State

Benchmarks exist for: Vec, Option, Result, Pair, String, CatList, Coyoneda
(all 4 variants), Thunk, Trampoline, Free, RcLazy, ArcLazy, identity function.

Vec has the most thorough coverage (18 benchmark groups comparing fp vs std).
CatList compares against Vec and LinkedList across multiple input sizes.
Coyoneda compares all 4 variants across varying map depths.

## Priority 1: Optics

No benchmarks exist for any optics operations despite the profunctor optics
system being one of the library's headline features.

Needed benchmarks:

- **Lens get/set vs direct field access.** Measures profunctor encoding overhead.
  Compare `view(lens, s)` vs `s.field` and `set(lens, s, a)` vs `S { field: a, ..s }`.
- **Prism review/match vs pattern matching.** Compare `preview(prism, s)` vs
  `match s { Variant(x) => Some(x), _ => None }`.
- **Iso forward/backward vs direct conversion.** Compare `view(iso, s)` vs
  direct function call.
- **Composition chains.** Lens-then-lens, lens-then-prism, traversal-then-lens
  at varying depths (1, 2, 3, 5 composed optics). This is the most important
  optics benchmark since composition is where profunctor overhead compounds.
- **Traversal fold vs direct iteration.** Compare `fold_of(traversal, s)` vs
  manual iteration.
- **Indexed optics.** IndexedLens, IndexedTraversal vs non-indexed equivalents.

These should use realistic nested structs (2-3 levels deep) rather than
trivial wrappers.

## Priority 2: Fallible Lazy Types

`TryThunk`, `TrySendThunk`, `TryTrampoline`, `TryLazy` (Rc and Arc) have
zero benchmarks despite being documented as core types in lazy-evaluation.md.

Needed benchmarks:

- **TryThunk map/bind chains** at varying depths (1, 5, 10, 25, 50, 100)
  with both Ok and Err paths. Compare vs Thunk wrapping Result manually.
- **TryTrampoline recursive evaluation** with varying recursion depths
  (100, 1000, 10000). Compare vs Trampoline wrapping Result.
- **TryLazy evaluate** for both RcTryLazy and ArcTryLazy. Measure first
  evaluation, cached access, and error caching.
- **TrySendThunk** vs TryThunk to show Send overhead.
- **Error frequency impact.** Same computation with 0%, 10%, 50% error
  rates to show early-exit benefits.

## Priority 3: Newtype Wrappers (Zero-Cost Verification)

The newtype wrappers (Additive, Multiplicative, Dual, First, Last,
Conjunctive, Disjunctive) claim to be zero-cost. This should be verified.

Needed benchmarks:

- **append vs raw operation.** `append(Additive(x), Additive(y))` vs `x + y`,
  `append(Multiplicative(x), Multiplicative(y))` vs `x * y`, etc.
- **fold_map with monoid wrappers.** `fold_map(|x| Additive(x), vec)` vs
  `vec.iter().sum()`.
- **empty construction.** `empty::<Additive<i32>>()` vs `0i32`.

These should confirm zero overhead via identical timings. If overhead exists,
it indicates a missed optimization.

## Priority 4: CatList Missing Operations

CatList benchmarks cover structural operations (cons, snoc, append, uncons,
iteration) but miss type class operations.

Needed benchmarks:

- **Fold left/right** vs Vec fold. CatList iteration overhead on folding.
- **Fold map** vs Vec fold_map.
- **Traverse** (Result). CatList traverse vs Vec traverse.
- **Filter/compact.** CatList filter vs Vec filter.

These complete the CatList story: structural operations are already shown to
be O(1), but iteration-heavy type class operations may show different
characteristics due to the internal flattening cost.

## Priority 5: SendThunk and Identity

- **SendThunk**: Map/bind chains comparing vs Thunk to quantify the Send
  bound overhead (Box<dyn FnOnce + Send> vs Box<dyn FnOnce>).
- **Identity**: Map, bind, fold comparing fp vs direct operation to verify
  zero-cost wrapper claim. Should show identical timings.

## Priority 6: Parallel Operations

Only `par_fold_map` on Vec is benchmarked.

Needed benchmarks:

- **par_map** on Vec at varying sizes (1000, 10000, 100000) to show crossover
  point where rayon parallelism pays off.
- **par_filter_map, par_compact** on Vec.
- **par_fold_map** on CatList (if implemented).
- **Sequential vs parallel comparison** for each operation to show speedup
  factor.

These require the `rayon` feature and are most meaningful on larger input
sizes where parallelism overhead is amortized.

## Not Worth Benchmarking

- **Numerical type classes** (Semiring, Ring, Field): Trivial trait impls
  delegating to primitive ops. Overhead is a function call that gets inlined.
- **Bifunctor/Contravariant**: Simple single-pass operations with predictable
  cost.
- **ControlFlow, Tuple1, Tuple2**: Thin wrappers with no interesting
  performance characteristics.
- **Category/Semigroupoid**: Function composition; overhead is negligible.

## Graph Generation

Once benchmarks are in place, Criterion generates HTML reports with line plots
at `target/criterion/<group>/report/index.html`. For documentation:

1. Run `just bench -p fp-library` to generate reports.
2. Export key graphs as PNGs.
3. Commit to `benchmarks/` directory in the repo root.
4. Reference from docs/benchmarking.md and the README.

This follows the pattern used by purescript-catenable-lists.
