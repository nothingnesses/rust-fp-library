//! Thread-safe clonable function wrappers.
//!
//! This module defines the [`SendClonableFn`] trait, which provides an abstraction for thread-safe clonable wrappers over closures.

use super::clonable_fn::ClonableFn;
use crate::Apply;
use std::ops::Deref;

/// Abstraction for thread-safe clonable wrappers over closures.
///
/// This trait extends [`ClonableFn`] to enforce `Send + Sync` bounds on the
/// wrapped closure and the wrapper itself. This is implemented by types like
/// [`ArcFnBrand`][crate::brands::ArcFnBrand] but not [`RcFnBrand`][crate::brands::RcFnBrand].
///
/// The lifetime `'a` ensures the function doesn't outlive referenced data,
/// while generic types `A` and `B` represent the input and output types, respectively.
pub trait SendClonableFn: ClonableFn {
	type SendOf<'a, A, B>: Clone + Send + Sync + Deref<Target = dyn 'a + Fn(A) -> B + Send + Sync>;

	/// Creates a new thread-safe clonable function wrapper.
	///
	/// This method wraps a closure into a thread-safe clonable function wrapper.
	///
	/// ### Type Signature
	///
	/// `forall a b. SendClonableFn f => (a -> b) -> f a b`
	///
	/// ### Type Parameters
	///
	/// * `A`: The input type of the function.
	/// * `B`: The output type of the function.
	///
	/// ### Parameters
	///
	/// * `f`: The closure to wrap. Must be `Send + Sync`.
	///
	/// ### Returns
	///
	/// The wrapped thread-safe clonable function.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::classes::send_clonable_fn::SendClonableFn;
	/// use fp_library::brands::ArcFnBrand;
	/// use std::thread;
	///
	/// let f = <ArcFnBrand as SendClonableFn>::new_send(|x: i32| x * 2);
	///
	/// // Can be sent to another thread
	/// let handle = thread::spawn(move || {
	///     assert_eq!(f(5), 10);
	/// });
	/// handle.join().unwrap();
	/// ```
	fn new_send<'a, A, B>(
		f: impl 'a + Fn(A) -> B + Send + Sync
	) -> Apply!(brand: Self, kind: SendClonableFn, output: SendOf, lifetimes: ('a), types: (A, B));
}

/// Creates a new thread-safe clonable function wrapper.
///
/// Free function version that dispatches to [the type class' associated function][`SendClonableFn::new_send`].
///
/// ### Type Signature
///
/// `forall a b. SendClonableFn f => (a -> b) -> f a b`
///
/// ### Type Parameters
///
/// * `F`: The brand of the thread-safe clonable function wrapper.
/// * `A`: The input type of the function.
/// * `B`: The output type of the function.
///
/// ### Parameters
///
/// * `f`: The closure to wrap. Must be `Send + Sync`.
///
/// ### Returns
///
/// The wrapped thread-safe clonable function.
///
/// ### Examples
///
/// ```
/// use fp_library::classes::send_clonable_fn::new_send;
/// use fp_library::brands::ArcFnBrand;
/// use std::thread;
///
/// let f = new_send::<ArcFnBrand, _, _>(|x: i32| x * 2);
///
/// // Can be sent to another thread
/// let handle = thread::spawn(move || {
///     assert_eq!(f(5), 10);
/// });
/// handle.join().unwrap();
/// ```
pub fn new_send<'a, F, A, B>(
	f: impl 'a + Fn(A) -> B + Send + Sync
) -> Apply!(brand: F, kind: SendClonableFn, output: SendOf, lifetimes: ('a), types: (A, B))
where
	F: SendClonableFn,
{
	<F as SendClonableFn>::new_send(f)
}
