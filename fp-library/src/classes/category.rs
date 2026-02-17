//! Categories, which are semigroupoids with an identity element for each object.
//!
//! A category consists of objects and morphisms between them, with composition and identity.
//!
//! ### Examples
//!
//! ```
//! use fp_library::{
//! 	brands::*,
//! 	functions::*,
//! };
//!
//! let id = category_identity::<RcFnBrand, i32>();
//! assert_eq!(id(5), 5);
//! ```

use {
	super::semigroupoid::Semigroupoid,
	crate::{Apply, kinds::*},
	fp_macros::{document_signature, document_type_parameters},
};

/// A type class for categories, which are semigroupoids with an identity element.
///
/// A category consists of objects and morphisms between them, with composition and identity.
///
/// ### Laws
///
/// `Category` instances must satisfy the identity law:
/// * Identity: `compose(identity, p) = compose(p, identity)`.
pub trait Category: Semigroupoid {
	/// Returns the identity morphism.
	///
	/// The identity morphism is a morphism that maps every object to itself.
	#[document_signature]
	///
	#[document_type_parameters("The type of the object.")]
	///
	/// ### Returns
	///
	/// The identity morphism.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// };
	///
	/// let id = category_identity::<RcFnBrand, i32>();
	/// assert_eq!(id(5), 5);
	/// ```
	fn identity<A: 'static>() -> Apply!(<Self as Kind!( type Of<T, U>; )>::Of<A, A>);
}

/// Returns the identity morphism.
///
/// Free function version that dispatches to [the type class' associated function][`Category::identity`].
#[document_signature]
///
#[document_type_parameters(
	"The brand of the category.",
	"The type of the object."
)]
///
/// ### Returns
///
/// The identity morphism.
///
/// ### Examples
///
/// ```
/// use fp_library::{
/// 	brands::*,
/// 	functions::*,
/// };
///
/// let id = category_identity::<RcFnBrand, i32>();
/// assert_eq!(id(5), 5);
/// ```
pub fn identity<Brand: Category, A: 'static>()
-> Apply!(<Brand as Kind!( type Of<T, U>; )>::Of<A, A>) {
	Brand::identity::<A>()
}
