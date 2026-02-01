use crate::{
	Apply,
	brands::ThunkBrand,
	classes::{
		ApplyFirst, ApplySecond, CloneableFn, Deferrable, Evaluable, Foldable, Functor, Lift,
		MonadRec, Monoid, Pointed, Semiapplicative, Semigroup, Semimonad,
	},
	impl_kind,
	kinds::*,
	types::{Lazy, LazyConfig, step::Step},
};
use fp_macros::doc_params;
use fp_macros::doc_type_params;
use fp_macros::hm_signature;

/// A deferred computation that produces a value of type `A`.
///
/// `Thunk` is NOT memoized - each call to [`Thunk::evaluate`] re-executes the computation.
/// This type exists to build computation chains without allocation overhead.
///
/// Unlike [`Trampoline`](crate::types::Trampoline), `Thunk` does NOT require `'static` and CAN implement
/// HKT traits like [`Functor`], [`Semimonad`], etc.
///
/// ### Trade-offs vs `Trampoline`
///
/// | Aspect         | `Thunk<'a, A>`              | `Trampoline<A>`            |
/// |----------------|-----------------------------|----------------------------|
/// | HKT compatible | ✅ Yes                      | ❌ No (requires `'static`) |
/// | Stack-safe     | ⚠️ Partial (tail_rec_m only) | ✅ Yes (unlimited)         |
/// | Lifetime       | `'a` (can borrow)           | `'static` only             |
/// | Use case       | Glue code, composition      | Deep recursion, pipelines  |
///
/// ### Algebraic Properties
///
/// `Thunk` is a proper Monad:
/// - `pure(a).evaluate() == a` (left identity).
/// - `thunk.bind(pure) == thunk` (right identity).
/// - `thunk.bind(f).bind(g) == thunk.bind(|a| f(a).bind(g))` (associativity).
///
/// ### Type Parameters
///
/// * `A`: The type of the value produced by the computation.
///
/// ### Fields
///
/// * `0`: The closure that performs the computation.
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
/// // Only when we call evaluate() does it execute:
/// let result = computation.evaluate();
/// assert_eq!(result, 11);
/// ```
pub struct Thunk<'a, A>(Box<dyn FnOnce() -> A + 'a>);

