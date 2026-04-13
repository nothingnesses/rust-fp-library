//! Deferred, non-memoized fallible computation with higher-kinded type support.
//!
//! The fallible counterpart to [`Thunk`](crate::types::Thunk). Each call to [`TryThunk::evaluate`] re-executes the computation and returns a [`Result`]. Supports borrowing and lifetime polymorphism.

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			Apply,
			brands::{
				TryThunkBrand,
				TryThunkErrAppliedBrand,
				TryThunkOkAppliedBrand,
			},
			classes::{
				ApplyFirst,
				ApplySecond,
				Bifoldable,
				Bifunctor,
				CloneFn,
				Deferrable,
				Foldable,
				FoldableWithIndex,
				Functor,
				FunctorWithIndex,
				LazyConfig,
				Lift,
				LiftFn,
				MonadRec,
				Monoid,
				Pointed,
				Semiapplicative,
				Semigroup,
				Semimonad,
				WithIndex,
			},
			impl_kind,
			kinds::*,
			types::{
				ArcTryLazy,
				Lazy,
				RcTryLazy,
				Thunk,
				TryLazy,
				TrySendThunk,
				TryTrampoline,
			},
		},
		core::ops::ControlFlow,
		fp_macros::*,
		std::{
			fmt,
			sync::Arc,
		},
	};

	/// A deferred computation that may fail with error type `E`.
	///
	/// This is [`Thunk<'a, Result<A, E>>`] with ergonomic combinators for error handling.
	/// Like [`Thunk`], this is NOT memoized. Each [`TryThunk::evaluate`] re-executes.
	/// Unlike [`Thunk`], the result is [`Result<A, E>`].
	#[document_type_parameters(
		"The lifetime of the computation.",
		"The type of the value produced by the computation on success.",
		"The type of the error produced by the computation on failure."
	)]
	///
	/// ### Higher-Kinded Type Representation
	///
	/// This type has multiple higher-kinded representations:
	/// - [`TryThunkBrand`](crate::brands::TryThunkBrand): fully polymorphic over both error and success types (bifunctor).
	/// - [`TryThunkErrAppliedBrand<E>`](crate::brands::TryThunkErrAppliedBrand): the error type is fixed, polymorphic over the success type (functor over `Ok`).
	/// - [`TryThunkOkAppliedBrand<A>`](crate::brands::TryThunkOkAppliedBrand): the success type is fixed, polymorphic over the error type (functor over `Err`).
	///
	/// ### When to Use
	///
	/// Use `TryThunk` for lightweight fallible deferred computation with full HKT support.
	/// It is not stack-safe for deep [`bind`](TryThunk::bind) chains. For stack-safe fallible
	/// recursion, use [`TryTrampoline`](crate::types::TryTrampoline). For memoized fallible
	/// computation, use [`TryLazy`](crate::types::TryLazy).
	///
	/// ### Algebraic Properties
	///
	/// `TryThunk` forms a monad over the success type `A` (with `E` fixed):
	/// - `TryThunk::pure(a).bind(f).evaluate() == f(a).evaluate()` (left identity).
	/// - `thunk.bind(TryThunk::pure).evaluate() == thunk.evaluate()` (right identity).
	/// - `thunk.bind(f).bind(g).evaluate() == thunk.bind(|a| f(a).bind(g)).evaluate()` (associativity).
	///
	/// On the error channel, `bind` short-circuits: if the computation produces `Err(e)`,
	/// the continuation `f` is never called.
	///
	/// ### Stack Safety
	///
	/// `TryThunk::bind` chains are **not** stack-safe. Each nested [`bind`](TryThunk::bind)
	/// adds a frame to the call stack, so sufficiently deep chains will cause a stack overflow.
	/// For stack-safe fallible recursion, use [`TryTrampoline`](crate::types::TryTrampoline).
	///
	/// ### Limitations
	///
	/// **Cannot implement `Traversable`**: `TryThunk` wraps a `FnOnce` closure, which cannot be
	/// cloned because `FnOnce` is consumed when called. The [`Traversable`](crate::classes::Traversable)
	/// trait requires `Clone` bounds on the result type, making it fundamentally incompatible
	/// with `TryThunk`'s design. This mirrors the same limitation on [`Thunk`].
	pub struct TryThunk<'a, A, E>(
		/// The internal `Thunk` wrapping a `Result`.
		Thunk<'a, Result<A, E>>,
	);

	#[document_type_parameters(
		"The lifetime of the computation.",
		"The type of the success value.",
		"The type of the error value."
	)]
	#[document_parameters("The `TryThunk` instance.")]
	impl<'a, A: 'a, E: 'a> TryThunk<'a, A, E> {
		/// Creates a new `TryThunk` from a thunk.
		#[document_signature]
		///
		#[document_parameters("The thunk to wrap.")]
		///
		#[document_returns("A new `TryThunk` instance.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let try_thunk: TryThunk<i32, ()> = TryThunk::new(|| Ok(42));
		/// assert_eq!(try_thunk.evaluate(), Ok(42));
		/// ```
		#[inline]
		pub fn new(f: impl FnOnce() -> Result<A, E> + 'a) -> Self {
			TryThunk(Thunk::new(f))
		}

		/// Returns a pure value (already computed).
		#[document_signature]
		///
		#[document_parameters("The value to wrap.")]
		///
		#[document_returns("A new `TryThunk` instance containing the value.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let try_thunk: TryThunk<i32, ()> = TryThunk::pure(42);
		/// assert_eq!(try_thunk.evaluate(), Ok(42));
		/// ```
		#[inline]
		pub fn pure(a: A) -> Self {
			TryThunk(Thunk::pure(Ok(a)))
		}

		/// Defers a computation that returns a TryThunk.
		#[document_signature]
		///
		#[document_parameters("The thunk that returns a `TryThunk`.")]
		///
		#[document_returns("A new `TryThunk` instance.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let try_thunk: TryThunk<i32, ()> = TryThunk::defer(|| TryThunk::ok(42));
		/// assert_eq!(try_thunk.evaluate(), Ok(42));
		/// ```
		#[inline]
		pub fn defer(f: impl FnOnce() -> TryThunk<'a, A, E> + 'a) -> Self {
			TryThunk(Thunk::defer(move || f().0))
		}

		/// Alias for [`pure`](Self::pure), provided for readability.
		///
		/// Both `TryThunk::ok(x)` and `TryThunk::pure(x)` produce the same result: a
		/// deferred computation that succeeds with `x`. The `ok` variant mirrors the
		/// `Result::Ok` constructor name, making intent clearer when working directly
		/// with `TryThunk` values rather than through HKT abstractions.
		#[document_signature]
		///
		#[document_parameters("The value to wrap.")]
		///
		#[document_returns("A new `TryThunk` instance containing the value.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let try_thunk: TryThunk<i32, ()> = TryThunk::ok(42);
		/// assert_eq!(try_thunk.evaluate(), Ok(42));
		/// ```
		#[inline]
		pub fn ok(a: A) -> Self {
			TryThunk(Thunk::pure(Ok(a)))
		}

		/// Returns a pure error.
		#[document_signature]
		///
		#[document_parameters("The error to wrap.")]
		///
		#[document_returns("A new `TryThunk` instance containing the error.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let try_thunk: TryThunk<i32, &str> = TryThunk::err("error");
		/// assert_eq!(try_thunk.evaluate(), Err("error"));
		/// ```
		#[inline]
		pub fn err(e: E) -> Self {
			TryThunk(Thunk::pure(Err(e)))
		}

		/// Monadic bind: chains computations.
		#[document_signature]
		///
		#[document_type_parameters("The type of the result of the new computation.")]
		///
		#[document_parameters("The function to apply to the result of the computation.")]
		///
		#[document_returns("A new `TryThunk` instance representing the chained computation.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let try_thunk: TryThunk<i32, ()> = TryThunk::ok(21).bind(|x| TryThunk::ok(x * 2));
		/// assert_eq!(try_thunk.evaluate(), Ok(42));
		/// ```
		#[inline]
		pub fn bind<B: 'a>(
			self,
			f: impl FnOnce(A) -> TryThunk<'a, B, E> + 'a,
		) -> TryThunk<'a, B, E> {
			TryThunk(self.0.bind(|result| match result {
				Ok(a) => f(a).0,
				Err(e) => Thunk::pure(Err(e)),
			}))
		}

		/// Functor map: transforms the result.
		#[document_signature]
		///
		#[document_type_parameters("The type of the result of the transformation.")]
		///
		#[document_parameters("The function to apply to the result of the computation.")]
		///
		#[document_returns("A new `TryThunk` instance with the transformed result.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let try_thunk: TryThunk<i32, ()> = TryThunk::ok(21).map(|x| x * 2);
		/// assert_eq!(try_thunk.evaluate(), Ok(42));
		/// ```
		#[inline]
		pub fn map<B: 'a>(
			self,
			func: impl FnOnce(A) -> B + 'a,
		) -> TryThunk<'a, B, E> {
			TryThunk(self.0.map(|result| result.map(func)))
		}

		/// Map error: transforms the error.
		#[document_signature]
		///
		#[document_type_parameters("The type of the new error.")]
		///
		#[document_parameters("The function to apply to the error.")]
		///
		#[document_returns("A new `TryThunk` instance with the transformed error.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let try_thunk: TryThunk<i32, i32> = TryThunk::err(21).map_err(|x| x * 2);
		/// assert_eq!(try_thunk.evaluate(), Err(42));
		/// ```
		#[inline]
		pub fn map_err<E2: 'a>(
			self,
			f: impl FnOnce(E) -> E2 + 'a,
		) -> TryThunk<'a, A, E2> {
			TryThunk(self.0.map(|result| result.map_err(f)))
		}

		/// Recovers from an error.
		#[document_signature]
		///
		#[document_parameters("The function to apply to the error value.")]
		///
		#[document_returns("A new `TryThunk` that attempts to recover from failure.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let try_thunk: TryThunk<i32, &str> = TryThunk::err("error").catch(|_| TryThunk::ok(42));
		/// assert_eq!(try_thunk.evaluate(), Ok(42));
		/// ```
		#[inline]
		pub fn catch(
			self,
			f: impl FnOnce(E) -> TryThunk<'a, A, E> + 'a,
		) -> Self {
			TryThunk(self.0.bind(|result| match result {
				Ok(a) => Thunk::pure(Ok(a)),
				Err(e) => f(e).0,
			}))
		}

		/// Recovers from an error using a fallible recovery function that may produce a different error type.
		///
		/// Unlike [`catch`](TryThunk::catch), `catch_with` allows the recovery function to return a
		/// `TryThunk` with a different error type `E2`. On success, the value is passed through
		/// unchanged. On failure, the recovery function is applied to the error value and its result
		/// is evaluated.
		#[document_signature]
		///
		#[document_type_parameters("The error type produced by the recovery computation.")]
		///
		#[document_parameters("The monadic recovery function applied to the error value.")]
		///
		#[document_returns(
			"A new `TryThunk` that either passes through the success value or uses the result of the recovery computation."
		)]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let recovered: TryThunk<i32, i32> =
		/// 	TryThunk::<i32, &str>::err("error").catch_with(|_| TryThunk::err(42));
		/// assert_eq!(recovered.evaluate(), Err(42));
		///
		/// let ok: TryThunk<i32, i32> = TryThunk::<i32, &str>::ok(1).catch_with(|_| TryThunk::err(42));
		/// assert_eq!(ok.evaluate(), Ok(1));
		/// ```
		#[inline]
		pub fn catch_with<E2: 'a>(
			self,
			f: impl FnOnce(E) -> TryThunk<'a, A, E2> + 'a,
		) -> TryThunk<'a, A, E2> {
			TryThunk(Thunk::new(move || match self.evaluate() {
				Ok(a) => Ok(a),
				Err(e) => f(e).evaluate(),
			}))
		}

		/// Maps both the success and error values simultaneously.
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
		#[document_returns("A new `TryThunk` with both sides transformed.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let ok: TryThunk<i32, i32> = TryThunk::pure(5);
		/// assert_eq!(ok.bimap(|x| x * 2, |e| e + 1).evaluate(), Ok(10));
		///
		/// let err: TryThunk<i32, i32> = TryThunk::err(5);
		/// assert_eq!(err.bimap(|x| x * 2, |e| e + 1).evaluate(), Err(6));
		/// ```
		pub fn bimap<B: 'a, E2: 'a>(
			self,
			f: impl FnOnce(A) -> B + 'a,
			g: impl FnOnce(E) -> E2 + 'a,
		) -> TryThunk<'a, B, E2> {
			TryThunk(self.0.map(|result| match result {
				Ok(a) => Ok(f(a)),
				Err(e) => Err(g(e)),
			}))
		}

		/// Forces evaluation and returns the result.
		#[document_signature]
		///
		#[document_returns("The result of the computation.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let try_thunk: TryThunk<i32, ()> = TryThunk::ok(42);
		/// assert_eq!(try_thunk.evaluate(), Ok(42));
		/// ```
		#[inline]
		pub fn evaluate(self) -> Result<A, E> {
			self.0.evaluate()
		}

		/// Combines two `TryThunk`s, running both and combining their results.
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
		#[document_returns("A new `TryThunk` producing the combined result.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let t1: TryThunk<i32, String> = TryThunk::ok(10);
		/// let t2: TryThunk<i32, String> = TryThunk::ok(20);
		/// let t3 = t1.lift2(t2, |a, b| a + b);
		/// assert_eq!(t3.evaluate(), Ok(30));
		///
		/// let t4: TryThunk<i32, String> = TryThunk::err("fail".to_string());
		/// let t5: TryThunk<i32, String> = TryThunk::ok(20);
		/// let t6 = t4.lift2(t5, |a, b| a + b);
		/// assert_eq!(t6.evaluate(), Err("fail".to_string()));
		/// ```
		#[inline]
		pub fn lift2<B: 'a, C: 'a>(
			self,
			other: TryThunk<'a, B, E>,
			f: impl FnOnce(A, B) -> C + 'a,
		) -> TryThunk<'a, C, E> {
			self.bind(move |a| other.map(move |b| f(a, b)))
		}

		/// Sequences two `TryThunk`s, discarding the first result.
		///
		/// Short-circuits on error: if `self` fails, `other` is never evaluated.
		#[document_signature]
		///
		#[document_type_parameters("The type of the second computation's success value.")]
		///
		#[document_parameters("The second computation.")]
		///
		#[document_returns(
			"A new `TryThunk` that runs both computations and returns the result of the second."
		)]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let t1: TryThunk<i32, String> = TryThunk::ok(10);
		/// let t2: TryThunk<i32, String> = TryThunk::ok(20);
		/// let t3 = t1.then(t2);
		/// assert_eq!(t3.evaluate(), Ok(20));
		///
		/// let t4: TryThunk<i32, String> = TryThunk::err("fail".to_string());
		/// let t5: TryThunk<i32, String> = TryThunk::ok(20);
		/// let t6 = t4.then(t5);
		/// assert_eq!(t6.evaluate(), Err("fail".to_string()));
		/// ```
		#[inline]
		pub fn then<B: 'a>(
			self,
			other: TryThunk<'a, B, E>,
		) -> TryThunk<'a, B, E> {
			self.bind(move |_| other)
		}

		/// Performs tail-recursive monadic computation with error handling.
		///
		/// The step function `f` is called in a loop, avoiding stack growth.
		/// Each iteration evaluates `f(state)` and inspects the resulting
		/// [`Result`] and [`ControlFlow`]: `Ok(ControlFlow::Continue(next))` continues with
		/// `next`, `Ok(ControlFlow::Break(a))` breaks out and returns `Ok(a)`, and
		/// `Err(e)` short-circuits with `Err(e)`.
		///
		/// Unlike the [`MonadRec`] implementation for
		/// [`TryThunkErrAppliedBrand<E>`](crate::brands::TryThunkErrAppliedBrand),
		/// this method does not require `E: 'static`, so it works with
		/// borrowed error types like `&'a str`.
		///
		/// # Step Function
		///
		/// The function `f` is bounded by `Fn`, so it is callable multiple
		/// times by shared reference. Each iteration of the loop calls `f`
		/// without consuming it.
		#[document_signature]
		///
		#[document_type_parameters("The type of the loop state.")]
		///
		#[document_parameters(
			"The step function that produces the next state, the final result, or an error.",
			"The initial state."
		)]
		///
		#[document_returns("A `TryThunk` that, when evaluated, runs the tail-recursive loop.")]
		///
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	core::ops::ControlFlow,
		/// 	fp_library::types::*,
		/// };
		///
		/// let result: TryThunk<i32, &str> = TryThunk::tail_rec_m(
		/// 	|x| {
		/// 		TryThunk::ok(
		/// 			if x < 1000 { ControlFlow::Continue(x + 1) } else { ControlFlow::Break(x) },
		/// 		)
		/// 	},
		/// 	0,
		/// );
		/// assert_eq!(result.evaluate(), Ok(1000));
		/// ```
		pub fn tail_rec_m<S>(
			f: impl Fn(S) -> TryThunk<'a, ControlFlow<A, S>, E> + 'a,
			initial: S,
		) -> Self
		where
			S: 'a, {
			TryThunk::new(move || {
				let mut state = initial;
				loop {
					match f(state).evaluate() {
						Ok(ControlFlow::Continue(next)) => state = next,
						Ok(ControlFlow::Break(a)) => break Ok(a),
						Err(e) => break Err(e),
					}
				}
			})
		}

		/// Arc-wrapped version of [`tail_rec_m`](TryThunk::tail_rec_m) for
		/// non-Clone closures.
		///
		/// Use this when your closure captures non-Clone state. The closure is
		/// wrapped in [`Arc`] internally, which provides the required `Clone`
		/// implementation. The step function must be `Send + Sync` because
		/// `Arc` requires these bounds.
		#[document_signature]
		///
		#[document_type_parameters("The type of the loop state.")]
		///
		#[document_parameters(
			"The step function that produces the next state, the final result, or an error.",
			"The initial state."
		)]
		///
		#[document_returns("A `TryThunk` that, when evaluated, runs the tail-recursive loop.")]
		///
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	core::ops::ControlFlow,
		/// 	fp_library::types::*,
		/// 	std::sync::{
		/// 		Arc,
		/// 		atomic::{
		/// 			AtomicUsize,
		/// 			Ordering,
		/// 		},
		/// 	},
		/// };
		///
		/// let counter = Arc::new(AtomicUsize::new(0));
		/// let counter_clone = Arc::clone(&counter);
		/// let result: TryThunk<i32, ()> = TryThunk::arc_tail_rec_m(
		/// 	move |x| {
		/// 		counter_clone.fetch_add(1, Ordering::SeqCst);
		/// 		TryThunk::ok(if x < 100 { ControlFlow::Continue(x + 1) } else { ControlFlow::Break(x) })
		/// 	},
		/// 	0,
		/// );
		/// assert_eq!(result.evaluate(), Ok(100));
		/// assert_eq!(counter.load(Ordering::SeqCst), 101);
		/// ```
		pub fn arc_tail_rec_m<S>(
			f: impl Fn(S) -> TryThunk<'a, ControlFlow<A, S>, E> + Send + Sync + 'a,
			initial: S,
		) -> Self
		where
			S: 'a, {
			let f = Arc::new(f);
			let wrapper = move |s: S| {
				let f = Arc::clone(&f);
				f(s)
			};
			Self::tail_rec_m(wrapper, initial)
		}

		/// Converts this `TryThunk` into a memoized [`RcTryLazy`].
		///
		/// The resulting `RcTryLazy` will evaluate the computation on first
		/// access and cache the result for subsequent accesses.
		#[document_signature]
		///
		#[document_returns("A memoized `RcTryLazy` wrapping this computation.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let thunk: TryThunk<i32, ()> = TryThunk::ok(42);
		/// let lazy: RcTryLazy<i32, ()> = thunk.into_rc_try_lazy();
		/// assert_eq!(lazy.evaluate(), Ok(&42));
		/// ```
		#[inline]
		pub fn into_rc_try_lazy(self) -> RcTryLazy<'a, A, E> {
			RcTryLazy::from(self)
		}

		/// Evaluates this `TryThunk` and wraps the result in a thread-safe [`ArcTryLazy`].
		///
		/// The thunk is evaluated eagerly because its inner closure is not
		/// `Send`. The result is stored in an `ArcTryLazy` for thread-safe sharing.
		#[document_signature]
		///
		#[document_returns("A thread-safe memoized `ArcTryLazy` wrapping this computation.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let thunk: TryThunk<i32, ()> = TryThunk::ok(42);
		/// let lazy: ArcTryLazy<i32, ()> = thunk.into_arc_try_lazy();
		/// assert_eq!(lazy.evaluate(), Ok(&42));
		/// ```
		#[inline]
		pub fn into_arc_try_lazy(self) -> ArcTryLazy<'a, A, E>
		where
			A: Send + Sync + 'a,
			E: Send + Sync + 'a, {
			let result = self.evaluate();
			ArcTryLazy::new(move || result)
		}
	}

	#[document_type_parameters(
		"The lifetime of the computation.",
		"The type of the computed value.",
		"The type of the error value."
	)]
	#[document_parameters("The `TryThunk` to operate on.")]
	impl<'a, A: 'a, E: 'a> TryThunk<'a, A, E> {
		/// Creates a `TryThunk` that catches unwinds (panics), converting the
		/// panic payload using a custom conversion function.
		///
		/// The closure `f` is executed when the thunk is evaluated. If `f`
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
			"A new `TryThunk` instance where panics are converted to `Err(E)` via the handler."
		)]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let thunk = TryThunk::<i32, i32>::catch_unwind_with(
		/// 	|| {
		/// 		if true {
		/// 			panic!("oops")
		/// 		}
		/// 		42
		/// 	},
		/// 	|_payload| -1,
		/// );
		/// assert_eq!(thunk.evaluate(), Err(-1));
		/// ```
		pub fn catch_unwind_with(
			f: impl FnOnce() -> A + std::panic::UnwindSafe + 'a,
			handler: impl FnOnce(Box<dyn std::any::Any + Send>) -> E + 'a,
		) -> Self {
			TryThunk::new(move || std::panic::catch_unwind(f).map_err(handler))
		}

		/// Unwraps the newtype, returning the inner `Thunk<'a, Result<A, E>>`.
		#[document_signature]
		///
		#[document_returns("The underlying `Thunk` that produces a `Result`.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let try_thunk: TryThunk<i32, ()> = TryThunk::pure(42);
		/// let inner = try_thunk.into_inner();
		/// assert_eq!(inner.evaluate(), Ok(42));
		/// ```
		pub fn into_inner(self) -> Thunk<'a, Result<A, E>> {
			self.0
		}
	}

	#[document_type_parameters(
		"The lifetime of the computation.",
		"The type of the computed value."
	)]
	impl<'a, A: 'a> TryThunk<'a, A, String> {
		/// Creates a `TryThunk` that catches unwinds (panics).
		///
		/// The closure is executed when the thunk is evaluated. If the closure
		/// panics, the panic payload is converted to a `String` error. If the
		/// closure returns normally, the value is wrapped in `Ok`.
		///
		/// This is a convenience wrapper around [`catch_unwind_with`](TryThunk::catch_unwind_with)
		/// that uses the default panic payload to string conversion.
		#[document_signature]
		///
		#[document_parameters("The closure that might panic.")]
		///
		#[document_returns(
			"A new `TryThunk` instance where panics are converted to `Err(String)`."
		)]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let thunk = TryThunk::<i32, String>::catch_unwind(|| {
		/// 	if true {
		/// 		panic!("oops")
		/// 	}
		/// 	42
		/// });
		/// assert_eq!(thunk.evaluate(), Err("oops".to_string()));
		/// ```
		pub fn catch_unwind(f: impl FnOnce() -> A + std::panic::UnwindSafe + 'a) -> Self {
			Self::catch_unwind_with(f, crate::utils::panic_payload_to_string)
		}
	}

	#[document_type_parameters(
		"The lifetime of the computation.",
		"The type of the success value.",
		"The type of the error value.",
		"The memoization configuration."
	)]
	impl<'a, A, E, Config> From<Lazy<'a, A, Config>> for TryThunk<'a, A, E>
	where
		A: Clone + 'a,
		E: 'a,
		Config: LazyConfig,
	{
		#[document_signature]
		#[document_parameters("The lazy value to convert.")]
		#[document_returns("A new `TryThunk` instance that wraps the lazy value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		/// let lazy = Lazy::<_, RcLazyConfig>::pure(42);
		/// let thunk: TryThunk<i32, ()> = TryThunk::from(lazy);
		/// assert_eq!(thunk.evaluate(), Ok(42));
		/// ```
		fn from(memo: Lazy<'a, A, Config>) -> Self {
			TryThunk::new(move || Ok(memo.evaluate().clone()))
		}
	}

	#[document_type_parameters(
		"The lifetime of the computation.",
		"The type of the success value.",
		"The type of the error value.",
		"The memoization configuration."
	)]
	impl<'a, A, E, Config> From<TryLazy<'a, A, E, Config>> for TryThunk<'a, A, E>
	where
		A: Clone + 'a,
		E: Clone + 'a,
		Config: LazyConfig,
	{
		/// Converts a [`TryLazy`] value into a [`TryThunk`] by cloning the memoized result.
		///
		/// This conversion clones both the success and error values on each evaluation.
		/// The cost depends on the [`Clone`] implementations of `A` and `E`.
		#[document_signature]
		#[document_parameters("The fallible lazy value to convert.")]
		#[document_returns("A new `TryThunk` instance that wraps the fallible lazy value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		/// let lazy = TryLazy::<_, _, RcLazyConfig>::new(|| Ok::<i32, ()>(42));
		/// let thunk = TryThunk::from(lazy);
		/// assert_eq!(thunk.evaluate(), Ok(42));
		/// ```
		fn from(memo: TryLazy<'a, A, E, Config>) -> Self {
			TryThunk::new(move || memo.evaluate().cloned().map_err(Clone::clone))
		}
	}

	#[document_type_parameters(
		"The lifetime of the computation.",
		"The type of the success value.",
		"The type of the error value."
	)]
	impl<'a, A: 'a, E: 'a> From<Thunk<'a, A>> for TryThunk<'a, A, E> {
		#[document_signature]
		#[document_parameters("The thunk to convert.")]
		#[document_returns("A new `TryThunk` instance that wraps the thunk.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		/// let thunk = Thunk::new(|| 42);
		/// let try_thunk: TryThunk<i32, ()> = TryThunk::from(thunk);
		/// assert_eq!(try_thunk.evaluate(), Ok(42));
		/// ```
		fn from(eval: Thunk<'a, A>) -> Self {
			TryThunk(eval.map(Ok))
		}
	}

	#[document_type_parameters(
		"The lifetime of the computation.",
		"The type of the success value.",
		"The type of the error value."
	)]
	impl<'a, A: 'a, E: 'a> From<Result<A, E>> for TryThunk<'a, A, E> {
		#[document_signature]
		#[document_parameters("The result to convert.")]
		#[document_returns("A new `TryThunk` instance that produces the result.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		/// let ok_thunk: TryThunk<i32, String> = TryThunk::from(Ok(42));
		/// assert_eq!(ok_thunk.evaluate(), Ok(42));
		///
		/// let err_thunk: TryThunk<i32, String> = TryThunk::from(Err("error".to_string()));
		/// assert_eq!(err_thunk.evaluate(), Err("error".to_string()));
		/// ```
		fn from(result: Result<A, E>) -> Self {
			TryThunk(Thunk::pure(result))
		}
	}

	#[document_type_parameters("The type of the success value.", "The type of the error value.")]
	impl<A: 'static, E: 'static> From<TryTrampoline<A, E>> for TryThunk<'static, A, E> {
		/// Converts a [`TryTrampoline`] into a `TryThunk`.
		///
		/// The resulting `TryThunk` will evaluate the trampoline when forced.
		#[document_signature]
		#[document_parameters("The fallible trampoline to convert.")]
		#[document_returns("A new `TryThunk` instance that evaluates the trampoline.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let tramp: TryTrampoline<i32, String> = TryTrampoline::ok(42);
		/// let thunk: TryThunk<i32, String> = TryThunk::from(tramp);
		/// assert_eq!(thunk.evaluate(), Ok(42));
		/// ```
		fn from(tramp: TryTrampoline<A, E>) -> Self {
			TryThunk::new(move || tramp.evaluate())
		}
	}

	#[document_type_parameters(
		"The lifetime of the computation.",
		"The type of the success value.",
		"The type of the error value."
	)]
	impl<'a, A: 'a, E: 'a> From<TrySendThunk<'a, A, E>> for TryThunk<'a, A, E> {
		/// Converts a [`TrySendThunk`] into a [`TryThunk`] by erasing the `Send` bound.
		///
		/// This delegates to the [`SendThunk`](crate::types::SendThunk) to
		/// [`Thunk`] conversion, which is a zero-cost unsizing coercion: the inner
		/// `Box<dyn FnOnce() -> Result<A, E> + Send + 'a>` is coerced to
		/// `Box<dyn FnOnce() -> Result<A, E> + 'a>`.
		#[document_signature]
		#[document_parameters("The send try-thunk to convert.")]
		#[document_returns("A `TryThunk` wrapping the same deferred computation.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		/// let send_thunk: TrySendThunk<i32, ()> = TrySendThunk::pure(42);
		/// let thunk: TryThunk<i32, ()> = TryThunk::from(send_thunk);
		/// assert_eq!(thunk.evaluate(), Ok(42));
		/// ```
		fn from(send_thunk: TrySendThunk<'a, A, E>) -> Self {
			TryThunk(Thunk::from(send_thunk.into_inner()))
		}
	}

	#[document_type_parameters(
		"The lifetime of the computation.",
		"The type of the success value.",
		"The type of the error value."
	)]
	impl<'a, A, E> Deferrable<'a> for TryThunk<'a, A, E>
	where
		A: 'a,
		E: 'a,
	{
		/// Creates a `TryThunk` from a computation that produces it.
		#[document_signature]
		///
		#[document_parameters("A thunk that produces the try thunk.")]
		///
		#[document_returns("The deferred try thunk.")]
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
		/// let task: TryThunk<i32, ()> = Deferrable::defer(|| TryThunk::ok(42));
		/// assert_eq!(task.evaluate(), Ok(42));
		/// ```
		fn defer(f: impl FnOnce() -> Self + 'a) -> Self
		where
			Self: Sized, {
			TryThunk::defer(f)
		}
	}

	impl_kind! {
		#[no_inferable_brand]
		impl<E: 'static> for TryThunkErrAppliedBrand<E> {
			#[document_default]
			type Of<'a, A: 'a>: 'a = TryThunk<'a, A, E>;
		}
	}

	#[document_type_parameters("The error type.")]
	impl<E: 'static> Functor for TryThunkErrAppliedBrand<E> {
		/// Maps a function over the result of a `TryThunk` computation.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the computation.",
			"The type of the value inside the `TryThunk`.",
			"The type of the result of the transformation."
		)]
		///
		#[document_parameters(
			"The function to apply to the result of the computation.",
			"The `TryThunk` instance."
		)]
		///
		#[document_returns("A new `TryThunk` instance with the transformed result.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let try_thunk: TryThunk<i32, ()> = pure::<TryThunkErrAppliedBrand<()>, _>(10);
		/// let mapped = explicit::map::<TryThunkErrAppliedBrand<()>, _, _, _, _>(|x| x * 2, try_thunk);
		/// assert_eq!(mapped.evaluate(), Ok(20));
		/// ```
		fn map<'a, A: 'a, B: 'a>(
			func: impl Fn(A) -> B + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			fa.map(func)
		}
	}

	#[document_type_parameters("The error type.")]
	impl<E: 'static> Pointed for TryThunkErrAppliedBrand<E> {
		/// Wraps a value in a `TryThunk` context.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the computation.",
			"The type of the value to wrap."
		)]
		///
		#[document_parameters("The value to wrap.")]
		///
		#[document_returns("A new `TryThunk` instance containing the value.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let try_thunk: TryThunk<i32, ()> = pure::<TryThunkErrAppliedBrand<()>, _>(42);
		/// assert_eq!(try_thunk.evaluate(), Ok(42));
		/// ```
		fn pure<'a, A: 'a>(a: A) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>) {
			TryThunk::ok(a)
		}
	}

	#[document_type_parameters("The error type.")]
	impl<E: 'static> Lift for TryThunkErrAppliedBrand<E> {
		/// Lifts a binary function into the `TryThunk` context.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the computation.",
			"The type of the first value.",
			"The type of the second value.",
			"The type of the result."
		)]
		///
		#[document_parameters(
			"The binary function to apply.",
			"The first `TryThunk`.",
			"The second `TryThunk`."
		)]
		///
		#[document_returns(
			"A new `TryThunk` instance containing the result of applying the function."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let eval1: TryThunk<i32, ()> = pure::<TryThunkErrAppliedBrand<()>, _>(10);
		/// let eval2: TryThunk<i32, ()> = pure::<TryThunkErrAppliedBrand<()>, _>(20);
		/// let result = explicit::lift2::<TryThunkErrAppliedBrand<()>, _, _, _, _, _, _>(
		/// 	|a, b| a + b,
		/// 	eval1,
		/// 	eval2,
		/// );
		/// assert_eq!(result.evaluate(), Ok(30));
		/// ```
		fn lift2<'a, A, B, C>(
			func: impl Fn(A, B) -> C + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			fb: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, C>)
		where
			A: Clone + 'a,
			B: Clone + 'a,
			C: 'a, {
			fa.bind(move |a| fb.map(move |b| func(a, b)))
		}
	}

	#[document_type_parameters("The error type.")]
	impl<E: 'static> ApplyFirst for TryThunkErrAppliedBrand<E> {}

	#[document_type_parameters("The error type.")]
	impl<E: 'static> ApplySecond for TryThunkErrAppliedBrand<E> {}

	#[document_type_parameters("The error type.")]
	impl<E: 'static> Semiapplicative for TryThunkErrAppliedBrand<E> {
		/// Applies a function wrapped in `TryThunk` to a value wrapped in `TryThunk`.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the computation.",
			"The brand of the cloneable function wrapper.",
			"The type of the input.",
			"The type of the result."
		)]
		///
		#[document_parameters(
			"The `TryThunk` containing the function.",
			"The `TryThunk` containing the value."
		)]
		///
		#[document_returns(
			"A new `TryThunk` instance containing the result of applying the function."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let func: TryThunk<_, ()> =
		/// 	pure::<TryThunkErrAppliedBrand<()>, _>(lift_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2));
		/// let val: TryThunk<_, ()> = pure::<TryThunkErrAppliedBrand<()>, _>(21);
		/// let result = apply::<RcFnBrand, TryThunkErrAppliedBrand<()>, _, _>(func, val);
		/// assert_eq!(result.evaluate(), Ok(42));
		/// ```
		fn apply<'a, FnBrand: 'a + CloneFn, A: 'a + Clone, B: 'a>(
			ff: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, <FnBrand as CloneFn>::Of<'a, A, B>>),
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			ff.bind(move |f| {
				fa.map(
					#[expect(clippy::redundant_closure, reason = "Required for move semantics")]
					move |a| f(a),
				)
			})
		}
	}

	#[document_type_parameters("The error type.")]
	impl<E: 'static> Semimonad for TryThunkErrAppliedBrand<E> {
		/// Chains `TryThunk` computations.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the computation.",
			"The type of the result of the first computation.",
			"The type of the result of the new computation."
		)]
		///
		#[document_parameters(
			"The first `TryThunk`.",
			"The function to apply to the result of the computation."
		)]
		///
		#[document_returns("A new `TryThunk` instance representing the chained computation.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let try_thunk: TryThunk<i32, ()> = pure::<TryThunkErrAppliedBrand<()>, _>(10);
		/// let result = explicit::bind::<TryThunkErrAppliedBrand<()>, _, _, _, _>(try_thunk, |x| {
		/// 	pure::<TryThunkErrAppliedBrand<()>, _>(x * 2)
		/// });
		/// assert_eq!(result.evaluate(), Ok(20));
		/// ```
		fn bind<'a, A: 'a, B: 'a>(
			ma: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			func: impl Fn(A) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) + 'a,
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			ma.bind(func)
		}
	}

	#[document_type_parameters("The error type.")]
	impl<E: 'static> MonadRec for TryThunkErrAppliedBrand<E> {
		/// Performs tail-recursive monadic computation.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the computation.",
			"The type of the initial value and loop state.",
			"The type of the result."
		)]
		///
		#[document_parameters("The step function.", "The initial value.")]
		///
		#[document_returns("The result of the computation.")]
		///
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	core::ops::ControlFlow,
		/// 	fp_library::{
		/// 		brands::*,
		/// 		classes::*,
		/// 		functions::*,
		/// 		types::*,
		/// 	},
		/// };
		///
		/// let result = tail_rec_m::<TryThunkErrAppliedBrand<()>, _, _>(
		/// 	|x| {
		/// 		pure::<TryThunkErrAppliedBrand<()>, _>(
		/// 			if x < 1000 { ControlFlow::Continue(x + 1) } else { ControlFlow::Break(x) },
		/// 		)
		/// 	},
		/// 	0,
		/// );
		/// assert_eq!(result.evaluate(), Ok(1000));
		/// ```
		fn tail_rec_m<'a, A: 'a, B: 'a>(
			f: impl Fn(
				A,
			)
				-> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, ControlFlow<B, A>>)
			+ 'a,
			a: A,
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			TryThunk::new(move || {
				let mut current = a;
				loop {
					match f(current).evaluate() {
						Ok(ControlFlow::Continue(next)) => current = next,
						Ok(ControlFlow::Break(res)) => break Ok(res),
						Err(e) => break Err(e),
					}
				}
			})
		}
	}

	#[document_type_parameters("The error type.")]
	impl<E: 'static> Foldable for TryThunkErrAppliedBrand<E> {
		/// Folds the `TryThunk` from the right.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the computation.",
			"The brand of the cloneable function to use.",
			"The type of the elements in the structure.",
			"The type of the accumulator."
		)]
		///
		#[document_parameters(
			"The function to apply to each element and the accumulator.",
			"The initial value of the accumulator.",
			"The `TryThunk` to fold."
		)]
		///
		#[document_returns("The final accumulator value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let try_thunk: TryThunk<i32, ()> = pure::<TryThunkErrAppliedBrand<()>, _>(10);
		/// let result = explicit::fold_right::<RcFnBrand, TryThunkErrAppliedBrand<()>, _, _, _, _>(
		/// 	|a, b| a + b,
		/// 	5,
		/// 	try_thunk,
		/// );
		/// assert_eq!(result, 15);
		/// ```
		fn fold_right<'a, FnBrand, A: 'a + Clone, B: 'a>(
			func: impl Fn(A, B) -> B + 'a,
			initial: B,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> B
		where
			FnBrand: CloneFn + 'a, {
			match fa.evaluate() {
				Ok(a) => func(a, initial),
				Err(_) => initial,
			}
		}

		/// Folds the `TryThunk` from the left.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the computation.",
			"The brand of the cloneable function to use.",
			"The type of the elements in the structure.",
			"The type of the accumulator."
		)]
		///
		#[document_parameters(
			"The function to apply to the accumulator and each element.",
			"The initial value of the accumulator.",
			"The `TryThunk` to fold."
		)]
		///
		#[document_returns("The final accumulator value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let try_thunk: TryThunk<i32, ()> = pure::<TryThunkErrAppliedBrand<()>, _>(10);
		/// let result = explicit::fold_left::<RcFnBrand, TryThunkErrAppliedBrand<()>, _, _, _, _>(
		/// 	|b, a| b + a,
		/// 	5,
		/// 	try_thunk,
		/// );
		/// assert_eq!(result, 15);
		/// ```
		fn fold_left<'a, FnBrand, A: 'a + Clone, B: 'a>(
			func: impl Fn(B, A) -> B + 'a,
			initial: B,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> B
		where
			FnBrand: CloneFn + 'a, {
			match fa.evaluate() {
				Ok(a) => func(initial, a),
				Err(_) => initial,
			}
		}

		/// Maps the value to a monoid and returns it.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the computation.",
			"The brand of the cloneable function to use.",
			"The type of the elements in the structure.",
			"The type of the monoid."
		)]
		///
		#[document_parameters("The mapping function.", "The TryThunk to fold.")]
		///
		#[document_returns("The monoid value.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let try_thunk: TryThunk<i32, ()> = pure::<TryThunkErrAppliedBrand<()>, _>(10);
		/// let result = explicit::fold_map::<RcFnBrand, TryThunkErrAppliedBrand<()>, _, _, _, _>(
		/// 	|a: i32| a.to_string(),
		/// 	try_thunk,
		/// );
		/// assert_eq!(result, "10");
		/// ```
		fn fold_map<'a, FnBrand, A: 'a + Clone, M>(
			func: impl Fn(A) -> M + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> M
		where
			M: Monoid + 'a,
			FnBrand: CloneFn + 'a, {
			match fa.evaluate() {
				Ok(a) => func(a),
				Err(_) => M::empty(),
			}
		}
	}

	#[document_type_parameters(
		"The lifetime of the computation.",
		"The success value type.",
		"The error value type."
	)]
	impl<'a, A: Semigroup + 'a, E: 'a> Semigroup for TryThunk<'a, A, E> {
		/// Combines two `TryThunk`s by combining their results.
		#[document_signature]
		///
		#[document_parameters("The first `TryThunk`.", "The second `TryThunk`.")]
		///
		#[document_returns("A new `TryThunk` containing the combined result.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let t1: TryThunk<String, ()> = pure::<TryThunkErrAppliedBrand<()>, _>("Hello".to_string());
		/// let t2: TryThunk<String, ()> = pure::<TryThunkErrAppliedBrand<()>, _>(" World".to_string());
		/// let t3 = append::<_>(t1, t2);
		/// assert_eq!(t3.evaluate(), Ok("Hello World".to_string()));
		/// ```
		fn append(
			a: Self,
			b: Self,
		) -> Self {
			TryThunk::new(move || {
				let a_val = a.evaluate()?;
				let b_val = b.evaluate()?;
				Ok(Semigroup::append(a_val, b_val))
			})
		}
	}

	#[document_type_parameters(
		"The lifetime of the computation.",
		"The success value type.",
		"The error value type."
	)]
	impl<'a, A: Monoid + 'a, E: 'a> Monoid for TryThunk<'a, A, E> {
		/// Returns the identity `TryThunk`.
		#[document_signature]
		///
		#[document_returns("A `TryThunk` producing the identity value of `A`.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	classes::*,
		/// 	types::*,
		/// };
		///
		/// let t: TryThunk<String, ()> = TryThunk::empty();
		/// assert_eq!(t.evaluate(), Ok("".to_string()));
		/// ```
		fn empty() -> Self {
			TryThunk(Thunk::pure(Ok(Monoid::empty())))
		}
	}

	impl_kind! {
		/// HKT branding for the `TryThunk` type.
		///
		/// The type parameters for `Of` are ordered `E`, then `A` (Error, then Success).
		/// This follows the same convention as `ResultBrand`, matching functional
		/// programming expectations (like Haskell's `Either e a`) where the success
		/// type is the last parameter.
		for TryThunkBrand {
			type Of<'a, E: 'a, A: 'a>: 'a = TryThunk<'a, A, E>;
		}
	}

	impl Bifunctor for TryThunkBrand {
		/// Maps functions over the values in the `TryThunk`.
		///
		/// This method applies one function to the error value and another to the success value.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The type of the error value.",
			"The type of the mapped error value.",
			"The type of the success value.",
			"The type of the mapped success value."
		)]
		///
		#[document_parameters(
			"The function to apply to the error.",
			"The function to apply to the success.",
			"The `TryThunk` to map over."
		)]
		///
		#[document_returns("A new `TryThunk` containing the mapped values.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let x: TryThunk<i32, i32> = TryThunk::ok(5);
		/// assert_eq!(
		/// 	explicit::bimap::<TryThunkBrand, _, _, _, _, _, _>((|e| e + 1, |s| s * 2), x).evaluate(),
		/// 	Ok(10)
		/// );
		///
		/// let y: TryThunk<i32, i32> = TryThunk::err(5);
		/// assert_eq!(
		/// 	explicit::bimap::<TryThunkBrand, _, _, _, _, _, _>((|e| e + 1, |s| s * 2), y).evaluate(),
		/// 	Err(6)
		/// );
		/// ```
		fn bimap<'a, A: 'a, B: 'a, C: 'a, D: 'a>(
			f: impl Fn(A) -> B + 'a,
			g: impl Fn(C) -> D + 'a,
			p: Apply!(<Self as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, A, C>),
		) -> Apply!(<Self as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, B, D>) {
			TryThunk(p.0.map(move |result| match result {
				Ok(c) => Ok(g(c)),
				Err(a) => Err(f(a)),
			}))
		}
	}

	impl Bifoldable for TryThunkBrand {
		/// Folds a `TryThunk` using two step functions, right-associatively.
		///
		/// Dispatches to `f` for error values and `g` for success values.
		/// The thunk is evaluated to determine which branch to fold.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The brand of the cloneable function to use.",
			"The error type (first position).",
			"The success type (second position).",
			"The accumulator type."
		)]
		///
		#[document_parameters(
			"The step function applied to the error value.",
			"The step function applied to the success value.",
			"The initial accumulator.",
			"The `TryThunk` to fold."
		)]
		///
		#[document_returns("`f(e, z)` for `Err(e)`, or `g(a, z)` for `Ok(a)`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// assert_eq!(
		/// 	explicit::bi_fold_right::<RcFnBrand, TryThunkBrand, _, _, _, _, _>(
		/// 		(|e: i32, acc| acc - e, |s: i32, acc| acc + s),
		/// 		10,
		/// 		TryThunk::err(3),
		/// 	),
		/// 	7
		/// );
		/// assert_eq!(
		/// 	explicit::bi_fold_right::<RcFnBrand, TryThunkBrand, _, _, _, _, _>(
		/// 		(|e: i32, acc| acc - e, |s: i32, acc| acc + s),
		/// 		10,
		/// 		TryThunk::ok(5),
		/// 	),
		/// 	15
		/// );
		/// ```
		fn bi_fold_right<'a, FnBrand: CloneFn + 'a, A: 'a + Clone, B: 'a + Clone, C: 'a>(
			f: impl Fn(A, C) -> C + 'a,
			g: impl Fn(B, C) -> C + 'a,
			z: C,
			p: Apply!(<Self as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, A, B>),
		) -> C {
			match p.evaluate() {
				Err(a) => f(a, z),
				Ok(b) => g(b, z),
			}
		}

		/// Folds a `TryThunk` using two step functions, left-associatively.
		///
		/// Dispatches to `f` for error values and `g` for success values.
		/// The thunk is evaluated to determine which branch to fold.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The brand of the cloneable function to use.",
			"The error type (first position).",
			"The success type (second position).",
			"The accumulator type."
		)]
		///
		#[document_parameters(
			"The step function applied to the error value.",
			"The step function applied to the success value.",
			"The initial accumulator.",
			"The `TryThunk` to fold."
		)]
		///
		#[document_returns("`f(z, e)` for `Err(e)`, or `g(z, a)` for `Ok(a)`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// assert_eq!(
		/// 	explicit::bi_fold_left::<RcFnBrand, TryThunkBrand, _, _, _, _, _>(
		/// 		(|acc, e: i32| acc - e, |acc, s: i32| acc + s),
		/// 		10,
		/// 		TryThunk::err(3),
		/// 	),
		/// 	7
		/// );
		/// assert_eq!(
		/// 	explicit::bi_fold_left::<RcFnBrand, TryThunkBrand, _, _, _, _, _>(
		/// 		(|acc, e: i32| acc - e, |acc, s: i32| acc + s),
		/// 		10,
		/// 		TryThunk::ok(5),
		/// 	),
		/// 	15
		/// );
		/// ```
		fn bi_fold_left<'a, FnBrand: CloneFn + 'a, A: 'a + Clone, B: 'a + Clone, C: 'a>(
			f: impl Fn(C, A) -> C + 'a,
			g: impl Fn(C, B) -> C + 'a,
			z: C,
			p: Apply!(<Self as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, A, B>),
		) -> C {
			match p.evaluate() {
				Err(a) => f(z, a),
				Ok(b) => g(z, b),
			}
		}

		/// Maps a `TryThunk`'s value to a monoid using two functions and returns the result.
		///
		/// Dispatches to `f` for error values and `g` for success values,
		/// returning the monoid value.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The brand of the cloneable function to use.",
			"The error type (first position).",
			"The success type (second position).",
			"The monoid type."
		)]
		///
		#[document_parameters(
			"The function mapping the error to the monoid.",
			"The function mapping the success to the monoid.",
			"The `TryThunk` to fold."
		)]
		///
		#[document_returns("`f(e)` for `Err(e)`, or `g(a)` for `Ok(a)`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// assert_eq!(
		/// 	explicit::bi_fold_map::<RcFnBrand, TryThunkBrand, _, _, _, _, _>(
		/// 		(|e: i32| e.to_string(), |s: i32| s.to_string()),
		/// 		TryThunk::err(3),
		/// 	),
		/// 	"3".to_string()
		/// );
		/// assert_eq!(
		/// 	explicit::bi_fold_map::<RcFnBrand, TryThunkBrand, _, _, _, _, _>(
		/// 		(|e: i32| e.to_string(), |s: i32| s.to_string()),
		/// 		TryThunk::ok(5),
		/// 	),
		/// 	"5".to_string()
		/// );
		/// ```
		fn bi_fold_map<'a, FnBrand: CloneFn + 'a, A: 'a + Clone, B: 'a + Clone, M>(
			f: impl Fn(A) -> M + 'a,
			g: impl Fn(B) -> M + 'a,
			p: Apply!(<Self as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, A, B>),
		) -> M
		where
			M: Monoid + 'a, {
			match p.evaluate() {
				Err(a) => f(a),
				Ok(b) => g(b),
			}
		}
	}

	impl_kind! {
		#[no_inferable_brand]
		/// HKT branding for `TryThunk` with the success type `A` fixed.
		///
		/// This is the "dual-channel" encoding for `TryThunk`:
		/// - [`TryThunkErrAppliedBrand<E>`] fixes the error type and is polymorphic over `Ok` values,
		///   giving a standard `Functor`/`Monad` that maps and chains success values.
		/// - `TryThunkOkAppliedBrand<A>` fixes the success type and is polymorphic over `Err` values,
		///   giving a `Functor`/`Monad` that maps and chains error values.
		///
		/// Together they allow the same `TryThunk<'a, A, E>` to participate in HKT abstractions
		/// on either channel. For example, `pure::<TryThunkErrAppliedBrand<E>, _>(x)` produces
		/// `Ok(x)`, while `pure::<TryThunkOkAppliedBrand<A>, _>(e)` produces `Err(e)`.
		impl<A: 'static> for TryThunkOkAppliedBrand<A> {
			#[document_default]
			type Of<'a, E: 'a>: 'a = TryThunk<'a, A, E>;
		}
	}

	#[document_type_parameters("The success type.")]
	impl<A: 'static> Functor for TryThunkOkAppliedBrand<A> {
		/// Maps a function over the error value in the `TryThunk`.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the computation.",
			"The type of the error value inside the `TryThunk`.",
			"The type of the result of the transformation."
		)]
		///
		#[document_parameters("The function to apply to the error.", "The `TryThunk` instance.")]
		///
		#[document_returns("A new `TryThunk` instance with the transformed error.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let try_thunk: TryThunk<i32, i32> = pure::<TryThunkOkAppliedBrand<i32>, _>(10);
		/// let mapped = explicit::map::<TryThunkOkAppliedBrand<i32>, _, _, _, _>(|x| x * 2, try_thunk);
		/// assert_eq!(mapped.evaluate(), Err(20));
		/// ```
		fn map<'a, E: 'a, E2: 'a>(
			func: impl Fn(E) -> E2 + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, E>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, E2>) {
			fa.map_err(func)
		}
	}

	#[document_type_parameters("The success type.")]
	impl<A: 'static> Pointed for TryThunkOkAppliedBrand<A> {
		/// Wraps a value in a `TryThunk` context (as error).
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the computation.",
			"The type of the value to wrap."
		)]
		///
		#[document_parameters("The value to wrap.")]
		///
		#[document_returns("A new `TryThunk` instance containing the value as an error.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let try_thunk: TryThunk<i32, i32> = pure::<TryThunkOkAppliedBrand<i32>, _>(42);
		/// assert_eq!(try_thunk.evaluate(), Err(42));
		/// ```
		fn pure<'a, E: 'a>(e: E) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, E>) {
			TryThunk::err(e)
		}
	}

	#[document_type_parameters("The success type.")]
	impl<A: 'static> Lift for TryThunkOkAppliedBrand<A> {
		/// Lifts a binary function into the `TryThunk` context (over error).
		///
		/// # Evaluation strategy
		///
		/// This implementation uses **fail-fast** semantics, consistent with
		/// [`bind`](TryThunk::bind): `fa` is evaluated first, and if it is `Ok`,
		/// the result is returned immediately without evaluating `fb`. If `fa` is
		/// `Err`, `fb` is evaluated next; if `fb` is `Ok`, that `Ok` is returned.
		/// The function is only called when both sides are `Err`.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the computation.",
			"The type of the first error value.",
			"The type of the second error value.",
			"The type of the result error value."
		)]
		///
		#[document_parameters(
			"The binary function to apply to the errors.",
			"The first `TryThunk`.",
			"The second `TryThunk`."
		)]
		///
		#[document_returns(
			"A new `TryThunk` instance containing the result of applying the function to the errors."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let eval1: TryThunk<i32, i32> = pure::<TryThunkOkAppliedBrand<i32>, _>(10);
		/// let eval2: TryThunk<i32, i32> = pure::<TryThunkOkAppliedBrand<i32>, _>(20);
		/// let result = explicit::lift2::<TryThunkOkAppliedBrand<i32>, _, _, _, _, _, _>(
		/// 	|a, b| a + b,
		/// 	eval1,
		/// 	eval2,
		/// );
		/// assert_eq!(result.evaluate(), Err(30));
		/// ```
		fn lift2<'a, E1, E2, E3>(
			func: impl Fn(E1, E2) -> E3 + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, E1>),
			fb: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, E2>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, E3>)
		where
			E1: Clone + 'a,
			E2: Clone + 'a,
			E3: 'a, {
			TryThunk::new(move || match fa.evaluate() {
				Ok(a) => Ok(a),
				Err(e1) => match fb.evaluate() {
					Ok(a) => Ok(a),
					Err(e2) => Err(func(e1, e2)),
				},
			})
		}
	}

	#[document_type_parameters("The success type.")]
	impl<A: 'static> ApplyFirst for TryThunkOkAppliedBrand<A> {}

	#[document_type_parameters("The success type.")]
	impl<A: 'static> ApplySecond for TryThunkOkAppliedBrand<A> {}

	#[document_type_parameters("The success type.")]
	impl<A: 'static> Semiapplicative for TryThunkOkAppliedBrand<A> {
		/// Applies a function wrapped in `TryThunk` (as error) to a value wrapped in `TryThunk` (as error).
		///
		/// # Evaluation strategy
		///
		/// This implementation uses **fail-fast** semantics, consistent with
		/// [`bind`](TryThunk::bind): `ff` is evaluated first, and if it is `Ok`,
		/// the result is returned immediately without evaluating `fa`. If `ff` is
		/// `Err`, `fa` is evaluated next; if `fa` is `Ok`, that `Ok` is returned.
		/// The function is only applied when both sides are `Err`.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the computation.",
			"The brand of the cloneable function wrapper.",
			"The type of the input error.",
			"The type of the result error."
		)]
		///
		#[document_parameters(
			"The `TryThunk` containing the function (in Err).",
			"The `TryThunk` containing the value (in Err)."
		)]
		///
		#[document_returns(
			"A new `TryThunk` instance containing the result of applying the function."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let func: TryThunk<i32, _> =
		/// 	pure::<TryThunkOkAppliedBrand<i32>, _>(lift_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2));
		/// let val: TryThunk<i32, _> = pure::<TryThunkOkAppliedBrand<i32>, _>(21);
		/// let result = apply::<RcFnBrand, TryThunkOkAppliedBrand<i32>, _, _>(func, val);
		/// assert_eq!(result.evaluate(), Err(42));
		/// ```
		fn apply<'a, FnBrand: 'a + CloneFn, E1: 'a + Clone, E2: 'a>(
			ff: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, <FnBrand as CloneFn>::Of<'a, E1, E2>>),
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, E1>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, E2>) {
			TryThunk::new(move || match ff.evaluate() {
				Ok(a) => Ok(a),
				Err(f) => match fa.evaluate() {
					Ok(a) => Ok(a),
					Err(e) => Err(f(e)),
				},
			})
		}
	}

	#[document_type_parameters("The success type.")]
	impl<A: 'static> Semimonad for TryThunkOkAppliedBrand<A> {
		/// Chains `TryThunk` computations (over error).
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the computation.",
			"The type of the result of the first computation (error).",
			"The type of the result of the new computation (error)."
		)]
		///
		#[document_parameters(
			"The first `TryThunk`.",
			"The function to apply to the error result of the computation."
		)]
		///
		#[document_returns("A new `TryThunk` instance representing the chained computation.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let try_thunk: TryThunk<i32, i32> = pure::<TryThunkOkAppliedBrand<i32>, _>(10);
		/// let result = explicit::bind::<TryThunkOkAppliedBrand<i32>, _, _, _, _>(try_thunk, |x| {
		/// 	pure::<TryThunkOkAppliedBrand<i32>, _>(x * 2)
		/// });
		/// assert_eq!(result.evaluate(), Err(20));
		/// ```
		fn bind<'a, E1: 'a, E2: 'a>(
			ma: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, E1>),
			func: impl Fn(E1) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, E2>) + 'a,
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, E2>) {
			TryThunk::new(move || match ma.evaluate() {
				Ok(a) => Ok(a),
				Err(e) => func(e).evaluate(),
			})
		}
	}

	#[document_type_parameters("The success type.")]
	impl<A: 'static> MonadRec for TryThunkOkAppliedBrand<A> {
		/// Performs tail-recursive monadic computation over the error channel.
		///
		/// The step function returns `TryThunk<A, ControlFlow<E2, E>>`. The loop
		/// continues while the error is `ControlFlow::Continue`, terminates with the
		/// final error on `ControlFlow::Break`, and short-circuits on `Ok`.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the computation.",
			"The type of the initial value and loop state.",
			"The type of the result."
		)]
		///
		#[document_parameters("The step function.", "The initial value.")]
		///
		#[document_returns("The result of the computation.")]
		///
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	core::ops::ControlFlow,
		/// 	fp_library::{
		/// 		brands::*,
		/// 		classes::*,
		/// 		functions::*,
		/// 		types::*,
		/// 	},
		/// };
		///
		/// let result = tail_rec_m::<TryThunkOkAppliedBrand<i32>, _, _>(
		/// 	|x| {
		/// 		pure::<TryThunkOkAppliedBrand<i32>, _>(
		/// 			if x < 1000 { ControlFlow::Continue(x + 1) } else { ControlFlow::Break(x) },
		/// 		)
		/// 	},
		/// 	0,
		/// );
		/// assert_eq!(result.evaluate(), Err(1000));
		/// ```
		fn tail_rec_m<'a, E: 'a, E2: 'a>(
			f: impl Fn(
				E,
			)
				-> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, ControlFlow<E2, E>>)
			+ 'a,
			e: E,
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, E2>) {
			TryThunk::new(move || {
				let mut current = e;
				loop {
					match f(current).evaluate() {
						Err(ControlFlow::Continue(next)) => current = next,
						Err(ControlFlow::Break(res)) => break Err(res),
						Ok(a) => break Ok(a),
					}
				}
			})
		}
	}

	#[document_type_parameters("The success type.")]
	impl<A: 'static> Foldable for TryThunkOkAppliedBrand<A> {
		/// Folds the `TryThunk` from the right (over error).
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the computation.",
			"The brand of the cloneable function to use.",
			"The type of the elements in the structure.",
			"The type of the accumulator."
		)]
		///
		#[document_parameters(
			"The function to apply to each element and the accumulator.",
			"The initial value of the accumulator.",
			"The `TryThunk` to fold."
		)]
		///
		#[document_returns("The final accumulator value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let try_thunk: TryThunk<i32, i32> = pure::<TryThunkOkAppliedBrand<i32>, _>(10);
		/// let result = explicit::fold_right::<RcFnBrand, TryThunkOkAppliedBrand<i32>, _, _, _, _>(
		/// 	|a, b| a + b,
		/// 	5,
		/// 	try_thunk,
		/// );
		/// assert_eq!(result, 15);
		/// ```
		fn fold_right<'a, FnBrand, E: 'a + Clone, B: 'a>(
			func: impl Fn(E, B) -> B + 'a,
			initial: B,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, E>),
		) -> B
		where
			FnBrand: CloneFn + 'a, {
			match fa.evaluate() {
				Err(e) => func(e, initial),
				Ok(_) => initial,
			}
		}

		/// Folds the `TryThunk` from the left (over error).
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the computation.",
			"The brand of the cloneable function to use.",
			"The type of the elements in the structure.",
			"The type of the accumulator."
		)]
		///
		#[document_parameters(
			"The function to apply to the accumulator and each element.",
			"The initial value of the accumulator.",
			"The `TryThunk` to fold."
		)]
		///
		#[document_returns("The final accumulator value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let try_thunk: TryThunk<i32, i32> = pure::<TryThunkOkAppliedBrand<i32>, _>(10);
		/// let result = explicit::fold_left::<RcFnBrand, TryThunkOkAppliedBrand<i32>, _, _, _, _>(
		/// 	|b, a| b + a,
		/// 	5,
		/// 	try_thunk,
		/// );
		/// assert_eq!(result, 15);
		/// ```
		fn fold_left<'a, FnBrand, E: 'a + Clone, B: 'a>(
			func: impl Fn(B, E) -> B + 'a,
			initial: B,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, E>),
		) -> B
		where
			FnBrand: CloneFn + 'a, {
			match fa.evaluate() {
				Err(e) => func(initial, e),
				Ok(_) => initial,
			}
		}

		/// Maps the value to a monoid and returns it (over error).
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the computation.",
			"The brand of the cloneable function to use.",
			"The type of the elements in the structure.",
			"The type of the monoid."
		)]
		///
		#[document_parameters("The mapping function.", "The TryThunk to fold.")]
		///
		#[document_returns("The monoid value.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let try_thunk: TryThunk<i32, i32> = pure::<TryThunkOkAppliedBrand<i32>, _>(10);
		/// let result = explicit::fold_map::<RcFnBrand, TryThunkOkAppliedBrand<i32>, _, _, _, _>(
		/// 	|a: i32| a.to_string(),
		/// 	try_thunk,
		/// );
		/// assert_eq!(result, "10");
		/// ```
		fn fold_map<'a, FnBrand, E: 'a + Clone, M>(
			func: impl Fn(E) -> M + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, E>),
		) -> M
		where
			M: Monoid + 'a,
			FnBrand: CloneFn + 'a, {
			match fa.evaluate() {
				Err(e) => func(e),
				Ok(_) => M::empty(),
			}
		}
	}

	#[document_type_parameters(
		"The lifetime of the computation.",
		"The type of the success value.",
		"The type of the error value."
	)]
	#[document_parameters("The try-thunk to format.")]
	impl<'a, A, E> fmt::Debug for TryThunk<'a, A, E> {
		/// Formats the try-thunk without evaluating it.
		#[document_signature]
		#[document_parameters("The formatter.")]
		#[document_returns("The formatting result.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		/// let thunk = TryThunk::new(|| Ok::<i32, ()>(42));
		/// assert_eq!(format!("{:?}", thunk), "TryThunk(<unevaluated>)");
		/// ```
		fn fmt(
			&self,
			f: &mut fmt::Formatter<'_>,
		) -> fmt::Result {
			f.write_str("TryThunk(<unevaluated>)")
		}
	}

	#[document_type_parameters("The error type.")]
	impl<E: 'static> WithIndex for TryThunkErrAppliedBrand<E> {
		type Index = ();
	}

	#[document_type_parameters("The error type.")]
	impl<E: 'static> FunctorWithIndex for TryThunkErrAppliedBrand<E> {
		/// Maps a function over the success value in the `TryThunk`, providing the index `()`.
		#[document_signature]
		#[document_type_parameters(
			"The lifetime of the computation.",
			"The type of the success value inside the `TryThunk`.",
			"The type of the result of applying the function."
		)]
		#[document_parameters(
			"The function to apply to the value and its index.",
			"The `TryThunk` to map over."
		)]
		#[document_returns(
			"A new `TryThunk` containing the result of applying the function, or the original error."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::TryThunkErrAppliedBrand,
		/// 	classes::functor_with_index::FunctorWithIndex,
		/// 	types::*,
		/// };
		///
		/// let x: TryThunk<i32, ()> = TryThunk::pure(5);
		/// let y = <TryThunkErrAppliedBrand<()> as FunctorWithIndex>::map_with_index(|_, i| i * 2, x);
		/// assert_eq!(y.evaluate(), Ok(10));
		/// ```
		fn map_with_index<'a, A: 'a, B: 'a>(
			f: impl Fn((), A) -> B + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			fa.map(move |a| f((), a))
		}
	}

	#[document_type_parameters("The error type.")]
	impl<E: 'static> FoldableWithIndex for TryThunkErrAppliedBrand<E> {
		/// Folds the `TryThunk` using a monoid, providing the index `()`.
		#[document_signature]
		#[document_type_parameters(
			"The lifetime of the computation.",
			"The brand of the cloneable function to use.",
			"The type of the success value inside the `TryThunk`.",
			"The monoid type."
		)]
		#[document_parameters(
			"The function to apply to the value and its index.",
			"The `TryThunk` to fold."
		)]
		#[document_returns("The monoid value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::foldable_with_index::FoldableWithIndex,
		/// 	types::*,
		/// };
		///
		/// let x: TryThunk<i32, ()> = TryThunk::pure(5);
		/// let y = <TryThunkErrAppliedBrand<()> as FoldableWithIndex>::fold_map_with_index::<
		/// 	RcFnBrand,
		/// 	_,
		/// 	_,
		/// >(|_, i: i32| i.to_string(), x);
		/// assert_eq!(y, "5".to_string());
		/// ```
		fn fold_map_with_index<'a, FnBrand, A: 'a + Clone, R: Monoid>(
			f: impl Fn((), A) -> R + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> R
		where
			FnBrand: LiftFn + 'a, {
			match fa.evaluate() {
				Ok(a) => f((), a),
				Err(_) => R::empty(),
			}
		}
	}

	#[document_type_parameters("The success type.")]
	impl<A: 'static> WithIndex for TryThunkOkAppliedBrand<A> {
		type Index = ();
	}

	#[document_type_parameters("The success type.")]
	impl<A: 'static> FunctorWithIndex for TryThunkOkAppliedBrand<A> {
		/// Maps a function over the error value in the `TryThunk`, providing the index `()`.
		#[document_signature]
		#[document_type_parameters(
			"The lifetime of the computation.",
			"The type of the error value inside the `TryThunk`.",
			"The type of the result of applying the function."
		)]
		#[document_parameters(
			"The function to apply to the error and its index.",
			"The `TryThunk` to map over."
		)]
		#[document_returns(
			"A new `TryThunk` containing the original success or the transformed error."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::TryThunkOkAppliedBrand,
		/// 	classes::functor_with_index::FunctorWithIndex,
		/// 	types::*,
		/// };
		///
		/// let x: TryThunk<i32, i32> = TryThunk::err(5);
		/// let y = <TryThunkOkAppliedBrand<i32> as FunctorWithIndex>::map_with_index(|_, e| e * 2, x);
		/// assert_eq!(y.evaluate(), Err(10));
		/// ```
		fn map_with_index<'a, E: 'a, E2: 'a>(
			f: impl Fn((), E) -> E2 + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, E>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, E2>) {
			fa.map_err(move |e| f((), e))
		}
	}

	#[document_type_parameters("The success type.")]
	impl<A: 'static> FoldableWithIndex for TryThunkOkAppliedBrand<A> {
		/// Folds the `TryThunk` over the error using a monoid, providing the index `()`.
		#[document_signature]
		#[document_type_parameters(
			"The lifetime of the computation.",
			"The brand of the cloneable function to use.",
			"The type of the error value inside the `TryThunk`.",
			"The monoid type."
		)]
		#[document_parameters(
			"The function to apply to the error and its index.",
			"The `TryThunk` to fold."
		)]
		#[document_returns("The monoid value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::foldable_with_index::FoldableWithIndex,
		/// 	types::*,
		/// };
		///
		/// let x: TryThunk<i32, i32> = TryThunk::err(5);
		/// let y = <TryThunkOkAppliedBrand<i32> as FoldableWithIndex>::fold_map_with_index::<
		/// 	RcFnBrand,
		/// 	_,
		/// 	_,
		/// >(|_, e: i32| e.to_string(), x);
		/// assert_eq!(y, "5".to_string());
		/// ```
		fn fold_map_with_index<'a, FnBrand, E: 'a + Clone, R: Monoid>(
			f: impl Fn((), E) -> R + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, E>),
		) -> R
		where
			FnBrand: LiftFn + 'a, {
			match fa.evaluate() {
				Err(e) => f((), e),
				Ok(_) => R::empty(),
			}
		}
	}
}
pub use inner::*;

