//! Implementation of the `TryTask` type.
//!
//! This module provides the [`TryTask`] type, which represents a lazy, stack-safe computation that may fail.
//! It is a wrapper around `Task<Result<A, E>>`.
//!
//! ### Examples
//!
//! ```
//! use fp_library::types::*;
//!
//! let task: TryTask<i32, String> = TryTask::ok(10)
//!     .map(|x| x * 2)
//!     .and_then(|x| TryTask::ok(x + 5));
//!
//! assert_eq!(task.run(), Ok(25));
//! ```

use crate::types::task::Task;

/// A lazy, stack-safe computation that may fail with an error.
///
/// This is `Task<Result<A, E>>` with ergonomic combinators.
///
/// ### Type Parameters
///
/// * `A`: The type of the success value.
/// * `E`: The type of the error value.
///
/// ### Fields
///
/// * `inner`: The internal `Task` wrapping a `Result`.
///
/// ### Examples
///
/// ```
/// use fp_library::types::*;
///
/// let task: TryTask<i32, String> = TryTask::ok(10);
/// assert_eq!(task.run(), Ok(10));
/// ```
pub struct TryTask<A: 'static, E: 'static> {
	inner: Task<Result<A, E>>,
}

impl<A: 'static + Send, E: 'static + Send> TryTask<A, E> {
	/// Creates a successful `TryTask`.
	///
	/// ### Type Signature
	///
	/// `forall e a. a -> TryTask a e`
	///
	/// ### Type Parameters
	///
	/// * `A`: The type of the success value.
	/// * `E`: The type of the error value.
	///
	/// ### Parameters
	///
	/// * `a`: The success value.
	///
	/// ### Returns
	///
	/// A `TryTask` representing success.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::types::*;
	///
	/// let task: TryTask<i32, String> = TryTask::ok(42);
	/// assert_eq!(task.run(), Ok(42));
	/// ```
	pub fn ok(a: A) -> Self {
		TryTask { inner: Task::now(Ok(a)) }
	}

	/// Creates a failed `TryTask`.
	///
	/// ### Type Signature
	///
	/// `forall e a. e -> TryTask a e`
	///
	/// ### Type Parameters
	///
	/// * `A`: The type of the success value.
	/// * `E`: The type of the error value.
	///
	/// ### Parameters
	///
	/// * `e`: The error value.
	///
	/// ### Returns
	///
	/// A `TryTask` representing failure.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::types::*;
	///
	/// let task: TryTask<i32, String> = TryTask::err("error".to_string());
	/// assert_eq!(task.run(), Err("error".to_string()));
	/// ```
	pub fn err(e: E) -> Self {
		TryTask { inner: Task::now(Err(e)) }
	}

	/// Creates a lazy `TryTask` that may fail.
	///
	/// ### Type Signature
	///
	/// `forall e a. (Unit -> Result a e) -> TryTask a e`
	///
	/// ### Type Parameters
	///
	/// * `A`: The type of the success value.
	/// * `E`: The type of the error value.
	/// * `F`: The type of the closure.
	///
	/// ### Parameters
	///
	/// * `f`: The closure to execute.
	///
	/// ### Returns
	///
	/// A `TryTask` that executes `f` when run.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::types::*;
	///
	/// let task: TryTask<i32, String> = TryTask::try_later(|| Ok(42));
	/// assert_eq!(task.run(), Ok(42));
	/// ```
	pub fn try_later<F>(f: F) -> Self
	where
		F: FnOnce() -> Result<A, E> + 'static,
	{
		TryTask { inner: Task::later(f) }
	}

