//! Implementations for [atomically reference-counted][std::sync::Arc]
//! [closures][Fn] (`Arc<dyn Fn(A) -> B>`).

use crate::{
	Apply,
	brands::ArcFnBrand,
	classes::{
		category::Category, clonable_fn::ClonableFn, function::Function, semigroupoid::Semigroupoid,
		send_clonable_fn::SendClonableFn,
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
	type Of<'a, A, B> = Apply!(brand: Self, signature: ('a, A, B));

	/// Creates a new `Arc`-wrapped function.
	///
	/// # Type Signature
	///
	/// `forall a b. Function ArcFnBrand => (a -> b) -> ArcFnBrand a b`
	///
	/// # Parameters
	///
	/// * `f`: The function to wrap.
	///
	/// # Returns
	///
	/// An `Arc`-wrapped function.
	///
	/// # Examples
	///
	/// ```
	/// use fp_library::brands::ArcFnBrand;
	/// use fp_library::classes::function::Function;
	///
	/// let f = <ArcFnBrand as Function>::new(|x: i32| x * 2);
	/// assert_eq!(f(5), 10);
	/// ```
	fn new<'a, A, B>(
		f: impl 'a + Fn(A) -> B
	) -> Apply!(brand: Self, kind: Function, lifetimes: ('a), types: (A, B)) {
		Arc::new(f)
	}
}

impl ClonableFn for ArcFnBrand {
	type Of<'a, A, B> = Apply!(brand: Self, signature: ('a, A, B));

	/// Creates a new clonable function wrapper.
	///
	/// # Type Signature
	///
	/// `forall a b. ClonableFn ArcFnBrand => (a -> b) -> ArcFnBrand a b`
	///
	/// # Parameters
	///
	/// * `f`: The function to wrap.
	///
	/// # Returns
	///
	/// An `Arc`-wrapped clonable function.
	///
	/// # Examples
	///
	/// ```
	/// use fp_library::brands::ArcFnBrand;
	/// use fp_library::classes::clonable_fn::ClonableFn;
	///
	/// let f = <ArcFnBrand as ClonableFn>::new(|x: i32| x * 2);
	/// assert_eq!(f(5), 10);
	/// ```
	fn new<'a, A, B>(
		f: impl 'a + Fn(A) -> B
	) -> Apply!(brand: Self, kind: ClonableFn, lifetimes: ('a), types: (A, B)) {
		Arc::new(f)
	}
}

impl SendClonableFn for ArcFnBrand {
	type SendOf<'a, A, B> = Arc<dyn 'a + Fn(A) -> B + Send + Sync>;

	/// Creates a new thread-safe clonable function wrapper.
	///
	/// # Type Signature
	///
	/// `forall a b. SendClonableFn ArcFnBrand => (a -> b) -> ArcFnBrand a b`
	///
	/// # Parameters
	///
	/// * `f`: The function to wrap. Must be `Send + Sync`.
	///
	/// # Returns
	///
	/// An `Arc`-wrapped thread-safe clonable function.
	///
	/// # Examples
	///
	/// ```
	/// use fp_library::brands::ArcFnBrand;
	/// use fp_library::classes::send_clonable_fn::SendClonableFn;
	/// use std::thread;
	///
	/// let f = <ArcFnBrand as SendClonableFn>::new_send(|x: i32| x * 2);
	///
	/// let handle = thread::spawn(move || {
	///     assert_eq!(f(5), 10);
	/// });
	/// handle.join().unwrap();
	/// ```
	fn new_send<'a, A, B>(
		f: impl 'a + Fn(A) -> B + Send + Sync,
	) -> Apply!(brand: Self, kind: SendClonableFn, output: SendOf, lifetimes: ('a), types: (A, B)) {
		Arc::new(f)
	}
}

impl Semigroupoid for ArcFnBrand {
	/// Composes two `Arc`-wrapped functions.
	///
	/// # Type Signature
	///
	/// `forall b c d. Semigroupoid ArcFnBrand => (ArcFnBrand c d, ArcFnBrand b c) -> ArcFnBrand b d`
	///
	/// # Parameters
	///
	/// * `f`: The second function to apply.
	/// * `g`: The first function to apply.
	///
	/// # Returns
	///
	/// The composed function `f . g`.
	///
	/// # Examples
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
		f: Apply!(brand: Self, signature: ('a, C, D)),
		g: Apply!(brand: Self, signature: ('a, B, C)),
	) -> Apply!(brand: Self, signature: ('a, B, D)) {
		<Self as ClonableFn>::new(move |b| f(g(b)))
	}
}

impl Category for ArcFnBrand {
	/// Returns the identity function wrapped in an `Arc`.
	///
	/// # Type Signature
	///
	/// `forall a. Category ArcFnBrand => () -> ArcFnBrand a a`
	///
	/// # Returns
	///
	/// The identity function.
	///
	/// # Examples
	///
	/// ```
	/// use fp_library::brands::ArcFnBrand;
	/// use fp_library::classes::category::Category;
	///
	/// let id = ArcFnBrand::identity::<i32>();
	/// assert_eq!(id(5), 5);
	/// ```
	fn identity<'a, A>() -> Apply!(brand: Self, signature: ('a, A, A)) {
		Arc::new(|a| a)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::classes::{category::Category, clonable_fn::ClonableFn, semigroupoid::Semigroupoid};
	use quickcheck_macros::quickcheck;

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
