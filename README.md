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
- **Brand inference:** For types with a single brand (Option, Vec, etc.), the brand is inferred automatically. No turbofish needed: `map(f, Some(5))`. Types with multiple brands use `_explicit` variants.
- **Macros:** `trait_kind!`/`impl_kind!`/`Apply!` for HKT encoding, `m_do!`/`a_do!` for monadic/applicative do-notation (with inferred and explicit brand modes).
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
fp-library = { version = "0.15", features = ["rayon"] }

# Multiple features
fp-library = { version = "0.15", features = ["rayon", "serde"] }
```

### Example: Using `Functor` with `Option`

The brand is inferred automatically from the container type:

```rust
use fp_library::functions::*;

fn main() {
	// Brand inferred from Option<i32>
	let y = map(|i: i32| i * 2, Some(5));
	assert_eq!(y, Some(10));

	// Brand inferred from &Vec<i32> (by-reference dispatch)
	let v = vec![1, 2, 3];
	let y = map(|i: &i32| *i + 10, &v);
	assert_eq!(y, vec![11, 12, 13]);
}
```

For types with multiple brands (e.g., `Result`), use the `_explicit` variant:

```rust
use fp_library::{brands::*, functions::*};

fn main() {
	let y = map_explicit::<ResultErrAppliedBrand<&str>, _, _, _, _>(|i| i * 2, Ok::<i32, &str>(5));
	assert_eq!(y, Ok(10));
}
```

### Example: Monadic Do-Notation with `m_do!`

The `m_do!` macro provides Haskell/PureScript-style do-notation for flat monadic code.
It desugars `<-` binds into nested `bind` calls.

```rust
use fp_library::{brands::*, functions::*, m_do};

fn main() {
	// Inferred mode: brand inferred from container types
	let result = m_do!({
		x <- Some(5);
		y <- Some(x + 1);
		let z = x * y;
		Some(z)
	});
	assert_eq!(result, Some(30));

	// Explicit mode: for ambiguous types or when pure() is needed
	let result = m_do!(VecBrand {
		x <- vec![1, 2];
		y <- vec![10, 20];
		pure(x + y)
	});
	assert_eq!(result, vec![11, 21, 12, 22]);
}
```

## How it Works

**Higher-Kinded Types:** The library encodes HKTs using lightweight higher-kinded polymorphism (the "Brand" pattern). Each type constructor has a zero-sized brand type (e.g., `OptionBrand`) that implements `Kind` traits mapping brands back to concrete types. See [hkt.md](fp-library/docs/hkt.md).

**Zero-Cost Abstractions:** Core operations use uncurried semantics with `impl Fn` for static dispatch and zero heap allocation. Dynamic dispatch (`dyn Fn`) is reserved for cases where functions must be stored as data. See [zero-cost.md](fp-library/docs/zero-cost.md).

**Lazy Evaluation:** A granular hierarchy of lazy types (`Thunk`, `Trampoline`, `Lazy`) lets you choose trade-offs between stack safety, memoization, lifetimes, and thread safety. Each has a fallible `Try*` counterpart. See [lazy-evaluation.md](fp-library/docs/lazy-evaluation.md).

**Thread Safety & Parallelism:** A parallel trait hierarchy (`ParFunctor`, `ParFoldable`, etc.) mirrors the sequential one. When the `rayon` feature is enabled, `par_*` functions use true parallel execution. See [parallelism.md](fp-library/docs/parallelism.md).

## Documentation

- [API Documentation](https://docs.rs/fp-library): The complete API reference on docs.rs.
- [Features & Type Class Hierarchy](fp-library/docs/features.md): Full feature list with hierarchy diagrams.
- [Higher-Kinded Types](fp-library/docs/hkt.md): The Brand pattern and HKT encoding.
- [Zero-Cost Abstractions](fp-library/docs/zero-cost.md): Uncurried semantics and static dispatch.
- [Lazy Evaluation](fp-library/docs/lazy-evaluation.md): Guide to the lazy evaluation and memoization types.
- [Pointer Abstraction](fp-library/docs/pointer-abstraction.md): Pointer hierarchy, `FnBrand<P>`, and shared memoization.
- [Coyoneda Implementations](fp-library/docs/coyoneda.md): Trade-offs between the four free functor variants.
- [Thread Safety & Parallelism](fp-library/docs/parallelism.md): Parallel trait hierarchy and rayon support.
- [Optics Analysis](fp-library/docs/optics-analysis.md): Optics coverage comparison with PureScript.
- [Profunctor Analysis](fp-library/docs/profunctor-analysis.md): Profunctor class hierarchy comparison with PureScript.
- [Std Library Coverage](fp-library/docs/std-coverage-checklist.md): Type class coverage for standard library types.
- [Architecture & Design](fp-library/docs/architecture.md): Module organization and documentation conventions.
- [Benchmarks](fp-library/docs/benchmarking.md): Performance results, graphs, and benchmark coverage.
- [Limitations and Workarounds](fp-library/docs/limitations-and-workarounds.md): Rust type system constraints and how the library addresses them.

## Contributing

We welcome contributions! Please feel free to submit a [Pull Request](https://github.com/nothingnesses/rust-fp-library/compare).

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
just fmt     # Format all files (Rust, Nix, Markdown, YAML, TOML)
just clippy  # Run clippy
just test    # Run all tests (cached; only re-runs when content changes)
just doc     # Build docs (must produce zero warnings)
just verify  # Run fmt, check, clippy, doc, test in order
just clean   # Remove build artifacts and test cache
```

Run `just --list` to see all available recipes.

### Project Structure

- `fp-library/src/brands`: Zero-sized brand marker types for HKT encoding.
- `fp-library/src/kinds`: Kind traits, InferableBrand traits, and type application machinery.
- `fp-library/src/classes`: Type class trait definitions (Functor, Monad, Foldable, etc.).
- `fp-library/src/dispatch`: Val/Ref dispatch traits routing to by-value or by-reference trait methods.
- `fp-library/src/functions`: Inference-based free function wrappers (the primary user API).
- `fp-library/src/types`: Concrete type implementations of type classes.
- `fp-macros`: Procedural macros for HKT traits, do-notation, and documentation generation.

### Release Process

For maintainers, the release process is documented in [release-process.md](fp-library/docs/release-process.md).

### Benchmarking

This project uses [Criterion.rs](https://github.com/criterion-rs/criterion.rs) for benchmarking to ensure zero-cost abstractions and detect performance regressions.

```sh
just bench -p fp-library                               # To run all benchmarks
just bench -p fp-library --bench benchmarks -- --list  # To list available benchmarks
just bench -p fp-library --bench benchmarks -- Vec     # To run a specific benchmark (e.g., `Vec`)
```

Benchmark reports are generated in `target/criterion/report/index.html`.

## License

This project is licensed under the [Blue Oak Model License 1.0.0](LICENSE).

## References

- [Lightweight higher-kinded polymorphism](https://web.archive.org/web/20220104164033/https://www.lpw25.net/papers/flops2014.pdf)
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
- [haskell_bits](https://github.com/clintonmead/haskell_bits)
