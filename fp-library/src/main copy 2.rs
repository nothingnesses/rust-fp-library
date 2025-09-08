use core::fmt;
use std::{
	fmt::{Debug, Formatter},
	marker::PhantomData,
	ops::Deref,
	rc::Rc,
};

fn main() {}

#[macro_export]
macro_rules! make_trait_kind {
  (
    $kind_trait_name:ident,
    $lifetimes:tt,
    $types:tt,
    $kind_signature:literal
  ) => {
    make_trait_kind!(
      @impl $kind_trait_name,
      $lifetimes,
      $types,
      $kind_signature
    );
  };
  (@impl $kind_trait_name:ident, (), (), $kind_signature:literal) => {
    pub trait $kind_trait_name { type Output; }
  };
  (@impl $kind_trait_name:ident, ($($lifetimes:lifetime),+), (), $kind_signature:literal) => {
    pub trait $kind_trait_name { type Output<$($lifetimes),*>; }
  };
  (@impl $kind_trait_name:ident, (), ($($types:ident),+), $kind_signature:literal) => {
    pub trait $kind_trait_name { type Output<$($types),*>; }
  };
  (@impl $kind_trait_name:ident, ($($lifetimes:lifetime),+), ($($types:ident),+), $kind_signature:literal) => {
    pub trait $kind_trait_name { type Output<$($lifetimes),*, $($types),*>; }
  };
}

#[macro_export]
macro_rules! make_type_apply {
  (
    $apply_alias_name:ident,
    $kind_trait_name:ident,
    $lifetimes:tt,
    $types:tt,
    $kind_signature:literal
  ) => {
    make_type_apply!(
      @impl $apply_alias_name,
      $kind_trait_name,
      $lifetimes,
      $types,
      $kind_signature
    );
  };
  (@impl $apply_alias_name:ident, $kind_trait_name:ident, (), (), $kind_signature:literal) => {
    pub type $apply_alias_name<Brand> = <Brand as $kind_trait_name>::Output;
  };
  (@impl $apply_alias_name:ident, $kind_trait_name:ident, ($($lifetimes:lifetime),+), (), $kind_signature:literal) => {
    pub type $apply_alias_name<$($lifetimes),*, Brand> = <Brand as $kind_trait_name>::Output<$($lifetimes),*>;
  };
  (@impl $apply_alias_name:ident, $kind_trait_name:ident, (), ($($types:ident),+), $kind_signature:literal) => {
    pub type $apply_alias_name<Brand $(, $types)*> = <Brand as $kind_trait_name>::Output<$($types),*>;
  };
  (@impl $apply_alias_name:ident, $kind_trait_name:ident, ($($lifetimes:lifetime),+), ($($types:ident),+), $kind_signature:literal) => {
    pub type $apply_alias_name<$($lifetimes),*, Brand $(, $types)*> = <Brand as $kind_trait_name>::Output<$($lifetimes),* $(, $types)*>;
  };
}

