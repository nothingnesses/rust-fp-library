//! Implementations for [`Thunk`], a deferred computation type.
//!
//! This module provides the [`Thunk`] type, which represents a deferred computation that produces a value.
//! Unlike [`Task`](crate::types::Trampoline), `Thunk` is not stack-safe for deep recursion but supports higher-kinded types and borrowing.

use crate::{
	Apply,
	brands::ThunkBrand,
	classes::{
		Defer, apply_first::ApplyFirst, apply_second::ApplySecond, cloneable_fn::CloneableFn,
		foldable::Foldable, functor::Functor, lift::Lift, monad_rec::MonadRec, monoid::Monoid,
		pointed::Pointed, runnable::Runnable, semiapplicative::Semiapplicative,
		semigroup::Semigroup, semimonad::Semimonad,
	},
	impl_kind,
	kinds::*,
	types::{Lazy, LazyConfig, step::Step},
};

/// A deferred computation that produces a value of type `A`.
///
/// `Thunk` is NOT memoized - each call to `run()` re-executes the computation.
/// This type exists to build computation chains without allocation overhead.
///
/// Unlike [`Task`](crate::types::Trampoline), `Thunk` does NOT require `'static` and CAN implement
/// HKT traits like [`Functor`], [`Semimonad`], etc.
///
/// ### Trade-offs vs Task
///
/// | Aspect         | `Thunk<'a, A>`               | `Task<A>`                    |
/// |----------------|---------------------------|----------------------------|
/// | HKT compatible | ✅ Yes                    | ❌ No (requires `'static`) |
/// | Stack-safe     | ⚠️ Partial (tail_rec_m only) | ✅ Yes (unlimited)         |
/// | Lifetime       | `'a` (can borrow)         | `'static` only             |
/// | Use case       | Glue code, composition    | Deep recursion, pipelines  |
///
/// ### Algebraic Properties
///
/// `Thunk` is a proper Monad:
/// - `pure(a).run() == a` (left identity)
/// - `eval.bind(pure) == eval` (right identity)
/// - `eval.bind(f).bind(g) == eval.bind(|a| f(a).bind(g))` (associativity)
///
/// ### Type Parameters
///
/// * `A`: The type of the value produced by the computation.
///
/// ### Fields
///
/// * `thunk`: The closure that performs the computation.
///
/// ### Examples
///
/// ```
/// use fp_library::types::*;
///
/// let computation = Thunk::new(|| 5)
///     .map(|x| x * 2)
///     .map(|x| x + 1);
///
/// // No computation has happened yet!
/// // Only when we call run() does it execute:
/// let result = computation.run();
/// assert_eq!(result, 11);
/// ```
pub struct Thunk<'a, A> {
	thunk: Box<dyn FnOnce() -> A + 'a>,
}

