//! Implementations for [`Eval`], a deferred computation type.
//!
//! This module provides the [`Eval`] type, which represents a deferred computation that produces a value.
//! Unlike [`Task`](crate::types::Task), `Eval` is not stack-safe for deep recursion but supports higher-kinded types and borrowing.

use crate::{
	Apply,
	brands::EvalBrand,
	classes::{
		Defer, apply_first::ApplyFirst, apply_second::ApplySecond, cloneable_fn::CloneableFn,
		foldable::Foldable, functor::Functor, lift::Lift, monad_rec::MonadRec, monoid::Monoid,
		pointed::Pointed, runnable::Runnable, semiapplicative::Semiapplicative,
		semigroup::Semigroup, semimonad::Semimonad,
	},
	impl_kind,
	kinds::*,
	types::{Memo, MemoConfig, step::Step},
};

/// A deferred computation that produces a value of type `A`.
///
/// `Eval` is NOT memoized - each call to `run()` re-executes the computation.
/// This type exists to build computation chains without allocation overhead.
///
/// Unlike [`Task`](crate::types::Task), `Eval` does NOT require `'static` and CAN implement
/// HKT traits like [`Functor`], [`Semimonad`], etc.
///
/// ### Trade-offs vs Task
///
/// | Aspect         | `Eval<'a, A>`               | `Task<A>`                    |
/// |----------------|---------------------------|----------------------------|
/// | HKT compatible | ✅ Yes                    | ❌ No (requires `'static`) |
/// | Stack-safe     | ⚠️ Partial (tail_rec_m only) | ✅ Yes (unlimited)         |
/// | Lifetime       | `'a` (can borrow)         | `'static` only             |
/// | Use case       | Glue code, composition    | Deep recursion, pipelines  |
///
/// ### Algebraic Properties
///
/// `Eval` is a proper Monad:
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
/// let computation = Eval::new(|| 5)
///     .map(|x| x * 2)
///     .map(|x| x + 1);
///
/// // No computation has happened yet!
/// // Only when we call run() does it execute:
/// let result = computation.run();
/// assert_eq!(result, 11);
/// ```
pub struct Eval<'a, A> {
	thunk: Box<dyn FnOnce() -> A + 'a>,
}

impl<'a, A: 'a> Eval<'a, A> {
	/// Creates a new Eval from a thunk.
	///
	/// ### Type Signature
	///
	/// `forall a. (Unit -> a) -> Eval a`
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
	/// A new `Eval` instance.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::types::*;
	///
	/// let eval = Eval::new(|| 42);
	/// assert_eq!(eval.run(), 42);
	/// ```
	pub fn new<F>(f: F) -> Self
	where
		F: FnOnce() -> A + 'a,
	{
		Eval { thunk: Box::new(f) }
	}

	/// Returns a pure value (already computed).
	///
	/// ### Type Signature
	///
	/// `forall a. a -> Eval a`
	///
	/// ### Parameters
	///
	/// * `a`: The value to wrap.
	///
	/// ### Returns
	///
	/// A new `Eval` instance containing the value.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::types::*;
	///
	/// let eval = Eval::pure(42);
	/// assert_eq!(eval.run(), 42);
	/// ```
	pub fn pure(a: A) -> Self
	where
		A: 'a,
	{
		Eval::new(move || a)
	}

	/// Defers a computation that returns an Eval.
	///
	/// ### Type Signature
	///
	/// `forall a. (Unit -> Eval a) -> Eval a`
	///
	/// ### Type Parameters
	///
	/// * `F`: The type of the thunk.
	///
	/// ### Parameters
	///
	/// * `f`: The thunk that returns an `Eval`.
	///
	/// ### Returns
	///
	/// A new `Eval` instance.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::types::*;
	///
	/// let eval = Eval::defer(|| Eval::pure(42));
	/// assert_eq!(eval.run(), 42);
	/// ```
	pub fn defer<F>(f: F) -> Self
	where
		F: FnOnce() -> Eval<'a, A> + 'a,
	{
		Eval::new(move || f().run())
	}

	/// Monadic bind: chains computations.
	///
	/// Note: Each `bind` adds to the call stack. For deep recursion
	/// (>1000 levels), use [`Task`](crate::types::Task) instead.
	///
	/// ### Type Signature
	///
	/// `forall b a. (a -> Eval b, Eval a) -> Eval b`
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
	/// A new `Eval` instance representing the chained computation.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::types::*;
	///
	/// let eval = Eval::pure(21).bind(|x| Eval::pure(x * 2));
	/// assert_eq!(eval.run(), 42);
	/// ```
	pub fn bind<B: 'a, F>(
		self,
		f: F,
	) -> Eval<'a, B>
	where
		F: FnOnce(A) -> Eval<'a, B> + 'a,
	{
		Eval::new(move || {
			let a = (self.thunk)();
			let eval_b = f(a);
			(eval_b.thunk)()
		})
	}

	/// Functor map: transforms the result.
	///
	/// ### Type Signature
	///
	/// `forall b a. (a -> b, Eval a) -> Eval b`
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
	/// A new `Eval` instance with the transformed result.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::types::*;
	///
	/// let eval = Eval::pure(21).map(|x| x * 2);
	/// assert_eq!(eval.run(), 42);
	/// ```
	pub fn map<B: 'a, F>(
		self,
		f: F,
	) -> Eval<'a, B>
	where
		F: FnOnce(A) -> B + 'a,
	{
		Eval::new(move || f((self.thunk)()))
	}

	/// Forces evaluation and returns the result.
	///
	/// ### Type Signature
	///
	/// `forall a. Eval a -> a`
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
	/// let eval = Eval::pure(42);
	/// assert_eq!(eval.run(), 42);
	/// ```
	pub fn run(self) -> A {
		(self.thunk)()
	}
}