impl<'a, A: 'a> Thunk<'a, A> {
	/// Creates a new `Thunk` from a thunk.
	///
	/// ### Type Signature
	///
	/// `forall a. (Unit -> a) -> Thunk a`
	///
	/// ### Type Parameters
	///
	#[doc_type_params("The type of the thunk.")]
	///
	/// ### Parameters
	///
	#[doc_params("The thunk to wrap.")]
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
	/// let thunk = Thunk::new(|| 42);
	/// assert_eq!(thunk.evaluate(), 42);
	/// ```
	pub fn new<F>(f: F) -> Self
	where
		F: FnOnce() -> A + 'a,
	{
		Thunk(Box::new(f))
	}

	/// Returns a pure value (already computed).
	///
	/// ### Type Signature
	///
	/// `forall a. a -> Thunk a`
	///
	/// ### Parameters
	///
	#[doc_params("The value to wrap.")]
	///
	/// ### Returns
	///
	/// A new `Thunk` instance containing the value.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, functions::*, classes::*};
	///
	/// let thunk = pure::<ThunkBrand, _>(42);
	/// assert_eq!(thunk.evaluate(), 42);
	/// ```
	pub fn pure(a: A) -> Self
	where
		A: 'a,
	{
		Thunk::new(move || a)
	}

	/// Defers a computation that returns a Thunk.
	///
	/// ### Type Signature
	///
	/// `forall a. (Unit -> Thunk a) -> Thunk a`
	///
	/// ### Type Parameters
	///
	#[doc_type_params("The type of the thunk.")]
	///
	/// ### Parameters
	///
	#[doc_params("The thunk that returns a `Thunk`.")]
	///
	/// ### Returns
	///
	/// A new `Thunk` instance.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, functions::*, types::*};
	///
	/// let thunk = Thunk::defer(|| pure::<ThunkBrand, _>(42));
	/// assert_eq!(thunk.evaluate(), 42);
	/// ```
	pub fn defer<F>(f: F) -> Self
	where
		F: FnOnce() -> Thunk<'a, A> + 'a,
	{
		Thunk::new(move || f().evaluate())
	}

	/// Monadic bind: chains computations.
	///
	/// Note: Each `bind` adds to the call stack. For deep recursion,
	/// use [`Trampoline`](crate::types::Trampoline) instead.
	///
	/// ### Type Signature
	///
	/// `forall b a. (a -> Thunk b, Thunk a) -> Thunk b`
	///
	/// ### Type Parameters
	///
	#[doc_type_params(
		"The type of the result of the new computation.",
		"The type of the function to apply."
	)]
	///
	/// ### Parameters
	///
	#[doc_params("The function to apply to the result of the computation.")]
	///
	/// ### Returns
	///
	/// A new `Thunk` instance representing the chained computation.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, functions::*};
	///
	/// let thunk = pure::<ThunkBrand, _>(21).bind(|x| pure::<ThunkBrand, _>(x * 2));
	/// assert_eq!(thunk.evaluate(), 42);
	/// ```
	pub fn bind<B: 'a, F>(
		self,
		f: F,
	) -> Thunk<'a, B>
	where
		F: FnOnce(A) -> Thunk<'a, B> + 'a,
	{
		Thunk::new(move || {
			let a = (self.0)();
			let thunk_b = f(a);
			(thunk_b.0)()
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
	#[doc_type_params(
		"The type of the result of the transformation.",
		"The type of the transformation function."
	)]
	///
	/// ### Parameters
	///
	#[doc_params("The function to apply to the result of the computation.")]
	///
	/// ### Returns
	///
	/// A new `Thunk` instance with the transformed result.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, functions::*};
	///
	/// let thunk = pure::<ThunkBrand, _>(21).map(|x| x * 2);
	/// assert_eq!(thunk.evaluate(), 42);
	/// ```
	pub fn map<B: 'a, F>(
		self,
		f: F,
	) -> Thunk<'a, B>
	where
		F: FnOnce(A) -> B + 'a,
	{
		Thunk::new(move || f((self.0)()))
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
	/// use fp_library::{brands::*, functions::*};
	///
	/// let thunk = pure::<ThunkBrand, _>(42);
	/// assert_eq!(thunk.evaluate(), 42);
	/// ```
	pub fn evaluate(self) -> A {
		(self.0)()
	}
}

impl<'a, A, Config> From<Lazy<'a, A, Config>> for Thunk<'a, A>
where
	A: Clone + 'a,
	Config: LazyConfig,
{
	fn from(lazy: Lazy<'a, A, Config>) -> Self {
		Thunk::new(move || lazy.evaluate().clone())
	}
}

impl_kind! {
	for ThunkBrand {
		type Of<'a, A: 'a>: 'a = Thunk<'a, A>;
	}
}

impl<'a, A: 'a> Deferrable<'a> for Thunk<'a, A> {
	fn defer<FnBrand: 'a + CloneableFn>(f: <FnBrand as CloneableFn>::Of<'a, (), Self>) -> Self
	where
		Self: Sized,
	{
		Thunk::defer(move || f(()))
	}
}

impl Functor for ThunkBrand {
	/// Maps a function over the result of a `Thunk` computation.
	///
	/// ### Type Signature
	///
	#[hm_signature(Functor)]
	///
	/// ### Type Parameters
	///
	#[doc_type_params(
		"The lifetime of the computation.",
		"The type of the value inside the `Thunk`.",
		"The type of the result of the transformation.",
		"The type of the transformation function."
	)]
	///
	/// ### Parameters
	///
	#[doc_params(
		"The function to apply to the result of the computation.",
		"The `Thunk` instance."
	)]
	///
	/// ### Returns
	///
	/// A new `Thunk` instance with the transformed result.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, functions::*};
	///
	/// let thunk = pure::<ThunkBrand, _>(10);
	/// let mapped = map::<ThunkBrand, _, _, _>(|x| x * 2, thunk);
	/// assert_eq!(mapped.evaluate(), 20);
	/// ```
	fn map<'a, A: 'a, B: 'a, Func>(
		func: Func,
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)
	where
		Func: Fn(A) -> B + 'a,
	{
		fa.map(func)
	}
}