make_trait_kind!(Kind0L1T, (), (A), "* -> *");
make_trait_kind!(Kind1L0T, ('a), (), "' -> *");
make_trait_kind!(Kind1L2T, ('a), (A, B), "' -> * -> * -> *");
make_type_apply!(Apply0L1T, Kind0L1T, (), (A), "* -> *");
make_type_apply!(Apply1L0T, Kind1L0T, ('a), (), "' -> *");
make_type_apply!(Apply1L2T, Kind1L2T, ('a), (A, B), "' -> * -> * -> *");

/// Abstraction for clonable wrappers over closures.
pub trait ClonableFn: Kind1L2T + Clone {
	fn new<'a, A: 'a, B: 'a>(f: impl 'a + Fn(A) -> B) -> Apply1L2T<'a, Self, A, B>
	where
		Self::Output<'a, A, B>: 'a;
}
pub type ApplyFn<'a, Brand, A, B> = Apply1L2T<'a, Brand, A, B>;
pub fn compose<'a, ClonableFnBrand: 'a + ClonableFn, A: 'a, B: 'a, C: 'a>(
	f: Apply1L2T<'a, ClonableFnBrand, B, C>
) -> Apply1L2T<
	'a,
	ClonableFnBrand,
	Apply1L2T<'a, ClonableFnBrand, A, B>,
	Apply1L2T<'a, ClonableFnBrand, A, C>,
>
where
	Apply1L2T<'a, ClonableFnBrand, B, C>: Clone + Deref<Target = dyn Fn(B) -> C>,
	Apply1L2T<'a, ClonableFnBrand, A, B>: Deref<Target = dyn Fn(A) -> B>,
{
	ClonableFnBrand::new(move |g: Apply1L2T<'a, ClonableFnBrand, A, B>| {
		let f = f.clone();
		ClonableFnBrand::new(move |a| f(g(a)))
	})
}
pub fn identity<A>(a: A) -> A {
	a
}

pub trait Semigroup<'b> {
	fn append<'a, ClonableFnBrand: 'a + 'b + ClonableFn>(
		a: Self
	) -> Apply1L2T<'a, ClonableFnBrand, Self, Self>
	where
		Self: Clone + Sized,
		'b: 'a,
		Apply1L2T<'a, ClonableFnBrand, Self, Self>: Deref<Target = dyn 'a + Fn(Self) -> Self>;
}
pub trait HktSemigroup: Kind1L0T
where
	for<'a> Apply1L0T<'a, Self>: Semigroup<'a>,
{
}
pub fn append<'a, ClonableFnBrand: 'a + ClonableFn, HktBrand: HktSemigroup>(
	a: Apply1L0T<'a, HktBrand>
) -> ApplyFn<'a, ClonableFnBrand, Apply1L0T<'a, HktBrand>, Apply1L0T<'a, HktBrand>>
where
	for<'b> Apply1L0T<'b, HktBrand>: Semigroup<'b> + Clone,
	ApplyFn<'a, ClonableFnBrand, Apply1L0T<'a, HktBrand>, Apply1L0T<'a, HktBrand>>:
		Deref<Target = dyn 'a + Fn(Apply1L0T<'a, HktBrand>) -> Apply1L0T<'a, HktBrand>>,
{
	<Apply1L0T<'a, HktBrand> as Semigroup<'a>>::append::<ClonableFnBrand>(a)
}
pub trait Monoid<'a>: Semigroup<'a> {
	fn empty() -> Self;
}
pub trait HktMonoid: HktSemigroup
where
	for<'a> Apply1L0T<'a, Self>: Monoid<'a>,
{
}
pub fn empty<'a, HktBrand>() -> Apply1L0T<'a, HktBrand>
where
	HktBrand: HktMonoid,
	for<'b> Apply1L0T<'b, HktBrand>: Monoid<'b>,
{
	<Apply1L0T<'a, HktBrand> as Monoid<'a>>::empty()
}
pub trait Semigroupoid: Kind1L2T {
	fn compose<'a, ClonableFnBrand: 'a + ClonableFn, B: 'a, C: 'a, D: 'a>(
		f: Apply1L2T<'a, Self, C, D>
	) -> ApplyFn<'a, ClonableFnBrand, Apply1L2T<'a, Self, B, C>, Apply1L2T<'a, Self, B, D>>;
}
pub fn semigroupoid_compose<
	'a,
	ClonableFnBrand: 'a + ClonableFn,
	Brand: Semigroupoid,
	B: 'a,
	C: 'a,
	D: 'a,
>(
	f: Apply1L2T<'a, Brand, C, D>
) -> ApplyFn<'a, ClonableFnBrand, Apply1L2T<'a, Brand, B, C>, Apply1L2T<'a, Brand, B, D>>
where
	ApplyFn<'a, ClonableFnBrand, Apply1L2T<'a, Brand, B, C>, Apply1L2T<'a, Brand, B, D>>:
		Deref<Target = dyn 'a + Fn(Apply1L2T<'a, Brand, B, C>) -> Apply1L2T<'a, Brand, B, D>>,
{
	Brand::compose::<'a, ClonableFnBrand, B, C, D>(f)
}
pub trait Category: Semigroupoid {
	fn identity<'a, A: 'a>() -> Apply1L2T<'a, Self, A, A>;
}
pub fn category_identity<'a, Brand: Category, A: 'a>() -> Apply1L2T<'a, Brand, A, A> {
	Brand::identity::<'a, _>()
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RcFnBrand;
impl Kind1L2T for RcFnBrand {
	type Output<'a, A, B> = Rc<dyn 'a + Fn(A) -> B>;
}
impl ClonableFn for RcFnBrand {
	fn new<'a, A: 'a, B: 'a>(f: impl 'a + Fn(A) -> B) -> Apply1L2T<'a, Self, A, B> {
		Rc::new(f)
	}
}
impl Semigroupoid for RcFnBrand {
	fn compose<'a, ClonableFnBrand: 'a + ClonableFn, B: 'a, C: 'a, D: 'a>(
		f: Apply1L2T<'a, Self, C, D>
	) -> ApplyFn<'a, ClonableFnBrand, Apply1L2T<'a, Self, B, C>, Apply1L2T<'a, Self, B, D>>
	where
		Apply1L2T<'a, Self, C, D>: Clone + Deref<Target = dyn 'a + Fn(C) -> D>,
		Apply1L2T<'a, Self, B, C>: Deref<Target = dyn 'a + Fn(B) -> C>,
	{
		ClonableFnBrand::new(move |g: Apply1L2T<'a, Self, B, C>| {
			let f = f.clone();
			Self::new(move |a| f(g(a)))
		})
	}
}
impl Category for RcFnBrand {
	fn identity<'a, T: 'a>() -> Apply1L2T<'a, Self, T, T> {
		Self::new::<'a, _, _>(identity)
	}
}

