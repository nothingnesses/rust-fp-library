//! Arrows, which represent computations that can be composed and satisfy the laws of category and strength.
//!
//! An arrow is a type constructor that is both a [`Category`] and a [`Strong`] profunctor,
//! with the additional ability to lift pure functions into the arrow context.
//!
//! ### Examples
//!
//! ```
//! use fp_library::{
//! 	brands::*,
//! 	functions::*,
//! };
//!
//! let f = arrow::<RcFnBrand, _, _>(|x: i32| x * 2);
//! assert_eq!(f(5), 10);
//! ```

use {
	super::{category::Category, strong::Strong},
	crate::{Apply, kinds::*},
	fp_macros::{document_parameters, document_signature, document_type_parameters},
};

/// A type class for arrows.
///
/// An arrow is a [`Category`] that is also a [`Strong`] profunctor.
/// It provides the `arrow` method to lift a pure function into the arrow context.
///
/// ### Laws
///
/// `Arrow` instances must satisfy the following laws (in addition to `Category` and `Strong` laws):
/// * Identity: `arrow(id) = id`.
/// * Composition: `arrow(g ∘ f) = arrow(g) ∘ arrow(f)`.
/// * Naturality: `first(arrow(f)) ∘ arrow(fst) = arrow(fst) ∘ arrow(f)`.
pub trait Arrow: Category + Strong {
	/// Lifts a pure function into the arrow context.
	///
	/// This method takes a closure and returns an arrow instance that represents that function.
	#[document_signature]
	///
	#[document_type_parameters(
		"The input type of the arrow.",
		"The output type of the arrow."
	)]
	///
	#[document_parameters("The closure to lift into an arrow.")]
	///
	/// ### Returns
	///
	/// The arrow instance.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// };
	///
	/// let f = arrow::<RcFnBrand, _, _>(|x: i32| x * 2);
	/// assert_eq!(f(5), 10);
	/// ```
	fn arrow<A, B>(
		f: impl Fn(A) -> B + 'static
	) -> Apply!(<Self as Kind!( type Of<T, U>; )>::Of<A, B>);
}

impl<Brand> Arrow for Brand
where
	Brand: Category + Strong,
{
	fn arrow<A, B>(
		f: impl Fn(A) -> B + 'static
	) -> Apply!(<Self as Kind!( type Of<T, U>; )>::Of<A, B>) {
		Brand::lmap(f, Brand::identity())
	}
}

/// Lifts a pure function into the arrow context.
///
/// Free function version that dispatches to [the type class' associated function][`Arrow::arrow`].
#[document_signature]
///
#[document_type_parameters(
	"The brand of the arrow.",
	"The input type of the arrow.",
	"The output type of the arrow."
)]
///
#[document_parameters("The closure to lift into an arrow.")]
///
/// ### Returns
///
/// The arrow instance.
///
/// ### Examples
///
/// ```
/// use fp_library::{
/// 	brands::*,
/// 	functions::*,
/// };
///
/// let f = arrow::<RcFnBrand, _, _>(|x: i32| x * 2);
/// assert_eq!(f(5), 10);
/// ```
pub fn arrow<Brand, A, B>(
	f: impl Fn(A) -> B + 'static
) -> Apply!(<Brand as Kind!( type Of<T, U>; )>::Of<A, B>)
where
	Brand: Arrow,
{
	Brand::arrow(f)
}
