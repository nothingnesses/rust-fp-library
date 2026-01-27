//! Implementation of the `Thunk` type.
//!
//! This module provides the [`Thunk`] type, which represents a suspended computation,
//! used with the [`Free`](crate::types::Free) monad.
//!
//! ### Examples
//!
//! ```
//! use fp_library::types::*;
//!
//! let thunk = Thunk::new(|| 42);
//! assert_eq!(thunk.run(), 42);
//! ```

use crate::{
	Apply,
	brands::ThunkBrand,
	classes::{
		ApplyFirst, ApplySecond, CloneableFn, Defer, Foldable, Functor, Lift, MonadRec, Monoid,
		Pointed, Runnable, Semiapplicative, Semigroup, Semimonad,
	},
	impl_kind,
	kinds::*,
	types::Step,
};

/// A suspended computation that produces a value of type `A`.
///
/// `Thunk` wraps a closure that takes no arguments and returns a value.
/// It is used to delay evaluation until the value is needed.
///
/// ### Type Parameters
///
/// * `A`: The type of the value produced by the thunk.
///
/// ### Fields
///
/// * `0`: The boxed closure.
///
/// ### Examples
///
/// ```
/// use fp_library::types::*;
///
/// let thunk = Thunk::new(|| 1 + 1);
/// assert_eq!(thunk.run(), 2);
/// ```
pub struct Thunk<'a, A>(Box<dyn FnOnce() -> A + 'a>);

impl<'a, A> Thunk<'a, A> {
	/// Creates a new `Thunk` from a closure.
	///
	/// ### Type Signature
	///
	/// `forall a. (FnOnce() -> a) -> Thunk a`
	///
	/// ### Parameters
	///
	/// * `f`: The closure to suspend.
	///
	/// ### Returns
	///
	/// A new `Thunk` containing the closure.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::types::*;
	///
	/// let thunk = Thunk::new(|| 42);
	/// ```
	pub fn new(f: impl FnOnce() -> A + 'a) -> Self {
		Thunk(Box::new(f))
	}

	/// Runs the thunk, returning the result.
	///
	/// ### Type Signature
	///
	/// `forall a. Thunk a -> a`
	///
	/// ### Returns
	///
	/// The result of the suspended computation.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::types::*;
	///
	/// let thunk = Thunk::new(|| 42);
	/// assert_eq!(thunk.run(), 42);
	/// ```
	pub fn run(self) -> A {
		(self.0)()
	}
}

impl_kind! {
	for ThunkBrand {
		type Of<'a, A: 'a>: 'a = Thunk<'a, A>;
	}
}

impl<'a, A: 'a> Defer<'a> for Thunk<'a, A> {
	fn defer<FnBrand: 'a + CloneableFn>(f: <FnBrand as CloneableFn>::Of<'a, (), Self>) -> Self
	where
		Self: Sized,
	{
		Thunk::new(move || f(()).run())
	}
}

impl Functor for ThunkBrand {
	/// Maps a function over the value in the thunk.
	///
	/// ### Type Signature
	///
	/// `forall b a. Functor Thunk => (a -> b, Thunk a) -> Thunk b`
	///
	/// ### Type Parameters
	///
	/// * `B`: The type of the result of applying the function.
	/// * `A`: The type of the value inside the thunk.
	/// * `F`: The type of the function to apply.
	///
	/// ### Parameters
	///
	/// * `f`: The function to apply.
	/// * `fa`: The thunk to map over.
	///
	/// ### Returns
	///
	/// A new thunk that, when run, applies the function to the result of the original thunk.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, functions::*, types::*};
	///
	/// let thunk = Thunk::new(|| 5);
	/// let mapped = map::<ThunkBrand, _, _, _>(|x| x * 2, thunk);
	/// assert_eq!(mapped.run(), 10);
	/// ```
	fn map<'a, B: 'a, A: 'a, F>(
		f: F,
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)
	where
		F: Fn(A) -> B + 'a,
	{
		Thunk::new(move || f(fa.run()))
	}
}

impl Runnable for ThunkBrand {
	/// Runs the thunk, producing the inner value.
	///
	/// ### Type Signature
	///
	/// `forall a. Runnable Thunk => Thunk a -> a`
	///
	/// ### Type Parameters
	///
	/// * `A`: The type of the value inside the thunk.
	///
	/// ### Parameters
	///
	/// * `fa`: The thunk to run.
	///
	/// ### Returns
	///
	/// The result of running the thunk.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, functions::*, types::*};
	///
	/// let thunk = Thunk::new(|| 42);
	/// assert_eq!(runnable_run::<ThunkBrand, _>(thunk), 42);
	/// ```
	fn run<'a, A: 'a>(fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)) -> A {
		fa.run()
	}
}

