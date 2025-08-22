//! Implementations for [`Vec`].

pub mod concrete_vec;

use crate::{
	aliases::ArcFn,
	functions::{apply, map, pure, traverse},
	hkt::{Apply1, Kind1},
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

	pub fn deconstruct<A>(slice: &[A]) -> Option<Pair<A, Apply1<Self, A>>>
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
	/// use std::sync::Arc;
	///
	/// assert_eq!(
	///     apply::<VecBrand, _, _>(vec![] as Vec<Arc<dyn Fn(i32) -> i32>>)(vec![1, 2, 3]),
	///     vec![] as Vec<i32>
	/// );
	/// assert_eq!(
	///     apply::<VecBrand, _, _>(vec![Arc::new(identity), Arc::new(|x: i32| x * 2)])(vec![1, 2]),
	///     vec![1, 2, 2, 4]
	/// );
	/// ```
	fn apply<'a, A: 'a + Clone, B: 'a>(
		ff: Apply1<Self, ArcFn<'a, A, B>>
	) -> ArcFn<'a, Apply1<Self, A>, Apply1<Self, B>>
	where
		Apply1<Self, ArcFn<'a, A, B>>: Clone,
	{
		Arc::new(move |fa| {
			ff.iter()
				.cloned()
				.flat_map(|f| fa.iter().cloned().map(&*f).collect::<Vec<_>>())
				.collect()
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
			fa.iter().cloned().flat_map(|a| fb.iter().map(move |_b| a.to_owned())).collect()
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
	/// use std::sync::Arc;
	///
	/// assert_eq!(
	///     bind::<VecBrand, _, _>(vec![] as Vec<()>)(Arc::new(|_| pure::<VecBrand, _>(1))),
	///     vec![] as Vec<i32>
	/// );
	/// assert_eq!(
	///     bind::<VecBrand, _, _>(vec![1, 2])(Arc::new(|x| vec![x, x * 2])),
	///     vec![1, 2, 2, 4]
	/// );
	/// ```
	fn bind<'a, A: 'a + Clone, B>(
		ma: Apply1<Self, A>
	) -> ArcFn<'a, ArcFn<'a, A, Apply1<Self, B>>, Apply1<Self, B>> {
		Arc::new(move |f| ma.iter().cloned().flat_map(&*f).collect())
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

impl<'a> Traversable<'a> for VecBrand {
	// traverse f Vec.empty = pure Vec.empty
	// traverse f (Vec.construct head tail) = (apply ((map Vec.construct) (f head))) ((traverse f) tail)
	/// # Examples
	///
	/// ```
	/// use fp_library::{brands::{VecBrand, OptionBrand}, functions::traverse};
	/// use std::sync::Arc;
	///
	/// assert_eq!(
	///     traverse::<VecBrand, OptionBrand, i32, i32>(Arc::new(|x| Some(x * 2)))(vec![1, 2, 3]),
	///     Some(vec![2, 4, 6])
	/// );
	/// assert_eq!(
	///     traverse::<VecBrand, OptionBrand, i32, i32>(Arc::new(|_x| None))(vec![1, 2, 3]),
	///     None
	/// );
	/// ```
	fn traverse<F: Applicative, A: 'a + Clone, B: 'a + Clone>(
		f: ArcFn<'a, A, Apply1<F, B>>
	) -> ArcFn<'a, Apply1<Self, A>, Apply1<F, Apply1<Self, B>>>
	where
		Apply1<F, B>: 'a + Clone,
		Apply1<F, ArcFn<'a, Apply1<Self, B>, Apply1<Self, B>>>: Clone,
	{
		Arc::new(move |ta| {
			match VecBrand::deconstruct(&ta) {
				None => pure::<F, _>(vec![]),
				Some(Pair(head, tail)) => {
					// cons: a -> (t a -> t a)
					let cons = Arc::new(VecBrand::construct);
					// map: (a -> b) -> f a -> f b
					// cons: a -> (t a -> t a)
					// map cons = f a -> f (t a -> t a)
					let map_cons = map::<F, _, _>(cons);
					// f: a -> f b
					// head: a
					// f head: f b
					let f_head = f(head);
					// traverse: (a -> f b) -> t a -> f (t b)
					// f: a -> f b
					// traverse f: t a -> f (t b)
					let traverse_f = traverse::<Self, F, _, _>(f.clone());
					// traverse f: t a -> f (t b)
					// tail: t a
					// (traverse f) tail: f (t b)
					let traverse_f_tail = traverse_f(tail);
					// map cons: f a -> f (t a -> t a)
					// f head: f b
					// (map cons) (f head): f (t b -> t b)
					let map_cons_f_head = map_cons(f_head);
					// apply: f (a -> b) -> f a -> f b
					// (map cons) (f head): f (t b -> t b)
					// apply ((map cons) (f head)): f (t b) -> f (t b)
					let apply_map_cons_f_head = apply::<F, _, _>(map_cons_f_head);
					// apply ((map cons) (f head)): f (t b) -> f (t b)
					// (traverse f) tail: f (t b)
					// apply ((map cons) (f head)) ((traverse f) tail): f (t b)
					apply_map_cons_f_head(traverse_f_tail)
				}
			}
		})
	}
}
