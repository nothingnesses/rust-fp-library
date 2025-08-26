//! Implementations for [`Endomorphism`], a wrapper for endomorphisms (functions from a type to itself) that enables monoidal operations.

use crate::{
	functions::{compose, identity},
	typeclasses::{ClonableFn, Monoid, Semigroup, clonable_fn::ApplyFn},
};

#[derive(Clone)]
/// A wrapper for endomorphisms (functions from a type to itself) that enables monoidal operations.
///
/// `Endomorphism<A>` represents a function `A -> A`.
///
/// It exists to provide a monoid instance where:
///
/// * The binary operation (`append`) is function composition.
/// * The identity element (`empty`) is the identity function.
///
/// This allows combining transformations in a composable, associative way with a clear identity,
/// which is useful for building pipelines of transformations or accumulating operations.
///
/// The wrapped function can be accessed directly via the `.0` field.
///
/// # Examples
///
/// ```
/// use fp_library::{
///     brands::RcFnBrand,
///     functions::{append, empty},
///     typeclasses::ClonableFn,
///     types::endomorphism::Endomorphism
/// };
/// use std::rc::Rc;
///
/// // Create endomorphisms
/// let f = Endomorphism(<RcFnBrand as ClonableFn>::new(|x: i32| x * 2));
/// let g = Endomorphism(<RcFnBrand as ClonableFn>::new(|x: i32| x + 1));
///
/// // Compose functions (f after g)
/// let fg = append::<RcFnBrand, Endomorphism<'_, RcFnBrand, i32>>(f)(g);
/// assert_eq!(fg.0(3), 8); // double(increment(3)) = 8
///
/// // Identity element
/// let id = empty::<Endomorphism<'_, RcFnBrand, i32>>();
/// assert_eq!(id.0(42), 42);
/// ```
pub struct Endomorphism<'a, ClonableFnBrand: ClonableFn, A: 'a>(
	pub ApplyFn<'a, ClonableFnBrand, A, A>,
);

impl<'b, ClonableFnBrandSelf: ClonableFn + 'b, A: 'b> Semigroup
	for Endomorphism<'b, ClonableFnBrandSelf, A>
{
	/// # Examples
	///
	/// ```
	/// use fp_library::{
	///     brands::RcFnBrand,
	///     functions::append,
	///     typeclasses::ClonableFn,
	///     types::endomorphism::Endomorphism
	/// };
	/// use std::rc::Rc;
	///
	/// let double = <RcFnBrand as ClonableFn>::new(|x: i32| x * 2);
	/// let increment = <RcFnBrand as ClonableFn>::new(|x: i32| x + 1);
	///
	/// assert_eq!(
	///     (append::<RcFnBrand, Endomorphism<'static, RcFnBrand, i32>>(Endomorphism(double))(Endomorphism(increment.clone()))).0(3),
	///     8
	/// );
	/// assert_eq!(
	///     (append::<RcFnBrand, Endomorphism<'static, RcFnBrand, i32>>(Endomorphism(increment.clone()))(Endomorphism(increment))).0(3),
	///     5
	/// );
	/// ```
	fn append<'a, ClonableFnBrand: 'a + ClonableFn>(
		a: Self
	) -> ApplyFn<'a, ClonableFnBrand, Self, Self>
	where
		Self: Sized,
	{
		ClonableFnBrand::new(move |b: Self| {
			Endomorphism(compose::<ClonableFnBrandSelf, _, _, _>(a.0.clone())(b.0))
		})
	}
}

impl<'a, ClonableFnBrand: ClonableFn + 'a, A: 'a> Monoid for Endomorphism<'a, ClonableFnBrand, A> {
	/// # Examples
	///
	/// ```
	/// use fp_library::{brands::RcFnBrand, functions::empty, types::Endomorphism};
	///
	/// assert_eq!(empty::<Endomorphism<'static, RcFnBrand, i32>>().0(5), 5);
	/// assert_eq!(empty::<Endomorphism<'static, RcFnBrand, String>>().0("test".to_string()), "test");
	/// ```
	fn empty() -> Self {
		Endomorphism(ClonableFnBrand::new(identity))
	}
}
