//! A type class for categories, which are semigroupoids with an identity element.
//!
//! A category consists of objects and morphisms between them, with composition and identity.
//!
//! ### Examples
//!
//! ```
//! use fp_library::{brands::*, functions::*};
//!
//! let id = category_identity::<RcFnBrand, i32>();
//! assert_eq!(id(5), 5);
//! ```

use super::semigroupoid::Semigroupoid;
use crate::{Apply, kinds::*};
use fp_macros::doc_type_params;
use fp_macros::hm_signature;

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
	///
	/// ### Type Signature
	///
	#[hm_signature(Category)]
	///
	/// ### Type Parameters
	///
	#[doc_type_params("The lifetime of the morphism.", "The type of the object.")]
	///
	/// ### Returns
	///
	/// The identity morphism.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, functions::*};
	///
	/// let id = category_identity::<RcFnBrand, i32>();
	/// assert_eq!(id(5), 5);
	/// ```
	fn identity<'a, A>() -> Apply!(<Self as Kind!( type Of<'a, T, U>; )>::Of<'a, A, A>);
}

/// Returns the identity morphism.
///
/// Free function version that dispatches to [the type class' associated function][`Category::identity`].
///
/// ### Type Signature
///
#[hm_signature(Category)]
///
/// ### Type Parameters
///
#[doc_type_params(
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
/// use fp_library::{brands::*, functions::*};
///
/// let id = category_identity::<RcFnBrand, i32>();
/// assert_eq!(id(5), 5);
/// ```
pub fn identity<'a, Brand: Category, A>()
-> Apply!(<Brand as Kind!( type Of<'a, T, U>; )>::Of<'a, A, A>) {
	Brand::identity()
}