#[cfg(test)]
#[expect(
	clippy::unwrap_used,
	clippy::panic,
	reason = "Tests use panicking operations for brevity and clarity"
)]
mod tests {
	use {
		super::*,
		crate::types::Thunk,
		quickcheck_macros::quickcheck,
	};

	/// Tests success path.
	///
	/// Verifies that `TryThunk::ok` creates a successful computation.
	#[test]
	fn test_success() {
		let try_thunk: TryThunk<i32, ()> = TryThunk::ok(42);
		assert_eq!(try_thunk.evaluate(), Ok(42));
	}

	/// Tests that `TryThunk::pure` creates a successful computation.
	#[test]
	fn test_pure() {
		let try_thunk: TryThunk<i32, ()> = TryThunk::pure(42);
		assert_eq!(try_thunk.evaluate(), Ok(42));
	}

	/// Tests failure path.
	///
	/// Verifies that `TryThunk::err` creates a failed computation.
	#[test]
	fn test_failure() {
		let try_thunk: TryThunk<i32, &str> = TryThunk::err("error");
		assert_eq!(try_thunk.evaluate(), Err("error"));
	}

	/// Tests `TryThunk::map`.
	///
	/// Verifies that `map` transforms the success value.
	#[test]
	fn test_map() {
		let try_thunk: TryThunk<i32, ()> = TryThunk::ok(21).map(|x| x * 2);
		assert_eq!(try_thunk.evaluate(), Ok(42));
	}

