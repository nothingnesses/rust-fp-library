//! Implementations for [`Option`].
//!
//! This module provides implementations of functional programming traits for the standard library [`Option`] type.

use crate::{
	Apply,
	brands::OptionBrand,
	classes::{
		applicative::Applicative, apply_first::ApplyFirst, apply_second::ApplySecond,
		clonable_fn::ClonableFn, foldable::Foldable, functor::Functor, lift::Lift, monoid::Monoid,
		par_foldable::ParFoldable, pointed::Pointed, semiapplicative::Semiapplicative,
		semimonad::Semimonad, send_clonable_fn::SendClonableFn, traversable::Traversable,
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
	/// This method applies a function to the value inside the option, producing a new option with the transformed value. If the option is `None`, it returns `None`.
	///
	/// ### Type Signature
	///
	/// `forall a b. Functor Option => (a -> b, Option a) -> Option b`
	///
	/// ### Parameters
	///
	/// * `f`: The function to apply to the value.
	/// * `fa`: The option to map over.
	///
	/// ### Returns
	///
	/// A new option containing the result of applying the function, or `None`.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::classes::functor::Functor;
	/// use fp_library::brands::OptionBrand;
	///
	/// let x = Some(5);
	/// let y = OptionBrand::map(|i| i * 2, x);
	/// assert_eq!(y, Some(10));
	///
	/// // Using the free function
	/// use fp_library::classes::functor::map;
	/// assert_eq!(map::<OptionBrand, _, _, _>(|x: i32| x * 2, Some(5)), Some(10));
	/// assert_eq!(map::<OptionBrand, _, _, _>(|x: i32| x * 2, None), None);
	/// ```
	fn map<'a, F, A: 'a, B: 'a>(
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
	/// This method lifts a binary function to operate on values within the option context.
	///
	/// ### Type Signature
	///
	/// `forall a b c. Lift Option => ((a, b) -> c, Option a, Option b) -> Option c`
	///
	/// ### Parameters
	///
	/// * `f`: The binary function to apply.
	/// * `fa`: The first option.
	/// * `fb`: The second option.
	///
	/// ### Returns
	///
	/// `Some(f(a, b))` if both options are `Some`, otherwise `None`.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::classes::lift::Lift;
	/// use fp_library::brands::OptionBrand;
	///
	/// let x = Some(1);
	/// let y = Some(2);
	/// let z = OptionBrand::lift2(|a, b| a + b, x, y);
	/// assert_eq!(z, Some(3));
	///
	/// // Using the free function
	/// use fp_library::classes::lift::lift2;
	/// assert_eq!(lift2::<OptionBrand, _, _, _, _>(|x: i32, y: i32| x + y, Some(1), Some(2)), Some(3));
	/// assert_eq!(lift2::<OptionBrand, _, _, _, _>(|x: i32, y: i32| x + y, Some(1), None), None);
	/// ```
	fn lift2<'a, F, A, B, C>(
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
	/// This method wraps a value in an option context.
	///
	/// ### Type Signature
	///
	/// `forall a. Pointed Option => a -> Option a`
	///
	/// ### Parameters
	///
	/// * `a`: The value to wrap.
	///
	/// ### Returns
	///
	/// `Some(a)`.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::classes::pointed::Pointed;
	/// use fp_library::brands::OptionBrand;
	///
	/// let x = OptionBrand::pure(5);
	/// assert_eq!(x, Some(5));
	///
	/// // Using the free function
	/// use fp_library::classes::pointed::pure;
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
	/// This method applies a function wrapped in an option to a value wrapped in an option.
	///
	/// ### Type Signature
	///
	/// `forall a b. Semiapplicative Option => (Option (a -> b), Option a) -> Option b`
	///
	/// ### Parameters
	///
	/// * `ff`: The option containing the function.
	/// * `fa`: The option containing the value.
	///
	/// ### Returns
	///
	/// `Some(f(a))` if both are `Some`, otherwise `None`.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::classes::semiapplicative::Semiapplicative;
	/// use fp_library::classes::clonable_fn::ClonableFn;
	/// use fp_library::brands::{OptionBrand};
	/// use fp_library::brands::RcFnBrand;
	/// use std::rc::Rc;
	///
	/// let f = Some(<RcFnBrand as ClonableFn>::new(|x: i32| x * 2));
	/// let x = Some(5);
	/// let y = OptionBrand::apply::<RcFnBrand, i32, i32>(f, x);
	/// assert_eq!(y, Some(10));
	///
	/// // Using the free function
	/// use fp_library::classes::semiapplicative::apply;
	/// let f = Some(<RcFnBrand as ClonableFn>::new(|x: i32| x * 2));
	/// assert_eq!(apply::<RcFnBrand, OptionBrand, _, _>(f, Some(5)), Some(10));
	/// ```
	fn apply<'a, FnBrand: 'a + ClonableFn, A: 'a + Clone, B: 'a>(
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
	/// This method chains two option computations, where the second computation depends on the result of the first.
	///
	/// ### Type Signature
	///
	/// `forall a b. Semimonad Option => (Option a, a -> Option b) -> Option b`
	///
	/// ### Parameters
	///
	/// * `ma`: The first option.
	/// * `f`: The function to apply to the value inside the option.
	///
	/// ### Returns
	///
	/// The result of applying `f` to the value if `ma` is `Some`, otherwise `None`.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::classes::semimonad::Semimonad;
	/// use fp_library::brands::OptionBrand;
	///
	/// let x = Some(5);
	/// let y = OptionBrand::bind(x, |i| Some(i * 2));
	/// assert_eq!(y, Some(10));
	///
	/// // Using the free function
	/// use fp_library::classes::semimonad::bind;
	/// assert_eq!(bind::<OptionBrand, _, _, _>(Some(5), |x| Some(x * 2)), Some(10));
	/// assert_eq!(bind::<OptionBrand, _, _, _>(None, |x: i32| Some(x * 2)), None);
	/// ```
	fn bind<'a, F, A: 'a, B: 'a>(
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
	/// This method performs a right-associative fold of the option. If the option is `Some(a)`, it applies the function to `a` and the initial value. If `None`, it returns the initial value.
	///
	/// ### Type Signature
	///
	/// `forall a b. Foldable Option => ((a, b) -> b, b, Option a) -> b`
	///
	/// ### Parameters
	///
	/// * `func`: The folding function.
	/// * `initial`: The initial value.
	/// * `fa`: The option to fold.
	///
	/// ### Returns
	///
	/// `func(a, initial)` if `fa` is `Some(a)`, otherwise `initial`.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::classes::foldable::Foldable;
	/// use fp_library::brands::OptionBrand;
	/// use fp_library::brands::RcFnBrand;
	///
	/// let x = Some(5);
	/// let y = OptionBrand::fold_right::<RcFnBrand, _, _, _>(|a, b| a + b, 10, x);
	/// assert_eq!(y, 15);
	///
	/// // Using the free function
	/// use fp_library::classes::foldable::fold_right;
	/// assert_eq!(fold_right::<RcFnBrand, OptionBrand, _, _, _>(|x: i32, acc| x + acc, 0, Some(5)), 5);
	/// assert_eq!(fold_right::<RcFnBrand, OptionBrand, _, _, _>(|x: i32, acc| x + acc, 0, None), 0);
	/// ```
	fn fold_right<'a, FnBrand, Func, A: 'a, B: 'a>(
		func: Func,
		initial: B,
		fa: Apply!(brand: Self, signature: ('a, A: 'a) -> 'a),
	) -> B
	where
		Func: Fn(A, B) -> B + 'a,
		FnBrand: ClonableFn + 'a,
	{
		match fa {
			Some(a) => func(a, initial),
			None => initial,
		}
	}

	/// Folds the option from the left.
	///
	/// This method performs a left-associative fold of the option. If the option is `Some(a)`, it applies the function to the initial value and `a`. If `None`, it returns the initial value.
	///
	/// ### Type Signature
	///
	/// `forall a b. Foldable Option => ((b, a) -> b, b, Option a) -> b`
	///
	/// ### Parameters
	///
	/// * `func`: The function to apply to the accumulator and each element.
	/// * `initial`: The initial value of the accumulator.
	/// * `fa`: The option to fold.
	///
	/// ### Returns
	///
	/// `f(initial, a)` if `fa` is `Some(a)`, otherwise `initial`.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::classes::foldable::Foldable;
	/// use fp_library::brands::OptionBrand;
	/// use fp_library::brands::RcFnBrand;
	///
	/// let x = Some(5);
	/// let y = OptionBrand::fold_left::<RcFnBrand, _, _, _>(|b, a| b + a, 10, x);
	/// assert_eq!(y, 15);
	///
	/// // Using the free function
	/// use fp_library::classes::foldable::fold_left;
	/// assert_eq!(fold_left::<RcFnBrand, OptionBrand, _, _, _>(|acc, x: i32| acc + x, 0, Some(5)), 5);
	/// ```
	fn fold_left<'a, FnBrand, Func, A: 'a, B: 'a>(
		func: Func,
		initial: B,
		fa: Apply!(brand: Self, signature: ('a, A: 'a) -> 'a),
	) -> B
	where
		Func: Fn(B, A) -> B + 'a,
		FnBrand: ClonableFn + 'a,
	{
		match fa {
			Some(a) => func(initial, a),
			None => initial,
		}
	}

	/// Maps the value to a monoid and returns it, or returns empty.
	///
	/// This method maps the element of the option to a monoid. If the option is `None`, it returns the monoid's identity element.
	///
	/// ### Type Signature
	///
	/// `forall a m. (Foldable Option, Monoid m) => ((a) -> m, Option a) -> m`
	///
	/// ### Parameters
	///
	/// * `func`: The mapping function.
	/// * `fa`: The option to fold.
	///
	/// ### Returns
	///
	/// `func(a)` if `fa` is `Some(a)`, otherwise `M::empty()`.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::classes::foldable::Foldable;
	/// use fp_library::brands::OptionBrand;
	/// use fp_library::types::string; // Import to bring Monoid impl for String into scope
	/// use fp_library::brands::RcFnBrand;
	///
	/// let x = Some(5);
	/// let y = OptionBrand::fold_map::<RcFnBrand, _, _, _>(|a: i32| a.to_string(), x);
	/// assert_eq!(y, "5".to_string());
	///
	/// // Using the free function
	/// use fp_library::classes::foldable::fold_map;
	/// assert_eq!(fold_map::<RcFnBrand, OptionBrand, _, _, _>(|x: i32| x.to_string(), Some(5)), "5".to_string());
	/// ```
	fn fold_map<'a, FnBrand, Func, A: 'a, M>(
		func: Func,
		fa: Apply!(brand: Self, signature: ('a, A: 'a) -> 'a),
	) -> M
	where
		M: Monoid + 'a,
		Func: Fn(A) -> M + 'a,
		FnBrand: ClonableFn + 'a,
	{
		match fa {
			Some(a) => func(a),
			None => M::empty(),
		}
	}
}

impl Traversable for OptionBrand {
	/// Traverses the option with an applicative function.
	///
	/// This method maps the element of the option to a computation, evaluates it, and wraps the result in the applicative context. If `None`, it returns `pure(None)`.
	///
	/// ### Type Signature
	///
	/// `forall a b f. (Traversable Option, Applicative f) => (a -> f b, Option a) -> f (Option b)`
	///
	/// ### Parameters
	///
	/// * `func`: The function to apply to each element, returning a value in an applicative context.
	/// * `ta`: The option to traverse.
	///
	/// ### Returns
	///
	/// The option wrapped in the applicative context.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::classes::traversable::Traversable;
	/// use fp_library::brands::OptionBrand;
	///
	/// let x = Some(5);
	/// let y = OptionBrand::traverse::<OptionBrand, _, _, _>(|a| Some(a * 2), x);
	/// assert_eq!(y, Some(Some(10)));
	///
	/// // Using the free function
	/// use fp_library::classes::traversable::traverse;
	/// assert_eq!(traverse::<OptionBrand, OptionBrand, _, _, _>(|x| Some(x * 2), Some(5)), Some(Some(10)));
	/// ```
	fn traverse<'a, F: Applicative, Func, A: 'a + Clone, B: 'a + Clone>(
		func: Func,
		ta: Apply!(brand: Self, signature: ('a, A: 'a) -> 'a),
	) -> Apply!(brand: F, signature: ('a, Apply!(brand: Self, signature: ('a, B: 'a) -> 'a): 'a) -> 'a)
	where
		Func: Fn(A) -> Apply!(brand: F, signature: ('a, B: 'a) -> 'a) + 'a,
		Apply!(brand: Self, signature: ('a, B: 'a) -> 'a): Clone,
	{
		match ta {
			Some(a) => F::map(|b| Some(b), func(a)),
			None => F::pure(None),
		}
	}
	/// Sequences an option of applicative.
	///
	/// This method evaluates the computation inside the option and wraps the result in the applicative context. If `None`, it returns `pure(None)`.
	///
	/// ### Type Signature
	///
	/// `forall a f. (Traversable Option, Applicative f) => (Option (f a)) -> f (Option a)`
	///
	/// ### Parameters
	///
	/// * `ta`: The option containing the applicative value.
	///
	/// # Returns
	///
	/// The option wrapped in the applicative context.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::classes::traversable::Traversable;
	/// use fp_library::brands::OptionBrand;
	///
	/// let x = Some(Some(5));
	/// let y = OptionBrand::sequence::<OptionBrand, _>(x);
	/// assert_eq!(y, Some(Some(5)));
	///
	/// // Using the free function
	/// use fp_library::classes::traversable::sequence;
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

impl<FnBrand: SendClonableFn> ParFoldable<FnBrand> for OptionBrand {
	/// Maps the value to a monoid and returns it, or returns empty, in parallel.
	///
	/// This method maps the element of the option to a monoid. Since `Option` contains at most one element, no actual parallelism occurs, but the interface is satisfied.
	///
	/// ### Type Signature
	///
	/// `forall a m. (ParFoldable Option, Monoid m, Send m, Sync m) => (f a m, Option a) -> m`
	///
	/// ### Parameters
	///
	/// * `func`: The mapping function.
	/// * `fa`: The option to fold.
	///
	/// ### Returns
	///
	/// The combined monoid value.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::classes::par_foldable::ParFoldable;
	/// use fp_library::brands::{OptionBrand, ArcFnBrand};
	/// use fp_library::classes::send_clonable_fn::SendClonableFn;
	/// use fp_library::classes::send_clonable_fn::new_send;
	///
	/// let x = Some(1);
	/// let f = new_send::<ArcFnBrand, _, _>(|x: i32| x.to_string());
	/// let y = <OptionBrand as ParFoldable<ArcFnBrand>>::par_fold_map(f, x);
	/// assert_eq!(y, "1".to_string());
	///
	/// // Using the free function
	/// use fp_library::classes::par_foldable::par_fold_map;
	/// let x = Some(1);
	/// let f = new_send::<ArcFnBrand, _, _>(|x: i32| x.to_string());
	/// assert_eq!(par_fold_map::<ArcFnBrand, OptionBrand, _, _>(f, x), "1".to_string());
	/// ```
	fn par_fold_map<'a, A, M>(
		func: Apply!(brand: FnBrand, kind: SendClonableFn, output: SendOf, lifetimes: ('a), types: (A, M)),
		fa: Apply!(brand: Self, signature: ('a, A: 'a) -> 'a),
	) -> M
	where
		A: 'a + Clone + Send + Sync,
		M: Monoid + Send + Sync + 'a,
	{
		match fa {
			Some(a) => func(a),
			None => M::empty(),
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		brands::{ArcFnBrand, RcFnBrand},
		classes::{
			functor::map,
			par_foldable::{par_fold_map, par_fold_right},
			pointed::pure,
			semiapplicative::apply,
			semimonad::bind,
			send_clonable_fn::new_send,
		},
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
		apply::<RcFnBrand, OptionBrand, _, _>(
			pure::<OptionBrand, _>(<RcFnBrand as ClonableFn>::new(identity)),
			v,
		) == v
	}

	/// Tests the homomorphism law for Applicative.
	#[quickcheck]
	fn applicative_homomorphism(x: i32) -> bool {
		let f = |x: i32| x.wrapping_mul(2);
		apply::<RcFnBrand, OptionBrand, _, _>(
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
		let vw = apply::<RcFnBrand, OptionBrand, _, _>(v.clone(), w.clone());
		let rhs = apply::<RcFnBrand, OptionBrand, _, _>(u.clone(), vw);

		// LHS: pure(compose) <*> u <*> v <*> w
		// equivalent to (u . v) <*> w
		let uv = match (u, v) {
			(Some(uf), Some(vf)) => {
				let composed = move |x| uf(vf(x));
				Some(<RcFnBrand as ClonableFn>::new(composed))
			}
			_ => None,
		};

		let lhs = apply::<RcFnBrand, OptionBrand, _, _>(uv, w);

		lhs == rhs
	}

	/// Tests the interchange law for Applicative.
	#[quickcheck]
	fn applicative_interchange(y: i32) -> bool {
		// u <*> pure y = pure ($ y) <*> u
		let f = |x: i32| x.wrapping_mul(2);
		let u = pure::<OptionBrand, _>(<RcFnBrand as ClonableFn>::new(f));

		let lhs = apply::<RcFnBrand, OptionBrand, _, _>(u.clone(), pure::<OptionBrand, _>(y));

		let rhs_fn = <RcFnBrand as ClonableFn>::new(move |f: std::rc::Rc<dyn Fn(i32) -> i32>| f(y));
		let rhs = apply::<RcFnBrand, OptionBrand, _, _>(pure::<OptionBrand, _>(rhs_fn), u);

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
			crate::classes::foldable::fold_right::<RcFnBrand, OptionBrand, _, _, _>(
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
			crate::classes::foldable::fold_left::<RcFnBrand, OptionBrand, _, _, _>(
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

	// ParFoldable Tests

	/// Tests `par_fold_map` on `None`.
	#[test]
	fn par_fold_map_none() {
		let x: Option<i32> = None;
		let f = new_send::<ArcFnBrand, _, _>(|x: i32| x.to_string());
		assert_eq!(par_fold_map::<ArcFnBrand, OptionBrand, _, _>(f, x), "".to_string());
	}

	/// Tests `par_fold_map` on `Some`.
	#[test]
	fn par_fold_map_some() {
		let x = Some(5);
		let f = new_send::<ArcFnBrand, _, _>(|x: i32| x.to_string());
		assert_eq!(par_fold_map::<ArcFnBrand, OptionBrand, _, _>(f, x), "5".to_string());
	}

	/// Tests `par_fold_right` on `Some`.
	#[test]
	fn par_fold_right_some() {
		let x = Some(5);
		let f = new_send::<ArcFnBrand, _, _>(|(a, b): (i32, i32)| a + b);
		assert_eq!(par_fold_right::<ArcFnBrand, OptionBrand, _, _>(f, 10, x), 15);
	}
}
