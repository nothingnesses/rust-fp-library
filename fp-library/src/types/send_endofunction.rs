//! SendEndofunction wrapper.
//!
//! This module defines the [`SendEndofunction`] struct, which wraps a thread-safe function from a type to itself (an endofunction)
//! and provides [`Semigroup`] and [`Monoid`] instances based on function composition and identity.

use crate::{
	classes::{monoid::Monoid, semigroup::Semigroup, send_clonable_fn::SendClonableFn},
	functions::identity,
};
use std::{
	fmt::{self, Debug, Formatter},
	hash::Hash,
};

/// A thread-safe wrapper for endofunctions (functions from a set to the same set) that enables monoidal operations.
///
/// `SendEndofunction a` represents a function `a -> a` that is `Send + Sync`.
///
/// It exists to provide a monoid instance where:
///
/// * The binary operation [append][Semigroup::append] is [function composition][crate::functions::compose].
/// * The identity element [empty][Monoid::empty] is the [identity function][crate::functions::identity].
///
/// The wrapped function can be accessed directly via the [`.0` field][SendEndofunction#structfield.0].
///
/// ### Type Parameters
///
/// * `FnBrand`: The brand of the thread-safe clonable function wrapper.
/// * `A`: The input and output type of the function.
///
/// ### Fields
///
/// * `0`: The wrapped thread-safe function.
///
/// ### Examples
///
/// ```
/// use fp_library::functions::*;
/// use fp_library::types::send_endofunction::SendEndofunction;
/// use fp_library::brands::ArcFnBrand;
///
/// let f = SendEndofunction::<ArcFnBrand, _>::new(send_clonable_fn_new::<ArcFnBrand, _, _>(|x: i32| x * 2));
/// assert_eq!(f.0(5), 10);
/// ```
pub struct SendEndofunction<'a, FnBrand: SendClonableFn, A>(
	pub <FnBrand as SendClonableFn>::SendOf<'a, A, A>,
);

impl<'a, FnBrand: SendClonableFn, A> SendEndofunction<'a, FnBrand, A> {
	/// Creates a new `SendEndofunction`.
	///
	/// This function wraps a thread-safe function `a -> a` in a `SendEndofunction` struct.
	///
	/// ### Type Signature
	///
	/// `forall fn_brand a. (a -> a) -> SendEndofunction fn_brand a`
	///
	/// ### Type Parameters
	///
	/// * `FnBrand`: The brand of the function (e.g., `ArcFnBrand`).
	/// * `A`: The input and output type of the function.
	///
	/// ### Parameters
	///
	/// * `f`: The function to wrap.
	///
	/// ### Returns
	///
	/// A new `SendEndofunction`.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::functions::*;
	/// use fp_library::types::send_endofunction::SendEndofunction;
	/// use fp_library::brands::ArcFnBrand;
	///
	/// let f = SendEndofunction::<ArcFnBrand, _>::new(send_clonable_fn_new::<ArcFnBrand, _, _>(|x: i32| x * 2));
	/// assert_eq!(f.0(5), 10);
	/// ```
	pub fn new(f: <FnBrand as SendClonableFn>::SendOf<'a, A, A>) -> Self {
		Self(f)
	}
}

impl<'a, FnBrand: SendClonableFn, A> Clone for SendEndofunction<'a, FnBrand, A> {
	fn clone(&self) -> Self {
		Self::new(self.0.clone())
	}
}

impl<'a, FnBrand: SendClonableFn, A> Debug for SendEndofunction<'a, FnBrand, A>
where
	<FnBrand as SendClonableFn>::SendOf<'a, A, A>: Debug,
{
	fn fmt(
		&self,
		fmt: &mut Formatter<'_>,
	) -> fmt::Result {
		fmt.debug_tuple("SendEndofunction").field(&self.0).finish()
	}
}

impl<'a, FnBrand: SendClonableFn, A> Eq for SendEndofunction<'a, FnBrand, A> where
	<FnBrand as SendClonableFn>::SendOf<'a, A, A>: Eq
{
}