	/// Tests `TryThunk::map_err`.
	///
	/// Verifies that `map_err` transforms the error value.
	#[test]
	fn test_map_err() {
		let try_thunk: TryThunk<i32, i32> = TryThunk::err(21).map_err(|x| x * 2);
		assert_eq!(try_thunk.evaluate(), Err(42));
	}

	/// Tests `TryThunk::bind`.
	///
	/// Verifies that `bind` chains computations.
	#[test]
	fn test_bind() {
		let try_thunk: TryThunk<i32, ()> = TryThunk::ok(21).bind(|x| TryThunk::ok(x * 2));
		assert_eq!(try_thunk.evaluate(), Ok(42));
	}

	/// Tests borrowing in TryThunk.
	///
	/// Verifies that `TryThunk` can capture references.
	#[test]
	fn test_borrowing() {
		let x = 42;
		let try_thunk: TryThunk<&i32, ()> = TryThunk::new(|| Ok(&x));
		assert_eq!(try_thunk.evaluate(), Ok(&42));
	}

	/// Tests `TryThunk::bind` failure propagation.
	///
	/// Verifies that if the first computation fails, the second one is not executed.
	#[test]
	fn test_bind_failure() {
		let try_thunk = TryThunk::<i32, &str>::err("error").bind(|x| TryThunk::ok(x * 2));
		assert_eq!(try_thunk.evaluate(), Err("error"));
	}

