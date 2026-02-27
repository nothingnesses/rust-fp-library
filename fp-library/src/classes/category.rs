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
	crate::{
		Apply,
		kinds::*,
	},
	fp_macros::{
		document_signature,
		document_type_parameters,
	},
};

/// A type class for categories, which are semigroupoids with an identity element.
///
/// A category consists of objects and morphisms between them, with composition and identity.
///
/// ### Hierarchy Unification
///
/// By inheriting from [`Semigroupoid`], this trait implicitly requires [`Kind_266801a817966495`].
/// This unification ensures that categorical identity morphisms also satisfy the strict lifetime
/// requirements where the object type must outlive the morphism's application lifetime.
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
	#[document_type_parameters("The lifetime of the morphism.", "The type of the object.")]
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
	fn identity<'a, A>() -> Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, A, A>);
}

/// Returns the identity morphism.
///
/// Free function version that dispatches to [the type class' associated function][`Category::identity`].
#[document_signature]
///
#[document_type_parameters(
	"The lifetime of the morphism.",
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
pub fn identity<'a, Brand: Category, A>()
-> Apply!(<Brand as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, A, A>) {
	Brand::identity()
}
