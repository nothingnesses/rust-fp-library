//! Concrete data types, their corresponding implementations and type aliases.
//!
//! This module provides implementations of various functional programming
//! data structures and wrappers, including `Identity`, `Lazy`, and extensions
//! for standard library types like `Option` and `Result`.
//!
//! ### Examples
//!
//! ```
//! use fp_library::types::Identity;
//!
//! let x = Identity(5);
//! assert_eq!(x.0, 5);
//! ```

/// Thread-safe reference-counted pointer abstraction using [`Arc`](std::sync::Arc).
///
/// Provides trait implementations for using `Arc` in the library's pointer abstraction hierarchy.
///
/// ### Examples
///
/// ```
/// use fp_library::{brands::*, functions::*};
///
/// let ptr = send_ref_counted_pointer_new::<ArcBrand, _>(42);
/// assert_eq!(*ptr, 42);
/// ```
pub mod arc_ptr;

/// Efficient queue-like structure with O(1) append and O(1) amortized uncons.
///
/// Implements the ["Reflection without Remorse"](http://okmij.org/ftp/Haskell/zseq.pdf) data structure used to enable O(1) left-associated [`bind`](crate::functions::bind) operations in the [`Free`] monad.
///
/// ### Examples
///
/// ```
/// use fp_library::types::cat_list::CatList;
///
/// let list = CatList::singleton(1)
///     .snoc(2)
///     .snoc(3)
///     .append(CatList::singleton(4));
///
/// let mut result = Vec::new();
/// let mut current = list;
/// while let Some((head, tail)) = current.uncons() {
///     result.push(head);
///     current = tail;
/// }
/// assert_eq!(result, vec![1, 2, 3, 4]);
/// ```
pub mod cat_list;

/// Wrapper for endofunctions (functions `a -> a`) with [`Semigroup`](crate::classes::semigroup::Semigroup) and [`Monoid`](crate::classes::monoid::Monoid) instances based on function composition.
///
/// Used to treat function composition as a monoidal operation where [`append`](crate::functions::append) composes functions and [`empty`](crate::functions::empty) is the identity function.
pub mod endofunction;

/// Wrapper for endomorphisms (morphisms `c a a` in a category) with [`Semigroup`](crate::classes::semigroup::Semigroup) and [`Monoid`](crate::classes::monoid::Monoid) instances based on categorical composition.
///
/// A more general form of `Endofunction` that works with any [`Category`](crate::classes::category::Category), not just functions.
pub mod endomorphism;

/// Reference-counted cloneable function wrappers with [`Semigroupoid`](crate::classes::semigroupoid::Semigroupoid) and [`Category`](crate::classes::category::Category) instances.
///
/// Provides the [`FnBrand`](crate::brands::FnBrand) abstraction for wrapping closures in `Rc<dyn Fn>` or `Arc<dyn Fn>` for use in higher-kinded contexts.
pub mod fn_brand;

