//! A type class for monads that support stack-safe tail recursion.
//!
//! ### Examples
//!
//! ```
//! use fp_library::{
//!     brands::*,
//!     classes::*,
//!     types::*,
//!     functions::tail_rec_m,
//! };
//!
//! // A tail-recursive function to calculate factorial
//! fn factorial(n: u64) -> Eval<'static, u64> {
//!     tail_rec_m::<EvalBrand, _, _, _>(
//!         |(n, acc)| {
//!             if n == 0 {
//!                 Eval::pure(Step::Done(acc))
//!             } else {
//!                 Eval::pure(Step::Loop((n - 1, n * acc)))
//!             }
//!         },
//!         (n, 1),
//!     )
//! }
//!
//! assert_eq!(factorial(5).run(), 120);
//! ```

use crate::{Apply, classes::monad::Monad, kinds::*, types::step::Step};

/// A type class for monads that support stack-safe tail recursion.
///
/// ### Important Design Note
///
/// `Eval<'a, A>` CAN implement this trait (HKT-compatible).
/// `Task<A>` CANNOT implement this trait (requires `'static`).
///
/// For deep recursion (10,000+ calls), prefer `Task::tail_rec_m` which is
/// guaranteed stack-safe. `Eval`'s trait-based `tail_rec_m` will overflow
/// the stack at ~8000 recursive calls.
///
/// ### Laws
///
/// 1. **Equivalence**: `tail_rec_m(f, a)` produces the same result as the
///    recursive definition.
///
/// 2. **Safety varies**: Eval is NOT stack-safe for deep recursion.
///    Use Task for guaranteed stack safety.
pub trait MonadRec: Monad {
	/// Performs tail-recursive monadic computation.
	///
	/// ### Type Signature
	///
	/// `forall m b a. MonadRec m => (a -> m (Step a b), a) -> m b`
	///
	/// ### Type Parameters
	///
	/// * `B`: The type of the result.
	/// * `A`: The type of the initial value and loop state.
	/// * `F`: The type of the step function.
	///
	/// ### Parameters
	///
	/// * `f`: The step function.
	/// * `a`: The initial value.
	///
	/// ### Returns
	///
	/// The result of the computation.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{
	///     brands::*,
	///     classes::*,
	///     types::*,
	/// };
	///
	/// let result = EvalBrand::tail_rec_m(
	///     |n| {
	///         if n < 10 {
	///             Eval::pure(Step::Loop(n + 1))
	///         } else {
	///             Eval::pure(Step::Done(n))
	///         }
	///     },
	///     0,
	/// );
	///
	/// assert_eq!(result.run(), 10);
	/// ```
	fn tail_rec_m<'a, A: 'a, B: 'a, F>(
		f: F,
		a: A,
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)
	where
		F: Fn(A) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Step<A, B>>)
			+ Clone
			+ 'a;
}

/// Performs tail-recursive monadic computation.
///
/// Free function version that dispatches to [the type class' associated function][`MonadRec::tail_rec_m`].
///
/// ### Type Signature
///
/// `forall m b a. MonadRec m => (a -> m (Step a b), a) -> m b`
///
/// ### Type Parameters
///
/// * `Brand`: The brand of the monad.
/// * `B`: The type of the result.
/// * `A`: The type of the initial value and loop state.
/// * `F`: The type of the step function.
///
/// ### Parameters
///
/// * `f`: The step function.
/// * `a`: The initial value.
///
/// ### Returns
///
/// The result of the computation.
///
/// ### Examples
///
/// ```
/// use fp_library::{
///     brands::*,
///     classes::*,
///     types::*,
///     functions::tail_rec_m,
/// };
///
/// let result = tail_rec_m::<EvalBrand, _, _, _>(
///     |n| {
///         if n < 10 {
///             Eval::pure(Step::Loop(n + 1))
///         } else {
///             Eval::pure(Step::Done(n))
///         }
///     },
///     0,
/// );
///
/// assert_eq!(result.run(), 10);
/// ```
pub fn tail_rec_m<'a, Brand: MonadRec, A: 'a, B: 'a, F>(
	f: F,
	a: A,
) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)
where
	F: Fn(A) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Step<A, B>>)
		+ Clone
		+ 'a,
{
	Brand::tail_rec_m(f, a)
}