impl<'a, A: 'a> Thunk<'a, A> {
	/// Creates a new Thunk from a thunk.
	///
	/// ### Type Signature
	///
	/// `forall a. (Unit -> a) -> Thunk a`
	///
	/// ### Type Parameters
	///
	/// * `F`: The type of the thunk.
	///
	/// ### Parameters
	///
	/// * `f`: The thunk to wrap.
	///
	/// ### Returns
	///
	/// A new `Thunk` instance.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::types::*;
	///
	/// let eval = Thunk::new(|| 42);
	/// assert_eq!(eval.run(), 42);
	/// ```
	pub fn new<F>(f: F) -> Self
	where
		F: FnOnce() -> A + 'a,
	{
		Thunk { thunk: Box::new(f) }
	}

	/// Returns a pure value (already computed).
	///
	/// ### Type Signature
	///
	/// `forall a. a -> Thunk a`
	///
	/// ### Parameters
	///
	/// * `a`: The value to wrap.
	///
	/// ### Returns
	///
	/// A new `Thunk` instance containing the value.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::types::*;
	///
	/// let eval = Thunk::pure(42);
	/// assert_eq!(eval.run(), 42);
	/// ```
	pub fn pure(a: A) -> Self
	where
		A: 'a,
	{
		Thunk::new(move || a)
	}

	/// Defers a computation that returns an Thunk.
	///
	/// ### Type Signature
	///
	/// `forall a. (Unit -> Thunk a) -> Thunk a`
	///
	/// ### Type Parameters
	///
	/// * `F`: The type of the thunk.
	///
	/// ### Parameters
	///
	/// * `f`: The thunk that returns an `Thunk`.
	///
	/// ### Returns
	///
	/// A new `Thunk` instance.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::types::*;
	///
	/// let eval = Thunk::defer(|| Thunk::pure(42));
	/// assert_eq!(eval.run(), 42);
	/// ```
	pub fn defer<F>(f: F) -> Self
	where
		F: FnOnce() -> Thunk<'a, A> + 'a,
	{
		Thunk::new(move || f().run())
	}

	/// Monadic bind: chains computations.
	///
	/// Note: Each `bind` adds to the call stack. For deep recursion
	/// (>1000 levels), use [`Task`](crate::types::Trampoline) instead.
	///
	/// ### Type Signature
	///
	/// `forall b a. (a -> Thunk b, Thunk a) -> Thunk b`
	///
	/// ### Type Parameters
	///
	/// * `B`: The type of the result of the new computation.
	/// * `F`: The type of the function to apply.
	///
	/// ### Parameters
	///
	/// * `f`: The function to apply to the result of the computation.
	///
	/// ### Returns
	///
	/// A new `Thunk` instance representing the chained computation.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::types::*;
	///
	/// let eval = Thunk::pure(21).bind(|x| Thunk::pure(x * 2));
	/// assert_eq!(eval.run(), 42);
	/// ```
	pub fn bind<B: 'a, F>(
		self,
		f: F,
	) -> Thunk<'a, B>
	where
		F: FnOnce(A) -> Thunk<'a, B> + 'a,
	{
		Thunk::new(move || {
			let a = (self.thunk)();
			let eval_b = f(a);
			(eval_b.thunk)()
		})
	}

	/// Functor map: transforms the result.
	///
	/// ### Type Signature
	///
	/// `forall b a. (a -> b, Thunk a) -> Thunk b`
	///
	/// ### Type Parameters
	///
	/// * `B`: The type of the result of the transformation.
	/// * `F`: The type of the transformation function.
	///
	/// ### Parameters
	///
	/// * `f`: The function to apply to the result of the computation.
	///
	/// ### Returns
	///
	/// A new `Thunk` instance with the transformed result.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::types::*;
	///
	/// let eval = Thunk::pure(21).map(|x| x * 2);
	/// assert_eq!(eval.run(), 42);
	/// ```
	pub fn map<B: 'a, F>(
		self,
		f: F,
	) -> Thunk<'a, B>
	where
		F: FnOnce(A) -> B + 'a,
	{
		Thunk::new(move || f((self.thunk)()))
	}

	/// Forces evaluation and returns the result.
	///
	/// ### Type Signature
	///
	/// `forall a. Thunk a -> a`
	///
	/// ### Returns
	///
	/// The result of the computation.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::types::*;
	///
	/// let eval = Thunk::pure(42);
	/// assert_eq!(eval.run(), 42);
	/// ```
	pub fn run(self) -> A {
		(self.thunk)()
	}
}

impl<'a, A, Config> From<Lazy<'a, A, Config>> for Thunk<'a, A>
where
	A: Clone + 'a,
	Config: LazyConfig,
{
	fn from(lazy: Lazy<'a, A, Config>) -> Self {
		Thunk::new(move || lazy.get().clone())
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
		Thunk::defer(move || f(()))
	}
}

impl Functor for ThunkBrand {
	/// Maps a function over the result of an Thunk computation.
	///
	/// ### Type Signature
	///
	/// `forall b a. Functor Thunk => (a -> b, Thunk a) -> Thunk b`
	///
	/// ### Type Parameters
	///
	/// * `B`: The type of the result of the transformation.
	/// * `A`: The type of the value inside the Thunk.
	/// * `F`: The type of the transformation function.
	///
	/// ### Parameters
	///
	/// * `f`: The function to apply to the result of the computation.
	/// * `fa`: The Thunk instance.
	///
	/// ### Returns
	///
	/// A new Thunk instance with the transformed result.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, classes::*, types::*};
	///
	/// let eval = Thunk::pure(10);
	/// let mapped = ThunkBrand::map(|x| x * 2, eval);
	/// assert_eq!(mapped.run(), 20);
	/// ```
	fn map<'a, B: 'a, A: 'a, F>(
		f: F,
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)
	where
		F: Fn(A) -> B + 'a,
	{
		fa.map(f)
	}
}

impl Pointed for ThunkBrand {
	/// Wraps a value in an Thunk context.
	///
	/// ### Type Signature
	///
	/// `forall a. Pointed Thunk => a -> Thunk a`
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
	/// A new Thunk instance containing the value.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, classes::*, types::*};
	///
	/// let eval: Thunk<i32> = ThunkBrand::pure(42);
	/// assert_eq!(eval.run(), 42);
	/// ```
	fn pure<'a, A: 'a>(a: A) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>) {
		Thunk::pure(a)
	}
}