	/// Tests `TryThunk::map` failure propagation.
	///
	/// Verifies that `map` is not executed if the computation fails.
	#[test]
	fn test_map_failure() {
		let try_thunk = TryThunk::<i32, &str>::err("error").map(|x| x * 2);
		assert_eq!(try_thunk.evaluate(), Err("error"));
	}

	/// Tests `TryThunk::map_err` success propagation.
	///
	/// Verifies that `map_err` is not executed if the computation succeeds.
	#[test]
	fn test_map_err_success() {
		let try_thunk = TryThunk::<i32, &str>::pure(42).map_err(|_| "new error");
		assert_eq!(try_thunk.evaluate(), Ok(42));
	}

	/// Tests `From<Lazy>`.
	#[test]
	fn test_try_thunk_from_memo() {
		use crate::types::RcLazy;
		let memo = RcLazy::new(|| 42);
		let try_thunk: TryThunk<i32, ()> = TryThunk::from(memo);
		assert_eq!(try_thunk.evaluate(), Ok(42));
	}

	/// Tests `From<TryLazy>`.
	#[test]
	fn test_try_thunk_from_try_memo() {
		use crate::types::RcTryLazy;
		let memo = RcTryLazy::new(|| Ok(42));
		let try_thunk: TryThunk<i32, ()> = TryThunk::from(memo);
		assert_eq!(try_thunk.evaluate(), Ok(42));
	}

