//! Implementations for [`SendEndofunction`], a thread-safe wrapper for endofunctions.

use crate::{
	Apply,
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
pub struct SendEndofunction<'a, CFB: SendClonableFn, A>(
	pub Apply!(brand: CFB, kind: SendClonableFn, output: SendOf, lifetimes: ('a), types: (A, A)),
);

impl<'a, CFB: SendClonableFn, A> SendEndofunction<'a, CFB, A> {
	/// Creates a new `SendEndofunction`.
	///
	/// # Type Signature
	///
	/// `forall a. (a -> a) -> SendEndofunction a`
	///
	/// # Parameters
	///
	/// * `f`: The function to wrap.
	///
	/// # Returns
	///
	/// A new `SendEndofunction`.
	pub fn new(
		f: Apply!(brand: CFB, kind: SendClonableFn, output: SendOf, lifetimes: ('a), types: (A, A))
	) -> Self {
		Self(f)
	}
}

impl<'a, CFB: SendClonableFn, A> Clone for SendEndofunction<'a, CFB, A> {
	fn clone(&self) -> Self {
		Self::new(self.0.clone())
	}
}

impl<'a, CFB: SendClonableFn, A> Debug for SendEndofunction<'a, CFB, A>
where
	Apply!(brand: CFB, kind: SendClonableFn, output: SendOf, lifetimes: ('a), types: (A, A)): Debug,
{
	fn fmt(
		&self,
		fmt: &mut Formatter<'_>,
	) -> fmt::Result {
		fmt.debug_tuple("SendEndofunction").field(&self.0).finish()
	}
}

impl<'a, CFB: SendClonableFn, A> Eq for SendEndofunction<'a, CFB, A> where
	Apply!(brand: CFB, kind: SendClonableFn, output: SendOf, lifetimes: ('a), types: (A, A)): Eq
{
}

impl<'a, CFB: SendClonableFn, A> Hash for SendEndofunction<'a, CFB, A>
where
	Apply!(brand: CFB, kind: SendClonableFn, output: SendOf, lifetimes: ('a), types: (A, A)): Hash,
{
	fn hash<H: std::hash::Hasher>(
		&self,
		state: &mut H,
	) {
		self.0.hash(state);
	}
}

impl<'a, CFB: SendClonableFn, A> Ord for SendEndofunction<'a, CFB, A>
where
	Apply!(brand: CFB, kind: SendClonableFn, output: SendOf, lifetimes: ('a), types: (A, A)): Ord,
{
	fn cmp(
		&self,
		other: &Self,
	) -> std::cmp::Ordering {
		self.0.cmp(&other.0)
	}
}

impl<'a, CFB: SendClonableFn, A> PartialEq for SendEndofunction<'a, CFB, A>
where
	Apply!(brand: CFB, kind: SendClonableFn, output: SendOf, lifetimes: ('a), types: (A, A)):
		PartialEq,
{
	fn eq(
		&self,
		other: &Self,
	) -> bool {
		self.0 == other.0
	}
}

impl<'a, CFB: SendClonableFn, A> PartialOrd for SendEndofunction<'a, CFB, A>
where
	Apply!(brand: CFB, kind: SendClonableFn, output: SendOf, lifetimes: ('a), types: (A, A)):
		PartialOrd,
{
	fn partial_cmp(
		&self,
		other: &Self,
	) -> Option<std::cmp::Ordering> {
		self.0.partial_cmp(&other.0)
	}
}

impl<'a, CFB: 'a + SendClonableFn, A: 'a + Send + Sync> Semigroup for SendEndofunction<'a, CFB, A> {
	/// Composes two endofunctions.
	///
	/// # Type Signature
	///
	/// `forall a. Semigroup (SendEndofunction a) => (SendEndofunction a, SendEndofunction a) -> SendEndofunction a`
	///
	/// # Parameters
	///
	/// * `a`: The second function to apply.
	/// * `b`: The first function to apply.
	///
	/// # Returns
	///
	/// The composed function `a . b`.
	fn append(
		a: Self,
		b: Self,
	) -> Self {
		let f = a.0;
		let g = b.0;
		// Compose: f . g
		Self::new(<CFB as SendClonableFn>::new_send(move |x| f(g(x))))
	}
}

impl<'a, CFB: 'a + SendClonableFn, A: 'a + Send + Sync> Monoid for SendEndofunction<'a, CFB, A> {
	/// Returns the identity endofunction.
	///
	/// # Type Signature
	///
	/// `forall a. Monoid (SendEndofunction a) => () -> SendEndofunction a`
	///
	/// # Returns
	///
	/// The identity function.
	fn empty() -> Self {
		Self::new(<CFB as SendClonableFn>::new_send(identity))
	}
}
