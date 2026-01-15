//! Implementations for [`Option`].

use crate::{
	Apply,
	brands::OptionBrand,
	classes::{
		applicative::Applicative, apply_first::ApplyFirst, apply_second::ApplySecond,
		clonable_fn::ClonableFn, foldable::Foldable, functor::Functor, lift::Lift, monoid::Monoid,
		pointed::Pointed, semiapplicative::Semiapplicative, semimonad::Semimonad,
		traversable::Traversable,
	},
	impl_kind,
	kinds::*,
};

impl_kind! {
	for OptionBrand {
		type Of<'a, A: 'a>: 'a = Option<A>;
	}
}

impl Functor for OptionBrand {
	/// Maps a function over the value in the option.
	///
	/// # Type Signature
	///
	/// `forall a b. Functor Option => (a -> b, Option a) -> Option b`
	///
	/// # Parameters
	///
	/// * `f`: The function to apply to the value.
	/// * `fa`: The option to map over.
	///
	/// # Returns
	///
	/// A new option containing the result of applying the function, or `None`.
	///
	/// # Examples
	///
	/// ```
	/// use fp_library::classes::functor::map;
	/// use fp_library::brands::OptionBrand;
	///
	/// assert_eq!(map::<OptionBrand, _, _, _>(|x: i32| x * 2, Some(5)), Some(10));
	/// assert_eq!(map::<OptionBrand, _, _, _>(|x: i32| x * 2, None), None);
	/// ```
	fn map<'a, A: 'a, B: 'a, F>(
		f: F,
		fa: Apply!(
			brand: Self,
			signature: ('a, A: 'a) -> 'a,
		),
	) -> Apply!(
		brand: Self,
		signature: ('a, B: 'a) -> 'a,
	)
	where
		F: Fn(A) -> B + 'a,
	{
		fa.map(f)
	}
}

