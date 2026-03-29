### Tasks to do & ideas to look into

- Optics: Implement missing functionality from [analysis](optics-analysis.md).
- Algebraic effects/effect system to implement extensible effects
  - [Analysis](../plans/effects/effects.md)
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
- Add benchmark outputs and graphs to repo to make them accessible? Maybe they should be in a separate repo, to prevent bloating this one?

### Parallel type classes

See `fp-library/src/classes/par_functor.rs`, `par_foldable.rs`, `par_compactable.rs`, `par_filterable.rs` (planned).

The four core parallel traits mirror the sequential hierarchy exactly: `ParFunctor`, `ParFoldable`, `ParCompactable`, `ParFilterable`. Indexed variants (`ParFunctorWithIndex`, `ParFoldableWithIndex`) are a natural follow-on.

**`ParTraversable`** — two distinct flavours with different feasibility:

- _Error accumulation_ (the `Validation` flavour): `par_traverse(f, ta)` runs all `f(a)` and accumulates all errors, rather than short-circuiting on the first `Err`. Implemented as `traverse::<T, ValidationBrand<E>, _, _>` using `Validation<E, A>`'s accumulating `Applicative`. Requires adding `Validation<E, A>` with its `Semiapplicative::apply` instance; no new HKT machinery beyond that. **Feasible.**
- _CPU parallelism of effectful functions_: run `f: A -> Result<B, E>` on all elements across rayon threads simultaneously. Requires a concurrent execution type analogous to PureScript's `ParAff` — a deferred task type where `apply` uses `rayon::join`. This conflicts with fp-library's `impl Fn` (not `FnOnce`) applicative model and requires `'static` bounds. For pure `A -> B` functions this reduces to `par_map`, which already exists. **Not currently feasible without major infrastructure.**
- _Practical alternative_: `par_map(f, ta)` (CPU parallel) followed by `sequence_validation` (error accumulation) composes both behaviours without needing a unified `par_traverse`.

**`ParWitherable`** — same two flavours as `ParTraversable`, since `Witherable: Traversable + Filterable`. The error-accumulation flavour (`wither` with `Validation<E, Option<B>>`) is feasible alongside `ParTraversable`. The CPU-parallel effectful flavour has the same obstacles.