impl Pointed for ThunkBrand {
	/// Wraps a value in a `Thunk`.
	///
	/// ### Type Signature
	///
	/// `forall a. a -> Thunk a`
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
	/// A new `Thunk` containing the value.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, functions::*, types::*};
	///
	/// let thunk = pure::<ThunkBrand, _>(42);
	/// assert_eq!(thunk.run(), 42);
	/// ```
	fn pure<'a, A: 'a>(a: A) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>) {
		Thunk::new(move || a)
	}
}

impl Lift for ThunkBrand {
	/// Lifts a binary function to operate on `Thunk`s.
	///
	/// ### Type Signature
	///
	/// `forall c a b. ((a, b) -> c, Thunk a, Thunk b) -> Thunk c`
	///
	/// ### Type Parameters
	///
	/// * `C`: The result type.
	/// * `A`: The first input type.
	/// * `B`: The second input type.
	/// * `F`: The function type.
	///
	/// ### Parameters
	///
	/// * `f`: The binary function.
	/// * `fa`: The first thunk.
	/// * `fb`: The second thunk.
	///
	/// ### Returns
	///
	/// A new `Thunk` containing the result of applying the function.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, classes::lift::Lift, functions::*, types::*};
	///
	/// let t1 = pure::<ThunkBrand, _>(10);
	/// let t2 = pure::<ThunkBrand, _>(20);
	/// let t3 = ThunkBrand::lift2(|a, b| a + b, t1, t2);
	/// assert_eq!(t3.run(), 30);
	/// ```
	fn lift2<'a, C, A, B, F>(
		f: F,
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		fb: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>),
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, C>)
	where
		F: Fn(A, B) -> C + 'a,
		A: Clone + 'a,
		B: Clone + 'a,
		C: 'a,
	{
		Thunk::new(move || f(fa.run(), fb.run()))
	}
}

impl ApplyFirst for ThunkBrand {}

impl ApplySecond for ThunkBrand {}

impl Semiapplicative for ThunkBrand {
	/// Applies a function wrapped in a `Thunk` to a value wrapped in a `Thunk`.
	///
	/// ### Type Signature
	///
	/// `forall b a. (Thunk (a -> b), Thunk a) -> Thunk b`
	///
	/// ### Type Parameters
	///
	/// * `FnBrand`: The brand of the function wrapper.
	/// * `B`: The result type.
	/// * `A`: The input type.
	///
	/// ### Parameters
	///
	/// * `ff`: The thunk containing the function.
	/// * `fa`: The thunk containing the value.
	///
	/// ### Returns
	///
	/// A new `Thunk` containing the result of applying the function.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, classes::semiapplicative::Semiapplicative, functions::*, types::*};
	///
	/// let tf = pure::<ThunkBrand, _>(cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2));
	/// let ta = pure::<ThunkBrand, _>(21);
	/// let res = ThunkBrand::apply::<RcFnBrand, _, _>(tf, ta);
	/// assert_eq!(res.run(), 42);
	/// ```
	fn apply<'a, FnBrand: 'a + CloneableFn, B: 'a, A: 'a + Clone>(
		ff: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, <FnBrand as CloneableFn>::Of<'a, A, B>>),
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
		Thunk::new(move || ff.run()(fa.run()))
	}
}

impl Semimonad for ThunkBrand {
	/// Chains two `Thunk` computations.
	///
	/// ### Type Signature
	///
	/// `forall b a. (Thunk a, a -> Thunk b) -> Thunk b`
	///
	/// ### Type Parameters
	///
	/// * `B`: The result type.
	/// * `A`: The input type.
	/// * `F`: The function type.
	///
	/// ### Parameters
	///
	/// * `ma`: The first thunk.
	/// * `f`: The function that produces the second thunk.
	///
	/// ### Returns
	///
	/// A new `Thunk` representing the chained computation.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, classes::semimonad::Semimonad, functions::*, types::*};
	///
	/// let ta = pure::<ThunkBrand, _>(10);
	/// let res = ThunkBrand::bind(ta, |x| pure::<ThunkBrand, _>(x + 5));
	/// assert_eq!(res.run(), 15);
	/// ```
	fn bind<'a, B: 'a, A: 'a, F>(
		ma: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		f: F,
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)
	where
		F: Fn(A) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) + 'a,
	{
		Thunk::new(move || f(ma.run()).run())
	}
}