	/// Tests `Thunk::into_try`.
	///
	/// Verifies that `From<Thunk>` converts a `Thunk` into a `TryThunk` that succeeds.
	#[test]
	fn test_try_thunk_from_eval() {
		let eval = Thunk::pure(42);
		let try_thunk: TryThunk<i32, ()> = TryThunk::from(eval);
		assert_eq!(try_thunk.evaluate(), Ok(42));
	}

	/// Tests `TryThunk::defer`.
	#[test]
	fn test_defer() {
		let try_thunk: TryThunk<i32, ()> = TryThunk::defer(|| TryThunk::ok(42));
		assert_eq!(try_thunk.evaluate(), Ok(42));
	}

	/// Tests `TryThunk::catch`.
	///
	/// Verifies that `catch` recovers from failure.
	#[test]
	fn test_catch() {
		let try_thunk: TryThunk<i32, &str> = TryThunk::err("error").catch(|_| TryThunk::ok(42));
		assert_eq!(try_thunk.evaluate(), Ok(42));
	}

	/// Tests `TryThunk::catch_with`.
	///
	/// Verifies that `catch_with` recovers from failure using a different error type.
	#[test]
	fn test_catch_with() {
		let recovered: TryThunk<i32, i32> =
			TryThunk::<i32, &str>::err("error").catch_with(|_| TryThunk::err(42));
		assert_eq!(recovered.evaluate(), Err(42));

		let ok: TryThunk<i32, i32> = TryThunk::<i32, &str>::ok(1).catch_with(|_| TryThunk::err(42));
		assert_eq!(ok.evaluate(), Ok(1));
	}

