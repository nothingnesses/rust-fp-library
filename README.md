# fp-library

[![crates.io](https://img.shields.io/crates/v/fp-library.svg)](https://crates.io/crates/fp-library)
[![docs.rs](https://docs.rs/fp-library/badge.svg)](https://docs.rs/fp-library)
[![GitHub License](https://img.shields.io/github/license/nothingnesses/rust-fp-library?color=blue)](https://github.com/nothingnesses/rust-fp-library/blob/main/LICENSE)

A functional programming library for Rust featuring your favourite higher-kinded types and type classes.

## At a Glance

- HKT emulation in stable Rust via type-level defunctionalization.
- Type class hierarchy inspired by PureScript / Haskell (`Functor`, `Monad`, `Foldable`, etc.).
- Brand inference: `map(|x| x + 1, Some(5))` with no turbofish needed.
- Val/Ref dispatch: one function handles both owned and borrowed containers.
- Zero-cost core operations (map, bind, fold, etc.) via static dispatch.
- Works with `std` types (`Option`, `Result`, `Vec`, etc.).
- Advanced features: optics, lazy evaluation, parallel traits.

## Motivation

Rust is a multi-paradigm language with strong functional programming features like iterators, closures, and algebraic data types. However, it lacks native support for **Higher-Kinded Types (HKT)**, which limits the ability to write generic code that abstracts over type constructors (e.g., writing a function that works for any `Monad`, whether it's `Option`, `Result`, or `Vec`). `fp-library` aims to bridge this gap.

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

For types with multiple brands (e.g., `Result`, which can be viewed as a functor over
either its `Ok` or `Err` type), use the `explicit` variant to select the brand:

```rust
use fp_library::{brands::*, functions::explicit::*};

fn main() {
	// ResultErrAppliedBrand fixes the error type, so map operates on the Ok value.
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
fp-library = "0.16"
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
fp-library = { version = "0.16", features = ["rayon"] }

# Multiple features
fp-library = { version = "0.16", features = ["rayon", "serde"] }
```

## How it Works

**Higher-Kinded Types:** The library encodes HKTs using lightweight higher-kinded polymorphism (the "Brand" pattern). Each type constructor has a zero-sized brand type (e.g., `OptionBrand`) that implements `Kind` traits mapping brands back to concrete types. See [Higher-Kinded Types](fp-library/docs/hkt.md).

**Dispatch System:** Free functions like `map` and `bind` infer the brand from the container type and route to by-value or by-reference trait methods automatically, so most call sites need no turbofish. For details, see [Brand Inference](fp-library/docs/brand-inference.md), [Val/Ref Dispatch](fp-library/docs/dispatch.md), and [Brand Dispatch Traits](fp-library/docs/brand-dispatch-traits.md).

**Zero-Cost Abstractions:** Core operations use uncurried semantics with `impl Fn` for static dispatch and zero heap allocation. Dynamic dispatch (`dyn Fn`) is reserved for cases where functions must be stored as data. See [Zero-Cost Abstractions](fp-library/docs/zero-cost.md).

**Lazy Evaluation:** A granular hierarchy of lazy types (`Thunk`, `Trampoline`, `Lazy`) lets you choose trade-offs between stack safety, memoization, lifetimes, and thread safety. Each has a fallible `Try*` counterpart. See [Lazy Evaluation](fp-library/docs/lazy-evaluation.md).

**Thread Safety & Parallelism:** A parallel trait hierarchy (`ParFunctor`, `ParFoldable`, etc.) mirrors the sequential one. When the `rayon` feature is enabled, `par_*` functions use true parallel execution. See [Thread Safety and Parallelism](fp-library/docs/parallelism.md).

## Documentation

- [API Documentation](https://docs.rs/fp-library): The complete API reference on docs.rs.
- [Features & Type Class Hierarchy](fp-library/docs/features.md): Full feature list with hierarchy diagrams.
- [Higher-Kinded Types](fp-library/docs/hkt.md): The Brand pattern and HKT encoding.
- [Brand Inference](fp-library/docs/brand-inference.md): User guide for turbofish-free dispatch and multi-brand inference.
- [Val/Ref Dispatch](fp-library/docs/dispatch.md): User guide for unified by-value and by-reference function dispatch.
- [Brand Dispatch Traits](fp-library/docs/brand-dispatch-traits.md): Implementer reference for trait shapes, Marker invariant, and inference resolution.
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
- [References](fp-library/docs/references.md): Papers, libraries, and resources that informed this project.

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

See [References](fp-library/docs/references.md) for papers, libraries, and other resources that informed this project.
