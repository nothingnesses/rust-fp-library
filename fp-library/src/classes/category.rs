//! Category theory abstractions.
//!
//! This module defines the [`Category`] trait, which extends [`Semigroupoid`] with an identity element.
//! A category consists of objects and morphisms between them, with composition and identity.

use super::semigroupoid::Semigroupoid;
use crate::{Apply, kinds::*};

/// A type class for categories.
///
/// `Category` extends [`Semigroupoid`] with an identity element.
///
/// # Laws
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
	/// `forall a. Category cat => () -> cat a a`
	///
	/// ### Type Parameters
	///
	/// * `A`: The type of the object.
	///
	/// ### Returns
	///
	/// The identity morphism.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::classes::category::Category;
	/// use fp_library::brands::RcFnBrand;
	///
	/// let id = RcFnBrand::identity::<i32>();
	/// assert_eq!(id(5), 5);
	/// ```
	fn identity<'a, A>() -> Apply!(
		brand: Self,
		signature: ('a, A, A),
	);
}

/// Returns the identity morphism.
///
/// Free function version that dispatches to [the type class' associated function][`Category::identity`].
///
/// ### Type Signature
///
/// `forall a. Category cat => () -> cat a a`
///
/// ### Type Parameters
///
/// * `Brand`: The brand of the category.
/// * `A`: The type of the object.
///
/// ### Returns
///
/// The identity morphism.
///
/// ### Examples
///
/// ```
/// use fp_library::classes::category::identity;
/// use fp_library::brands::RcFnBrand;
///
/// let id = identity::<RcFnBrand, i32>();
/// assert_eq!(id(5), 5);
/// ```
pub fn identity<'a, Brand: Category, A>() -> Apply!(
	brand: Brand,
	signature: ('a, A, A),
) {
	Brand::identity()
}