	/// Tests `TryThunkErrAppliedBrand` (Functor over Success).
	#[test]
	fn test_try_thunk_with_err_brand() {
		use crate::{
			brands::*,
			functions::*,
		};

		// Functor (map over success)
		let try_thunk: TryThunk<i32, ()> = TryThunk::ok(10);
		let mapped = explicit::map::<TryThunkErrAppliedBrand<()>, _, _, _, _>(|x| x * 2, try_thunk);
		assert_eq!(mapped.evaluate(), Ok(20));

		// Pointed (pure -> ok)
		let try_thunk: TryThunk<i32, ()> = pure::<TryThunkErrAppliedBrand<()>, _>(42);
		assert_eq!(try_thunk.evaluate(), Ok(42));

		// Semimonad (bind over success)
		let try_thunk: TryThunk<i32, ()> = TryThunk::ok(10);
		let bound = explicit::bind::<TryThunkErrAppliedBrand<()>, _, _, _, _>(try_thunk, |x| {
			pure::<TryThunkErrAppliedBrand<()>, _>(x * 2)
		});
		assert_eq!(bound.evaluate(), Ok(20));

		// Foldable (fold over success)
		let try_thunk: TryThunk<i32, ()> = TryThunk::ok(10);
		let folded = explicit::fold_right::<RcFnBrand, TryThunkErrAppliedBrand<()>, _, _, _, _>(
			|x, acc| x + acc,
			5,
			try_thunk,
		);
		assert_eq!(folded, 15);
	}

	/// Tests `Bifunctor` for `TryThunkBrand`.
	#[test]
	fn test_bifunctor() {
		use crate::{
			brands::*,
			classes::bifunctor::*,
		};

		let x: TryThunk<i32, i32> = TryThunk::ok(5);
		assert_eq!(bimap::<TryThunkBrand, _, _, _, _>(|e| e + 1, |s| s * 2, x).evaluate(), Ok(10));

		let y: TryThunk<i32, i32> = TryThunk::err(5);
		assert_eq!(bimap::<TryThunkBrand, _, _, _, _>(|e| e + 1, |s| s * 2, y).evaluate(), Err(6));
	}

	/// Tests `Bifoldable` for `TryThunkBrand` with `bi_fold_right`.
	///
	/// Verifies that error values are folded with `f` and success values with `g`.
	#[test]
	fn test_bifoldable_right() {
		use crate::{
			brands::*,
			functions::*,
		};

		// Error case: f(3, 10) = 10 - 3 = 7
		assert_eq!(
			explicit::bi_fold_right::<RcFnBrand, TryThunkBrand, _, _, _, _, _>(
				(|e: i32, acc| acc - e, |s: i32, acc| acc + s),
				10,
				TryThunk::err(3),
			),
			7
		);

		// Success case: g(5, 10) = 10 + 5 = 15
		assert_eq!(
			explicit::bi_fold_right::<RcFnBrand, TryThunkBrand, _, _, _, _, _>(
				(|e: i32, acc| acc - e, |s: i32, acc| acc + s),
				10,
				TryThunk::ok(5),
			),
			15
		);
	}

	/// Tests `Bifoldable` for `TryThunkBrand` with `bi_fold_left`.
	///
	/// Verifies left-associative folding over both error and success values.
	#[test]
	fn test_bifoldable_left() {
		use crate::{
			brands::*,
			functions::*,
		};

		assert_eq!(
			explicit::bi_fold_left::<RcFnBrand, TryThunkBrand, _, _, _, _, _>(
				(|acc, e: i32| acc - e, |acc, s: i32| acc + s),
				10,
				TryThunk::err(3),
			),
			7
		);

		assert_eq!(
			explicit::bi_fold_left::<RcFnBrand, TryThunkBrand, _, _, _, _, _>(
				(|acc, e: i32| acc - e, |acc, s: i32| acc + s),
				10,
				TryThunk::ok(5),
			),
			15
		);
	}

	/// Tests `Bifoldable` for `TryThunkBrand` with `bi_fold_map`.
	///
	/// Verifies that both error and success values can be mapped to a monoid.
	#[test]
	fn test_bifoldable_map() {
		use crate::{
			brands::*,
			functions::*,
		};

		assert_eq!(
			explicit::bi_fold_map::<RcFnBrand, TryThunkBrand, _, _, _, _, _>(
				(|e: i32| e.to_string(), |s: i32| s.to_string()),
				TryThunk::err(3),
			),
			"3".to_string()
		);

		assert_eq!(
			explicit::bi_fold_map::<RcFnBrand, TryThunkBrand, _, _, _, _, _>(
				(|e: i32| e.to_string(), |s: i32| s.to_string()),
				TryThunk::ok(5),
			),
			"5".to_string()
		);
	}

	/// Tests `MonadRec` for `TryThunkOkAppliedBrand` (tail recursion over error).
	///
	/// Verifies that the loop continues on `ControlFlow::Continue` and terminates on `ControlFlow::Break`.
	#[test]
	fn test_monad_rec_ok_applied() {
		use {
			crate::{
				brands::*,
				functions::*,
			},
			core::ops::ControlFlow,
		};

		let result = tail_rec_m::<TryThunkOkAppliedBrand<i32>, _, _>(
			|x| {
				pure::<TryThunkOkAppliedBrand<i32>, _>(
					if x < 100 { ControlFlow::Continue(x + 1) } else { ControlFlow::Break(x) },
				)
			},
			0,
		);
		assert_eq!(result.evaluate(), Err(100));
	}

	/// Tests `MonadRec` for `TryThunkOkAppliedBrand` short-circuits on `Ok`.
	///
	/// Verifies that encountering an `Ok` value terminates the loop immediately.
	#[test]
	fn test_monad_rec_ok_applied_short_circuit() {
		use {
			crate::{
				brands::*,
				functions::*,
			},
			core::ops::ControlFlow,
		};

		let result = tail_rec_m::<TryThunkOkAppliedBrand<i32>, _, _>(
			|x: i32| {
				if x == 5 {
					TryThunk::ok(42)
				} else {
					pure::<TryThunkOkAppliedBrand<i32>, _>(ControlFlow::<i32, i32>::Continue(x + 1))
				}
			},
			0,
		);
		assert_eq!(result.evaluate(), Ok(42));
	}

	/// Tests `catch_unwind` on `TryThunk`.
	///
	/// Verifies that panics are caught and converted to `Err(String)`.
	#[test]
	fn test_catch_unwind() {
		let thunk = TryThunk::<i32, String>::catch_unwind(|| {
			if true {
				panic!("oops")
			}
			42
		});
		assert_eq!(thunk.evaluate(), Err("oops".to_string()));
	}

