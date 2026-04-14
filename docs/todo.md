### Tasks to do & ideas to look into

- Add issue templates.
- Should the `*Brand` `impl`s in [types/](../fp-library/src/types) be moved into modules in [brands/](../fp-library/src/brands)?
- Should `Coyoneda` types, et. al, be moved to their own submodule? What about other types related to each other (newtype wrappers `Additive`, `Multiplicative`, `Conjunctive`, `Disjunctive`, etc.; `Thunk`, `Trampoline`, `Lazy`, etc.); do these also deserve their own submodules?
- Is it possible to use a combination of [PlugLifetime](https://github.com/Ereski/generic-std), [ForLifetime](https://github.com/danielhenrymantilla/higher-kinded-types.rs), nested curried application of a single `app` from the [LHKP paper](https://web.archive.org/web/20220104164033/https://www.lpw25.net/papers/flops2014.pdf) (would just be `Kind`, in our case), to obviate the need for having a family of `Kind_*` traits, and instead compose kinds from nested curried applications of lifetime and type parameter GAT primitives?
- Algebraic effects/effect system to implement extensible effects
  - [Analysis](plans/effects/effects.md)
  - [Eff](https://github.com/lexi-lambda/eff) [documentation](https://hasura.github.io/eff/Control-Effect.html)
- Write user stories for all types, traits, and modules. Each should have a one-line "I want to..." description explaining when and why a user would reach for it. See `fp-library/docs/coyoneda.md`, `fp-library/docs/lazy-evaluation.md`, and `fp-library/src/types/free.rs` for the pattern. Prioritize types that are easy to confuse with each other (e.g., Thunk vs Trampoline vs Lazy, the four Coyoneda variants, Functor vs RefFunctor vs SendRefFunctor).
- Optics: Implement missing functionality from [analysis](../fp-library/docs/optics-analysis.md).
- Inline `!`-notation within `m_do!`: allow `m_do! { pure(!fa + !fb) }` as shorthand that automatically lifts subexpressions into binds, similar to Idris's `!`-notation. Avoids unnecessary intermediate bindings when a value is used once, immediately. Implement as an incremental enhancement to `m_do!` rather than a standalone feature.
- Property-based tests for type class laws.
  - [Validity](https://github.com/NorfairKing/validity).
  - [Hegel](https://github.com/hegeldev/hegel-rust).
- For each equivalent abstraction in docs/benchmarks/comparisons.md / docs/std_coverage_checklist.md , there should be a property-based test to test for equivalence.
- [Mutation testing](https://github.com/sourcefrog/cargo-mutants).
- [Lazy, memoized data type](https://pursuit.purescript.org/packages/purescript-lazy/3.0.0/docs/Data.Lazy) for [dynamic programming](https://en.wikipedia.org/wiki/Dynamic_programming#Computer_science). (Partially implemented via `Lazy`, `TryLazy`).
- [Monadic stream functions](https://github.com/ivanperez-keera/dunai).
- Software Transactional Memory
  - [Wikipedia](https://en.wikipedia.org/wiki/Software_transactional_memory).
  - [Hackage](https://hackage.haskell.org/package/stm-2.5.3.1/docs/Control-Monad-STM.html).
- Inner vs outer iteration.
- Add benchmark outputs and graphs to repo to make them accessible. Options:
  - **Commit PNGs to a `benchmarks/` directory.** Simple, version-controlled, visible on GitHub. Reference from docs and README. Downside: goes stale unless regenerated before releases.
  - **GitHub Pages with Criterion reports.** Push the full Criterion HTML output to a `gh-pages` branch via CI. Always up to date, interactive charts. More setup.
  - **Separate repo.** Prevents bloating the main repo. Downside: harder to keep in sync with code changes.
  - Regardless of hosting, regenerating graphs should be part of the release process.
- Expand benchmark coverage per [benchmarking/coverage-gaps.md](plans/benchmarking/coverage-gaps.md). Priority order: optics, fallible lazy types, newtype wrappers (zero-cost verification), CatList type class ops, SendThunk/Identity, parallel operations.

### Deferred Ref-hierarchy items

- **SendRef variants for filterable/traversable/witherable**: `SendRefFilterable`, `SendRefTraversable`, `SendRefWitherable`, `SendRefFilterableWithIndex`, `SendRefTraversableWithIndex`. Not needed until a thread-safe memoized type implements filtering or traversal.
- **Par-Ref traits**: Parallel by-reference trait variants (`ParRefFunctor`, `ParRefFoldable`, etc.). Combine rayon parallelism with by-reference element access. Needs collection Ref impls first.

### Parallel type classes

The core parallel traits are implemented: `ParFunctor`, `ParFoldable`, `ParCompactable`, `ParFilterable`, `ParFunctorWithIndex`, `ParFoldableWithIndex`, `ParFilterableWithIndex`.

**`ParTraversable`** (not yet implemented) -two distinct flavours with different feasibility:

- _Error accumulation_ (the `Validation` flavour): `par_traverse(f, ta)` runs all `f(a)` and accumulates all errors, rather than short-circuiting on the first `Err`. Implemented as `traverse::<T, ValidationBrand<E>, _, _>` using `Validation<E, A>`'s accumulating `Applicative`. Requires adding `Validation<E, A>` with its `Semiapplicative::apply` instance; no new HKT machinery beyond that. **Feasible.**
- _CPU parallelism of effectful functions_: run `f: A -> Result<B, E>` on all elements across rayon threads simultaneously. Requires a concurrent execution type analogous to PureScript's `ParAff` -a deferred task type where `apply` uses `rayon::join`. This conflicts with fp-library's `impl Fn` (not `FnOnce`) applicative model and requires `'static` bounds. For pure `A -> B` functions this reduces to `par_map`, which already exists. **Not currently feasible without major infrastructure.**
- _Practical alternative_: `par_map(f, ta)` (CPU parallel) followed by `sequence_validation` (error accumulation) composes both behaviours without needing a unified `par_traverse`.

**`ParWitherable`** -same two flavours as `ParTraversable`, since `Witherable: Traversable + Filterable`. The error-accumulation flavour (`wither` with `Validation<E, Option<B>>`) is feasible alongside `ParTraversable`. The CPU-parallel effectful flavour has the same obstacles.
