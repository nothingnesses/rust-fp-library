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
//!
//! ## Features
//!
//! - **Higher-Kinded Types (HKT):** Implemented using lightweight higher-kinded polymorphism (type-level defunctionalization/brands).
//! - **Macros:** Procedural macros for working with HKTs and monadic code:
//!   - **HKT:** `trait_kind!`, `impl_kind!`, `Apply!`, `#[kind]` for defining and applying higher-kinded type encodings
//!   - **Do-Notation:** `m_do!` for monadic do-notation, `a_do!` for applicative do-notation
//! - **Type Classes:** A comprehensive collection of standard type classes including:
//!   - **Core:** `Functor`, `Contravariant`, `Pointed`, `Applicative`, `Semiapplicative`, `Monad`, `Semimonad`, `Semigroup`, `Monoid`, `Foldable`, `Traversable`, `Alt`, `Plus`, `Alternative`
//!   - **Applicative Utilities:** `Lift`, `ApplyFirst`, `ApplySecond`
//!   - **Monad Utilities:** `MonadPlus`, `MonadRec`, `Extract`
//!   - **Comonads:** `Extend`, `Comonad`
//!   - **Bifunctors:** `Bifunctor`, `Bifoldable`, `Bitraversable`
//!   - **Collections:** `Compactable`, `Filterable`, `Witherable`
//!   - **Indexed:** `WithIndex`, `FunctorWithIndex`, `FoldableWithIndex`, `TraversableWithIndex`
//!   - **Category Theory:** `Category`, `Semigroupoid`, `Profunctor`, `Strong`, `Choice`, `Closed`, `Cochoice`, `Costrong`, `Wander`
//!   - **Laziness & Effects:** `RefFunctor`, `SendRefFunctor`, `Deferrable`, `SendDeferrable`, `LazyConfig`
//!   - **Parallel:** `ParFunctor`, `ParCompactable`, `ParFilterable`, `ParFoldable`, `ParFunctorWithIndex`, `ParFoldableWithIndex`
//! - **Function & Pointer Abstractions:** Traits for abstracting over function wrappers and reference counting:
//!   - **Functions:** `Function`, `CloneableFn`, `SendCloneableFn`, `UnsizedCoercible`, `SendUnsizedCoercible`
//!   - **Pointers:** `Pointer`, `RefCountedPointer`, `SendRefCountedPointer`
//! - **Optics:** Composable data accessors using profunctor encoding (port of PureScript's `purescript-profunctor-lenses`):
//!   - **Iso / IsoPrime:** Isomorphism between two types
//!   - **Lens / LensPrime:** Focus on a field within a product type
//!   - **Prism / PrismPrime:** Focus on a variant within a sum type
//!   - **AffineTraversal / AffineTraversalPrime:** Optional focusing (combines Lens + Prism)
//!   - **Traversal / TraversalPrime:** Focus on multiple values
//!   - **Getter / GetterPrime:** Read-only access
//!   - **Setter / SetterPrime:** Write-only modification
//!   - **Fold / FoldPrime:** Collecting multiple values (read-only)
//!   - **Review / ReviewPrime:** Constructing values
//!   - **Grate / GratePrime:** Closed/zipping optics
//!   - **Indexed variants:** `IndexedLens`, `IndexedTraversal`, `IndexedGetter`, `IndexedFold`, `IndexedSetter`
//!   - **Composition:** `Composed` struct and `optics_compose` for zero-cost optic composition
//! - **Numeric Algebra:** `Semiring`, `Ring`, `CommutativeRing`, `EuclideanRing`, `DivisionRing`, `Field`, `HeytingAlgebra`
//! - **Newtype Wrappers:** `Additive`, `Multiplicative`, `Conjunctive`, `Disjunctive`, `First`, `Last`, `Dual`
//! - **Helper Functions:** Standard FP utilities:
//!   - `compose`, `constant`, `flip`, `identity`, `on`, `pipe`
//! - **Data Types:** Implementations for standard and custom types:
//!   - **Standard Library:** `Option`, `Result`, `Vec`, `String`
//!   - **Laziness, Memoization & Stack Safety:** `Lazy` (`RcLazy`, `ArcLazy`), `Thunk`, `SendThunk`, `Trampoline`, `Free`, `FreeStep`
//!   - **Fallible Variants:** `TryLazy` (`RcTryLazy`, `ArcTryLazy`), `TryThunk`, `TrySendThunk`, `TryTrampoline`
//!   - **Generic Containers:** `Identity`, `Pair`, `CatList`
//!   - **Function Wrappers:** `Endofunction`, `Endomorphism`
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
//! use fp_library::{
//! 	impl_kind,
//! 	kinds::*,
//! };
//!
//! pub struct OptionBrand;
//!
//! impl_kind! {
//! 	for OptionBrand {
//! 		type Of<'a, A: 'a>: 'a = Option<A>;
//! 	}
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
//! | Type                   | Primary Use Case                                                                                                            | Stack Safe?                  | Memoized? | Lifetimes  | Send?            | HKT Traits                           |
//! | :--------------------- | :-------------------------------------------------------------------------------------------------------------------------- | :--------------------------- | :-------- | :--------- | :--------------- | :----------------------------------- |
//! | **`Thunk<'a, A>`**     | **Glue Code & Borrowing.** Lightweight deferred computation. Best for short chains and working with references.             | Partial (`tail_rec_m` only)  | No        | `'a`       | No               | `Functor`, `Applicative`, `Monad`    |
//! | **`SendThunk<'a, A>`** | **Thread-Safe Glue Code.** Like `Thunk`, but the closure is `Send`. Enables truly lazy `into_arc_lazy()`.                   | No                           | No        | `'a`       | Yes              | No                                   |
//! | **`Trampoline<A>`**    | **Deep Recursion & Pipelines.** Heavy-duty computation. Uses a trampoline to guarantee stack safety for infinite recursion. | Yes                          | No        | `'static`  | No               | No                                   |
//! | **`Lazy<'a, A>`**      | **Caching.** Wraps a computation to ensure it runs at most once. `RcLazy` for single-threaded, `ArcLazy` for thread-safe.   | N/A                          | Yes       | `'a`       | Config-dependent | `RefFunctor`, `Foldable`             |
//!
//! Each of these has a fallible counterpart that wraps `Result<A, E>` with ergonomic error-handling combinators:
//!
//! | Type                         | Primary Use Case                                                                                                    | Stack Safe?                  | Memoized? | Lifetimes  | Send?            | HKT Traits                                                |
//! | :--------------------------- | :------------------------------------------------------------------------------------------------------------------ | :--------------------------- | :-------- | :--------- | :--------------- | :--------------------------------------------------------- |
//! | **`TryThunk<'a, A, E>`**     | **Fallible Glue Code.** Lightweight deferred computation that may fail. Best for short chains with error handling.  | Partial (`tail_rec_m` only)  | No        | `'a`       | No               | `Functor`, `Applicative`, `Monad`, `Bifunctor`, `Foldable` |
//! | **`TrySendThunk<'a, A, E>`** | **Thread-Safe Fallible Glue Code.** Like `TryThunk`, but the closure is `Send`.                                     | No                           | No        | `'a`       | Yes              | No                                                         |
//! | **`TryTrampoline<A, E>`**    | **Fallible Deep Recursion.** Stack-safe computation that may fail. Uses a trampoline for unlimited recursion depth. | Yes                          | No        | `'static`  | No               | No                                                         |
//! | **`TryLazy<'a, A, E>`**      | **Fallible Caching.** Computes a `Result` at most once and caches either the success value or error.                | N/A                          | Yes       | `'a`       | Config-dependent | `RefFunctor`, `Foldable`                                   |
//!
//! **Config-dependent Send:** `ArcLazy`/`ArcTryLazy` are `Send + Sync`; `RcLazy`/`RcTryLazy` are not.
//!
//! #### The "Why" of Multiple Types
//!
//! Unlike lazy languages (e.g., Haskell) where the runtime handles everything, Rust requires us to choose our trade-offs:
//!
//! 1. **`Thunk` vs `Trampoline`**: `Thunk` is faster and supports borrowing (`&'a T`). Its `tail_rec_m` is stack-safe, but deep `bind` chains will overflow the stack. `Trampoline` guarantees stack safety for all operations via a trampoline (the `Free` monad) but requires types to be `'static`. Note that `!Send` types like `Rc<T>` are fully supported. A key distinction is that `Thunk` implements `Functor`, `Applicative`, and `Monad` directly, making it suitable for generic programming, while `Trampoline` does not.
//! 2. **`Thunk` vs `SendThunk`**: `Thunk` wraps `Box<dyn FnOnce() -> A + 'a>` and is `!Send`. `SendThunk` wraps `Box<dyn FnOnce() -> A + Send + 'a>` and can cross thread boundaries. Use `SendThunk` when you need truly lazy `into_arc_lazy()` (converting to `ArcLazy` without eager evaluation), or when building deferred computation chains that will be consumed on another thread. `TrySendThunk` is the fallible counterpart.
//! 3. **Computation vs Caching**: `Thunk` and `Trampoline` describe _computations_ that are not memoized. Each instance is consumed on `.evaluate()` (which takes `self` by value), so the computation runs exactly once per instance, but constructing a new instance re-executes the work. `Lazy`, by contrast, caches the result so that all clones share a single evaluation. If you have an expensive operation (like a DB call), convert it to a `Lazy` to guarantee it runs at most once.
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
//! 	Val(i32),
//! 	Add(Box<Expr>, Box<Expr>),
//! 	Div(Box<Expr>, Box<Expr>),
//! }
//!
//! // 1. Stack-safe recursion with error handling (TryTrampoline)
//! fn eval(expr: &Expr) -> TryTrampoline<i32, String> {
//! 	let expr = expr.clone(); // Capture owned data for 'static closure
//! 	TryTrampoline::defer(move || match expr {
//! 		Expr::Val(n) => TryTrampoline::ok(n),
//! 		Expr::Add(lhs, rhs) => eval(&lhs).bind(move |l| eval(&rhs).map(move |r| l + r)),
//! 		Expr::Div(lhs, rhs) => eval(&lhs).bind(move |l| {
//! 			eval(&rhs).bind(move |r| {
//! 				if r == 0 {
//! 					TryTrampoline::err("Division by zero".to_string())
//! 				} else {
//! 					TryTrampoline::ok(l / r)
//! 				}
//! 			})
//! 		}),
//! 	})
//! }
//!
//! // Usage
//! fn main() {
//! 	let expr = Expr::Div(Box::new(Expr::Val(100)), Box::new(Expr::Val(2)));
//!
//! 	// 2. Memoize result (TryLazy)
//! 	// The evaluation runs at most once, even if accessed multiple times.
//! 	let result = RcTryLazy::new(move || eval(&expr).evaluate());
//!
//! 	// 3. Create deferred view (TryThunk)
//! 	// Borrow the cached result to format it.
//! 	let view: TryThunk<String, String> = TryThunk::new(|| {
//! 		let val = result.evaluate().map_err(|e| e.clone())?;
//! 		Ok(format!("Result: {}", val))
//! 	});
//!
//! 	assert_eq!(view.evaluate(), Ok("Result: 50".to_string()));
//! }
//! ```
//!
//! ### Thread Safety and Parallelism
//!
//! The library provides a parallel trait hierarchy that mirrors the sequential one.
//! All `par_*` free functions accept plain `impl Fn + Send + Sync` closures: no wrapper
//! types required. Element types require `A: Send`; closures require `Send + Sync`.
//!
//! | Parallel trait | Operations | Supertraits |
//! |---|---|---|
//! | `ParFunctor` | `par_map` | `Kind` |
//! | `ParCompactable` | `par_compact`, `par_separate` | `Kind` |
//! | `ParFilterable` | `par_filter_map`, `par_filter` | `ParFunctor + ParCompactable` |
//! | `ParFoldable` | `par_fold_map` | `Kind` |
//! | `ParFunctorWithIndex` | `par_map_with_index` | `ParFunctor + FunctorWithIndex` |
//! | `ParFoldableWithIndex` | `par_fold_map_with_index` | `ParFoldable + FoldableWithIndex` |
//!
//! `ParFilterable` provides default implementations of `par_filter_map` and `par_filter`
//! derived from `par_map` + `par_compact`; types can override them for single-pass efficiency.
//!
//! - **`SendCloneableFn`**: Extends `CloneableFn` to provide `Send + Sync` function wrappers. Implemented by `ArcFnBrand`.
//! - **Rayon Support**: When the `rayon` feature is enabled, `par_*` functions use rayon for true parallel execution. Otherwise they fall back to sequential equivalents.
//!
//! ```
//! use fp_library::{
//! 	brands::*,
//! 	functions::*,
//! };
//!
//! let v = vec![1, 2, 3, 4, 5];
//! // Map in parallel (uses rayon if feature is enabled)
//! let doubled: Vec<i32> = par_map::<VecBrand, _, _>(|x: i32| x * 2, v.clone());
//! assert_eq!(doubled, vec![2, 4, 6, 8, 10]);
//! // Compact options in parallel
//! let opts = vec![Some(1), None, Some(3), None, Some(5)];
//! let compacted: Vec<i32> = par_compact::<VecBrand, _>(opts);
//! assert_eq!(compacted, vec![1, 3, 5]);
//! // Fold in parallel
//! let result = par_fold_map::<VecBrand, _, _>(|x: i32| x.to_string(), v);
//! assert_eq!(result, "12345".to_string());
//! ```
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
//! let y = map::<OptionBrand, _, _>(|i| i * 2, x);
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

extern crate fp_macros;

pub mod brands;
pub mod classes;
pub mod functions;
pub mod kinds;
pub mod types;
pub(crate) mod utils;

pub use fp_macros::*;
