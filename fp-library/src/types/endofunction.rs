//! Implementations for [`Endofunction`], a wrapper for endofunctions (functions from a set to the same set) that enables monoidal operations.

use crate::{
	Apply,
	classes::{clonable_fn::ClonableFn, monoid::Monoid, semigroup::Semigroup},
	functions::identity,
};
use std::{
	fmt::{self, Debug, Formatter},
	hash::Hash,
};

/// A wrapper for endofunctions (functions from a set to the same set) that enables monoidal operations.
///
/// `Endofunction a` represents a function `a -> a`.
///
/// It exists to provide a monoid instance where:
///
/// * The binary operation [append][Semigroup::append] is [function composition][crate::functions::compose].
/// * The identity element [empty][Monoid::empty] is the [identity function][crate::functions::identity].
///
/// The wrapped function can be accessed directly via the [`.0` field][Endofunction#structfield.0].
pub struct Endofunction<'a, FnBrand: ClonableFn, A>(
	pub Apply!(brand: FnBrand, kind: ClonableFn, lifetimes: ('a), types: (A, A)),
);

impl<'a, FnBrand: ClonableFn, A> Endofunction<'a, FnBrand, A> {
	/// Creates a new `Endofunction`.
	///
	/// # Type Signature
	///
	/// `forall a. (a -> a) -> Endofunction a`
	///
	/// # Parameters
	///
	/// * `f`: The function to wrap.
	///
	/// # Returns
	///
	/// A new `Endofunction`.
	pub fn new(
		f: Apply!(brand: FnBrand, kind: ClonableFn, lifetimes: ('a), types: (A, A))
	) -> Self {
		Self(f)
	}
}

impl<'a, FnBrand: ClonableFn, A> Clone for Endofunction<'a, FnBrand, A> {
	fn clone(&self) -> Self {
		Self::new(self.0.clone())
	}
}

impl<'a, FnBrand: ClonableFn, A> Debug for Endofunction<'a, FnBrand, A>
where
	Apply!(brand: FnBrand, kind: ClonableFn, lifetimes: ('a), types: (A, A)): Debug,
{
	fn fmt(
		&self,
		fmt: &mut Formatter<'_>,
	) -> fmt::Result {
		fmt.debug_tuple("Endofunction").field(&self.0).finish()
	}
}

impl<'a, FnBrand: ClonableFn, A> Eq for Endofunction<'a, FnBrand, A> where
	Apply!(brand: FnBrand, kind: ClonableFn, lifetimes: ('a), types: (A, A)): Eq
{
}

impl<'a, FnBrand: ClonableFn, A> Hash for Endofunction<'a, FnBrand, A>
where
	Apply!(brand: FnBrand, kind: ClonableFn, lifetimes: ('a), types: (A, A)): Hash,
{
	fn hash<H: std::hash::Hasher>(
		&self,
		state: &mut H,
	) {
		self.0.hash(state);
	}
}

impl<'a, FnBrand: ClonableFn, A> Ord for Endofunction<'a, FnBrand, A>
where
	Apply!(brand: FnBrand, kind: ClonableFn, lifetimes: ('a), types: (A, A)): Ord,
{
	fn cmp(
		&self,
		other: &Self,
	) -> std::cmp::Ordering {
		self.0.cmp(&other.0)
	}
}

impl<'a, FnBrand: ClonableFn, A> PartialEq for Endofunction<'a, FnBrand, A>
where
	Apply!(brand: FnBrand, kind: ClonableFn, lifetimes: ('a), types: (A, A)): PartialEq,
{
	fn eq(
		&self,
		other: &Self,
	) -> bool {
		self.0 == other.0
	}
}

impl<'a, FnBrand: ClonableFn, A> PartialOrd for Endofunction<'a, FnBrand, A>
where
	Apply!(brand: FnBrand, kind: ClonableFn, lifetimes: ('a), types: (A, A)): PartialOrd,
{
	fn partial_cmp(
		&self,
		other: &Self,
	) -> Option<std::cmp::Ordering> {
		self.0.partial_cmp(&other.0)
	}
}

impl<'a, FnBrand: 'a + ClonableFn, A: 'a> Semigroup for Endofunction<'a, FnBrand, A> {
	/// Composes two endofunctions.
	///
	/// # Type Signature
	///
	/// `forall a. Semigroup (Endofunction a) => (Endofunction a, Endofunction a) -> Endofunction a`
	///
	/// # Parameters
	///
	/// * `a`: The second function to apply.
	/// * `b`: The first function to apply.
	///
	/// # Returns
	///
	/// The composed function `a . b`.
	///
	/// # Examples
	///
	/// ```
	/// use fp_library::types::endofunction::Endofunction;
	/// use fp_library::brands::RcFnBrand;
	/// use fp_library::classes::clonable_fn::ClonableFn;
	/// use fp_library::classes::semigroup::Semigroup;
	///
	/// let f = Endofunction::<RcFnBrand, _>::new(<RcFnBrand as ClonableFn>::new(|x: i32| x * 2));
	/// let g = Endofunction::<RcFnBrand, _>::new(<RcFnBrand as ClonableFn>::new(|x: i32| x + 1));
	/// let h = Semigroup::append(f, g);
	/// assert_eq!(h.0(5), 12); // (5 + 1) * 2
	/// ```
	fn append(
		a: Self,
		b: Self,
	) -> Self {
		let f = a.0;
		let g = b.0;
		// Compose: f . g
		Self::new(<FnBrand as ClonableFn>::new(move |x| f(g(x))))
	}
}

impl<'a, FnBrand: 'a + ClonableFn, A: 'a> Monoid for Endofunction<'a, FnBrand, A> {
	/// Returns the identity endofunction.
	///
	/// # Type Signature
	///
	/// `forall a. Monoid (Endofunction a) => () -> Endofunction a`
	///
	/// # Returns
	///
	/// The identity function.
	///
	/// # Examples
	///
	/// ```
	/// use fp_library::types::endofunction::Endofunction;
	/// use fp_library::brands::RcFnBrand;
	/// use fp_library::classes::monoid::Monoid;
	///
	/// let id = Endofunction::<RcFnBrand, i32>::empty();
	/// assert_eq!(id.0(5), 5);
	/// ```
	fn empty() -> Self {
		Self::new(<FnBrand as ClonableFn>::new(identity))
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
		let f = Endofunction::<RcFnBrand, _>::new(<RcFnBrand as ClonableFn>::new(|x: i32| {
			x.wrapping_add(1)
		}));
		let g = Endofunction::<RcFnBrand, _>::new(<RcFnBrand as ClonableFn>::new(|x: i32| {
			x.wrapping_mul(2)
		}));
		let h = Endofunction::<RcFnBrand, _>::new(<RcFnBrand as ClonableFn>::new(|x: i32| {
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
		let f = Endofunction::<RcFnBrand, _>::new(<RcFnBrand as ClonableFn>::new(|x: i32| {
			x.wrapping_add(1)
		}));
		let id = empty::<Endofunction<RcFnBrand, i32>>();

		let res = append(id, f.clone());
		res.0(val) == f.0(val)
	}

	/// Tests the right identity law for Monoid.
	#[quickcheck]
	fn monoid_right_identity(val: i32) -> bool {
		let f = Endofunction::<RcFnBrand, _>::new(<RcFnBrand as ClonableFn>::new(|x: i32| {
			x.wrapping_add(1)
		}));
		let id = empty::<Endofunction<RcFnBrand, i32>>();

		let res = append(f.clone(), id);
		res.0(val) == f.0(val)
	}
}
