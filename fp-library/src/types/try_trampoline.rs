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
				Trampoline,
				TryLazy,
			},
		},
		fp_macros::*,
	};

	/// A lazy, stack-safe computation that may fail with an error.
	///
	/// This is [`Trampoline<Result<A, E>>`] with ergonomic combinators.
	#[document_type_parameters("The type of the success value.", "The type of the error value.")]
	///
	#[document_fields("The internal `Trampoline` wrapping a `Result`.")]
	///
	pub struct TryTrampoline<A: 'static, E: 'static>(Trampoline<Result<A, E>>);

	#[document_type_parameters("The type of the success value.", "The type of the error value.")]
	#[document_parameters("The fallible trampoline computation.")]
	impl<A: 'static + Send, E: 'static + Send> TryTrampoline<A, E> {
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
		pub fn map<B: 'static + Send>(
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
		pub fn map_err<E2: 'static + Send>(
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
		pub fn bind<B: 'static + Send>(
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
		A: Send + 'static,
		E: Send + 'static,
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
		A: Clone + Send + 'static,
		E: Send + 'static,
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
		A: Clone + Send + 'static,
		E: Clone + Send + 'static,
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
	impl<A, E> Deferrable<'static> for TryTrampoline<A, E>
	where
		A: 'static + Send,
		E: 'static + Send,
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
}
pub use inner::*;

#[cfg(test)]
mod tests {
	use {
		super::*,
		crate::types::Trampoline,
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
}
