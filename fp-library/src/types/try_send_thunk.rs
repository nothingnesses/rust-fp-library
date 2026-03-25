//! Thread-safe deferred, non-memoized fallible computation.
//!
//! The fallible counterpart to [`SendThunk`](crate::types::SendThunk). Each
//! call to [`TrySendThunk::evaluate`] re-executes the computation and returns
//! a [`Result`]. Like [`SendThunk`](crate::types::SendThunk), the inner
//! closure is `Send`, enabling thread-safe deferred computation chains.

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			brands::TrySendThunkBrand,
			classes::{
				Deferrable,
				Monoid,
				Semigroup,
				SendDeferrable,
			},
			impl_kind,
			kinds::*,
			types::{
				ArcTryLazy,
				SendThunk,
				TryThunk,
			},
		},
		fp_macros::*,
		std::fmt,
	};

	/// A thread-safe deferred computation that may fail with error type `E`.
	///
	/// This is [`SendThunk<'a, Result<A, E>>`] with ergonomic combinators for
	/// error handling. Like [`SendThunk`], the inner closure is `Send`, so
	/// `TrySendThunk` can be transferred across thread boundaries. Like
	/// [`TryThunk`](crate::types::TryThunk), the result is [`Result<A, E>`].
	///
	/// Each [`TrySendThunk::evaluate`] re-executes the computation (it is NOT
	/// memoized). For memoized fallible thread-safe computation, use
	/// [`ArcTryLazy`](crate::types::ArcTryLazy).
	///
	/// ### Higher-Kinded Type Representation
	///
	/// The higher-kinded representation of this type constructor is
	/// [`TrySendThunkBrand`](crate::brands::TrySendThunkBrand), which is fully
	/// polymorphic over both error and success types.
	///
	/// ### HKT Trait Limitations
	///
	/// Standard HKT traits such as `Functor`, `Pointed`, `Semimonad`, and
	/// `Semiapplicative` cannot be implemented for `TrySendThunk` brands
	/// because their signatures do not require `Send` on the mapping or
	/// binding functions. Since `TrySendThunk` stores a
	/// `SendThunk<Result<A, E>>` (which wraps `Box<dyn FnOnce() -> Result<A, E> + Send>`),
	/// composing it with a non-`Send` closure would violate the `Send`
	/// invariant.
	///
	/// Use the inherent methods ([`map`](TrySendThunk::map),
	/// [`bind`](TrySendThunk::bind), [`map_err`](TrySendThunk::map_err),
	/// [`bimap`](TrySendThunk::bimap)) instead, which accept `Send` closures
	/// explicitly.
	///
	/// ### When to Use
	///
	/// Use `TrySendThunk` when you need a fallible deferred computation that
	/// must cross thread boundaries. For single-threaded fallible computation
	/// with full HKT support, use [`TryThunk`](crate::types::TryThunk). For
	/// memoized fallible thread-safe computation, use
	/// [`ArcTryLazy`](crate::types::ArcTryLazy).
	///
	/// ### Algebraic Properties
	///
	/// `TrySendThunk` forms a monad over the success type `A` (with `E` fixed):
	/// - `TrySendThunk::pure(a).bind(f).evaluate() == f(a).evaluate()` (left identity).
	/// - `thunk.bind(TrySendThunk::ok).evaluate() == thunk.evaluate()` (right identity).
	/// - `thunk.bind(f).bind(g).evaluate() == thunk.bind(|a| f(a).bind(g)).evaluate()` (associativity).
	///
	/// On the error channel, `bind` short-circuits: if the computation produces `Err(e)`,
	/// the continuation `f` is never called.
	///
	/// ### Stack Safety
	///
	/// `TrySendThunk::bind` chains are **not** stack-safe. Each nested
	/// [`bind`](TrySendThunk::bind) adds a frame to the call stack, so
	/// sufficiently deep chains will cause a stack overflow.
	///
	/// ### Limitations
	///
	/// **Cannot implement `Traversable`**: `TrySendThunk` wraps a `FnOnce`
	/// closure, which cannot be cloned because `FnOnce` is consumed when
	/// called. The [`Traversable`](crate::classes::Traversable) trait
	/// requires `Clone` bounds on the result type, making it fundamentally
	/// incompatible with `TrySendThunk`'s design.
	#[document_type_parameters(
		"The lifetime of the computation.",
		"The type of the value produced by the computation on success.",
		"The type of the error produced by the computation on failure."
	)]
	///
	pub struct TrySendThunk<'a, A, E>(
		/// The internal `SendThunk` wrapping a `Result`.
		SendThunk<'a, Result<A, E>>,
	);

	#[document_type_parameters(
		"The lifetime of the computation.",
		"The type of the success value.",
		"The type of the error value."
	)]
	#[document_parameters("The `TrySendThunk` instance.")]
	impl<'a, A: 'a, E: 'a> TrySendThunk<'a, A, E> {
		/// Creates a new `TrySendThunk` from a thread-safe closure.
		#[document_signature]
		///
		#[document_parameters("The thread-safe closure to wrap.")]
		///
		#[document_returns("A new `TrySendThunk` instance.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let try_thunk: TrySendThunk<i32, ()> = TrySendThunk::new(|| Ok(42));
		/// assert_eq!(try_thunk.evaluate(), Ok(42));
		/// ```
		#[inline]
		pub fn new(f: impl FnOnce() -> Result<A, E> + Send + 'a) -> Self {
			TrySendThunk(SendThunk::new(f))
		}

		/// Returns a pure value (already computed).
		#[document_signature]
		///
		#[document_parameters("The value to wrap.")]
		///
		#[document_returns("A new `TrySendThunk` instance containing the value.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let try_thunk: TrySendThunk<i32, ()> = TrySendThunk::pure(42);
		/// assert_eq!(try_thunk.evaluate(), Ok(42));
		/// ```
		#[inline]
		pub fn pure(a: A) -> Self
		where
			A: Send + 'a,
			E: Send + 'a, {
			TrySendThunk(SendThunk::pure(Ok(a)))
		}

		/// Defers a computation that returns a `TrySendThunk`.
		#[document_signature]
		///
		#[document_parameters("The thunk that returns a `TrySendThunk`.")]
		///
		#[document_returns("A new `TrySendThunk` instance.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let try_thunk: TrySendThunk<i32, ()> = TrySendThunk::defer(|| TrySendThunk::ok(42));
		/// assert_eq!(try_thunk.evaluate(), Ok(42));
		/// ```
		#[inline]
		pub fn defer(f: impl FnOnce() -> TrySendThunk<'a, A, E> + Send + 'a) -> Self {
			TrySendThunk(SendThunk::new(move || f().evaluate()))
		}

		/// Creates a successful computation.
		#[document_signature]
		///
		#[document_parameters("The value to wrap.")]
		///
		#[document_returns("A new `TrySendThunk` instance containing the value.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let try_thunk: TrySendThunk<i32, ()> = TrySendThunk::ok(42);
		/// assert_eq!(try_thunk.evaluate(), Ok(42));
		/// ```
		#[inline]
		pub fn ok(a: A) -> Self
		where
			A: Send + 'a,
			E: Send + 'a, {
			TrySendThunk(SendThunk::pure(Ok(a)))
		}

		/// Returns a pure error.
		#[document_signature]
		///
		#[document_parameters("The error to wrap.")]
		///
		#[document_returns("A new `TrySendThunk` instance containing the error.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let try_thunk: TrySendThunk<i32, &str> = TrySendThunk::err("error");
		/// assert_eq!(try_thunk.evaluate(), Err("error"));
		/// ```
		#[inline]
		pub fn err(e: E) -> Self
		where
			A: Send + 'a,
			E: Send + 'a, {
			TrySendThunk(SendThunk::pure(Err(e)))
		}

		/// Monadic bind: chains computations.
		///
		/// Note: Each `bind` adds to the call stack. This is **not** stack-safe
		/// for deep recursion.
		#[document_signature]
		///
		#[document_type_parameters("The type of the result of the new computation.")]
		///
		#[document_parameters("The function to apply to the result of the computation.")]
		///
		#[document_returns("A new `TrySendThunk` instance representing the chained computation.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let try_thunk: TrySendThunk<i32, ()> = TrySendThunk::ok(21).bind(|x| TrySendThunk::ok(x * 2));
		/// assert_eq!(try_thunk.evaluate(), Ok(42));
		/// ```
		#[inline]
		pub fn bind<B>(
			self,
			f: impl FnOnce(A) -> TrySendThunk<'a, B, E> + Send + 'a,
		) -> TrySendThunk<'a, B, E>
		where
			A: Send + 'a,
			B: Send + 'a,
			E: Send + 'a, {
			TrySendThunk(self.0.bind(move |result| match result {
				Ok(a) => SendThunk::new(move || f(a).evaluate()),
				Err(e) => SendThunk::pure(Err(e)),
			}))
		}

		/// Functor map: transforms the success value.
		#[document_signature]
		///
		#[document_type_parameters("The type of the result of the transformation.")]
		///
		#[document_parameters("The function to apply to the success value.")]
		///
		#[document_returns("A new `TrySendThunk` instance with the transformed result.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let try_thunk: TrySendThunk<i32, ()> = TrySendThunk::ok(21).map(|x| x * 2);
		/// assert_eq!(try_thunk.evaluate(), Ok(42));
		/// ```
		#[inline]
		pub fn map<B: 'a>(
			self,
			func: impl FnOnce(A) -> B + Send + 'a,
		) -> TrySendThunk<'a, B, E> {
			TrySendThunk(self.0.map(move |result| result.map(func)))
		}

		/// Map error: transforms the error value.
		#[document_signature]
		///
		#[document_type_parameters("The type of the new error.")]
		///
		#[document_parameters("The function to apply to the error.")]
		///
		#[document_returns("A new `TrySendThunk` instance with the transformed error.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let try_thunk: TrySendThunk<i32, i32> = TrySendThunk::err(21).map_err(|x| x * 2);
		/// assert_eq!(try_thunk.evaluate(), Err(42));
		/// ```
		#[inline]
		pub fn map_err<E2: 'a>(
			self,
			f: impl FnOnce(E) -> E2 + Send + 'a,
		) -> TrySendThunk<'a, A, E2> {
			TrySendThunk(self.0.map(move |result| result.map_err(f)))
		}

		/// Maps both the success and error values simultaneously.
		#[document_signature]
		///
		#[document_type_parameters(
			"The type of the transformed success value.",
			"The type of the transformed error value."
		)]
		///
		#[document_parameters(
			"The function to apply to the success value.",
			"The function to apply to the error value."
		)]
		///
		#[document_returns("A new `TrySendThunk` with both values transformed.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let ok: TrySendThunk<i32, i32> = TrySendThunk::ok(5);
		/// assert_eq!(ok.bimap(|s| s * 2, |e| e + 1).evaluate(), Ok(10));
		///
		/// let err: TrySendThunk<i32, i32> = TrySendThunk::err(5);
		/// assert_eq!(err.bimap(|s| s * 2, |e| e + 1).evaluate(), Err(6));
		/// ```
		#[inline]
		pub fn bimap<B: 'a, F: 'a>(
			self,
			f: impl FnOnce(A) -> B + Send + 'a,
			g: impl FnOnce(E) -> F + Send + 'a,
		) -> TrySendThunk<'a, B, F> {
			TrySendThunk(self.0.map(move |result| match result {
				Ok(a) => Ok(f(a)),
				Err(e) => Err(g(e)),
			}))
		}

		/// Recovers from an error.
		#[document_signature]
		///
		#[document_parameters("The function to apply to the error value.")]
		///
		#[document_returns("A new `TrySendThunk` that attempts to recover from failure.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let try_thunk: TrySendThunk<i32, &str> =
		/// 	TrySendThunk::err("error").catch(|_| TrySendThunk::ok(42));
		/// assert_eq!(try_thunk.evaluate(), Ok(42));
		/// ```
		#[inline]
		pub fn catch(
			self,
			f: impl FnOnce(E) -> TrySendThunk<'a, A, E> + Send + 'a,
		) -> Self
		where
			A: Send,
			E: Send, {
			TrySendThunk(self.0.bind(move |result| match result {
				Ok(a) => SendThunk::pure(Ok(a)),
				Err(e) => SendThunk::new(move || f(e).evaluate()),
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
		/// let try_thunk: TrySendThunk<i32, ()> = TrySendThunk::ok(42);
		/// assert_eq!(try_thunk.evaluate(), Ok(42));
		/// ```
		#[inline]
		pub fn evaluate(self) -> Result<A, E> {
			self.0.evaluate()
		}

		/// Combines two `TrySendThunk`s, running both and combining their results.
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
		#[document_returns("A new `TrySendThunk` producing the combined result.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let t1: TrySendThunk<i32, String> = TrySendThunk::ok(10);
		/// let t2: TrySendThunk<i32, String> = TrySendThunk::ok(20);
		/// let t3 = t1.lift2(t2, |a, b| a + b);
		/// assert_eq!(t3.evaluate(), Ok(30));
		///
		/// let t4: TrySendThunk<i32, String> = TrySendThunk::err("fail".to_string());
		/// let t5: TrySendThunk<i32, String> = TrySendThunk::ok(20);
		/// let t6 = t4.lift2(t5, |a, b| a + b);
		/// assert_eq!(t6.evaluate(), Err("fail".to_string()));
		/// ```
		#[inline]
		pub fn lift2<B, C>(
			self,
			other: TrySendThunk<'a, B, E>,
			f: impl FnOnce(A, B) -> C + Send + 'a,
		) -> TrySendThunk<'a, C, E>
		where
			A: Send + 'a,
			B: Send + 'a,
			C: Send + 'a,
			E: Send + 'a, {
			self.bind(move |a| other.map(move |b| f(a, b)))
		}

		/// Sequences two `TrySendThunk`s, discarding the first result.
		///
		/// Short-circuits on error: if `self` fails, `other` is never evaluated.
		#[document_signature]
		///
		#[document_type_parameters("The type of the second computation's success value.")]
		///
		#[document_parameters("The second computation.")]
		///
		#[document_returns(
			"A new `TrySendThunk` that runs both computations and returns the result of the second."
		)]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let t1: TrySendThunk<i32, String> = TrySendThunk::ok(10);
		/// let t2: TrySendThunk<i32, String> = TrySendThunk::ok(20);
		/// let t3 = t1.then(t2);
		/// assert_eq!(t3.evaluate(), Ok(20));
		///
		/// let t4: TrySendThunk<i32, String> = TrySendThunk::err("fail".to_string());
		/// let t5: TrySendThunk<i32, String> = TrySendThunk::ok(20);
		/// let t6 = t4.then(t5);
		/// assert_eq!(t6.evaluate(), Err("fail".to_string()));
		/// ```
		#[inline]
		pub fn then<B>(
			self,
			other: TrySendThunk<'a, B, E>,
		) -> TrySendThunk<'a, B, E>
		where
			A: Send + 'a,
			B: Send + 'a,
			E: Send + 'a, {
			self.bind(move |_| other)
		}

		/// Converts this `TrySendThunk` into a memoized, thread-safe [`ArcTryLazy`].
		///
		/// Unlike [`TryThunk::memoize_arc`](crate::types::TryThunk::memoize_arc),
		/// this does **not** evaluate eagerly. The inner `Send` closure is passed
		/// directly into `ArcTryLazy::new`, so evaluation is deferred until the
		/// `ArcTryLazy` is first accessed.
		#[document_signature]
		///
		#[document_returns("A thread-safe `ArcTryLazy` that evaluates this thunk on first access.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let thunk: TrySendThunk<i32, ()> = TrySendThunk::ok(42);
		/// let lazy: ArcTryLazy<i32, ()> = thunk.memoize_arc();
		/// assert_eq!(lazy.evaluate(), Ok(&42));
		/// ```
		#[inline]
		pub fn memoize_arc(self) -> ArcTryLazy<'a, A, E>
		where
			A: Send + Sync + 'a,
			E: Send + Sync + 'a, {
			ArcTryLazy::new(move || self.evaluate())
		}
	}

	#[document_type_parameters(
		"The lifetime of the computation.",
		"The type of the computed value.",
		"The type of the error value."
	)]
	impl<'a, A: 'a, E: 'a> TrySendThunk<'a, A, E> {
		/// Creates a `TrySendThunk` that catches unwinds (panics), converting
		/// the panic payload using a custom conversion function.
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
			"A new `TrySendThunk` instance where panics are converted to `Err(E)` via the handler."
		)]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let thunk = TrySendThunk::<i32, i32>::catch_unwind_with(
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
			f: impl FnOnce() -> A + Send + std::panic::UnwindSafe + 'a,
			handler: impl FnOnce(Box<dyn std::any::Any + Send>) -> E + Send + 'a,
		) -> Self {
			TrySendThunk::new(move || std::panic::catch_unwind(f).map_err(handler))
		}
	}

	#[document_type_parameters(
		"The lifetime of the computation.",
		"The type of the computed value."
	)]
	impl<'a, A: 'a> TrySendThunk<'a, A, String> {
		/// Creates a `TrySendThunk` that catches unwinds (panics).
		///
		/// The closure is executed when the thunk is evaluated. If the closure
		/// panics, the panic payload is converted to a `String` error. If the
		/// closure returns normally, the value is wrapped in `Ok`.
		///
		/// This is a convenience wrapper around
		/// [`catch_unwind_with`](TrySendThunk::catch_unwind_with) that uses
		/// the default panic payload to string conversion.
		#[document_signature]
		///
		#[document_parameters("The closure that might panic.")]
		///
		#[document_returns(
			"A new `TrySendThunk` instance where panics are converted to `Err(String)`."
		)]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let thunk = TrySendThunk::<i32, String>::catch_unwind(|| {
		/// 	if true {
		/// 		panic!("oops")
		/// 	}
		/// 	42
		/// });
		/// assert_eq!(thunk.evaluate(), Err("oops".to_string()));
		/// ```
		pub fn catch_unwind(f: impl FnOnce() -> A + Send + std::panic::UnwindSafe + 'a) -> Self {
			Self::catch_unwind_with(f, crate::utils::panic_payload_to_string)
		}
	}

	impl_kind! {
		/// HKT branding for the `TrySendThunk` type.
		///
		/// The type parameters for `Of` are ordered `E`, then `A` (Error, then
		/// Success). This follows the same convention as `TryThunkBrand` and
		/// `ResultBrand`.
		for TrySendThunkBrand {
			type Of<'a, E: 'a, A: 'a>: 'a = TrySendThunk<'a, A, E>;
		}
	}

	#[document_type_parameters(
		"The lifetime of the computation.",
		"The type of the success value.",
		"The type of the error value."
	)]
	impl<'a, A: 'a, E: 'a> From<TryThunk<'a, A, E>> for TrySendThunk<'a, A, E>
	where
		A: Send,
		E: Send,
	{
		/// Converts a [`TryThunk`] into a [`TrySendThunk`].
		///
		/// The `TryThunk` closure is not `Send`, so the conversion eagerly
		/// evaluates it and wraps the owned result in a new `TrySendThunk`.
		#[document_signature]
		#[document_parameters("The try-thunk to convert.")]
		#[document_returns("A new `TrySendThunk` wrapping the evaluated result.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		/// let thunk: TryThunk<i32, ()> = TryThunk::ok(42);
		/// let send_thunk = TrySendThunk::from(thunk);
		/// assert_eq!(send_thunk.evaluate(), Ok(42));
		/// ```
		fn from(thunk: TryThunk<'a, A, E>) -> Self {
			let result = thunk.evaluate();
			TrySendThunk::new(move || result)
		}
	}

	#[document_type_parameters(
		"The lifetime of the computation.",
		"The type of the success value.",
		"The type of the error value."
	)]
	impl<'a, A: 'a, E: 'a> From<Result<A, E>> for TrySendThunk<'a, A, E>
	where
		A: Send,
		E: Send,
	{
		/// Converts a [`Result`] into a [`TrySendThunk`].
		#[document_signature]
		#[document_parameters("The result to convert.")]
		#[document_returns("A new `TrySendThunk` instance that produces the result.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		/// let ok_thunk: TrySendThunk<i32, String> = TrySendThunk::from(Ok(42));
		/// assert_eq!(ok_thunk.evaluate(), Ok(42));
		///
		/// let err_thunk: TrySendThunk<i32, String> = TrySendThunk::from(Err("error".to_string()));
		/// assert_eq!(err_thunk.evaluate(), Err("error".to_string()));
		/// ```
		fn from(result: Result<A, E>) -> Self {
			TrySendThunk(SendThunk::pure(result))
		}
	}

	#[document_type_parameters(
		"The lifetime of the computation.",
		"The type of the success value.",
		"The type of the error value."
	)]
	impl<'a, A: 'a, E: 'a> From<SendThunk<'a, A>> for TrySendThunk<'a, A, E>
	where
		A: Send,
	{
		/// Converts a [`SendThunk`] into a [`TrySendThunk`] that always succeeds.
		#[document_signature]
		#[document_parameters("The send thunk to convert.")]
		#[document_returns("A new `TrySendThunk` wrapping the send thunk as a success.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		/// let thunk = SendThunk::pure(42);
		/// let try_thunk: TrySendThunk<i32, ()> = TrySendThunk::from(thunk);
		/// assert_eq!(try_thunk.evaluate(), Ok(42));
		/// ```
		fn from(thunk: SendThunk<'a, A>) -> Self {
			TrySendThunk(thunk.map(Ok))
		}
	}

	#[document_type_parameters(
		"The lifetime of the computation.",
		"The type of the success value.",
		"The type of the error value."
	)]
	impl<'a, A, E> Deferrable<'a> for TrySendThunk<'a, A, E>
	where
		A: 'a,
		E: 'a,
	{
		/// Creates a `TrySendThunk` from a computation that produces it.
		///
		/// The thunk `f` is called eagerly because `Deferrable::defer` does not
		/// require `Send` on the closure.
		#[document_signature]
		///
		#[document_parameters("A thunk that produces the try-send-thunk.")]
		///
		#[document_returns("The deferred try-send-thunk.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	classes::Deferrable,
		/// 	types::*,
		/// };
		///
		/// let task: TrySendThunk<i32, ()> = Deferrable::defer(|| TrySendThunk::ok(42));
		/// assert_eq!(task.evaluate(), Ok(42));
		/// ```
		fn defer(f: impl FnOnce() -> Self + 'a) -> Self
		where
			Self: Sized, {
			f()
		}
	}

	#[document_type_parameters(
		"The lifetime of the computation.",
		"The type of the success value.",
		"The type of the error value."
	)]
	impl<'a, A: Send + 'a, E: Send + 'a> SendDeferrable<'a> for TrySendThunk<'a, A, E> {
		/// Creates a `TrySendThunk` from a thread-safe computation that
		/// produces it.
		#[document_signature]
		///
		#[document_parameters("A thread-safe thunk that produces the try-send-thunk.")]
		///
		#[document_returns("The deferred try-send-thunk.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	classes::SendDeferrable,
		/// 	types::*,
		/// };
		///
		/// let task: TrySendThunk<i32, ()> = SendDeferrable::send_defer(|| TrySendThunk::ok(42));
		/// assert_eq!(task.evaluate(), Ok(42));
		/// ```
		fn send_defer(f: impl FnOnce() -> Self + Send + 'a) -> Self
		where
			Self: Sized, {
			TrySendThunk(SendThunk::new(move || f().evaluate()))
		}
	}

	#[document_type_parameters(
		"The lifetime of the computation.",
		"The success value type.",
		"The error value type."
	)]
	impl<'a, A: Semigroup + Send + 'a, E: Send + 'a> Semigroup for TrySendThunk<'a, A, E> {
		/// Combines two `TrySendThunk`s by combining their results.
		#[document_signature]
		///
		#[document_parameters("The first `TrySendThunk`.", "The second `TrySendThunk`.")]
		///
		#[document_returns("A new `TrySendThunk` containing the combined result.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	classes::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let t1: TrySendThunk<String, ()> = TrySendThunk::ok("Hello".to_string());
		/// let t2: TrySendThunk<String, ()> = TrySendThunk::ok(" World".to_string());
		/// let t3 = append::<_>(t1, t2);
		/// assert_eq!(t3.evaluate(), Ok("Hello World".to_string()));
		/// ```
		fn append(
			a: Self,
			b: Self,
		) -> Self {
			TrySendThunk::new(move || match (a.evaluate(), b.evaluate()) {
				(Ok(a_val), Ok(b_val)) => Ok(Semigroup::append(a_val, b_val)),
				(Err(e), _) => Err(e),
				(_, Err(e)) => Err(e),
			})
		}
	}

	#[document_type_parameters(
		"The lifetime of the computation.",
		"The success value type.",
		"The error value type."
	)]
	impl<'a, A: Monoid + Send + 'a, E: Send + 'a> Monoid for TrySendThunk<'a, A, E> {
		/// Returns the identity `TrySendThunk`.
		#[document_signature]
		///
		#[document_returns("A `TrySendThunk` producing the identity value of `A`.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	classes::*,
		/// 	types::*,
		/// };
		///
		/// let t: TrySendThunk<String, ()> = TrySendThunk::empty();
		/// assert_eq!(t.evaluate(), Ok("".to_string()));
		/// ```
		fn empty() -> Self {
			TrySendThunk::new(|| Ok(Monoid::empty()))
		}
	}

	#[document_type_parameters(
		"The lifetime of the computation.",
		"The type of the success value.",
		"The type of the error value."
	)]
	#[document_parameters("The try-send-thunk to format.")]
	impl<'a, A, E> fmt::Debug for TrySendThunk<'a, A, E> {
		/// Formats the try-send-thunk without evaluating it.
		#[document_signature]
		#[document_parameters("The formatter.")]
		#[document_returns("The formatting result.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		/// let thunk = TrySendThunk::new(|| Ok::<i32, ()>(42));
		/// assert_eq!(format!("{:?}", thunk), "TrySendThunk(<unevaluated>)");
		/// ```
		fn fmt(
			&self,
			f: &mut fmt::Formatter<'_>,
		) -> fmt::Result {
			f.write_str("TrySendThunk(<unevaluated>)")
		}
	}
}
pub use inner::*;

