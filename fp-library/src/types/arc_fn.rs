//! Atomically reference-counted function wrapper.
//!
//! This module defines the [`ArcFnBrand`] struct, which provides implementations for atomically reference-counted closures (`Arc<dyn Fn(A) -> B>`).
//! It implements [`Function`], [`ClonableFn`], [`SendClonableFn`], [`Semigroupoid`], and [`Category`].

use crate::{
	Apply,
	brands::ArcFnBrand,
	classes::{
		category::Category, clonable_fn::ClonableFn, function::Function,
		semigroupoid::Semigroupoid, send_clonable_fn::SendClonableFn,
	},
	impl_kind,
	kinds::*,
};
use std::sync::Arc;

impl_kind! {
	for ArcFnBrand {
		type Of<'a, A, B> = Arc<dyn 'a + Fn(A) -> B>;
	}
}

impl Function for ArcFnBrand {
	type Of<'a, A, B> = Apply!(<Self as trait { type Of<'a, T, U>; }>::Of<'a, A, B>);

	/// Creates a new function wrapper.
	///
	/// This function wraps the provided closure `f` into an `Arc`-wrapped function.
	///
	/// ### Type Signature
	///
	/// `forall a b. Function ArcFnBrand => (a -> b) -> ArcFnBrand a b`
	///
	/// ### Type Parameters
	///
	/// * `A`: The input type of the function.
	/// * `B`: The output type of the function.
	///
	/// ### Parameters
	///
	/// * `f`: The closure to wrap.
	///
	/// ### Returns
	///
	/// The wrapped function.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::brands::ArcFnBrand;
	/// use fp_library::classes::function::Function;
	///
	/// let f = <ArcFnBrand as Function>::new(|x: i32| x * 2);
	/// assert_eq!(f(5), 10);
	/// ```
	fn new<'a, A, B>(f: impl 'a + Fn(A) -> B) -> <Self as Function>::Of<'a, A, B> {
		Arc::new(f)
	}
}

impl ClonableFn for ArcFnBrand {
	type Of<'a, A, B> = Apply!(<Self as trait { type Of<'a, T, U>; }>::Of<'a, A, B>);

	/// Creates a new clonable function wrapper.
	///
	/// This function wraps the provided closure `f` into an `Arc`-wrapped clonable function.
	///
	/// ### Type Signature
	///
	/// `forall a b. ClonableFn ArcFnBrand => (a -> b) -> ArcFnBrand a b`
	///
	/// ### Type Parameters
	///
	/// * `A`: The input type of the function.
	/// * `B`: The output type of the function.
	///
	/// ### Parameters
	///
	/// * `f`: The closure to wrap.
	///
	/// ### Returns
	///
	/// The wrapped clonable function.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::brands::ArcFnBrand;
	/// use fp_library::classes::clonable_fn::ClonableFn;
	///
	/// let f = <ArcFnBrand as ClonableFn>::new(|x: i32| x * 2);
	/// assert_eq!(f(5), 10);
	/// ```
	fn new<'a, A, B>(f: impl 'a + Fn(A) -> B) -> <Self as ClonableFn>::Of<'a, A, B> {
		Arc::new(f)
	}
}

impl SendClonableFn for ArcFnBrand {
	type SendOf<'a, A, B> = Arc<dyn 'a + Fn(A) -> B + Send + Sync>;

	/// Creates a new thread-safe clonable function wrapper.
	///
	/// This method wraps a closure into an `Arc`-wrapped thread-safe clonable function.
	///
	/// ### Type Signature
	///
	/// `forall a b. SendClonableFn ArcFnBrand => (a -> b) -> ArcFnBrand a b`
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
	/// use fp_library::brands::ArcFnBrand;
	/// use fp_library::classes::send_clonable_fn::SendClonableFn;
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
	) -> <Self as SendClonableFn>::SendOf<'a, A, B> {
		Arc::new(f)
	}
}

