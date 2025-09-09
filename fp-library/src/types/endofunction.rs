//! Implementations for [`Endofunction`], a wrapper for endofunctions (functions from a set to the same set) that enables monoidal operations.

use core::fmt;
use std::{
	fmt::{Debug, Formatter},
	hash::Hash,
	marker::PhantomData,
};

use crate::{
	classes::{
		Category, ClonableFn, Monoid, Semigroup, clonable_fn::ApplyClonableFn, monoid::Monoid1L0T,
		semigroup::Semigroup1L0T,
	},
	functions::{compose, identity},
	hkt::Kind1L0T,
};

/// A wrapper for endofunctions (functions from a set to the same set) that enables monoidal operations.
///
/// `Endofunction a` represents a function `a -> a`.
///
/// It exists to provide a monoid instance where:
///
/// * The binary operation [append][Semigroup::append] is [function composition][crate::functions::compose].
/// * The identity element [empty][Monoid::empty] is the [identity function][crate::functions::identity].
///
/// The wrapped function can be accessed directly via the [`.0` field][Endofunction#structfield.0].
///
/// # Examples
///
/// ```
/// use fp_library::{
///     brands::{EndofunctionBrand, RcFnBrand},
///     functions::{append, empty},
///     classes::ClonableFn,
///     types::Endofunction,
/// };
/// use std::rc::Rc;
///
/// // Create Endofunctions
/// let f = Endofunction(<RcFnBrand as ClonableFn>::new(|x: i32| x * 2));
/// let g = Endofunction(<RcFnBrand as ClonableFn>::new(|x: i32| x + 1));
///
/// // Compose functions (f after g)
/// let fg = append::<RcFnBrand, EndofunctionBrand<RcFnBrand, i32>>(f)(g);
/// assert_eq!(fg.0(3), 8); // double(increment(3)) = 8
///
/// // Identity element
/// let id = empty::<EndofunctionBrand<RcFnBrand, i32>>();
/// assert_eq!(id.0(42), 42);
/// ```
pub struct Endofunction<'a, ClonableFnBrand: ClonableFn, A: 'a>(
	pub ApplyClonableFn<'a, ClonableFnBrand, A, A>,
);

impl<'a, ClonableFnBrand: ClonableFn, A> Endofunction<'a, ClonableFnBrand, A> {
	pub fn new(a: ApplyClonableFn<'a, ClonableFnBrand, A, A>) -> Self {
		Self(a)
	}
}

impl<'a, ClonableFnBrand: ClonableFn, A> Clone for Endofunction<'a, ClonableFnBrand, A> {
	fn clone(&self) -> Self {
		Self::new(self.0.clone())
	}
}

impl<'a, ClonableFnBrand: ClonableFn, A> Debug for Endofunction<'a, ClonableFnBrand, A>
where
	ApplyClonableFn<'a, ClonableFnBrand, A, A>: Debug,
{
	fn fmt(
		&self,
		fmt: &mut Formatter<'_>,
	) -> fmt::Result {
		fmt.debug_tuple("Endofunction").field(&self.0).finish()
	}
}

impl<'a, ClonableFnBrand: ClonableFn, A> Eq for Endofunction<'a, ClonableFnBrand, A> where
	ApplyClonableFn<'a, ClonableFnBrand, A, A>: Eq
{
}

impl<'a, ClonableFnBrand: ClonableFn, A> Hash for Endofunction<'a, ClonableFnBrand, A>
where
	ApplyClonableFn<'a, ClonableFnBrand, A, A>: Hash,
{
	fn hash<H: std::hash::Hasher>(
		&self,
		state: &mut H,
	) {
		self.0.hash(state);
	}
}

impl<'a, ClonableFnBrand: ClonableFn, A> Ord for Endofunction<'a, ClonableFnBrand, A>
where
	ApplyClonableFn<'a, ClonableFnBrand, A, A>: Ord,
{
	fn cmp(
		&self,
		other: &Self,
	) -> std::cmp::Ordering {
		self.0.cmp(&other.0)
	}
}

