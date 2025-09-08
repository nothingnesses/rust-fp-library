//! Implementations for [`Vec`].

pub mod concrete_vec;

use crate::{
	classes::{
		Applicative, ApplyFirst, ApplySecond, ClonableFn, Foldable, Functor, Pointed,
		Semiapplicative, Semimonad, Traversable, clonable_fn::ApplyFn,
	},
	functions::{apply, map, pure, traverse},
	hkt::{Apply0L1T, Kind0L1T},
	types::Pair,
};

pub struct VecBrand;

impl Kind0L1T for VecBrand {
	type Output<A> = Vec<A>;
}

impl VecBrand {
	/// Constructs a new vector by prepending a value to an existing vector.
	///
	/// # Type Signature
	///
	/// `forall a. a -> Vec a -> Vec a`
	///
	/// # Parameters
	///
	/// * `head`: A value to prepend to the vector.
	/// * `tail`: A vector to prepend the value to.
	///
	/// # Returns
	///
	/// A new vector consisting of the `head` element prepended to the `tail` vector.
	///
	/// # Examples
	///
	/// ```
	/// use fp_library::brands::{RcFnBrand, VecBrand};
	///
	/// let head = 1;
	/// let tail = vec![2, 3];
	/// let new_vec = (VecBrand::construct::<RcFnBrand, _>(head))(tail);
	/// assert_eq!(new_vec, vec![1, 2, 3]);
	///
	/// let empty_tail = vec![];
	/// let single_element = (VecBrand::construct::<RcFnBrand, _>(42))(empty_tail);
	/// assert_eq!(single_element, vec![42]);
	/// ```
	pub fn construct<'a, ClonableFnBrand: 'a + ClonableFn, A>(
		head: A
	) -> ApplyFn<'a, ClonableFnBrand, Apply0L1T<Self, A>, Apply0L1T<Self, A>>
	where
		A: Clone,
	{
		ClonableFnBrand::new(move |tail| [vec![head.to_owned()], tail].concat())
	}

	/// Deconstructs a slice into its head element and tail vector.
	///
	/// # Type Signature
	///
	/// `forall a. &[a] -> Option (Pair a (Vec a))`
	///
	/// # Parameters
	///
	/// * `slice`: The vector slice to deconstruct.
	///
	/// # Returns
	///
	/// An [`Option`] containing a [`Pair`] of the head element and the remaining tail vector,
	/// or [`None`] if the slice is empty.
	///
	/// # Examples
	///
	/// ```
	/// use fp_library::{brands::VecBrand, types::Pair};
	///
	/// let vec = vec![1, 2, 3];
	/// let deconstructed = VecBrand::deconstruct(&vec);
	/// assert_eq!(deconstructed, Some(Pair(1, vec![2, 3])));
	///
	/// let empty: Vec<i32> = vec![];
	/// assert_eq!(VecBrand::deconstruct(&empty), None);
	/// ```
	pub fn deconstruct<A>(slice: &[A]) -> Option<Pair<A, Apply0L1T<Self, A>>>
	where
		A: Clone,
	{
		match &slice {
			[] => None,
			[head, tail @ ..] => Some(Pair(head.to_owned(), tail.to_owned())),
		}
	}
}

impl Functor for VecBrand {
	/// # Examples
	///
	/// ```
	/// use fp_library::{brands::{VecBrand, RcFnBrand}, functions::{identity, map}};
	/// use std::rc::Rc;
	///
	/// assert_eq!(
	///     map::<RcFnBrand, VecBrand, _, _>(Rc::new(identity))(vec![] as Vec<()>),
	///     vec![]
	/// );
	/// assert_eq!(
	///     map::<RcFnBrand, VecBrand, _, _>(Rc::new(|x: i32| x * 2))(vec![1, 2, 3]),
	///     vec![2, 4, 6]
	/// );
	/// ```
	fn map<'a, ClonableFnBrand: 'a + ClonableFn, A: 'a, B: 'a>(
		f: ApplyFn<'a, ClonableFnBrand, A, B>
	) -> ApplyFn<'a, ClonableFnBrand, Apply0L1T<Self, A>, Apply0L1T<Self, B>> {
		ClonableFnBrand::new(move |fa: Apply0L1T<Self, _>| fa.into_iter().map(&*f).collect())
	}
}

