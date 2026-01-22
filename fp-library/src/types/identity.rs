//! Implementations for [`Identity`], a type that wraps a value.
//!
//! This module provides implementations of functional programming traits for the [`Identity`] type.

use crate::{
	Apply,
	brands::IdentityBrand,
	classes::{
		applicative::Applicative, apply_first::ApplyFirst, apply_second::ApplySecond,
		cloneable_fn::CloneableFn, foldable::Foldable, functor::Functor, lift::Lift,
		monoid::Monoid, par_foldable::ParFoldable, pointed::Pointed,
		semiapplicative::Semiapplicative, semimonad::Semimonad, send_cloneable_fn::SendCloneableFn,
		traversable::Traversable,
	},
	impl_kind,
	kinds::*,
};

/// Wraps a value.
///
/// The `Identity` type represents a trivial wrapper around a value. It is the simplest possible container.
/// It is often used as a base case for higher-kinded types or when a container is required but no additional effect is needed.
///
/// ### Type Parameters
///
/// * `A`: The type of the wrapped value.
///
/// ### Fields
///
/// * `0`: The wrapped value.
///
/// ### Examples
///
/// ```
/// use fp_library::types::Identity;
///
/// let x = Identity(5);
/// assert_eq!(x.0, 5);
/// ```
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Identity<A>(pub A);

impl_kind! {
	for IdentityBrand {
		type Of<'a, A: 'a>: 'a = Identity<A>;
	}
}

impl Functor for IdentityBrand {
	/// Maps a function over the value in the identity.
	///
	/// This method applies a function to the value inside the identity, producing a new identity with the transformed value.
	///
	/// ### Type Signature
	///
	/// `forall b a. Functor Identity => (a -> b, Identity a) -> Identity b`
	///
	/// ### Type Parameters
	///
	/// * `B`: The type of the result of applying the function.
	/// * `A`: The type of the value inside the identity.
	/// * `F`: The type of the function to apply.
	///
	/// ### Parameters
	///
	/// * `f`: The function to apply.
	/// * `fa`: The identity to map over.
	///
	/// ### Returns
	///
	/// A new identity containing the result of applying the function.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::functions::*;
	/// use fp_library::brands::IdentityBrand;
	/// use fp_library::types::Identity;
	///
	/// let x = Identity(5);
	/// let y = map::<IdentityBrand, _, _, _>(|i| i * 2, x);
	/// assert_eq!(y, Identity(10));
	/// ```
	fn map<'a, B: 'a, A: 'a, F>(
		f: F,
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)
	where
		F: Fn(A) -> B + 'a,
	{
		Identity(f(fa.0))
	}
}

impl Lift for IdentityBrand {
	/// Lifts a binary function into the identity context.
	///
	/// This method lifts a binary function to operate on values within the identity context.
	///
	/// ### Type Signature
	///
	/// `forall c a b. Lift Identity => ((a, b) -> c, Identity a, Identity b) -> Identity c`
	///
	/// ### Type Parameters
	///
	/// * `C`: The return type of the function.
	/// * `A`: The type of the first identity's value.
	/// * `B`: The type of the second identity's value.
	/// * `F`: The type of the binary function.
	///
	/// ### Parameters
	///
	/// * `f`: The binary function to apply.
	/// * `fa`: The first identity.
	/// * `fb`: The second identity.
	///
	/// ### Returns
	///
	/// A new identity containing the result of applying the function.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::functions::*;
	/// use fp_library::brands::IdentityBrand;
	/// use fp_library::types::Identity;
	///
	/// let x = Identity(1);
	/// let y = Identity(2);
	/// let z = lift2::<IdentityBrand, _, _, _, _>(|a, b| a + b, x, y);
	/// assert_eq!(z, Identity(3));
	/// ```
	fn lift2<'a, C, A, B, F>(
		f: F,
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		fb: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>),
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, C>)
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
	/// This method wraps a value in an identity context.
	///
	/// ### Type Signature
	///
	/// `forall a. Pointed Identity => a -> Identity a`
	///
	/// ### Type Parameters
	///
	/// * `A`: The type of the value to wrap.
	///
	/// ### Parameters
	///
	/// * `a`: The value to wrap.
	///
	/// ### Returns
	///
	/// An identity containing the value.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::functions::*;
	/// use fp_library::brands::IdentityBrand;
	/// use fp_library::types::Identity;
	///
	/// let x = pure::<IdentityBrand, _>(5);
	/// assert_eq!(x, Identity(5));
	/// ```
	fn pure<'a, A: 'a>(a: A) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>) {
		Identity(a)
	}
}