impl MonadRec for ThunkBrand {
	/// Performs tail-recursive `Thunk` computation.
	///
	/// ### Type Signature
	///
	/// `forall b a. (a -> Thunk (Step a b), a) -> Thunk b`
	///
	/// ### Type Parameters
	///
	/// * `B`: The result type.
	/// * `A`: The state type.
	/// * `F`: The step function type.
	///
	/// ### Parameters
	///
	/// * `f`: The step function.
	/// * `a`: The initial state.
	///
	/// ### Returns
	///
	/// A new `Thunk` representing the recursive computation.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, classes::monad_rec::MonadRec, functions::*, types::*};
	///
	/// let res = ThunkBrand::tail_rec_m(
	///     |x| {
	///         pure::<ThunkBrand, _>(if x < 10 {
	///             Step::Loop(x + 1)
	///         } else {
	///             Step::Done(x)
	///         })
	///     },
	///     0,
	/// );
	/// assert_eq!(res.run(), 10);
	/// ```
	fn tail_rec_m<'a, A: 'a, B: 'a, F>(
		f: F,
		a: A,
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)
	where
		F: Fn(A) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Step<A, B>>)
			+ Clone
			+ 'a,
	{
		Thunk::new(move || {
			let mut current = a;
			loop {
				match f(current).run() {
					Step::Loop(next) => current = next,
					Step::Done(res) => break res,
				}
			}
		})
	}
}

impl Foldable for ThunkBrand {
	/// Folds the `Thunk` by applying a function to its value and an initial accumulator.
	///
	/// ### Type Signature
	///
	/// `forall b a. ((a, b) -> b, b, Thunk a) -> b`
	///
	/// ### Type Parameters
	///
	/// * `FnBrand`: The brand of the function wrapper (unused).
	/// * `B`: The accumulator type.
	/// * `A`: The value type.
	/// * `Func`: The folding function type.
	///
	/// ### Parameters
	///
	/// * `func`: The folding function.
	/// * `initial`: The initial accumulator.
	/// * `fa`: The thunk to fold.
	///
	/// ### Returns
	///
	/// The final accumulator value.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, classes::foldable::Foldable, functions::*, types::*};
	///
	/// let t = pure::<ThunkBrand, _>(10);
	/// let res = ThunkBrand::fold_right::<RcFnBrand, _, _, _>(|a, b| a + b, 5, t);
	/// assert_eq!(res, 15);
	/// ```
	fn fold_right<'a, FnBrand, B: 'a, A: 'a + Clone, Func>(
		func: Func,
		initial: B,
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> B
	where
		Func: Fn(A, B) -> B + 'a,
		FnBrand: CloneableFn + 'a,
	{
		func(fa.run(), initial)
	}
}

impl<'a, A: Semigroup + 'a> Semigroup for Thunk<'a, A> {
	/// Combines two `Thunk`s by combining their results.
	///
	/// ### Type Signature
	///
	/// `forall a. Semigroup a => (Thunk a, Thunk a) -> Thunk a`
	///
	/// ### Parameters
	///
	/// * `a`: The first thunk.
	/// * `b`: The second thunk.
	///
	/// ### Returns
	///
	/// A new `Thunk` containing the combined result.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, classes::semigroup::Semigroup, functions::*, types::*};
	///
	/// let t1 = pure::<ThunkBrand, _>("Hello".to_string());
	/// let t2 = pure::<ThunkBrand, _>(" World".to_string());
	/// let t3 = Semigroup::append(t1, t2);
	/// assert_eq!(t3.run(), "Hello World");
	/// ```
	fn append(
		a: Self,
		b: Self,
	) -> Self {
		Thunk::new(move || Semigroup::append(a.run(), b.run()))
	}
}