	/// Tests `catch_unwind` on `TryThunk` with a non-panicking closure.
	///
	/// Verifies that a successful closure wraps the value in `Ok`.
	#[test]
	fn test_catch_unwind_success() {
		let thunk = TryThunk::<i32, String>::catch_unwind(|| 42);
		assert_eq!(thunk.evaluate(), Ok(42));
	}

	/// Tests `TryThunkOkAppliedBrand` (Functor over Error).
	#[test]
	fn test_try_thunk_with_ok_brand() {
		use crate::{
			brands::*,
			functions::*,
		};

		// Functor (map over error)
		let try_thunk: TryThunk<i32, i32> = TryThunk::err(10);
		let mapped = explicit::map::<TryThunkOkAppliedBrand<i32>, _, _, _, _>(|x| x * 2, try_thunk);
		assert_eq!(mapped.evaluate(), Err(20));

		// Pointed (pure -> err)
		let try_thunk: TryThunk<i32, i32> = pure::<TryThunkOkAppliedBrand<i32>, _>(42);
		assert_eq!(try_thunk.evaluate(), Err(42));

		// Semimonad (bind over error)
		let try_thunk: TryThunk<i32, i32> = TryThunk::err(10);
		let bound = explicit::bind::<TryThunkOkAppliedBrand<i32>, _, _, _, _>(try_thunk, |x| {
			pure::<TryThunkOkAppliedBrand<i32>, _>(x * 2)
		});
		assert_eq!(bound.evaluate(), Err(20));

		// Foldable (fold over error)
		let try_thunk: TryThunk<i32, i32> = TryThunk::err(10);
		let folded = explicit::fold_right::<RcFnBrand, TryThunkOkAppliedBrand<i32>, _, _, _, _>(
			|x, acc| x + acc,
			5,
			try_thunk,
		);
		assert_eq!(folded, 15);
	}

	/// Tests `From<Result>` with `Ok`.
	///
	/// Verifies that converting an `Ok` result produces a successful `TryThunk`.
	#[test]
	fn test_try_thunk_from_result_ok() {
		let try_thunk: TryThunk<i32, String> = TryThunk::from(Ok(42));
		assert_eq!(try_thunk.evaluate(), Ok(42));
	}

	/// Tests `From<Result>` with `Err`.
	///
	/// Verifies that converting an `Err` result produces a failed `TryThunk`.
	#[test]
	fn test_try_thunk_from_result_err() {
		let try_thunk: TryThunk<i32, String> = TryThunk::from(Err("error".to_string()));
		assert_eq!(try_thunk.evaluate(), Err("error".to_string()));
	}

	/// Tests `From<TrySendThunk>` with `Ok`.
	///
	/// Verifies that converting a successful `TrySendThunk` produces a successful `TryThunk`.
	#[test]
	fn test_try_thunk_from_try_send_thunk_ok() {
		use crate::types::TrySendThunk;
		let send: TrySendThunk<i32, ()> = TrySendThunk::pure(42);
		let thunk: TryThunk<i32, ()> = TryThunk::from(send);
		assert_eq!(thunk.evaluate(), Ok(42));
	}

	/// Tests `From<TrySendThunk>` with `Err`.
	///
	/// Verifies that converting a failed `TrySendThunk` produces a failed `TryThunk`.
	#[test]
	fn test_try_thunk_from_try_send_thunk_err() {
		use crate::types::TrySendThunk;
		let send: TrySendThunk<i32, String> = TrySendThunk::err("fail".to_string());
		let thunk: TryThunk<i32, String> = TryThunk::from(send);
		assert_eq!(thunk.evaluate(), Err("fail".to_string()));
	}

	// QuickCheck Law Tests

	// Functor Laws (via HKT, TryThunkErrAppliedBrand)

	/// Functor identity: `map(id, t).evaluate() == t.evaluate()`.
	#[quickcheck]
	fn functor_identity(x: i32) -> bool {
		use crate::{
			brands::*,
			functions::*,
		};
		let t: TryThunk<i32, i32> = TryThunk::ok(x);
		explicit::map::<TryThunkErrAppliedBrand<i32>, _, _, _, _>(|a| a, t).evaluate() == Ok(x)
	}

	/// Functor composition: `map(f . g, t) == map(f, map(g, t))`.
	#[quickcheck]
	fn functor_composition(x: i32) -> bool {
		use crate::{
			brands::*,
			functions::*,
		};
		let f = |a: i32| a.wrapping_add(1);
		let g = |a: i32| a.wrapping_mul(2);
		let lhs = explicit::map::<TryThunkErrAppliedBrand<i32>, _, _, _, _>(
			move |a| f(g(a)),
			TryThunk::ok(x),
		)
		.evaluate();
		let rhs = explicit::map::<TryThunkErrAppliedBrand<i32>, _, _, _, _>(
			f,
			explicit::map::<TryThunkErrAppliedBrand<i32>, _, _, _, _>(g, TryThunk::ok(x)),
		)
		.evaluate();
		lhs == rhs
	}

	// Monad Laws (via HKT, TryThunkErrAppliedBrand)

	/// Monad left identity: `pure(a).bind(f) == f(a)` (for Ok values).
	#[quickcheck]
	fn monad_left_identity(a: i32) -> bool {
		use crate::{
			brands::*,
			functions::*,
		};
		let f = |x: i32| pure::<TryThunkErrAppliedBrand<i32>, _>(x.wrapping_mul(2));
		let lhs = explicit::bind::<TryThunkErrAppliedBrand<i32>, _, _, _, _>(
			pure::<TryThunkErrAppliedBrand<i32>, _>(a),
			f,
		)
		.evaluate();
		let rhs = f(a).evaluate();
		lhs == rhs
	}

	/// Monad right identity: `t.bind(pure) == t`.
	#[quickcheck]
	fn monad_right_identity(x: i32) -> bool {
		use crate::{
			brands::*,
			functions::*,
		};
		let lhs = explicit::bind::<TryThunkErrAppliedBrand<i32>, _, _, _, _>(
			pure::<TryThunkErrAppliedBrand<i32>, _>(x),
			pure::<TryThunkErrAppliedBrand<i32>, _>,
		)
		.evaluate();
		lhs == Ok(x)
	}

	/// Monad associativity: `m.bind(f).bind(g) == m.bind(|a| f(a).bind(g))`.
	#[quickcheck]
	fn monad_associativity(x: i32) -> bool {
		use crate::{
			brands::*,
			functions::*,
		};
		let f = |a: i32| pure::<TryThunkErrAppliedBrand<i32>, _>(a.wrapping_add(1));
		let g = |a: i32| pure::<TryThunkErrAppliedBrand<i32>, _>(a.wrapping_mul(3));
		let m: TryThunk<i32, i32> = TryThunk::ok(x);
		let m2: TryThunk<i32, i32> = TryThunk::ok(x);
		let lhs = explicit::bind::<TryThunkErrAppliedBrand<i32>, _, _, _, _>(
			explicit::bind::<TryThunkErrAppliedBrand<i32>, _, _, _, _>(m, f),
			g,
		)
		.evaluate();
		let rhs = explicit::bind::<TryThunkErrAppliedBrand<i32>, _, _, _, _>(m2, move |a| {
			explicit::bind::<TryThunkErrAppliedBrand<i32>, _, _, _, _>(f(a), g)
		})
		.evaluate();
		lhs == rhs
	}

	/// Error short-circuit: `TryThunk::err(e).bind(f).evaluate() == Err(e)`.
	#[quickcheck]
	fn error_short_circuit(e: i32) -> bool {
		let t: TryThunk<i32, i32> = TryThunk::err(e);
		t.bind(|x| TryThunk::ok(x.wrapping_add(1))).evaluate() == Err(e)
	}

	/// Tests `TryThunk::lift2` with two successful values.
	///
	/// Verifies that `lift2` combines results from both computations.
	#[test]
	fn test_lift2_ok_ok() {
		let t1: TryThunk<i32, String> = TryThunk::ok(10);
		let t2: TryThunk<i32, String> = TryThunk::ok(20);
		let t3 = t1.lift2(t2, |a, b| a + b);
		assert_eq!(t3.evaluate(), Ok(30));
	}

	/// Tests `TryThunk::lift2` short-circuits on first error.
	///
	/// Verifies that if the first computation fails, the second is not evaluated.
	#[test]
	fn test_lift2_err_ok() {
		let t1: TryThunk<i32, String> = TryThunk::err("first".to_string());
		let t2: TryThunk<i32, String> = TryThunk::ok(20);
		let t3 = t1.lift2(t2, |a, b| a + b);
		assert_eq!(t3.evaluate(), Err("first".to_string()));
	}

	/// Tests `TryThunk::lift2` propagates second error.
	///
	/// Verifies that if the second computation fails, the error is propagated.
	#[test]
	fn test_lift2_ok_err() {
		let t1: TryThunk<i32, String> = TryThunk::ok(10);
		let t2: TryThunk<i32, String> = TryThunk::err("second".to_string());
		let t3 = t1.lift2(t2, |a, b| a + b);
		assert_eq!(t3.evaluate(), Err("second".to_string()));
	}

	/// Tests `TryThunk::then` with two successful values.
	///
	/// Verifies that `then` discards the first result and returns the second.
	#[test]
	fn test_then_ok_ok() {
		let t1: TryThunk<i32, String> = TryThunk::ok(10);
		let t2: TryThunk<i32, String> = TryThunk::ok(20);
		let t3 = t1.then(t2);
		assert_eq!(t3.evaluate(), Ok(20));
	}

	/// Tests `TryThunk::then` short-circuits on first error.
	///
	/// Verifies that if the first computation fails, the second is not evaluated.
	#[test]
	fn test_then_err_ok() {
		let t1: TryThunk<i32, String> = TryThunk::err("first".to_string());
		let t2: TryThunk<i32, String> = TryThunk::ok(20);
		let t3 = t1.then(t2);
		assert_eq!(t3.evaluate(), Err("first".to_string()));
	}

	/// Tests `TryThunk::then` propagates second error.
	///
	/// Verifies that if the second computation fails, the error is propagated.
	#[test]
	fn test_then_ok_err() {
		let t1: TryThunk<i32, String> = TryThunk::ok(10);
		let t2: TryThunk<i32, String> = TryThunk::err("second".to_string());
		let t3 = t1.then(t2);
		assert_eq!(t3.evaluate(), Err("second".to_string()));
	}

	/// Tests `TryThunk::into_rc_try_lazy` basic usage.
	///
	/// Verifies that converting a thunk produces a lazy value with the same result.
	#[test]
	fn test_into_rc_try_lazy() {
		let thunk: TryThunk<i32, ()> = TryThunk::ok(42);
		let lazy = thunk.into_rc_try_lazy();
		assert_eq!(lazy.evaluate(), Ok(&42));
	}

	/// Tests `TryThunk::into_rc_try_lazy` caching behavior.
	///
	/// Verifies that the memoized value is computed only once.
	#[test]
	fn test_into_rc_try_lazy_caching() {
		use std::{
			cell::RefCell,
			rc::Rc,
		};

		let counter = Rc::new(RefCell::new(0));
		let counter_clone = counter.clone();
		let thunk: TryThunk<i32, ()> = TryThunk::new(move || {
			*counter_clone.borrow_mut() += 1;
			Ok(42)
		});
		let lazy = thunk.into_rc_try_lazy();

		assert_eq!(*counter.borrow(), 0);
		assert_eq!(lazy.evaluate(), Ok(&42));
		assert_eq!(*counter.borrow(), 1);
		assert_eq!(lazy.evaluate(), Ok(&42));
		assert_eq!(*counter.borrow(), 1);
	}

	/// Tests `TryThunk::into_arc_try_lazy` basic usage.
	///
	/// Verifies that converting a thunk produces a thread-safe lazy value.
	#[test]
	fn test_into_arc_try_lazy() {
		let thunk: TryThunk<i32, ()> = TryThunk::ok(42);
		let lazy = thunk.into_arc_try_lazy();
		assert_eq!(lazy.evaluate(), Ok(&42));
	}

	/// Tests `TryThunk::into_arc_try_lazy` is Send and Sync.
	///
	/// Verifies that the resulting `ArcTryLazy` can be shared across threads.
	#[test]
	fn test_into_arc_try_lazy_send_sync() {
		use std::thread;

		let thunk: TryThunk<i32, String> = TryThunk::ok(42);
		let lazy = thunk.into_arc_try_lazy();
		let lazy_clone = lazy.clone();

		let handle = thread::spawn(move || {
			assert_eq!(lazy_clone.evaluate(), Ok(&42));
		});

		assert_eq!(lazy.evaluate(), Ok(&42));
		handle.join().unwrap();
	}

	/// Tests `TryThunk::catch_unwind_with` with a panicking closure.
	///
	/// Verifies that the custom handler converts the panic payload.
	#[test]
	fn test_catch_unwind_with_panic() {
		let thunk = TryThunk::<i32, i32>::catch_unwind_with(
			|| {
				if true {
					panic!("oops")
				}
				42
			},
			|_payload| -1,
		);
		assert_eq!(thunk.evaluate(), Err(-1));
	}

	/// Tests `TryThunk::catch_unwind_with` with a non-panicking closure.
	///
	/// Verifies that a successful closure wraps the value in `Ok`.
	#[test]
	fn test_catch_unwind_with_success() {
		let thunk = TryThunk::<i32, i32>::catch_unwind_with(|| 42, |_payload| -1);
		assert_eq!(thunk.evaluate(), Ok(42));
	}

	// 7.3: Bifunctor law QuickCheck tests for TryThunkBrand