impl<'a, A, Config> From<Memo<'a, A, Config>> for Eval<'a, A>
where
	A: Clone + 'a,
	Config: MemoConfig,
{
	fn from(memo: Memo<'a, A, Config>) -> Self {
		Eval::new(move || memo.get().clone())
	}
}

impl_kind! {
	for EvalBrand {
		type Of<'a, A: 'a>: 'a = Eval<'a, A>;
	}
}

impl<'a, A: 'a> Defer<'a> for Eval<'a, A> {
	fn defer<FnBrand: 'a + CloneableFn>(f: <FnBrand as CloneableFn>::Of<'a, (), Self>) -> Self
	where
		Self: Sized,
	{
		Eval::defer(move || f(()))
	}
}

impl Functor for EvalBrand {
	/// Maps a function over the result of an Eval computation.
	///
	/// ### Type Signature
	///
	/// `forall b a. Functor Eval => (a -> b, Eval a) -> Eval b`
	///
	/// ### Type Parameters
	///
	/// * `B`: The type of the result of the transformation.
	/// * `A`: The type of the value inside the Eval.
	/// * `F`: The type of the transformation function.
	///
	/// ### Parameters
	///
	/// * `f`: The function to apply to the result of the computation.
	/// * `fa`: The Eval instance.
	///
	/// ### Returns
	///
	/// A new Eval instance with the transformed result.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, classes::*, types::*};
	///
	/// let eval = Eval::pure(10);
	/// let mapped = EvalBrand::map(|x| x * 2, eval);
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

impl Pointed for EvalBrand {
	/// Wraps a value in an Eval context.
	///
	/// ### Type Signature
	///
	/// `forall a. Pointed Eval => a -> Eval a`
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
	/// A new Eval instance containing the value.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, classes::*, types::*};
	///
	/// let eval: Eval<i32> = EvalBrand::pure(42);
	/// assert_eq!(eval.run(), 42);
	/// ```
	fn pure<'a, A: 'a>(a: A) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>) {
		Eval::pure(a)
	}
}

