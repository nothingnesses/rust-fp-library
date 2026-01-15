//! Implementations for [`Vec`].

use crate::{
	Apply,
	brands::VecBrand,
	classes::{
		applicative::Applicative, apply_first::ApplyFirst, apply_second::ApplySecond,
		clonable_fn::ClonableFn, foldable::Foldable, functor::Functor, lift::Lift, monoid::Monoid,
		pointed::Pointed, semiapplicative::Semiapplicative, semigroup::Semigroup,
		semimonad::Semimonad, traversable::Traversable,
	},
	impl_kind,
	kinds::*,
};

impl_kind! {
	for VecBrand {
		type Of<'a, A: 'a>: 'a = Vec<A>;
	}
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
	/// use fp_library::brands::VecBrand;
	///
	/// let head = 1;
	/// let tail = vec![2, 3];
	/// let new_vec = VecBrand::construct(head, tail);
	/// assert_eq!(new_vec, vec![1, 2, 3]);
	///
	/// let empty_tail = vec![];
	/// let single_element = VecBrand::construct(42, empty_tail);
	/// assert_eq!(single_element, vec![42]);
	/// ```
	pub fn construct<A>(
		head: A,
		tail: Vec<A>,
	) -> Vec<A>
	where
		A: Clone,
	{
		[vec![head], tail].concat()
	}

	/// Deconstructs a slice into its head element and tail vector.
	///
	/// # Type Signature
	///
	/// `forall a. &[a] -> Option (a, Vec a)`
	///
	/// # Parameters
	///
	/// * `slice`: The vector slice to deconstruct.
	///
	/// # Returns
	///
	/// An [`Option`] containing a tuple of the head element and the remaining tail vector,
	/// or [`None`] if the slice is empty.
	///
	/// # Examples
	///
	/// ```
	/// use fp_library::brands::VecBrand;
	///
	/// let vec = vec![1, 2, 3];
	/// let deconstructed = VecBrand::deconstruct(&vec);
	/// assert_eq!(deconstructed, Some((1, vec![2, 3])));
	///
	/// let empty: Vec<i32> = vec![];
	/// assert_eq!(VecBrand::deconstruct(&empty), None);
	/// ```
	pub fn deconstruct<A>(slice: &[A]) -> Option<(A, Vec<A>)>
	where
		A: Clone,
	{
		match slice {
			[] => None,
			[head, tail @ ..] => Some((head.clone(), tail.to_vec())),
		}
	}
}