impl Lift for ThunkBrand {
	/// Lifts a binary function into the Thunk context.
	///
	/// ### Type Signature
	///
	/// `forall c a b. Lift Thunk => ((a, b) -> c, Thunk a, Thunk b) -> Thunk c`
	///
	/// ### Type Parameters
	///
	/// * `C`: The type of the result.
	/// * `A`: The type of the first value.
	/// * `B`: The type of the second value.
	/// * `F`: The type of the binary function.
	///
	/// ### Parameters
	///
	/// * `f`: The binary function to apply.
	/// * `fa`: The first Thunk.
	/// * `fb`: The second Thunk.
	///
	/// ### Returns
	///
	/// A new Thunk instance containing the result of applying the function.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, classes::*, types::*};
	///
	/// let eval1 = Thunk::pure(10);
	/// let eval2 = Thunk::pure(20);
	/// let result = ThunkBrand::lift2(|a, b| a + b, eval1, eval2);
	/// assert_eq!(result.run(), 30);
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
		fa.bind(move |a| fb.map(move |b| f(a, b)))
	}
}

impl ApplyFirst for ThunkBrand {}
impl ApplySecond for ThunkBrand {}

impl Semiapplicative for ThunkBrand {
	/// Applies a function wrapped in Thunk to a value wrapped in Thunk.
	///
	/// ### Type Signature
	///
	/// `forall fn_brand b a. Semiapplicative Thunk => (Thunk (fn_brand a b), Thunk a) -> Thunk b`
	///
	/// ### Type Parameters
	///
	/// * `FnBrand`: The brand of the function wrapper.
	/// * `B`: The type of the result.
	/// * `A`: The type of the input.
	///
	/// ### Parameters
	///
	/// * `ff`: The Thunk containing the function.
	/// * `fa`: The Thunk containing the value.
	///
	/// ### Returns
	///
	/// A new Thunk instance containing the result of applying the function.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, classes::*, types::*, functions::*};
	///
	/// let func = Thunk::pure(cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2));
	/// let val = Thunk::pure(21);
	/// let result = ThunkBrand::apply::<RcFnBrand, _, _>(func, val);
	/// assert_eq!(result.run(), 42);
	/// ```
	fn apply<'a, FnBrand: 'a + CloneableFn, B: 'a, A: 'a + Clone>(
		ff: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, <FnBrand as CloneableFn>::Of<'a, A, B>>),
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
		ff.bind(move |f| {
			fa.map(
				#[allow(clippy::redundant_closure)]
				move |a| f(a),
			)
		})
	}
}

impl Semimonad for ThunkBrand {
	/// Chains Thunk computations.
	///
	/// ### Type Signature
	///
	/// `forall b a. Semimonad Thunk => (Thunk a, a -> Thunk b) -> Thunk b`
	///
	/// ### Type Parameters
	///
	/// * `B`: The type of the result of the new computation.
	/// * `A`: The type of the result of the first computation.
	/// * `F`: The type of the function to apply.
	///
	/// ### Parameters
	///
	/// * `ma`: The first Thunk.
	/// * `f`: The function to apply to the result of the computation.
	///
	/// ### Returns
	///
	/// A new Thunk instance representing the chained computation.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{
	///     brands::*,
	///     classes::*,
	///     types::*,
	/// };
	///
	/// let eval = Thunk::pure(10);
	/// let result = ThunkBrand::bind(eval, |x| Thunk::pure(x * 2));
	/// assert_eq!(result.run(), 20);
	/// ```
	fn bind<'a, B: 'a, A: 'a, F>(
		ma: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		f: F,
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)
	where
		F: Fn(A) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) + 'a,
	{
		ma.bind(f)
	}
}

impl MonadRec for ThunkBrand {
	/// Performs tail-recursive monadic computation.
	///
	/// ### Type Signature
	///
	/// `forall m b a. MonadRec Thunk => (a -> Thunk (Step a b), a) -> Thunk b`
	///
	/// ### Type Parameters
	///
	/// * `B`: The type of the result.
	/// * `A`: The type of the initial value and loop state.
	/// * `F`: The type of the step function.
	///
	/// ### Parameters
	///
	/// * `f`: The step function.
	/// * `initial`: The initial value.
	///
	/// ### Returns
	///
	/// The result of the computation.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, classes::*, types::*};
	///
	/// let result = ThunkBrand::tail_rec_m(
	///     |x| Thunk::pure(if x < 1000 { Step::Loop(x + 1) } else { Step::Done(x) }),
	///     0,
	/// );
	/// assert_eq!(result.run(), 1000);
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

impl Runnable for ThunkBrand {
	/// Runs the eval, producing the inner value.
	///
	/// ### Type Signature
	///
	/// `forall a. Runnable Thunk => Thunk a -> a`
	///
	/// ### Type Parameters
	///
	/// * `A`: The type of the value inside the eval.
	///
	/// ### Parameters
	///
	/// * `fa`: The eval to run.
	///
	/// ### Returns
	///
	/// The result of running the eval.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, classes::*, types::*};
	///
	/// let eval = Thunk::new(|| 42);
	/// assert_eq!(ThunkBrand::run(eval), 42);
	/// ```
	fn run<'a, A: 'a>(fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)) -> A {
		fa.run()
	}
}

