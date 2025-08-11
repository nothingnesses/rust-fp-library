use std::{marker::PhantomData, sync::Arc};

fn main() {}

#[macro_export]
macro_rules! make_trait_kind {
	(
		$KindN:ident,
		$ApplyN:ident,
		$kind_string:literal,
		()
	) => {
		#[doc = concat!(
			"Trait for [brands][crate::brands] of [types][crate::types] of kind `",
			$kind_string,
			"`."
		)]
		pub trait $KindN {
			type Output;
		}

		impl<Brand> Kind<()> for Brand
		where
			Brand: $KindN,
		{
			type Output = $ApplyN<Brand>;
		}
	};
	(
		$KindN:ident,
		$ApplyN:ident,
		$kind_string:literal,
		($($Generics:ident),+)
	) => {
		#[doc = concat!(
			"Trait for [brands][crate::brands] of [types][crate::types] of kind `",
			$kind_string,
			"`."
		)]
		pub trait $KindN<$($Generics),+> {
			type Output;
		}

		impl<Brand, $($Generics),+> Kind<($($Generics,)+)> for Brand
		where
			Brand: $KindN<$($Generics),+>,
		{
			type Output = $ApplyN<Brand, $($Generics),+>;
		}
	};
}

#[macro_export]
macro_rules! make_type_apply {
	(
		$KindN:ident,
		$ApplyN:ident,
		$kind_string:literal,
		()
	) => {
		#[doc = concat!(
			"Alias for [types][crate::types] of kind `",
			$kind_string,
			"`."
		)]
		pub type $ApplyN<Brand> = <Brand as $KindN>::Output;
	};
	(
		$KindN:ident,
		$ApplyN:ident,
		$kind_string:literal,
		($($Generics:ident),+)
	) => {
		#[doc = concat!(
			"Alias for [types][crate::types] of kind `",
			$kind_string,
			"`."
		)]
		pub type $ApplyN<Brand, $($Generics),+> = <Brand as $KindN<$($Generics),+>>::Output;
	};
}

#[macro_export]
macro_rules! make_trait_brand {
	(
		$BrandN:ident,
		$kind_string:literal,
		()
	) => {
		#[doc = concat!(
			"[`BrandN` trait][crate::hkt::brands] for [types][crate::types] with kind `",
			$kind_string,
			"`."
		)]
		pub trait $BrandN<Concrete>
		where
			Self: Kind<()>,
		{
			fn inject(a: Concrete) -> Apply<Self, ()>;
			fn project(a: Apply<Self, ()>) -> Concrete;
		}

		impl<Me, Concrete> Brand<Concrete, ()> for Me
		where
			Me: Kind<()> + $BrandN<Concrete>,
		{
			fn inject(a: Concrete) -> Apply<Self, ()> {
				<Me as $BrandN<Concrete>>::inject(a)
			}

			fn project(a: Apply<Self, ()>) -> Concrete {
				<Me as $BrandN<Concrete>>::project(a)
			}
		}
	};
	(
		$BrandN:ident,
		$kind_string:literal,
		($($Generics:ident),+)
	) => {
		#[doc = concat!(
			"[`BrandN` trait][crate::hkt::brands] for [types][crate::types] with kind `",
			$kind_string,
			"`."
		)]
		pub trait $BrandN<Concrete, $($Generics),+>
		where
			Self: Kind<($($Generics,)+)>,
		{
			fn inject(a: Concrete) -> Apply<Self, ($($Generics,)+)>;
			fn project(a: Apply<Self, ($($Generics,)+)>) -> Concrete;
		}

		impl<Me, Concrete, $($Generics),+> Brand<Concrete, ($($Generics,)+)> for Me
		where
			Me: Kind<($($Generics,)+)> + $BrandN<Concrete, $($Generics),+>,
		{
			fn inject(a: Concrete) -> Apply<Self, ($($Generics,)+)> {
				<Me as $BrandN<Concrete, $($Generics),+>>::inject(a)
			}

			fn project(a: Apply<Self, ($($Generics,)+)>) -> Concrete {
				<Me as $BrandN<Concrete, $($Generics),+>>::project(a)
			}
		}
	};
}

