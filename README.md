# fp-library

[![crates.io](https://img.shields.io/crates/v/fp-library.svg)](https://crates.io/crates/fp-library)
[![docs.rs](https://docs.rs/fp-library/badge.svg)](https://docs.rs/fp-library)
[![GitHub License](https://img.shields.io/github/license/nothingnesses/rust-fp-library?color=blue)](https://github.com/nothingnesses/rust-fp-library/blob/main/LICENSE)

A functional programming library for Rust featuring your favourite higher-kinded types and type classes.

## Features

- **Higher-Kinded Types** via lightweight higher-kinded polymorphism (brand pattern). Write generic code over `Functor`, `Monad`, `Traversable`, etc. that works with `Option`, `Result`, `Vec`, or your own types.
- **Type classes** covering the standard FP hierarchy: `Functor` through `Monad`, `Foldable`/`Traversable`, `Alt`/`Alternative`, `Comonad`, `Bifunctor`, `Filterable`/`Witherable`, indexed variants, and parallel counterparts.
- **Profunctor optics** (Lens, Prism, Iso, Traversal, AffineTraversal, Getter, Setter, Fold, Review, Grate) with zero-cost composition and indexed variants. Port of PureScript's `purescript-profunctor-lenses`.
- **Lazy evaluation types** with explicit trade-offs: `Thunk` (lightweight), `Trampoline` (stack-safe), `Lazy` (memoized), each with `Send` and fallible (`Try*`) variants.
- **Free functors** (`Coyoneda`, `RcCoyoneda`, `ArcCoyoneda`, `CoyonedaExplicit`) for deferred mapping with different cloning, threading, and fusion trade-offs.
- **Macros:** `trait_kind!`/`impl_kind!`/`Apply!` for HKT encoding, `m_do!` for monadic do-notation, `a_do!` for applicative do-notation.
- **Numeric algebra:** `Semiring`, `Ring`, `EuclideanRing`, `Field`, `HeytingAlgebra`.
- **Zero-cost abstractions:** Uncurried semantics with `impl Fn` for static dispatch. Dynamic dispatch reserved for functions-as-data.
- **Thread safety:** Parallel trait hierarchy (`ParFunctor`, `ParFoldable`, etc.) with optional `rayon` support.

## Motivation

Rust is a multi-paradigm language with strong functional programming features like iterators, closures, and algebraic data types. However, it lacks native support for **Higher-Kinded Types (HKT)**, which limits the ability to write generic code that abstracts over type constructors (e.g., writing a function that works for any `Monad`, whether it's `Option`, `Result`, or `Vec`).

`fp-library` aims to bridge this gap by providing:

1.  A robust encoding of HKTs in stable Rust.
2.  A comprehensive set of standard type classes (`Functor`, `Monad`, `Traversable`, etc.).
3.  Zero-cost abstractions that respect Rust's performance characteristics.

## Usage

Add `fp-library` to your `Cargo.toml`:

```toml
[dependencies]
fp-library = "0.14"
```

### Crate Features

The library offers optional features that can be enabled in your `Cargo.toml`:

- **`rayon`**: Enables true parallel execution for `par_*` functions using the [rayon](https://github.com/rayon-rs/rayon) library. Without this feature, `par_*` functions fall back to sequential equivalents.
- **`serde`**: Enables serialization and deserialization support for pure data types using the [serde](https://github.com/serde-rs/serde) library.
- **`stacker`**: Enables adaptive stack growth for deep `Coyoneda`, `RcCoyoneda`, and `ArcCoyoneda` map chains via the [stacker](https://github.com/rust-lang/stacker) crate. Without this feature, deeply chained maps can overflow the stack.

To enable features:

```toml
[dependencies]
# Single feature
fp-library = { version = "0.14", features = ["rayon"] }

# Multiple features
fp-library = { version = "0.14", features = ["rayon", "serde"] }
```

### Example: Using `Functor` with `Option`

```rust
use fp_library::{brands::*, functions::*};

fn main() {
	let x = Some(5);
	// Map a function over the `Option` using the `Functor` type class
	let y = map::<OptionBrand, _, _>(|i| i * 2, x);
	assert_eq!(y, Some(10));
}
```

### Example: Monadic Do-Notation with `m_do!`

The `m_do!` macro provides Haskell/PureScript-style do-notation for flat monadic code.
It desugars `<-` binds into nested `bind` calls.

```rust
use fp_library::{brands::*, functions::*};
use fp_macros::m_do;

fn main() {
	let result = m_do!(OptionBrand {
		x <- Some(5);
		y <- Some(x + 1);
		let z = x * y;
		pure(z)
	});
	assert_eq!(result, Some(30));

	// Works with any monad brand
	let result = m_do!(VecBrand {
		x <- vec![1, 2];
		y <- vec![10, 20];
		pure(x + y)
	});
	assert_eq!(result, vec![11, 21, 12, 22]);
}
```

## How it Works

**Higher-Kinded Types:** The library encodes HKTs using lightweight higher-kinded polymorphism (the "Brand" pattern). Each type constructor has a zero-sized brand type (e.g., `OptionBrand`) that implements `Kind` traits mapping brands back to concrete types. See [docs/hkt.md](docs/hkt.md).

**Zero-Cost Abstractions:** Core operations use uncurried semantics with `impl Fn` for static dispatch and zero heap allocation. Dynamic dispatch (`dyn Fn`) is reserved for cases where functions must be stored as data. See [docs/zero-cost.md](docs/zero-cost.md).

**Lazy Evaluation:** A granular hierarchy of lazy types (`Thunk`, `Trampoline`, `Lazy`) lets you choose trade-offs between stack safety, memoization, lifetimes, and thread safety. Each has a fallible `Try*` counterpart. See [docs/lazy-evaluation.md](docs/lazy-evaluation.md).

**Thread Safety & Parallelism:** A parallel trait hierarchy (`ParFunctor`, `ParFoldable`, etc.) mirrors the sequential one. When the `rayon` feature is enabled, `par_*` functions use true parallel execution. See [docs/parallelism.md](docs/parallelism.md).

## Documentation

- [API Documentation](https://docs.rs/fp-library): The complete API reference on docs.rs.
- [Features & Type Class Hierarchy](docs/features.md): Full feature list with hierarchy diagrams.
- [Higher-Kinded Types](docs/hkt.md): The Brand pattern and HKT encoding.
- [Zero-Cost Abstractions](docs/zero-cost.md): Uncurried semantics and static dispatch.
- [Lazy Evaluation](docs/lazy-evaluation.md): Guide to the lazy evaluation and memoization types.
- [Pointer Abstraction](docs/pointer-abstraction.md): Pointer hierarchy, `FnBrand<P>`, and shared memoization.
- [Coyoneda Implementations](docs/coyoneda.md): Trade-offs between the four free functor variants.
- [Thread Safety & Parallelism](docs/parallelism.md): Parallel trait hierarchy and rayon support.
- [Optics Analysis](docs/optics-analysis.md): Optics coverage comparison with PureScript.
- [Profunctor Analysis](docs/profunctor-analysis.md): Profunctor class hierarchy comparison with PureScript.
- [Std Library Coverage](docs/std-coverage-checklist.md): Type class coverage for standard library types.
- [Architecture & Design](docs/architecture.md): Module organization and documentation conventions.
- [Limitations](docs/limitations.md): Details all current limitations.

## Contributing

We welcome contributions! Please feel free to submit a Pull Request.

### Development Environment

This project uses [Nix](https://nixos.org/) to manage the development environment.

1.  Install [Nix Package Manager](https://nixos.org/download/).
2.  Install [nix-direnv](https://github.com/nix-community/nix-direnv) (recommended) for automatic environment loading.

To set up the environment:

```sh
# If using direnv
direnv allow

# Or manually enter the shell
nix develop
```

This will provide a shell with the correct Rust version and dependencies.

### Building and Testing

All commands are run via [just](https://github.com/casey/just) recipes defined in the project's `justfile`. Never run `cargo` directly; the `justfile` handles Nix environment setup automatically.

```sh
just fmt                                       # Format all files (Rust, Nix, Markdown, YAML, TOML)
just clippy --workspace --all-features         # Run clippy
just test --all-features                       # Run all tests (cached; only re-runs when source changes)
just doc --workspace --all-features --no-deps  # Build docs (must produce zero warnings)
just verify                                    # Run fmt, clippy, doc, test in order
```

Run `just --list` to see all available recipes.

### Project Structure

- `fp-library/src/classes`: Contains the definitions of type classes (traits).
- `fp-library/src/types`: Contains implementations of type classes for various data types.
- `fp-library/src/kinds`: Contains the machinery for higher-kinded types.
- `fp-library/src/brands`: Contains type brands used for HKT encoding.
- `fp-library/src/functions`: Contains general helper functions.
- `fp-macros`: Procedural macros for generating HKT traits and implementations.

### Release Process

For maintainers, the release process is documented in [docs/release-process.md](docs/release-process.md).

### Benchmarking

This project uses [Criterion.rs](https://github.com/criterion-rs/criterion.rs) for benchmarking to ensure zero-cost abstractions and detect performance regressions.

To run all benchmarks:

```sh
just bench -p fp-library
```

To list available benchmarks:

```sh
just bench -p fp-library --bench benchmarks -- --list
```

To run a specific benchmark (e.g., `Vec`):

```sh
just bench -p fp-library --bench benchmarks -- Vec
```

Benchmark reports are generated in `target/criterion/report/index.html`.

## License

This project is licensed under the [Blue Oak Model License 1.0.0](LICENSE).

## References

- [Lightweight higher-kinded polymorphism](https://www.cl.cam.ac.uk/~jdy22/papers/lightweight-higher-kinded-polymorphism.pdf)
- [Typeclassopedia](https://wiki.haskell.org/Typeclassopedia)
- [Lean Mathlib Prelude](https://leanprover-community.github.io/mathlib4_docs/Init/Prelude.html)
- [PureScript Pursuit](https://pursuit.purescript.org/)
- [Haskell base package Prelude](https://hackage.haskell.org/package/base-4.21.0.0/docs/Prelude.html)
- [PureScript Typeclass Hierarchy](https://jordanmartinez.github.io/purescript-jordans-reference-site/content/91-Type-Classes/index.html)
- [Where to find theoretical background (i.e., resources) behind PureScript classes?](https://discourse.purescript.org/t/where-to-find-theoretical-background-i-e-resources-behind-purescript-classes/535)
- [Counterexamples of Type Classes](https://blog.functorial.com/posts/2015-12-06-Counterexamples.html)
- [Haskell semigroupoids package](https://github.com/ekmett/semigroupoids)
  - [Class names](https://github.com/ekmett/semigroupoids/issues/26)
- [Why not Pointed?](https://wiki.haskell.org/Why_not_Pointed%3F)
- [Pluggable lifetimes](https://docs.rs/generic-std/latest/generic_std/plug/trait.PlugLifetime.html)
- [Scala Cats](https://typelevel.org/cats/)
