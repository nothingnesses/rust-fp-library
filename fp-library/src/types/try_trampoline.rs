//! Stack-safe fallible computation type with guaranteed safety for unlimited recursion depth.
//!
//! Wraps [`Trampoline<Result<A, E>>`](crate::types::Trampoline) with ergonomic combinators for error handling. Provides complete stack safety for fallible computations that may recurse deeply.
//!
//! ### Examples
//!
//! ```
//! use fp_library::types::*;
//!
//! let task: TryTrampoline<i32, String> =
//! 	TryTrampoline::ok(10).map(|x| x * 2).bind(|x| TryTrampoline::ok(x + 5));
//!
//! assert_eq!(task.evaluate(), Ok(25));
//! ```

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			classes::{
				Deferrable,
				Monoid,
				Semigroup,
			},
			types::{
				Lazy,
				LazyConfig,
				Step,
				Trampoline,
				TryLazy,
				TryLazyConfig,
			},
		},
		fp_macros::*,
		std::fmt,
	};

	/// A lazy, stack-safe computation that may fail with an error.
	///
	/// This is [`Trampoline<Result<A, E>>`] with ergonomic combinators.
	///
	/// ### When to Use
	///
	/// Use `TryTrampoline` for stack-safe fallible recursion. It provides unlimited recursion
	/// depth without stack overflow, but requires `'static` types and does not have HKT brands.
	/// For lightweight fallible deferred computation with HKT support, use
	/// [`TryThunk`](crate::types::TryThunk). For memoized fallible computation, use
	/// [`TryLazy`](crate::types::TryLazy).
	#[document_type_parameters("The type of the success value.", "The type of the error value.")]
	///
	pub struct TryTrampoline<A: 'static, E: 'static>(
		/// The internal `Trampoline` wrapping a `Result`.
		Trampoline<Result<A, E>>,
	);

	#[document_type_parameters("The type of the success value.", "The type of the error value.")]
	#[document_parameters("The fallible trampoline computation.")]
	impl<A: 'static, E: 'static> TryTrampoline<A, E> {
		/// Creates a successful `TryTrampoline`.
		#[document_signature]
		///
		#[document_parameters("The success value.")]
		///
		#[document_returns("A `TryTrampoline` representing success.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let task: TryTrampoline<i32, String> = TryTrampoline::ok(42);
		/// assert_eq!(task.evaluate(), Ok(42));
		/// ```
		#[inline]
		pub fn ok(a: A) -> Self {
			TryTrampoline(Trampoline::pure(Ok(a)))
		}

		/// Creates a successful `TryTrampoline`.
		///
		/// This is an alias for [`ok`](TryTrampoline::ok), provided for consistency
		/// with other types in the library.
		#[document_signature]
		///
		#[document_parameters("The success value.")]
		///
		#[document_returns("A `TryTrampoline` representing success.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let task: TryTrampoline<i32, String> = TryTrampoline::pure(42);
		/// assert_eq!(task.evaluate(), Ok(42));
		/// ```
		pub fn pure(a: A) -> Self {
			Self::ok(a)
		}

		/// Unwraps the newtype to expose the inner `Trampoline<Result<A, E>>`.
		#[document_signature]
		///
		#[document_returns("The inner `Trampoline<Result<A, E>>`.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let task: TryTrampoline<i32, String> = TryTrampoline::ok(42);
		/// let inner: Trampoline<Result<i32, String>> = task.into_inner();
		/// assert_eq!(inner.evaluate(), Ok(42));
		/// ```
		pub fn into_inner(self) -> Trampoline<Result<A, E>> {
			self.0
		}

		/// Creates a failed `TryTrampoline`.
		#[document_signature]
		///
		#[document_parameters("The error value.")]
		///
		#[document_returns("A `TryTrampoline` representing failure.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let task: TryTrampoline<i32, String> = TryTrampoline::err("error".to_string());
		/// assert_eq!(task.evaluate(), Err("error".to_string()));
		/// ```
		#[inline]
		pub fn err(e: E) -> Self {
			TryTrampoline(Trampoline::pure(Err(e)))
		}

		/// Creates a lazy `TryTrampoline` that may fail.
		#[document_signature]
		///
		#[document_parameters("The closure to execute.")]
		///
		#[document_returns("A `TryTrampoline` that executes `f` when run.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let task: TryTrampoline<i32, String> = TryTrampoline::new(|| Ok(42));
		/// assert_eq!(task.evaluate(), Ok(42));
		/// ```
		#[inline]
		pub fn new(f: impl FnOnce() -> Result<A, E> + 'static) -> Self {
			TryTrampoline(Trampoline::new(f))
		}

		/// Defers the construction of a `TryTrampoline`.
		///
		/// Use this for stack-safe recursion.
		#[document_signature]
		///
		#[document_parameters("A thunk that returns the next step.")]
		///
		#[document_returns("A `TryTrampoline` that executes `f` to get the next step.")]
		///
		#[document_examples]
		///
		/// Stack-safe recursion:
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// fn factorial(
		/// 	n: i32,
		/// 	acc: i32,
		/// ) -> TryTrampoline<i32, String> {
		/// 	if n < 0 {
		/// 		TryTrampoline::err("Negative input".to_string())
		/// 	} else if n == 0 {
		/// 		TryTrampoline::ok(acc)
		/// 	} else {
		/// 		TryTrampoline::defer(move || factorial(n - 1, n * acc))
		/// 	}
		/// }
		///
		/// let task = factorial(5, 1);
		/// assert_eq!(task.evaluate(), Ok(120));
		/// ```
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let task: TryTrampoline<i32, String> = TryTrampoline::defer(|| TryTrampoline::ok(42));
		/// assert_eq!(task.evaluate(), Ok(42));
		/// ```
		#[inline]
		pub fn defer(f: impl FnOnce() -> TryTrampoline<A, E> + 'static) -> Self {
			TryTrampoline(Trampoline::defer(move || f().0))
		}

		/// Maps over the success value.
		#[document_signature]
		///
		#[document_type_parameters("The type of the new success value.")]
		///
		#[document_parameters("The function to apply to the success value.")]
		///
		#[document_returns("A new `TryTrampoline` with the transformed success value.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let task: TryTrampoline<i32, String> = TryTrampoline::ok(10).map(|x| x * 2);
		/// assert_eq!(task.evaluate(), Ok(20));
		/// ```
		#[inline]
		pub fn map<B: 'static>(
			self,
			func: impl FnOnce(A) -> B + 'static,
		) -> TryTrampoline<B, E> {
			TryTrampoline(self.0.map(|result| result.map(func)))
		}

		/// Maps over the error value.
		#[document_signature]
		///
		#[document_type_parameters("The type of the new error value.")]
		///
		#[document_parameters("The function to apply to the error value.")]
		///
		#[document_returns("A new `TryTrampoline` with the transformed error value.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let task: TryTrampoline<i32, String> =
		/// 	TryTrampoline::err("error".to_string()).map_err(|e| e.to_uppercase());
		/// assert_eq!(task.evaluate(), Err("ERROR".to_string()));
		/// ```
		#[inline]
		pub fn map_err<E2: 'static>(
			self,
			func: impl FnOnce(E) -> E2 + 'static,
		) -> TryTrampoline<A, E2> {
			TryTrampoline(self.0.map(|result| result.map_err(func)))
		}

		/// Maps over both the success and error values simultaneously.
		#[document_signature]
		///
		#[document_type_parameters(
			"The type of the new success value.",
			"The type of the new error value."
		)]
		///
		#[document_parameters(
			"The function to apply to the success value.",
			"The function to apply to the error value."
		)]
		///
		#[document_returns("A new `TryTrampoline` with both sides transformed.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let ok_task: TryTrampoline<i32, String> = TryTrampoline::ok(10);
		/// let mapped = ok_task.bimap(|x| x * 2, |e| e.len());
		/// assert_eq!(mapped.evaluate(), Ok(20));
		///
		/// let err_task: TryTrampoline<i32, String> = TryTrampoline::err("hello".to_string());
		/// let mapped = err_task.bimap(|x| x * 2, |e| e.len());
		/// assert_eq!(mapped.evaluate(), Err(5));
		/// ```
		#[inline]
		pub fn bimap<B: 'static, F: 'static>(
			self,
			f: impl FnOnce(A) -> B + 'static,
			g: impl FnOnce(E) -> F + 'static,
		) -> TryTrampoline<B, F> {
			TryTrampoline(self.0.map(|result| match result {
				Ok(a) => Ok(f(a)),
				Err(e) => Err(g(e)),
			}))
		}

		/// Chains fallible computations.
		#[document_signature]
		///
		#[document_type_parameters("The type of the new success value.")]
		///
		#[document_parameters("The function to apply to the success value.")]
		///
		#[document_returns("A new `TryTrampoline` that chains `f` after this task.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let task: TryTrampoline<i32, String> = TryTrampoline::ok(10).bind(|x| TryTrampoline::ok(x * 2));
		/// assert_eq!(task.evaluate(), Ok(20));
		/// ```
		#[inline]
		pub fn bind<B: 'static>(
			self,
			f: impl FnOnce(A) -> TryTrampoline<B, E> + 'static,
		) -> TryTrampoline<B, E> {
			TryTrampoline(self.0.bind(|result| match result {
				Ok(a) => f(a).0,
				Err(e) => Trampoline::pure(Err(e)),
			}))
		}

		/// Recovers from an error.
		#[document_signature]
		///
		#[document_parameters("The function to apply to the error value.")]
		///
		#[document_returns("A new `TryTrampoline` that attempts to recover from failure.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let task: TryTrampoline<i32, String> =
		/// 	TryTrampoline::err("error".to_string()).catch(|_| TryTrampoline::ok(42));
		/// assert_eq!(task.evaluate(), Ok(42));
		/// ```
		#[inline]
		pub fn catch(
			self,
			f: impl FnOnce(E) -> TryTrampoline<A, E> + 'static,
		) -> Self {
			TryTrampoline(self.0.bind(|result| match result {
				Ok(a) => Trampoline::pure(Ok(a)),
				Err(e) => f(e).0,
			}))
		}

		/// Recovers from an error using a fallible recovery function that may produce a different error type.
		///
		/// Unlike [`catch`](TryTrampoline::catch), `catch_with` allows the recovery function to return a
		/// `TryTrampoline` with a different error type `E2`. On success, the value is passed through
		/// unchanged. On failure, the recovery function is applied to the error value and its
		/// resulting `TryTrampoline` is composed via `bind`, preserving stack safety through deeply chained recovery operations.
		#[document_signature]
		///
		#[document_type_parameters("The error type produced by the recovery computation.")]
		///
		#[document_parameters("The monadic recovery function applied to the error value.")]
		///
		#[document_returns(
			"A new `TryTrampoline` that either passes through the success value or uses the result of the recovery computation."
		)]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let recovered: TryTrampoline<i32, i32> =
		/// 	TryTrampoline::<i32, &str>::err("error").catch_with(|_| TryTrampoline::err(42));
		/// assert_eq!(recovered.evaluate(), Err(42));
		///
		/// let ok: TryTrampoline<i32, i32> =
		/// 	TryTrampoline::<i32, &str>::ok(1).catch_with(|_| TryTrampoline::err(42));
		/// assert_eq!(ok.evaluate(), Ok(1));
		/// ```
		#[inline]
		pub fn catch_with<E2: 'static>(
			self,
			f: impl FnOnce(E) -> TryTrampoline<A, E2> + 'static,
		) -> TryTrampoline<A, E2> {
			TryTrampoline(self.0.bind(move |result| match result {
				Ok(a) => Trampoline::pure(Ok(a)),
				Err(e) => f(e).0,
			}))
		}

		/// Combines two `TryTrampoline`s, running both and combining results.
		///
		/// Short-circuits on error: if `self` fails, `other` is never evaluated.
		#[document_signature]
		///
		#[document_type_parameters(
			"The type of the second computation's success value.",
			"The type of the combined result."
		)]
		///
		#[document_parameters("The second computation.", "The function to combine the results.")]
		///
		#[document_returns("A new `TryTrampoline` producing the combined result.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let t1: TryTrampoline<i32, String> = TryTrampoline::ok(10);
		/// let t2: TryTrampoline<i32, String> = TryTrampoline::ok(20);
		/// let t3 = t1.lift2(t2, |a, b| a + b);
		/// assert_eq!(t3.evaluate(), Ok(30));
		/// ```
		#[inline]
		pub fn lift2<B: 'static, C: 'static>(
			self,
			other: TryTrampoline<B, E>,
			f: impl FnOnce(A, B) -> C + 'static,
		) -> TryTrampoline<C, E> {
			self.bind(move |a| other.map(move |b| f(a, b)))
		}

		/// Sequences two `TryTrampoline`s, discarding the first result.
		///
		/// Short-circuits on error: if `self` fails, `other` is never evaluated.
		#[document_signature]
		///
		#[document_type_parameters("The type of the second computation's success value.")]
		///
		#[document_parameters("The second computation.")]
		///
		#[document_returns(
			"A new `TryTrampoline` that runs both computations and returns the result of the second."
		)]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let t1: TryTrampoline<i32, String> = TryTrampoline::ok(10);
		/// let t2: TryTrampoline<i32, String> = TryTrampoline::ok(20);
		/// let t3 = t1.then(t2);
		/// assert_eq!(t3.evaluate(), Ok(20));
		/// ```
		#[inline]
		pub fn then<B: 'static>(
			self,
			other: TryTrampoline<B, E>,
		) -> TryTrampoline<B, E> {
			self.bind(move |_| other)
		}

		/// Stack-safe tail recursion for fallible computations.
		///
		/// The step function returns `TryTrampoline<Step<S, A>, E>`, and the
		/// loop short-circuits on error.
		///
		/// # Clone Bound
		///
		/// The function `f` must implement `Clone` because each iteration
		/// of the recursion may need its own copy. Most closures naturally
		/// implement `Clone` when all their captures implement `Clone`.
		///
		/// For closures that don't implement `Clone`, use `arc_tail_rec_m`
		/// which wraps the closure in `Arc` internally.
		#[document_signature]
		///
		#[document_type_parameters("The type of the state.")]
		///
		#[document_parameters(
			"The function that performs one step of the recursion.",
			"The initial state."
		)]
		///
		#[document_returns("A `TryTrampoline` that performs the recursion.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::{
		/// 	Step,
		/// 	TryTrampoline,
		/// };
		///
		/// // Fallible factorial using tail recursion
		/// fn factorial(n: i32) -> TryTrampoline<i32, String> {
		/// 	TryTrampoline::tail_rec_m(
		/// 		|(n, acc)| {
		/// 			if n < 0 {
		/// 				TryTrampoline::err("Negative input".to_string())
		/// 			} else if n <= 1 {
		/// 				TryTrampoline::ok(Step::Done(acc))
		/// 			} else {
		/// 				TryTrampoline::ok(Step::Loop((n - 1, n * acc)))
		/// 			}
		/// 		},
		/// 		(n, 1),
		/// 	)
		/// }
		///
		/// assert_eq!(factorial(5).evaluate(), Ok(120));
		/// assert_eq!(factorial(-1).evaluate(), Err("Negative input".to_string()));
		/// ```
		pub fn tail_rec_m<S: 'static>(
			f: impl Fn(S) -> TryTrampoline<Step<S, A>, E> + Clone + 'static,
			initial: S,
		) -> Self {
			fn go<S: 'static, A: 'static, E: 'static>(
				f: impl Fn(S) -> TryTrampoline<Step<S, A>, E> + Clone + 'static,
				s: S,
			) -> Trampoline<Result<A, E>> {
				Trampoline::defer(move || {
					let result = f(s);
					result.0.bind(move |r| match r {
						Ok(Step::Loop(next)) => go(f, next),
						Ok(Step::Done(a)) => Trampoline::pure(Ok(a)),
						Err(e) => Trampoline::pure(Err(e)),
					})
				})
			}
			TryTrampoline(go(f, initial))
		}

		/// Arc-wrapped version of `tail_rec_m` for non-Clone closures.
		///
		/// Use this when your closure captures non-Clone state.
		#[document_signature]
		///
		#[document_type_parameters("The type of the state.")]
		///
		#[document_parameters(
			"The function that performs one step of the recursion.",
			"The initial state."
		)]
		///
		#[document_returns("A `TryTrampoline` that performs the recursion.")]
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	fp_library::types::{
		/// 		Step,
		/// 		TryTrampoline,
		/// 	},
		/// 	std::sync::{
		/// 		Arc,
		/// 		atomic::{
		/// 			AtomicUsize,
		/// 			Ordering,
		/// 		},
		/// 	},
		/// };
		///
		/// // Closure captures non-Clone state
		/// let counter = Arc::new(AtomicUsize::new(0));
		/// let counter_clone = Arc::clone(&counter);
		/// let task: TryTrampoline<i32, String> = TryTrampoline::arc_tail_rec_m(
		/// 	move |n| {
		/// 		counter_clone.fetch_add(1, Ordering::SeqCst);
		/// 		if n == 0 {
		/// 			TryTrampoline::ok(Step::Done(0))
		/// 		} else {
		/// 			TryTrampoline::ok(Step::Loop(n - 1))
		/// 		}
		/// 	},
		/// 	100,
		/// );
		/// assert_eq!(task.evaluate(), Ok(0));
		/// assert_eq!(counter.load(Ordering::SeqCst), 101);
		/// ```
		pub fn arc_tail_rec_m<S: 'static>(
			f: impl Fn(S) -> TryTrampoline<Step<S, A>, E> + 'static,
			initial: S,
		) -> Self {
			use std::sync::Arc;
			let f = Arc::new(f);
			let wrapper = move |s: S| {
				let f = Arc::clone(&f);
				f(s)
			};
			Self::tail_rec_m(wrapper, initial)
		}

		/// Runs the computation, returning the result.
		#[document_signature]
		///
		#[document_returns("The result of the computation.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let task: TryTrampoline<i32, String> = TryTrampoline::ok(42);
		/// assert_eq!(task.evaluate(), Ok(42));
		/// ```
		#[inline]
		pub fn evaluate(self) -> Result<A, E> {
			self.0.evaluate()
		}

		/// Combines two `TryTrampoline` values using the inner type's [`Semigroup`].
		///
		/// Both computations are evaluated. If both succeed, their results are
		/// combined via [`Semigroup::append`]. If either fails, the first error
		/// is propagated (short-circuiting on the left).
		#[document_signature]
		///
		#[document_parameters(
			"The second `TryTrampoline` whose result will be combined with this one."
		)]
		///
		#[document_returns("A new `TryTrampoline` producing the combined result.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let t1: TryTrampoline<Vec<i32>, String> = TryTrampoline::ok(vec![1, 2]);
		/// let t2: TryTrampoline<Vec<i32>, String> = TryTrampoline::ok(vec![3, 4]);
		/// assert_eq!(t1.append(t2).evaluate(), Ok(vec![1, 2, 3, 4]));
		/// ```
		#[inline]
		pub fn append(
			self,
			other: TryTrampoline<A, E>,
		) -> TryTrampoline<A, E>
		where
			A: Semigroup + 'static, {
			self.lift2(other, Semigroup::append)
		}

		/// Creates a `TryTrampoline` that produces the identity element for the given [`Monoid`].
		#[document_signature]
		///
		#[document_returns(
			"A `TryTrampoline` producing the monoid identity element wrapped in `Ok`."
		)]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let t: TryTrampoline<Vec<i32>, String> = TryTrampoline::empty();
		/// assert_eq!(t.evaluate(), Ok(Vec::<i32>::new()));
		/// ```
		#[inline]
		pub fn empty() -> TryTrampoline<A, E>
		where
			A: Monoid + 'static, {
			TryTrampoline::ok(Monoid::empty())
		}

		/// Converts this `TryTrampoline` into a memoized [`RcTryLazy`](crate::types::RcTryLazy) value.
		///
		/// The computation will be evaluated at most once; subsequent accesses
		/// return the cached result.
		#[document_signature]
		///
		#[document_returns(
			"A memoized `RcTryLazy` value that evaluates this trampoline on first access."
		)]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let task: TryTrampoline<i32, String> = TryTrampoline::ok(42);
		/// let lazy = task.into_rc_try_lazy();
		/// assert_eq!(lazy.evaluate(), Ok(&42));
		/// ```
		#[inline]
		pub fn into_rc_try_lazy(self) -> crate::types::RcTryLazy<'static, A, E> {
			crate::types::RcTryLazy::from(self)
		}

		/// Evaluates this `TryTrampoline` and wraps the result in a thread-safe [`ArcTryLazy`](crate::types::ArcTryLazy).
		///
		/// The trampoline is evaluated eagerly because its inner closures are
		/// not `Send`. The result is stored in an `ArcTryLazy` for thread-safe sharing.
		#[document_signature]
		///
		#[document_returns("A thread-safe `ArcTryLazy` containing the eagerly evaluated result.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let task: TryTrampoline<i32, String> = TryTrampoline::ok(42);
		/// let lazy = task.into_arc_try_lazy();
		/// assert_eq!(lazy.evaluate(), Ok(&42));
		/// ```
		#[inline]
		pub fn into_arc_try_lazy(self) -> crate::types::ArcTryLazy<'static, A, E>
		where
			A: Send + Sync,
			E: Send + Sync, {
			crate::types::ArcTryLazy::from(self)
		}
	}

	#[document_type_parameters("The type of the computed value.", "The type of the error value.")]
	impl<A: 'static, E: 'static> TryTrampoline<A, E> {
		/// Creates a `TryTrampoline` that catches unwinds (panics), converting the
		/// panic payload using a custom conversion function.
		///
		/// The closure `f` is executed when the trampoline is evaluated. If `f`
		/// panics, the panic payload is passed to `handler` to produce the
		/// error value. If `f` returns normally, the value is wrapped in `Ok`.
		#[document_signature]
		///
		#[document_parameters(
			"The closure that might panic.",
			"The function that converts a panic payload into the error type."
		)]
		///
		#[document_returns(
			"A new `TryTrampoline` instance where panics are converted to `Err(E)` via the handler."
		)]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let task = TryTrampoline::<i32, i32>::catch_unwind_with(
		/// 	|| {
		/// 		if true {
		/// 			panic!("oops")
		/// 		}
		/// 		42
		/// 	},
		/// 	|_payload| -1,
		/// );
		/// assert_eq!(task.evaluate(), Err(-1));
		/// ```
		pub fn catch_unwind_with(
			f: impl FnOnce() -> A + std::panic::UnwindSafe + 'static,
			handler: impl FnOnce(Box<dyn std::any::Any + Send>) -> E + 'static,
		) -> Self {
			Self::new(move || std::panic::catch_unwind(f).map_err(handler))
		}
	}

	#[document_type_parameters("The type of the computed value.")]
	impl<A: 'static> TryTrampoline<A, String> {
		/// Creates a `TryTrampoline` that catches unwinds (panics).
		///
		/// The closure is executed when the trampoline is evaluated. If the closure
		/// panics, the panic payload is converted to a `String` error. If the
		/// closure returns normally, the value is wrapped in `Ok`.
		///
		/// This is a convenience wrapper around [`catch_unwind_with`](TryTrampoline::catch_unwind_with)
		/// that uses the default panic payload to string conversion.
		#[document_signature]
		///
		#[document_parameters("The closure that might panic.")]
		///
		#[document_returns(
			"A new `TryTrampoline` instance where panics are converted to `Err(String)`."
		)]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let task = TryTrampoline::<i32, String>::catch_unwind(|| {
		/// 	if true {
		/// 		panic!("oops")
		/// 	}
		/// 	42
		/// });
		/// assert_eq!(task.evaluate(), Err("oops".to_string()));
		/// ```
		pub fn catch_unwind(f: impl FnOnce() -> A + std::panic::UnwindSafe + 'static) -> Self {
			Self::catch_unwind_with(f, crate::utils::panic_payload_to_string)
		}
	}

	#[document_type_parameters("The type of the success value.", "The type of the error value.")]
	impl<A, E> From<Trampoline<A>> for TryTrampoline<A, E>
	where
		A: 'static,
		E: 'static,
	{
		#[document_signature]
		#[document_parameters("The trampoline computation to convert.")]
		#[document_returns("A new `TryTrampoline` instance that wraps the trampoline.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		/// let task = Trampoline::pure(42);
		/// let try_task: TryTrampoline<i32, ()> = TryTrampoline::from(task);
		/// assert_eq!(try_task.evaluate(), Ok(42));
		/// ```
		fn from(task: Trampoline<A>) -> Self {
			TryTrampoline(task.map(Ok))
		}
	}

	#[document_type_parameters(
		"The type of the success value.",
		"The type of the error value.",
		"The memoization configuration."
	)]
	impl<A, E, Config> From<Lazy<'static, A, Config>> for TryTrampoline<A, E>
	where
		A: Clone + 'static,
		E: 'static,
		Config: LazyConfig,
	{
		/// Converts a [`Lazy`] value into a [`TryTrampoline`] that defers evaluation.
		///
		/// The `Lazy` is not forced at conversion time; instead, the `TryTrampoline`
		/// defers evaluation until it is run. This is the same deferred semantics as
		/// [`From<TryLazy>`](#impl-From<TryLazy<'static,+A,+E,+Config>>-for-TryTrampoline<A,+E>).
		#[document_signature]
		#[document_parameters("The lazy value to convert.")]
		#[document_returns("A new `TryTrampoline` instance that wraps the lazy value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		/// let lazy = Lazy::<_, RcLazyConfig>::pure(42);
		/// let try_task: TryTrampoline<i32, ()> = TryTrampoline::from(lazy);
		/// assert_eq!(try_task.evaluate(), Ok(42));
		/// ```
		fn from(memo: Lazy<'static, A, Config>) -> Self {
			TryTrampoline(Trampoline::new(move || Ok(memo.evaluate().clone())))
		}
	}

	#[document_type_parameters(
		"The type of the success value.",
		"The type of the error value.",
		"The memoization configuration."
	)]
	impl<A, E, Config> From<TryLazy<'static, A, E, Config>> for TryTrampoline<A, E>
	where
		A: Clone + 'static,
		E: Clone + 'static,
		Config: TryLazyConfig,
	{
		/// Converts a [`TryLazy`] value into a [`TryTrampoline`] that defers evaluation.
		///
		/// This conversion defers forcing the `TryLazy` until the `TryTrampoline` is run.
		/// The cost depends on the [`Clone`] implementations of `A` and `E`.
		#[document_signature]
		#[document_parameters("The fallible lazy value to convert.")]
		#[document_returns("A new `TryTrampoline` instance that wraps the fallible lazy value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		/// let lazy = TryLazy::<_, _, RcLazyConfig>::new(|| Ok::<i32, ()>(42));
		/// let try_task = TryTrampoline::from(lazy);
		/// assert_eq!(try_task.evaluate(), Ok(42));
		/// ```
		fn from(memo: TryLazy<'static, A, E, Config>) -> Self {
			TryTrampoline(Trampoline::new(move || {
				let result = memo.evaluate();
				match result {
					Ok(a) => Ok(a.clone()),
					Err(e) => Err(e.clone()),
				}
			}))
		}
	}

	#[document_type_parameters("The type of the success value.", "The type of the error value.")]
	impl<A, E> From<crate::types::TryThunk<'static, A, E>> for TryTrampoline<A, E>
	where
		A: 'static,
		E: 'static,
	{
		/// Converts a `'static` [`TryThunk`](crate::types::TryThunk) into a `TryTrampoline`.
		///
		/// This lifts a non-stack-safe `TryThunk` into the stack-safe `TryTrampoline`
		/// execution model. The resulting `TryTrampoline` evaluates the thunk when run.
		#[document_signature]
		#[document_parameters("The fallible thunk to convert.")]
		#[document_returns("A new `TryTrampoline` instance that evaluates the thunk.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		/// let thunk = TryThunk::new(|| Ok::<i32, String>(42));
		/// let task = TryTrampoline::from(thunk);
		/// assert_eq!(task.evaluate(), Ok(42));
		/// ```
		fn from(thunk: crate::types::TryThunk<'static, A, E>) -> Self {
			TryTrampoline::new(move || thunk.evaluate())
		}
	}

	#[document_type_parameters("The type of the success value.", "The type of the error value.")]
	impl<A, E> From<Result<A, E>> for TryTrampoline<A, E>
	where
		A: 'static,
		E: 'static,
	{
		#[document_signature]
		#[document_parameters("The result to convert.")]
		#[document_returns("A new `TryTrampoline` instance that produces the result.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		/// let ok_task: TryTrampoline<i32, String> = TryTrampoline::from(Ok(42));
		/// assert_eq!(ok_task.evaluate(), Ok(42));
		///
		/// let err_task: TryTrampoline<i32, String> = TryTrampoline::from(Err("error".to_string()));
		/// assert_eq!(err_task.evaluate(), Err("error".to_string()));
		/// ```
		fn from(result: Result<A, E>) -> Self {
			TryTrampoline(Trampoline::pure(result))
		}
	}

	#[document_type_parameters("The type of the success value.", "The type of the error value.")]
	impl<A, E> Deferrable<'static> for TryTrampoline<A, E>
	where
		A: 'static,
		E: 'static,
	{
		/// Creates a value from a computation that produces the value.
		#[document_signature]
		///
		#[document_parameters("A thunk that produces the value.")]
		///
		#[document_returns("The deferred value.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::Deferrable,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let task: TryTrampoline<i32, String> = Deferrable::defer(|| TryTrampoline::ok(42));
		/// assert_eq!(task.evaluate(), Ok(42));
		/// ```
		fn defer(f: impl FnOnce() -> Self + 'static) -> Self
		where
			Self: Sized, {
			TryTrampoline(Trampoline::defer(move || f().0))
		}
	}

	#[document_type_parameters("The type of the success value.", "The type of the error value.")]
	impl<A, E> Semigroup for TryTrampoline<A, E>
	where
		A: Semigroup + 'static,
		E: 'static,
	{
		/// Combines two `TryTrampoline` computations.
		///
		/// Both computations are evaluated; if both succeed, their results are combined
		/// using the inner `Semigroup`. If either fails, the first error is propagated.
		#[document_signature]
		///
		#[document_parameters(
			"The first `TryTrampoline` computation.",
			"The second `TryTrampoline` computation."
		)]
		///
		#[document_returns(
			"A `TryTrampoline` that evaluates both and combines the results, or propagates the first error."
		)]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	classes::Semigroup,
		/// 	types::*,
		/// };
		///
		/// let a: TryTrampoline<String, ()> = TryTrampoline::ok("hello".to_string());
		/// let b: TryTrampoline<String, ()> = TryTrampoline::ok(" world".to_string());
		/// let combined = Semigroup::append(a, b);
		/// assert_eq!(combined.evaluate(), Ok("hello world".to_string()));
		/// ```
		fn append(
			a: Self,
			b: Self,
		) -> Self {
			a.lift2(b, Semigroup::append)
		}
	}

	#[document_type_parameters("The type of the success value.", "The type of the error value.")]
	impl<A, E> Monoid for TryTrampoline<A, E>
	where
		A: Monoid + 'static,
		E: 'static,
	{
		/// Returns a `TryTrampoline` containing the monoidal identity.
		#[document_signature]
		///
		#[document_returns("A `TryTrampoline` that succeeds with the monoidal identity element.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	classes::Monoid,
		/// 	types::*,
		/// };
		///
		/// let e: TryTrampoline<String, ()> = Monoid::empty();
		/// assert_eq!(e.evaluate(), Ok(String::new()));
		/// ```
		fn empty() -> Self {
			TryTrampoline::ok(A::empty())
		}
	}

	#[document_type_parameters("The type of the success value.", "The type of the error value.")]
	#[document_parameters("The try-trampoline to format.")]
	impl<A: 'static, E: 'static> fmt::Debug for TryTrampoline<A, E> {
		/// Formats the try-trampoline without evaluating it.
		#[document_signature]
		#[document_parameters("The formatter.")]
		#[document_returns("The formatting result.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		/// let task = TryTrampoline::<i32, ()>::ok(42);
		/// assert_eq!(format!("{:?}", task), "TryTrampoline(<unevaluated>)");
		/// ```
		fn fmt(
			&self,
			f: &mut fmt::Formatter<'_>,
		) -> fmt::Result {
			f.write_str("TryTrampoline(<unevaluated>)")
		}
	}
}
pub use inner::*;

