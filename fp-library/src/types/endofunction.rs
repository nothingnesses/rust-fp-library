use crate::{
	classes::{cloneable_fn::CloneableFn, monoid::Monoid, semigroup::Semigroup},
	functions::identity,
};
use fp_macros::doc_params;
use fp_macros::hm_signature;
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
///
/// ### Type Parameters
///
/// * `FnBrand`: The brand of the cloneable function wrapper.
/// * `A`: The input and output type of the function.
///
/// ### Fields
///
/// * `0`: The wrapped function.
///
/// ### Examples
///
/// ```
/// use fp_library::{brands::*, functions::*, types::*};
///
/// let f = Endofunction::<RcFnBrand, _>::new(cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2));
/// assert_eq!(f.0(5), 10);
/// ```
pub struct Endofunction<'a, FnBrand: CloneableFn, A>(pub <FnBrand as CloneableFn>::Of<'a, A, A>);

impl<'a, FnBrand: CloneableFn, A> Endofunction<'a, FnBrand, A> {
	/// Creates a new `Endofunction`.
	///
	/// This function wraps a function `a -> a` in an `Endofunction` struct.
	///
	/// ### Type Signature
	///
	/// `forall fn_brand a. (a -> a) -> Endofunction fn_brand a`
	///
	/// ### Type Parameters
	///
	/// * `FnBrand`: The brand of the function (e.g., `RcFnBrand`).
	/// * `A`: The input and output type of the function.
	///
	/// ### Parameters
	///
	#[doc_params("The function to wrap.")]
	///
	/// ### Returns
	///
	/// A new `Endofunction`.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, functions::*, types::*};
	///
	/// let f = Endofunction::<RcFnBrand, _>::new(cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2));
	/// assert_eq!(f.0(5), 10);
	/// ```
	pub fn new(f: <FnBrand as CloneableFn>::Of<'a, A, A>) -> Self {
		Self(f)
	}
}

impl<'a, FnBrand: CloneableFn, A> Clone for Endofunction<'a, FnBrand, A> {
	fn clone(&self) -> Self {
		Self::new(self.0.clone())
	}
}

impl<'a, FnBrand: CloneableFn, A> Debug for Endofunction<'a, FnBrand, A>
where
	<FnBrand as CloneableFn>::Of<'a, A, A>: Debug,
{
	fn fmt(
		&self,
		fmt: &mut Formatter<'_>,
	) -> fmt::Result {
		fmt.debug_tuple("Endofunction").field(&self.0).finish()
	}
}

impl<'a, FnBrand: CloneableFn, A> Eq for Endofunction<'a, FnBrand, A> where
	<FnBrand as CloneableFn>::Of<'a, A, A>: Eq
{
}

impl<'a, FnBrand: CloneableFn, A> Hash for Endofunction<'a, FnBrand, A>
where
	<FnBrand as CloneableFn>::Of<'a, A, A>: Hash,
{
	fn hash<H: std::hash::Hasher>(
		&self,
		state: &mut H,
	) {
		self.0.hash(state);
	}
}

impl<'a, FnBrand: CloneableFn, A> Ord for Endofunction<'a, FnBrand, A>
where
	<FnBrand as CloneableFn>::Of<'a, A, A>: Ord,
{
	fn cmp(
		&self,
		other: &Self,
	) -> std::cmp::Ordering {
		self.0.cmp(&other.0)
	}
}

impl<'a, FnBrand: CloneableFn, A> PartialEq for Endofunction<'a, FnBrand, A>
where
	<FnBrand as CloneableFn>::Of<'a, A, A>: PartialEq,
{
	fn eq(
		&self,
		other: &Self,
	) -> bool {
		self.0 == other.0
	}
}

impl<'a, FnBrand: CloneableFn, A> PartialOrd for Endofunction<'a, FnBrand, A>
where
	<FnBrand as CloneableFn>::Of<'a, A, A>: PartialOrd,
{
	fn partial_cmp(
		&self,
		other: &Self,
	) -> Option<std::cmp::Ordering> {
		self.0.partial_cmp(&other.0)
	}
}

impl<'a, FnBrand: 'a + CloneableFn, A: 'a> Semigroup for Endofunction<'a, FnBrand, A> {
	/// The result of combining the two values using the semigroup operation.
	///
	/// This method composes two endofunctions into a single endofunction.
	/// Note that `Endofunction` composition is reversed relative to standard function composition:
	/// `append(f, g)` results in `f . g` (read as "f after g"), meaning `g` is applied first, then `f`.
	///
	/// ### Type Signature
	///
	#[hm_signature(Semigroup)]
	///
	/// ### Parameters
	///
	#[doc_params(
		"The second function to apply (the outer function).",
		"The first function to apply (the inner function)."
	)]
	///
	/// ### Returns
	///
	/// The composed function `a . b`.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, functions::*, types::*};
	///
	/// let f = Endofunction::<RcFnBrand, _>::new(cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2));
	/// let g = Endofunction::<RcFnBrand, _>::new(cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x + 1));
	///
	/// // f(g(x)) = (x + 1) * 2
	/// let h = append::<_>(f, g);
	/// assert_eq!(h.0(5), 12);
	/// ```
	fn append(
		a: Self,
		b: Self,
	) -> Self {
		let f = a.0;
		let g = b.0;
		// Compose: f . g
		Self::new(<FnBrand as CloneableFn>::new(move |x| f(g(x))))
	}
}

impl<'a, FnBrand: 'a + CloneableFn, A: 'a> Monoid for Endofunction<'a, FnBrand, A> {
	/// The identity element.
	///
	/// This method returns the identity endofunction, which wraps the identity function.
	///
	/// ### Type Signature
	///
	#[hm_signature(Monoid)]
	///
	/// ### Returns
	///
	/// The identity endofunction.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, functions::*, types::*};
	///
	/// let id = empty::<Endofunction<RcFnBrand, i32>>();
	/// assert_eq!(id.0(5), 5);
	/// ```
	fn empty() -> Self {
		Self::new(<FnBrand as CloneableFn>::new(identity))
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		brands::RcFnBrand,
		classes::{cloneable_fn::CloneableFn, monoid::empty, semigroup::append},
	};
	use quickcheck_macros::quickcheck;

	// Semigroup Laws

	/// Tests the associativity law for Semigroup.
	#[quickcheck]
	fn semigroup_associativity(val: i32) -> bool {
		let f = Endofunction::<RcFnBrand, _>::new(<RcFnBrand as CloneableFn>::new(|x: i32| {
			x.wrapping_add(1)
		}));
		let g = Endofunction::<RcFnBrand, _>::new(<RcFnBrand as CloneableFn>::new(|x: i32| {
			x.wrapping_mul(2)
		}));
		let h = Endofunction::<RcFnBrand, _>::new(<RcFnBrand as CloneableFn>::new(|x: i32| {
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
		let f = Endofunction::<RcFnBrand, _>::new(<RcFnBrand as CloneableFn>::new(|x: i32| {
			x.wrapping_add(1)
		}));
		let id = empty::<Endofunction<RcFnBrand, i32>>();

		let res = append(id, f.clone());
		res.0(val) == f.0(val)
	}

	/// Tests the right identity law for Monoid.
	#[quickcheck]
	fn monoid_right_identity(val: i32) -> bool {
		let f = Endofunction::<RcFnBrand, _>::new(<RcFnBrand as CloneableFn>::new(|x: i32| {
			x.wrapping_add(1)
		}));
		let id = empty::<Endofunction<RcFnBrand, i32>>();

		let res = append(f.clone(), id);
		res.0(val) == f.0(val)
	}
}
