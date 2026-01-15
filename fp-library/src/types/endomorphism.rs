//! Implementations for [`Endomorphism`], a wrapper for endomorphisms (morphisms from an object to the same object) that enables monoidal operations.

use crate::{
	Apply,
	classes::{category::Category, monoid::Monoid, semigroup::Semigroup},
	kinds::*,
};
use std::{
	fmt::{self, Debug, Formatter},
	hash::Hash,
};

/// A wrapper for endomorphisms (morphisms from an object to the same object) that enables monoidal operations.
///
/// `Endomorphism c a` represents a morphism `c a a` where `c` is a `Category`.
/// For the category of functions, this represents functions of type `a -> a`.
///
/// It exists to provide a monoid instance where:
///
/// * The binary operation [append][Semigroup::append] is [morphism composition][crate::classes::semigroupoid::Semigroupoid::compose].
/// * The identity element [empty][Monoid::empty] is the [identity morphism][Category::identity].
///
/// The wrapped morphism can be accessed directly via the [`.0` field][Endomorphism#structfield.0].
pub struct Endomorphism<'a, C: Category, A>(pub Apply!(brand: C, signature: ('a, A, A)));

impl<'a, C: Category, A> Endomorphism<'a, C, A> {
	/// Creates a new `Endomorphism`.
	///
	/// # Type Signature
	///
	/// `forall a c. Category c => c a a -> Endomorphism c a`
	///
	/// # Parameters
	///
	/// * `f`: The morphism to wrap.
	///
	/// # Returns
	///
	/// A new `Endomorphism`.
	pub fn new(f: Apply!(brand: C, signature: ('a, A, A))) -> Self {
		Self(f)
	}
}

impl<'a, C: Category, A> Clone for Endomorphism<'a, C, A>
where
	Apply!(brand: C, signature: ('a, A, A)): Clone,
{
	fn clone(&self) -> Self {
		Self::new(self.0.clone())
	}
}

impl<'a, C: Category, A> Debug for Endomorphism<'a, C, A>
where
	Apply!(brand: C, signature: ('a, A, A)): Debug,
{
	fn fmt(
		&self,
		fmt: &mut Formatter<'_>,
	) -> fmt::Result {
		fmt.debug_tuple("Endomorphism").field(&self.0).finish()
	}
}

impl<'a, C: Category, A> Eq for Endomorphism<'a, C, A> where
	Apply!(brand: C, signature: ('a, A, A)): Eq
{
}

impl<'a, C: Category, A> Hash for Endomorphism<'a, C, A>
where
	Apply!(brand: C, signature: ('a, A, A)): Hash,
{
	fn hash<H: std::hash::Hasher>(
		&self,
		state: &mut H,
	) {
		self.0.hash(state);
	}
}

impl<'a, C: Category, A> Ord for Endomorphism<'a, C, A>
where
	Apply!(brand: C, signature: ('a, A, A)): Ord,
{
	fn cmp(
		&self,
		other: &Self,
	) -> std::cmp::Ordering {
		self.0.cmp(&other.0)
	}
}

impl<'a, C: Category, A> PartialEq for Endomorphism<'a, C, A>
where
	Apply!(brand: C, signature: ('a, A, A)): PartialEq,
{
	fn eq(
		&self,
		other: &Self,
	) -> bool {
		self.0 == other.0
	}
}

impl<'a, C: Category, A> PartialOrd for Endomorphism<'a, C, A>
where
	Apply!(brand: C, signature: ('a, A, A)): PartialOrd,
{
	fn partial_cmp(
		&self,
		other: &Self,
	) -> Option<std::cmp::Ordering> {
		self.0.partial_cmp(&other.0)
	}
}

impl<'a, C: Category, A: 'a> Semigroup for Endomorphism<'a, C, A> {
	/// Composes two endomorphisms.
	///
	/// # Type Signature
	///
	/// `forall a c. Semigroup (Endomorphism c a) => (Endomorphism c a, Endomorphism c a) -> Endomorphism c a`
	///
	/// # Parameters
	///
	/// * `a`: The second morphism to apply.
	/// * `b`: The first morphism to apply.
	///
	/// # Returns
	///
	/// The composed morphism `a . b`.
	///
	/// # Examples
	///
	/// ```
	/// use fp_library::types::endomorphism::Endomorphism;
	/// use fp_library::brands::RcFnBrand;
	/// use fp_library::classes::clonable_fn::ClonableFn;
	/// use fp_library::classes::semigroup::Semigroup;
	///
	/// let f = Endomorphism::<RcFnBrand, _>::new(<RcFnBrand as ClonableFn>::new(|x: i32| x * 2));
	/// let g = Endomorphism::<RcFnBrand, _>::new(<RcFnBrand as ClonableFn>::new(|x: i32| x + 1));
	/// let h = Semigroup::append(f, g);
	/// assert_eq!(h.0(5), 12); // (5 + 1) * 2
	/// ```
	fn append(
		a: Self,
		b: Self,
	) -> Self {
		Self::new(C::compose(a.0, b.0))
	}
}

impl<'a, C: Category, A: 'a> Monoid for Endomorphism<'a, C, A> {
	/// Returns the identity endomorphism.
	///
	/// # Type Signature
	///
	/// `forall a c. Monoid (Endomorphism c a) => () -> Endomorphism c a`
	///
	/// # Returns
	///
	/// The identity morphism.
	///
	/// # Examples
	///
	/// ```
	/// use fp_library::types::endomorphism::Endomorphism;
	/// use fp_library::brands::RcFnBrand;
	/// use fp_library::classes::monoid::Monoid;
	///
	/// let id = Endomorphism::<RcFnBrand, i32>::empty();
	/// assert_eq!(id.0(5), 5);
	/// ```
	fn empty() -> Self {
		Self::new(C::identity())
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		brands::RcFnBrand,
		classes::{clonable_fn::ClonableFn, monoid::empty, semigroup::append},
	};
	use quickcheck_macros::quickcheck;

	// Semigroup Laws

	/// Tests the associativity law for Semigroup.
	#[quickcheck]
	fn semigroup_associativity(val: i32) -> bool {
		let f = Endomorphism::<RcFnBrand, _>::new(<RcFnBrand as ClonableFn>::new(|x: i32| {
			x.wrapping_add(1)
		}));
		let g = Endomorphism::<RcFnBrand, _>::new(<RcFnBrand as ClonableFn>::new(|x: i32| {
			x.wrapping_mul(2)
		}));
		let h = Endomorphism::<RcFnBrand, _>::new(<RcFnBrand as ClonableFn>::new(|x: i32| {
			x.wrapping_sub(3)
		}));

		let lhs = append(f.clone(), append(g.clone(), h.clone()));
		let rhs = append(append(f, g), h);

		lhs.0(val) == rhs.0(val)
	}

	// Monoid Laws

	/// Tests the left identity law for Monoid.
	#[quickcheck]
	fn monoid_left_identity(val: i32) -> bool {
		let f = Endomorphism::<RcFnBrand, _>::new(<RcFnBrand as ClonableFn>::new(|x: i32| {
			x.wrapping_add(1)
		}));
		let id = empty::<Endomorphism<RcFnBrand, i32>>();

		let res = append(id, f.clone());
		res.0(val) == f.0(val)
	}

	/// Tests the right identity law for Monoid.
	#[quickcheck]
	fn monoid_right_identity(val: i32) -> bool {
		let f = Endomorphism::<RcFnBrand, _>::new(<RcFnBrand as ClonableFn>::new(|x: i32| {
			x.wrapping_add(1)
		}));
		let id = empty::<Endomorphism<RcFnBrand, i32>>();

		let res = append(f.clone(), id);
		res.0(val) == f.0(val)
	}
}
