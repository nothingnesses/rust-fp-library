### Tasks to do & ideas to look into

* Optics: Implement missing functionality from [analysis](optics-analysis.md).
* Algebraic effects/effect system to implement extensible effects
	* [Eff](https://github.com/lexi-lambda/eff) [documentation](https://hasura.github.io/eff/Control-Effect.html)
* `Alternative` type class (requires `Plus` and `Applicative`).
* Kleisli composition (`compose_kleisli`, `>=>` equivalent). Composes monadic functions `A -> F<B>` and `B -> F<C>` into `A -> F<C>` without explicit `bind` threading. Enables point-free monadic pipelines and reusable monadic function building blocks.
* `do!` macro. Desugars sequential monadic binds from flat syntax into nested `bind` calls, e.g. `do! { x <- fa; y <- g(x); pure(x + y) }` becomes `bind(fa, |x| bind(g(x), |y| pure(x + y)))`. Eliminates rightward drift from deeply nested closures.
* Property-based tests for type class laws.
	* [Validity](https://github.com/NorfairKing/validity).
* Add a diagram of the typeclass/trait hierarchy and reasoning/justification for why the current hierarchy is as it is.
* For each equivalent abstraction in docs/benchmarks/comparisons.md / docs/std_coverage_checklist.md , there should be a property-based test to test for equivalence.
* [Mutation testing](https://github.com/sourcefrog/cargo-mutants).
* [Lazy, memoized data type](https://pursuit.purescript.org/packages/purescript-lazy/3.0.0/docs/Data.Lazy) for [dynamic programming](https://en.wikipedia.org/wiki/Dynamic_programming#Computer_science). (Partially implemented via `Lazy`, `TryLazy`).
* [Monadic stream functions](https://github.com/ivanperez-keera/dunai).

### Questions
* Inner or outer iteration?
* Add benchmark outputs and graphs to repo to make them accessible? Maybe they should be in a separate repo, to prevent bloating this one?