impl Pointed for ThunkBrand {
	/// Wraps a value in a `Thunk` context.
	///
	/// ### Type Signature
	///
	#[hm_signature(Pointed)]
	///
	/// ### Type Parameters
	///
	#[doc_type_params("The lifetime of the computation.", "The type of the value to wrap.")]
	///
	/// ### Parameters
	///
	#[doc_params("The value to wrap.")]
	///
	/// ### Returns
	///
	/// A new `Thunk` instance containing the value.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, functions::*, types::*};
	///
	/// let thunk: Thunk<i32> = pure::<ThunkBrand, _>(42);
	/// assert_eq!(thunk.evaluate(), 42);
	/// ```
	fn pure<'a, A: 'a>(a: A) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>) {
		Thunk::pure(a)
	}
}

impl Lift for ThunkBrand {
	/// Lifts a binary function into the `Thunk` context.
	///
	/// ### Type Signature
	///
	#[hm_signature(Lift)]
	///
	/// ### Type Parameters
	///
	#[doc_type_params(
		"The lifetime of the computation.",
		"The type of the first value.",
		"The type of the second value.",
		"The type of the result.",
		"The type of the binary function."
	)]
	///
	/// ### Parameters
	///
	#[doc_params("The binary function to apply.", "The first `Thunk`.", "The second `Thunk`.")]
	///
	/// ### Returns
	///
	/// A new `Thunk` instance containing the result of applying the function.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, functions::*};
	///
	/// let eval1 = pure::<ThunkBrand, _>(10);
	/// let eval2 = pure::<ThunkBrand, _>(20);
	/// let result = lift2::<ThunkBrand, _, _, _, _>(|a, b| a + b, eval1, eval2);
	/// assert_eq!(result.evaluate(), 30);
	/// ```
	fn lift2<'a, A, B, C, Func>(
		func: Func,
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		fb: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>),
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, C>)
	where
		Func: Fn(A, B) -> C + 'a,
		A: Clone + 'a,
		B: Clone + 'a,
		C: 'a,
	{
		fa.bind(move |a| fb.map(move |b| func(a, b)))
	}
}

impl ApplyFirst for ThunkBrand {}
impl ApplySecond for ThunkBrand {}

impl Semiapplicative for ThunkBrand {
	/// Applies a function wrapped in `Thunk` to a value wrapped in `Thunk`.
	///
	/// ### Type Signature
	///
	#[hm_signature(Semiapplicative)]
	///
	/// ### Type Parameters
	///
	#[doc_type_params(
		"The lifetime of the computation.",
		"The brand of the cloneable function wrapper.",
		"The type of the input.",
		"The type of the result."
	)]
	///
	/// ### Parameters
	///
	#[doc_params("The `Thunk` containing the function.", "The `Thunk` containing the value.")]
	///
	/// ### Returns
	///
	/// A new `Thunk` instance containing the result of applying the function.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, functions::*};
	///
	/// let func = pure::<ThunkBrand, _>(cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2));
	/// let val = pure::<ThunkBrand, _>(21);
	/// let result = apply::<RcFnBrand, ThunkBrand, _, _>(func, val);
	/// assert_eq!(result.evaluate(), 42);
	/// ```
	fn apply<'a, FnBrand: 'a + CloneableFn, A: 'a + Clone, B: 'a>(
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
	/// Chains `Thunk` computations.
	///
	/// ### Type Signature
	///
	#[hm_signature(Semimonad)]
	///
	/// ### Type Parameters
	///
	#[doc_type_params(
		"The lifetime of the computation.",
		"The type of the result of the first computation.",
		"The type of the result of the new computation.",
		"The type of the function to apply."
	)]
	///
	/// ### Parameters
	///
	#[doc_params("The first `Thunk`.", "The function to apply to the result of the computation.")]
	///
	/// ### Returns
	///
	/// A new `Thunk` instance representing the chained computation.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, functions::*};
	///
	/// let thunk = pure::<ThunkBrand, _>(10);
	/// let result = bind::<ThunkBrand, _, _, _>(thunk, |x| pure::<ThunkBrand, _>(x * 2));
	/// assert_eq!(result.evaluate(), 20);
	/// ```
	fn bind<'a, A: 'a, B: 'a, Func>(
		ma: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		func: Func,
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)
	where
		Func: Fn(A) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) + 'a,
	{
		ma.bind(func)
	}
}