pub struct Endomorphism<'a, CategoryBrand: Category, A: 'a>(pub Apply1L2T<'a, CategoryBrand, A, A>);
impl<'a, CategoryBrand: Category, A: 'a> Clone for Endomorphism<'a, CategoryBrand, A>
where
	Apply1L2T<'a, CategoryBrand, A, A>: Clone,
{
	fn clone(&self) -> Self {
		Endomorphism(self.0.clone())
	}
}
impl<'a, CategoryBrand: Category, A: 'a> Debug for Endomorphism<'a, CategoryBrand, A>
where
	Apply1L2T<'a, CategoryBrand, A, A>: Debug,
{
	fn fmt(
		&self,
		f: &mut Formatter<'_>,
	) -> fmt::Result {
		f.debug_tuple("Endomorphism").field(&self.0).finish()
	}
}

// Added dummy Semigroup impl to satisfy Monoid's trait bound.
// A correct implementation is not possible with the current trait designs.
impl<'a, CategoryBrand: Category, A: 'a> Semigroup<'a> for Endomorphism<'a, CategoryBrand, A> {
	fn append<'b, ClonableFnBrand: 'b + 'a + ClonableFn>(
		_a: Self
	) -> ApplyFn<'b, ClonableFnBrand, Self, Self>
	where
		Self: Sized + Clone,
		'a: 'b,
		ApplyFn<'b, ClonableFnBrand, Self, Self>: Deref<Target = dyn 'b + Fn(Self) -> Self>,
	{
		unimplemented!(
			"This impl is impossible to satisfy correctly with the current Semigroup trait design"
		);
	}
}

impl<'a, CategoryBrand: Category, A: 'a> Monoid<'a> for Endomorphism<'a, CategoryBrand, A>
where
	Apply1L2T<'a, CategoryBrand, A, A>: Clone,
{
	fn empty() -> Self {
		Endomorphism(CategoryBrand::identity::<'a, _>())
	}
}

pub struct EndomorphismHkt<CategoryBrand: Category, A>(PhantomData<(CategoryBrand, A)>);
impl<CategoryBrand: Category, A: 'static> Kind1L0T for EndomorphismHkt<CategoryBrand, A> {
	type Output<'a> = Endomorphism<'a, CategoryBrand, A>;
}
impl<CategoryBrand: Category + 'static, A: 'static> HktSemigroup
	for EndomorphismHkt<CategoryBrand, A>