#[cfg(test)]
mod tests {
	use {
		super::*,
		crate::{
			classes::{
				monoid::empty,
				semigroup::append,
			},
			types::{
				SendThunk,
				TryThunk,
			},
		},
	};

	#[test]
	fn test_ok() {
		let t: TrySendThunk<i32, ()> = TrySendThunk::ok(42);
		assert_eq!(t.evaluate(), Ok(42));
	}

	#[test]
	fn test_err() {
		let t: TrySendThunk<i32, &str> = TrySendThunk::err("error");
		assert_eq!(t.evaluate(), Err("error"));
	}

	#[test]
	fn test_new() {
		let t: TrySendThunk<i32, ()> = TrySendThunk::new(|| Ok(1 + 2));
		assert_eq!(t.evaluate(), Ok(3));
	}

	#[test]
	fn test_pure() {
		let t: TrySendThunk<i32, ()> = TrySendThunk::pure(42);
		assert_eq!(t.evaluate(), Ok(42));
	}

	#[test]
	fn test_defer() {
		let t: TrySendThunk<i32, ()> = TrySendThunk::defer(|| TrySendThunk::ok(99));
		assert_eq!(t.evaluate(), Ok(99));
	}

	#[test]
	fn test_map() {
		let t: TrySendThunk<i32, ()> = TrySendThunk::ok(21).map(|x| x * 2);
		assert_eq!(t.evaluate(), Ok(42));
	}