impl MonadRec for ThunkBrand {
	/// Performs tail-recursive monadic computation.
	///
	/// ### Type Signature
	///
	#[hm_signature(MonadRec)]
	///
	/// ### Type Parameters
	///
	#[doc_type_params(
		"The lifetime of the computation.",
		"The type of the initial value and loop state.",
		"The type of the result.",
		"The type of the step function."
	)]
	///
	/// ### Parameters
	///
	#[doc_params("The step function.", "The initial value.")]
	///
	/// ### Returns
	///
	/// The result of the computation.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, classes::*, functions::*, types::*};
	///
	/// let result = tail_rec_m::<ThunkBrand, _, _, _>(
	///     |x| pure::<ThunkBrand, _>(if x < 1000 { Step::Loop(x + 1) } else { Step::Done(x) }),
	///     0,
	/// );
	/// assert_eq!(result.evaluate(), 1000);
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
				match f(current).evaluate() {
					Step::Loop(next) => current = next,
					Step::Done(res) => break res,
				}
			}
		})
	}
}

impl Evaluable for ThunkBrand {
	/// Runs the eval, producing the inner value.
	///
	/// ### Type Signature
	///
	#[hm_signature(Runnable)]
	///
	/// ### Type Parameters
	///
	#[doc_type_params(
		"The lifetime of the computation.",
		"The type of the value inside the thunk."
	)]
	///
	/// ### Parameters
	///
	#[doc_params("The eval to run.")]
	///
	/// ### Returns
	///
	/// The result of running the thunk.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, classes::*, functions::*, types::*};
	///
	/// let thunk = Thunk::new(|| 42);
	/// assert_eq!(evaluate::<ThunkBrand, _>(thunk), 42);
	/// ```
	fn evaluate<'a, A: 'a>(fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)) -> A {
		fa.evaluate()
	}
}

impl Foldable for ThunkBrand {
	/// Folds the `Thunk` from the right.
	///
	/// ### Type Signature
	///
	#[hm_signature(Foldable)]
	///
	/// ### Type Parameters
	///
	#[doc_type_params(
		"The lifetime of the computation.",
		"The brand of the cloneable function to use.",
		"The type of the elements in the structure.",
		"The type of the accumulator.",
		"The type of the folding function."
	)]
	///
	/// ### Parameters
	///
	#[doc_params(
		"The function to apply to each element and the accumulator.",
		"The initial value of the accumulator.",
		"The `Thunk` to fold."
	)]
	///
	/// ### Returns
	///
	/// The final accumulator value.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, functions::*};
	///
	/// let thunk = pure::<ThunkBrand, _>(10);
	/// let result = fold_right::<RcFnBrand, ThunkBrand, _, _, _>(|a, b| a + b, 5, thunk);
	/// assert_eq!(result, 15);
	/// ```
	fn fold_right<'a, FnBrand, A: 'a, B: 'a, Func>(
		func: Func,
		initial: B,
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> B
	where
		Func: Fn(A, B) -> B + 'a,
		FnBrand: CloneableFn + 'a,
	{
		func(fa.evaluate(), initial)
	}

	/// Folds the `Thunk` from the left.
	///
	/// ### Type Signature
	///
	#[hm_signature(Foldable)]
	///
	/// ### Type Parameters
	///
	#[doc_type_params(
		"The lifetime of the computation.",
		"The brand of the cloneable function to use.",
		"The type of the elements in the structure.",
		"The type of the accumulator.",
		"The type of the folding function."
	)]
	///
	/// ### Parameters
	///
	#[doc_params(
		"The function to apply to the accumulator and each element.",
		"The initial value of the accumulator.",
		"The `Thunk` to fold."
	)]
	///
	/// ### Returns
	///
	/// The final accumulator value.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, functions::*};
	///
	/// let thunk = pure::<ThunkBrand, _>(10);
	/// let result = fold_left::<RcFnBrand, ThunkBrand, _, _, _>(|b, a| b + a, 5, thunk);
	/// assert_eq!(result, 15);
	/// ```
	fn fold_left<'a, FnBrand, A: 'a, B: 'a, Func>(
		func: Func,
		initial: B,
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> B
	where
		Func: Fn(B, A) -> B + 'a,
		FnBrand: CloneableFn + 'a,
	{
		func(initial, fa.evaluate())
	}

	/// Maps the value to a monoid and returns it.
	///
	/// ### Type Signature
	///
	#[hm_signature(Foldable)]
	///
	/// ### Type Parameters
	///
	#[doc_type_params(
		"The lifetime of the computation.",
		"The brand of the cloneable function to use.",
		"The type of the elements in the structure.",
		"The type of the monoid.",
		"The type of the mapping function."
	)]
	///
	/// ### Parameters
	///
	#[doc_params("The mapping function.", "The Thunk to fold.")]
	///
	/// ### Returns
	///
	/// The monoid value.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, functions::*};
	///
	/// let thunk = pure::<ThunkBrand, _>(10);
	/// let result = fold_map::<RcFnBrand, ThunkBrand, _, _, _>(|a| a.to_string(), thunk);
	/// assert_eq!(result, "10");
	/// ```
	fn fold_map<'a, FnBrand, A: 'a, M, Func>(
		func: Func,
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> M
	where
		M: Monoid + 'a,
		Func: Fn(A) -> M + 'a,
		FnBrand: CloneableFn + 'a,
	{
		func(fa.evaluate())
	}
}

