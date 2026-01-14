### Tasks to do & ideas to look into

* Replace `Output` with `Of` so it makes more ergonomic sense. Add comment about how `Of` is actually just represents the output of the type-level application.
* Replace use of `Apply` aliases with `Apply!` in repro_macro.rs. Delete repro_macro.rs after.
* [`Compactable`, `Filterable`, `Witherable`](https://github.com/reazen/relude/issues/268).
	* [Composable filters using Witherable optics](https://chrispenner.ca/posts/witherable-optics).
	* [purescript-filterable](https://pursuit.purescript.org/packages/purescript-filterable/5.0.0).
* Property-based tests for type class laws.
	* [Validity](https://github.com/NorfairKing/validity).
* Add a diagram of the typeclass/trait hierarchy and reasoning/justification for why the current hierarchy is as it is.
* For each equivalent abstraction in docs/benchmarks/comparisons.md / docs/std_coverage_checklist.md , there should be a property-based test to test for equivalence.
* Look into making a Function/ClonableFn trait that doesn't input a lifetime parameter so we can implement Kind0L1T (and, consequently, Functor, etc.) for LazyBrand.
* Serde (de)serialisation for types.
* Paralellisation using rayon.
* [Mutation testing](https://github.com/sourcefrog/cargo-mutants).
* [Lazy, memoized data type](https://pursuit.purescript.org/packages/purescript-lazy/3.0.0/docs/Data.Lazy) for [dynamic programming](https://en.wikipedia.org/wiki/Dynamic_programming#Computer_science).
* [Monadic stream functions](https://github.com/ivanperez-keera/dunai).

### Questions
* Inner or outer iteration?
* Add benchmark outputs and graphs to repo to make them accessible? Maybe they should be in a separate repo, to prevent bloating this one?