#[cfg(test)]
mod tests {
	use {
		super::*,
		crate::types::{
			Step,
			Trampoline,
		},
		quickcheck_macros::quickcheck,
	};

	/// Tests `TryTrampoline::ok`.
	///
	/// Verifies that `ok` creates a successful task.
	#[test]
	fn test_try_task_ok() {
		let task: TryTrampoline<i32, String> = TryTrampoline::ok(42);
		assert_eq!(task.evaluate(), Ok(42));
	}

	/// Tests `TryTrampoline::err`.
	///
	/// Verifies that `err` creates a failed task.
	#[test]
	fn test_try_task_err() {
		let task: TryTrampoline<i32, String> = TryTrampoline::err("error".to_string());
		assert_eq!(task.evaluate(), Err("error".to_string()));
	}

	/// Tests `TryTrampoline::map`.
	///
	/// Verifies that `map` transforms the success value.
	#[test]
	fn test_try_task_map() {
		let task: TryTrampoline<i32, String> = TryTrampoline::ok(10).map(|x| x * 2);
		assert_eq!(task.evaluate(), Ok(20));
	}

	/// Tests `TryTrampoline::map_err`.
	///
	/// Verifies that `map_err` transforms the error value.
	#[test]
	fn test_try_task_map_err() {
		let task: TryTrampoline<i32, String> =
			TryTrampoline::err("error".to_string()).map_err(|e| e.to_uppercase());
		assert_eq!(task.evaluate(), Err("ERROR".to_string()));
	}

