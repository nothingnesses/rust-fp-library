# fp-library

[![crates.io](https://img.shields.io/crates/v/fp-library.svg)](https://crates.io/crates/fp-library)
[![docs.rs](https://docs.rs/fp-library/badge.svg)](https://docs.rs/fp-library)
[![GitHub License](https://img.shields.io/github/license/nothingnesses/rust-fp-library?color=blue)](https://github.com/nothingnesses/rust-fp-library/blob/main/LICENSE)


A functional programming library for Rust featuring your favourite higher-kinded types and type classes.

## Usage

### Zero-Cost Abstractions (v2)

The library provides zero-cost, uncurried type classes in the `v2` module.

#### Functor

```rust
use fp_library::v2::classes::functor::map;
use fp_library::brands::OptionBrand;

let x = Some(5);
let y = map::<OptionBrand, _, _, _>(|i| i * 2, x);
assert_eq!(y, Some(10));
```

#### Monad

```rust
use fp_library::v2::classes::semimonad::bind;
use fp_library::brands::OptionBrand;

let x = Some(5);
let y = bind::<OptionBrand, _, _, _>(x, |i| Some(i * 2));
assert_eq!(y, Some(10));
```

#### Traversable

```rust
use fp_library::v2::classes::traversable::traverse;
use fp_library::brands::{OptionBrand, VecBrand};

let x = vec![1, 2, 3];
let y = traverse::<VecBrand, OptionBrand, _, _, _>(|i| Some(i * 2), x);
assert_eq!(y, Some(vec![2, 4, 6]));
```

## References
* [Lightweight higher-kinded polymorphism](https://www.cl.cam.ac.uk/~jdy22/papers/lightweight-higher-kinded-polymorphism.pdf)
* [Typeclassopedia](https://wiki.haskell.org/Typeclassopedia)
* [Lean Mathlib Prelude](https://leanprover-community.github.io/mathlib4_docs/Init/Prelude.html)
* [PureScript Pursuit](https://pursuit.purescript.org/)
* [Haskell base package Prelude](https://hackage.haskell.org/package/base-4.21.0.0/docs/Prelude.html)
* [PureScript Typeclass Hierarchy](https://jordanmartinez.github.io/purescript-jordans-reference-site/content/91-Type-Classes/index.html)
* [Where to find theoretical background (i.e., resources) behind PureScript classes?](https://discourse.purescript.org/t/where-to-find-theoretical-background-i-e-resources-behind-purescript-classes/535)
* [Counterexamples of Type Classes](https://blog.functorial.com/posts/2015-12-06-Counterexamples.html)
* [Haskell semigroupoids package](https://github.com/ekmett/semigroupoids)
	* [Class names](https://github.com/ekmett/semigroupoids/issues/26)
* [Why not Pointed?](https://wiki.haskell.org/Why_not_Pointed%3F)
* [Pluggable lifetimes](https://docs.rs/generic-std/latest/generic_std/plug/trait.PlugLifetime.html)