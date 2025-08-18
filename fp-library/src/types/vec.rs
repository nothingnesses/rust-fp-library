//! Implementations for [`Vec`].

pub mod concrete_vec;

use crate::{
	aliases::ArcFn,
	functions::{apply, map, pure, traverse},
	hkt::{Apply1, Brand1, Kind1},
	typeclasses::{
		Applicative, Apply as TypeclassApply, ApplyFirst, ApplySecond, Bind, Foldable, Functor,
		Pure, Traversable,
	},
	types::Pair,
};
pub use concrete_vec::*;
use std::sync::Arc;

pub struct VecBrand;

impl Kind1 for VecBrand {
	type Output<A> = Vec<A>;
}

impl VecBrand {
	pub fn construct<'a, A>(head: A) -> ArcFn<'a, Apply1<Self, A>, Apply1<Self, A>>
	where
		A: 'a + Clone,
	{
		Arc::new(move |tail| [vec![head.to_owned()], tail].concat())
	}

	pub fn deconstruct<'a, A>(slice: &[A]) -> Option<Pair<A, Apply1<Self, A>>>
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
	fn pure<A>(a: A) -> Apply1<Self, A> {
		vec![a]
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
	fn map<'a, A: 'a, B: 'a>(f: ArcFn<'a, A, B>) -> ArcFn<'a, Apply1<Self, A>, Apply1<Self, B>> {
		Arc::new(move |fa| fa.into_iter().map(&*f).collect())
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
	fn apply<'a, F: 'a + Fn(A) -> B, A: 'a + Clone, B: 'a>(
		ff: Apply1<Self, F>
	) -> ArcFn<'a, Apply1<Self, A>, Apply1<Self, B>>
	where
		Apply1<Self, F>: Clone,
	{
		Arc::new(move |fa| {
			ff.to_owned().into_iter().flat_map(|f| fa.iter().cloned().map(f)).collect()
		})
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
	fn apply_first<'a, A: 'a + Clone, B>(
		fa: Apply1<Self, A>
	) -> ArcFn<'a, Apply1<Self, B>, Apply1<Self, A>>
	where
		Apply1<Self, A>: Clone,
	{
		Arc::new(move |fb| {
			fa.to_owned().into_iter().flat_map(|a| fb.iter().map(move |_b| a.to_owned())).collect()
		})
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
	fn apply_second<'a, A: 'a, B: 'a + Clone>(
		fa: Apply1<Self, A>
	) -> ArcFn<'a, Apply1<Self, B>, Apply1<Self, B>>
	where
		Apply1<Self, A>: Clone,
	{
		Arc::new(move |fb| fa.to_owned().into_iter().flat_map(|_a| fb.iter().cloned()).collect())
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
	fn bind<'a, F: Fn(A) -> Apply1<Self, B>, A: 'a + Clone, B>(
		ma: Apply1<Self, A>
	) -> ArcFn<'a, F, Apply1<Self, B>> {
		Arc::new(move |f| ma.to_owned().into_iter().flat_map(|a| f(a)).collect())
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
	fn fold_right<'a, A: 'a + Clone, B: 'a + Clone>(
		f: ArcFn<'a, A, ArcFn<'a, B, B>>
	) -> ArcFn<'a, B, ArcFn<'a, Apply1<Self, A>, B>> {
		Arc::new(move |b| {
			let f = f.clone();
			Arc::new(move |fa| {
				fa.iter().rfold(b.to_owned(), {
					let f = f.clone();
					let f = move |b, a| f(a)(b);
					move |b, a| f(b, a.to_owned())
				})
			})
		})
	}
}
