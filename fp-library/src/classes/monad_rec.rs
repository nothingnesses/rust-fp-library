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
//! fn factorial(n: u64) -> Thunk<'static, u64> {
//!     tail_rec_m::<ThunkBrand, _, _, _>(
//!         |(n, acc)| {
//!             if n == 0 {
//!                 Thunk::pure(Step::Done(acc))
//!             } else {
//!                 Thunk::pure(Step::Loop((n - 1, n * acc)))
//!             }
//!         },
//!         (n, 1),
//!     )
//! }
//!
//! assert_eq!(factorial(5).evaluate(), 120);
//! ```

use fp_macros::doc_params;
use fp_macros::doc_type_params;
use crate::{Apply, classes::monad::Monad, kinds::*, types::step::Step};
use fp_macros::hm_signature;

/// A type class for monads that support stack-safe tail recursion.
///
/// ### Important Design Note
///
/// [`Thunk<'a, A>`](crate::types::Thunk) CAN implement this trait (HKT-compatible).
/// [`Trampoline<A>`](crate::types::Trampoline) CANNOT implement this trait (requires `'static`).
///
/// `Thunk`'s `tail_rec_m` implementation uses a loop and is stack-safe.
/// However, `Thunk`'s `bind` chains are NOT stack-safe.
/// `Trampoline` is stack-safe for both `tail_rec_m` and `bind` chains.
///
/// ### Laws
///
/// 1. **Equivalence**: `tail_rec_m(f, a)` produces the same result as the
///    recursive definition.
///
/// 2. **Safety varies**: `Thunk` is stack-safe for `tail_rec_m` but not for deep `bind` chains.
///    `Trampoline` is guaranteed stack-safe for all operations.
pub trait MonadRec: Monad {
	/// Performs tail-recursive monadic computation.
	///
	/// ### Type Signature
	///
	/// `forall m a b. MonadRec m => (a -> m (Step a b), a) -> m b`
	///
	/// ### Type Parameters
	///
	#[doc_type_params(
		"Undocumented",
		"The type of the initial value and loop state.",
		"The type of the result.",
		"The type of the step function."
	)]	///
	/// ### Parameters
	///
	#[doc_params(
		"The step function.",
		"The initial value."
	)]	///
	/// ### Returns
	///
	/// The result of the computation.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, functions::*, types::*};
	///
	/// let result = tail_rec_m::<ThunkBrand, _, _, _>(
	///     |n| {
	///         if n < 10 {
	///             Thunk::pure(Step::Loop(n + 1))
	///         } else {
	///             Thunk::pure(Step::Done(n))
	///         }
	///     },
	///     0,
	/// );
	///
	/// assert_eq!(result.evaluate(), 10);
	/// ```
	fn tail_rec_m<'a, A: 'a, B: 'a, Func>(
		func: Func,
		initial: A,
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)
	where
		Func: Fn(A) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Step<A, B>>)
			+ Clone
			+ 'a;
}

/// Performs tail-recursive monadic computation.
///
/// Free function version that dispatches to [the type class' associated function][`MonadRec::tail_rec_m`].
///
/// ### Type Signature
///
#[hm_signature(MonadRec)]
///
/// ### Type Parameters
///
#[doc_type_params(
	"Undocumented",
	"The brand of the monad.",
	"The type of the initial value and loop state.",
	"The type of the result.",
	("A", "The type of the initial value and loop state.")
)]///
/// ### Parameters
///
#[doc_params(
	"The step function.",
	"The initial value."
)]///
/// ### Returns
///
/// The result of the computation.
///
/// ### Examples
///
/// ```
/// use fp_library::{brands::*, functions::*, types::*};
///
/// let result = tail_rec_m::<ThunkBrand, _, _, _>(
///     |n| {
///         if n < 10 {
///             Thunk::pure(Step::Loop(n + 1))
///         } else {
///             Thunk::pure(Step::Done(n))
///         }
///     },
///     0,
/// );
///
/// assert_eq!(result.evaluate(), 10);
/// ```
pub fn tail_rec_m<'a, Brand: MonadRec, A: 'a, B: 'a, Func>(
	func: Func,
	initial: A,
) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)
where
	Func: Fn(A) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Step<A, B>>)
		+ Clone
		+ 'a,
{
	Brand::tail_rec_m(func, initial)
}
