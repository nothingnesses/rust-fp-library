//! Implementations for [`Identity`], a type that wraps a value.

use crate::{
	brands::IdentityBrand,
	classes::{
		applicative::Applicative,
		apply_first::ApplyFirst,
		apply_second::ApplySecond,
		clonable_fn::{ApplyClonableFn, ClonableFn},
		foldable::Foldable,
		functor::Functor,
		lift::Lift,
		monoid::Monoid,
		pointed::Pointed,
		semiapplicative::Semiapplicative,
		semimonad::Semimonad,
		traversable::Traversable,
	},
	hkt::{Apply1L1T, Kind1L1T},
};

/// Wraps a value.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Identity<A>(pub A);

impl Kind1L1T for IdentityBrand {
	type Output<'a, A: 'a> = Identity<A>;
}

impl Functor for IdentityBrand {
	/// Maps a function over the value in the identity.
	///
	/// # Type Signature
	///
	/// `forall a b. Functor Identity => (a -> b, Identity a) -> Identity b`
	///
	/// # Parameters
	///
	/// * `f`: The function to apply.
	/// * `fa`: The identity to map over.
	///
	/// # Returns
	///
	/// A new identity containing the result of applying the function.
	///
	/// # Examples
	///
	/// ```
	/// use fp_library::classes::functor::map;
	/// use fp_library::brands::IdentityBrand;
	/// use fp_library::types::Identity;
	///
	/// assert_eq!(map::<IdentityBrand, _, _, _>(|x: i32| x * 2, Identity(5)), Identity(10));
	/// ```
	fn map<'a, A: 'a, B: 'a, F>(
		f: F,
		fa: Apply1L1T<'a, Self, A>,
	) -> Apply1L1T<'a, Self, B>
	where
		F: Fn(A) -> B + 'a,
	{
		Identity(f(fa.0))
	}
}

impl Lift for IdentityBrand {
	/// Lifts a binary function into the identity context.
	///
	/// # Type Signature
	///
	/// `forall a b c. Lift Identity => ((a, b) -> c, Identity a, Identity b) -> Identity c`
	///
	/// # Parameters
	///
	/// * `f`: The binary function to apply.
	/// * `fa`: The first identity.
	/// * `fb`: The second identity.
	///
	/// # Returns
	///
	/// A new identity containing the result of applying the function.
	///
	/// # Examples
	///
	/// ```
	/// use fp_library::classes::lift::lift2;
	/// use fp_library::brands::IdentityBrand;
	/// use fp_library::types::Identity;
	///
	/// assert_eq!(
	///     lift2::<IdentityBrand, _, _, _, _>(|x: i32, y: i32| x + y, Identity(1), Identity(2)),
	///     Identity(3)
	/// );
	/// ```
	fn lift2<'a, A, B, C, F>(
		f: F,
		fa: Apply1L1T<'a, Self, A>,
		fb: Apply1L1T<'a, Self, B>,
	) -> Apply1L1T<'a, Self, C>
	where
		F: Fn(A, B) -> C + 'a,
		A: 'a,
		B: 'a,
		C: 'a,
	{
		Identity(f(fa.0, fb.0))
	}
}

impl Pointed for IdentityBrand {
	/// Wraps a value in an identity.
	///
	/// # Type Signature
	///
	/// `forall a. Pointed Identity => a -> Identity a`
	///
	/// # Parameters
	///
	/// * `a`: The value to wrap.
	///
	/// # Returns
	///
	/// An identity containing the value.
	///
	/// # Examples
	///
	/// ```
	/// use fp_library::classes::pointed::pure;
	/// use fp_library::brands::IdentityBrand;
	/// use fp_library::types::Identity;
	///
	/// assert_eq!(pure::<IdentityBrand, _>(5), Identity(5));
	/// ```
	fn pure<'a, A: 'a>(a: A) -> Apply1L1T<'a, Self, A> {
		Identity(a)
	}
}

impl ApplyFirst for IdentityBrand {}
impl ApplySecond for IdentityBrand {}