	/// Tests `TryTrampoline::bind`.
	///
	/// Verifies that `bind` chains computations.
	#[test]
	fn test_try_task_bind() {
		let task: TryTrampoline<i32, String> =
			TryTrampoline::ok(10).bind(|x| TryTrampoline::ok(x * 2));
		assert_eq!(task.evaluate(), Ok(20));
	}

	/// Tests `TryTrampoline::or_else`.
	///
	/// Verifies that `or_else` recovers from failure.
	#[test]
	fn test_try_task_or_else() {
		let task: TryTrampoline<i32, String> =
			TryTrampoline::err("error".to_string()).catch(|_| TryTrampoline::ok(42));
		assert_eq!(task.evaluate(), Ok(42));
	}

	/// Tests `TryTrampoline::catch_with`.
	///
	/// Verifies that `catch_with` recovers from failure using a different error type.
	#[test]
	fn test_catch_with_recovers() {
		let task: TryTrampoline<i32, i32> = TryTrampoline::<i32, String>::err("error".to_string())
			.catch_with(|_| TryTrampoline::err(42));
		assert_eq!(task.evaluate(), Err(42));
	}

	/// Tests `TryTrampoline::catch_with` when the computation succeeds.
	///
	/// Verifies that success values pass through unchanged.
	#[test]
	fn test_catch_with_success_passes_through() {
		let task: TryTrampoline<i32, i32> =
			TryTrampoline::<i32, String>::ok(1).catch_with(|_| TryTrampoline::err(42));
		assert_eq!(task.evaluate(), Ok(1));
	}