impl ApplyFirst for IdentityBrand {}
impl ApplySecond for IdentityBrand {}

impl Semiapplicative for IdentityBrand {
	/// Applies a wrapped function to a wrapped value.
	///
	/// This method applies a function wrapped in an identity to a value wrapped in an identity.
	///
	/// ### Type Signature
	///
	/// `forall fn_brand b a. Semiapplicative Identity => (Identity (fn_brand a b), Identity a) -> Identity b`
	///
	/// ### Type Parameters
	///
	/// * `FnBrand`: The brand of the cloneable function wrapper.
	/// * `B`: The type of the output value.
	/// * `A`: The type of the input value.
	///
	/// ### Parameters
	///
	/// * `ff`: The identity containing the function.
	/// * `fa`: The identity containing the value.
	///
	/// ### Returns
	///
	/// A new identity containing the result of applying the function.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, classes::*, functions::*, types::*};
	///
	/// let f = Identity(cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2));
	/// let x = Identity(5);
	/// let y = apply::<RcFnBrand, IdentityBrand, _, _>(f, x);
	/// assert_eq!(y, Identity(10));
	/// ```
	fn apply<'a, FnBrand: 'a + CloneableFn, B: 'a, A: 'a + Clone>(
		ff: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, <FnBrand as CloneableFn>::Of<'a, A, B>>),
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
		Identity(ff.0(fa.0))
	}
}

impl Semimonad for IdentityBrand {
	/// Chains identity computations.
	///
	/// This method chains two identity computations, where the second computation depends on the result of the first.
	///
	/// ### Type Signature
	///
	/// `forall b a. Semimonad Identity => (Identity a, a -> Identity b) -> Identity b`
	///
	/// ### Type Parameters
	///
	/// * `B`: The type of the result of the second computation.
	/// * `A`: The type of the result of the first computation.
	/// * `F`: The type of the function to apply.
	///
	/// ### Parameters
	///
	/// * `ma`: The first identity.
	/// * `f`: The function to apply to the value inside the identity.
	///
	/// ### Returns
	///
	/// The result of applying `f` to the value.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::functions::*;
	/// use fp_library::brands::IdentityBrand;
	/// use fp_library::types::Identity;
	///
	/// let x = Identity(5);
	/// let y = bind::<IdentityBrand, _, _, _>(x, |i| Identity(i * 2));
	/// assert_eq!(y, Identity(10));
	/// ```
	fn bind<'a, B: 'a, A: 'a, F>(
		ma: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		f: F,
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)
	where
		F: Fn(A) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) + 'a,
	{
		f(ma.0)
	}
}

impl Foldable for IdentityBrand {
	/// Folds the identity from the right.
	///
	/// This method performs a right-associative fold of the identity. Since `Identity` contains only one element, this is equivalent to applying the function to the element and the initial value.
	///
	/// ### Type Signature
	///
	/// `forall b a. Foldable Identity => ((a, b) -> b, b, Identity a) -> b`
	///
	/// ### Type Parameters
	///
	/// * `FnBrand`: The brand of the cloneable function to use.
	/// * `B`: The type of the accumulator.
	/// * `A`: The type of the elements in the structure.
	/// * `Func`: The type of the folding function.
	///
	/// ### Parameters
	///
	/// * `func`: The function to apply to each element and the accumulator.
	/// * `initial`: The initial value of the accumulator.
	/// * `fa`: The identity to fold.
	///
	/// ### Returns
	///
	/// The final accumulator value.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::functions::*;
	/// use fp_library::brands::{IdentityBrand, RcFnBrand};
	/// use fp_library::types::Identity;
	///
	/// let x = Identity(5);
	/// let y = fold_right::<RcFnBrand, IdentityBrand, _, _, _>(|a, b| a + b, 10, x);
	/// assert_eq!(y, 15);
	/// ```
	fn fold_right<'a, FnBrand, B: 'a, A: 'a, Func>(
		func: Func,
		initial: B,
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> B
	where
		Func: Fn(A, B) -> B + 'a,
		FnBrand: CloneableFn + 'a,
	{
		func(fa.0, initial)
	}

