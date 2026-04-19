#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![allow(clippy::tabs_in_doc_comments)]

//! # fp-library
//!
//! A functional programming library for Rust featuring your favourite higher-kinded types and type classes.
//!
//! ## Motivation
//!
//! Rust is a multi-paradigm language with strong functional programming features like iterators, closures, and algebraic data types. However, it lacks native support for **Higher-Kinded Types (HKT)**, which limits the ability to write generic code that abstracts over type constructors (e.g., writing a function that works for any `Monad`, whether it's `Option`, `Result`, or `Vec`).
//!
//! `fp-library` aims to bridge this gap by providing:
//!
//! 1.  A robust encoding of HKTs in stable Rust.
//! 2.  A comprehensive set of standard type classes (`Functor`, `Monad`, `Traversable`, etc.).
//! 3.  Zero-cost abstractions that respect Rust's performance characteristics.
//!
//! ## Examples
//!
//! ### Using `Functor` with `Option`
//!
//! The brand is inferred automatically from the container type:
//!
//! ```
//! use fp_library::functions::*;
//!
//! // Brand inferred from Option<i32>
//! let y = map(|i: i32| i * 2, Some(5));
//! assert_eq!(y, Some(10));
//!
//! // Brand inferred from &Vec<i32> (by-reference dispatch)
//! let v = vec![1, 2, 3];
//! let y = map(|i: &i32| *i + 10, &v);
//! assert_eq!(y, vec![11, 12, 13]);
//! ```
//!
//! For types with multiple brands (e.g., `Result`), annotate the closure's input
//! type to disambiguate which brand applies:
//!
//! ```
//! use fp_library::functions::*;
//!
//! // Closure annotation pins A = i32, selecting ResultErrAppliedBrand<&str>
//! let y = map(|i: i32| i * 2, Ok::<i32, &str>(5));
//! assert_eq!(y, Ok(10));
//! ```
//!
//! For diagonal cases (e.g., `Result<T, T>`) where the closure cannot disambiguate,
//! use the `explicit` variant with a turbofish:
//!
//! ```
//! use fp_library::{
//! 	brands::*,
//! 	functions::explicit::*,
//! };
//!
//! let y = map::<ResultErrAppliedBrand<i32>, _, _, _, _>(|i| i * 2, Ok::<i32, i32>(5));
//! assert_eq!(y, Ok(10));
//! ```
//!
//! ### Monadic Do-Notation with `m_do!`
//!
//! The `m_do!` macro provides Haskell/PureScript-style do-notation for flat monadic code.
//! It desugars `<-` binds into nested [`bind`](functions::bind) calls.
//!
//! ```
//! use fp_library::{brands::*, functions::*, m_do};
//!
//! // Inferred mode: brand inferred from container types
//! let result = m_do!({
//! 	x <- Some(5);
//! 	y <- Some(x + 1);
//! 	let z = x * y;
//! 	Some(z)
//! });
//! assert_eq!(result, Some(30));
//!
//! // Explicit mode: for ambiguous types or when pure() is needed
//! let result = m_do!(VecBrand {
//! 	x <- vec![1, 2];
//! 	y <- vec![10, 20];
//! 	pure(x + y)
//! });
//! assert_eq!(result, vec![11, 21, 12, 22]);
//! ```
//! ## Features
//!
//! For a detailed breakdown of all features, type class hierarchies,
//! data types, and macros, see the [Features documentation][crate::docs::features].
//!
//! ## How it Works
//!
//! **Higher-Kinded Types:** The library encodes HKTs using lightweight higher-kinded polymorphism
//! (the "Brand" pattern). Each type constructor has a zero-sized brand type (e.g., `OptionBrand`)
//! that implements `Kind` traits mapping brands back to concrete types.
//! See [Higher-Kinded Types][crate::docs::hkt].
//!
//! **Brand Inference:** A user guide for turbofish-free dispatch. Shows how the compiler infers
//! brands from container types, how closure annotations disambiguate multi-brand types like
//! `Result`, and when to fall back to `explicit::` functions.
//! See [Brand Inference][crate::docs::brand_inference].
//!
//! **Val/Ref Dispatch:** A user guide for unified function dispatch. Each free function routes to
//! either a by-value or by-reference trait method based on the closure's argument type (or container
//! ownership for closureless operations). Dispatch and brand inference compose through the shared
//! `FA` type parameter. See [Val/Ref Dispatch][crate::docs::dispatch].
//!
//! **Brand Dispatch Traits:** The implementer reference for the `Kind` and `InferableBrand` trait
//! families, covering trait shapes, the Marker invariant, inference resolution steps, impl
//! landscape, and coverage matrix.
//! See [Brand Dispatch Traits][crate::docs::brand_dispatch_traits].
//!
//! **Zero-Cost Abstractions:** Core operations use uncurried semantics with `impl Fn` for static
//! dispatch and zero heap allocation. Dynamic dispatch (`dyn Fn`) is reserved for cases where
//! functions must be stored as data.
//! See [Zero-Cost Abstractions][crate::docs::zero_cost].
//!
//! **Lazy Evaluation:** A granular hierarchy of lazy types (`Thunk`, `Trampoline`, `Lazy`) lets you
//! choose trade-offs between stack safety, memoization, lifetimes, and thread safety. Each has a
//! fallible `Try*` counterpart.
//! See [Lazy Evaluation][crate::docs::lazy_evaluation].
//!
//! **Thread Safety & Parallelism:** A parallel trait hierarchy (`ParFunctor`, `ParFoldable`, etc.)
//! mirrors the sequential one. When the `rayon` feature is enabled, `par_*` functions use true
//! parallel execution.
//! See [Thread Safety and Parallelism][crate::docs::parallelism].
//!
//! ## Documentation
//!
//! - [Features & Type Class Hierarchy][crate::docs::features]: Full feature list with hierarchy diagrams.
//! - [Higher-Kinded Types][crate::docs::hkt]: The Brand pattern and HKT encoding.
//! - [Brand Inference][crate::docs::brand_inference]: User guide for turbofish-free dispatch and multi-brand inference.
//! - [Val/Ref Dispatch][crate::docs::dispatch]: User guide for unified by-value and by-reference function dispatch.
//! - [Brand Dispatch Traits][crate::docs::brand_dispatch_traits]: Implementer reference for trait shapes, Marker invariant, and inference resolution.
//! - [Zero-Cost Abstractions][crate::docs::zero_cost]: Uncurried semantics and static dispatch.
//! - [Pointer Abstraction][crate::docs::pointer_abstraction]: Pointer hierarchy, `FnBrand<P>`, and shared memoization.
//! - [Lazy Evaluation][crate::docs::lazy_evaluation]: Guide to the lazy evaluation and memoization types.
//! - [Coyoneda Implementations][crate::docs::coyoneda]: Trade-offs between the four free functor variants.
//! - [Thread Safety & Parallelism][crate::docs::parallelism]: Parallel trait hierarchy and rayon support.
//! - [Limitations and Workarounds][crate::docs::limitations_and_workarounds]: Rust type system constraints and how the library addresses them.
//! - [Project Structure][crate::docs::project_structure]: Module layout and dependency graph.
//! - [Architecture & Design][crate::docs::architecture]: Design decisions and documentation conventions.
//! - [Optics Analysis][crate::docs::optics_analysis]: Optics coverage comparison with PureScript.
//! - [Profunctor Analysis][crate::docs::profunctor_analysis]: Profunctor class hierarchy comparison with PureScript.
//! - [Std Library Coverage][crate::docs::std_coverage_checklist]: Type class coverage for standard library types.
//!-  [Benchmarks](crate::docs::benchmarking): Performance results, graphs, and benchmark coverage.
//!
//! ## Crate Features
//!
//! - **`rayon`**: Enables true parallel execution for `par_*` functions using the [rayon](https://github.com/rayon-rs/rayon) library. Without this feature, `par_*` functions fall back to sequential equivalents.
//! - **`serde`**: Enables serialization and deserialization support for pure data types using the [serde](https://github.com/serde-rs/serde) library.
//! - **`stacker`**: Enables adaptive stack growth for deep `Coyoneda`, `RcCoyoneda`, and `ArcCoyoneda` map chains via the [stacker](https://github.com/rust-lang/stacker) crate. Without this feature, deeply chained maps can overflow the stack.

extern crate fp_macros;
// Allow the proc macro output to reference this crate via the absolute
// path `::fp_library::dispatch::{Val, Ref}`. Without this, `::fp_library`
// only resolves in external crates, not inside fp-library itself.
extern crate self as fp_library;

pub mod brands;
pub mod classes;
pub mod dispatch;
pub mod docs;
pub mod functions;
pub mod kinds;
pub mod types;
pub(crate) mod utils;

pub use fp_macros::*;
