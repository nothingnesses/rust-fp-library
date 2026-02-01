### Tasks to do & ideas to look into

* Better Workflow Example in readme and lib
* Sort order of MonadRec type params
* TryThunk should:
	* Have catch like TryTrampoline
	* Implement HKT classes
* Add extra trait methods for type classes and methods for brand/type structs. Make trait implementations use type methods. E.g., add a map method for Identity, make the Functor implementation use it, like how Functor for Vec uses Iterator map internally.
* Property-based tests for type class laws.
	* [Validity](https://github.com/NorfairKing/validity).
* Add a diagram of the typeclass/trait hierarchy and reasoning/justification for why the current hierarchy is as it is.
* For each equivalent abstraction in docs/benchmarks/comparisons.md / docs/std_coverage_checklist.md , there should be a property-based test to test for equivalence.
* Look into making a Function/CloneableFn trait that doesn't input a lifetime parameter so we can implement Kind0L1T (and, consequently, Functor, etc.) for LazyBrand.
* Serde (de)serialisation for types.
* [Mutation testing](https://github.com/sourcefrog/cargo-mutants).
* [Lazy, memoized data type](https://pursuit.purescript.org/packages/purescript-lazy/3.0.0/docs/Data.Lazy) for [dynamic programming](https://en.wikipedia.org/wiki/Dynamic_programming#Computer_science).
* [Monadic stream functions](https://github.com/ivanperez-keera/dunai).

### Questions
* Inner or outer iteration?
* Add benchmark outputs and graphs to repo to make them accessible? Maybe they should be in a separate repo, to prevent bloating this one?
* Should Lazy have pure and defer?
