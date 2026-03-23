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
			classes::Deferrable,
			types::{
				Lazy,
				LazyConfig,
				Step,
				Trampoline,
				TryLazy,
			},
		},
		fp_macros::*,
		std::fmt,
	};

	/// A lazy, stack-safe computation that may fail with an error.
	///
	/// This is [`Trampoline<Result<A, E>>`] with ergonomic combinators.
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
		pub fn ok(a: A) -> Self {
			TryTrampoline(Trampoline::pure(Ok(a)))
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
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let task: TryTrampoline<i32, String> = TryTrampoline::defer(|| TryTrampoline::ok(42));
		/// assert_eq!(task.evaluate(), Ok(42));
		/// ```
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
		pub fn map_err<E2: 'static>(
			self,
			func: impl FnOnce(E) -> E2 + 'static,
		) -> TryTrampoline<A, E2> {
			TryTrampoline(self.0.map(|result| result.map_err(func)))
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
		pub fn catch(
			self,
			f: impl FnOnce(E) -> TryTrampoline<A, E> + 'static,
		) -> Self {
			TryTrampoline(self.0.bind(|result| match result {
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
			TryTrampoline(Trampoline::tail_rec_m(
				move |state: Result<S, E>| match state {
					Err(e) => Trampoline::pure(Step::Done(Err(e))),
					Ok(s) => f(s).0.map(|result| match result {
						Ok(Step::Loop(next)) => Step::Loop(Ok(next)),
						Ok(Step::Done(a)) => Step::Done(Ok(a)),
						Err(e) => Step::Done(Err(e)),
					}),
				},
				Ok(initial),
			))
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
		pub fn evaluate(self) -> Result<A, E> {
			self.0.evaluate()
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
			TryTrampoline(Trampoline::pure(Ok(memo.evaluate().clone())))
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
		Config: LazyConfig,
	{
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
			TryTrampoline(Trampoline::pure(memo.evaluate().cloned().map_err(Clone::clone)))
		}
	}

	#[document_type_parameters("The type of the success value.", "The type of the error value.")]
	impl<A, E> From<crate::types::TryThunk<'static, A, E>> for TryTrampoline<A, E>
	where
		A: 'static,
		E: 'static,
	{
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

	/// Tests `TryTrampoline::new`.
	///
	/// Verifies that `new` creates a lazy task.
	#[test]
	fn test_try_task_new() {
		let task: TryTrampoline<i32, String> = TryTrampoline::new(|| Ok(42));
		assert_eq!(task.evaluate(), Ok(42));
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
}