impl<'a, ClonableFnBrand: ClonableFn, A> PartialEq for Endofunction<'a, ClonableFnBrand, A>
where
	ApplyClonableFn<'a, ClonableFnBrand, A, A>: PartialEq,
{
	fn eq(
		&self,
		other: &Self,
	) -> bool {
		self.0 == other.0
	}
}

impl<'a, ClonableFnBrand: ClonableFn, A> PartialOrd for Endofunction<'a, ClonableFnBrand, A>
where
	ApplyClonableFn<'a, ClonableFnBrand, A, A>: PartialOrd,
{
	fn partial_cmp(
		&self,
		other: &Self,
	) -> Option<std::cmp::Ordering> {
		self.0.partial_cmp(&other.0)
	}
}

impl<'b, CFB: 'b + ClonableFn, A> Semigroup<'b> for Endofunction<'b, CFB, A> {
	/// # Examples
	///
	/// ```
	/// use fp_library::{
	///     brands::{EndofunctionBrand, RcFnBrand},
	///     functions::append,
	///     classes::ClonableFn,
	///     types::Endofunction,
	/// };
	/// use std::rc::Rc;
	///
	/// let double = <RcFnBrand as ClonableFn>::new(|x: i32| x * 2);
	/// let increment = <RcFnBrand as ClonableFn>::new(|x: i32| x + 1);
	///
	/// assert_eq!(
	///     (append::<RcFnBrand, EndofunctionBrand<RcFnBrand, i32>>(Endofunction(double))(Endofunction(increment.clone()))).0(3),
	///     8
	/// );
	/// assert_eq!(
	///     (append::<RcFnBrand, EndofunctionBrand<RcFnBrand, i32>>(Endofunction(increment.clone()))(Endofunction(increment))).0(3),
	///     5
	/// );
	/// ```
	fn append<'a, ClonableFnBrand: 'a + 'b + ClonableFn>(
		a: Self
	) -> ApplyClonableFn<'a, ClonableFnBrand, Self, Self>
	where
		Self: Sized,
		'b: 'a,
	{
		<ClonableFnBrand as ClonableFn>::new(move |b: Self| {
			Self::new(compose::<'b, CFB, _, _, _>(a.0.clone())(b.0))
		})
	}
}

impl<'a, ClonableFnBrand: 'a + ClonableFn, A> Monoid<'a> for Endofunction<'a, ClonableFnBrand, A> {
	/// # Examples
	///
	/// ```
	/// use fp_library::{
	///     brands::{EndofunctionBrand, RcFnBrand},
	///     functions::empty,
	///     types::Endofunction,
	/// };
	///
	/// assert_eq!(empty::<EndofunctionBrand<RcFnBrand, i32>>().0(5), 5);
	/// assert_eq!(empty::<EndofunctionBrand<RcFnBrand, String>>().0("test".to_string()), "test");
	/// ```
	fn empty() -> Self {
		Self::new(<ClonableFnBrand as ClonableFn>::new(identity))
	}
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct EndofunctionBrand<CategoryBrand: Category, A>(PhantomData<(CategoryBrand, A)>);

impl<ClonableFnBrand: ClonableFn, A: 'static> Kind1L0T for EndofunctionBrand<ClonableFnBrand, A> {
	type Output<'a> = Endofunction<'a, ClonableFnBrand, A>;
}

impl<ClonableFnBrand: 'static + ClonableFn, A: 'static> Semigroup1L0T
	for EndofunctionBrand<ClonableFnBrand, A>
where
	for<'a> ApplyClonableFn<'a, ClonableFnBrand, A, A>: Clone,
{
}

impl<ClonableFnBrand: 'static + ClonableFn, A: 'static> Monoid1L0T
	for EndofunctionBrand<ClonableFnBrand, A>
where
	for<'a> ApplyClonableFn<'a, ClonableFnBrand, A, A>: Clone,
{
}
