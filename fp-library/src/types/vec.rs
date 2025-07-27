//! Implementations for [`Vec`].

use crate::{
	brands::{Brand, Brand1},
	hkt::{Apply, Kind, Kind1},
	impl_brand,
	typeclasses::{Apply as TypeclassApply, ApplyFirst, ApplySecond, Bind, Functor, Pure},
};

impl_brand!(
	/// [Brand][crate::brands] for [`Vec`].
	VecBrand,
	Vec,
	Kind1,
	Brand1,
	(A)
);

impl Pure for VecBrand {
	/// # Examples
	///
	/// ```
	/// use fp_library::{brands::VecBrand, functions::pure};
	///
	/// assert_eq!(pure::<VecBrand, _>(1), vec![1]);
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
	///
	/// assert_eq!(map::<VecBrand, _, _, _>(identity)(vec![] as Vec<()>), vec![]);
	/// assert_eq!(map::<VecBrand, _, _, _>(|x: i32| x * 2)(vec![1, 2, 3]), vec![2, 4, 6]);
	/// ```
	fn map<F, A, B>(f: F) -> impl Fn(Apply<Self, (A,)>) -> Apply<Self, (B,)>
	where
		Self: Kind<(A,)> + Kind<(B,)>,
		F: Fn(A) -> B,
	{
		move |fa| {
			<Self as Brand<_, (_,)>>::inject(
				<Self as Brand<_, (_,)>>::project(fa).into_iter().map(&f).collect(),
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
	/// assert_eq!(apply::<VecBrand, _, _, _>(vec![] as Vec<fn(i32) -> i32>)(vec![1, 2, 3]), vec![] as Vec<i32>);
	/// assert_eq!(apply::<VecBrand, _, _, _>(vec![identity, |x: i32| x * 2])(vec![1, 2]), vec![1, 2, 2, 4]);
	/// ```
	fn apply<F, A, B>(ff: Apply<Self, (F,)>) -> impl Fn(Apply<Self, (A,)>) -> Apply<Self, (B,)>
	where
		Self: Kind<(F,)> + Kind<(A,)> + Kind<(B,)>,
		F: Fn(A) -> B,
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
	/// assert_eq!(apply_first::<VecBrand, _, _>(vec![] as Vec<i32>)(vec![1, 2]), vec![] as Vec<i32>);
	/// assert_eq!(apply_first::<VecBrand, _, _>(vec![1, 2])(vec![] as Vec<i32>), vec![] as Vec<i32>);
	/// assert_eq!(apply_first::<VecBrand, _, _>(vec![1, 2])(vec![3, 4]), vec![1, 1, 2, 2]);
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
	/// assert_eq!(apply_second::<VecBrand, _, _>(vec![] as Vec<i32>)(vec![1, 2]), vec![] as Vec<i32>);
	/// assert_eq!(apply_second::<VecBrand, _, _>(vec![1, 2])(vec![] as Vec<i32>), vec![] as Vec<i32>);
	/// assert_eq!(apply_second::<VecBrand, _, _>(vec![1, 2])(vec![3, 4]), vec![3, 4, 3, 4]);
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
	/// assert_eq!(bind::<VecBrand, _, _, _>(vec![] as Vec<()>)(|_| pure::<VecBrand, _>(1)), vec![] as Vec<i32>);
	/// assert_eq!(bind::<VecBrand, _, _, _>(vec![1, 2])(|x| vec![x, x * 2]), vec![1, 2, 2, 4]);
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
