### Tasks to do & ideas to look into

- Optics: Implement missing functionality from [analysis](optics-analysis.md).
- Algebraic effects/effect system to implement extensible effects
  - [Analysis](plans/effects/effects.md)
  - [Eff](https://github.com/lexi-lambda/eff) [documentation](https://hasura.github.io/eff/Control-Effect.html)
- Inline `!`-notation within `m_do!`: allow `m_do! { pure(!fa + !fb) }` as shorthand that automatically lifts subexpressions into binds, similar to Idris's `!`-notation. Avoids unnecessary intermediate bindings when a value is used once, immediately. Implement as an incremental enhancement to `m_do!` rather than a standalone feature.
- Property-based tests for type class laws.
  - [Validity](https://github.com/NorfairKing/validity).
- Add a diagram of the typeclass/trait hierarchy and reasoning/justification for why the current hierarchy is as it is.
- For each equivalent abstraction in docs/benchmarks/comparisons.md / docs/std_coverage_checklist.md , there should be a property-based test to test for equivalence.
- [Mutation testing](https://github.com/sourcefrog/cargo-mutants).
- [Lazy, memoized data type](https://pursuit.purescript.org/packages/purescript-lazy/3.0.0/docs/Data.Lazy) for [dynamic programming](https://en.wikipedia.org/wiki/Dynamic_programming#Computer_science). (Partially implemented via `Lazy`, `TryLazy`).
- [Monadic stream functions](https://github.com/ivanperez-keera/dunai).
- Inner vs outer iteration.
- Add benchmark outputs and graphs to repo to make them accessible. Options:
  - **Commit PNGs to a `benchmarks/` directory.** Simple, version-controlled, visible on GitHub. Reference from docs and README. Downside: goes stale unless regenerated before releases.
  - **GitHub Pages with Criterion reports.** Push the full Criterion HTML output to a `gh-pages` branch via CI. Always up to date, interactive charts. More setup.
  - **Separate repo.** Prevents bloating the main repo. Downside: harder to keep in sync with code changes.
  - Regardless of hosting, regenerating graphs should be part of the release process.
- Write user stories for all types, traits, and modules. Each should have a one-line "I want to..." description explaining when and why a user would reach for it. See `docs/coyoneda.md`, `docs/lazy-evaluation.md`, and `fp-library/src/types/free.rs` for the pattern. Prioritize types that are easy to confuse with each other (e.g., Thunk vs Trampoline vs Lazy, the four Coyoneda variants, Functor vs RefFunctor vs SendRefFunctor).
- Expand benchmark coverage per [benchmarking/coverage-gaps.md](plans/benchmarking/coverage-gaps.md). Priority order: optics, fallible lazy types, newtype wrappers (zero-cost verification), CatList type class ops, SendThunk/Identity, parallel operations.

### Parallel type classes

The core parallel traits are implemented: `ParFunctor`, `ParFoldable`, `ParCompactable`, `ParFilterable`, `ParFunctorWithIndex`, `ParFoldableWithIndex`, `ParFilterableWithIndex`.

**`ParTraversable`** (not yet implemented) -two distinct flavours with different feasibility:

- _Error accumulation_ (the `Validation` flavour): `par_traverse(f, ta)` runs all `f(a)` and accumulates all errors, rather than short-circuiting on the first `Err`. Implemented as `traverse::<T, ValidationBrand<E>, _, _>` using `Validation<E, A>`'s accumulating `Applicative`. Requires adding `Validation<E, A>` with its `Semiapplicative::apply` instance; no new HKT machinery beyond that. **Feasible.**
- _CPU parallelism of effectful functions_: run `f: A -> Result<B, E>` on all elements across rayon threads simultaneously. Requires a concurrent execution type analogous to PureScript's `ParAff` -a deferred task type where `apply` uses `rayon::join`. This conflicts with fp-library's `impl Fn` (not `FnOnce`) applicative model and requires `'static` bounds. For pure `A -> B` functions this reduces to `par_map`, which already exists. **Not currently feasible without major infrastructure.**
- _Practical alternative_: `par_map(f, ta)` (CPU parallel) followed by `sequence_validation` (error accumulation) composes both behaviours without needing a unified `par_traverse`.

**`ParWitherable`** -same two flavours as `ParTraversable`, since `Witherable: Traversable + Filterable`. The error-accumulation flavour (`wither` with `Validation<E, Option<B>>`) is feasible alongside `ParTraversable`. The CPU-parallel effectful flavour has the same obstacles.