impl<'a, FnBrand: SendClonableFn, A> Hash for SendEndofunction<'a, FnBrand, A>
where
	<FnBrand as SendClonableFn>::SendOf<'a, A, A>: Hash,
{
	fn hash<H: std::hash::Hasher>(
		&self,
		state: &mut H,
	) {
		self.0.hash(state);
	}
}

impl<'a, FnBrand: SendClonableFn, A> Ord for SendEndofunction<'a, FnBrand, A>
where
	<FnBrand as SendClonableFn>::SendOf<'a, A, A>: Ord,
{
	fn cmp(
		&self,
		other: &Self,
	) -> std::cmp::Ordering {
		self.0.cmp(&other.0)
	}
}

impl<'a, FnBrand: SendClonableFn, A> PartialEq for SendEndofunction<'a, FnBrand, A>
where
	<FnBrand as SendClonableFn>::SendOf<'a, A, A>: PartialEq,
{
	fn eq(
		&self,
		other: &Self,
	) -> bool {
		self.0 == other.0
	}
}

impl<'a, FnBrand: SendClonableFn, A> PartialOrd for SendEndofunction<'a, FnBrand, A>
where
	<FnBrand as SendClonableFn>::SendOf<'a, A, A>: PartialOrd,
{
	fn partial_cmp(
		&self,
		other: &Self,
	) -> Option<std::cmp::Ordering> {
		self.0.partial_cmp(&other.0)
	}
}

impl<'a, FnBrand: 'a + SendClonableFn, A: 'a + Send + Sync> Semigroup
	for SendEndofunction<'a, FnBrand, A>
{
	/// The result of combining the two values using the semigroup operation.
	///
	/// This method combines two endofunctions into a single endofunction.
	/// Note that `SendEndofunction` composition is reversed relative to standard function composition:
	/// `append(f, g)` results in `f . g` (read as "f after g"), meaning `g` is applied first, then `f`.
	///
	/// ### Type Signature
	///
	/// `forall fn_brand a. Semigroup (SendEndofunction fn_brand a) => (SendEndofunction fn_brand a, SendEndofunction fn_brand a) -> SendEndofunction fn_brand a`
	///
	/// ### Parameters
	///
	/// * `a`: The second function to apply (the outer function).
	/// * `b`: The first function to apply (the inner function).
	///
	/// ### Returns
	///
	/// The composed function `a . b`.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::functions::*;
	/// use fp_library::types::send_endofunction::SendEndofunction;
	/// use fp_library::brands::ArcFnBrand;
	///
	/// let f = SendEndofunction::<ArcFnBrand, _>::new(send_clonable_fn_new::<ArcFnBrand, _, _>(|x: i32| x * 2));
	/// let g = SendEndofunction::<ArcFnBrand, _>::new(send_clonable_fn_new::<ArcFnBrand, _, _>(|x: i32| x + 1));
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
		Self::new(<FnBrand as SendClonableFn>::send_clonable_fn_new(move |x| f(g(x))))
	}
}

impl<'a, FnBrand: 'a + SendClonableFn, A: 'a + Send + Sync> Monoid
	for SendEndofunction<'a, FnBrand, A>
{
	/// The identity element.
	///
	/// This method returns the identity endofunction, which wraps the identity function.
	///
	/// ### Type Signature
	///
	/// `forall fn_brand a. Monoid (SendEndofunction fn_brand a) => () -> SendEndofunction fn_brand a`
	///
	/// ### Returns
	///
	/// The identity endofunction.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::functions::*;
	/// use fp_library::types::send_endofunction::SendEndofunction;
	/// use fp_library::brands::ArcFnBrand;
	///
	/// let id = empty::<SendEndofunction<ArcFnBrand, i32>>();
	/// assert_eq!(id.0(5), 5);
	/// ```
	fn empty() -> Self {
		Self::new(<FnBrand as SendClonableFn>::send_clonable_fn_new(identity))
	}
}