impl Lift for OptionBrand {
	/// Lifts a binary function into the option context.
	///
	/// # Type Signature
	///
	/// `forall a b c. Lift Option => ((a, b) -> c, Option a, Option b) -> Option c`
	///
	/// # Parameters
	///
	/// * `f`: The binary function to apply.
	/// * `fa`: The first option.
	/// * `fb`: The second option.
	///
	/// # Returns
	///
	/// `Some(f(a, b))` if both options are `Some`, otherwise `None`.
	///
	/// # Examples
	///
	/// ```
	/// use fp_library::classes::lift::lift2;
	/// use fp_library::brands::OptionBrand;
	///
	/// assert_eq!(lift2::<OptionBrand, _, _, _, _>(|x: i32, y: i32| x + y, Some(1), Some(2)), Some(3));
	/// assert_eq!(lift2::<OptionBrand, _, _, _, _>(|x: i32, y: i32| x + y, Some(1), None), None);
	/// ```
	fn lift2<'a, A, B, C, F>(
		f: F,
		fa: Apply!(brand: Self, signature: ('a, A: 'a) -> 'a),
		fb: Apply!(brand: Self, signature: ('a, B: 'a) -> 'a),
	) -> Apply!(brand: Self, signature: ('a, C: 'a) -> 'a)
	where
		F: Fn(A, B) -> C + 'a,
		A: 'a,
		B: 'a,
		C: 'a,
	{
		fa.zip(fb).map(|(a, b)| f(a, b))
	}
}

impl Pointed for OptionBrand {
	/// Wraps a value in an option.
	///
	/// # Type Signature
	///
	/// `forall a. Pointed Option => a -> Option a`
	///
	/// # Parameters
	///
	/// * `a`: The value to wrap.
	///
	/// # Returns
	///
	/// `Some(a)`.
	///
	/// # Examples
	///
	/// ```
	/// use fp_library::classes::pointed::pure;
	/// use fp_library::brands::OptionBrand;
	///
	/// assert_eq!(pure::<OptionBrand, _>(5), Some(5));
	/// ```
	fn pure<'a, A: 'a>(a: A) -> Apply!(brand: Self, signature: ('a, A: 'a) -> 'a) {
		Some(a)
	}
}

impl ApplyFirst for OptionBrand {}
impl ApplySecond for OptionBrand {}

impl Semiapplicative for OptionBrand {
	/// Applies a wrapped function to a wrapped value.
	///
	/// # Type Signature
	///
	/// `forall a b. Semiapplicative Option => (Option (a -> b), Option a) -> Option b`
	///
	/// # Parameters
	///
	/// * `ff`: The option containing the function.
	/// * `fa`: The option containing the value.
	///
	/// # Returns
	///
	/// `Some(f(a))` if both are `Some`, otherwise `None`.
	///
	/// # Examples
	///
	/// ```
	/// use fp_library::classes::semiapplicative::apply;
	/// use fp_library::classes::clonable_fn::ClonableFn;
	/// use fp_library::brands::{OptionBrand};
	/// use fp_library::brands::RcFnBrand;
	/// use std::rc::Rc;
	///
	/// let f = Some(<RcFnBrand as ClonableFn>::new(|x: i32| x * 2));
	/// assert_eq!(apply::<OptionBrand, _, _, RcFnBrand>(f, Some(5)), Some(10));
	/// ```
	fn apply<'a, A: 'a + Clone, B: 'a, FnBrand: 'a + ClonableFn>(
		ff: Apply!(brand: Self, signature: ('a, Apply!(brand: FnBrand, kind: ClonableFn, lifetimes: ('a), types: (A, B)): 'a) -> 'a),
		fa: Apply!(brand: Self, signature: ('a, A: 'a) -> 'a),
	) -> Apply!(brand: Self, signature: ('a, B: 'a) -> 'a) {
		match (ff, fa) {
			(Some(f), Some(a)) => Some(f(a)),
			_ => None,
		}
	}
}

impl Semimonad for OptionBrand {
	/// Chains option computations.
	///
	/// # Type Signature
	///
	/// `forall a b. Semimonad Option => (Option a, a -> Option b) -> Option b`
	///
	/// # Parameters
	///
	/// * `ma`: The first option.
	/// * `f`: The function to apply to the value inside the option.
	///
	/// # Returns
	///
	/// The result of applying `f` to the value if `ma` is `Some`, otherwise `None`.
	///
	/// # Examples
	///
	/// ```
	/// use fp_library::classes::semimonad::bind;
	/// use fp_library::brands::OptionBrand;
	///
	/// assert_eq!(bind::<OptionBrand, _, _, _>(Some(5), |x| Some(x * 2)), Some(10));
	/// assert_eq!(bind::<OptionBrand, _, _, _>(None, |x: i32| Some(x * 2)), None);
	/// ```
	fn bind<'a, A: 'a, B: 'a, F>(
		ma: Apply!(brand: Self, signature: ('a, A: 'a) -> 'a),
		f: F,
	) -> Apply!(brand: Self, signature: ('a, B: 'a) -> 'a)
	where
		F: Fn(A) -> Apply!(brand: Self, signature: ('a, B: 'a) -> 'a) + 'a,
	{
		ma.and_then(f)
	}
}

impl Foldable for OptionBrand {
	/// Folds the option from the right.
	///
	/// # Type Signature
	///
	/// `forall a b. Foldable Option => ((a, b) -> b, b, Option a) -> b`
	///
	/// # Parameters
	///
	/// * `f`: The folding function.
	/// * `init`: The initial value.
	/// * `fa`: The option to fold.
	///
	/// # Returns
	///
	/// `f(a, init)` if `fa` is `Some(a)`, otherwise `init`.
	///
	/// # Examples
	///
	/// ```
	/// use fp_library::classes::foldable::fold_right;
	/// use fp_library::brands::OptionBrand;
	///
	/// assert_eq!(fold_right::<OptionBrand, _, _, _>(|x: i32, acc| x + acc, 0, Some(5)), 5);
	/// assert_eq!(fold_right::<OptionBrand, _, _, _>(|x: i32, acc| x + acc, 0, None), 0);
	/// ```
	fn fold_right<'a, A: 'a, B: 'a, F>(
		f: F,
		init: B,
		fa: Apply!(brand: Self, signature: ('a, A: 'a) -> 'a),
	) -> B
	where
		F: Fn(A, B) -> B + 'a,
	{
		match fa {
			Some(a) => f(a, init),
			None => init,
		}
	}

	/// Folds the option from the left.
	///
	/// # Type Signature
	///
	/// `forall a b. Foldable Option => ((b, a) -> b, b, Option a) -> b`
	///
	/// # Parameters
	///
	/// * `f`: The folding function.
	/// * `init`: The initial value.
	/// * `fa`: The option to fold.
	///
	/// # Returns
	///
	/// `f(init, a)` if `fa` is `Some(a)`, otherwise `init`.
	///
	/// # Examples
	///
	/// ```
	/// use fp_library::classes::foldable::fold_left;
	/// use fp_library::brands::OptionBrand;
	///
	/// assert_eq!(fold_left::<OptionBrand, _, _, _>(|acc, x: i32| acc + x, 0, Some(5)), 5);
	/// ```
	fn fold_left<'a, A: 'a, B: 'a, F>(
		f: F,
		init: B,
		fa: Apply!(brand: Self, signature: ('a, A: 'a) -> 'a),
	) -> B
	where
		F: Fn(B, A) -> B + 'a,
	{
		match fa {
			Some(a) => f(init, a),
			None => init,
		}
	}

	/// Maps the value to a monoid and returns it, or returns empty.
	///
	/// # Type Signature
	///
	/// `forall a m. (Foldable Option, Monoid m) => ((a) -> m, Option a) -> m`
	///
	/// # Parameters
	///
	/// * `f`: The mapping function.
	/// * `fa`: The option to fold.
	///
	/// # Returns
	///
	/// `f(a)` if `fa` is `Some(a)`, otherwise `M::empty()`.
	///
	/// # Examples
	///
	/// ```
	/// use fp_library::classes::foldable::fold_map;
	/// use fp_library::brands::OptionBrand;
	/// use fp_library::types::string; // Import to bring Monoid impl for String into scope
	///
	/// assert_eq!(fold_map::<OptionBrand, _, _, _>(|x: i32| x.to_string(), Some(5)), "5".to_string());
	/// ```
	fn fold_map<'a, A: 'a, M, F>(
		f: F,
		fa: Apply!(brand: Self, signature: ('a, A: 'a) -> 'a),
	) -> M
	where
		M: Monoid + 'a,
		F: Fn(A) -> M + 'a,
	{
		match fa {
			Some(a) => f(a),
			None => M::empty(),
		}
	}
}

impl Traversable for OptionBrand {
	/// Traverses the option with an applicative function.
	///
	/// # Type Signature
	///
	/// `forall a b f. (Traversable Option, Applicative f) => (a -> f b, Option a) -> f (Option b)`
	///
	/// # Parameters
	///
	/// * `f`: The function to apply.
	/// * `ta`: The option to traverse.
	///
	/// # Returns
	///
	/// The option wrapped in the applicative context.
	///
	/// # Examples
	///
	/// ```
	/// use fp_library::classes::traversable::traverse;
	/// use fp_library::brands::OptionBrand;
	///
	/// assert_eq!(traverse::<OptionBrand, OptionBrand, _, _, _>(|x| Some(x * 2), Some(5)), Some(Some(10)));
	/// ```
	fn traverse<'a, F: Applicative, A: 'a + Clone, B: 'a + Clone, Func>(
		f: Func,
		ta: Apply!(brand: Self, signature: ('a, A: 'a) -> 'a),
	) -> Apply!(brand: F, signature: ('a, Apply!(brand: Self, signature: ('a, B: 'a) -> 'a): 'a) -> 'a)
	where
		Func: Fn(A) -> Apply!(brand: F, signature: ('a, B: 'a) -> 'a) + 'a,
		Apply!(brand: Self, signature: ('a, B: 'a) -> 'a): Clone,
	{
		match ta {
			Some(a) => F::map(|b| Some(b), f(a)),
			None => F::pure(None),
		}
	}

	/// Sequences an option of applicative.
	///
	/// # Type Signature
	///
	/// `forall a f. (Traversable Option, Applicative f) => (Option (f a)) -> f (Option a)`
	///
	/// # Parameters
	///
	/// * `ta`: The option containing the applicative value.
	///
	/// # Returns
	///
	/// The option wrapped in the applicative context.
	///
	/// # Examples
	///
	/// ```
	/// use fp_library::classes::traversable::sequence;
	/// use fp_library::brands::OptionBrand;
	///
	/// assert_eq!(sequence::<OptionBrand, OptionBrand, _>(Some(Some(5))), Some(Some(5)));
	/// ```
	fn sequence<'a, F: Applicative, A: 'a + Clone>(
		ta: Apply!(brand: Self, signature: ('a, Apply!(brand: F, signature: ('a, A: 'a) -> 'a): 'a) -> 'a)
	) -> Apply!(brand: F, signature: ('a, Apply!(brand: Self, signature: ('a, A: 'a) -> 'a): 'a) -> 'a)
	where
		Apply!(brand: F, signature: ('a, A: 'a) -> 'a): Clone,
		Apply!(brand: Self, signature: ('a, A: 'a) -> 'a): Clone,
	{
		match ta {
			Some(fa) => F::map(|a| Some(a), fa),
			None => F::pure(None),
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		brands::RcFnBrand,
		classes::{functor::map, pointed::pure, semiapplicative::apply, semimonad::bind},
		functions::{compose, identity},
	};
	use quickcheck_macros::quickcheck;

	// Functor Laws

	/// Tests the identity law for Functor.
	#[quickcheck]
	fn functor_identity(x: Option<i32>) -> bool {
		map::<OptionBrand, _, _, _>(identity, x) == x
	}

	/// Tests the composition law for Functor.
	#[quickcheck]
	fn functor_composition(x: Option<i32>) -> bool {
		let f = |x: i32| x.wrapping_add(1);
		let g = |x: i32| x.wrapping_mul(2);
		map::<OptionBrand, _, _, _>(compose(f, g), x)
			== map::<OptionBrand, _, _, _>(f, map::<OptionBrand, _, _, _>(g, x))
	}

	// Applicative Laws

	/// Tests the identity law for Applicative.
	#[quickcheck]
	fn applicative_identity(v: Option<i32>) -> bool {
		apply::<OptionBrand, _, _, RcFnBrand>(
			pure::<OptionBrand, _>(<RcFnBrand as ClonableFn>::new(identity)),
			v,
		) == v
	}

	/// Tests the homomorphism law for Applicative.
	#[quickcheck]
	fn applicative_homomorphism(x: i32) -> bool {
		let f = |x: i32| x.wrapping_mul(2);
		apply::<OptionBrand, _, _, RcFnBrand>(
			pure::<OptionBrand, _>(<RcFnBrand as ClonableFn>::new(f)),
			pure::<OptionBrand, _>(x),
		) == pure::<OptionBrand, _>(f(x))
	}

	/// Tests the composition law for Applicative.
	#[quickcheck]
	fn applicative_composition(
		w: Option<i32>,
		u_is_some: bool,
		v_is_some: bool,
	) -> bool {
		let v_fn = |x: i32| x.wrapping_mul(2);
		let u_fn = |x: i32| x.wrapping_add(1);

		let v = if v_is_some {
			pure::<OptionBrand, _>(<RcFnBrand as ClonableFn>::new(v_fn))
		} else {
			None
		};
		let u = if u_is_some {
			pure::<OptionBrand, _>(<RcFnBrand as ClonableFn>::new(u_fn))
		} else {
			None
		};

		// RHS: u <*> (v <*> w)
		let vw = apply::<OptionBrand, _, _, RcFnBrand>(v.clone(), w.clone());
		let rhs = apply::<OptionBrand, _, _, RcFnBrand>(u.clone(), vw);

		// LHS: pure(compose) <*> u <*> v <*> w
		// equivalent to (u . v) <*> w
		let uv = match (u, v) {
			(Some(uf), Some(vf)) => {
				let composed = move |x| uf(vf(x));
				Some(<RcFnBrand as ClonableFn>::new(composed))
			}
			_ => None,
		};

		let lhs = apply::<OptionBrand, _, _, RcFnBrand>(uv, w);

		lhs == rhs
	}

	/// Tests the interchange law for Applicative.
	#[quickcheck]
	fn applicative_interchange(y: i32) -> bool {
		// u <*> pure y = pure ($ y) <*> u
		let f = |x: i32| x.wrapping_mul(2);
		let u = pure::<OptionBrand, _>(<RcFnBrand as ClonableFn>::new(f));

		let lhs = apply::<OptionBrand, _, _, RcFnBrand>(u.clone(), pure::<OptionBrand, _>(y));

		let rhs_fn = <RcFnBrand as ClonableFn>::new(move |f: std::rc::Rc<dyn Fn(i32) -> i32>| f(y));
		let rhs = apply::<OptionBrand, _, _, RcFnBrand>(pure::<OptionBrand, _>(rhs_fn), u);

		lhs == rhs
	}

	// Monad Laws

	/// Tests the left identity law for Monad.
	#[quickcheck]
	fn monad_left_identity(a: i32) -> bool {
		let f = |x: i32| Some(x.wrapping_mul(2));
		bind::<OptionBrand, _, _, _>(pure::<OptionBrand, _>(a), f) == f(a)
	}

	/// Tests the right identity law for Monad.
	#[quickcheck]
	fn monad_right_identity(m: Option<i32>) -> bool {
		bind::<OptionBrand, _, _, _>(m, pure::<OptionBrand, _>) == m
	}

	/// Tests the associativity law for Monad.
	#[quickcheck]
	fn monad_associativity(m: Option<i32>) -> bool {
		let f = |x: i32| Some(x.wrapping_mul(2));
		let g = |x: i32| Some(x.wrapping_add(1));
		bind::<OptionBrand, _, _, _>(bind::<OptionBrand, _, _, _>(m, f), g)
			== bind::<OptionBrand, _, _, _>(m, |x| bind::<OptionBrand, _, _, _>(f(x), g))
	}

	// Edge Cases

	/// Tests `map` on `None`.
	#[test]
	fn map_none() {
		assert_eq!(map::<OptionBrand, _, _, _>(|x: i32| x + 1, None), None);
	}

	/// Tests `bind` on `None`.
	#[test]
	fn bind_none() {
		assert_eq!(bind::<OptionBrand, _, _, _>(None, |x: i32| Some(x + 1)), None);
	}

	/// Tests `bind` returning `None`.
	#[test]
	fn bind_returning_none() {
		assert_eq!(bind::<OptionBrand, _, _, _>(Some(5), |_| None::<i32>), None);
	}

	/// Tests `fold_right` on `None`.
	#[test]
	fn fold_right_none() {
		assert_eq!(
			crate::classes::foldable::fold_right::<OptionBrand, _, _, _>(
				|x: i32, acc| x + acc,
				0,
				None
			),
			0
		);
	}

	/// Tests `fold_left` on `None`.
	#[test]
	fn fold_left_none() {
		assert_eq!(
			crate::classes::foldable::fold_left::<OptionBrand, _, _, _>(
				|acc, x: i32| acc + x,
				0,
				None
			),
			0
		);
	}

	/// Tests `traverse` on `None`.
	#[test]
	fn traverse_none() {
		assert_eq!(
			crate::classes::traversable::traverse::<OptionBrand, OptionBrand, _, _, _>(
				|x: i32| Some(x + 1),
				None
			),
			Some(None)
		);
	}

	/// Tests `traverse` returning `None`.
	#[test]
	fn traverse_returning_none() {
		assert_eq!(
			crate::classes::traversable::traverse::<OptionBrand, OptionBrand, _, _, _>(
				|_: i32| None::<i32>,
				Some(5)
			),
			None
		);
	}
}