	/// Maps over the success value.
	///
	/// ### Type Signature
	///
	/// `forall e b a. (a -> b, TryTask a e) -> TryTask b e`
	///
	/// ### Type Parameters
	///
	/// * `B`: The type of the new success value.
	/// * `F`: The type of the mapping function.
	///
	/// ### Parameters
	///
	/// * `f`: The function to apply to the success value.
	///
	/// ### Returns
	///
	/// A new `TryTask` with the transformed success value.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::types::*;
	///
	/// let task: TryTask<i32, String> = TryTask::ok(10).map(|x| x * 2);
	/// assert_eq!(task.run(), Ok(20));
	/// ```
	pub fn map<B: 'static + Send, F>(
		self,
		f: F,
	) -> TryTask<B, E>
	where
		F: FnOnce(A) -> B + 'static,
	{
		TryTask { inner: self.inner.map(|result| result.map(f)) }
	}

	/// Maps over the error value.
	///
	/// ### Type Signature
	///
	/// `forall e2 e a. (e -> e2, TryTask a e) -> TryTask a e2`
	///
	/// ### Type Parameters
	///
	/// * `E2`: The type of the new error value.
	/// * `F`: The type of the mapping function.
	///
	/// ### Parameters
	///
	/// * `f`: The function to apply to the error value.
	///
	/// ### Returns
	///
	/// A new `TryTask` with the transformed error value.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::types::*;
	///
	/// let task: TryTask<i32, String> = TryTask::err("error".to_string())
	///     .map_err(|e| e.to_uppercase());
	/// assert_eq!(task.run(), Err("ERROR".to_string()));
	/// ```
	pub fn map_err<E2: 'static + Send, F>(
		self,
		f: F,
	) -> TryTask<A, E2>
	where
		F: FnOnce(E) -> E2 + 'static,
	{
		TryTask { inner: self.inner.map(|result| result.map_err(f)) }
	}

	/// Chains fallible computations.
	///
	/// ### Type Signature
	///
	/// `forall e b a. (a -> TryTask b e, TryTask a e) -> TryTask b e`
	///
	/// ### Type Parameters
	///
	/// * `B`: The type of the new success value.
	/// * `F`: The type of the binding function.
	///
	/// ### Parameters
	///
	/// * `f`: The function to apply to the success value.
	///
	/// ### Returns
	///
	/// A new `TryTask` that chains `f` after this task.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::types::*;
	///
	/// let task: TryTask<i32, String> = TryTask::ok(10).and_then(|x| TryTask::ok(x * 2));
	/// assert_eq!(task.run(), Ok(20));
	/// ```
	pub fn and_then<B: 'static + Send, F>(
		self,
		f: F,
	) -> TryTask<B, E>
	where
		F: FnOnce(A) -> TryTask<B, E> + 'static,
	{
		TryTask {
			inner: self.inner.flat_map(|result| match result {
				Ok(a) => f(a).inner,
				Err(e) => Task::now(Err(e)),
			}),
		}
	}

	/// Recovers from an error.
	///
	/// ### Type Signature
	///
	/// `forall e a. (e -> TryTask a e, TryTask a e) -> TryTask a e`
	///
	/// ### Type Parameters
	///
	/// * `F`: The type of the recovery function.
	///
	/// ### Parameters
	///
	/// * `f`: The function to apply to the error value.
	///
	/// ### Returns
	///
	/// A new `TryTask` that attempts to recover from failure.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::types::*;
	///
	/// let task: TryTask<i32, String> = TryTask::err("error".to_string())
	///     .or_else(|_| TryTask::ok(42));
	/// assert_eq!(task.run(), Ok(42));
	/// ```
	pub fn or_else<F>(
		self,
		f: F,
	) -> Self
	where
		F: FnOnce(E) -> TryTask<A, E> + 'static,
	{
		TryTask {
			inner: self.inner.flat_map(|result| match result {
				Ok(a) => Task::now(Ok(a)),
				Err(e) => f(e).inner,
			}),
		}
	}

	/// Runs the computation, returning the result.
	///
	/// ### Type Signature
	///
	/// `forall e a. TryTask a e -> Result a e`
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
	/// let task: TryTask<i32, String> = TryTask::ok(42);
	/// assert_eq!(task.run(), Ok(42));
	/// ```
	pub fn run(self) -> Result<A, E> {
		self.inner.run()
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	/// Tests `TryTask::ok`.
	///
	/// Verifies that `ok` creates a successful task.
	#[test]
	fn test_try_task_ok() {
		let task: TryTask<i32, String> = TryTask::ok(42);
		assert_eq!(task.run(), Ok(42));
	}

	/// Tests `TryTask::err`.
	///
	/// Verifies that `err` creates a failed task.
	#[test]
	fn test_try_task_err() {
		let task: TryTask<i32, String> = TryTask::err("error".to_string());
		assert_eq!(task.run(), Err("error".to_string()));
	}

	/// Tests `TryTask::map`.
	///
	/// Verifies that `map` transforms the success value.
	#[test]
	fn test_try_task_map() {
		let task: TryTask<i32, String> = TryTask::ok(10).map(|x| x * 2);
		assert_eq!(task.run(), Ok(20));
	}

	/// Tests `TryTask::map_err`.
	///
	/// Verifies that `map_err` transforms the error value.
	#[test]
	fn test_try_task_map_err() {
		let task: TryTask<i32, String> =
			TryTask::err("error".to_string()).map_err(|e| e.to_uppercase());
		assert_eq!(task.run(), Err("ERROR".to_string()));
	}

	/// Tests `TryTask::and_then`.
	///
	/// Verifies that `and_then` chains computations.
	#[test]
	fn test_try_task_and_then() {
		let task: TryTask<i32, String> = TryTask::ok(10).and_then(|x| TryTask::ok(x * 2));
		assert_eq!(task.run(), Ok(20));
	}

	/// Tests `TryTask::or_else`.
	///
	/// Verifies that `or_else` recovers from failure.
	#[test]
	fn test_try_task_or_else() {
		let task: TryTask<i32, String> =
			TryTask::err("error".to_string()).or_else(|_| TryTask::ok(42));
		assert_eq!(task.run(), Ok(42));
	}

	/// Tests `TryTask::try_later`.
	///
	/// Verifies that `try_later` creates a lazy task.
	#[test]
	fn test_try_task_try_later() {
		let task: TryTask<i32, String> = TryTask::try_later(|| Ok(42));
		assert_eq!(task.run(), Ok(42));
	}
}
