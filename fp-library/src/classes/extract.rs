//! Types that hold exactly one value which can be extracted.
//!
//! [`Extract`] is the inverse of [`Deferrable`](crate::classes::Deferrable): where
//! `Deferrable` constructs a value lazily from a thunk, `Extract` forces/extracts
//! the inner value. For types whose brand implements `Extract` (e.g.,
//! [`ThunkBrand`](crate::brands::ThunkBrand)), `extract(defer(|| x)) == x`
//! forms a round-trip. Note that `Extract` is a brand-level trait (implemented
//! by `ThunkBrand`), while `Deferrable` is a value-level trait (implemented by
//! concrete types like [`Thunk`](crate::types::Thunk)).
//!
//! This trait is used by [`Free::evaluate`](crate::types::Free::evaluate) to
//! execute the effects in a [`Free`](crate::types::Free) monad.
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
//! assert_eq!(extract::<ThunkBrand, _>(thunk), 42);
//! ```

#[fp_macros::document_module]
mod inner {
	use {
		crate::kinds::*,
		fp_macros::*,
	};

	/// A type containing exactly one extractable value, providing a natural
	/// transformation `F ~> Id`.
	///
	/// This trait witnesses that a type always holds a single value that can be
	/// extracted by running its effect. It is used by
	/// [`Free::evaluate`](crate::types::Free::evaluate) to execute the effects in a
	/// [`Free`](crate::types::Free) monad.
	///
	/// `Extract` is the inverse of [`Deferrable`](crate::classes::Deferrable):
	/// `Deferrable` constructs lazy values from thunks, while `Extract` forces and
	/// extracts them. For types whose brand implements `Extract` (e.g.,
	/// `ThunkBrand`), the round-trip law `extract(defer(|| x)) == x` holds.
	///
	/// Implemented by types that always contain exactly one value and can
	/// surrender ownership of it. [`Lazy`](crate::types::Lazy) cannot implement
	/// this trait because forcing it returns `&A` (a reference), not an owned `A`.
	/// [`Trampoline`](crate::types::Trampoline) does not have a brand and therefore
	/// cannot participate in HKT traits.
	///
	/// # Laws
	///
	/// **Pure-extract:** extracting a pure value returns the original value.
	/// For any `x: A`:
	///
	/// ```text
	/// extract(pure(x)) == x
	/// ```
	///
	/// This law states that wrapping a value in the type and immediately
	/// extracting it is the identity.
	#[kind(type Of<'a, A: 'a>: 'a;)]
	pub trait Extract {
		/// Extracts the inner value.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the value.",
			"The type of the value inside the container."
		)]
		///
		#[document_parameters("The container to extract from.")]
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
		/// let thunk = Thunk::new(|| 42);
		/// assert_eq!(extract::<ThunkBrand, _>(thunk), 42);
		/// ```
		fn extract<'a, A: 'a>(
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)
		) -> A;
	}

	/// Extracts the inner value.
	///
	/// Free function version that dispatches to [the type class' associated function][`Extract::extract`].
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the value.",
		"The extractable type.",
		"The type of the value inside the container."
	)]
	///
	#[document_parameters("The container to extract from.")]
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
	/// let thunk = Thunk::new(|| 42);
	/// assert_eq!(extract::<ThunkBrand, _>(thunk), 42);
	/// ```
	pub fn extract<'a, F, A>(fa: Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)) -> A
	where
		F: Extract,
		A: 'a, {
		F::extract(fa)
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

	/// Extract pure-extract law: extract(pure(x)) == x.
	#[quickcheck]
	fn prop_extract_pure(x: i32) -> bool {
		let fa = Thunk::pure(x);
		extract::<ThunkBrand, _>(fa) == x
	}
}