where
	for<'a> Apply1L2T<'a, CategoryBrand, A, A>: Clone,
{
}
impl<CategoryBrand: Category + 'static, A: 'static> HktMonoid for EndomorphismHkt<CategoryBrand, A> where
	for<'a> Apply1L2T<'a, CategoryBrand, A, A>: Clone
{
}

/// A type class for structures that can be folded to a single value.
pub trait Foldable: Kind0L1T {
	/// Maps values to a monoid and combines them.
	fn fold_map<'a, ClonableFnBrand: 'a + ClonableFn, A, M>(
		f: Apply1L2T<'a, ClonableFnBrand, A, Apply1L0T<'a, M>>
	) -> Apply1L2T<'a, ClonableFnBrand, Apply0L1T<Self, A>, Apply1L0T<'a, M>>
	where
		M: HktMonoid,
		for<'b> Apply1L0T<'b, M>: 'a + Monoid<'b> + Clone,
		Apply1L0T<'a, M>: 'static,
		ClonableFnBrand: Category + 'static,
		A: 'static,
		Apply0L1T<Self, A>: 'a,
		Apply1L2T<'a, ClonableFnBrand, A, Apply1L0T<'a, M>>:
			Deref<Target = dyn 'a + Fn(A) -> Apply1L0T<'a, M>>,
		Apply1L2T<'a, ClonableFnBrand, Apply1L0T<'a, M>, Apply1L0T<'a, M>>:
			Deref<Target = dyn 'a + Fn(Apply1L0T<'a, M>) -> Apply1L0T<'a, M>>,
		Apply1L2T<
			'a,
			ClonableFnBrand,
			Apply1L0T<'a, M>,
			Apply1L2T<'a, ClonableFnBrand, Apply0L1T<Self, A>, Apply1L0T<'a, M>>,
		>: Deref<
			Target = dyn 'a
			             + Fn(
				Apply1L0T<'a, M>,
			)
				-> Apply1L2T<'a, ClonableFnBrand, Apply0L1T<Self, A>, Apply1L0T<'a, M>>,
		>,
		Apply1L2T<'a, ClonableFnBrand, Apply1L0T<'a, M>, Apply1L0T<'a, M>>: Clone,
		Endomorphism<'a, ClonableFnBrand, Apply1L0T<'a, M>>: Monoid<'a>,
		Apply1L2T<
			'a,
			ClonableFnBrand,
			A,
			Apply1L2T<'a, ClonableFnBrand, Apply1L0T<'a, M>, Apply1L0T<'a, M>>,
		>: Clone
			+ Deref<
				Target = dyn 'a
				             + Fn(
					A,
				) -> Apply1L2T<
					'a,
					ClonableFnBrand,
					Apply1L0T<'a, M>,
					Apply1L0T<'a, M>,
				>,
			>,
		Apply1L2T<'a, ClonableFnBrand, A, Endomorphism<'a, ClonableFnBrand, Apply1L0T<'a, M>>>:
			Clone
				+ Deref<
					Target = dyn 'a + Fn(A) -> Endomorphism<'a, ClonableFnBrand, Apply1L0T<'a, M>>,
				>,
		Apply1L2T<
			'a,
			ClonableFnBrand,
			Apply0L1T<Self, A>,
			Endomorphism<'a, ClonableFnBrand, Apply1L0T<'a, M>>,
		>: Deref<
			Target = dyn 'a
			             + Fn(
				Apply0L1T<Self, A>,
			) -> Endomorphism<'a, ClonableFnBrand, Apply1L0T<'a, M>>,
		>,
	{
		let curried_folder = ClonableFnBrand::new(move |a: A| {
			let f_a = (&f)(a);
			ClonableFnBrand::new(move |b: Apply1L0T<'a, M>| {
				let appended = append::<'a, ClonableFnBrand, M>(f_a.clone());
				(&appended)(b)
			})
		});
		let fold = Self::fold_right::<'a, ClonableFnBrand, A, Apply1L0T<'a, M>>(curried_folder);
		let applied_empty = (&fold)(empty::<'a, M>());
		applied_empty
	}

	/// Folds the structure by applying a function from right to left.
	fn fold_right<'a, ClonableFnBrand, A, B>(
		f: Apply1L2T<'a, ClonableFnBrand, A, Apply1L2T<'a, ClonableFnBrand, B, B>>
	) -> Apply1L2T<'a, ClonableFnBrand, B, Apply1L2T<'a, ClonableFnBrand, Apply0L1T<Self, A>, B>>
	where
		ClonableFnBrand: Category + ClonableFn + 'a + 'static,
		A: 'static,
		B: 'a + Clone + 'static,
		Apply0L1T<Self, A>: 'a,
		Apply1L2T<'a, ClonableFnBrand, B, B>: Clone,
		Endomorphism<'a, ClonableFnBrand, B>: Monoid<'a>,
		Apply1L2T<'a, ClonableFnBrand, A, Apply1L2T<'a, ClonableFnBrand, B, B>>:
			Clone + Deref<Target = dyn 'a + Fn(A) -> Apply1L2T<'a, ClonableFnBrand, B, B>>,
		Apply1L2T<'a, ClonableFnBrand, A, Endomorphism<'a, ClonableFnBrand, B>>:
			Clone + Deref<Target = dyn 'a + Fn(A) -> Endomorphism<'a, ClonableFnBrand, B>>,
		Apply1L2T<'a, ClonableFnBrand, Apply0L1T<Self, A>, Endomorphism<'a, ClonableFnBrand, B>>:
			Deref<Target = dyn 'a + Fn(Apply0L1T<Self, A>) -> Endomorphism<'a, ClonableFnBrand, B>>,
		Apply1L2T<'a, ClonableFnBrand, B, B>: Deref<Target = dyn 'a + Fn(B) -> B>,
		Apply1L2T<'a, ClonableFnBrand, B, Apply1L2T<'a, ClonableFnBrand, Apply0L1T<Self, A>, B>>:
			Deref<Target = dyn 'a + Fn(B) -> Apply1L2T<'a, ClonableFnBrand, Apply0L1T<Self, A>, B>>,
		Apply1L2T<
			'a,
			ClonableFnBrand,
			Endomorphism<'a, ClonableFnBrand, B>,
			Endomorphism<'a, ClonableFnBrand, B>,
		>: Clone
			+ Deref<
				Target = dyn 'a
				             + Fn(
					Endomorphism<'a, ClonableFnBrand, B>,
				) -> Endomorphism<'a, ClonableFnBrand, B>,
			>,
		Apply1L2T<
			'a,
			ClonableFnBrand,
			A,
			Apply1L2T<
				'a,
				ClonableFnBrand,
				Endomorphism<'a, ClonableFnBrand, B>,
				Endomorphism<'a, ClonableFnBrand, B>,
			>,
		>: Clone
			+ Deref<
				Target = dyn 'a
				             + Fn(
					A,
				) -> Apply1L2T<
					'a,
					ClonableFnBrand,
					Endomorphism<'a, ClonableFnBrand, B>,
					Endomorphism<'a, ClonableFnBrand, B>,
				>,
			>,
		Apply1L2T<
			'a,
			ClonableFnBrand,
			Apply0L1T<Self, A>,
			Endomorphism<'a, ClonableFnBrand, Endomorphism<'a, ClonableFnBrand, B>>,
		>: Deref<
			Target = dyn 'a
			             + Fn(
				Apply0L1T<Self, A>,
			) -> Endomorphism<
				'a,
				ClonableFnBrand,
				Endomorphism<'a, ClonableFnBrand, B>,
			>,
		>,
	{
		ClonableFnBrand::new(move |b: B| {
			ClonableFnBrand::new({
				let f = f.clone();
				move |fa: Apply0L1T<Self, A>| {
					let f_clone = f.clone();
					let f_endo = ClonableFnBrand::new(move |a: A| Endomorphism((&f_clone)(a)));
					let fold_map_f = Self::fold_map::<
						'a,
						ClonableFnBrand,
						A,
						EndomorphismHkt<ClonableFnBrand, B>,
					>(f_endo);
					let composed_endo = (&fold_map_f)(fa);
					(&(composed_endo.0))(b.clone())
				}
			})
		})
	}
}
