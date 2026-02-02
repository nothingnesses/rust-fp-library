#![warn(missing_docs)]

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
//!   - **Core:** `Functor`, `Applicative`, `Monad`, `Semigroup`, `Monoid`, `Foldable`, `Traversable`
//!   - **Collections:** `Compactable`, `Filterable`, `Witherable`
//!   - **Category Theory:** `Category`, `Semigroupoid`
//!   - **Utilities:** `Pointed`, `Lift`, `ApplyFirst`, `ApplySecond`, `Semiapplicative`, `Semimonad`
//!   - **Advanced/Internal:** `MonadRec`, `RefFunctor`, `Defer`, `SendDefer`
//!   - **Function & Pointer Abstractions:** `Function`, `CloneableFn`, `SendCloneableFn`, `ParFoldable`, `Pointer`, `RefCountedPointer`, `SendRefCountedPointer`
//! - **Helper Functions:** Standard FP utilities:
//!   - `compose`, `constant`, `flip`, `identity`
//! - **Data Types:** Implementations for standard and custom types:
//!   - **Standard Library:** `Option`, `Result`, `Vec`, `String`
//!   - **Laziness, Memoization & Stack Safety:** `Lazy`, `Thunk`, `Trampoline`, `Free`
//!   - **Generic Containers:** `Identity`, `Pair`
//!   - **Function Wrappers:** `Endofunction`, `Endomorphism`, `SendEndofunction`
//!   - **Marker Types:** `RcBrand`, `ArcBrand`, `FnBrand`
//!
//! ## How it Works
//!
//! ### Higher-Kinded Types (HKT)
//!
//! Since Rust doesn't support HKTs directly (i.e., it's not possible to use `Option` in `impl Functor for Option`, instead of `Option<T>`), this library uses **Lightweight Higher-Kinded Polymorphism** (also known as the "Brand" pattern or type-level defunctionalization).
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
//! - **Functions as Data:** When functions are stored in data structures (e.g., inside a `Vec` for `Semiapplicative::apply`, or in `Lazy` thunks), they must often be "type-erased" (wrapped in `Rc<dyn Fn>` or `Arc<dyn Fn>`). This is because every closure in Rust has a unique, anonymous type. To store multiple different closures in the same container, or to compose functions dynamically (like in `Endofunction`), they must be coerced to a common trait object.
//! - **Lazy Evaluation:** The `Lazy` type relies on storing a thunk that can be cloned and evaluated later, which typically requires reference counting and dynamic dispatch.
//!
//! For these specific cases, the library provides `Brand` types (like `RcFnBrand` and `ArcFnBrand`) to let you choose the appropriate wrapper (single-threaded vs. thread-safe) while keeping the rest of your code zero-cost. The library uses a unified `Pointer` hierarchy to abstract over these choices.
//!
//! ### Lazy Evaluation & Effect System
//!
//! Rust is an eagerly evaluated language. To enable functional patterns like deferred execution and safe recursion, `fp-library` provides a granular set of types that let you opt-in to specific behaviors without paying for unnecessary overhead.
//!
//! | Type                | Primary Use Case                                                                                                            | Stack Safe?                    | Memoized? | Lifetimes?   | HKT Traits                           |
//! | :------------------ | :-------------------------------------------------------------------------------------------------------------------------- | :----------------------------- | :-------- | :----------- | :----------------------------------- |
//! | **`Thunk<'a, A>`**  | **Glue Code & Borrowing.** Lightweight deferred computation. Best for short chains and working with references.             | ⚠️ Partial (`tail_rec_m` only) | ❌ No     | ✅ `'a`      | ✅ `Functor`, `Applicative`, `Monad` |
//! | **`Trampoline<A>`** | **Deep Recursion & Pipelines.** Heavy-duty computation. Uses a trampoline to guarantee stack safety for infinite recursion. | ✅ Yes                         | ❌ No     | ❌ `'static` | ❌ No                                |
//! | **`Lazy<'a, A>`**   | **Caching.** Wraps a computation to ensure it runs at most once.                                                            | N/A                            | ✅ Yes    | ✅ `'a`      | ✅ `RefFunctor`                      |
//!
//! #### The "Why" of Three Types
//!
//! Unlike lazy languages (e.g., Haskell) where the runtime handles everything, Rust requires us to choose our trade-offs:
//!
//! 1. **`Thunk` vs `Trampoline`**: `Thunk` is faster and supports borrowing (`&'a T`). Its `tail_rec_m` is stack-safe, but deep `bind` chains will overflow the stack. `Trampoline` guarantees stack safety for all operations via a trampoline (the `Free` monad) but requires types to be `'static` and `Send`. A key distinction is that `Thunk` implements `Functor`, `Applicative`, and `Monad` directly, making it suitable for generic programming, while `Trampoline` does not.
//! 2. **Computation vs Caching**: `Thunk` and `Trampoline` describe _computations_—they re-run every time you call `.evaluate()`. If you have an expensive operation (like a DB call), convert it to a `Lazy` to cache the result.
//!
//! #### Workflow Example: Expression Evaluator
//!
//! A robust pattern is to use `TryTrampoline` for stack-safe, fallible recursion, `TryLazy` to memoize expensive results, and `TryThunk` to create lightweight views.
//!
//! Consider an expression evaluator that handles division errors and deep recursion:
//!
//! ```rust
//! use fp_library::types::*;
//!
//! #[derive(Clone)]
//! enum Expr {
//!     Val(i32),
//!     Add(Box<Expr>, Box<Expr>),
//!     Div(Box<Expr>, Box<Expr>),
//! }
//!
//! // 1. Stack-safe recursion with error handling (TryTrampoline)
//! fn eval(expr: &Expr) -> TryTrampoline<i32, String> {
//!     let expr = expr.clone(); // Capture owned data for 'static closure
//!     TryTrampoline::defer(move || match expr {
//!         Expr::Val(n) => TryTrampoline::ok(n),
//!         Expr::Add(lhs, rhs) => {
//!             eval(&lhs).bind(move |l| eval(&rhs).map(move |r| l + r))
//!         }
//!         Expr::Div(lhs, rhs) => {
//!             eval(&lhs).bind(move |l| {
//!                 eval(&rhs).bind(move |r| {
//!                     if r == 0 {
//!                         TryTrampoline::err("Division by zero".to_string())
//!                     } else {
//!                         TryTrampoline::ok(l / r)
//!                     }
//!                 })
//!             })
//!         }
//!     })
//! }
//!
//! // Usage
//! fn main() {
//!     let expr = Expr::Div(Box::new(Expr::Val(100)), Box::new(Expr::Val(2)));
//!
//!     // 2. Memoize result (TryLazy)
//!     // The evaluation runs at most once, even if accessed multiple times.
//!     let result = RcTryLazy::new(move || eval(&expr).evaluate());
//!
//!     // 3. Create deferred view (TryThunk)
//!     // Borrow the cached result to format it.
//!     let view: TryThunk<String, String> = TryThunk::new(|| {
//!         let val = result.evaluate().map_err(|e| e.clone())?;
//!         Ok(format!("Result: {}", val))
//!     });
//!
//!     assert_eq!(view.evaluate(), Ok("Result: 50".to_string()));
//! }
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