impl Lift for EvalBrand {
	/// Lifts a binary function into the Eval context.
	///
	/// ### Type Signature
	///
	/// `forall c a b. Lift Eval => ((a, b) -> c, Eval a, Eval b) -> Eval c`
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
	/// * `fa`: The first Eval.
	/// * `fb`: The second Eval.
	///
	/// ### Returns
	///
	/// A new Eval instance containing the result of applying the function.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, classes::*, types::*};
	///
	/// let eval1 = Eval::pure(10);
	/// let eval2 = Eval::pure(20);
	/// let result = EvalBrand::lift2(|a, b| a + b, eval1, eval2);
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

impl ApplyFirst for EvalBrand {}
impl ApplySecond for EvalBrand {}

impl Semiapplicative for EvalBrand {
	/// Applies a function wrapped in Eval to a value wrapped in Eval.
	///
	/// ### Type Signature
	///
	/// `forall fn_brand b a. Semiapplicative Eval => (Eval (fn_brand a b), Eval a) -> Eval b`
	///
	/// ### Type Parameters
	///
	/// * `FnBrand`: The brand of the function wrapper.
	/// * `B`: The type of the result.
	/// * `A`: The type of the input.
	///
	/// ### Parameters
	///
	/// * `ff`: The Eval containing the function.
	/// * `fa`: The Eval containing the value.
	///
	/// ### Returns
	///
	/// A new Eval instance containing the result of applying the function.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, classes::*, types::*, functions::*};
	///
	/// let func = Eval::pure(cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2));
	/// let val = Eval::pure(21);
	/// let result = EvalBrand::apply::<RcFnBrand, _, _>(func, val);
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

impl Semimonad for EvalBrand {
	/// Chains Eval computations.
	///
	/// ### Type Signature
	///
	/// `forall b a. Semimonad Eval => (Eval a, a -> Eval b) -> Eval b`
	///
	/// ### Type Parameters
	///
	/// * `B`: The type of the result of the new computation.
	/// * `A`: The type of the result of the first computation.
	/// * `F`: The type of the function to apply.
	///
	/// ### Parameters
	///
	/// * `ma`: The first Eval.
	/// * `f`: The function to apply to the result of the computation.
	///
	/// ### Returns
	///
	/// A new Eval instance representing the chained computation.
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
	/// let eval = Eval::pure(10);
	/// let result = EvalBrand::bind(eval, |x| Eval::pure(x * 2));
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

impl MonadRec for EvalBrand {
	/// Performs tail-recursive monadic computation.
	///
	/// ### Type Signature
	///
	/// `forall m b a. MonadRec Eval => (a -> Eval (Step a b), a) -> Eval b`
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
	/// let result = EvalBrand::tail_rec_m(
	///     |x| Eval::pure(if x < 1000 { Step::Loop(x + 1) } else { Step::Done(x) }),
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
		Eval::new(move || {
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

impl Runnable for EvalBrand {
	/// Runs the eval, producing the inner value.
	///
	/// ### Type Signature
	///
	/// `forall a. Runnable Eval => Eval a -> a`
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
	/// let eval = Eval::new(|| 42);
	/// assert_eq!(EvalBrand::run(eval), 42);
	/// ```
	fn run<'a, A: 'a>(fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)) -> A {
		fa.run()
	}
}

impl Foldable for EvalBrand {
	/// Folds the Eval from the right.
	///
	/// ### Type Signature
	///
	/// `forall fn_brand b a. Foldable Eval => ((a, b) -> b, b, Eval a) -> b`
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
	/// * `fa`: The Eval to fold.
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
	/// let eval = Eval::pure(10);
	/// let result = EvalBrand::fold_right::<RcFnBrand, _, _, _>(|a, b| a + b, 5, eval);
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

	/// Folds the Eval from the left.
	///
	/// ### Type Signature
	///
	/// `forall fn_brand b a. Foldable Eval => ((b, a) -> b, b, Eval a) -> b`
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
	/// * `fa`: The Eval to fold.
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
	/// let eval = Eval::pure(10);
	/// let result = EvalBrand::fold_left::<RcFnBrand, _, _, _>(|b, a| b + a, 5, eval);
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
	/// `forall fn_brand m a. (Foldable Eval, Monoid m) => ((a) -> m, Eval a) -> m`
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
	/// * `fa`: The Eval to fold.
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
	/// let eval = Eval::pure(10);
	/// let result = EvalBrand::fold_map::<RcFnBrand, String, _, _>(|a| a.to_string(), eval);
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

impl<'a, A: Semigroup + 'a> Semigroup for Eval<'a, A> {
	/// Combines two `Eval`s by combining their results.
	///
	/// ### Type Signature
	///
	/// `forall a. Semigroup a => (Eval a, Eval a) -> Eval a`
	///
	/// ### Parameters
	///
	/// * `a`: The first eval.
	/// * `b`: The second eval.
	///
	/// ### Returns
	///
	/// A new `Eval` containing the combined result.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, classes::*, functions::*};
	///
	/// let t1 = pure::<EvalBrand, _>("Hello".to_string());
	/// let t2 = pure::<EvalBrand, _>(" World".to_string());
	/// let t3 = Semigroup::append(t1, t2);
	/// assert_eq!(t3.run(), "Hello World");
	/// ```
	fn append(
		a: Self,
		b: Self,
	) -> Self {
		Eval::new(move || Semigroup::append(a.run(), b.run()))
	}
}

impl<'a, A: Monoid + 'a> Monoid for Eval<'a, A> {
	/// Returns the identity `Eval`.
	///
	/// ### Type Signature
	///
	/// `forall a. Monoid a => () -> Eval a`
	///
	/// ### Returns
	///
	/// A `Eval` producing the identity value of `A`.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{classes::*, types::*};
	///
	/// let t: Eval<String> = Monoid::empty();
	/// assert_eq!(t.run(), "");
	/// ```
	fn empty() -> Self {
		Eval::new(|| Monoid::empty())
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	/// Tests basic execution of Eval.
	///
	/// Verifies that `Eval::new` creates a computation that can be run to produce the expected value.
	#[test]
	fn test_basic_execution() {
		let eval = Eval::new(|| 42);
		assert_eq!(eval.run(), 42);
	}

	/// Tests `Eval::pure`.
	///
	/// Verifies that `Eval::pure` creates a computation that returns the provided value.
	#[test]
	fn test_pure() {
		let eval = Eval::pure(42);
		assert_eq!(eval.run(), 42);
	}

	/// Tests borrowing in Eval.
	///
	/// Verifies that `Eval` can capture references to values on the stack.
	#[test]
	fn test_borrowing() {
		let x = 42;
		let eval = Eval::new(|| &x);
		assert_eq!(eval.run(), &42);
	}

	/// Tests `Eval::map`.
	///
	/// Verifies that `map` transforms the result of the computation.
	#[test]
	fn test_map() {
		let eval = Eval::pure(21).map(|x| x * 2);
		assert_eq!(eval.run(), 42);
	}

	/// Tests `Eval::bind`.
	///
	/// Verifies that `bind` chains computations correctly.
	#[test]
	fn test_bind() {
		let eval = Eval::pure(21).bind(|x| Eval::pure(x * 2));
		assert_eq!(eval.run(), 42);
	}

	/// Tests `Eval::defer`.
	///
	/// Verifies that `defer` allows creating an `Eval` from a thunk that returns an `Eval`.
	#[test]
	fn test_defer() {
		let eval = Eval::defer(|| Eval::pure(42));
		assert_eq!(eval.run(), 42);
	}

	/// Tests `From<Memo>`.
	#[test]
	fn test_eval_from_memo() {
		use crate::types::RcMemo;
		let memo = RcMemo::new(|| 42);
		let eval = Eval::from(memo);
		assert_eq!(eval.run(), 42);
	}

	/// Tests the `Semigroup` implementation for `Eval`.
	///
	/// Verifies that `append` correctly combines two evals.
	#[test]
	fn test_eval_semigroup() {
		use crate::classes::semigroup::append;
		use crate::{brands::*, functions::*};
		let t1 = pure::<EvalBrand, _>("Hello".to_string());
		let t2 = pure::<EvalBrand, _>(" World".to_string());
		let t3 = append(t1, t2);
		assert_eq!(t3.run(), "Hello World");
	}

	/// Tests the `Monoid` implementation for `Eval`.
	///
	/// Verifies that `empty` returns the identity element.
	#[test]
	fn test_eval_monoid() {
		use crate::classes::monoid::empty;
		let t: Eval<String> = empty();
		assert_eq!(t.run(), "");
	}
}
