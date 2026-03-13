//! Alternative functors, combining [`Applicative`](crate::classes::Applicative) and [`Plus`](crate::classes::Plus).
//!
//! `Alternative` provides the ability to choose between computations (via [`Alt`](crate::classes::Alt))
//! with an identity element (via [`Plus`](crate::classes::Plus)), combined with applicative
//! lifting (via [`Applicative`](crate::classes::Applicative)).
//!
//! ### Examples
//!
//! ```
//! use fp_library::{
//! 	brands::*,
//! 	classes::*,
//! 	functions::*,
//! };
//!
//! // guard filters based on a condition
//! let result: Vec<()> = guard::<VecBrand>(true);
//! assert_eq!(result, vec![()]);
//!
//! let result: Vec<()> = guard::<VecBrand>(false);
//! assert_eq!(result, vec![]);
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

	/// A type class combining [`Applicative`] and [`Plus`].
	///
	/// `Alternative` has no members of its own; it specifies that the type
	/// constructor has both [`Applicative`] and [`Plus`] instances.
	///
	/// ### Laws
	///
	/// `Alternative` instances must satisfy the following laws:
	/// * Distributivity: `apply(alt(f, g), x) = alt(apply(f, x), apply(g, x))`.
	/// * Annihilation: `apply(empty, f) = empty`.
	#[document_examples]
	///
	/// Alternative laws for [`Vec`]:
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	classes::*,
	/// 	functions::*,
	/// };
	///
	/// // Annihilation: apply(empty, f) = empty
	/// let f: Vec<i32> = vec![1, 2, 3];
	/// let empty_fns: Vec<std::rc::Rc<dyn Fn(i32) -> i32>> = plus_empty::<VecBrand, _>();
	/// let result = apply::<RcFnBrand, VecBrand, _, _>(empty_fns, f);
	/// assert_eq!(result, plus_empty::<VecBrand, i32>());
	/// ```
	pub trait Alternative: Applicative + Plus {}

	/// Blanket implementation of [`Alternative`].
	#[document_type_parameters("The brand type.")]
	impl<Brand> Alternative for Brand where Brand: Applicative + Plus {}

	/// Fails using [`Plus`] if a condition does not hold, or succeeds using
	/// [`Applicative`] if it does.
	///
	/// This is useful in monadic/applicative comprehensions to filter results.
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the alternative functor."
	)]
	///
	#[document_parameters("The condition to check.")]
	///
	#[document_returns("`pure(())` if the condition is `true`, `empty` otherwise.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	classes::*,
	/// 	functions::*,
	/// };
	///
	/// // Using guard to filter in a Vec comprehension
	/// let result: Vec<()> = guard::<VecBrand>(true);
	/// assert_eq!(result, vec![()]);
	///
	/// let result: Vec<()> = guard::<VecBrand>(false);
	/// assert_eq!(result, vec![]);
	/// ```
	pub fn guard<'a, Brand: Alternative>(
		condition: bool
	) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, ()>) {
		if condition { Brand::pure(()) } else { Brand::empty() }
	}
}

pub use inner::*;