impl Semiapplicative for IdentityBrand {
	/// Applies a wrapped function to a wrapped value.
	///
	/// # Type Signature
	///
	/// `forall a b. Semiapplicative Identity => (Identity (a -> b), Identity a) -> Identity b`
	///
	/// # Parameters
	///
	/// * `ff`: The identity containing the function.
	/// * `fa`: The identity containing the value.
	///
	/// # Returns
	///
	/// A new identity containing the result of applying the function.
	///
	/// # Examples
	///
	/// ```
	/// use fp_library::classes::semiapplicative::apply;
	/// use fp_library::classes::clonable_fn::ClonableFn;
	/// use fp_library::brands::{IdentityBrand};
	/// use fp_library::types::Identity;
	/// use fp_library::brands::RcFnBrand;
	/// use std::rc::Rc;
	///
	/// let f = Identity(<RcFnBrand as ClonableFn>::new(|x: i32| x * 2));
	/// assert_eq!(apply::<IdentityBrand, _, _, RcFnBrand>(f, Identity(5)), Identity(10));
	/// ```
	fn apply<'a, A: 'a + Clone, B: 'a, FnBrand: 'a + ClonableFn>(
		ff: Apply1L1T<'a, Self, ApplyClonableFn<'a, FnBrand, A, B>>,
		fa: Apply1L1T<'a, Self, A>,
	) -> Apply1L1T<'a, Self, B> {
		Identity(ff.0(fa.0))
	}
}

impl Semimonad for IdentityBrand {
	/// Chains identity computations.
	///
	/// # Type Signature
	///
	/// `forall a b. Semimonad Identity => (Identity a, a -> Identity b) -> Identity b`
	///
	/// # Parameters
	///
	/// * `ma`: The first identity.
	/// * `f`: The function to apply to the value inside the identity.
	///
	/// # Returns
	///
	/// The result of applying `f` to the value.
	///
	/// # Examples
	///
	/// ```
	/// use fp_library::classes::semimonad::bind;
	/// use fp_library::brands::IdentityBrand;
	/// use fp_library::types::Identity;
	///
	/// assert_eq!(
	///     bind::<IdentityBrand, _, _, _>(Identity(5), |x| Identity(x * 2)),
	///     Identity(10)
	/// );
	/// ```
	fn bind<'a, A: 'a, B: 'a, F>(
		ma: Apply1L1T<'a, Self, A>,
		f: F,
	) -> Apply1L1T<'a, Self, B>
	where
		F: Fn(A) -> Apply1L1T<'a, Self, B> + 'a,
	{
		f(ma.0)
	}
}

impl Foldable for IdentityBrand {
	/// Folds the identity from the right.
	///
	/// # Type Signature
	///
	/// `forall a b. Foldable Identity => ((a, b) -> b, b, Identity a) -> b`
	///
	/// # Parameters
	///
	/// * `f`: The folding function.
	/// * `init`: The initial value.
	/// * `fa`: The identity to fold.
	///
	/// # Returns
	///
	/// `f(a, init)`.
	///
	/// # Examples
	///
	/// ```
	/// use fp_library::classes::foldable::fold_right;
	/// use fp_library::brands::IdentityBrand;
	/// use fp_library::types::Identity;
	///
	/// assert_eq!(fold_right::<IdentityBrand, _, _, _>(|x: i32, acc| x + acc, 0, Identity(5)), 5);
	/// ```
	fn fold_right<'a, A: 'a, B: 'a, F>(
		f: F,
		init: B,
		fa: Apply1L1T<'a, Self, A>,
	) -> B
	where
		F: Fn(A, B) -> B + 'a,
	{
		f(fa.0, init)
	}

	/// Folds the identity from the left.
	///
	/// # Type Signature
	///
	/// `forall a b. Foldable Identity => ((b, a) -> b, b, Identity a) -> b`
	///
	/// # Parameters
	///
	/// * `f`: The folding function.
	/// * `init`: The initial value.
	/// * `fa`: The identity to fold.
	///
	/// # Returns
	///
	/// `f(init, a)`.
	///
	/// # Examples
	///
	/// ```
	/// use fp_library::classes::foldable::fold_left;
	/// use fp_library::brands::IdentityBrand;
	/// use fp_library::types::Identity;
	///
	/// assert_eq!(fold_left::<IdentityBrand, _, _, _>(|acc, x: i32| acc + x, 0, Identity(5)), 5);
	/// ```
	fn fold_left<'a, A: 'a, B: 'a, F>(
		f: F,
		init: B,
		fa: Apply1L1T<'a, Self, A>,
	) -> B
	where
		F: Fn(B, A) -> B + 'a,
	{
		f(init, fa.0)
	}

	/// Maps the value to a monoid and returns it.
	///
	/// # Type Signature
	///
	/// `forall a m. (Foldable Identity, Monoid m) => ((a) -> m, Identity a) -> m`
	///
	/// # Parameters
	///
	/// * `f`: The mapping function.
	/// * `fa`: The identity to fold.
	///
	/// # Returns
	///
	/// `f(a)`.
	///
	/// # Examples
	///
	/// ```
	/// use fp_library::classes::foldable::fold_map;
	/// use fp_library::brands::IdentityBrand;
	/// use fp_library::types::Identity;
	/// use fp_library::types::string; // Import to bring Monoid impl for String into scope
	///
	/// assert_eq!(fold_map::<IdentityBrand, _, _, _>(|x: i32| x.to_string(), Identity(5)), "5".to_string());
	/// ```
	fn fold_map<'a, A: 'a, M, F>(
		f: F,
		fa: Apply1L1T<'a, Self, A>,
	) -> M
	where
		M: Monoid + 'a,
		F: Fn(A) -> M + 'a,
	{
		f(fa.0)
	}
}