impl Semiapplicative for VecBrand {
	/// # Examples
	///
	/// ```
	/// use fp_library::{brands::{VecBrand, RcFnBrand}, functions::{apply, identity}};
	/// use std::rc::Rc;
	///
	/// assert_eq!(
	///     apply::<RcFnBrand, VecBrand, _, _>(vec![] as Vec<Rc<dyn Fn(i32) -> i32>>)(vec![1, 2, 3]),
	///     vec![] as Vec<i32>
	/// );
	/// assert_eq!(
	///     apply::<RcFnBrand, VecBrand, _, _>(vec![Rc::new(identity), Rc::new(|x: i32| x * 2)])(vec![1, 2]),
	///     vec![1, 2, 2, 4]
	/// );
	/// ```
	fn apply<'a, ClonableFnBrand: 'a + ClonableFn, A: 'a + Clone, B: 'a>(
		ff: Apply0L1T<Self, ApplyFn<'a, ClonableFnBrand, A, B>>
	) -> ApplyFn<'a, ClonableFnBrand, Apply0L1T<Self, A>, Apply0L1T<Self, B>> {
		ClonableFnBrand::new(move |fa: Apply0L1T<Self, _>| {
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
	/// use fp_library::{brands::{VecBrand, RcFnBrand}, functions::apply_first};
	/// use std::rc::Rc;
	///
	/// assert_eq!(
	///     apply_first::<RcFnBrand, VecBrand, _, _>(vec![] as Vec<i32>)(vec![1, 2]),
	///     vec![] as Vec<i32>
	/// );
	/// assert_eq!(
	///     apply_first::<RcFnBrand, VecBrand, _, _>(vec![1, 2])(vec![] as Vec<i32>),
	///     vec![] as Vec<i32>
	/// );
	/// assert_eq!(
	///     apply_first::<RcFnBrand, VecBrand, _, _>(vec![1, 2])(vec![3, 4]),
	///     vec![1, 1, 2, 2]
	/// );
	/// ```
	fn apply_first<'a, ClonableFnBrand: 'a + ClonableFn, A: 'a + Clone, B: Clone>(
		fa: Apply0L1T<Self, A>
	) -> ApplyFn<'a, ClonableFnBrand, Apply0L1T<Self, B>, Apply0L1T<Self, A>> {
		ClonableFnBrand::new(move |fb: Apply0L1T<Self, _>| {
			fa.iter().cloned().flat_map(|a| fb.iter().map(move |_b| a.to_owned())).collect()
		})
	}
}

impl ApplySecond for VecBrand {
	/// # Examples
	///
	/// ```
	/// use fp_library::{brands::{VecBrand, RcFnBrand}, functions::apply_second};
	/// use std::rc::Rc;
	///
	/// assert_eq!(
	///     apply_second::<RcFnBrand, VecBrand, _, _>(vec![] as Vec<i32>)(vec![1, 2]),
	///     vec![] as Vec<i32>
	/// );
	/// assert_eq!(
	///     apply_second::<RcFnBrand, VecBrand, _, _>(vec![1, 2])(vec![] as Vec<i32>),
	///     vec![] as Vec<i32>
	/// );
	/// assert_eq!(
	///     apply_second::<RcFnBrand, VecBrand, _, _>(vec![1, 2])(vec![3, 4]),
	///     vec![3, 4, 3, 4]
	/// );
	/// ```
	fn apply_second<'a, ClonableFnBrand: 'a + ClonableFn, A: 'a + Clone, B: 'a + Clone>(
		fa: Apply0L1T<Self, A>
	) -> ApplyFn<'a, ClonableFnBrand, Apply0L1T<Self, B>, Apply0L1T<Self, B>> {
		ClonableFnBrand::new(move |fb: Apply0L1T<Self, _>| {
			fa.iter().cloned().flat_map(|_a| fb.iter().cloned()).collect()
		})
	}
}

impl Pointed for VecBrand {
	/// # Examples
	///
	/// ```
	/// use fp_library::{brands::{RcFnBrand, VecBrand}, functions::pure};
	///
	/// assert_eq!(
	///     pure::<RcFnBrand, VecBrand, _>(1),
	///     vec![1]
	/// );
	/// ```
	fn pure<ClonableFnBrand: ClonableFn, A: Clone>(a: A) -> Apply0L1T<Self, A> {
		vec![a]
	}
}

impl Semimonad for VecBrand {
	/// # Examples
	///
	/// ```
	/// use fp_library::{brands::{VecBrand, RcFnBrand}, functions::{bind, pure}};
	/// use std::rc::Rc;
	///
	/// assert_eq!(
	///     bind::<RcFnBrand, VecBrand, _, _>(vec![] as Vec<()>)(Rc::new(|_| pure::<RcFnBrand, VecBrand, _>(1))),
	///     vec![] as Vec<i32>
	/// );
	/// assert_eq!(
	///     bind::<RcFnBrand, VecBrand, _, _>(vec![1, 2])(Rc::new(|x| vec![x, x * 2])),
	///     vec![1, 2, 2, 4]
	/// );
	/// ```
	fn bind<'a, ClonableFnBrand: 'a + ClonableFn, A: 'a + Clone, B: Clone>(
		ma: Apply0L1T<Self, A>
	) -> ApplyFn<
		'a,
		ClonableFnBrand,
		ApplyFn<'a, ClonableFnBrand, A, Apply0L1T<Self, B>>,
		Apply0L1T<Self, B>,
	> {
		ClonableFnBrand::new(move |f: ApplyFn<'a, ClonableFnBrand, _, _>| {
			ma.iter().cloned().flat_map(&*f).collect()
		})
	}
}