	/// Tests `TryTrampoline::new`.
	///
	/// Verifies that `new` creates a lazy task.
	#[test]
	fn test_try_task_new() {
		let task: TryTrampoline<i32, String> = TryTrampoline::new(|| Ok(42));
		assert_eq!(task.evaluate(), Ok(42));
	}

	/// Tests `TryTrampoline::pure`.
	///
	/// Verifies that `pure` creates a successful task.
	#[test]
	fn test_try_trampoline_pure() {
		let task: TryTrampoline<i32, String> = TryTrampoline::pure(42);
		assert_eq!(task.evaluate(), Ok(42));
	}

	/// Tests `TryTrampoline::into_inner`.
	///
	/// Verifies that `into_inner` unwraps the newtype.
	#[test]
	fn test_try_trampoline_into_inner() {
		let task: TryTrampoline<i32, String> = TryTrampoline::ok(42);
		let inner: Trampoline<Result<i32, String>> = task.into_inner();
		assert_eq!(inner.evaluate(), Ok(42));
	}

	/// Tests `From<Trampoline>`.
	#[test]
	fn test_try_task_from_task() {
		let task = Trampoline::pure(42);
		let try_task: TryTrampoline<i32, String> = TryTrampoline::from(task);
		assert_eq!(try_task.evaluate(), Ok(42));
	}