	#[test]
	fn test_map_failure_propagation() {
		let t = TrySendThunk::<i32, &str>::err("error").map(|x| x * 2);
		assert_eq!(t.evaluate(), Err("error"));
	}

	#[test]
	fn test_map_err() {
		let t: TrySendThunk<i32, i32> = TrySendThunk::err(21).map_err(|x| x * 2);
		assert_eq!(t.evaluate(), Err(42));
	}

	#[test]
	fn test_map_err_success_propagation() {
		let t = TrySendThunk::<i32, &str>::pure(42).map_err(|_| "new error");
		assert_eq!(t.evaluate(), Ok(42));
	}

	#[test]
	fn test_bimap_ok() {
		let t: TrySendThunk<i32, i32> = TrySendThunk::ok(5);
		assert_eq!(t.bimap(|s| s * 2, |e| e + 1).evaluate(), Ok(10));
	}

	#[test]
	fn test_bimap_err() {
		let t: TrySendThunk<i32, i32> = TrySendThunk::err(5);
		assert_eq!(t.bimap(|s| s * 2, |e| e + 1).evaluate(), Err(6));
	}

	#[test]
	fn test_bind() {
		let t: TrySendThunk<i32, ()> = TrySendThunk::ok(21).bind(|x| TrySendThunk::ok(x * 2));
		assert_eq!(t.evaluate(), Ok(42));
	}

