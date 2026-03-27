//! Functors whose effects can be evaluated to produce an inner value.
//!
//! This trait is used by [`Free::evaluate`](crate::types::Free::evaluate) to execute the effects
//! in a [`Free`](crate::types::Free) monad.
//!
//! ### Examples
//!
//! ```
//! use fp_library::{
//! 	brands::*,
//! 	functions::*,
//! 	types::*,
//! };
//!
//! let thunk = Thunk::new(|| 42);
//! assert_eq!(evaluate::<ThunkBrand, _>(thunk), 42);
//! ```

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			classes::*,
			kinds::*,
		},
		fp_macros::*,
	};

	/// A functor containing exactly one extractable value, providing a natural
	/// transformation `F ~> Id`.
	///
	/// This trait witnesses that a functor always holds a single value that can be
	/// extracted by running its effect. It is used by
	/// [`Free::evaluate`](crate::types::Free::evaluate) to execute the effects in a
	/// [`Free`](crate::types::Free) monad.
	///
	/// Implemented by functors that always contain exactly one value and can
	/// surrender ownership of it. [`Lazy`](crate::types::Lazy) cannot implement
	/// this trait because forcing it returns `&A` (a reference), not an owned `A`.
	/// [`Trampoline`](crate::types::Trampoline) does not have a brand and therefore
	/// cannot participate in HKT traits.
	///
	/// # Laws
	///
	/// **Map-extract:** extracting after mapping is the same as extracting and then
	/// applying the function. For any `f: A -> B` and `fa: F<A>`:
	///
	/// ```text
	/// evaluate(map(f, fa)) == f(evaluate(fa))
	/// ```
	///
	/// This law states that the functor wrapper does not alter the value observed
	/// by `evaluate`; mapping a function over the functor and then extracting
	/// always yields the same result as extracting first and applying the function
	/// afterwards.
	pub trait Evaluable: Functor {
		/// Evaluates the effect, producing the inner value.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the value.",
			"The type of the value inside the functor."
		)]
		///
		#[document_parameters("The functor instance to evaluate.")]
		///
		#[document_returns("The inner value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let eval = Thunk::new(|| 42);
		/// assert_eq!(evaluate::<ThunkBrand, _>(eval), 42);
		/// ```
		fn evaluate<'a, A: 'a>(
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)
		) -> A;
	}

	/// Evaluates the effect, producing the inner value.
	///
	/// Free function version that dispatches to [the type class' associated function][`Evaluable::evaluate`].
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the value.",
		"The evaluable functor.",
		"The type of the value inside the functor."
	)]
	///
	#[document_parameters("The functor instance to evaluate.")]
	///
	#[document_returns("The inner value.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// 	types::*,
	/// };
	///
	/// let eval = Thunk::new(|| 42);
	/// assert_eq!(evaluate::<ThunkBrand, _>(eval), 42);
	/// ```
	pub fn evaluate<'a, F, A>(fa: Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)) -> A
	where
		F: Evaluable,
		A: 'a, {
		F::evaluate(fa)
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
		quickcheck_macros::quickcheck,
	};

	/// Evaluable map-extract law: evaluate(map(f, fa)) == f(evaluate(fa)).
	#[quickcheck]
	fn prop_evaluable_map_extract(x: i32) -> bool {
		let f = |a: i32| a.wrapping_mul(3).wrapping_add(7);
		let fa = Thunk::new(|| x);
		let fa2 = Thunk::new(|| x);
		let lhs = evaluate::<ThunkBrand, _>(map::<ThunkBrand, _, _>(f, fa));
		let rhs = f(evaluate::<ThunkBrand, _>(fa2));
		lhs == rhs
	}
}
