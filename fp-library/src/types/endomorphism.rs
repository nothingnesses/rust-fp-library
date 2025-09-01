//! Implementations for [`Endomorphism`], a wrapper for endomorphisms (morphisms from an object to itself) that enables monoidal operations.

use core::fmt;
use std::{
	fmt::{Debug, Formatter},
	hash::Hash,
	marker::PhantomData,
};

use crate::{
	brands::RcFnBrand,
	classes::{
		Category, ClonableFn, Monoid, Semigroup, clonable_fn::ApplyFn, monoid::HktMonoid,
		semigroup::HktSemigroup,
	},
	hkt::{Apply1L2T, Kind1L0T},
};

/// A wrapper for endomorphisms (morphisms from an object to itself) that enables monoidal operations.
///
/// `Endomorphism c a` represents a morphism `c a a` where `c` is a `Category`.
/// For the category of functions, this represents functions of type `a -> a`.
///
/// It exists to provide a monoid instance where:
///
/// * The binary operation [append][Semigroup::append] is [morphism composition][Semigroupoid::compose].
/// * The identity element [empty][Monoid::empty] is the [identity morphism][Category::identity].
///
/// The wrapped morphism can be accessed directly via the [`.0` field][Endomorphism#structfield.0].
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
pub struct Endomorphism<'a, CategoryBrand: Category, A: 'a>(pub Apply1L2T<'a, CategoryBrand, A, A>);

impl<'a, CategoryBrand: Category, A> Endomorphism<'a, CategoryBrand, A> {
	pub fn new(a: Apply1L2T<'a, CategoryBrand, A, A>) -> Self {
		Self(a)
	}
}

impl<'a, CategoryBrand, A> Clone for Endomorphism<'a, CategoryBrand, A>
where
	CategoryBrand: Category + 'a,
	A: 'a,
	Apply1L2T<'a, CategoryBrand, A, A>: Clone,
{
	fn clone(&self) -> Self {
		Endomorphism(self.0.clone())
	}
}

impl<'a, CategoryBrand, A> Debug for Endomorphism<'a, CategoryBrand, A>
where
	CategoryBrand: Category + 'a,
	A: 'a,
	Apply1L2T<'a, CategoryBrand, A, A>: Debug,
{
	fn fmt(
		&self,
		fmt: &mut Formatter<'_>,
	) -> fmt::Result {
		fmt.debug_tuple("Endomorphism").field(&self.0).finish()
	}
}

impl<'a, CategoryBrand, A> Eq for Endomorphism<'a, CategoryBrand, A>
where
	CategoryBrand: Category + 'a,
	A: 'a,
	Apply1L2T<'a, CategoryBrand, A, A>: Eq,
{
}

impl<'a, CategoryBrand, A> Hash for Endomorphism<'a, CategoryBrand, A>
where
	CategoryBrand: Category + 'a,
	A: 'a,
	Apply1L2T<'a, CategoryBrand, A, A>: Hash,
{
	fn hash<H: std::hash::Hasher>(
		&self,
		state: &mut H,
	) {
		self.0.hash(state);
	}
}

impl<'a, CategoryBrand, A> Ord for Endomorphism<'a, CategoryBrand, A>
where
	CategoryBrand: Category + 'a,
	A: 'a,
	Apply1L2T<'a, CategoryBrand, A, A>: Ord,
{
	fn cmp(
		&self,
		other: &Self,
	) -> std::cmp::Ordering {
		self.0.cmp(&other.0)
	}
}

impl<'a, CategoryBrand, A> PartialEq for Endomorphism<'a, CategoryBrand, A>
where
	CategoryBrand: Category + 'a,
	A: 'a,
	Apply1L2T<'a, CategoryBrand, A, A>: PartialEq,
{
	fn eq(
		&self,
		other: &Self,
	) -> bool {
		self.0 == other.0
	}
}

impl<'a, CategoryBrand, A> PartialOrd for Endomorphism<'a, CategoryBrand, A>
where
	CategoryBrand: Category + 'a,
	A: 'a,
	Apply1L2T<'a, CategoryBrand, A, A>: PartialOrd,
{
	fn partial_cmp(
		&self,
		other: &Self,
	) -> Option<std::cmp::Ordering> {
		self.0.partial_cmp(&other.0)
	}
}

impl<'b, CategoryBrand, A> Semigroup<'b> for Endomorphism<'b, CategoryBrand, A>
where
	CategoryBrand: Category + 'b,
	A: 'b,
	Apply1L2T<'b, CategoryBrand, A, A>: Clone,
{
	/// # Examples
	///
	/// ```
	/// use fp_library::{
	///     brands::RcFnBrand,
	///     functions::append,
	///     classes::ClonableFn,
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
	fn append<'a, ClonableFnBrand: 'a + 'b + ClonableFn>(
		a: Self
	) -> ApplyFn<'a, ClonableFnBrand, Self, Self>
	where
		Self: Sized,
		'b: 'a,
	{
		ClonableFnBrand::new(move |b: Self| {
			Endomorphism(CategoryBrand::compose::<'b, RcFnBrand, _, _, _>(a.0.clone())(b.0))
		})
	}
}

impl<'a, CategoryBrand, A> Monoid<'a> for Endomorphism<'a, CategoryBrand, A>
where
	CategoryBrand: Category + 'a,
	A: 'a,
	Apply1L2T<'a, CategoryBrand, A, A>: Clone,
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
		Endomorphism(CategoryBrand::identity::<'a, _>())
	}
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct EndomorphismHkt<CategoryBrand: Category, A>(PhantomData<(CategoryBrand, A)>);

impl<CategoryBrand, A> Kind1L0T for EndomorphismHkt<CategoryBrand, A>
where
	A: 'static,
	CategoryBrand: Category,
{
	type Output<'a> = Endomorphism<'a, CategoryBrand, A>;
}

impl<CategoryBrand, A> HktSemigroup for EndomorphismHkt<CategoryBrand, A>
where
	CategoryBrand: Category + 'static,
	A: 'static,
	for<'a> Apply1L2T<'a, CategoryBrand, A, A>: Clone,
{
}

impl<CategoryBrand, A> HktMonoid for EndomorphismHkt<CategoryBrand, A>
where
	CategoryBrand: Category + 'static,
	A: 'static,
	for<'a> Apply1L2T<'a, CategoryBrand, A, A>: Clone,
{
}