	#[test]
	fn test_bind_failure_propagation() {
		let t = TrySendThunk::<i32, &str>::err("error").bind(|x| TrySendThunk::ok(x * 2));
		assert_eq!(t.evaluate(), Err("error"));
	}

	#[test]
	fn test_catch_recovers() {
		let t: TrySendThunk<i32, &str> = TrySendThunk::err("error").catch(|_| TrySendThunk::ok(42));
		assert_eq!(t.evaluate(), Ok(42));
	}

	#[test]
	fn test_catch_success_passes_through() {
		let t: TrySendThunk<i32, &str> = TrySendThunk::ok(42).catch(|_| TrySendThunk::ok(0));
		assert_eq!(t.evaluate(), Ok(42));
	}

	#[test]
	fn test_lift2() {
		let t1: TrySendThunk<i32, String> = TrySendThunk::ok(10);
		let t2: TrySendThunk<i32, String> = TrySendThunk::ok(20);
		let t3 = t1.lift2(t2, |a, b| a + b);
		assert_eq!(t3.evaluate(), Ok(30));
	}

	#[test]
	fn test_lift2_short_circuits() {
		let t1: TrySendThunk<i32, String> = TrySendThunk::err("fail".to_string());
		let t2: TrySendThunk<i32, String> = TrySendThunk::ok(20);
		let t3 = t1.lift2(t2, |a, b| a + b);
		assert_eq!(t3.evaluate(), Err("fail".to_string()));
	}

