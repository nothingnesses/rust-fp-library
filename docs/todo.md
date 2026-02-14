### Tasks to do & ideas to look into

* `Wander` type class (required for `Traversal`).
* `Closed` type class (required for `Grate`).
* Optics:
	* Full parity with `purescript-profunctor-lenses`.
	* Implement `Prism`, `Iso`, `Traversal`, `Grate`, `Fold`, `Getter`, `Setter`, `Review`.
	* Implement internal profunctors: `Market`, `Shop`, `Forget`, `Exchange`, `Stall`, `Grating`, `Bazaar`.
	* Indexed optics: `IndexedLens`, `IndexedTraversal`, `IndexedFold`, etc.
	* `fp-macros`: `#[derive(Lens)]`, `#[derive(Prism)]`.
* Algebraic effects/effect system to implement extensible effects
	* [Eff](https://github.com/lexi-lambda/eff) [documentation](https://hasura.github.io/eff/Control-Effect.html)
* `Alternative` type class (requires `Plus` and `Applicative`).
* Add extra trait methods for type classes and methods for brand/type structs. Make trait implementations use type methods. E.g., add a map method for Identity, make the Functor implementation use it, like how Functor for Vec uses Iterator map internally.
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