#[macro_export]
macro_rules! impl_brand {
	(
		$Brand:ident,
		$Concrete:ident,
		$KindN:ident,
		$BrandN:ident,
		()
	) => {
		#[doc = concat!(
			"[Brand][crate::brands] for [`",
			stringify!($Concrete),
			"`]."
		)]
		pub struct $Brand;

		impl $KindN for $Brand {
			type Output = $Concrete;
		}

		impl $BrandN<$Concrete> for $Brand {
			fn inject(a: $Concrete) -> Apply<Self, ()> {
				a
			}

			fn project(a: Apply<Self, ()>) -> $Concrete {
				a
			}
		}
	};
	(
		$Brand:ident,
		$Concrete:ident,
		$KindN:ident,
		$BrandN:ident,
		($($Generics:ident),+)
	) => {
		#[doc = concat!(
			"[Brand][crate::brands] for [`",
			stringify!($Concrete),
			"`]."
		)]
		pub struct $Brand;

		impl<$($Generics),+> $KindN<$($Generics),+> for $Brand {
			type Output = $Concrete<$($Generics),+>;
		}

		impl<$($Generics),+> $BrandN<$Concrete<$($Generics),+>, $($Generics,)+> for $Brand {
			fn inject(a: $Concrete<$($Generics),+>) -> Apply<Self, ($($Generics,)+)> {
				a
			}

			fn project(a: Apply<Self, ($($Generics,)+)>) -> $Concrete<$($Generics),+> {
				a
			}
		}
	}
}

make_trait_kind!(Kind0, Apply0, "*", ());
make_trait_kind!(Kind1, Apply1, "* -> *", (A));
make_trait_kind!(Kind2, Apply2, "* -> * -> *", (A, B));
pub trait Kind<Parameters> {
	type Output;
}
make_type_apply!(Kind0, Apply0, "*", ());
make_type_apply!(Kind1, Apply1, "* -> *", (A));
make_type_apply!(Kind2, Apply2, "* -> * -> *", (A, B));
pub type Apply<Brand, Parameters> = <Brand as Kind<Parameters>>::Output;
pub trait Brand<Concrete, Parameters>: Kind<Parameters> {
	fn inject(a: Concrete) -> Self::Output;
	fn project(a: Self::Output) -> Concrete;
}
make_trait_brand!(Brand0, "*", ());
make_trait_brand!(Brand1, "* -> *", (A));
make_trait_brand!(Brand2, "* -> * -> *", (A, B));

pub type ClonableFn<'a, A, B> = Arc<dyn 'a + Fn(A) -> B>;

pub fn compose<'a, A: 'a, B: 'a, C: 'a>(
	f: ClonableFn<'a, B, C>
) -> ClonableFn<'a, ClonableFn<'a, A, B>, ClonableFn<'a, A, C>> {
	Arc::new(move |g| {
		let f = f.clone();
		Arc::new(move |a| f(g(a)))
	})
}

pub fn flip<'a, A: 'a, B: 'a + Clone, C: 'a>(
	f: ClonableFn<'a, A, ClonableFn<'a, B, C>>
) -> ClonableFn<'a, B, ClonableFn<'a, A, C>> {
	Arc::new(move |b| {
		let f = f.clone();
		Arc::new(move |a| (f(a))(b.to_owned()))
	})
}

pub fn identity<A>(a: A) -> A {
	a
}

pub trait Semigroup<'a>: Kind<()> {
	fn append(a: Apply<Self, ()>) -> ClonableFn<'a, Apply<Self, ()>, Apply<Self, ()>>;
}

pub fn append<'a, Brand>(a: Apply<Brand, ()>) -> ClonableFn<'a, Apply<Brand, ()>, Apply<Brand, ()>>
where
	Brand: Kind<()> + Semigroup<'a>,
{
	Brand::append(a)
}

pub trait Monoid<'a>: Semigroup<'a> {
	fn empty() -> Apply<Self, ()>;
}

pub fn empty<Brand>() -> Apply<Brand, ()>
where
	for<'a> Brand: Monoid<'a>,
{
	Brand::empty()
}

pub trait Functor {
	fn map<'a, A, B>(f: ClonableFn<'a, A, B>) -> impl Fn(Apply<Self, (A,)>) -> Apply<Self, (B,)>
	where
		Self: Kind<(A,)> + Kind<(B,)>;
}

pub fn map<'a, Brand, A, B>(
	f: ClonableFn<'a, A, B>
) -> impl Fn(Apply<Brand, (A,)>) -> Apply<Brand, (B,)>
where
	Brand: Kind<(A,)> + Kind<(B,)> + Functor + ?Sized,
{
	Brand::map(f)
}

pub trait Pure {
	fn pure<A>(a: A) -> Apply<Self, (A,)>
	where
		Self: Kind<(A,)>;
}

