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
//! ## Features
//!
//! - **Higher-Kinded Types (HKT):** Implemented using lightweight higher-kinded polymorphism (type-level defunctionalization/brands).
//! - **Macros:** Procedural macros (`def_kind!`, `impl_kind!`, `Apply!`) to simplify HKT boilerplate and type application.
//! - **Type Classes:** A comprehensive collection of standard type classes including:
//!   - `Functor`, `Applicative`, `Monad`
//!   - `Semigroup`, `Monoid`
//!   - `Foldable`, `Traversable`
//!   - `Compactable`, `Filterable`, `Witherable`
//!   - `Category`, `Semigroupoid`
//!   - `Pointed`, `Lift`
//!   - `ApplyFirst`, `ApplySecond`, `Semiapplicative`, `Semimonad`
//!   - `MonadRec`, `RefFunctor`
//!   - `Function`, `CloneableFn`, `SendCloneableFn`, `ParFoldable` (Function wrappers and thread-safe operations)
//!   - `Pointer`, `RefCountedPointer`, `SendRefCountedPointer` (Pointer abstraction)
//!   - `Defer`, `SendDefer`
//! - **Helper Functions:** Standard FP utilities:
//!   - `compose`, `constant`, `flip`, `identity`
//! - **Data Types:** Implementations for standard and custom types:
//!   - `Option`, `Result`, `Vec`, `String`
//!   - `Identity`, `Memo`, `Pair`
//!   - `Task`, `Eval`, `Free`
//!   - `Endofunction`, `Endomorphism`, `SendEndofunction`
//!   - `RcBrand`, `ArcBrand`, `FnBrand`
//!
//! ## How it Works
//!
//! ### Higher-Kinded Types (HKT)
//!
//! Since Rust doesn't support HKTs directly (e.g., `trait Functor<F<_>>`), this library uses **Lightweight Higher-Kinded Polymorphism** (also known as the "Brand" pattern or type-level defunctionalization).
//!
//! Each type constructor has a corresponding `Brand` type (e.g., `OptionBrand` for `Option`). These brands implement the `Kind` traits, which map the brand and generic arguments back to the concrete type. The library provides macros to simplify this process.
//!
//! ```rust
//! use fp_library::{impl_kind, kinds::*};
//!
//! pub struct OptionBrand;
//!
//! impl_kind! {
//!     for OptionBrand {
//!         type Of<'a, A: 'a>: 'a = Option<A>;
//!     }
//! }
//! ```
//!
//! ### Zero-Cost Abstractions & Uncurried Semantics
//!
//! Unlike many functional programming libraries that strictly adhere to curried functions (e.g., `map(f)(fa)`), `fp-library` adopts **uncurried semantics** (e.g., `map(f, fa)`) for its core abstractions.
//!
//! **Why?**
//! Traditional currying in Rust often requires:
//!
//! - Creating intermediate closures for each partial application.
//! - Heap-allocating these closures (boxing) or wrapping them in reference counters (`Rc`/`Arc`) to satisfy type system constraints.
//! - Dynamic dispatch (`dyn Fn`), which inhibits compiler optimizations like inlining.
//!
//! By using uncurried functions with `impl Fn` or generic bounds, `fp-library` achieves **zero-cost abstractions**:
//!
//! - **No Heap Allocation:** Operations like `map` and `bind` do not allocate intermediate closures.
//! - **Static Dispatch:** The compiler can fully monomorphize generic functions, enabling aggressive inlining and optimization.
//! - **Ownership Friendly:** Better integration with Rust's ownership and borrowing system.
//!
//! This approach ensures that using high-level functional abstractions incurs no runtime penalty compared to hand-written imperative code.
//!
//! **Exceptions:**
//! While the library strives for zero-cost abstractions, some operations inherently require dynamic dispatch or heap allocation due to Rust's type system:
//!
//! - **Functions as Data:** When functions are stored in data structures (e.g., inside a `Vec` for `Semiapplicative::apply`, or in `Memo` thunks), they must often be "type-erased" (wrapped in `Rc<dyn Fn>` or `Arc<dyn Fn>`). This is because every closure in Rust has a unique, anonymous type. To store multiple different closures in the same container, or to compose functions dynamically (like in `Endofunction`), they must be coerced to a common trait object.
//! - **Lazy Evaluation:** The `Memo` type relies on storing a thunk that can be cloned and evaluated later, which typically requires reference counting and dynamic dispatch.
//!
//! For these specific cases, the library provides `Brand` types (like `RcFnBrand` and `ArcFnBrand`) to let you choose the appropriate wrapper (single-threaded vs. thread-safe) while keeping the rest of your code zero-cost. The library uses a unified `Pointer` hierarchy to abstract over these choices.
//!
//! ### Lazy Evaluation & Effect System
//!
//! Rust is an eagerly evaluated language. To enable functional patterns like deferred execution and safe recursion, `fp-library` provides a granular set of types that let you opt-in to specific behaviors without paying for unnecessary overhead.
//!
//! | Type              | Primary Use Case                                                                                                            | Stack Safe? | Memoized? | Lifetimes?   | HKT Traits                           |
//! | :---------------- | :-------------------------------------------------------------------------------------------------------------------------- | :---------- | :-------- | :----------- | :----------------------------------- |
//! | **`Eval<'a, A>`** | **Glue Code & Borrowing.** Lightweight deferred computation. Best for short chains and working with references.             | ❌ No       | ❌ No     | ✅ `'a`      | ✅ `Functor`, `Applicative`, `Monad` |
//! | **`Task<A>`**     | **Deep Recursion & Pipelines.** Heavy-duty computation. Uses a trampoline to guarantee stack safety for infinite recursion. | ✅ Yes      | ❌ No     | ❌ `'static` | ❌ No                                |
//! | **`Memo<'a, A>`** | **Caching.** Wraps a computation to ensure it runs at most once.                                                            | N/A         | ✅ Yes    | ✅ `'a`      | ✅ `RefFunctor`                      |
//!
//! #### The "Why" of Three Types
//!
//! Unlike lazy languages (e.g., Haskell) where the runtime handles everything, Rust requires us to choose our trade-offs:
//!
//! 1.  **`Eval` vs `Task`**: `Eval` is faster and supports borrowing (`&'a T`), but will overflow the stack if you recurse too deeply. `Task` guarantees stack safety via a trampoline (the `Free` monad) but requires types to be `'static` and `Send`. A key distinction is that `Eval` implements `Functor`, `Applicative`, and `Monad` directly, making it suitable for generic programming, while `Task` does not.
//! 2.  **Computation vs Caching**: `Eval` and `Task` describe _computations_—they re-run every time you call `.run()`. If you have an expensive operation (like a DB call), convert it to a `Memo` to cache the result.
//!
//! #### Workflow Example
//!
//! A common pattern is to use `Task` for the heavy lifting (recursion), `Memo` to freeze the result, and `Eval` to borrow it later.
//!
//! ```rust
//! use fp_library::types::*;
//!
//! // 1. Use Task for stack-safe recursion
//! let heavy_computation = Task::tail_rec_m(|n| {
//!     if n < 1_000 { Task::pure(Step::Loop(n + 1)) } else { Task::pure(Step::Done(n)) }
//! }, 0);
//!
//! // 2. Convert to Memo to cache the result (runs once)
//! let cached = Memo::<_, RcMemoConfig>::from(heavy_computation);
//!
//! // 3. Use Eval to borrow the cached value without re-running
//! let view = Eval::new(|| {
//!     let val = cached.get();
//!     format!("Result: {}", val)
//! });
//! ```
//!
//! ### Thread Safety and Parallelism
//!
//! The library supports thread-safe operations through the `SendCloneableFn` extension trait and parallel folding via `ParFoldable`.
//!
//! - **`SendCloneableFn`**: Extends `CloneableFn` to provide `Send + Sync` function wrappers. Implemented by `ArcFnBrand`.
//! - **`ParFoldable`**: Provides `par_fold_map` and `par_fold_right` for parallel execution.
//! - **Rayon Support**: `VecBrand` supports parallel execution using `rayon` when the `rayon` feature is enabled.
//!
//! ```
//! use fp_library::{brands::*, functions::*};
//!
//! let v = vec![1, 2, 3, 4, 5];
//! // Create a thread-safe function wrapper
//! let f = send_cloneable_fn_new::<ArcFnBrand, _, _>(|x: i32| x.to_string());
//! // Fold in parallel (if rayon feature is enabled)
//! let result = par_fold_map::<ArcFnBrand, VecBrand, _, _>(f, v);
//! assert_eq!(result, "12345".to_string());
//! ```
//!
//! ## Example: Using `Functor` with `Option`
//!
//! ```
//! use fp_library::{brands::*, functions::*};
//!
//! let x = Some(5);
//! // Map a function over the `Option` using the `Functor` type class
//! let y = map::<OptionBrand, _, _, _>(|i| i * 2, x);
//! assert_eq!(y, Some(10));
//! ```
//!
//! ## Crate Features
//!
//! - **`rayon`**: Enables parallel folding operations (`ParFoldable`) and parallel execution support for `VecBrand` using the [rayon](https://github.com/rayon-rs/rayon) library.

extern crate fp_macros;

pub mod brands;
pub mod classes;
pub mod functions;
pub mod kinds;
pub mod types;

pub use fp_macros::Apply;
pub use fp_macros::Kind;
pub use fp_macros::def_kind;
pub use fp_macros::impl_kind;