	/// Folds the identity from the left.
	///
	/// This method performs a left-associative fold of the identity. Since `Identity` contains only one element, this is equivalent to applying the function to the initial value and the element.
	///
	/// ### Type Signature
	///
	/// `forall b a. Foldable Identity => ((b, a) -> b, b, Identity a) -> b`
	///
	/// ### Type Parameters
	///
	/// * `FnBrand`: The brand of the cloneable function to use.
	/// * `B`: The type of the accumulator.
	/// * `A`: The type of the elements in the structure.
	/// * `Func`: The type of the folding function.
	///
	/// ### Parameters
	///
	/// * `func`: The function to apply to the accumulator and each element.
	/// * `initial`: The initial value of the accumulator.
	/// * `fa`: The structure to fold.
	///
	/// ### Returns
	///
	/// The final accumulator value.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::functions::*;
	/// use fp_library::brands::{IdentityBrand, RcFnBrand};
	/// use fp_library::types::Identity;
	///
	/// let x = Identity(5);
	/// let y = fold_left::<RcFnBrand, IdentityBrand, _, _, _>(|b, a| b + a, 10, x);
	/// assert_eq!(y, 15);
	/// ```
	fn fold_left<'a, FnBrand, B: 'a, A: 'a, Func>(
		func: Func,
		initial: B,
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> B
	where
		Func: Fn(B, A) -> B + 'a,
		FnBrand: CloneableFn + 'a,
	{
		func(initial, fa.0)
	}

	/// Maps the value to a monoid and returns it.
	///
	/// This method maps the element of the identity to a monoid.
	///
	/// ### Type Signature
	///
	/// `forall m a. (Foldable Identity, Monoid m) => ((a) -> m, Identity a) -> m`
	///
	/// ### Type Parameters
	///
	/// * `FnBrand`: The brand of the cloneable function to use.
	/// * `M`: The type of the monoid.
	/// * `A`: The type of the elements in the structure.
	/// * `Func`: The type of the mapping function.
	///
	/// ### Parameters
	///
	/// * `func`: The mapping function.
	/// * `fa`: The identity to fold.
	///
	/// ### Returns
	///
	/// The monoid value.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::functions::*;
	/// use fp_library::brands::{IdentityBrand, RcFnBrand};
	/// use fp_library::types::{Identity, string}; // Import to bring Monoid impl for String into scope
	///
	/// let x = Identity(5);
	/// let y = fold_map::<RcFnBrand, IdentityBrand, _, _, _>(|a: i32| a.to_string(), x);
	/// assert_eq!(y, "5".to_string());
	/// ```
	fn fold_map<'a, FnBrand, M, A: 'a, Func>(
		func: Func,
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> M
	where
		M: Monoid + 'a,
		Func: Fn(A) -> M + 'a,
		FnBrand: CloneableFn + 'a,
	{
		func(fa.0)
	}
}

impl Traversable for IdentityBrand {
	/// Traverses the identity with an applicative function.
	///
	/// This method maps the element of the identity to a computation, evaluates it, and wraps the result in the applicative context.
	///
	/// ### Type Signature
	///
	/// `forall f b a. (Traversable Identity, Applicative f) => (a -> f b, Identity a) -> f (Identity b)`
	///
	/// ### Type Parameters
	///
	/// * `F`: The applicative context.
	/// * `B`: The type of the elements in the resulting traversable structure.
	/// * `A`: The type of the elements in the traversable structure.
	/// * `Func`: The type of the function to apply.
	///
	/// ### Parameters
	///
	/// * `func`: The function to apply to each element, returning a value in an applicative context.
	/// * `ta`: The identity to traverse.
	///
	/// ### Returns
	///
	/// The identity wrapped in the applicative context.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::functions::*;
	/// use fp_library::brands::{IdentityBrand, OptionBrand};
	/// use fp_library::types::Identity;
	///
	/// let x = Identity(5);
	/// let y = traverse::<IdentityBrand, OptionBrand, _, _, _>(|a| Some(a * 2), x);
	/// assert_eq!(y, Some(Identity(10)));
	/// ```
	fn traverse<'a, F: Applicative, B: 'a + Clone, A: 'a + Clone, Func>(
		func: Func,
		ta: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)>)
	where
		Func: Fn(A) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) + 'a,
		Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>): Clone,
	{
		F::map(|b| Identity(b), func(ta.0))
	}
	/// Sequences an identity of applicative.
	///
	/// This method evaluates the computation inside the identity and wraps the result in the applicative context.
	///
	/// ### Type Signature
	///
	/// `forall f a. (Traversable Identity, Applicative f) => (Identity (f a)) -> f (Identity a)`
	///
	/// ### Type Parameters
	///
	/// * `F`: The applicative context.
	/// * `A`: The type of the elements in the traversable structure.
	///
	/// ### Parameters
	///
	/// * `ta`: The identity containing the applicative value.
	///
	/// # Returns
	///
	/// The identity wrapped in the applicative context.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::functions::*;
	/// use fp_library::brands::{IdentityBrand, OptionBrand};
	/// use fp_library::types::Identity;
	///
	/// let x = Identity(Some(5));
	/// let y = sequence::<IdentityBrand, OptionBrand, _>(x);
	/// assert_eq!(y, Some(Identity(5)));
	/// ```
	fn sequence<'a, F: Applicative, A: 'a + Clone>(
		ta: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)>)
	) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)>)
	where
		Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>): Clone,
		Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>): Clone,
	{
		F::map(|a| Identity(a), ta.0)
	}
}