pub fn pure<Brand, A>(a: A) -> Apply<Brand, (A,)>
where
	Brand: Kind<(A,)> + Pure,
{
	Brand::pure(a)
}

pub trait TypeclassApply {
	fn apply<'a, F, A, B>(ff: Apply<Self, (F,)>) -> impl Fn(Apply<Self, (A,)>) -> Apply<Self, (B,)>
	where
		Self: Kind<(F,)> + Kind<(A,)> + Kind<(B,)>,
		Apply<Self, (F,)>: Clone,
		F: 'a + Fn(A) -> B,
		A: Clone;
}

pub fn apply<'a, Brand, F, A, B>(
	ff: Apply<Brand, (F,)>
) -> impl Fn(Apply<Brand, (A,)>) -> Apply<Brand, (B,)>
where
	Brand: Kind<(F,)> + Kind<(A,)> + Kind<(B,)> + TypeclassApply,
	Apply<Brand, (F,)>: Clone,
	F: 'a + Fn(A) -> B,
	A: Clone,
{
	Brand::apply::<F, _, _>(ff)
}

pub trait ApplyFirst {
	fn apply_first<A, B>(fa: Apply<Self, (A,)>) -> impl Fn(Apply<Self, (B,)>) -> Apply<Self, (A,)>
	where
		Self: Kind<(A,)> + Kind<(B,)>,
		Apply<Self, (A,)>: Clone,
		A: Clone,
		B: Clone;
}

pub fn apply_first<Brand, A, B>(
	fa: Apply<Brand, (A,)>
) -> impl Fn(Apply<Brand, (B,)>) -> Apply<Brand, (A,)>
where
	Brand: Kind<(A,)> + Kind<(B,)> + ApplyFirst,
	Apply<Brand, (A,)>: Clone,
	A: Clone,
	B: Clone,
{
	Brand::apply_first::<A, B>(fa)
}

pub trait ApplySecond {
	fn apply_second<A, B>(fa: Apply<Self, (A,)>) -> impl Fn(Apply<Self, (B,)>) -> Apply<Self, (B,)>
	where
		Self: Kind<(A,)> + Kind<(B,)>,
		Apply<Self, (A,)>: Clone,
		B: Clone;
}

pub fn apply_second<Brand, A, B>(
	fa: Apply<Brand, (A,)>
) -> impl Fn(Apply<Brand, (B,)>) -> Apply<Brand, (B,)>
where
	Brand: Kind<(A,)> + Kind<(B,)> + ApplySecond,
	Apply<Brand, (A,)>: Clone,
	B: Clone,
{
	Brand::apply_second::<A, B>(fa)
}

pub trait Applicative: Functor + Pure + TypeclassApply + ApplyFirst + ApplySecond {}

impl<Brand> Applicative for Brand where
	Brand: Functor + Pure + TypeclassApply + ApplyFirst + ApplySecond
{
}

pub trait Foldable {
	fn fold_left<'a, A, B>(
		f: ClonableFn<'a, B, ClonableFn<'a, A, B>>
	) -> ClonableFn<'a, B, ClonableFn<'a, Apply<Self, (A,)>, B>>
	where
		Self: 'a + Kind<(A,)>,
		A: 'a + Clone,
		B: 'a + Clone,
		Apply<Self, (A,)>: 'a,
	{
		Arc::new(move |b| {
			Arc::new({
				let f = f.clone();
				move |fa| {
					(((Self::fold_right(compose(flip(Arc::new(compose)))(flip(f.clone()))))(
						Arc::new(identity),
					))(fa))(b.clone())
				}
			})
		})
	}

	fn fold_map<'a, A, M>(
		f: ClonableFn<'a, A, Apply<M, ()>>
	) -> ClonableFn<'a, Apply<Self, (A,)>, Apply<M, ()>>
	where
		Self: Kind<(A,)>,
		A: 'a + Clone,
		M: Monoid<'a>,
		Apply<M, ()>: 'a + Clone,
	{
		Arc::new(move |fa| {
			((Self::fold_right(Arc::new(|a| (compose(Arc::new(M::append))(f.clone()))(a))))(
				M::empty(),
			))(fa)
		})
	}

	fn fold_right<'a, A, B>(
		f: ClonableFn<'a, A, ClonableFn<'a, B, B>>
	) -> ClonableFn<'a, B, ClonableFn<'a, Apply<Self, (A,)>, B>>
	where
		Self: 'a + Kind<(A,)>,
		A: 'a + Clone,
		B: 'a + Clone,
		Apply<Self, (A,)>: 'a,
	{
		Arc::new(move |b| {
			let f = f.clone();
			Arc::new(move |fa| {
				((Self::fold_map::<A, EndomorphismBrand<B>>(Arc::new({
					let f = f.clone();
					move |a| Endomorphism(f(a))
				}))(fa))
				.0)(b.to_owned())
			})
		})
	}
}

