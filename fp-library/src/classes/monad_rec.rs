//! Monads that support stack-safe tail recursion via [`ControlFlow`](core::ops::ControlFlow).
//!
//! ### Examples
//!
//! ```
//! use {
//! 	core::ops::ControlFlow,
//! 	fp_library::{
//! 		brands::*,
//! 		classes::*,
//! 		functions::tail_rec_m,
//! 		types::*,
//! 	},
//! };
//!
//! // A tail-recursive function to calculate factorial
//! fn factorial(n: u64) -> Thunk<'static, u64> {
//! 	tail_rec_m::<ThunkBrand, _, _>(
//! 		|(n, acc)| {
//! 			if n == 0 {
//! 				Thunk::pure(ControlFlow::Break(acc))
//! 			} else {
//! 				Thunk::pure(ControlFlow::Continue((n - 1, n * acc)))
//! 			}
//! 		},
//! 		(n, 1),
//! 	)
//! }
//!
//! assert_eq!(factorial(5).evaluate(), 120);
//! ```

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			classes::*,
			kinds::*,
		},
		core::ops::ControlFlow,
		fp_macros::*,
	};

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
	/// 1. **Identity**: `tail_rec_m(|a| pure(ControlFlow::Break(a)), x) == pure(x)`.
	///    Immediately wrapping a value in [`ControlFlow::Break`] must be equivalent
	///    to [`pure`](crate::classes::Pointed::pure).
	///
	/// ### Class Invariant
	///
	/// [`tail_rec_m`](MonadRec::tail_rec_m) must execute in constant stack space
	/// regardless of how many [`ControlFlow::Continue`] iterations occur. This is
	/// a structural requirement on the implementation, not an algebraic law.
	///
	/// ### Examples
	///
	/// Demonstrating the identity law with [`OptionBrand`](crate::brands::OptionBrand):
	///
	/// ```
	/// use {
	/// 	core::ops::ControlFlow,
	/// 	fp_library::{
	/// 		brands::*,
	/// 		functions::*,
	/// 	},
	/// };
	///
	/// // Identity law: tail_rec_m(|a| pure(ControlFlow::Break(a)), x) == pure(x)
	/// let result = tail_rec_m::<OptionBrand, _, _>(|a| Some(ControlFlow::Break(a)), 42);
	/// assert_eq!(result, Some(42));
	/// ```
	pub trait MonadRec: Monad {
		/// Performs tail-recursive monadic computation.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the computation.",
			"The type of the initial value and loop state.",
			"The type of the result."
		)]
		///
		#[document_parameters("The step function.", "The initial value.")]
		///
		#[document_returns("The result of the computation.")]
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	core::ops::ControlFlow,
		/// 	fp_library::{
		/// 		brands::*,
		/// 		functions::*,
		/// 		types::*,
		/// 	},
		/// };
		///
		/// let result = tail_rec_m::<ThunkBrand, _, _>(
		/// 	|n| {
		/// 		if n < 10 {
		/// 			Thunk::pure(ControlFlow::Continue(n + 1))
		/// 		} else {
		/// 			Thunk::pure(ControlFlow::Break(n))
		/// 		}
		/// 	},
		/// 	0,
		/// );
		///
		/// assert_eq!(result.evaluate(), 10);
		/// ```
		fn tail_rec_m<'a, A: 'a, B: 'a>(
			func: impl Fn(
				A,
			)
				-> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, ControlFlow<B, A>>)
			+ 'a,
			initial: A,
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>);
	}

	/// Performs tail-recursive monadic computation.
	///
	/// Free function version that dispatches to [the type class' associated function][`MonadRec::tail_rec_m`].
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the computation.",
		"The brand of the monad.",
		"The type of the initial value and loop state.",
		"The type of the result."
	)]
	///
	#[document_parameters("The step function.", "The initial value.")]
	///
	#[document_returns("The result of the computation.")]
	#[document_examples]
	///
	/// ```
	/// use {
	/// 	core::ops::ControlFlow,
	/// 	fp_library::{
	/// 		brands::*,
	/// 		functions::*,
	/// 		types::*,
	/// 	},
	/// };
	///
	/// let result = tail_rec_m::<ThunkBrand, _, _>(
	/// 	|n| {
	/// 		if n < 10 {
	/// 			Thunk::pure(ControlFlow::Continue(n + 1))
	/// 		} else {
	/// 			Thunk::pure(ControlFlow::Break(n))
	/// 		}
	/// 	},
	/// 	0,
	/// );
	///
	/// assert_eq!(result.evaluate(), 10);
	/// ```
	pub fn tail_rec_m<'a, Brand: MonadRec, A: 'a, B: 'a>(
		func: impl Fn(
			A,
		)
			-> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, ControlFlow<B, A>>)
		+ 'a,
		initial: A,
	) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
		Brand::tail_rec_m(func, initial)
	}
}

pub use inner::*;

#[cfg(test)]
mod tests {
	use {
		crate::{
			brands::*,
			functions::*,
			types::*,
		},
		core::ops::ControlFlow,
		quickcheck_macros::quickcheck,
	};

	/// MonadRec identity law for OptionBrand: tail_rec_m(|a| pure(Break(a)), x) == pure(x).
	#[quickcheck]
	fn prop_monad_rec_identity_option(x: i32) -> bool {
		let result = tail_rec_m::<OptionBrand, _, _>(|a| Some(ControlFlow::Break(a)), x);
		result == Some(x)
	}

	/// MonadRec identity law for ThunkBrand: tail_rec_m(|a| pure(Break(a)), x) == pure(x).
	#[quickcheck]
	fn prop_monad_rec_identity_thunk(x: i32) -> bool {
		let result = tail_rec_m::<ThunkBrand, _, _>(|a| Thunk::pure(ControlFlow::Break(a)), x);
		result.evaluate() == pure::<ThunkBrand, _>(x).evaluate()
	}
}
