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
//!     .bind(|x| TryTask::ok(x + 5));
//!
//! assert_eq!(task.run(), Ok(25));
//! ```

use crate::types::{Memo, MemoConfig, TryMemo, task::Task};

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
		TryTask { inner: Task::pure(Ok(a)) }
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
		TryTask { inner: Task::pure(Err(e)) }
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
	/// let task: TryTask<i32, String> = TryTask::new(|| Ok(42));
	/// assert_eq!(task.run(), Ok(42));
	/// ```
	pub fn new<F>(f: F) -> Self
	where
		F: FnOnce() -> Result<A, E> + 'static,
	{
		TryTask { inner: Task::new(f) }
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
	/// let task: TryTask<i32, String> = TryTask::ok(10).bind(|x| TryTask::ok(x * 2));
	/// assert_eq!(task.run(), Ok(20));
	/// ```
	pub fn bind<B: 'static + Send, F>(
		self,
		f: F,
	) -> TryTask<B, E>
	where
		F: FnOnce(A) -> TryTask<B, E> + 'static,
	{
		TryTask {
			inner: self.inner.bind(|result| match result {
				Ok(a) => f(a).inner,
				Err(e) => Task::pure(Err(e)),
			}),
		}
	}

	/// Alias for [`bind`](Self::bind).
	///
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
		self.bind(f)
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
			inner: self.inner.bind(|result| match result {
				Ok(a) => Task::pure(Ok(a)),
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

impl<A, E> From<Task<A>> for TryTask<A, E>
where
	A: Send + 'static,
	E: Send + 'static,
{
	fn from(task: Task<A>) -> Self {
		TryTask::new(move || Ok(task.run()))
	}
}

impl<A, E, Config> From<Memo<'static, A, Config>> for TryTask<A, E>
where
	A: Clone + Send + 'static,
	E: Send + 'static,
	Config: MemoConfig,
{
	fn from(memo: Memo<'static, A, Config>) -> Self {
		TryTask::new(move || Ok(memo.get().clone()))
	}
}

impl<A, E, Config> From<TryMemo<'static, A, E, Config>> for TryTask<A, E>
where
	A: Clone + Send + 'static,
	E: Clone + Send + 'static,
	Config: MemoConfig,
{
	fn from(memo: TryMemo<'static, A, E, Config>) -> Self {
		TryTask::new(move || memo.get().cloned().map_err(Clone::clone))
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

	/// Tests `TryTask::bind`.
	///
	/// Verifies that `bind` chains computations.
	#[test]
	fn test_try_task_bind() {
		let task: TryTask<i32, String> = TryTask::ok(10).bind(|x| TryTask::ok(x * 2));
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

	/// Tests `TryTask::new`.
	///
	/// Verifies that `new` creates a lazy task.
	#[test]
	fn test_try_task_new() {
		let task: TryTask<i32, String> = TryTask::new(|| Ok(42));
		assert_eq!(task.run(), Ok(42));
	}

	/// Tests `From<Task>`.
	#[test]
	fn test_try_task_from_task() {
		let task = Task::pure(42);
		let try_task: TryTask<i32, String> = TryTask::from(task);
		assert_eq!(try_task.run(), Ok(42));
	}

	/// Tests `From<Memo>`.
	#[test]
	fn test_try_task_from_memo() {
		use crate::types::ArcMemo;
		let memo = ArcMemo::new(|| 42);
		let try_task: TryTask<i32, String> = TryTask::from(memo);
		assert_eq!(try_task.run(), Ok(42));
	}

	/// Tests `From<TryMemo>`.
	#[test]
	fn test_try_task_from_try_memo() {
		use crate::types::ArcTryMemo;
		let memo = ArcTryMemo::new(|| Ok(42));
		let try_task: TryTask<i32, String> = TryTask::from(memo);
		assert_eq!(try_task.run(), Ok(42));
	}
}
