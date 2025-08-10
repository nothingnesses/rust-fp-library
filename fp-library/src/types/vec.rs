//! Implementations for [`Vec`].

pub mod concrete_vec;

use crate::{
	aliases::ClonableFn,
	functions::pure,
	hkt::{Apply, Brand, Brand1, Kind, Kind1},
	impl_brand,
	typeclasses::{
		Applicative, Apply as TypeclassApply, ApplyFirst, ApplySecond, Bind, Foldable, Functor,
		Pure, Traversable,
	},
	types::Pair,
};
pub use concrete_vec::*;
use std::sync::Arc;

impl_brand!(VecBrand, Vec, Kind1, Brand1, (A));

impl VecBrand {
	pub fn construct<A>(head: A) -> impl Fn(Vec<A>) -> Vec<A>
	where
		A: Clone,
	{
		move |tail| [vec![head.to_owned()], tail].concat()
	}

	pub fn deconstruct<A>(slice: &[A]) -> Option<Pair<A, Vec<A>>>
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
	/// # Examples
	///
	/// ```
	/// use fp_library::{brands::VecBrand, functions::pure};
	///
	/// assert_eq!(
	///     pure::<VecBrand, _>(1),
	///     vec![1]
	/// );
	/// ```
	fn pure<A>(a: A) -> Apply<Self, (A,)>
	where
		Self: Kind<(A,)>,
	{
		<Self as Brand<Vec<A>, (A,)>>::inject(vec![a])
	}
}

impl Functor for VecBrand {
	/// # Examples
	///
	/// ```
	/// use fp_library::{brands::VecBrand, functions::{identity, map}};
	/// use std::sync::Arc;
	///
	/// assert_eq!(
	///     map::<VecBrand, _, _>(Arc::new(identity))(vec![] as Vec<()>),
	///     vec![]
	/// );
	/// assert_eq!(
	///     map::<VecBrand, _, _>(Arc::new(|x: i32| x * 2))(vec![1, 2, 3]),
	///     vec![2, 4, 6]
	/// );
	/// ```
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
	/// # Examples
	///
	/// ```
	/// use fp_library::{brands::VecBrand, functions::{apply, identity}};
	///
	/// assert_eq!(
	///     apply::<VecBrand, _, _, _>(vec![] as Vec<fn(i32) -> i32>)(vec![1, 2, 3]),
	///     vec![] as Vec<i32>
	/// );
	/// assert_eq!(
	///     apply::<VecBrand, _, _, _>(vec![identity, |x: i32| x * 2])(vec![1, 2]),
	///     vec![1, 2, 2, 4]
	/// );
	/// ```
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
	/// # Examples
	///
	/// ```
	/// use fp_library::{brands::VecBrand, functions::apply_first};
	///
	/// assert_eq!(
	///     apply_first::<VecBrand, _, _>(vec![] as Vec<i32>)(vec![1, 2]),
	///     vec![] as Vec<i32>
	/// );
	/// assert_eq!(
	///     apply_first::<VecBrand, _, _>(vec![1, 2])(vec![] as Vec<i32>),
	///     vec![] as Vec<i32>
	/// );
	/// assert_eq!(
	///     apply_first::<VecBrand, _, _>(vec![1, 2])(vec![3, 4]),
	///     vec![1, 1, 2, 2]
	/// );
	/// ```
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
	/// # Examples
	///
	/// ```
	/// use fp_library::{brands::VecBrand, functions::apply_second};
	///
	/// assert_eq!(
	///     apply_second::<VecBrand, _, _>(vec![] as Vec<i32>)(vec![1, 2]),
	///     vec![] as Vec<i32>
	/// );
	/// assert_eq!(
	///     apply_second::<VecBrand, _, _>(vec![1, 2])(vec![] as Vec<i32>),
	///     vec![] as Vec<i32>
	/// );
	/// assert_eq!(
	///     apply_second::<VecBrand, _, _>(vec![1, 2])(vec![3, 4]),
	///     vec![3, 4, 3, 4]
	/// );
	/// ```
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

impl Bind for VecBrand {
	/// # Examples
	///
	/// ```
	/// use fp_library::{brands::VecBrand, functions::{bind, pure}};
	///
	/// assert_eq!(
	///     bind::<VecBrand, _, _, _>(vec![] as Vec<()>)(|_| pure::<VecBrand, _>(1)),
	///     vec![] as Vec<i32>
	/// );
	/// assert_eq!(
	///     bind::<VecBrand, _, _, _>(vec![1, 2])(|x| vec![x, x * 2]),
	///     vec![1, 2, 2, 4]
	/// );
	/// ```
	fn bind<F, A, B>(ma: Apply<Self, (A,)>) -> impl Fn(F) -> Apply<Self, (B,)>
	where
		Self: Kind<(A,)> + Kind<(B,)> + Sized,
		F: Fn(A) -> Apply<Self, (B,)>,
		Apply<Self, (A,)>: Clone,
	{
		move |f| {
			<Self as Brand<_, (_,)>>::inject(
				<Self as Brand<_, (_,)>>::project(ma.to_owned())
					.into_iter()
					.flat_map(|a| <Self as Brand<_, (B,)>>::project(f(a)))
					.collect(),
			)
		}
	}
}

impl Foldable for VecBrand {
	/// # Examples
	///
	/// ```
	/// use fp_library::{brands::VecBrand, functions::fold_right};
	/// use std::sync::Arc;
	///
	/// assert_eq!(
	///     fold_right::<VecBrand, _, _>(Arc::new(|item| Arc::new(move |carry| carry * 2 + item)))(0)(vec![1, 2, 3]),
	///     17
	/// );
	/// ```
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
	/// Vec.empty = pure Vec.empty
	/// Vec.constructor a as = (apply ((map Vec.constructor) (f a))) ((traverse f) as)
	fn traverse<'a, F, A, B>(
		f: ClonableFn<'a, A, Apply<F, (B,)>>
	) -> ClonableFn<'a, Apply<Self, (A,)>, Apply<F, (Apply<Self, (B,)>,)>>
	where
		Self: Kind<(A,)> + Kind<(B,)> + Kind<(Apply<F, (B,)>,)>,
		F: 'a + Kind<(B,)> + Kind<(Apply<Self, (B,)>,)> + Applicative,
		A: 'a + Clone,
		B: Clone,
		Apply<F, (B,)>: 'a,
	{
		Arc::new(move |ta| {
			match VecBrand::deconstruct(&(<Self as Brand<Vec<A>, _>>::project(ta))) {
				Some(_) => todo!(),
				None => pure::<F, _>(<Self as Brand<_, (B,)>>::inject(vec![])),
			}
		})
	}
}