impl Foldable for ThunkBrand {
	/// Folds the Thunk from the right.
	///
	/// ### Type Signature
	///
	/// `forall fn_brand b a. Foldable Thunk => ((a, b) -> b, b, Thunk a) -> b`
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
	/// * `fa`: The Thunk to fold.
	///
	/// ### Returns
	///
	/// The final accumulator value.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, classes::*, types::*};
	///
	/// let eval = Thunk::pure(10);
	/// let result = ThunkBrand::fold_right::<RcFnBrand, _, _, _>(|a, b| a + b, 5, eval);
	/// assert_eq!(result, 15);
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
		func(fa.run(), initial)
	}

	/// Folds the Thunk from the left.
	///
	/// ### Type Signature
	///
	/// `forall fn_brand b a. Foldable Thunk => ((b, a) -> b, b, Thunk a) -> b`
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
	/// * `fa`: The Thunk to fold.
	///
	/// ### Returns
	///
	/// The final accumulator value.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, classes::*, types::*};
	///
	/// let eval = Thunk::pure(10);
	/// let result = ThunkBrand::fold_left::<RcFnBrand, _, _, _>(|b, a| b + a, 5, eval);
	/// assert_eq!(result, 15);
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
		func(initial, fa.run())
	}

	/// Maps the value to a monoid and returns it.
	///
	/// ### Type Signature
	///
	/// `forall fn_brand m a. (Foldable Thunk, Monoid m) => ((a) -> m, Thunk a) -> m`
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
	/// * `fa`: The Thunk to fold.
	///
	/// ### Returns
	///
	/// The monoid value.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, classes::*, types::*};
	///
	/// let eval = Thunk::pure(10);
	/// let result = ThunkBrand::fold_map::<RcFnBrand, String, _, _>(|a| a.to_string(), eval);
	/// assert_eq!(result, "10");
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
		func(fa.run())
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
	/// * `a`: The first eval.
	/// * `b`: The second eval.
	///
	/// ### Returns
	///
	/// A new `Thunk` containing the combined result.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, classes::*, functions::*};
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

	/// Tests basic execution of Thunk.
	///
	/// Verifies that `Thunk::new` creates a computation that can be run to produce the expected value.
	#[test]
	fn test_basic_execution() {
		let eval = Thunk::new(|| 42);
		assert_eq!(eval.run(), 42);
	}

	/// Tests `Thunk::pure`.
	///
	/// Verifies that `Thunk::pure` creates a computation that returns the provided value.
	#[test]
	fn test_pure() {
		let eval = Thunk::pure(42);
		assert_eq!(eval.run(), 42);
	}

	/// Tests borrowing in Thunk.
	///
	/// Verifies that `Thunk` can capture references to values on the stack.
	#[test]
	fn test_borrowing() {
		let x = 42;
		let eval = Thunk::new(|| &x);
		assert_eq!(eval.run(), &42);
	}

	/// Tests `Thunk::map`.
	///
	/// Verifies that `map` transforms the result of the computation.
	#[test]
	fn test_map() {
		let eval = Thunk::pure(21).map(|x| x * 2);
		assert_eq!(eval.run(), 42);
	}

	/// Tests `Thunk::bind`.
	///
	/// Verifies that `bind` chains computations correctly.
	#[test]
	fn test_bind() {
		let eval = Thunk::pure(21).bind(|x| Thunk::pure(x * 2));
		assert_eq!(eval.run(), 42);
	}

	/// Tests `Thunk::defer`.
	///
	/// Verifies that `defer` allows creating an `Thunk` from a thunk that returns an `Thunk`.
	#[test]
	fn test_defer() {
		let eval = Thunk::defer(|| Thunk::pure(42));
		assert_eq!(eval.run(), 42);
	}

	/// Tests `From<Memo>`.
	#[test]
	fn test_eval_from_memo() {
		use crate::types::RcLazy;
		let memo = RcLazy::new(|| 42);
		let eval = Thunk::from(memo);
		assert_eq!(eval.run(), 42);
	}

	/// Tests the `Semigroup` implementation for `Thunk`.
	///
	/// Verifies that `append` correctly combines two evals.
	#[test]
	fn test_eval_semigroup() {
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
	fn test_eval_monoid() {
		use crate::classes::monoid::empty;
		let t: Thunk<String> = empty();
		assert_eq!(t.run(), "");
	}
}