impl<'a, A: Semigroup + 'a> Semigroup for Thunk<'a, A> {
	/// Combines two `Thunk`s by combining their results.
	///
	/// ### Type Signature
	///
	#[hm_signature(Semigroup)]
	///
	/// ### Parameters
	///
	#[doc_params("The first `Thunk`.", "The second `Thunk`.")]
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
	/// let t3 = append::<_>(t1, t2);
	/// assert_eq!(t3.evaluate(), "Hello World");
	/// ```
	fn append(
		a: Self,
		b: Self,
	) -> Self {
		Thunk::new(move || Semigroup::append(a.evaluate(), b.evaluate()))
	}
}

impl<'a, A: Monoid + 'a> Monoid for Thunk<'a, A> {
	/// Returns the identity `Thunk`.
	///
	/// ### Type Signature
	///
	#[hm_signature(Monoid)]
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
	/// let t: Thunk<String> = Thunk::empty();
	/// assert_eq!(t.evaluate(), "");
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
		let thunk = Thunk::new(|| 42);
		assert_eq!(thunk.evaluate(), 42);
	}

	/// Tests `Thunk::pure`.
	///
	/// Verifies that `Thunk::pure` creates a computation that returns the provided value.
	#[test]
	fn test_pure() {
		let thunk = Thunk::pure(42);
		assert_eq!(thunk.evaluate(), 42);
	}

	/// Tests borrowing in Thunk.
	///
	/// Verifies that `Thunk` can capture references to values on the stack.
	#[test]
	fn test_borrowing() {
		let x = 42;
		let thunk = Thunk::new(|| &x);
		assert_eq!(thunk.evaluate(), &42);
	}

	/// Tests `Thunk::map`.
	///
	/// Verifies that `map` transforms the result of the computation.
	#[test]
	fn test_map() {
		let thunk = Thunk::pure(21).map(|x| x * 2);
		assert_eq!(thunk.evaluate(), 42);
	}

	/// Tests `Thunk::bind`.
	///
	/// Verifies that `bind` chains computations correctly.
	#[test]
	fn test_bind() {
		let thunk = Thunk::pure(21).bind(|x| Thunk::pure(x * 2));
		assert_eq!(thunk.evaluate(), 42);
	}

	/// Tests `Thunk::defer`.
	///
	/// Verifies that `defer` allows creating an `Thunk` from a thunk that returns an `Thunk`.
	#[test]
	fn test_defer() {
		let thunk = Thunk::defer(|| Thunk::pure(42));
		assert_eq!(thunk.evaluate(), 42);
	}

	/// Tests `From<Lazy>`.
	#[test]
	fn test_eval_from_memo() {
		use crate::types::RcLazy;
		let memo = RcLazy::new(|| 42);
		let thunk = Thunk::from(memo);
		assert_eq!(thunk.evaluate(), 42);
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
		assert_eq!(t3.evaluate(), "Hello World");
	}

	/// Tests the `Monoid` implementation for `Thunk`.
	///
	/// Verifies that `empty` returns the identity element.
	#[test]
	fn test_eval_monoid() {
		use crate::classes::monoid::empty;
		let t: Thunk<String> = empty();
		assert_eq!(t.evaluate(), "");
	}
}