impl<FnBrand: SendCloneableFn> ParFoldable<FnBrand> for IdentityBrand {
	/// Maps the value to a monoid and returns it in parallel.
	///
	/// This method maps the element of the identity to a monoid. Since `Identity` contains only one element, no actual parallelism occurs, but the interface is satisfied.
	///
	/// ### Type Signature
	///
	/// `forall fn_brand m a. (ParFoldable Identity, Monoid m, Send m, Sync m) => (fn_brand a m, Identity a) -> m`
	///
	/// ### Type Parameters
	///
	/// * `M`: The monoid type (must be `Send + Sync`).
	/// * `A`: The element type (must be `Send + Sync`).
	///
	/// ### Parameters
	///
	/// * `func`: The mapping function.
	/// * `fa`: The identity to fold.
	///
	/// ### Returns
	///
	/// The combined monoid value.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::functions::*;
	/// use fp_library::brands::{IdentityBrand, ArcFnBrand};
	/// use fp_library::types::Identity;
	///
	/// let x = Identity(1);
	/// let f = send_cloneable_fn_new::<ArcFnBrand, _, _>(|x: i32| x.to_string());
	/// let y = par_fold_map::<ArcFnBrand, IdentityBrand, _, _>(f, x);
	/// assert_eq!(y, "1".to_string());
	/// ```
	fn par_fold_map<'a, M, A>(
		func: <FnBrand as SendCloneableFn>::SendOf<'a, A, M>,
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> M
	where
		A: 'a + Clone + Send + Sync,
		M: Monoid + Send + Sync + 'a,
	{
		func(fa.0)
	}

