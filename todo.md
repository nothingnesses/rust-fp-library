### Todos/ideas to look into

* For each equivalent abstraction in docs/benchmarks/comparisons.md / docs/std_coverage_checklist.md , there should be a property-based test to test for equivalence.
* Figure out a better naming scheme for `Kind` traits to better handle lifetime usage/application and update [the macros](https://github.com/nothingnesses/rust-fp-library/blob/main/fp-library/src/macros.rs) accordingly.
* Add a diagram of the typeclass/trait hierarchy and reasoning/justification for why the current hierarchy is as it is.
* Look into making a Function/ClonableFn trait that doesn't input a lifetime parameter so we can implement Kind0L1T (and, consequently, Functor, etc.) for LazyBrand.
* Serde (de)serialisation for types.
* Paralellisation using rayon.
* Property-based tests for type class laws.
	* [Validity](https://github.com/NorfairKing/validity).
* [Mutation testing](https://github.com/sourcefrog/cargo-mutants).
* [Lazy, memoized data type](https://pursuit.purescript.org/packages/purescript-lazy/3.0.0/docs/Data.Lazy) for [dynamic programming](https://en.wikipedia.org/wiki/Dynamic_programming#Computer_science).
* [`Compactable`, `Filterable`, `Witherable`](https://github.com/reazen/relude/issues/268).
	* [Composable filters using Witherable optics](https://chrispenner.ca/posts/witherable-optics).
	* [purescript-filterable](https://pursuit.purescript.org/packages/purescript-filterable/5.0.0).
* [Monadic stream functions](https://github.com/ivanperez-keera/dunai).

### Questions
* Inner or outer iteration?
* Add benchmark outputs and graphs to repo to make them accessible? Maybe they should be in a separate repo, to prevent bloating this one?