impl Semigroupoid for ArcFnBrand {
	/// Takes morphisms `f` and `g` and returns the morphism `f . g` (`f` composed with `g`).
	///
	/// This method composes two `Arc`-wrapped functions `f` and `g` to produce a new function that represents the application of `g` followed by `f`.
	///
	/// ### Type Signature
	///
	/// `forall b c d. Semigroupoid ArcFnBrand => (ArcFnBrand c d, ArcFnBrand b c) -> ArcFnBrand b d`
	///
	/// ### Type Parameters
	///
	/// * `B`: The source type of the first morphism.
	/// * `C`: The target type of the first morphism and the source type of the second morphism.
	/// * `D`: The target type of the second morphism.
	///
	/// ### Parameters
	///
	/// * `f`: The second morphism to apply (from C to D).
	/// * `g`: The first morphism to apply (from B to C).
	///
	/// ### Returns
	///
	/// The composed morphism (from B to D).
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::brands::ArcFnBrand;
	/// use fp_library::classes::semigroupoid::Semigroupoid;
	/// use fp_library::classes::clonable_fn::ClonableFn;
	///
	/// let f = <ArcFnBrand as ClonableFn>::new(|x: i32| x * 2);
	/// let g = <ArcFnBrand as ClonableFn>::new(|x: i32| x + 1);
	/// let h = ArcFnBrand::compose(f, g);
	/// assert_eq!(h(5), 12); // (5 + 1) * 2
	/// ```
	fn compose<'a, B: 'a, C: 'a, D: 'a>(
		f: Apply!(<Self as trait { type Of<'a, T, U>; }>::Of<'a, C, D>),
		g: Apply!(<Self as trait { type Of<'a, T, U>; }>::Of<'a, B, C>),
	) -> Apply!(<Self as trait { type Of<'a, T, U>; }>::Of<'a, B, D>) {
		<Self as ClonableFn>::new(move |b| f(g(b)))
	}
}

impl Category for ArcFnBrand {
	/// Returns the identity morphism.
	///
	/// The identity morphism is a function that maps every object to itself, wrapped in an `Arc`.
	///
	/// ### Type Signature
	///
	/// `forall a. Category ArcFnBrand => () -> ArcFnBrand a a`
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
	/// use fp_library::brands::ArcFnBrand;
	/// use fp_library::classes::category::Category;
	///
	/// let id = ArcFnBrand::identity::<i32>();
	/// assert_eq!(id(5), 5);
	/// ```
	fn identity<'a, A>() -> Apply!(<Self as trait { type Of<'a, T, U>; }>::Of<'a, A, A>) {
		Arc::new(|a| a)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::classes::{
		category::Category, clonable_fn::ClonableFn, semigroupoid::Semigroupoid,
		send_clonable_fn::SendClonableFn,
	};
	use quickcheck_macros::quickcheck;
	use std::thread;

	// SendClonableFn Tests

	/// Tests that `new_send` creates a callable function.
	#[test]
	fn new_send_callable() {
		let f = <ArcFnBrand as SendClonableFn>::new_send(|x: i32| x * 2);
		assert_eq!(f(5), 10);
	}

	/// Tests that the function can be cloned.
	#[test]
	fn send_clonable_clone() {
		let f = <ArcFnBrand as SendClonableFn>::new_send(|x: i32| x * 2);
		let g = f.clone();
		assert_eq!(g(5), 10);
	}

	/// Tests that `SendOf` is `Send` (can be sent to another thread).
	#[test]
	fn send_of_is_send() {
		let f = <ArcFnBrand as SendClonableFn>::new_send(|x: i32| x * 2);
		let handle = thread::spawn(move || f(5));
		assert_eq!(handle.join().unwrap(), 10);
	}

	/// Tests that `SendOf` is `Sync` (can be shared across threads).
	#[test]
	fn send_of_is_sync() {
		let f = <ArcFnBrand as SendClonableFn>::new_send(|x: i32| x * 2);
		let f_clone = f.clone();
		let handle = thread::spawn(move || f_clone(5));
		assert_eq!(f(5), 10);
		assert_eq!(handle.join().unwrap(), 10);
	}

	// Semigroupoid Laws

	/// Tests the associativity law for Semigroupoid.
	#[quickcheck]
	fn semigroupoid_associativity(x: i32) -> bool {
		let f = <ArcFnBrand as ClonableFn>::new(|x: i32| x.wrapping_add(1));
		let g = <ArcFnBrand as ClonableFn>::new(|x: i32| x.wrapping_mul(2));
		let h = <ArcFnBrand as ClonableFn>::new(|x: i32| x.wrapping_sub(3));

		let lhs = ArcFnBrand::compose(f.clone(), ArcFnBrand::compose(g.clone(), h.clone()));
		let rhs = ArcFnBrand::compose(ArcFnBrand::compose(f, g), h);

		lhs(x) == rhs(x)
	}

	// Category Laws

	/// Tests the left identity law for Category.
	#[quickcheck]
	fn category_left_identity(x: i32) -> bool {
		let f = <ArcFnBrand as ClonableFn>::new(|x: i32| x.wrapping_add(1));
		let id = ArcFnBrand::identity::<i32>();

		let lhs = ArcFnBrand::compose(id, f.clone());
		let rhs = f;

		lhs(x) == rhs(x)
	}

	/// Tests the right identity law for Category.
	#[quickcheck]
	fn category_right_identity(x: i32) -> bool {
		let f = <ArcFnBrand as ClonableFn>::new(|x: i32| x.wrapping_add(1));
		let id = ArcFnBrand::identity::<i32>();

		let lhs = ArcFnBrand::compose(f.clone(), id);
		let rhs = f;

		lhs(x) == rhs(x)
	}
}