	/// Tests `From<Lazy>`.
	#[test]
	fn test_try_task_from_memo() {
		use crate::types::ArcLazy;
		let memo = ArcLazy::new(|| 42);
		let try_task: TryTrampoline<i32, String> = TryTrampoline::from(memo);
		assert_eq!(try_task.evaluate(), Ok(42));
	}

	/// Tests `From<TryLazy>`.
	#[test]
	fn test_try_task_from_try_memo() {
		use crate::types::ArcTryLazy;
		let memo = ArcTryLazy::new(|| Ok(42));
		let try_task: TryTrampoline<i32, String> = TryTrampoline::from(memo);
		assert_eq!(try_task.evaluate(), Ok(42));
	}

	/// Tests `TryTrampoline::lift2` with two successful values.
	///
	/// Verifies that `lift2` combines results from both computations.
	#[test]
	fn test_try_task_lift2_success() {
		let t1: TryTrampoline<i32, String> = TryTrampoline::ok(10);
		let t2: TryTrampoline<i32, String> = TryTrampoline::ok(20);
		let t3 = t1.lift2(t2, |a, b| a + b);
		assert_eq!(t3.evaluate(), Ok(30));
	}

	/// Tests `TryTrampoline::lift2` short-circuits on first error.
	///
	/// Verifies that if the first computation fails, the second is not evaluated.
	#[test]
	fn test_try_task_lift2_first_error() {
		let t1: TryTrampoline<i32, String> = TryTrampoline::err("first".to_string());
		let t2: TryTrampoline<i32, String> = TryTrampoline::ok(20);
		let t3 = t1.lift2(t2, |a, b| a + b);
		assert_eq!(t3.evaluate(), Err("first".to_string()));
	}