impl Foldable for VecBrand {
	/// # Examples
	///
	/// ```
	/// use fp_library::{brands::{VecBrand, RcFnBrand}, functions::fold_right};
	/// use std::rc::Rc;
	///
	/// assert_eq!(
	///     fold_right::<RcFnBrand, VecBrand, _, _>(Rc::new(|item| Rc::new(move |carry| carry * 2 + item)))(0)(vec![1, 2, 3]),
	///     17
	/// );
	/// ```
	fn fold_right<'a, ClonableFnBrand: 'a + ClonableFn, A: Clone, B: Clone>(
		f: ApplyFn<'a, ClonableFnBrand, A, ApplyFn<'a, ClonableFnBrand, B, B>>
	) -> ApplyFn<'a, ClonableFnBrand, B, ApplyFn<'a, ClonableFnBrand, Apply0L1T<Self, A>, B>> {
		ClonableFnBrand::new(move |b: B| {
			let f = f.clone();
			ClonableFnBrand::new(move |fa: Apply0L1T<Self, A>| {
				fa.iter().rfold(b.to_owned(), {
					let f = f.clone();
					let f = move |b, a| f(a)(b);
					move |b, a| f(b, a.to_owned())
				})
			})
		})
	}
}

impl Traversable for VecBrand {
	// traverse f Vec.empty = pure Vec.empty
	// traverse f (Vec.construct head tail) = (apply ((map Vec.construct) (f head))) ((traverse f) tail)
	/// # Examples
	///
	/// ```
	/// use fp_library::{brands::{VecBrand, RcFnBrand, OptionBrand}, functions::traverse};
	/// use std::rc::Rc;
	///
	/// assert_eq!(
	///     traverse::<RcFnBrand, VecBrand, OptionBrand, i32, i32>(Rc::new(|x| Some(x * 2)))(vec![1, 2, 3]),
	///     Some(vec![2, 4, 6])
	/// );
	/// assert_eq!(
	///     traverse::<RcFnBrand, VecBrand, OptionBrand, i32, i32>(Rc::new(|_x| None))(vec![1, 2, 3]),
	///     None
	/// );
	/// ```
	fn traverse<'a, ClonableFnBrand: 'a + ClonableFn, F: Applicative, A: Clone, B: 'a + Clone>(
		f: ApplyFn<'a, ClonableFnBrand, A, Apply0L1T<F, B>>
	) -> ApplyFn<'a, ClonableFnBrand, Apply0L1T<Self, A>, Apply0L1T<F, Apply0L1T<Self, B>>>
	where
		Apply0L1T<F, B>: Clone,
		Apply0L1T<F, ApplyFn<'a, ClonableFnBrand, Apply0L1T<Self, B>, Apply0L1T<Self, B>>>: Clone,
		Apply0L1T<Self, B>: 'a,
		Apply0L1T<Self, Apply0L1T<F, B>>: 'a,
	{
		ClonableFnBrand::new(move |ta: Apply0L1T<Self, _>| {
			match VecBrand::deconstruct(&ta) {
				None => pure::<ClonableFnBrand, F, _>(vec![]),
				Some(Pair(head, tail)) => {
					// cons: a -> (t a -> t a)
					let cons = ClonableFnBrand::new(VecBrand::construct::<ClonableFnBrand, _>);
					// map: (a -> b) -> f a -> f b
					// cons: a -> (t a -> t a)
					// map cons = f a -> f (t a -> t a)
					let map_cons = map::<ClonableFnBrand, F, _, _>(cons);
					// f: a -> f b
					// head: a
					// f head: f b
					let f_head = f(head);
					// traverse: (a -> f b) -> t a -> f (t b)
					// f: a -> f b
					// traverse f: t a -> f (t b)
					let traverse_f = traverse::<ClonableFnBrand, Self, F, _, _>(f.clone());
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
					let apply_map_cons_f_head = apply::<ClonableFnBrand, F, _, _>(map_cons_f_head);
					// apply ((map cons) (f head)): f (t b) -> f (t b)
					// (traverse f) tail: f (t b)
					// apply ((map cons) (f head)) ((traverse f) tail): f (t b)
					apply_map_cons_f_head(traverse_f_tail)
				}
			}
		})
	}
}