	/// Bifunctor identity: `bimap(id, id, t) == t`.
	#[quickcheck]
	fn bifunctor_identity_ok(x: i32) -> bool {
		use crate::{
			brands::*,
			classes::bifunctor::*,
		};
		let t: TryThunk<i32, i32> = TryThunk::ok(x);
		bimap::<TryThunkBrand, _, _, _, _>(|e| e, |a| a, t).evaluate() == Ok(x)
	}

	/// Bifunctor identity on error path: `bimap(id, id, err(e)) == err(e)`.
	#[quickcheck]
	fn bifunctor_identity_err(e: i32) -> bool {
		use crate::{
			brands::*,
			classes::bifunctor::*,
		};
		let t: TryThunk<i32, i32> = TryThunk::err(e);
		bimap::<TryThunkBrand, _, _, _, _>(|e| e, |a| a, t).evaluate() == Err(e)
	}

	/// Bifunctor composition: `bimap(f1 . f2, g1 . g2, t) == bimap(f1, g1, bimap(f2, g2, t))`.
	#[quickcheck]
	fn bifunctor_composition_ok(x: i32) -> bool {
		use crate::{
			brands::*,
			classes::bifunctor::*,
		};
		let f1 = |a: i32| a.wrapping_add(1);
		let f2 = |a: i32| a.wrapping_mul(2);
		let g1 = |a: i32| a.wrapping_add(10);
		let g2 = |a: i32| a.wrapping_mul(3);

		let lhs = bimap::<TryThunkBrand, _, _, _, _>(
			move |e| f1(f2(e)),
			move |a| g1(g2(a)),
			TryThunk::ok(x),
		)
		.evaluate();
		let rhs = bimap::<TryThunkBrand, _, _, _, _>(
			f1,
			g1,
			bimap::<TryThunkBrand, _, _, _, _>(f2, g2, TryThunk::ok(x)),
		)
		.evaluate();
		lhs == rhs
	}

	/// Bifunctor composition on error path.
	#[quickcheck]
	fn bifunctor_composition_err(e: i32) -> bool {
		use crate::{
			brands::*,
			classes::bifunctor::*,
		};
		let f1 = |a: i32| a.wrapping_add(1);
		let f2 = |a: i32| a.wrapping_mul(2);
		let g1 = |a: i32| a.wrapping_add(10);
		let g2 = |a: i32| a.wrapping_mul(3);

		let lhs = bimap::<TryThunkBrand, _, _, _, _>(
			move |e| f1(f2(e)),
			move |a| g1(g2(a)),
			TryThunk::err(e),
		)
		.evaluate();
		let rhs = bimap::<TryThunkBrand, _, _, _, _>(
			f1,
			g1,
			bimap::<TryThunkBrand, _, _, _, _>(f2, g2, TryThunk::err(e)),
		)
		.evaluate();
		lhs == rhs
	}

	// 7.4: Error-channel monad law QuickCheck tests via TryThunkOkAppliedBrand

	/// Error-channel monad left identity: `pure(a).bind(f) == f(a)`.
	#[quickcheck]
	fn error_monad_left_identity(a: i32) -> bool {
		use crate::{
			brands::*,
			functions::*,
		};
		let f = |x: i32| pure::<TryThunkOkAppliedBrand<i32>, _>(x.wrapping_mul(2));
		let lhs = explicit::bind::<TryThunkOkAppliedBrand<i32>, _, _, _, _>(
			pure::<TryThunkOkAppliedBrand<i32>, _>(a),
			f,
		)
		.evaluate();
		let rhs = f(a).evaluate();
		lhs == rhs
	}

	/// Error-channel monad right identity: `m.bind(pure) == m`.
	#[quickcheck]
	fn error_monad_right_identity(x: i32) -> bool {
		use crate::{
			brands::*,
			functions::*,
		};
		let lhs = explicit::bind::<TryThunkOkAppliedBrand<i32>, _, _, _, _>(
			pure::<TryThunkOkAppliedBrand<i32>, _>(x),
			pure::<TryThunkOkAppliedBrand<i32>, _>,
		)
		.evaluate();
		lhs == Err(x)
	}

	/// Error-channel monad associativity: `m.bind(f).bind(g) == m.bind(|a| f(a).bind(g))`.
	#[quickcheck]
	fn error_monad_associativity(x: i32) -> bool {
		use crate::{
			brands::*,
			functions::*,
		};
		let f = |a: i32| pure::<TryThunkOkAppliedBrand<i32>, _>(a.wrapping_add(1));
		let g = |a: i32| pure::<TryThunkOkAppliedBrand<i32>, _>(a.wrapping_mul(3));
		let m: TryThunk<i32, i32> = TryThunk::err(x);
		let m2: TryThunk<i32, i32> = TryThunk::err(x);
		let lhs = explicit::bind::<TryThunkOkAppliedBrand<i32>, _, _, _, _>(
			explicit::bind::<TryThunkOkAppliedBrand<i32>, _, _, _, _>(m, f),
			g,
		)
		.evaluate();
		let rhs = explicit::bind::<TryThunkOkAppliedBrand<i32>, _, _, _, _>(m2, move |a| {
			explicit::bind::<TryThunkOkAppliedBrand<i32>, _, _, _, _>(f(a), g)
		})
		.evaluate();
		lhs == rhs
	}

	// 7.5: Semigroup/Monoid law QuickCheck tests for TryThunk

	/// Semigroup associativity for TryThunk: `append(append(a, b), c) == append(a, append(b, c))`.
	#[quickcheck]
	fn try_thunk_semigroup_associativity(
		a: String,
		b: String,
		c: String,
	) -> bool {
		use crate::classes::semigroup::append;

		let ta: TryThunk<String, ()> = TryThunk::ok(a.clone());
		let tb: TryThunk<String, ()> = TryThunk::ok(b.clone());
		let tc: TryThunk<String, ()> = TryThunk::ok(c.clone());
		let ta2: TryThunk<String, ()> = TryThunk::ok(a);
		let tb2: TryThunk<String, ()> = TryThunk::ok(b);
		let tc2: TryThunk<String, ()> = TryThunk::ok(c);
		let lhs = append(append(ta, tb), tc).evaluate();
		let rhs = append(ta2, append(tb2, tc2)).evaluate();
		lhs == rhs
	}

	/// Monoid left identity for TryThunk: `append(empty(), a) == a`.
	#[quickcheck]
	fn try_thunk_monoid_left_identity(x: String) -> bool {
		use crate::classes::{
			monoid::empty,
			semigroup::append,
		};

		let a: TryThunk<String, ()> = TryThunk::ok(x.clone());
		let lhs: TryThunk<String, ()> = append(empty(), a);
		lhs.evaluate() == Ok(x)
	}

	/// Monoid right identity for TryThunk: `append(a, empty()) == a`.
	#[quickcheck]
	fn try_thunk_monoid_right_identity(x: String) -> bool {
		use crate::classes::{
			monoid::empty,
			semigroup::append,
		};

		let a: TryThunk<String, ()> = TryThunk::ok(x.clone());
		let rhs: TryThunk<String, ()> = append(a, empty());
		rhs.evaluate() == Ok(x)
	}

	// 7.6: Thread safety test for TryThunk::into_arc_try_lazy

	/// Tests that `TryThunk::into_arc_try_lazy` produces a thread-safe value
	/// and all threads see the same cached result.
	#[test]
	fn test_into_arc_try_lazy_thread_safety() {
		use std::{
			sync::{
				Arc,
				atomic::{
					AtomicUsize,
					Ordering,
				},
			},
			thread,
		};

		let counter = Arc::new(AtomicUsize::new(0));
		let counter_clone = Arc::clone(&counter);
		let thunk: TryThunk<i32, String> = TryThunk::new(move || {
			counter_clone.fetch_add(1, Ordering::SeqCst);
			Ok(42)
		});
		let lazy = thunk.into_arc_try_lazy();

		let handles: Vec<_> = (0 .. 8)
			.map(|_| {
				let lazy_clone = lazy.clone();
				thread::spawn(move || {
					assert_eq!(lazy_clone.evaluate(), Ok(&42));
				})
			})
			.collect();

		assert_eq!(lazy.evaluate(), Ok(&42));
		for h in handles {
			h.join().unwrap();
		}

		// The closure should have been invoked exactly once.
		assert_eq!(counter.load(Ordering::SeqCst), 1);
	}

	/// Tests `Semigroup::append` short-circuits when the first operand is `Err`.
	///
	/// Verifies that the second operand is never evaluated when the first fails.
	#[test]
	fn test_semigroup_append_first_err_short_circuits() {
		use {
			crate::classes::semigroup::append,
			std::cell::Cell,
		};

		let counter = Cell::new(0u32);
		let t1: TryThunk<String, &str> = TryThunk::err("first failed");
		let t2: TryThunk<String, &str> = TryThunk::new(|| {
			counter.set(counter.get() + 1);
			Ok("second".to_string())
		});
		let result = append(t1, t2);
		assert_eq!(result.evaluate(), Err("first failed"));
		assert_eq!(counter.get(), 0, "second operand should not have been evaluated");
	}

	/// Tests `Semigroup::append` propagates the error when the second operand fails.
	///
	/// Verifies that when the first operand succeeds but the second fails, the error
	/// from the second operand is returned.
	#[test]
	fn test_semigroup_append_second_err_propagates() {
		use crate::classes::semigroup::append;

		let t1: TryThunk<String, &str> = TryThunk::pure("hello".to_string());
		let t2: TryThunk<String, &str> = TryThunk::err("second failed");
		let result = append(t1, t2);
		assert_eq!(result.evaluate(), Err("second failed"));
	}

	/// Tests `TryThunk::tail_rec_m` computes a simple recursive sum.
	///
	/// Verifies that the loop accumulates correctly and terminates with `Break`.
	#[test]
	fn test_tail_rec_m_success() {
		use core::ops::ControlFlow;
		let result: TryThunk<i32, ()> = TryThunk::tail_rec_m(
			|x| {
				TryThunk::ok(
					if x < 1000 { ControlFlow::Continue(x + 1) } else { ControlFlow::Break(x) },
				)
			},
			0,
		);
		assert_eq!(result.evaluate(), Ok(1000));
	}

	/// Tests `TryThunk::tail_rec_m` short-circuits on error.
	///
	/// Verifies that when the step function returns `Err`, the loop terminates
	/// immediately and propagates the error.
	#[test]
	fn test_tail_rec_m_early_error() {
		use core::ops::ControlFlow;
		let result: TryThunk<i32, &str> = TryThunk::tail_rec_m(
			|x| {
				if x == 5 {
					TryThunk::err("stopped at 5")
				} else {
					TryThunk::ok(ControlFlow::Continue(x + 1))
				}
			},
			0,
		);
		assert_eq!(result.evaluate(), Err("stopped at 5"));
	}

	/// Tests `TryThunk::tail_rec_m` stack safety with many iterations.
	///
	/// Verifies that the loop does not overflow the stack with 100,000 iterations.
	#[test]
	fn test_tail_rec_m_stack_safety() {
		use core::ops::ControlFlow;
		let iterations: i64 = 100_000;
		let result: TryThunk<i64, ()> = TryThunk::tail_rec_m(
			|acc| {
				TryThunk::ok(
					if acc < iterations {
						ControlFlow::Continue(acc + 1)
					} else {
						ControlFlow::Break(acc)
					},
				)
			},
			0i64,
		);
		assert_eq!(result.evaluate(), Ok(iterations));
	}

	/// Tests `TryThunk::arc_tail_rec_m` computes correctly and tracks calls.
	///
	/// Verifies that the Arc-wrapped variant produces the same result as
	/// `tail_rec_m` and that the step function is called the expected number
	/// of times.
	#[test]
	fn test_arc_tail_rec_m_success() {
		use {
			core::ops::ControlFlow,
			std::sync::{
				Arc,
				atomic::{
					AtomicUsize,
					Ordering,
				},
			},
		};
		let counter = Arc::new(AtomicUsize::new(0));
		let counter_clone = Arc::clone(&counter);
		let result: TryThunk<i32, ()> = TryThunk::arc_tail_rec_m(
			move |x| {
				counter_clone.fetch_add(1, Ordering::SeqCst);
				TryThunk::ok(
					if x < 100 { ControlFlow::Continue(x + 1) } else { ControlFlow::Break(x) },
				)
			},
			0,
		);
		assert_eq!(result.evaluate(), Ok(100));
		assert_eq!(counter.load(Ordering::SeqCst), 101);
	}

	/// Tests `TryThunk::arc_tail_rec_m` short-circuits on error.
	///
	/// Verifies that the Arc-wrapped variant propagates errors immediately.
	#[test]
	fn test_arc_tail_rec_m_early_error() {
		use core::ops::ControlFlow;
		let result: TryThunk<i32, &str> = TryThunk::arc_tail_rec_m(
			|x| {
				if x == 5 {
					TryThunk::err("stopped at 5")
				} else {
					TryThunk::ok(ControlFlow::Continue(x + 1))
				}
			},
			0,
		);
		assert_eq!(result.evaluate(), Err("stopped at 5"));
	}

	/// Tests `TryThunk::arc_tail_rec_m` stack safety with many iterations.
	///
	/// Verifies that the Arc-wrapped variant does not overflow the stack.
	#[test]
	fn test_arc_tail_rec_m_stack_safety() {
		use core::ops::ControlFlow;
		let iterations: i64 = 100_000;
		let result: TryThunk<i64, ()> = TryThunk::arc_tail_rec_m(
			|acc| {
				TryThunk::ok(
					if acc < iterations {
						ControlFlow::Continue(acc + 1)
					} else {
						ControlFlow::Break(acc)
					},
				)
			},
			0i64,
		);
		assert_eq!(result.evaluate(), Ok(iterations));
	}
}