	/// Folds the identity from the right in parallel.
	///
	/// This method performs a right-associative fold of the identity. Since `Identity` contains only one element, no actual parallelism occurs.
	///
	/// ### Type Signature
	///
	/// `forall fn_brand b a. ParFoldable Identity => (fn_brand (a, b) b, b, Identity a) -> b`
	///
	/// ### Type Parameters
	///
	/// * `B`: The accumulator type (must be `Send + Sync`).
	/// * `A`: The element type (must be `Send + Sync`).
	///
	/// ### Parameters
	///
	/// * `func`: The thread-safe function to apply to each element and the accumulator.
	/// * `initial`: The initial value of the accumulator.
	/// * `fa`: The identity to fold.
	///
	/// ### Returns
	///
	/// The final accumulator value.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::functions::*;
	/// use fp_library::brands::{IdentityBrand, ArcFnBrand};
	/// use fp_library::types::Identity;
	///
	/// let x = Identity(1);
	/// let f = send_cloneable_fn_new::<ArcFnBrand, _, _>(|(a, b): (i32, i32)| a + b);
	/// let y = par_fold_right::<ArcFnBrand, IdentityBrand, _, _>(f, 10, x);
	/// assert_eq!(y, 11);
	/// ```
	fn par_fold_right<'a, B, A>(
		func: <FnBrand as SendCloneableFn>::SendOf<'a, (A, B), B>,
		initial: B,
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> B
	where
		A: 'a + Clone + Send + Sync,
		B: Send + Sync + 'a,
	{
		func((fa.0, initial))
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
		apply::<RcFnBrand, IdentityBrand, _, _>(
			pure::<IdentityBrand, _>(<RcFnBrand as CloneableFn>::new(identity)),
			v,
		) == v
	}

	/// Tests the homomorphism law for Applicative.
	#[quickcheck]
	fn applicative_homomorphism(x: i32) -> bool {
		let f = |x: i32| x.wrapping_mul(2);
		apply::<RcFnBrand, IdentityBrand, _, _>(
			pure::<IdentityBrand, _>(<RcFnBrand as CloneableFn>::new(f)),
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

		let v = pure::<IdentityBrand, _>(<RcFnBrand as CloneableFn>::new(v_fn));
		let u = pure::<IdentityBrand, _>(<RcFnBrand as CloneableFn>::new(u_fn));

		// RHS: u <*> (v <*> w)
		let vw = apply::<RcFnBrand, IdentityBrand, _, _>(v.clone(), w.clone());
		let rhs = apply::<RcFnBrand, IdentityBrand, _, _>(u.clone(), vw);

		// LHS: pure(compose) <*> u <*> v <*> w
		// equivalent to (u . v) <*> w
		let composed = move |x| u_fn(v_fn(x));
		let uv = pure::<IdentityBrand, _>(<RcFnBrand as CloneableFn>::new(composed));

		let lhs = apply::<RcFnBrand, IdentityBrand, _, _>(uv, w);

		lhs == rhs
	}

	/// Tests the interchange law for Applicative.
	#[quickcheck]
	fn applicative_interchange(y: i32) -> bool {
		// u <*> pure y = pure ($ y) <*> u
		let f = |x: i32| x.wrapping_mul(2);
		let u = pure::<IdentityBrand, _>(<RcFnBrand as CloneableFn>::new(f));

		let lhs = apply::<RcFnBrand, IdentityBrand, _, _>(u.clone(), pure::<IdentityBrand, _>(y));

		let rhs_fn =
			<RcFnBrand as CloneableFn>::new(move |f: std::rc::Rc<dyn Fn(i32) -> i32>| f(y));
		let rhs = apply::<RcFnBrand, IdentityBrand, _, _>(pure::<IdentityBrand, _>(rhs_fn), u);

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
			crate::classes::foldable::fold_right::<RcFnBrand, IdentityBrand, _, _, _>(
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
			crate::classes::foldable::fold_left::<RcFnBrand, IdentityBrand, _, _, _>(
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

	// ParFoldable Tests

	/// Tests `par_fold_map`.
	#[test]
	fn par_fold_map_test() {
		use crate::{brands::*, functions::*};

		let x = Identity(1);
		let f = send_cloneable_fn_new::<ArcFnBrand, _, _>(|x: i32| x.to_string());
		assert_eq!(par_fold_map::<ArcFnBrand, IdentityBrand, _, _>(f, x), "1".to_string());
	}

	/// Tests `par_fold_right`.
	#[test]
	fn par_fold_right_test() {
		use crate::{brands::*, functions::*};

		let x = Identity(1);
		let f = send_cloneable_fn_new::<ArcFnBrand, _, _>(|(a, b): (i32, i32)| a + b);
		assert_eq!(par_fold_right::<ArcFnBrand, IdentityBrand, _, _>(f, 10, x), 11);
	}
}