pub fn fold_left<'a, Brand, A, B>(
	f: ClonableFn<'a, B, ClonableFn<'a, A, B>>
) -> ClonableFn<'a, B, ClonableFn<'a, Apply<Brand, (A,)>, B>>
where
	Brand: 'a + Kind<(A,)> + Foldable,
	A: 'a + Clone,
	B: 'a + Clone,
	Apply<Brand, (A,)>: 'a,
{
	Brand::fold_left(f)
}

pub fn fold_map<'a, Brand, A, M>(
	f: ClonableFn<'a, A, Apply<M, ()>>
) -> ClonableFn<'a, Apply<Brand, (A,)>, Apply<M, ()>>
where
	Brand: Kind<(A,)> + Foldable,
	A: 'a + Clone,
	M: Monoid<'a>,
	Apply<M, ()>: 'a + Clone,
{
	Brand::fold_map::<_, M>(f)
}

pub fn fold_right<'a, Brand, A, B>(
	f: ClonableFn<'a, A, ClonableFn<'a, B, B>>
) -> ClonableFn<'a, B, ClonableFn<'a, Apply<Brand, (A,)>, B>>
where
	Brand: 'a + Kind<(A,)> + Foldable,
	A: 'a + Clone,
	B: 'a + Clone,
	Apply<Brand, (A,)>: 'a,
{
	Brand::fold_right(f)
}

pub trait Traversable: Functor + Foldable {
	fn traverse<'a, F, A, B>(
		f: ClonableFn<'a, A, Apply<F, (B,)>>
	) -> ClonableFn<'a, Apply<Self, (A,)>, Apply<F, (Apply<Self, (B,)>,)>>
	where
		Self: Kind<(A,)> + Kind<(B,)> + Kind<(Apply<F, (B,)>,)>,
		F: 'a + Kind<(B,)> + Kind<(Apply<Self, (B,)>,)> + Applicative,
		A: 'a + Clone,
		B: Clone,
		Apply<F, (B,)>: 'a + Clone,
		Apply<Self, (B,)>: Clone,
	{
		Arc::new(move |ta| Self::sequence::<F, B>(map::<Self, _, Apply<F, (B,)>>(f.clone())(ta)))
	}

	fn sequence<F, A>(t: Apply<Self, (Apply<F, (A,)>,)>) -> Apply<F, (Apply<Self, (A,)>,)>
	where
		Self: Kind<(Apply<F, (A,)>,)> + Kind<(A,)>,
		F: Kind<(A,)> + Kind<(Apply<Self, (A,)>,)> + Applicative,
		A: Clone,
		Apply<F, (A,)>: Clone,
		Apply<Self, (A,)>: Clone,
	{
		(Self::traverse::<F, _, A>(Arc::new(identity)))(t)
	}
}

pub fn traverse<'a, Brand, F, A, B>(
	f: ClonableFn<'a, A, Apply<F, (B,)>>
) -> ClonableFn<'a, Apply<Brand, (A,)>, Apply<F, (Apply<Brand, (B,)>,)>>
where
	Brand: Kind<(A,)> + Kind<(B,)> + Kind<(Apply<F, (B,)>,)> + Traversable,
	F: 'a + Kind<(B,)> + Kind<(Apply<Brand, (B,)>,)> + Applicative,
	A: 'a + Clone,
	B: Clone,
	Apply<F, (B,)>: 'a + Clone,
	Apply<Brand, (B,)>: Clone,
{
	Brand::traverse::<F, _, B>(f)
}

pub fn sequence<Brand, F, A>(t: Apply<Brand, (Apply<F, (A,)>,)>) -> Apply<F, (Apply<Brand, (A,)>,)>
where
	Brand: Kind<(Apply<F, (A,)>,)> + Kind<(A,)> + Traversable,
	F: Kind<(A,)> + Kind<(Apply<Brand, (A,)>,)> + Applicative,
	A: Clone,
	Apply<F, (A,)>: Clone,
	Apply<Brand, (A,)>: Clone,
{
	Brand::sequence::<F, A>(t)
}

#[derive(Clone)]
pub struct Endomorphism<'a, A>(pub Arc<dyn 'a + Fn(A) -> A>);

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct EndomorphismBrand<'a, A>(A, PhantomData<&'a A>);

