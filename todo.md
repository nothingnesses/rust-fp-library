### Todos/ideas to look into

* Widen the blanket implementation for `Applicative` to allow `Vec` to implement `Traversable`.
* Property-based tests for typeclass laws.
	* [Validity](https://github.com/NorfairKing/validity).
* [Mutation testing](https://github.com/sourcefrog/cargo-mutants).
* [Lazy, memoized data type](https://pursuit.purescript.org/packages/purescript-lazy/3.0.0/docs/Data.Lazy) for [dynamic programming](https://en.wikipedia.org/wiki/Dynamic_programming#Computer_science).
* Reference-counted typeclass that can be implemented by `Rc` and `Arc` so functions can input or output either.
* Typeclass with `get` and `set` methods for types that can hold values.
* [Pluggable lifetimes](https://docs.rs/generic-std/latest/generic_std/plug/trait.PlugLifetime.html) - might obviate the need to add explicit lifetimes to `Semigroup`.
* [`Compactable`, `Filterable`, `Witherable`](https://github.com/reazen/relude/issues/268).
	* [Composable filters using Witherable optics](https://chrispenner.ca/posts/witherable-optics).
	* [purescript-filterable](https://pursuit.purescript.org/packages/purescript-filterable/5.0.0).
* [Monadic stream functions](https://github.com/ivanperez-keera/dunai).

### Questions
* Inner or outer iteration?
* What's the benefit of [couch/lifted technique of using unsafe references](https://git.sr.ht/~couch/lifted/tree/753f7762cf7ce589f945dcd56bdf59c1d59184aa/item/src/lib.rs#L219-255) over `inject` and `project` methods?
