//! A functor whose effects can be "run" to produce the inner value.
//!
//! This trait is used by [`Free::run`](crate::types::Free::run) to execute the effects
//! in a `Free` monad.
//!
//! ### Examples
//!
//! ```
//! use fp_library::{brands::*, functions::*, types::*};
//!
//! let thunk = Thunk::new(|| 42);
//! assert_eq!(runnable_run::<ThunkBrand, _>(thunk), 42);
//! ```

use crate::{Apply, classes::functor::Functor, kinds::*};

/// A functor whose effects can be "run" to produce the inner value.
///
/// This trait is used by [`Free::run`](crate::types::Free::run) to execute the effects
/// in a `Free` monad.
pub trait Runnable: Functor {
	/// Runs the effect, producing the inner value.
	///
	/// ### Type Signature
	///
	/// `forall a. Runnable f => f a -> a`
	///
	/// ### Type Parameters
	///
	/// * `A`: The type of the value inside the functor.
	///
	/// ### Parameters
	///
	/// * `fa`: The functor instance to run.
	///
	/// ### Returns
	///
	/// The inner value.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, functions::*, types::*};
	///
	/// let thunk = Thunk::new(|| 42);
	/// assert_eq!(runnable_run::<ThunkBrand, _>(thunk), 42);
	/// ```
	fn run<'a, A: 'a>(fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)) -> A;
}

/// Runs the effect, producing the inner value.
///
/// Free function version that dispatches to [the type class' associated function][`Runnable::run`].
///
/// ### Type Signature
///
/// `forall a. Runnable f => f a -> a`
///
/// ### Type Parameters
///
/// * `F`: The runnable functor.
/// * `A`: The type of the value inside the functor.
///
/// ### Parameters
///
/// * `fa`: The functor instance to run.
///
/// ### Returns
///
/// The inner value.
///
/// ### Examples
///
/// ```
/// use fp_library::{brands::*, functions::*, types::*};
///
/// let thunk = Thunk::new(|| 42);
/// assert_eq!(runnable_run::<ThunkBrand, _>(thunk), 42);
/// ```
pub fn run<'a, F, A>(fa: Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)) -> A
where
	F: Runnable,
	A: 'a,
{
	F::run(fa)
}