impl Functor for VecBrand {
	/// Maps a function over the vector.
	///
	/// # Type Signature
	///
	/// `forall a b. Functor Vec => (a -> b, Vec a) -> Vec b`
	///
	/// # Parameters
	///
	/// * `f`: The function to apply to each element.
	/// * `fa`: The vector to map over.
	///
	/// # Returns
	///
	/// A new vector containing the results of applying the function.
	///
	/// # Examples
	///
	/// ```
	/// use fp_library::classes::functor::map;
	/// use fp_library::brands::VecBrand;
	///
	/// assert_eq!(map::<VecBrand, _, _, _>(|x: i32| x * 2, vec![1, 2, 3]), vec![2, 4, 6]);
	/// ```
	fn map<'a, A: 'a, B: 'a, F>(
		f: F,
		fa: Apply!(brand: Self, signature: ('a, A: 'a) -> 'a),
	) -> Apply!(brand: Self, signature: ('a, B: 'a) -> 'a)
	where
		F: Fn(A) -> B + 'a,
	{
		fa.into_iter().map(f).collect()
	}
}

impl Lift for VecBrand {
	/// Lifts a binary function into the vector context (Cartesian product).
	///
	/// # Type Signature
	///
	/// `forall a b c. Lift Vec => ((a, b) -> c, Vec a, Vec b) -> Vec c`
	///
	/// # Parameters
	///
	/// * `f`: The binary function to apply.
	/// * `fa`: The first vector.
	/// * `fb`: The second vector.
	///
	/// # Returns
	///
	/// A new vector containing the results of applying the function to all pairs of elements.
	///
	/// # Examples
	///
	/// ```
	/// use fp_library::classes::lift::lift2;
	/// use fp_library::brands::VecBrand;
	///
	/// assert_eq!(
	///     lift2::<VecBrand, _, _, _, _>(|x, y| x + y, vec![1, 2], vec![10, 20]),
	///     vec![11, 21, 12, 22]
	/// );
	/// ```
	fn lift2<'a, A, B, C, F>(
		f: F,
		fa: Apply!(brand: Self, signature: ('a, A: 'a) -> 'a),
		fb: Apply!(brand: Self, signature: ('a, B: 'a) -> 'a),
	) -> Apply!(brand: Self, signature: ('a, C: 'a) -> 'a)
	where
		F: Fn(A, B) -> C + 'a,
		A: Clone + 'a,
		B: Clone + 'a,
		C: 'a,
	{
		fa.iter().flat_map(|a| fb.iter().map(|b| f(a.clone(), b.clone()))).collect()
	}
}

impl Pointed for VecBrand {
	/// Wraps a value in a vector.
	///
	/// # Type Signature
	///
	/// `forall a. Pointed Vec => a -> Vec a`
	///
	/// # Parameters
	///
	/// * `a`: The value to wrap.
	///
	/// # Returns
	///
	/// A vector containing the single value.
	///
	/// # Examples
	///
	/// ```
	/// use fp_library::classes::pointed::pure;
	/// use fp_library::brands::VecBrand;
	///
	/// assert_eq!(pure::<VecBrand, _>(5), vec![5]);
	/// ```
	fn pure<'a, A: 'a>(a: A) -> Apply!(brand: Self, signature: ('a, A: 'a) -> 'a) {
		vec![a]
	}
}

impl ApplyFirst for VecBrand {}
impl ApplySecond for VecBrand {}

impl Semiapplicative for VecBrand {
	/// Applies wrapped functions to wrapped values (Cartesian product).
	///
	/// # Type Signature
	///
	/// `forall a b. Semiapplicative Vec => (Vec (a -> b), Vec a) -> Vec b`
	///
	/// # Parameters
	///
	/// * `ff`: The vector containing the functions.
	/// * `fa`: The vector containing the values.
	///
	/// # Returns
	///
	/// A new vector containing the results of applying each function to each value.
	///
	/// # Examples
	///
	/// ```
	/// use fp_library::classes::semiapplicative::apply;
	/// use fp_library::classes::clonable_fn::ClonableFn;
	/// use fp_library::brands::{VecBrand};
	/// use fp_library::brands::RcFnBrand;
	/// use std::rc::Rc;
	///
	/// let funcs = vec![
	///     <RcFnBrand as ClonableFn>::new(|x: i32| x + 1),
	///     <RcFnBrand as ClonableFn>::new(|x: i32| x * 2),
	/// ];
	/// assert_eq!(apply::<VecBrand, _, _, RcFnBrand>(funcs, vec![1, 2]), vec![2, 3, 2, 4]);
	/// ```
	fn apply<'a, A: 'a + Clone, B: 'a, FnBrand: 'a + ClonableFn>(
		ff: Apply!(brand: Self, signature: ('a, Apply!(brand: FnBrand, kind: ClonableFn, lifetimes: ('a), types: (A, B)): 'a) -> 'a),
		fa: Apply!(brand: Self, signature: ('a, A: 'a) -> 'a),
	) -> Apply!(brand: Self, signature: ('a, B: 'a) -> 'a) {
		ff.iter().flat_map(|f| fa.iter().map(move |a| f(a.clone()))).collect()
	}
}

impl Semimonad for VecBrand {
	/// Chains vector computations (flat_map).
	///
	/// # Type Signature
	///
	/// `forall a b. Semimonad Vec => (Vec a, a -> Vec b) -> Vec b`
	///
	/// # Parameters
	///
	/// * `ma`: The first vector.
	/// * `f`: The function to apply to each element, returning a vector.
	///
	/// # Returns
	///
	/// A new vector containing the flattened results.
	///
	/// # Examples
	///
	/// ```
	/// use fp_library::classes::semimonad::bind;
	/// use fp_library::brands::VecBrand;
	///
	/// assert_eq!(
	///     bind::<VecBrand, _, _, _>(vec![1, 2], |x| vec![x, x * 2]),
	///     vec![1, 2, 2, 4]
	/// );
	/// ```
	fn bind<'a, A: 'a, B: 'a, F>(
		ma: Apply!(brand: Self, signature: ('a, A: 'a) -> 'a),
		f: F,
	) -> Apply!(brand: Self, signature: ('a, B: 'a) -> 'a)
	where
		F: Fn(A) -> Apply!(brand: Self, signature: ('a, B: 'a) -> 'a) + 'a,
	{
		ma.into_iter().flat_map(f).collect()
	}
}

impl Foldable for VecBrand {
	/// Folds the vector from the right.
	///
	/// # Type Signature
	///
	/// `forall a b. Foldable Vec => ((a, b) -> b, b, Vec a) -> b`
	///
	/// # Parameters
	///
	/// * `f`: The folding function.
	/// * `init`: The initial value.
	/// * `fa`: The vector to fold.
	///
	/// # Returns
	///
	/// The final accumulator value.
	///
	/// # Examples
	///
	/// ```
	/// use fp_library::classes::foldable::fold_right;
	/// use fp_library::brands::VecBrand;
	///
	/// assert_eq!(fold_right::<VecBrand, _, _, _>(|x: i32, acc| x + acc, 0, vec![1, 2, 3]), 6);
	/// ```
	fn fold_right<'a, A: 'a, B: 'a, F>(
		f: F,
		init: B,
		fa: Apply!(brand: Self, signature: ('a, A: 'a) -> 'a),
	) -> B
	where
		F: Fn(A, B) -> B + 'a,
	{
		fa.into_iter().rev().fold(init, |acc, x| f(x, acc))
	}

	/// Folds the vector from the left.
	///
	/// # Type Signature
	///
	/// `forall a b. Foldable Vec => ((b, a) -> b, b, Vec a) -> b`
	///
	/// # Parameters
	///
	/// * `f`: The folding function.
	/// * `init`: The initial value.
	/// * `fa`: The vector to fold.
	///
	/// # Returns
	///
	/// The final accumulator value.
	///
	/// # Examples
	///
	/// ```
	/// use fp_library::classes::foldable::fold_left;
	/// use fp_library::brands::VecBrand;
	///
	/// assert_eq!(fold_left::<VecBrand, _, _, _>(|acc, x: i32| acc + x, 0, vec![1, 2, 3]), 6);
	/// ```
	fn fold_left<'a, A: 'a, B: 'a, F>(
		f: F,
		init: B,
		fa: Apply!(brand: Self, signature: ('a, A: 'a) -> 'a),
	) -> B
	where
		F: Fn(B, A) -> B + 'a,
	{
		fa.into_iter().fold(init, f)
	}

	/// Maps the values to a monoid and combines them.
	///
	/// # Type Signature
	///
	/// `forall a m. (Foldable Vec, Monoid m) => ((a) -> m, Vec a) -> m`
	///
	/// # Parameters
	///
	/// * `f`: The mapping function.
	/// * `fa`: The vector to fold.
	///
	/// # Returns
	///
	/// The combined monoid value.
	///
	/// # Examples
	///
	/// ```
	/// use fp_library::classes::foldable::fold_map;
	/// use fp_library::brands::VecBrand;
	/// use fp_library::types::string; // Import to bring Monoid impl for String into scope
	///
	/// assert_eq!(
	///     fold_map::<VecBrand, _, _, _>(|x: i32| x.to_string(), vec![1, 2, 3]),
	///     "123".to_string()
	/// );
	/// ```
	fn fold_map<'a, A: 'a, M, F>(
		f: F,
		fa: Apply!(brand: Self, signature: ('a, A: 'a) -> 'a),
	) -> M
	where
		M: Monoid + 'a,
		F: Fn(A) -> M + 'a,
	{
		fa.into_iter().map(f).fold(M::empty(), |acc, x| M::append(acc, x))
	}
}

impl Traversable for VecBrand {
	/// Traverses the vector with an applicative function.
	///
	/// # Type Signature
	///
	/// `forall a b f. (Traversable Vec, Applicative f) => (a -> f b, Vec a) -> f (Vec b)`
	///
	/// # Parameters
	///
	/// * `f`: The function to apply.
	/// * `ta`: The vector to traverse.
	///
	/// # Returns
	///
	/// The vector wrapped in the applicative context.
	///
	/// # Examples
	///
	/// ```
	/// use fp_library::classes::traversable::traverse;
	/// use fp_library::brands::{OptionBrand, VecBrand};
	///
	/// assert_eq!(
	///     traverse::<VecBrand, OptionBrand, _, _, _>(|x| Some(x * 2), vec![1, 2, 3]),
	///     Some(vec![2, 4, 6])
	/// );
	/// ```
	fn traverse<'a, F: Applicative, A: 'a + Clone, B: 'a + Clone, Func>(
		f: Func,
		ta: Apply!(brand: Self, signature: ('a, A: 'a) -> 'a),
	) -> Apply!(brand: F, signature: ('a, Apply!(brand: Self, signature: ('a, B: 'a) -> 'a): 'a) -> 'a)
	where
		Func: Fn(A) -> Apply!(brand: F, signature: ('a, B: 'a) -> 'a) + 'a,
		Apply!(brand: Self, signature: ('a, B: 'a) -> 'a): Clone,
	{
		let len = ta.len();
		ta.into_iter().fold(F::pure(Vec::with_capacity(len)), |acc, x| {
			F::lift2(
				|mut v, b| {
					v.push(b);
					v
				},
				acc,
				f(x),
			)
		})
	}

	/// Sequences a vector of applicative.
	///
	/// # Type Signature
	///
	/// `forall a f. (Traversable Vec, Applicative f) => (Vec (f a)) -> f (Vec a)`
	///
	/// # Parameters
	///
	/// * `ta`: The vector containing the applicative values.
	///
	/// # Returns
	///
	/// The vector wrapped in the applicative context.
	///
	/// # Examples
	///
	/// ```
	/// use fp_library::classes::traversable::sequence;
	/// use fp_library::brands::{OptionBrand, VecBrand};
	///
	/// assert_eq!(
	///     sequence::<VecBrand, OptionBrand, _>(vec![Some(1), Some(2)]),
	///     Some(vec![1, 2])
	/// );
	/// ```
	fn sequence<'a, F: Applicative, A: 'a + Clone>(
		ta: Apply!(brand: Self, signature: ('a, Apply!(brand: F, signature: ('a, A: 'a) -> 'a): 'a) -> 'a)
	) -> Apply!(brand: F, signature: ('a, Apply!(brand: Self, signature: ('a, A: 'a) -> 'a): 'a) -> 'a)
	where
		Apply!(brand: F, signature: ('a, A: 'a) -> 'a): Clone,
		Apply!(brand: Self, signature: ('a, A: 'a) -> 'a): Clone,
	{
		let len = ta.len();
		ta.into_iter().fold(F::pure(Vec::with_capacity(len)), |acc, x| {
			F::lift2(
				|mut v, a| {
					v.push(a);
					v
				},
				acc,
				x,
			)
		})
	}
}

impl<A: Clone> Semigroup for Vec<A> {
	/// Appends one vector to another.
	///
	/// # Type Signature
	///
	/// `forall a. Semigroup (Vec a) => (Vec a, Vec a) -> Vec a`
	///
	/// # Parameters
	///
	/// * `a`: The first vector.
	/// * `b`: The second vector.
	///
	/// # Returns
	///
	/// The concatenated vector.
	///
	/// # Examples
	///
	/// ```
	/// use fp_library::classes::semigroup::append;
	///
	/// assert_eq!(append(vec![1, 2], vec![3, 4]), vec![1, 2, 3, 4]);
	/// ```
	fn append(
		a: Self,
		b: Self,
	) -> Self {
		[a, b].concat()
	}
}

impl<A: Clone> Monoid for Vec<A> {
	/// Returns an empty vector.
	///
	/// # Type Signature
	///
	/// `forall a. Monoid (Vec a) => () -> Vec a`
	///
	/// # Returns
	///
	/// An empty vector.
	///
	/// # Examples
	///
	/// ```
	/// use fp_library::classes::monoid::empty;
	///
	/// assert_eq!(empty::<Vec<i32>>(), vec![]);
	/// ```
	fn empty() -> Self {
		Vec::new()
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		brands::RcFnBrand,
		classes::{
			functor::map, monoid::empty, pointed::pure, semiapplicative::apply, semigroup::append,
			semimonad::bind,
		},
		functions::{compose, identity},
	};
	use quickcheck_macros::quickcheck;

	// Functor Laws

	/// Tests the identity law for Functor.
	#[quickcheck]
	fn functor_identity(x: Vec<i32>) -> bool {
		map::<VecBrand, _, _, _>(identity, x.clone()) == x
	}

	/// Tests the composition law for Functor.
	#[quickcheck]
	fn functor_composition(x: Vec<i32>) -> bool {
		let f = |x: i32| x.wrapping_add(1);
		let g = |x: i32| x.wrapping_mul(2);
		map::<VecBrand, _, _, _>(compose(f, g), x.clone())
			== map::<VecBrand, _, _, _>(f, map::<VecBrand, _, _, _>(g, x))
	}

	// Applicative Laws

	/// Tests the identity law for Applicative.
	#[quickcheck]
	fn applicative_identity(v: Vec<i32>) -> bool {
		apply::<VecBrand, _, _, RcFnBrand>(
			pure::<VecBrand, _>(<RcFnBrand as ClonableFn>::new(identity)),
			v.clone(),
		) == v
	}

	/// Tests the homomorphism law for Applicative.
	#[quickcheck]
	fn applicative_homomorphism(x: i32) -> bool {
		let f = |x: i32| x.wrapping_mul(2);
		apply::<VecBrand, _, _, RcFnBrand>(
			pure::<VecBrand, _>(<RcFnBrand as ClonableFn>::new(f)),
			pure::<VecBrand, _>(x),
		) == pure::<VecBrand, _>(f(x))
	}

	/// Tests the composition law for Applicative.
	#[quickcheck]
	fn applicative_composition(
		w: Vec<i32>,
		u_seeds: Vec<i32>,
		v_seeds: Vec<i32>,
	) -> bool {
		let u_fns: Vec<_> = u_seeds
			.iter()
			.map(|&i| <RcFnBrand as ClonableFn>::new(move |x: i32| x.wrapping_add(i)))
			.collect();
		let v_fns: Vec<_> = v_seeds
			.iter()
			.map(|&i| <RcFnBrand as ClonableFn>::new(move |x: i32| x.wrapping_mul(i)))
			.collect();

		// RHS: u <*> (v <*> w)
		let vw = apply::<VecBrand, _, _, RcFnBrand>(v_fns.clone(), w.clone());
		let rhs = apply::<VecBrand, _, _, RcFnBrand>(u_fns.clone(), vw);

		// LHS: pure(compose) <*> u <*> v <*> w
		// equivalent to (u . v) <*> w
		// We construct (u . v) manually as the cartesian product of compositions
		let uv_fns: Vec<_> = u_fns
			.iter()
			.flat_map(|uf: &std::rc::Rc<dyn Fn(i32) -> i32>| {
				v_fns.iter().map(move |vf: &std::rc::Rc<dyn Fn(i32) -> i32>| {
					let uf = uf.clone();
					let vf = vf.clone();
					<RcFnBrand as ClonableFn>::new(move |x| uf(vf(x)))
				})
			})
			.collect();

		let lhs = apply::<VecBrand, _, _, RcFnBrand>(uv_fns, w);

		lhs == rhs
	}

	/// Tests the interchange law for Applicative.
	#[quickcheck]
	fn applicative_interchange(y: i32) -> bool {
		// u <*> pure y = pure ($ y) <*> u
		let f = |x: i32| x.wrapping_mul(2);
		let u = vec![<RcFnBrand as ClonableFn>::new(f)];

		let lhs = apply::<VecBrand, _, _, RcFnBrand>(u.clone(), pure::<VecBrand, _>(y));

		let rhs_fn = <RcFnBrand as ClonableFn>::new(move |f: std::rc::Rc<dyn Fn(i32) -> i32>| f(y));
		let rhs = apply::<VecBrand, _, _, RcFnBrand>(pure::<VecBrand, _>(rhs_fn), u);

		lhs == rhs
	}

	// Semigroup Laws

	/// Tests the associativity law for Semigroup.
	#[quickcheck]
	fn semigroup_associativity(
		a: Vec<i32>,
		b: Vec<i32>,
		c: Vec<i32>,
	) -> bool {
		append(a.clone(), append(b.clone(), c.clone())) == append(append(a, b), c)
	}

	// Monoid Laws

	/// Tests the left identity law for Monoid.
	#[quickcheck]
	fn monoid_left_identity(a: Vec<i32>) -> bool {
		append(empty::<Vec<i32>>(), a.clone()) == a
	}

	/// Tests the right identity law for Monoid.
	#[quickcheck]
	fn monoid_right_identity(a: Vec<i32>) -> bool {
		append(a.clone(), empty::<Vec<i32>>()) == a
	}

	// Monad Laws

	/// Tests the left identity law for Monad.
	#[quickcheck]
	fn monad_left_identity(a: i32) -> bool {
		let f = |x: i32| vec![x.wrapping_mul(2)];
		bind::<VecBrand, _, _, _>(pure::<VecBrand, _>(a), f) == f(a)
	}

	/// Tests the right identity law for Monad.
	#[quickcheck]
	fn monad_right_identity(m: Vec<i32>) -> bool {
		bind::<VecBrand, _, _, _>(m.clone(), pure::<VecBrand, _>) == m
	}

	/// Tests the associativity law for Monad.
	#[quickcheck]
	fn monad_associativity(m: Vec<i32>) -> bool {
		let f = |x: i32| vec![x.wrapping_mul(2)];
		let g = |x: i32| vec![x.wrapping_add(1)];
		bind::<VecBrand, _, _, _>(bind::<VecBrand, _, _, _>(m.clone(), f), g)
			== bind::<VecBrand, _, _, _>(m, |x| bind::<VecBrand, _, _, _>(f(x), g))
	}

	// Edge Cases

	/// Tests `map` on an empty vector.
	#[test]
	fn map_empty() {
		assert_eq!(
			map::<VecBrand, _, _, _>(|x: i32| x + 1, vec![] as Vec<i32>),
			vec![] as Vec<i32>
		);
	}

	/// Tests `bind` on an empty vector.
	#[test]
	fn bind_empty() {
		assert_eq!(
			bind::<VecBrand, _, _, _>(vec![] as Vec<i32>, |x: i32| vec![x + 1]),
			vec![] as Vec<i32>
		);
	}

	/// Tests `bind` returning an empty vector.
	#[test]
	fn bind_returning_empty() {
		assert_eq!(
			bind::<VecBrand, _, _, _>(vec![1, 2, 3], |_| vec![] as Vec<i32>),
			vec![] as Vec<i32>
		);
	}

	/// Tests `fold_right` on an empty vector.
	#[test]
	fn fold_right_empty() {
		assert_eq!(
			crate::classes::foldable::fold_right::<VecBrand, _, _, _>(
				|x: i32, acc| x + acc,
				0,
				vec![]
			),
			0
		);
	}

	/// Tests `fold_left` on an empty vector.
	#[test]
	fn fold_left_empty() {
		assert_eq!(
			crate::classes::foldable::fold_left::<VecBrand, _, _, _>(
				|acc, x: i32| acc + x,
				0,
				vec![]
			),
			0
		);
	}

	/// Tests `traverse` on an empty vector.
	#[test]
	fn traverse_empty() {
		use crate::brands::OptionBrand;
		assert_eq!(
			crate::classes::traversable::traverse::<VecBrand, OptionBrand, _, _, _>(
				|x: i32| Some(x + 1),
				vec![]
			),
			Some(vec![])
		);
	}

	/// Tests `traverse` returning an empty vector.
	#[test]
	fn traverse_returning_empty() {
		use crate::brands::OptionBrand;
		assert_eq!(
			crate::classes::traversable::traverse::<VecBrand, OptionBrand, _, _, _>(
				|_: i32| None::<i32>,
				vec![1, 2, 3]
			),
			None
		);
	}

	/// Tests `construct` with an empty tail.
	#[test]
	fn construct_empty_tail() {
		assert_eq!(VecBrand::construct(1, vec![]), vec![1]);
	}

	/// Tests `deconstruct` on an empty slice.
	#[test]
	fn deconstruct_empty() {
		assert_eq!(VecBrand::deconstruct::<i32>(&[]), None);
	}
}