	#[test]
	fn test_then() {
		let t1: TrySendThunk<i32, String> = TrySendThunk::ok(10);
		let t2: TrySendThunk<i32, String> = TrySendThunk::ok(20);
		assert_eq!(t1.then(t2).evaluate(), Ok(20));
	}

	#[test]
	fn test_then_short_circuits() {
		let t1: TrySendThunk<i32, String> = TrySendThunk::err("fail".to_string());
		let t2: TrySendThunk<i32, String> = TrySendThunk::ok(20);
		assert_eq!(t1.then(t2).evaluate(), Err("fail".to_string()));
	}

	#[test]
	fn test_catch_unwind_with() {
		let t = TrySendThunk::<i32, i32>::catch_unwind_with(
			|| {
				if true {
					panic!("oops")
				}
				42
			},
			|_payload| -1,
		);
		assert_eq!(t.evaluate(), Err(-1));
	}

	#[test]
	fn test_catch_unwind_with_no_panic() {
		let t = TrySendThunk::<i32, i32>::catch_unwind_with(|| 42, |_payload| -1);
		assert_eq!(t.evaluate(), Ok(42));
	}

	#[test]
	fn test_catch_unwind() {
		let t = TrySendThunk::<i32, String>::catch_unwind(|| {
			if true {
				panic!("oops")
			}
			42
		});
		assert_eq!(t.evaluate(), Err("oops".to_string()));
	}

