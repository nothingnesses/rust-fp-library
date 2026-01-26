//! Implementation of the `Thunk` type and `ThunkF` functor.
//!
//! This module provides the [`Thunk`] type, which represents a suspended computation,
//! and the [`ThunkFBrand`] functor, which allows `Thunk` to be used with the [`Free`](crate::types::Free) monad.
//!
//! ### Examples
//!
//! ```
//! use fp_library::types::Thunk;
//!
//! let thunk = Thunk::new(|| 42);
//! assert_eq!(thunk.force(), 42);
//! ```

use crate::{Apply, brands::ThunkFBrand, classes::functor::Functor, impl_kind, kinds::*};

/// A suspended computation that produces a value of type `A`.
///
/// `Thunk` wraps a closure that takes no arguments and returns a value.
/// It is used to delay evaluation until the value is needed.
///
/// ### Type Parameters
///
/// * `A`: The type of the value produced by the thunk.
///
/// ### Fields
///
/// * `0`: The boxed closure.
///
/// ### Examples
///
/// ```
/// use fp_library::types::Thunk;
///
/// let thunk = Thunk::new(|| 1 + 1);
/// assert_eq!(thunk.force(), 2);
/// ```
pub struct Thunk<'a, A>(Box<dyn FnOnce() -> A + 'a>);

impl<'a, A> Thunk<'a, A> {
	/// Creates a new `Thunk` from a closure.
	///
	/// ### Type Signature
	///
	/// `forall a. (FnOnce() -> a) -> Thunk a`
	///
	/// ### Parameters
	///
	/// * `f`: The closure to suspend.
	///
	/// ### Returns
	///
	/// A new `Thunk` containing the closure.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::types::Thunk;
	///
	/// let thunk = Thunk::new(|| 42);
	/// ```
	pub fn new(f: impl FnOnce() -> A + 'a) -> Self {
		Thunk(Box::new(f))
	}

	/// Forces the evaluation of the thunk, returning the result.
	///
	/// ### Type Signature
	///
	/// `forall a. Thunk a -> a`
	///
	/// ### Returns
	///
	/// The result of the suspended computation.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::types::Thunk;
	///
	/// let thunk = Thunk::new(|| 42);
	/// assert_eq!(thunk.force(), 42);
	/// ```
	pub fn force(self) -> A {
		(self.0)()
	}
}

impl_kind! {
	for ThunkFBrand {
		type Of<'a, A: 'a>: 'a = Thunk<'a, A>;
	}
}

impl Functor for ThunkFBrand {
	/// Maps a function over the value in the thunk.
	///
	/// ### Type Signature
	///
	/// `forall b a. Functor ThunkF => (a -> b, Thunk a) -> Thunk b`
	///
	/// ### Type Parameters
	///
	/// * `B`: The type of the result of applying the function.
	/// * `A`: The type of the value inside the thunk.
	/// * `F`: The type of the function to apply.
	///
	/// ### Parameters
	///
	/// * `f`: The function to apply.
	/// * `fa`: The thunk to map over.
	///
	/// ### Returns
	///
	/// A new thunk that, when forced, applies the function to the result of the original thunk.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, functions::*, types::*};
	///
	/// let thunk = Thunk::new(|| 5);
	/// let mapped = map::<ThunkFBrand, _, _, _>(|x| x * 2, thunk);
	/// assert_eq!(mapped.force(), 10);
	/// ```
	fn map<'a, B: 'a, A: 'a, F>(
		f: F,
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)
	where
		F: Fn(A) -> B + 'a,
	{
		Thunk::new(move || f(fa.force()))
	}
}

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
	fn run_effect<'a, A: 'a>(
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)
	) -> A;
}

impl Runnable for ThunkFBrand {
	/// Runs the thunk, producing the inner value.
	///
	/// ### Type Signature
	///
	/// `forall a. Runnable ThunkF => Thunk a -> a`
	///
	/// ### Type Parameters
	///
	/// * `A`: The type of the value inside the thunk.
	///
	/// ### Parameters
	///
	/// * `fa`: The thunk to run.
	///
	/// ### Returns
	///
	/// The result of forcing the thunk.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, types::*};
	///
	/// let thunk = Thunk::new(|| 42);
	/// assert_eq!(ThunkFBrand::run_effect(thunk), 42);
	/// ```
	fn run_effect<'a, A: 'a>(
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)
	) -> A {
		fa.force()
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	/// Tests the `Thunk::new` and `Thunk::force` methods.
	///
	/// Verifies that a thunk can be created from a closure and forced to produce the expected value.
	#[test]
	fn test_thunk_execution() {
		let thunk = Thunk::new(|| 42);
		assert_eq!(thunk.force(), 42);
	}

	/// Tests the `Functor` implementation for `ThunkFBrand`.
	///
	/// Verifies that `map` correctly transforms the value inside a thunk.
	#[test]
	fn test_thunk_functor() {
		use crate::classes::functor::map;
		let thunk = Thunk::new(|| 5);
		let mapped = map::<ThunkFBrand, _, _, _>(|x| x * 2, thunk);
		assert_eq!(mapped.force(), 10);
	}

	/// Tests the `Runnable` implementation for `ThunkFBrand`.
	///
	/// Verifies that `run_effect` correctly forces the thunk.
	#[test]
	fn test_thunk_runnable() {
		let thunk = Thunk::new(|| 42);
		assert_eq!(ThunkFBrand::run_effect(thunk), 42);
	}
}
