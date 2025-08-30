//! Implementations for [`Endomorphism`], a wrapper for endomorphisms (functions from a type to the same type) that enables monoidal operations.

use crate::{
	classes::{
		Category, ClonableFn, Monoid, Semigroup, Semigroupoid, SmartPointer, clonable_fn::ApplyFn,
		smart_pointer::SmartPointerInner,
	},
	hkt::Apply1L2T,
	types::rc::RcBrand,
};

#[derive(Clone)]
/// A wrapper for endomorphisms (functions from a type to the same type) that enables monoidal operations.
///
/// `Endomorphism<A>` represents a function `A -> A`.
///
/// It exists to provide a monoid instance where:
///
/// * The binary operation [`append`][Semigroup::append] is [function composition][crate::functions::compose].
/// * The identity element [`empty`][Monoid::empty] is the [identity function][crate::functions::identity].
///
/// This allows combining transformations in a composable, associative way with a clear identity,
/// which is useful for building pipelines of transformations or accumulating operations.
///
/// The wrapped function can be accessed directly via the [`.0` field][Endomorphism#structfield.0].
///
/// # Examples
///
/// ```
/// use fp_library::{
///     brands::RcFnBrand,
///     functions::{append, empty},
///     classes::ClonableFn,
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
pub struct Endomorphism<'a, SmartPointerBrand: SmartPointer, C: Semigroupoid, A: 'a>(
	pub SmartPointerInner<SmartPointerBrand, Apply1L2T<'a, C, A, A>>,
);

impl<'b, SmartPointerBrand: SmartPointer, C: Semigroupoid, A: 'b> Semigroup
	for Endomorphism<'b, SmartPointerBrand, C, A>
{
	/// # Examples
	///
	/// ```
	/// use fp_library::{
	///     brands::RcFnBrand,
	///     functions::append,
	///     types::endomorphism::Endomorphism
	/// };
	///
	/// let double = RcFnBrand::new(|x: i32| x * 2);
	/// let increment = RcFnBrand::new(|x: i32| x + 1);
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
		ClonableFnBrand::new::<'a, _, _>(move |b: Self| {
			let meh = {
				let bla = C::compose::<'a, ClonableFnBrand, _, _, _>(*a.0)(*b.0);
				bla
			};
			Endomorphism::<SmartPointerBrand, C, A>(meh).0
		})
	}
}

impl<'a, SmartPointerBrand: SmartPointer, C: Category, A: 'a> Monoid
	for Endomorphism<'a, SmartPointerBrand, C, A>
{
	/// # Examples
	///
	/// ```
	/// use fp_library::{brands::RcFnBrand, functions::empty, types::Endomorphism};
	///
	/// assert_eq!(empty::<Endomorphism<'static, RcFnBrand, i32>>().0(5), 5);
	/// assert_eq!(empty::<Endomorphism<'static, RcFnBrand, String>>().0("test".to_string()), "test");
	/// ```
	fn empty() -> Self {
		let aaa = C::identity::<'a, A>();
		*Endomorphism::<SmartPointerBrand, C, A>(aaa).0
	}
}
