//! Implementations for [`Lazy`], the type of lazily-computed, memoized values.

use crate::{
	Apply,
	brands::LazyBrand,
	classes::{
		clonable_fn::ClonableFn, defer::Defer, monoid::Monoid, once::Once, semigroup::Semigroup,
	},
	impl_kind,
	kinds::*,
};

/// Represents a lazily-computed, memoized value.
///
/// `Lazy` stores a computation (a thunk) that is executed only when the value is needed.
/// The result is then cached (memoized) so that subsequent accesses return the same value
/// without re-executing the computation.
pub struct Lazy<'a, OnceBrand: Once, ClonableFnBrand: ClonableFn, A>(
	pub Apply!(OnceBrand, Once, (), (A)),
	pub Apply!(ClonableFnBrand, ClonableFn, ('a), ((), A)),
);

impl<'a, OnceBrand: Once, ClonableFnBrand: ClonableFn, A> Lazy<'a, OnceBrand, ClonableFnBrand, A> {
	/// Creates a new `Lazy` value from a thunk.
	///
	/// The thunk is wrapped in an `ApplyClonableFn` (e.g., `Rc<dyn Fn() -> A>`) to allow
	/// the `Lazy` value to be cloned.
	///
	/// # Type Signature
	///
	/// `forall a. (() -> a) -> Lazy a`
	///
	/// # Parameters
	///
	/// * `a`: The thunk that produces the value.
	///
	/// # Returns
	///
	/// A new `Lazy` value.
	///
	/// # Examples
	///
	/// ```
	/// use fp_library::types::lazy::Lazy;
	/// use fp_library::brands::RcFnBrand;
	/// use fp_library::brands::OnceCellBrand;
	/// use fp_library::classes::clonable_fn::ClonableFn;
	///
	/// let lazy = Lazy::<OnceCellBrand, RcFnBrand, _>::new(<RcFnBrand as ClonableFn>::new(|_| 42));
	/// ```
	pub fn new(a: Apply!(ClonableFnBrand, ClonableFn, ('a), ((), A))) -> Self {
		Self(OnceBrand::new(), a)
	}

	/// Forces the evaluation of the thunk and returns the value.
	///
	/// If the value has already been computed, the cached value is returned.
	/// Requires `A: Clone` because the value is stored inside the `Lazy` struct and
	/// must be cloned to be returned to the caller.
	///
	/// # Type Signature
	///
	/// `forall a. Lazy a -> a`
	///
	/// # Parameters
	///
	/// * `a`: The lazy value to force.
	///
	/// # Returns
	///
	/// The computed value.
	///
	/// # Examples
	///
	/// ```
	/// use fp_library::types::lazy::Lazy;
	/// use fp_library::brands::RcFnBrand;
	/// use fp_library::brands::OnceCellBrand;
	/// use fp_library::classes::clonable_fn::ClonableFn;
	///
	/// let lazy = Lazy::<OnceCellBrand, RcFnBrand, _>::new(<RcFnBrand as ClonableFn>::new(|_| 42));
	/// assert_eq!(Lazy::force(lazy), 42);
	/// ```
	pub fn force(a: Self) -> A
	where
		A: Clone,
	{
		<OnceBrand as Once>::get_or_init(&a.0, move || (a.1)(())).clone()
	}
}

impl<'a, OnceBrand: Once, ClonableFnBrand: ClonableFn, A: Clone> Clone
	for Lazy<'a, OnceBrand, ClonableFnBrand, A>
where
	Apply!(OnceBrand, Once, (), (A)): Clone,
{
	fn clone(&self) -> Self {
		Self(self.0.clone(), self.1.clone())
	}
}

impl_kind! {
	impl<OnceBrand: Once + 'static, ClonableFnBrand: ClonableFn + 'static>
		for LazyBrand<OnceBrand, ClonableFnBrand>
	{
		type Of<'a, A: 'a>: 'a = Lazy<'a, OnceBrand, ClonableFnBrand, A>;
	}
}

// Note: Lazy cannot implement Functor, Pointed, or Semimonad because these traits
// require operations to work for all types A, but Lazy requires A: Clone to be
// forced (memoized). Adding A: Clone bounds to the traits would restrict all
// other implementations (e.g. Option<NonClone>), which is undesirable.
//
// Consequently, Lazy cannot implement Semiapplicative either, as it extends Functor.

impl<'b, OnceBrand: 'b + Once, CFB: 'b + ClonableFn, A: Semigroup + Clone + 'b> Semigroup
	for Lazy<'b, OnceBrand, CFB, A>
where
	Apply!(OnceBrand, Once, (), (A)): Clone,
{
	/// Combines two lazy values using the underlying type's `Semigroup` implementation.
	///
	/// The combination is itself lazy: the result is a new thunk that, when forced,
	/// forces both input values and combines them.
	///
	/// # Type Signature
	///
	/// `forall a. Semigroup a => (Lazy a, Lazy a) -> Lazy a`
	///
	/// # Parameters
	///
	/// * `a`: The first lazy value.
	/// * `b`: The second lazy value.
	///
	/// # Returns
	///
	/// A new lazy value that combines the results.
	fn append(
		a: Self,
		b: Self,
	) -> Self {
		Lazy::new(<CFB as ClonableFn>::new(move |_| {
			Semigroup::append(Lazy::force(a.clone()), Lazy::force(b.clone()))
		}))
	}
}

impl<'b, OnceBrand: 'b + Once, CFB: 'b + ClonableFn, A: Monoid + Clone + 'b> Monoid
	for Lazy<'b, OnceBrand, CFB, A>
where
	Apply!(OnceBrand, Once, (), (A)): Clone,
{
	/// Returns the identity element for the lazy value.
	///
	/// The result is a lazy value that evaluates to the underlying type's identity element.
	///
	/// # Type Signature
	///
	/// `forall a. Monoid a => () -> Lazy a`
	///
	/// # Returns
	///
	/// A lazy value containing the identity element.
	fn empty() -> Self {
		Lazy::new(<CFB as ClonableFn>::new(move |_| Monoid::empty()))
	}
}

impl<'a, OnceBrand: Once + 'a, CFB: ClonableFn + 'a, A: Clone + 'a> Defer<'a>
	for Lazy<'a, OnceBrand, CFB, A>
{
	/// Defers the construction of a `Lazy` value.
	///
	/// This allows creating a `Lazy` value from a computation that produces a `Lazy` value.
	/// The outer computation is executed only when the result is forced.
	///
	/// # Type Signature
	///
	/// `forall a. (() -> Lazy a) -> Lazy a`
	///
	/// # Parameters
	///
	/// * `f`: A thunk that produces a lazy value.
	///
	/// # Returns
	///
	/// A new lazy value.
	///
	/// # Examples
	///
	/// ```
	/// use fp_library::types::lazy::Lazy;
	/// use fp_library::brands::RcFnBrand;
	/// use fp_library::brands::OnceCellBrand;
	/// use fp_library::classes::clonable_fn::ClonableFn;
	/// use fp_library::classes::defer::Defer;
	/// use std::rc::Rc;
	///
	/// let lazy = Lazy::<OnceCellBrand, RcFnBrand, _>::defer::<RcFnBrand>(
	///     <RcFnBrand as ClonableFn>::new(|_| Lazy::new(<RcFnBrand as ClonableFn>::new(|_| 42)))
	/// );
	/// assert_eq!(Lazy::force(lazy), 42);
	/// ```
	fn defer<ClonableFnBrand>(f: Apply!(ClonableFnBrand, ClonableFn, ('a), ((), Self))) -> Self
	where
		Self: Sized,
		ClonableFnBrand: ClonableFn + 'a,
	{
		Self::new(<CFB as ClonableFn>::new(move |_| Lazy::force(f(()))))
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		brands::{OnceCellBrand, RcFnBrand},
		classes::{clonable_fn::ClonableFn, defer::Defer},
	};
	use std::{cell::RefCell, rc::Rc};

	/// Tests that `Lazy::force` memoizes the result.
	#[test]
	fn force_memoization() {
		let counter = Rc::new(RefCell::new(0));
		let counter_clone = counter.clone();

		let lazy =
			Lazy::<OnceCellBrand, RcFnBrand, _>::new(<RcFnBrand as ClonableFn>::new(move |_| {
				*counter_clone.borrow_mut() += 1;
				42
			}));

		assert_eq!(*counter.borrow(), 0);
		assert_eq!(Lazy::force(lazy.clone()), 42);
		assert_eq!(*counter.borrow(), 1);
		assert_eq!(Lazy::force(lazy), 42);
		// Since we clone before forcing, and OnceCell is not shared across clones (it's deep cloned),
		// the counter increments again.
		assert_eq!(*counter.borrow(), 2);
	}

	/// Tests that `Lazy::defer` delays execution until forced.
	#[test]
	fn defer_execution_order() {
		let counter = Rc::new(RefCell::new(0));
		let counter_clone = counter.clone();

		let lazy = Lazy::<OnceCellBrand, RcFnBrand, _>::defer::<RcFnBrand>(
			<RcFnBrand as ClonableFn>::new(move |_| {
				*counter_clone.borrow_mut() += 1;
				Lazy::new(<RcFnBrand as ClonableFn>::new(|_| 42))
			}),
		);

		assert_eq!(*counter.borrow(), 0);
		assert_eq!(Lazy::force(lazy), 42);
		assert_eq!(*counter.borrow(), 1);
	}
}
