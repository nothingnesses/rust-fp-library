//! Implementations for [reference-counted][std::rc::Rc] [closures][Fn]
//! (`Rc<dyn Fn(A) -> B>`).

use crate::{
	Apply,
	brands::RcFnBrand,
	classes::{
		category::Category, clonable_fn::ClonableFn, function::Function, semigroupoid::Semigroupoid,
	},
	impl_kind,
	kinds::*,
};
use std::rc::Rc;

impl_kind! {
	for RcFnBrand {
		type Of<'a, A, B> = Rc<dyn 'a + Fn(A) -> B>;
	}
}

impl Function for RcFnBrand {
	type Of<'a, A, B> = Apply!(Self, Kind_fcf9d56b89a0b8b9, ('a), (A, B));

	/// Creates a new `Rc`-wrapped function.
	///
	/// # Type Signature
	///
	/// `forall a b. Function RcFnBrand => (a -> b) -> RcFnBrand a b`
	///
	/// # Parameters
	///
	/// * `f`: The function to wrap.
	///
	/// # Returns
	///
	/// An `Rc`-wrapped function.
	///
	/// # Examples
	///
	/// ```
	/// use fp_library::brands::RcFnBrand;
	/// use fp_library::classes::function::Function;
	///
	/// let f = <RcFnBrand as Function>::new(|x: i32| x * 2);
	/// assert_eq!(f(5), 10);
	/// ```
	fn new<'a, A, B>(f: impl 'a + Fn(A) -> B) -> Apply!(Self, Function, ('a), (A, B)) {
		Rc::new(f)
	}
}

impl ClonableFn for RcFnBrand {
	type Of<'a, A, B> = Apply!(Self, Kind_fcf9d56b89a0b8b9, ('a), (A, B));

	/// Creates a new `Rc`-wrapped clonable function.
	///
	/// # Type Signature
	///
	/// `forall a b. ClonableFn RcFnBrand => (a -> b) -> RcFnBrand a b`
	///
	/// # Parameters
	///
	/// * `f`: The function to wrap.
	///
	/// # Returns
	///
	/// An `Rc`-wrapped clonable function.
	///
	/// # Examples
	///
	/// ```
	/// use fp_library::brands::RcFnBrand;
	/// use fp_library::classes::clonable_fn::ClonableFn;
	///
	/// let f = <RcFnBrand as ClonableFn>::new(|x: i32| x * 2);
	/// assert_eq!(f(5), 10);
	/// ```
	fn new<'a, A, B>(f: impl 'a + Fn(A) -> B) -> Apply!(Self, ClonableFn, ('a), (A, B)) {
		Rc::new(f)
	}
}

impl Semigroupoid for RcFnBrand {
	/// Composes two `Rc`-wrapped functions.
	///
	/// # Type Signature
	///
	/// `forall b c d. Semigroupoid RcFnBrand => (RcFnBrand c d, RcFnBrand b c) -> RcFnBrand b d`
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
	/// use fp_library::brands::RcFnBrand;
	/// use fp_library::classes::semigroupoid::Semigroupoid;
	/// use fp_library::classes::clonable_fn::ClonableFn;
	///
	/// let f = <RcFnBrand as ClonableFn>::new(|x: i32| x * 2);
	/// let g = <RcFnBrand as ClonableFn>::new(|x: i32| x + 1);
	/// let h = RcFnBrand::compose(f, g);
	/// assert_eq!(h(5), 12); // (5 + 1) * 2
	/// ```
	fn compose<'a, B: 'a, C: 'a, D: 'a>(
		f: Apply!(Self, Kind_fcf9d56b89a0b8b9, ('a), (C, D)),
		g: Apply!(Self, Kind_fcf9d56b89a0b8b9, ('a), (B, C)),
	) -> Apply!(Self, Kind_fcf9d56b89a0b8b9, ('a), (B, D)) {
		<Self as ClonableFn>::new(move |b| f(g(b)))
	}
}

impl Category for RcFnBrand {
	/// Returns the identity function wrapped in an `Rc`.
	///
	/// # Type Signature
	///
	/// `forall a. Category RcFnBrand => () -> RcFnBrand a a`
	///
	/// # Returns
	///
	/// The identity function.
	///
	/// # Examples
	///
	/// ```
	/// use fp_library::brands::RcFnBrand;
	/// use fp_library::classes::category::Category;
	///
	/// let id = RcFnBrand::identity::<i32>();
	/// assert_eq!(id(5), 5);
	/// ```
	fn identity<'a, A>() -> Apply!(Self, Kind_fcf9d56b89a0b8b9, ('a), (A, A)) {
		Rc::new(|a| a)
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
		let f = <RcFnBrand as ClonableFn>::new(|x: i32| x.wrapping_add(1));
		let g = <RcFnBrand as ClonableFn>::new(|x: i32| x.wrapping_mul(2));
		let h = <RcFnBrand as ClonableFn>::new(|x: i32| x.wrapping_sub(3));

		let lhs = RcFnBrand::compose(f.clone(), RcFnBrand::compose(g.clone(), h.clone()));
		let rhs = RcFnBrand::compose(RcFnBrand::compose(f, g), h);

		lhs(x) == rhs(x)
	}

	// Category Laws

	/// Tests the left identity law for Category.
	#[quickcheck]
	fn category_left_identity(x: i32) -> bool {
		let f = <RcFnBrand as ClonableFn>::new(|x: i32| x.wrapping_add(1));
		let id = RcFnBrand::identity::<i32>();

		let lhs = RcFnBrand::compose(id, f.clone());
		let rhs = f;

		lhs(x) == rhs(x)
	}

	/// Tests the right identity law for Category.
	#[quickcheck]
	fn category_right_identity(x: i32) -> bool {
		let f = <RcFnBrand as ClonableFn>::new(|x: i32| x.wrapping_add(1));
		let id = RcFnBrand::identity::<i32>();

		let lhs = RcFnBrand::compose(f.clone(), id);
		let rhs = f;

		lhs(x) == rhs(x)
	}
}