	/// Tests `TryTrampoline::lift2` propagates second error.
	///
	/// Verifies that if the second computation fails, the error is propagated.
	#[test]
	fn test_try_task_lift2_second_error() {
		let t1: TryTrampoline<i32, String> = TryTrampoline::ok(10);
		let t2: TryTrampoline<i32, String> = TryTrampoline::err("second".to_string());
		let t3 = t1.lift2(t2, |a, b| a + b);
		assert_eq!(t3.evaluate(), Err("second".to_string()));
	}

	/// Tests `TryTrampoline::then` with two successful values.
	///
	/// Verifies that `then` discards the first result and returns the second.
	#[test]
	fn test_try_task_then_success() {
		let t1: TryTrampoline<i32, String> = TryTrampoline::ok(10);
		let t2: TryTrampoline<i32, String> = TryTrampoline::ok(20);
		let t3 = t1.then(t2);
		assert_eq!(t3.evaluate(), Ok(20));
	}

	/// Tests `TryTrampoline::then` short-circuits on first error.
	///
	/// Verifies that if the first computation fails, the second is not evaluated.
	#[test]
	fn test_try_task_then_first_error() {
		let t1: TryTrampoline<i32, String> = TryTrampoline::err("first".to_string());
		let t2: TryTrampoline<i32, String> = TryTrampoline::ok(20);
		let t3 = t1.then(t2);
		assert_eq!(t3.evaluate(), Err("first".to_string()));
	}

	/// Tests `TryTrampoline::then` propagates second error.
	///
	/// Verifies that if the second computation fails, the error is propagated.
	#[test]
	fn test_try_task_then_second_error() {
		let t1: TryTrampoline<i32, String> = TryTrampoline::ok(10);
		let t2: TryTrampoline<i32, String> = TryTrampoline::err("second".to_string());
		let t3 = t1.then(t2);
		assert_eq!(t3.evaluate(), Err("second".to_string()));
	}

	/// Tests `TryTrampoline::tail_rec_m` with a factorial computation.
	///
	/// Verifies both the success and error paths.
	#[test]
	fn test_try_task_tail_rec_m() {
		fn factorial(n: i32) -> TryTrampoline<i32, String> {
			TryTrampoline::tail_rec_m(
				|(n, acc)| {
					if n < 0 {
						TryTrampoline::err("Negative input".to_string())
					} else if n <= 1 {
						TryTrampoline::ok(Step::Done(acc))
					} else {
						TryTrampoline::ok(Step::Loop((n - 1, n * acc)))
					}
				},
				(n, 1),
			)
		}

		assert_eq!(factorial(5).evaluate(), Ok(120));
		assert_eq!(factorial(0).evaluate(), Ok(1));
		assert_eq!(factorial(1).evaluate(), Ok(1));
	}

	/// Tests `TryTrampoline::tail_rec_m` error short-circuit.
	///
	/// Verifies that an error terminates the recursion immediately.
	#[test]
	fn test_try_task_tail_rec_m_error() {
		let task: TryTrampoline<i32, String> = TryTrampoline::tail_rec_m(
			|n: i32| {
				if n >= 5 {
					TryTrampoline::err(format!("too large: {}", n))
				} else {
					TryTrampoline::ok(Step::Loop(n + 1))
				}
			},
			0,
		);

		assert_eq!(task.evaluate(), Err("too large: 5".to_string()));
	}

	/// Tests `TryTrampoline::tail_rec_m` stack safety.
	///
	/// Verifies that `tail_rec_m` does not overflow the stack with 100,000+ iterations.
	#[test]
	fn test_try_task_tail_rec_m_stack_safety() {
		let n = 100_000u64;
		let task: TryTrampoline<u64, String> = TryTrampoline::tail_rec_m(
			|(remaining, acc)| {
				if remaining == 0 {
					TryTrampoline::ok(Step::Done(acc))
				} else {
					TryTrampoline::ok(Step::Loop((remaining - 1, acc + remaining)))
				}
			},
			(n, 0u64),
		);

		assert_eq!(task.evaluate(), Ok(n * (n + 1) / 2));
	}

	/// Tests `TryTrampoline::tail_rec_m` stack safety with error at the end.
	///
	/// Verifies that error short-circuit works after many successful iterations.
	#[test]
	fn test_try_task_tail_rec_m_stack_safety_error() {
		let n = 100_000u64;
		let task: TryTrampoline<u64, String> = TryTrampoline::tail_rec_m(
			|remaining| {
				if remaining == 0 {
					TryTrampoline::err("done iterating".to_string())
				} else {
					TryTrampoline::ok(Step::Loop(remaining - 1))
				}
			},
			n,
		);

		assert_eq!(task.evaluate(), Err("done iterating".to_string()));
	}

	/// Tests `TryTrampoline::arc_tail_rec_m` with non-Clone closures.
	///
	/// Verifies that `arc_tail_rec_m` works with closures capturing `Arc<AtomicUsize>`.
	#[test]
	fn test_try_task_arc_tail_rec_m() {
		use std::sync::{
			Arc,
			atomic::{
				AtomicUsize,
				Ordering,
			},
		};

		let counter = Arc::new(AtomicUsize::new(0));
		let counter_clone = Arc::clone(&counter);

		let task: TryTrampoline<i32, String> = TryTrampoline::arc_tail_rec_m(
			move |n| {
				counter_clone.fetch_add(1, Ordering::SeqCst);
				if n == 0 {
					TryTrampoline::ok(Step::Done(0))
				} else {
					TryTrampoline::ok(Step::Loop(n - 1))
				}
			},
			10,
		);

		assert_eq!(task.evaluate(), Ok(0));
		assert_eq!(counter.load(Ordering::SeqCst), 11);
	}

	/// Tests `TryTrampoline::arc_tail_rec_m` error short-circuit.
	///
	/// Verifies that `arc_tail_rec_m` correctly propagates errors.
	#[test]
	fn test_try_task_arc_tail_rec_m_error() {
		use std::sync::{
			Arc,
			atomic::{
				AtomicUsize,
				Ordering,
			},
		};

		let counter = Arc::new(AtomicUsize::new(0));
		let counter_clone = Arc::clone(&counter);

		let task: TryTrampoline<i32, String> = TryTrampoline::arc_tail_rec_m(
			move |n| {
				counter_clone.fetch_add(1, Ordering::SeqCst);
				if n >= 5 {
					TryTrampoline::err(format!("too large: {}", n))
				} else {
					TryTrampoline::ok(Step::Loop(n + 1))
				}
			},
			0,
		);

		assert_eq!(task.evaluate(), Err("too large: 5".to_string()));
		// Should have been called 6 times (0, 1, 2, 3, 4, 5)
		assert_eq!(counter.load(Ordering::SeqCst), 6);
	}

	/// Tests `From<TryThunk>` with a successful thunk.
	#[test]
	fn test_try_task_from_try_thunk_ok() {
		use crate::types::TryThunk;
		let thunk = TryThunk::new(|| Ok::<i32, String>(42));
		let task = TryTrampoline::from(thunk);
		assert_eq!(task.evaluate(), Ok(42));
	}

	/// Tests `From<TryThunk>` with a failed thunk.
	#[test]
	fn test_try_task_from_try_thunk_err() {
		use crate::types::TryThunk;
		let thunk = TryThunk::new(|| Err::<i32, String>("error".to_string()));
		let task = TryTrampoline::from(thunk);
		assert_eq!(task.evaluate(), Err("error".to_string()));
	}