impl<'a, A> Kind0 for EndomorphismBrand<'a, A> {
	type Output = Endomorphism<'a, A>;
}

impl<'a, A> Brand0<Endomorphism<'a, A>> for EndomorphismBrand<'a, A> {
	fn inject(a: Endomorphism<'a, A>) -> Apply<Self, ()> {
		a
	}
	fn project(a: Apply<Self, ()>) -> Endomorphism<'a, A> {
		a
	}
}

impl<'a, A> Semigroup<'a> for EndomorphismBrand<'a, A> {
	fn append(a: Apply<Self, ()>) -> ClonableFn<'a, Apply<Self, ()>, Apply<Self, ()>> {
		let a = <Self as Brand<_, _>>::project(a).0;
		Arc::new(move |b| Endomorphism(compose(a.clone())(<Self as Brand<_, _>>::project(b).0)))
	}
}

impl<'a, A> Monoid<'a> for EndomorphismBrand<'a, A> {
	fn empty() -> Apply<Self, ()> {
		Endomorphism(Arc::new(identity))
	}
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Pair<First, Second>(pub First, pub Second);

impl_brand!(PairBrand, Pair, Kind2, Brand2, (A, B));

impl<First, Second> Pair<First, Second>
where
	First: Clone,
{
	pub fn new(first: First) -> impl Fn(Second) -> Self {
		move |second| Pair(first.to_owned(), second)
	}
}

impl_brand!(VecBrand, Vec, Kind1, Brand1, (A));

impl<'a> VecBrand {
	pub fn construct<A>(head: A) -> ClonableFn<'a, Apply<Self, (A,)>, Apply<Self, (A,)>>
	where
		A: 'a + Clone,
	{
		Arc::new(move |tail| [vec![head.to_owned()], tail].concat())
	}

	pub fn deconstruct<A>(slice: &[A]) -> Option<Pair<A, Apply<Self, (A,)>>>
	where
		A: Clone,
	{
		match &slice {
			[] => None,
			[head, tail @ ..] => Some(Pair(head.to_owned(), tail.to_owned())),
		}
	}
}

impl Pure for VecBrand {
	fn pure<A>(a: A) -> Apply<Self, (A,)>
	where
		Self: Kind<(A,)>,
	{
		<Self as Brand<Vec<A>, (A,)>>::inject(vec![a])
	}
}

impl Functor for VecBrand {
	fn map<'a, A, B>(f: ClonableFn<'a, A, B>) -> impl Fn(Apply<Self, (A,)>) -> Apply<Self, (B,)>
	where
		Self: Kind<(A,)> + Kind<(B,)>,
	{
		move |fa| {
			<Self as Brand<_, (_,)>>::inject(
				<Self as Brand<_, (_,)>>::project(fa).into_iter().map(&*f).collect(),
			)
		}
	}
}

impl TypeclassApply for VecBrand {
	fn apply<'a, F, A, B>(ff: Apply<Self, (F,)>) -> impl Fn(Apply<Self, (A,)>) -> Apply<Self, (B,)>
	where
		Self: Kind<(F,)> + Kind<(A,)> + Kind<(B,)>,
		F: 'a + Fn(A) -> B,
		A: Clone,
		Apply<Self, (F,)>: Clone,
	{
		move |fa| {
			let fa = <Self as Brand<_, (_,)>>::project(fa);
			<Self as Brand<_, (_,)>>::inject(
				<Self as Brand<_, (F,)>>::project(ff.to_owned())
					.into_iter()
					.flat_map(|f| fa.iter().cloned().map(f))
					.collect(),
			)
		}
	}
}

impl ApplyFirst for VecBrand {
	fn apply_first<A, B>(fa: Apply<Self, (A,)>) -> impl Fn(Apply<Self, (B,)>) -> Apply<Self, (A,)>
	where
		Self: Kind<(A,)> + Kind<(B,)>,
		A: Clone,
		B: Clone,
		Apply<Self, (A,)>: Clone,
	{
		move |fb| {
			let fb = <Self as Brand<_, (B,)>>::project(fb);
			<Self as Brand<_, (_,)>>::inject(
				<Self as Brand<_, (A,)>>::project(fa.to_owned())
					.into_iter()
					.flat_map(|a| fb.iter().cloned().map(move |_b| a.to_owned()))
					.collect(),
			)
		}
	}
}

