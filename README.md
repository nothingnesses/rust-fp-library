# fp-library

[![crates.io](https://img.shields.io/crates/v/fp-library.svg)](https://crates.io/crates/fp-library)
[![docs.rs](https://docs.rs/fp-library/badge.svg)](https://docs.rs/fp-library)
[![GitHub License](https://img.shields.io/github/license/nothingnesses/rust-fp-library?color=blue)](https://github.com/nothingnesses/rust-fp-library/blob/main/LICENSE)

A functional programming library for Rust featuring your favourite higher-kinded types and type classes.

## At a Glance

- HKT emulation in stable Rust via the Brand pattern.
- Type class hierarchy inspired by Haskell / PureScript.
- Brand inference: `map(|x| x + 1, Some(5))` with no turbofish needed.
- Val/Ref dispatch: one function handles both owned and borrowed containers.
- Zero-cost abstractions (no runtime overhead).
- Works with `std` types (`Option`, `Result`, `Vec`, etc.).
- Advanced features: optics, lazy evaluation, parallel traits.

## Motivation

Rust is a multi-paradigm language with strong functional programming features like iterators, closures, and algebraic data types. However, it lacks native support for **Higher-Kinded Types (HKT)**, which limits the ability to write generic code that abstracts over type constructors (e.g., writing a function that works for any `Monad`, whether it's `Option`, `Result`, or `Vec`).

`fp-library` aims to bridge this gap by providing:

1.  A robust encoding of HKTs in stable Rust.
2.  A comprehensive set of standard type classes (`Functor`, `Monad`, `Traversable`, etc.).
3.  Zero-cost abstractions that respect Rust's performance characteristics.

## Examples

### Using `Functor` with `Option`

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

For types with multiple brands (e.g., `Result`), use the `explicit` variant:

```rust
use fp_library::{brands::*, functions::explicit::*};

fn main() {
	let y = map::<ResultErrAppliedBrand<&str>, _, _, _, _>(|i| i * 2, Ok::<i32, &str>(5));
	assert_eq!(y, Ok(10));
}
```

### Monadic Do-Notation with `m_do!`

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

## Usage

Add `fp-library` to your `Cargo.toml`:

```toml
[dependencies]
fp-library = "0.15"
```

## Features

For a detailed breakdown of all features, type class hierarchies (with Mermaid diagrams),
data types, and macros, see the [Features documentation](fp-library/docs/features.md).

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

## How it Works

**Higher-Kinded Types:** The library encodes HKTs using lightweight higher-kinded polymorphism (the "Brand" pattern). Each type constructor has a zero-sized brand type (e.g., `OptionBrand`) that implements `Kind` traits mapping brands back to concrete types. See [Higher-Kinded Types](fp-library/docs/hkt.md).

**Brand Inference:** `InferableBrand` traits provide the reverse mapping (concrete type -> brand), letting the compiler infer brands automatically. `trait_kind!` and `impl_kind!` generate both mappings. See [Brand Inference](fp-library/docs/brand-inference.md).

**Val/Ref Dispatch:** Each free function routes to either a by-value or by-reference trait method based on the closure's argument type (or container ownership for closureless operations). Dispatch and brand inference compose through the shared `FA` type parameter. See [Val/Ref Dispatch](fp-library/docs/dispatch.md).

**Zero-Cost Abstractions:** Core operations use uncurried semantics with `impl Fn` for static dispatch and zero heap allocation. Dynamic dispatch (`dyn Fn`) is reserved for cases where functions must be stored as data. See [Zero-Cost Abstractions](fp-library/docs/zero-cost.md).

**Lazy Evaluation:** A granular hierarchy of lazy types (`Thunk`, `Trampoline`, `Lazy`) lets you choose trade-offs between stack safety, memoization, lifetimes, and thread safety. Each has a fallible `Try*` counterpart. See [Lazy Evaluation](fp-library/docs/lazy-evaluation.md).

**Thread Safety & Parallelism:** A parallel trait hierarchy (`ParFunctor`, `ParFoldable`, etc.) mirrors the sequential one. When the `rayon` feature is enabled, `par_*` functions use true parallel execution. See [Thread Safety and Parallelism](fp-library/docs/parallelism.md).

## Documentation

- [API Documentation](https://docs.rs/fp-library): The complete API reference on docs.rs.
- [Features & Type Class Hierarchy](fp-library/docs/features.md): Full feature list with hierarchy diagrams.
- [Higher-Kinded Types](fp-library/docs/hkt.md): The Brand pattern and HKT encoding.
- [Brand Inference](fp-library/docs/brand-inference.md): How InferableBrand eliminates turbofish for common types.
- [Val/Ref Dispatch](fp-library/docs/dispatch.md): How unified free functions route to by-value or by-reference trait methods.
- [Zero-Cost Abstractions](fp-library/docs/zero-cost.md): Uncurried semantics and static dispatch.
- [Pointer Abstraction](fp-library/docs/pointer-abstraction.md): Pointer hierarchy, `FnBrand<P>`, and shared memoization.
- [Lazy Evaluation](fp-library/docs/lazy-evaluation.md): Guide to the lazy evaluation and memoization types.
- [Coyoneda Implementations](fp-library/docs/coyoneda.md): Trade-offs between the four free functor variants.
- [Thread Safety & Parallelism](fp-library/docs/parallelism.md): Parallel trait hierarchy and rayon support.
- [Limitations and Workarounds](fp-library/docs/limitations-and-workarounds.md): Rust type system constraints and how the library addresses them.
- [Project Structure](fp-library/docs/project-structure.md): Module layout and dependency graph.
- [Architecture & Design](fp-library/docs/architecture.md): Design decisions and documentation conventions.
- [Optics Analysis](fp-library/docs/optics-analysis.md): Optics coverage comparison with PureScript.
- [Profunctor Analysis](fp-library/docs/profunctor-analysis.md): Profunctor class hierarchy comparison with PureScript.
- [Std Library Coverage](fp-library/docs/std-coverage-checklist.md): Type class coverage for standard library types.
- [Benchmarks](fp-library/docs/benchmarking.md): Performance results, graphs, and benchmark coverage.

## Contributing

We welcome contributions!

To get started:

- Check out our [Contributing Guide](CONTRIBUTING.md) for environment setup and development workflows.
- Read the [documentation files](#documentation) to get a high-level understanding of the project.
- Join the conversation in [GitHub Issues](https://github.com/nothingnesses/rust-fp-library/issues).

Please ensure all PRs pass `just verify` before submission.

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