	#[test]
	fn test_catch_unwind_no_panic() {
		let t = TrySendThunk::<i32, String>::catch_unwind(|| 42);
		assert_eq!(t.evaluate(), Ok(42));
	}

	#[test]
	fn test_memoize_arc() {
		let t: TrySendThunk<i32, ()> = TrySendThunk::ok(42);
		let lazy = t.memoize_arc();
		assert_eq!(lazy.evaluate(), Ok(&42));
		// Second access returns cached value.
		assert_eq!(lazy.evaluate(), Ok(&42));
	}

	#[test]
	fn test_memoize_arc_thread_safety() {
		let t: TrySendThunk<i32, ()> = TrySendThunk::ok(42);
		let lazy = t.memoize_arc();
		let lazy_clone = lazy.clone();
		let handle = std::thread::spawn(move || {
			assert_eq!(lazy_clone.evaluate(), Ok(&42));
		});
		assert_eq!(lazy.evaluate(), Ok(&42));
		handle.join().unwrap();
	}

	#[test]
	fn test_semigroup() {
		let t1: TrySendThunk<String, ()> = TrySendThunk::ok("Hello".to_string());
		let t2: TrySendThunk<String, ()> = TrySendThunk::ok(" World".to_string());
		let t3 = append(t1, t2);
		assert_eq!(t3.evaluate(), Ok("Hello World".to_string()));
	}