impl ApplySecond for VecBrand {
	fn apply_second<A, B>(fa: Apply<Self, (A,)>) -> impl Fn(Apply<Self, (B,)>) -> Apply<Self, (B,)>
	where
		Self: Kind<(A,)> + Kind<(B,)>,
		Apply<Self, (A,)>: Clone,
		B: Clone,
	{
		move |fb| {
			let fb = <Self as Brand<_, (B,)>>::project(fb);
			<Self as Brand<_, (_,)>>::inject(
				<Self as Brand<_, (A,)>>::project(fa.to_owned())
					.into_iter()
					.flat_map(|_a| fb.iter().cloned())
					.collect(),
			)
		}
	}
}

impl Foldable for VecBrand {
	fn fold_right<'a, A, B>(
		f: ClonableFn<'a, A, ClonableFn<'a, B, B>>
	) -> ClonableFn<'a, B, ClonableFn<'a, Apply<Self, (A,)>, B>>
	where
		Self: 'a + Kind<(A,)>,
		A: 'a + Clone,
		B: 'a + Clone,
		Apply<Self, (A,)>: 'a,
	{
		Arc::new(move |b| {
			let f = f.clone();
			Arc::new(move |fa| {
				<VecBrand as Brand<_, _>>::project(fa).iter().rfold(b.to_owned(), {
					let f = f.clone();
					let f = move |b, a| f(a)(b);
					move |b, a| f(b, a.to_owned())
				})
			})
		})
	}
}

impl Traversable for VecBrand {
	/// traverse f Vec.empty = pure Vec.empty
	/// traverse f (Vec.construct head tail) = (apply ((map Vec.construct) (f head))) ((traverse f) tail)
	fn traverse<'a, F, A, B>(
		f: ClonableFn<'a, A, Apply<F, (B,)>>
	) -> ClonableFn<'a, Apply<Self, (A,)>, Apply<F, (Apply<Self, (B,)>,)>>
	where
		Self: Kind<(A,)> + Kind<(B,)> + Kind<(Apply<F, (B,)>,)>,
		F: 'a + Kind<(B,)> + Kind<(Apply<Self, (B,)>,)> + Applicative,
		A: 'a + Clone,
		B: Clone,
		Apply<F, (B,)>: 'a + Clone,
		Apply<Self, (B,)>: Clone,
	{
		Arc::new(move |ta| {
			match VecBrand::deconstruct(&(<Self as Brand<Vec<A>, _>>::project(ta))) {
				None => pure::<F, _>(<Self as Brand<_, (B,)>>::inject(vec![])),
				Some(Pair(head, tail)) => {
					// cons: a -> (t a -> t a)
					let cons: ClonableFn<
						'a,
						A,
						ClonableFn<'a, Apply<Self, (A,)>, Apply<Self, (A,)>>,
					> = Arc::new(VecBrand::construct);
					// map: (a -> b) -> f a -> f b
					// cons: a -> (t a -> t a)
					// map cons = f a -> f (t a -> t a)
					let map_cons: ClonableFn<
						'a,
						Apply<F, (A,)>,
						Apply<F, (ClonableFn<'a, Apply<Self, (A,)>, Apply<Self, (A,)>>,)>,
					> = Arc::new(map(cons));
					// f: a -> f b
					// head: a
					// f head: f b
					let f_head: Apply<F, (B,)> = f(head);
					// traverse: (a -> f b) -> t a -> f (t b)
					// f: a -> f b
					// traverse f: t a -> f (t b)
					// tail: t a
					// (traverse f) tail: f (t b)
					let traverse_f_tail: Apply<F, (Apply<Self, (B,)>,)> = traverse(f)(tail);
					// map cons: f a -> f (t a -> t a)
					// f head: f b
					// (map cons) (f head): f (t b -> t b)
					let map_cons_f_head: Apply<
						F,
						(ClonableFn<'a, Apply<Self, (B,)>, Apply<Self, (B,)>>,),
					> = map_cons(f_head);
					// apply: f (a -> b) -> f a -> f b
					// (map cons) (f head): f (t b -> t b)
					// apply ((map cons) (f head)): f (t b) -> f (t b)
					// (traverse f) tail: f (t b)
					// apply ((map cons) (f head)) ((traverse f) tail): f (t b)
					apply::<
						F,
						Apply<F, (ClonableFn<'a, Apply<Self, (B,)>, Apply<Self, (B,)>>,)>,
						Apply<Self, (B,)>,
						Apply<Self, (B,)>,
					>(map_cons_f_head)(traverse_f_tail)
				}
			}
		})
	}
}