impl<'a, A: Monoid + 'a> Monoid for Thunk<'a, A> {
	/// Returns the identity `Thunk`.
	///
	/// ### Type Signature
	///
	/// `forall a. Monoid a => () -> Thunk a`
	///
	/// ### Returns
	///
	/// A `Thunk` producing the identity value of `A`.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{classes::*, types::*};
	///
	/// let t: Thunk<String> = Monoid::empty();
	/// assert_eq!(t.run(), "");
	/// ```
	fn empty() -> Self {
		Thunk::new(|| Monoid::empty())
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	/// Tests the `Thunk::new` and `Thunk::run` methods.
	///
	/// Verifies that a thunk can be created from a closure and run to produce the expected value.
	#[test]
	fn test_thunk_execution() {
		let thunk = Thunk::new(|| 42);
		assert_eq!(thunk.run(), 42);
	}

	/// Tests the `Functor` implementation for `ThunkBrand`.
	///
	/// Verifies that `map` correctly transforms the value inside a thunk.
	#[test]
	fn test_thunk_functor() {
		use crate::classes::functor::map;
		let thunk = Thunk::new(|| 5);
		let mapped = map::<ThunkBrand, _, _, _>(|x| x * 2, thunk);
		assert_eq!(mapped.run(), 10);
	}

	/// Tests the `Runnable` implementation for `ThunkBrand`.
	///
	/// Verifies that `run` correctly runs the thunk.
	#[test]
	fn test_thunk_runnable() {
		use crate::classes::Runnable;
		let thunk = Thunk::new(|| 42);
		assert_eq!(ThunkBrand::run(thunk), 42);
	}

	/// Tests Defer implementation.
	#[test]
	fn test_defer() {
		use crate::brands::RcFnBrand;
		use crate::classes::defer::defer;
		use crate::functions::cloneable_fn_new;

		let thunk: Thunk<i32> =
			defer::<Thunk<i32>, RcFnBrand>(cloneable_fn_new::<RcFnBrand, _, _>(|_| {
				Thunk::new(|| 42)
			}));
		assert_eq!(thunk.run(), 42);
	}

	/// Tests the `Pointed` implementation for `ThunkBrand`.
	///
	/// Verifies that `pure` creates a thunk that returns the expected value.
	#[test]
	fn test_thunk_pointed() {
		use crate::{brands::*, functions::*};
		let thunk = pure::<ThunkBrand, _>(42);
		assert_eq!(thunk.run(), 42);
	}

	/// Tests the `Lift` implementation for `ThunkBrand`.
	///
	/// Verifies that `lift2` correctly combines two thunks using a binary function.
	#[test]
	fn test_thunk_lift() {
		use crate::classes::lift::lift2;
		use crate::{brands::*, functions::*};
		let t1 = pure::<ThunkBrand, _>(10);
		let t2 = pure::<ThunkBrand, _>(20);
		let t3 = lift2::<ThunkBrand, _, _, _, _>(|a, b| a + b, t1, t2);
		assert_eq!(t3.run(), 30);
	}

	/// Tests the `Semiapplicative` implementation for `ThunkBrand`.
	///
	/// Verifies that `apply` correctly applies a function inside a thunk to a value inside a thunk.
	#[test]
	fn test_thunk_semiapplicative() {
		use crate::brands::RcFnBrand;
		use crate::classes::semiapplicative::apply;
		use crate::functions::cloneable_fn_new;
		use crate::functions::pure;

		let tf = pure::<ThunkBrand, _>(cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2));
		let ta = pure::<ThunkBrand, _>(21);
		let res = apply::<RcFnBrand, ThunkBrand, _, _>(tf, ta);
		assert_eq!(res.run(), 42);
	}

	/// Tests the `Semimonad` implementation for `ThunkBrand`.
	///
	/// Verifies that `bind` correctly chains computations.
	#[test]
	fn test_thunk_semimonad() {
		use crate::classes::semimonad::bind;
		use crate::{brands::*, functions::*};
		let ta = pure::<ThunkBrand, _>(10);
		let res = bind::<ThunkBrand, _, _, _>(ta, |x| pure::<ThunkBrand, _>(x + 5));
		assert_eq!(res.run(), 15);
	}

	/// Tests the `MonadRec` implementation for `ThunkBrand`.
	///
	/// Verifies that `tail_rec_m` correctly performs tail recursion without stack overflow.
	#[test]
	fn test_thunk_monad_rec() {
		use crate::classes::monad_rec::tail_rec_m;
		use crate::types::Step;
		use crate::{brands::*, functions::*};

		let res = tail_rec_m::<ThunkBrand, _, _, _>(
			|x| pure::<ThunkBrand, _>(if x < 10 { Step::Loop(x + 1) } else { Step::Done(x) }),
			0,
		);
		assert_eq!(res.run(), 10);
	}

	/// Tests the `Foldable` implementation for `ThunkBrand`.
	///
	/// Verifies that `fold_right` correctly folds the thunk.
	#[test]
	fn test_thunk_foldable() {
		use crate::brands::RcFnBrand;
		use crate::classes::foldable::fold_right;
		use crate::{brands::*, functions::*};

		let t = pure::<ThunkBrand, _>(10);
		let res = fold_right::<RcFnBrand, ThunkBrand, _, _, _>(|a, b| a + b, 5, t);
		assert_eq!(res, 15);
	}

	/// Tests the `Semigroup` implementation for `Thunk`.
	///
	/// Verifies that `append` correctly combines two thunks.
	#[test]
	fn test_thunk_semigroup() {
		use crate::classes::semigroup::append;
		use crate::{brands::*, functions::*};
		let t1 = pure::<ThunkBrand, _>("Hello".to_string());
		let t2 = pure::<ThunkBrand, _>(" World".to_string());
		let t3 = append(t1, t2);
		assert_eq!(t3.run(), "Hello World");
	}

	/// Tests the `Monoid` implementation for `Thunk`.
	///
	/// Verifies that `empty` returns the identity element.
	#[test]
	fn test_thunk_monoid() {
		use crate::classes::monoid::empty;
		let t: Thunk<String> = empty();
		assert_eq!(t.run(), "");
	}
}