/// Stack-safe Free monad over a functor with O(1) [`bind`](crate::functions::bind) operations.
///
/// Enables building computation chains without stack overflow by using a catenable list of continuations. Note: requires `'static` types and cannot implement the library's HKT traits due to type erasure.
///
/// ## Comparison with PureScript
///
/// This implementation is based on the PureScript [`Control.Monad.Free`](https://github.com/purescript/purescript-free/blob/master/src/Control/Monad/Free.purs) module
/// and the ["Reflection without Remorse"](http://okmij.org/ftp/Haskell/zseq.pdf) technique. It shares the same core algorithmic properties (O(1) bind, stack safety)
/// but differs significantly in its intended use case and API surface.
///
/// ### Key Differences
///
/// 1. **Interpretation Strategy**:
///    * **PureScript**: Designed as a generic Abstract Syntax Tree (AST) that can be interpreted into *any* target
///      monad using `runFree` or `foldFree` by providing a natural transformation at runtime.
///    * **Rust**: Designed primarily for **stack-safe execution** of computations. The interpretation logic is
///      baked into the [`Runnable`](crate::classes::runnable::Runnable) trait implemented by the functor `F`.
///      The [`Free::run`] method relies on `F` knowing how to "run" itself.
///
/// 2. **API Surface**:
///    * **PureScript**: Rich API including `liftF`, `hoistFree`, `resume`, `foldFree`.
///    * **Rust**: Minimal API focused on construction (`pure`, `roll`, `bind`) and execution (`run`).
///      * `liftF` is missing (use `roll` + `map`).
///      * `resume` is missing (cannot inspect the computation step-by-step).
///      * `hoistFree` is missing.
///
/// 3. **Terminology**:
///    * Rust's `Free::roll` corresponds to PureScript's `wrap`.
///
/// ### Capabilities and Limitations
///
/// **What it CAN do:**
/// * Provide stack-safe recursion for monadic computations (trampolining).
/// * Prevent stack overflows when chaining many `bind` operations.
/// * Execute self-describing effects (like [`Thunk`]).
///
/// **What it CANNOT do (easily):**
/// * Act as a generic DSL where the interpretation is decoupled from the operation type.
///   * *Example*: You cannot easily define a `DatabaseOp` enum and interpret it differently for
///     production (SQL) and testing (InMemory) using this `Free` implementation, because
///     `DatabaseOp` must implement a single `Runnable` trait.
/// * Inspect the structure of the computation (introspection) via `resume`.
///
/// ### Lifetimes and Memory Management
///
/// * **PureScript**: Relies on a garbage collector and `unsafeCoerce`. This allows it to ignore
///   lifetimes and ownership, enabling a simpler implementation that supports all types.
/// * **Rust**: Relies on ownership and `Box<dyn Any>` for type erasure. `Any` requires `'static`
///   to ensure memory safety (preventing use-after-free of references). This forces `Free` to
///   only work with `'static` types, preventing it from implementing the library's HKT traits
///   which require lifetime polymorphism.
///
/// ### Examples
///
/// ```
/// use fp_library::{brands::*, types::*};
///
/// // ✅ CAN DO: Stack-safe recursion
/// let free = Free::<ThunkBrand, _>::pure(42)
///     .bind(|x| Free::pure(x + 1));
/// ```
pub mod free;

/// Trivial wrapper that contains a single value.
///
/// The simplest possible container type, often used as a base case for higher-kinded types or when a container is required but no additional effect is needed.
pub mod identity;

/// Memoized lazy evaluation with shared cache semantics.
///
/// Computes a value at most once on first access and caches the result. All clones share the same cache. Available in both single-threaded [`RcLazy`] and thread-safe [`ArcLazy`] variants.
pub mod lazy;

/// Functional programming trait implementations for the standard library [`Option`] type.
///
/// Extends `Option` with [`Functor`](crate::classes::functor::Functor), [`Monad`](crate::classes::semimonad::Semimonad), [`Foldable`](crate::classes::foldable::Foldable), [`Traversable`](crate::classes::traversable::Traversable), [`Filterable`](crate::classes::filterable::Filterable), and [`Witherable`](crate::classes::witherable::Witherable) instances.
pub mod option;

/// Two-value container with [`Bifunctor`](crate::classes::bifunctor::Bifunctor) and dual [`Functor`](crate::classes::functor::Functor) instances.
///
/// Can be used as a bifunctor over both values, or as a functor/monad by fixing either the first value [`PairWithFirstBrand`](crate::brands::PairWithFirstBrand) or second value [`PairWithSecondBrand`](crate::brands::PairWithSecondBrand).
pub mod pair;

/// Single-threaded reference-counted pointer abstraction using [`Rc`](std::rc::Rc).
///
/// Provides trait implementations for using `Rc` in the library's pointer abstraction hierarchy. Not thread-safe; use [`ArcBrand`](crate::brands::ArcBrand) for multi-threaded contexts.
///
/// ### Examples
///
/// ```
/// use fp_library::{brands::*, functions::*};
///
/// let ptr = pointer_new::<RcBrand, _>(42);
/// assert_eq!(*ptr, 42);
/// ```
pub mod rc_ptr;

/// Functional programming trait implementations for the standard library [`Result`] type.
///
/// Extends `Result` with dual functor/monad instances: [`ResultWithErrBrand`](crate::brands::ResultWithErrBrand) (standard Result monad) functors over the success value, while [`ResultWithOkBrand`](crate::brands::ResultWithOkBrand) functors over the error value.
pub mod result;

/// Thread-safe wrapper for endofunctions with [`Semigroup`](crate::classes::semigroup::Semigroup) and [`Monoid`](crate::classes::monoid::Monoid) instances.
///
/// The `Send + Sync` counterpart to [`Endofunction`], wrapping functions that can be safely shared across threads.
pub mod send_endofunction;

