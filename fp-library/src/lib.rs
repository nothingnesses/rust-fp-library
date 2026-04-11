#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![allow(clippy::tabs_in_doc_comments)]

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
#![doc = include_str!("../docs/features.md")]
//!
//! ## How it Works
#![doc = include_str!("../docs/hkt.md")]
#![doc = include_str!("../docs/zero-cost.md")]
#![doc = include_str!("../docs/lazy-evaluation.md")]
#![doc = include_str!("../docs/parallelism.md")]
//!
//! ## Example: Using `Functor` with `Option`
//!
//! ```
//! use fp_library::{
//! 	brands::*,
//! 	functions::*,
//! };
//!
//! let x = Some(5);
//! // Map a function over the `Option` using the `Functor` type class
//! let y = map_explicit::<OptionBrand, _, _, _, _>(|i| i * 2, x);
//! assert_eq!(y, Some(10));
//! ```
//!
//! ## Example: Monadic Do-Notation with `m_do!`
//!
//! The `m_do!` macro provides Haskell/PureScript-style do-notation for flat monadic code.
//! It desugars `<-` binds into nested [`bind`](functions::bind) calls.
//!
//! ```
//! use fp_library::{brands::*, functions::*};
//! use fp_macros::m_do;
//!
//! let result = m_do!(OptionBrand {
//! 	x <- Some(5);
//! 	y <- Some(x + 1);
//! 	let z = x * y;
//! 	pure(z)
//! });
//! assert_eq!(result, Some(30));
//!
//! // Works with any monad brand
//! let result = m_do!(VecBrand {
//! 	x <- vec![1, 2];
//! 	y <- vec![10, 20];
//! 	pure(x + y)
//! });
//! assert_eq!(result, vec![11, 21, 12, 22]);
//! ```
//!
//! ## Crate Features
//!
//! - **`rayon`**: Enables true parallel execution for `par_*` functions using the [rayon](https://github.com/rayon-rs/rayon) library. Without this feature, `par_*` functions fall back to sequential equivalents.
//! - **`serde`**: Enables serialization and deserialization support for pure data types using the [serde](https://github.com/serde-rs/serde) library.
//! - **`stacker`**: Enables adaptive stack growth for deep `Coyoneda`, `RcCoyoneda`, and `ArcCoyoneda` map chains via the [stacker](https://github.com/rust-lang/stacker) crate. Without this feature, deeply chained maps can overflow the stack.

extern crate fp_macros;

pub mod brands;
pub mod classes;
pub mod dispatch;
pub mod functions;
pub mod kinds;
pub mod types;
pub(crate) mod utils;

pub use fp_macros::*;