impl Traversable for IdentityBrand {
	/// Traverses the identity with an applicative function.
	///
	/// # Type Signature
	///
	/// `forall a b f. (Traversable Identity, Applicative f) => (a -> f b, Identity a) -> f (Identity b)`
	///
	/// # Parameters
	///
	/// * `f`: The function to apply.
	/// * `ta`: The identity to traverse.
	///
	/// # Returns
	///
	/// The identity wrapped in the applicative context.
	///
	/// # Examples
	///
	/// ```
	/// use fp_library::classes::traversable::traverse;
	/// use fp_library::brands::{IdentityBrand, OptionBrand};
	/// use fp_library::types::Identity;
	///
	/// assert_eq!(
	///     traverse::<IdentityBrand, OptionBrand, _, _, _>(|x| Some(x * 2), Identity(5)),
	///     Some(Identity(10))
	/// );
	/// ```
	fn traverse<'a, F: Applicative, A: 'a + Clone, B: 'a + Clone, Func>(
		f: Func,
		ta: Apply1L1T<'a, Self, A>,
	) -> Apply1L1T<'a, F, Apply1L1T<'a, Self, B>>
	where
		Func: Fn(A) -> Apply1L1T<'a, F, B> + 'a,
		Apply1L1T<'a, Self, B>: Clone,
	{
		F::map(|b| Identity(b), f(ta.0))
	}

	/// Sequences an identity of applicative.
	///
	/// # Type Signature
	///
	/// `forall a f. (Traversable Identity, Applicative f) => (Identity (f a)) -> f (Identity a)`
	///
	/// # Parameters
	///
	/// * `ta`: The identity containing the applicative value.
	///
	/// # Returns
	///
	/// The identity wrapped in the applicative context.
	///
	/// # Examples
	///
	/// ```
	/// use fp_library::classes::traversable::sequence;
	/// use fp_library::brands::{IdentityBrand, OptionBrand};
	/// use fp_library::types::Identity;
	///
	/// assert_eq!(
	///     sequence::<IdentityBrand, OptionBrand, _>(Identity(Some(5))),
	///     Some(Identity(5))
	/// );
	/// ```
	fn sequence<'a, F: Applicative, A: 'a + Clone>(
		ta: Apply1L1T<'a, Self, Apply1L1T<'a, F, A>>
	) -> Apply1L1T<'a, F, Apply1L1T<'a, Self, A>>
	where
		Apply1L1T<'a, F, A>: Clone,
		Apply1L1T<'a, Self, A>: Clone,
	{
		F::map(|a| Identity(a), ta.0)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		brands::{OptionBrand, RcFnBrand},
		classes::{functor::map, pointed::pure, semiapplicative::apply, semimonad::bind},
		functions::{compose, identity},
	};
	use quickcheck_macros::quickcheck;

	// Functor Laws

	/// Tests the identity law for Functor.
	#[quickcheck]
	fn functor_identity(x: i32) -> bool {
		let x = Identity(x);
		map::<IdentityBrand, _, _, _>(identity, x) == x
	}

	/// Tests the composition law for Functor.
	#[quickcheck]
	fn functor_composition(x: i32) -> bool {
		let x = Identity(x);
		let f = |x: i32| x.wrapping_add(1);
		let g = |x: i32| x.wrapping_mul(2);
		map::<IdentityBrand, _, _, _>(compose(f, g), x)
			== map::<IdentityBrand, _, _, _>(f, map::<IdentityBrand, _, _, _>(g, x))
	}

	// Applicative Laws

	/// Tests the identity law for Applicative.
	#[quickcheck]
	fn applicative_identity(v: i32) -> bool {
		let v = Identity(v);
		apply::<IdentityBrand, _, _, RcFnBrand>(
			pure::<IdentityBrand, _>(<RcFnBrand as ClonableFn>::new(identity)),
			v,
		) == v
	}

	/// Tests the homomorphism law for Applicative.
	#[quickcheck]
	fn applicative_homomorphism(x: i32) -> bool {
		let f = |x: i32| x.wrapping_mul(2);
		apply::<IdentityBrand, _, _, RcFnBrand>(
			pure::<IdentityBrand, _>(<RcFnBrand as ClonableFn>::new(f)),
			pure::<IdentityBrand, _>(x),
		) == pure::<IdentityBrand, _>(f(x))
	}

	/// Tests the composition law for Applicative.
	#[quickcheck]
	fn applicative_composition(
		w: i32,
		u_val: i32,
		v_val: i32,
	) -> bool {
		let w = Identity(w);
		let v_fn = move |x: i32| x.wrapping_mul(v_val);
		let u_fn = move |x: i32| x.wrapping_add(u_val);

		let v = pure::<IdentityBrand, _>(<RcFnBrand as ClonableFn>::new(v_fn));
		let u = pure::<IdentityBrand, _>(<RcFnBrand as ClonableFn>::new(u_fn));

		// RHS: u <*> (v <*> w)
		let vw = apply::<IdentityBrand, _, _, RcFnBrand>(v.clone(), w.clone());
		let rhs = apply::<IdentityBrand, _, _, RcFnBrand>(u.clone(), vw);

		// LHS: pure(compose) <*> u <*> v <*> w
		// equivalent to (u . v) <*> w
		let composed = move |x| u_fn(v_fn(x));
		let uv = pure::<IdentityBrand, _>(<RcFnBrand as ClonableFn>::new(composed));

		let lhs = apply::<IdentityBrand, _, _, RcFnBrand>(uv, w);

		lhs == rhs
	}

	/// Tests the interchange law for Applicative.
	#[quickcheck]
	fn applicative_interchange(y: i32) -> bool {
		// u <*> pure y = pure ($ y) <*> u
		let f = |x: i32| x.wrapping_mul(2);
		let u = pure::<IdentityBrand, _>(<RcFnBrand as ClonableFn>::new(f));

		let lhs = apply::<IdentityBrand, _, _, RcFnBrand>(u.clone(), pure::<IdentityBrand, _>(y));

		let rhs_fn = <RcFnBrand as ClonableFn>::new(move |f: std::rc::Rc<dyn Fn(i32) -> i32>| f(y));
		let rhs = apply::<IdentityBrand, _, _, RcFnBrand>(pure::<IdentityBrand, _>(rhs_fn), u);

		lhs == rhs
	}

	// Monad Laws

	/// Tests the left identity law for Monad.
	#[quickcheck]
	fn monad_left_identity(a: i32) -> bool {
		let f = |x: i32| Identity(x.wrapping_mul(2));
		bind::<IdentityBrand, _, _, _>(pure::<IdentityBrand, _>(a), f) == f(a)
	}

	/// Tests the right identity law for Monad.
	#[quickcheck]
	fn monad_right_identity(m: i32) -> bool {
		let m = Identity(m);
		bind::<IdentityBrand, _, _, _>(m, pure::<IdentityBrand, _>) == m
	}

	/// Tests the associativity law for Monad.
	#[quickcheck]
	fn monad_associativity(m: i32) -> bool {
		let m = Identity(m);
		let f = |x: i32| Identity(x.wrapping_mul(2));
		let g = |x: i32| Identity(x.wrapping_add(1));
		bind::<IdentityBrand, _, _, _>(bind::<IdentityBrand, _, _, _>(m, f), g)
			== bind::<IdentityBrand, _, _, _>(m, |x| bind::<IdentityBrand, _, _, _>(f(x), g))
	}

	// Edge Cases

	/// Tests the `map` function.
	#[test]
	fn map_test() {
		assert_eq!(map::<IdentityBrand, _, _, _>(|x: i32| x + 1, Identity(1)), Identity(2));
	}

	/// Tests the `bind` function.
	#[test]
	fn bind_test() {
		assert_eq!(bind::<IdentityBrand, _, _, _>(Identity(1), |x| Identity(x + 1)), Identity(2));
	}

	/// Tests the `fold_right` function.
	#[test]
	fn fold_right_test() {
		assert_eq!(
			crate::classes::foldable::fold_right::<IdentityBrand, _, _, _>(
				|x: i32, acc| x + acc,
				0,
				Identity(1)
			),
			1
		);
	}

	/// Tests the `fold_left` function.
	#[test]
	fn fold_left_test() {
		assert_eq!(
			crate::classes::foldable::fold_left::<IdentityBrand, _, _, _>(
				|acc, x: i32| acc + x,
				0,
				Identity(1)
			),
			1
		);
	}

	/// Tests the `traverse` function.
	#[test]
	fn traverse_test() {
		assert_eq!(
			crate::classes::traversable::traverse::<IdentityBrand, OptionBrand, _, _, _>(
				|x: i32| Some(x + 1),
				Identity(1)
			),
			Some(Identity(2))
		);
	}
}