	/// Tests bidirectional conversion between `TryThunk` and `TryTrampoline`.
	///
	/// Verifies that a value survives a round-trip through both conversions.
	#[test]
	fn test_try_thunk_try_trampoline_round_trip() {
		use crate::types::TryThunk;

		// TryThunk -> TryTrampoline -> TryThunk (Ok case)
		let thunk = TryThunk::new(|| Ok::<i32, String>(42));
		let tramp = TryTrampoline::from(thunk);
		let thunk_back: TryThunk<i32, String> = TryThunk::from(tramp);
		assert_eq!(thunk_back.evaluate(), Ok(42));

		// TryTrampoline -> TryThunk -> TryTrampoline (Err case)
		let tramp = TryTrampoline::err("fail".to_string());
		let thunk: TryThunk<i32, String> = TryThunk::from(tramp);
		let tramp_back = TryTrampoline::from(thunk);
		assert_eq!(tramp_back.evaluate(), Err("fail".to_string()));
	}

	/// Tests `From<Result>` with `Ok`.
	#[test]
	fn test_try_task_from_result_ok() {
		let task: TryTrampoline<i32, String> = TryTrampoline::from(Ok(42));
		assert_eq!(task.evaluate(), Ok(42));
	}

	/// Tests `From<Result>` with `Err`.
	#[test]
	fn test_try_task_from_result_err() {
		let task: TryTrampoline<i32, String> = TryTrampoline::from(Err("error".to_string()));
		assert_eq!(task.evaluate(), Err("error".to_string()));
	}

	// Tests for !Send types (Rc)

	/// Tests that `TryTrampoline` works with `Rc<T>`, a `!Send` type.
	///
	/// This verifies that the `Send` bound relaxation allows single-threaded
	/// stack-safe fallible recursion with reference-counted types.
	#[test]
	fn test_try_trampoline_with_rc() {
		use std::rc::Rc;

		let task: TryTrampoline<Rc<i32>, Rc<String>> = TryTrampoline::ok(Rc::new(42));
		assert_eq!(*task.evaluate().unwrap(), 42);
	}

	/// Tests `TryTrampoline::bind` with `Rc<T>`.
	///
	/// Verifies that `bind` works correctly when value and error types are `!Send`.
	#[test]
	fn test_try_trampoline_bind_with_rc() {
		use std::rc::Rc;

		let task: TryTrampoline<Rc<i32>, Rc<String>> =
			TryTrampoline::ok(Rc::new(10)).bind(|rc| TryTrampoline::ok(Rc::new(*rc * 2)));
		assert_eq!(*task.evaluate().unwrap(), 20);
	}

	/// Tests `TryTrampoline::tail_rec_m` with `Rc<T>`.
	///
	/// Verifies that stack-safe fallible recursion works with `!Send` types.
	#[test]
	fn test_try_trampoline_tail_rec_m_with_rc() {
		use std::rc::Rc;

		let task: TryTrampoline<Rc<u64>, Rc<String>> = TryTrampoline::tail_rec_m(
			|(n, acc): (u64, Rc<u64>)| {
				if n == 0 {
					TryTrampoline::ok(Step::Done(acc))
				} else {
					TryTrampoline::ok(Step::Loop((n - 1, Rc::new(*acc + n))))
				}
			},
			(100u64, Rc::new(0u64)),
		);
		assert_eq!(*task.evaluate().unwrap(), 5050);
	}

	/// Tests `catch_unwind` on `TryTrampoline`.
	///
	/// Verifies that panics are caught and converted to `Err(String)`.
	#[test]
	fn test_catch_unwind() {
		let task = TryTrampoline::<i32, String>::catch_unwind(|| {
			if true {
				panic!("oops")
			}
			42
		});
		assert_eq!(task.evaluate(), Err("oops".to_string()));
	}

	/// Tests `catch_unwind` on `TryTrampoline` with a non-panicking closure.
	///
	/// Verifies that a successful closure wraps the value in `Ok`.
	#[test]
	fn test_catch_unwind_success() {
		let task = TryTrampoline::<i32, String>::catch_unwind(|| 42);
		assert_eq!(task.evaluate(), Ok(42));
	}

	/// Tests `TryTrampoline::catch_unwind_with` with a panicking closure.
	///
	/// Verifies that the custom handler converts the panic payload.
	#[test]
	fn test_catch_unwind_with_panic() {
		let task = TryTrampoline::<i32, i32>::catch_unwind_with(
			|| {
				if true {
					panic!("oops")
				}
				42
			},
			|_payload| -1,
		);
		assert_eq!(task.evaluate(), Err(-1));
	}

	/// Tests `TryTrampoline::catch_unwind_with` with a non-panicking closure.
	///
	/// Verifies that a successful closure wraps the value in `Ok`.
	#[test]
	fn test_catch_unwind_with_success() {
		let task = TryTrampoline::<i32, i32>::catch_unwind_with(|| 42, |_payload| -1);
		assert_eq!(task.evaluate(), Ok(42));
	}

	// QuickCheck Law Tests (via inherent methods)

	// Functor Laws

	/// Functor identity: `ok(a).map(identity) == Ok(a)`.
	#[quickcheck]
	fn functor_identity(x: i32) -> bool {
		TryTrampoline::<i32, i32>::ok(x).map(|a| a).evaluate() == Ok(x)
	}

	/// Functor composition: `t.map(f . g) == t.map(g).map(f)`.
	#[quickcheck]
	fn functor_composition(x: i32) -> bool {
		let f = |a: i32| a.wrapping_add(1);
		let g = |a: i32| a.wrapping_mul(2);
		let lhs = TryTrampoline::<i32, i32>::ok(x).map(move |a| f(g(a))).evaluate();
		let rhs = TryTrampoline::<i32, i32>::ok(x).map(g).map(f).evaluate();
		lhs == rhs
	}

	// Monad Laws

	/// Monad left identity: `ok(a).bind(f) == f(a)`.
	#[quickcheck]
	fn monad_left_identity(a: i32) -> bool {
		let f = |x: i32| TryTrampoline::<i32, i32>::ok(x.wrapping_mul(2));
		TryTrampoline::ok(a).bind(f).evaluate() == f(a).evaluate()
	}

	/// Monad right identity: `m.bind(ok) == m`.
	#[quickcheck]
	fn monad_right_identity(x: i32) -> bool {
		TryTrampoline::<i32, i32>::ok(x).bind(TryTrampoline::ok).evaluate() == Ok(x)
	}

	/// Monad associativity: `m.bind(f).bind(g) == m.bind(|a| f(a).bind(g))`.
	#[quickcheck]
	fn monad_associativity(x: i32) -> bool {
		let f = |a: i32| TryTrampoline::<i32, i32>::ok(a.wrapping_add(1));
		let g = |a: i32| TryTrampoline::<i32, i32>::ok(a.wrapping_mul(3));
		let lhs = TryTrampoline::<i32, i32>::ok(x).bind(f).bind(g).evaluate();
		let rhs = TryTrampoline::<i32, i32>::ok(x).bind(move |a| f(a).bind(g)).evaluate();
		lhs == rhs
	}

	/// Error short-circuit: `err(e).bind(f).evaluate() == Err(e)`.
	#[quickcheck]
	fn error_short_circuit(e: i32) -> bool {
		TryTrampoline::<i32, i32>::err(e).bind(|x| TryTrampoline::ok(x.wrapping_add(1))).evaluate()
			== Err(e)
	}

	// Semigroup / Monoid tests

	/// Tests Semigroup::append with two successful computations.
	#[test]
	fn test_semigroup_append_both_ok() {
		use crate::classes::Semigroup;

		let a: TryTrampoline<String, ()> = TryTrampoline::ok("hello".to_string());
		let b: TryTrampoline<String, ()> = TryTrampoline::ok(" world".to_string());
		let result = Semigroup::append(a, b);
		assert_eq!(result.evaluate(), Ok("hello world".to_string()));
	}

