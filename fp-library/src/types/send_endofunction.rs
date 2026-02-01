use fp_macros::doc_params;
use crate::{
	classes::{monoid::Monoid, semigroup::Semigroup, send_cloneable_fn::SendCloneableFn},
	functions::identity,
};
use fp_macros::hm_signature;
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
/// * `FnBrand`: The brand of the thread-safe cloneable function wrapper.
/// * `A`: The input and output type of the function.
///
/// ### Fields
///
/// * `0`: The wrapped thread-safe function.
///
/// ### Examples
///
/// ```
/// use fp_library::{brands::*, functions::*, types::*};
///
/// let f = SendEndofunction::<ArcFnBrand, _>::new(send_cloneable_fn_new::<ArcFnBrand, _, _>(|x: i32| x * 2));
/// assert_eq!(f.0(5), 10);
/// ```
pub struct SendEndofunction<'a, FnBrand: SendCloneableFn, A>(
	pub <FnBrand as SendCloneableFn>::SendOf<'a, A, A>,
);

impl<'a, FnBrand: SendCloneableFn, A> SendEndofunction<'a, FnBrand, A> {
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
	#[doc_params(
		"The function to wrap."
	)]	///
	/// ### Returns
	///
	/// A new `SendEndofunction`.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, functions::*, types::*};
	///
	/// let f = SendEndofunction::<ArcFnBrand, _>::new(send_cloneable_fn_new::<ArcFnBrand, _, _>(|x: i32| x * 2));
	/// assert_eq!(f.0(5), 10);
	/// ```
	pub fn new(f: <FnBrand as SendCloneableFn>::SendOf<'a, A, A>) -> Self {
		Self(f)
	}
}

impl<'a, FnBrand: SendCloneableFn, A> Clone for SendEndofunction<'a, FnBrand, A> {
	fn clone(&self) -> Self {
		Self::new(self.0.clone())
	}
}

impl<'a, FnBrand: SendCloneableFn, A> Debug for SendEndofunction<'a, FnBrand, A>
where
	<FnBrand as SendCloneableFn>::SendOf<'a, A, A>: Debug,
{
	fn fmt(
		&self,
		fmt: &mut Formatter<'_>,
	) -> fmt::Result {
		fmt.debug_tuple("SendEndofunction").field(&self.0).finish()
	}
}

impl<'a, FnBrand: SendCloneableFn, A> Eq for SendEndofunction<'a, FnBrand, A> where
	<FnBrand as SendCloneableFn>::SendOf<'a, A, A>: Eq
{
}

impl<'a, FnBrand: SendCloneableFn, A> Hash for SendEndofunction<'a, FnBrand, A>
where
	<FnBrand as SendCloneableFn>::SendOf<'a, A, A>: Hash,
{
	fn hash<H: std::hash::Hasher>(
		&self,
		state: &mut H,
	) {
		self.0.hash(state);
	}
}

impl<'a, FnBrand: SendCloneableFn, A> Ord for SendEndofunction<'a, FnBrand, A>
where
	<FnBrand as SendCloneableFn>::SendOf<'a, A, A>: Ord,
{
	fn cmp(
		&self,
		other: &Self,
	) -> std::cmp::Ordering {
		self.0.cmp(&other.0)
	}
}

impl<'a, FnBrand: SendCloneableFn, A> PartialEq for SendEndofunction<'a, FnBrand, A>
where
	<FnBrand as SendCloneableFn>::SendOf<'a, A, A>: PartialEq,
{
	fn eq(
		&self,
		other: &Self,
	) -> bool {
		self.0 == other.0
	}
}

impl<'a, FnBrand: SendCloneableFn, A> PartialOrd for SendEndofunction<'a, FnBrand, A>
where
	<FnBrand as SendCloneableFn>::SendOf<'a, A, A>: PartialOrd,
{
	fn partial_cmp(
		&self,
		other: &Self,
	) -> Option<std::cmp::Ordering> {
		self.0.partial_cmp(&other.0)
	}
}

impl<'a, FnBrand: 'a + SendCloneableFn, A: 'a + Send + Sync> Semigroup
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
	#[hm_signature(Semigroup)]
	///
	/// ### Parameters
	///
	#[doc_params(
		"The second function to apply (the outer function).",
		"The first function to apply (the inner function)."
	)]	///
	/// ### Returns
	///
	/// The composed function `a . b`.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, functions::*, types::*};
	///
	/// let f = SendEndofunction::<ArcFnBrand, _>::new(send_cloneable_fn_new::<ArcFnBrand, _, _>(|x: i32| x * 2));
	/// let g = SendEndofunction::<ArcFnBrand, _>::new(send_cloneable_fn_new::<ArcFnBrand, _, _>(|x: i32| x + 1));
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
		Self::new(<FnBrand as SendCloneableFn>::send_cloneable_fn_new(move |x| f(g(x))))
	}
}

impl<'a, FnBrand: 'a + SendCloneableFn, A: 'a + Send + Sync> Monoid
	for SendEndofunction<'a, FnBrand, A>
{
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
	/// let id = empty::<SendEndofunction<ArcFnBrand, i32>>();
	/// assert_eq!(id.0(5), 5);
	/// ```
	fn empty() -> Self {
		Self::new(<FnBrand as SendCloneableFn>::send_cloneable_fn_new(identity))
	}
}
