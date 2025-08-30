use crate::{
	classes::{ClonableFn, clonable_fn::ApplyFn},
	hkt::{Apply1L2T, Kind1L2T},
};

/// A type class for semigroupoids.
///
/// A `Semigroupoid` is a set of objects and composable relationships
/// (morphisms) between them.
///
/// # Laws
///
/// Semigroupoid instances must satisfy the associative law:
/// * Associativity: `compose(p)(compose(q)(r)) = compose(compose(p)(q))(r)`.
///
/// # Examples
pub trait Semigroupoid: Kind1L2T {
	/// Takes morphisms `f` and `g` and returns the morphism `f . g` (`f` composed with `g`).
	///
	/// # Type Signature
	///
	/// `forall b c d. Semigroupoid a => a c d -> a b c -> a b d`
	///
	/// # Parameters
	///
	/// * `f`: A morphism of type `a c d`.
	/// * `g`: A morphism of type `a b c`.
	///
	/// # Returns
	///
	/// The morphism `f` composed with `g` of type `a b d`.
	fn compose<'a, ClonableFnBrand: 'a + ClonableFn, B, C, D>(
		f: Apply1L2T<'a, Self, C, D>
	) -> ApplyFn<'a, ClonableFnBrand, Apply1L2T<'a, Self, B, C>, Apply1L2T<'a, Self, B, D>>;
}

/// Takes morphisms `f` and `g` and returns the morphism `f . g` (`f` composed with `g`).
///
/// Free function version that dispatches to [the type class' associated function][`Semigroupoid::compose`].
///
/// # Type Signature
///
/// `forall b c d. Semigroupoid a => a c d -> a b c -> a b d`
///
/// # Parameters
///
/// * `f`: A morphism of type `a c d`.
/// * `g`: A morphism of type `a b c`.
///
/// # Returns
///
/// The morphism `f` composed with `g` of type `a b d`.
///
/// # Examples
///
/// ```
/// use fp_library::{brands::RcFnBrand, functions::compose};
/// use std::rc::Rc;
///
/// let add_one = Rc::new(|x: i32| x + 1);
/// let times_two = Rc::new(|x: i32| x * 2);
/// let times_two_add_one = compose::<RcFnBrand, RcFnBrand, _, _, _>(add_one)(times_two);
///
/// // 3 * 2 + 1 = 7
/// assert_eq!(times_two_add_one(3), 7);
/// ```
pub fn compose<'a, ClonableFnBrand: 'a + ClonableFn, Brand: Semigroupoid, B, C, D>(
	f: Apply1L2T<'a, Brand, C, D>
) -> ApplyFn<'a, ClonableFnBrand, Apply1L2T<'a, Brand, B, C>, Apply1L2T<'a, Brand, B, D>> {
	Brand::compose::<'a, ClonableFnBrand, B, C, D>(f)
}