	/// Tests Semigroup::append propagates the first error.
	#[test]
	fn test_semigroup_append_first_err() {
		use crate::classes::Semigroup;

		let a: TryTrampoline<String, String> = TryTrampoline::err("fail".to_string());
		let b: TryTrampoline<String, String> = TryTrampoline::ok(" world".to_string());
		let result = Semigroup::append(a, b);
		assert_eq!(result.evaluate(), Err("fail".to_string()));
	}

	/// Tests Semigroup::append propagates the second error.
	#[test]
	fn test_semigroup_append_second_err() {
		use crate::classes::Semigroup;

		let a: TryTrampoline<String, String> = TryTrampoline::ok("hello".to_string());
		let b: TryTrampoline<String, String> = TryTrampoline::err("fail".to_string());
		let result = Semigroup::append(a, b);
		assert_eq!(result.evaluate(), Err("fail".to_string()));
	}

	/// Tests Semigroup associativity law for TryTrampoline.
	#[quickcheck]
	fn semigroup_associativity(
		a: String,
		b: String,
		c: String,
	) -> bool {
		use crate::classes::Semigroup;

		let lhs = Semigroup::append(
			Semigroup::append(
				TryTrampoline::<String, ()>::ok(a.clone()),
				TryTrampoline::ok(b.clone()),
			),
			TryTrampoline::ok(c.clone()),
		)
		.evaluate();
		let rhs = Semigroup::append(
			TryTrampoline::<String, ()>::ok(a),
			Semigroup::append(TryTrampoline::ok(b), TryTrampoline::ok(c)),
		)
		.evaluate();
		lhs == rhs
	}

	/// Tests Monoid::empty returns Ok with the monoidal identity.
	#[test]
	fn test_monoid_empty() {
		use crate::classes::Monoid;

		let e: TryTrampoline<String, ()> = Monoid::empty();
		assert_eq!(e.evaluate(), Ok(String::new()));
	}

	/// Tests Monoid left identity law.
	#[quickcheck]
	fn monoid_left_identity(a: String) -> bool {
		use crate::classes::{
			Monoid,
			Semigroup,
		};

		let lhs = Semigroup::append(Monoid::empty(), TryTrampoline::<String, ()>::ok(a.clone()))
			.evaluate();
		lhs == Ok(a)
	}

	/// Tests Monoid right identity law.
	#[quickcheck]
	fn monoid_right_identity(a: String) -> bool {
		use crate::classes::{
			Monoid,
			Semigroup,
		};

		let lhs = Semigroup::append(TryTrampoline::<String, ()>::ok(a.clone()), Monoid::empty())
			.evaluate();
		lhs == Ok(a)
	}

	// bimap tests

	/// Tests bimap on a successful computation.
	#[test]
	fn test_bimap_ok() {
		let task: TryTrampoline<i32, String> = TryTrampoline::ok(10);
		let result = task.bimap(|x| x * 2, |e| e.len());
		assert_eq!(result.evaluate(), Ok(20));
	}

	/// Tests bimap on a failed computation.
	#[test]
	fn test_bimap_err() {
		let task: TryTrampoline<i32, String> = TryTrampoline::err("hello".to_string());
		let result = task.bimap(|x| x * 2, |e| e.len());
		assert_eq!(result.evaluate(), Err(5));
	}

	/// Tests bimap composes with map and map_err.
	#[quickcheck]
	fn bimap_consistent_with_map_and_map_err(x: i32) -> bool {
		let f = |a: i32| a.wrapping_add(1);
		let g = |e: i32| e.wrapping_mul(2);

		let via_bimap = TryTrampoline::<i32, i32>::ok(x).bimap(f, g).evaluate();
		let via_map = TryTrampoline::<i32, i32>::ok(x).map(f).evaluate();
		via_bimap == via_map
	}

	/// Tests bimap on error is consistent with map_err.
	#[quickcheck]
	fn bimap_err_consistent_with_map_err(e: i32) -> bool {
		let f = |a: i32| a.wrapping_add(1);
		let g = |e: i32| e.wrapping_mul(2);

		let via_bimap = TryTrampoline::<i32, i32>::err(e).bimap(f, g).evaluate();
		let via_map_err = TryTrampoline::<i32, i32>::err(e).map_err(g).evaluate();
		via_bimap == via_map_err
	}

	// into_rc_try_lazy / into_arc_try_lazy tests

	/// Tests `TryTrampoline::into_rc_try_lazy` with a successful computation.
	///
	/// Verifies that the returned `RcTryLazy` evaluates to the same result and
	/// memoizes it (the computation runs at most once).
	#[test]
	fn test_into_rc_try_lazy_ok() {
		let task: TryTrampoline<i32, String> = TryTrampoline::ok(42);
		let lazy = task.into_rc_try_lazy();
		assert_eq!(lazy.evaluate(), Ok(&42));
		// Second access returns the cached value.
		assert_eq!(lazy.evaluate(), Ok(&42));
	}

	/// Tests `TryTrampoline::into_rc_try_lazy` with a failed computation.
	///
	/// Verifies that the returned `RcTryLazy` memoizes the error result.
	#[test]
	fn test_into_rc_try_lazy_err() {
		let task: TryTrampoline<i32, String> = TryTrampoline::err("oops".to_string());
		let lazy = task.into_rc_try_lazy();
		assert_eq!(lazy.evaluate(), Err(&"oops".to_string()));
	}

	/// Tests `TryTrampoline::into_arc_try_lazy` with a successful computation.
	///
	/// Verifies that the returned `ArcTryLazy` evaluates to the same result.
	/// `into_arc_try_lazy` evaluates eagerly because the inner closures are not
	/// `Send`, so the result is stored immediately.
	#[test]
	fn test_into_arc_try_lazy_ok() {
		let task: TryTrampoline<i32, String> = TryTrampoline::ok(42);
		let lazy = task.into_arc_try_lazy();
		assert_eq!(lazy.evaluate(), Ok(&42));
		// Second access returns the cached value.
		assert_eq!(lazy.evaluate(), Ok(&42));
	}

	/// Tests `TryTrampoline::into_arc_try_lazy` with a failed computation.
	///
	/// Verifies that the returned `ArcTryLazy` memoizes the error result.
	#[test]
	fn test_into_arc_try_lazy_err() {
		let task: TryTrampoline<i32, String> = TryTrampoline::err("oops".to_string());
		let lazy = task.into_arc_try_lazy();
		assert_eq!(lazy.evaluate(), Err(&"oops".to_string()));
	}

	/// Tests `TryTrampoline::catch_with` stack safety.
	///
	/// Verifies that deeply chained `catch_with` calls do not overflow the stack.
	#[test]
	fn test_catch_with_stack_safety() {
		let n = 100_000u64;
		let mut task: TryTrampoline<u64, u64> = TryTrampoline::err(0);
		for i in 1 ..= n {
			task = task.catch_with(move |_| TryTrampoline::err(i));
		}
		assert_eq!(task.evaluate(), Err(n));
	}

	/// Tests `TryTrampoline::catch_with` stack safety on the success path.
	///
	/// Verifies that a deeply chained `catch_with` that eventually succeeds
	/// does not overflow the stack.
	#[test]
	fn test_catch_with_stack_safety_ok() {
		let n = 100_000u64;
		let mut task: TryTrampoline<u64, u64> = TryTrampoline::err(0);
		for i in 1 .. n {
			task = task.catch_with(move |_| TryTrampoline::err(i));
		}
		task = task.catch_with(|e| TryTrampoline::ok(e));
		assert_eq!(task.evaluate(), Ok(n - 1));
	}
}