	#[test]
	fn test_semigroup_error() {
		let t1: TrySendThunk<String, &str> = TrySendThunk::err("fail");
		let t2: TrySendThunk<String, &str> = TrySendThunk::ok(" World".to_string());
		let t3 = append(t1, t2);
		assert_eq!(t3.evaluate(), Err("fail"));
	}

	#[test]
	fn test_monoid() {
		let t: TrySendThunk<String, ()> = empty();
		assert_eq!(t.evaluate(), Ok("".to_string()));
	}

	#[test]
	fn test_from_try_thunk() {
		let thunk: TryThunk<i32, ()> = TryThunk::ok(42);
		let send_thunk = TrySendThunk::from(thunk);
		assert_eq!(send_thunk.evaluate(), Ok(42));
	}

	#[test]
	fn test_from_result() {
		let ok: TrySendThunk<i32, String> = TrySendThunk::from(Ok(42));
		assert_eq!(ok.evaluate(), Ok(42));

		let err: TrySendThunk<i32, String> = TrySendThunk::from(Err("error".to_string()));
		assert_eq!(err.evaluate(), Err("error".to_string()));
	}

	#[test]
	fn test_from_send_thunk() {
		let thunk = SendThunk::pure(42);
		let t: TrySendThunk<i32, ()> = TrySendThunk::from(thunk);
		assert_eq!(t.evaluate(), Ok(42));
	}

	#[test]
	fn test_debug() {
		let t = TrySendThunk::new(|| Ok::<i32, ()>(42));
		assert_eq!(format!("{:?}", t), "TrySendThunk(<unevaluated>)");
	}

	#[test]
	fn test_is_send() {
		fn assert_send<T: Send>() {}
		assert_send::<TrySendThunk<'static, i32, String>>();
	}

	#[test]
	fn test_send_across_thread() {
		let t: TrySendThunk<i32, ()> = TrySendThunk::ok(42);
		let handle = std::thread::spawn(move || t.evaluate());
		assert_eq!(handle.join().unwrap(), Ok(42));
	}

	#[test]
	fn test_deferrable() {
		use crate::classes::Deferrable;
		let t: TrySendThunk<i32, ()> = Deferrable::defer(|| TrySendThunk::ok(42));
		assert_eq!(t.evaluate(), Ok(42));
	}

	#[test]
	fn test_send_deferrable() {
		use crate::classes::SendDeferrable;
		let t: TrySendThunk<i32, ()> = SendDeferrable::send_defer(|| TrySendThunk::ok(42));
		assert_eq!(t.evaluate(), Ok(42));
	}
}