/// Control type representing Loop/Done states for tail-recursive computations.
///
/// Used by [`MonadRec`](crate::classes::monad_rec::MonadRec) to implement stack-safe tail recursion. [`Step::Loop`] continues iteration, while [`Step::Done`] terminates with a result.
///
/// ### Examples
///
/// ```
/// use fp_library::types::*;
///
/// // Count down from n to 0, accumulating the sum
/// fn sum_to_zero(n: i32, acc: i32) -> Step<(i32, i32), i32> {
///     if n <= 0 {
///         Step::Done(acc)
///     } else {
///         Step::Loop((n - 1, acc + n))
///     }
/// }
/// ```
pub mod step;

/// [`Semigroup`](crate::classes::semigroup::Semigroup) and [`Monoid`](crate::classes::monoid::Monoid) instances for the standard library [`String`] type.
///
/// Provides string concatenation as a monoidal operation with the empty string as the identity element.
pub mod string;

/// Deferred, non-memoized computation with higher-kinded type support.
///
/// Builds computation chains without stack safety guarantees but supports borrowing and lifetime polymorphism. Each call to [`Thunk::run`] re-executes the computation. For stack-safe alternatives, use [`Trampoline`].
pub mod thunk;

/// Stack-safe computation type with guaranteed safety for unlimited recursion depth.
///
/// Built on the [`Free`] monad with O(1) [`bind`](crate::functions::bind) operations. Provides complete stack safety at the cost of requiring `'static` types. Use this for deep recursion and heavy monadic pipelines.
///
/// ### Examples
///
/// ```
/// use fp_library::types::*;
///
/// let task = Trampoline::new(|| 1 + 1)
///     .bind(|x| Trampoline::new(move || x * 2))
///     .bind(|x| Trampoline::new(move || x + 10));
///
/// assert_eq!(task.run(), 14);
/// ```
pub mod trampoline;

/// Memoized lazy evaluation for fallible computations.
///
/// Computes a [`Result`] at most once and caches either the success value or error. All clones share the same cache. Available in both single-threaded [`RcTryLazy`] and thread-safe [`ArcTryLazy`] variants.
pub mod try_lazy;

/// Deferred, non-memoized fallible computation with higher-kinded type support.
///
/// The fallible counterpart to [`Thunk`]. Each call to [`TryThunk::run`] re-executes the computation and returns a [`Result`]. Supports borrowing and lifetime polymorphism.
pub mod try_thunk;

/// Stack-safe fallible computation type with guaranteed safety for unlimited recursion depth.
///
/// Wraps [`Trampoline<Result<A, E>>`](crate::types::Trampoline) with ergonomic combinators for error handling. Provides complete stack safety for fallible computations that may recurse deeply.
///
/// ### Examples
///
/// ```
/// use fp_library::types::*;
///
/// let task: TryTrampoline<i32, String> = TryTrampoline::ok(10)
///     .map(|x| x * 2)
///     .bind(|x| TryTrampoline::ok(x + 5));
///
/// assert_eq!(task.run(), Ok(25));
/// ```
pub mod try_trampoline;

/// Functional programming trait implementations for the standard library [`Vec`] type.
///
/// Extends `Vec` with [`Functor`](crate::classes::functor::Functor), [`Monad`](crate::classes::semimonad::Semimonad), [`Foldable`](crate::classes::foldable::Foldable), [`Traversable`](crate::classes::traversable::Traversable), [`Filterable`](crate::classes::filterable::Filterable), [`Witherable`](crate::classes::witherable::Witherable), and parallel folding instances.
pub mod vec;

pub use cat_list::CatList;
pub use endofunction::Endofunction;
pub use endomorphism::Endomorphism;
pub use free::Free;
pub use identity::Identity;
pub use lazy::{ArcLazy, ArcLazyConfig, Lazy, LazyConfig, RcLazy, RcLazyConfig};
pub use pair::Pair;
pub use send_endofunction::SendEndofunction;
pub use step::Step;
pub use thunk::Thunk;
pub use trampoline::Trampoline;
pub use try_lazy::{ArcTryLazy, RcTryLazy, TryLazy};
pub use try_thunk::TryThunk;
pub use try_trampoline::TryTrampoline;
