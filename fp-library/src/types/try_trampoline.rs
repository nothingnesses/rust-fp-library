use crate::types::{Lazy, LazyConfig, TryLazy, trampoline::Trampoline};

/// A lazy, stack-safe computation that may fail with an error.
///
/// This is [`Trampoline<Result<A, E>>`] with ergonomic combinators.
///
/// ### Type Parameters
///
/// * `A`: The type of the success value.
/// * `E`: The type of the error value.
///
/// ### Fields
///
/// * `0`: The internal `Trampoline` wrapping a `Result`.
///
/// ### Examples
///
/// ```
/// use fp_library::types::*;
///
/// let task: TryTrampoline<i32, String> = TryTrampoline::ok(10);
/// assert_eq!(task.evaluate(), Ok(10));
/// ```
pub struct TryTrampoline<A: 'static, E: 'static>(Trampoline<Result<A, E>>);

impl<A: 'static + Send, E: 'static + Send> TryTrampoline<A, E> {
	/// Creates a successful `TryTrampoline`.
	///
	/// ### Type Signature
	///
	/// `forall e a. a -> TryTrampoline a e`
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
	/// A `TryTrampoline` representing success.
	///
	/// ### Examples
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
	///
	/// ### Type Signature
	///
	/// `forall e a. e -> TryTrampoline a e`
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
	/// A `TryTrampoline` representing failure.
	///
	/// ### Examples
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
	///
	/// ### Type Signature
	///
	/// `forall e a. (Unit -> Result a e) -> TryTrampoline a e`
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
	/// A `TryTrampoline` that executes `f` when run.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::types::*;
	///
	/// let task: TryTrampoline<i32, String> = TryTrampoline::new(|| Ok(42));
	/// assert_eq!(task.evaluate(), Ok(42));
	/// ```
	pub fn new<F>(f: F) -> Self
	where
		F: FnOnce() -> Result<A, E> + 'static,
	{
		TryTrampoline(Trampoline::new(f))
	}

	/// Maps over the success value.
	///
	/// ### Type Signature
	///
	/// `forall e b a. (a -> b, TryTrampoline a e) -> TryTrampoline b e`
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
	/// A new `TryTrampoline` with the transformed success value.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::types::*;
	///
	/// let task: TryTrampoline<i32, String> = TryTrampoline::ok(10).map(|x| x * 2);
	/// assert_eq!(task.evaluate(), Ok(20));
	/// ```
	pub fn map<B: 'static + Send, Func>(
		self,
		func: Func,
	) -> TryTrampoline<B, E>
	where
		Func: FnOnce(A) -> B + 'static,
	{
		TryTrampoline(self.0.map(|result| result.map(func)))
	}

	/// Maps over the error value.
	///
	/// ### Type Signature
	///
	/// `forall e2 e a. (e -> e2, TryTrampoline a e) -> TryTrampoline a e2`
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
	/// A new `TryTrampoline` with the transformed error value.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::types::*;
	///
	/// let task: TryTrampoline<i32, String> = TryTrampoline::err("error".to_string())
	///     .map_err(|e| e.to_uppercase());
	/// assert_eq!(task.evaluate(), Err("ERROR".to_string()));
	/// ```
	pub fn map_err<E2: 'static + Send, Func>(
		self,
		func: Func,
	) -> TryTrampoline<A, E2>
	where
		Func: FnOnce(E) -> E2 + 'static,
	{
		TryTrampoline(self.0.map(|result| result.map_err(func)))
	}

	/// Chains fallible computations.
	///
	/// ### Type Signature
	///
	/// `forall e b a. (a -> TryTrampoline b e, TryTrampoline a e) -> TryTrampoline b e`
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
	/// A new `TryTrampoline` that chains `f` after this task.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::types::*;
	///
	/// let task: TryTrampoline<i32, String> = TryTrampoline::ok(10).bind(|x| TryTrampoline::ok(x * 2));
	/// assert_eq!(task.evaluate(), Ok(20));
	/// ```
	pub fn bind<B: 'static + Send, F>(
		self,
		f: F,
	) -> TryTrampoline<B, E>
	where
		F: FnOnce(A) -> TryTrampoline<B, E> + 'static,
	{
		TryTrampoline(self.0.bind(|result| match result {
			Ok(a) => f(a).0,
			Err(e) => Trampoline::pure(Err(e)),
		}))
	}

	/// Recovers from an error.
	///
	/// ### Type Signature
	///
	/// `forall e a. (e -> TryTrampoline a e, TryTrampoline a e) -> TryTrampoline a e`
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
	/// A new `TryTrampoline` that attempts to recover from failure.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::types::*;
	///
	/// let task: TryTrampoline<i32, String> = TryTrampoline::err("error".to_string())
	///     .catch(|_| TryTrampoline::ok(42));
	/// assert_eq!(task.evaluate(), Ok(42));
	/// ```
	pub fn catch<F>(
		self,
		f: F,
	) -> Self
	where
		F: FnOnce(E) -> TryTrampoline<A, E> + 'static,
	{
		TryTrampoline(self.0.bind(|result| match result {
			Ok(a) => Trampoline::pure(Ok(a)),
			Err(e) => f(e).0,
		}))
	}

	/// Runs the computation, returning the result.
	///
	/// ### Type Signature
	///
	/// `forall e a. TryTrampoline a e -> Result a e`
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
	/// let task: TryTrampoline<i32, String> = TryTrampoline::ok(42);
	/// assert_eq!(task.evaluate(), Ok(42));
	/// ```
	pub fn evaluate(self) -> Result<A, E> {
		self.0.evaluate()
	}
}

impl<A, E> From<Trampoline<A>> for TryTrampoline<A, E>
where
	A: Send + 'static,
	E: Send + 'static,
{
	fn from(task: Trampoline<A>) -> Self {
		TryTrampoline::new(move || Ok(task.evaluate()))
	}
}

impl<A, E, Config> From<Lazy<'static, A, Config>> for TryTrampoline<A, E>
where
	A: Clone + Send + 'static,
	E: Send + 'static,
	Config: LazyConfig,
{
	fn from(memo: Lazy<'static, A, Config>) -> Self {
		TryTrampoline::new(move || Ok(memo.evaluate().clone()))
	}
}

impl<A, E, Config> From<TryLazy<'static, A, E, Config>> for TryTrampoline<A, E>
where
	A: Clone + Send + 'static,
	E: Clone + Send + 'static,
	Config: LazyConfig,
{
	fn from(memo: TryLazy<'static, A, E, Config>) -> Self {
		TryTrampoline::new(move || memo.evaluate().cloned().map_err(Clone::clone))
	}
}

#[cfg(test)]
mod tests {
	use super::*;

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
