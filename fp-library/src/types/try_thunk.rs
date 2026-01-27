//! Implementations for [`TryThunk`], a fallible deferred computation type.
//!
//! This module provides the [`TryThunk`] type, which represents a deferred computation that may fail.
//! It is the fallible counterpart to [`Thunk`].

use crate::types::{Lazy, LazyConfig, Thunk, TryLazy};

/// A deferred computation that may fail with error type `E`.
///
/// Like [`Thunk`], this is NOT memoized. Each `run()` re-executes.
/// Unlike [`Thunk`], the result is `Result<A, E>`.
///
/// ### Type Parameters
///
/// * `A`: The type of the value produced by the computation on success.
/// * `E`: The type of the error produced by the computation on failure.
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
/// let computation: TryThunk<i32, &str> = TryThunk::new(|| {
///     Ok(42)
/// });
///
/// match computation.run() {
///     Ok(val) => assert_eq!(val, 42),
///     Err(_) => panic!("Should not fail"),
/// }
/// ```
pub struct TryThunk<'a, A, E> {
	thunk: Box<dyn FnOnce() -> Result<A, E> + 'a>,
}

impl<'a, A: 'a, E: 'a> TryThunk<'a, A, E> {
	/// Creates a new TryThunk from a thunk.
	///
	/// ### Type Signature
	///
	/// `forall e a. (Unit -> Result a e) -> TryThunk a e`
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
	/// A new `TryThunk` instance.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::types::*;
	///
	/// let try_eval: TryThunk<i32, ()> = TryThunk::new(|| Ok(42));
	/// assert_eq!(try_eval.run(), Ok(42));
	/// ```
	pub fn new<F>(f: F) -> Self
	where
		F: FnOnce() -> Result<A, E> + 'a,
	{
		TryThunk { thunk: Box::new(f) }
	}

	/// Returns a pure value (already computed).
	///
	/// ### Type Signature
	///
	/// `forall e a. a -> TryThunk a e`
	///
	/// ### Parameters
	///
	/// * `a`: The value to wrap.
	///
	/// ### Returns
	///
	/// A new `TryThunk` instance containing the value.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::types::*;
	///
	/// let try_eval: TryThunk<i32, ()> = TryThunk::pure(42);
	/// assert_eq!(try_eval.run(), Ok(42));
	/// ```
	pub fn pure(a: A) -> Self
	where
		A: 'a,
	{
		TryThunk::new(move || Ok(a))
	}

	/// Alias for [`pure`](Self::pure).
	///
	/// Creates a successful computation.
	///
	/// ### Type Signature
	///
	/// `forall e a. a -> TryThunk a e`
	///
	/// ### Parameters
	///
	/// * `a`: The value to wrap.
	///
	/// ### Returns
	///
	/// A new `TryThunk` instance containing the value.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::types::*;
	///
	/// let try_eval: TryThunk<i32, ()> = TryThunk::ok(42);
	/// assert_eq!(try_eval.run(), Ok(42));
	/// ```
	pub fn ok(a: A) -> Self
	where
		A: 'a,
	{
		Self::pure(a)
	}

	/// Returns a pure error.
	///
	/// ### Type Signature
	///
	/// `forall e a. e -> TryThunk a e`
	///
	/// ### Parameters
	///
	/// * `e`: The error to wrap.
	///
	/// ### Returns
	///
	/// A new `TryThunk` instance containing the error.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::types::*;
	///
	/// let try_eval: TryThunk<i32, &str> = TryThunk::err("error");
	/// assert_eq!(try_eval.run(), Err("error"));
	/// ```
	pub fn err(e: E) -> Self
	where
		E: 'a,
	{
		TryThunk::new(move || Err(e))
	}

	/// Monadic bind: chains computations.
	///
	/// ### Type Signature
	///
	/// `forall e b a. (a -> TryThunk b e, TryThunk a e) -> TryThunk b e`
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
	/// A new `TryThunk` instance representing the chained computation.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::types::*;
	///
	/// let try_eval: TryThunk<i32, ()> = TryThunk::pure(21).bind(|x| TryThunk::pure(x * 2));
	/// assert_eq!(try_eval.run(), Ok(42));
	/// ```
	pub fn bind<B: 'a, F>(
		self,
		f: F,
	) -> TryThunk<'a, B, E>
	where
		F: FnOnce(A) -> TryThunk<'a, B, E> + 'a,
	{
		TryThunk::new(move || match (self.thunk)() {
			Ok(a) => (f(a).thunk)(),
			Err(e) => Err(e),
		})
	}

	/// Alias for [`bind`](Self::bind).
	///
	/// Chains computations.
	///
	/// ### Type Signature
	///
	/// `forall e b a. (a -> TryThunk b e, TryThunk a e) -> TryThunk b e`
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
	/// A new `TryThunk` instance representing the chained computation.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::types::*;
	///
	/// let try_eval: TryThunk<i32, ()> = TryThunk::ok(21).and_then(|x| TryThunk::ok(x * 2));
	/// assert_eq!(try_eval.run(), Ok(42));
	/// ```
	pub fn and_then<B: 'a, F>(
		self,
		f: F,
	) -> TryThunk<'a, B, E>
	where
		F: FnOnce(A) -> TryThunk<'a, B, E> + 'a,
	{
		self.bind(f)
	}

	/// Functor map: transforms the result.
	///
	/// ### Type Signature
	///
	/// `forall e b a. (a -> b, TryThunk a e) -> TryThunk b e`
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
	/// A new `TryThunk` instance with the transformed result.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::types::*;
	///
	/// let try_eval: TryThunk<i32, ()> = TryThunk::pure(21).map(|x| x * 2);
	/// assert_eq!(try_eval.run(), Ok(42));
	/// ```
	pub fn map<B: 'a, F>(
		self,
		f: F,
	) -> TryThunk<'a, B, E>
	where
		F: FnOnce(A) -> B + 'a,
	{
		TryThunk::new(move || (self.thunk)().map(f))
	}

	/// Map error: transforms the error.
	///
	/// ### Type Signature
	///
	/// `forall e2 e a. (e -> e2, TryThunk a e) -> TryThunk a e2`
	///
	/// ### Type Parameters
	///
	/// * `E2`: The type of the new error.
	/// * `F`: The type of the transformation function.
	///
	/// ### Parameters
	///
	/// * `f`: The function to apply to the error.
	///
	/// ### Returns
	///
	/// A new `TryThunk` instance with the transformed error.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::types::*;
	///
	/// let try_eval: TryThunk<i32, i32> = TryThunk::err(21).map_err(|x| x * 2);
	/// assert_eq!(try_eval.run(), Err(42));
	/// ```
	pub fn map_err<E2: 'a, F>(
		self,
		f: F,
	) -> TryThunk<'a, A, E2>
	where
		F: FnOnce(E) -> E2 + 'a,
	{
		TryThunk::new(move || (self.thunk)().map_err(f))
	}

	/// Forces evaluation and returns the result.
	///
	/// ### Type Signature
	///
	/// `forall e a. TryThunk a e -> Result a e`
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
	/// let try_eval: TryThunk<i32, ()> = TryThunk::pure(42);
	/// assert_eq!(try_eval.run(), Ok(42));
	/// ```
	pub fn run(self) -> Result<A, E> {
		(self.thunk)()
	}
}

impl<'a, A, E, Config> From<Lazy<'a, A, Config>> for TryThunk<'a, A, E>
where
	A: Clone + 'a,
	E: 'a,
	Config: LazyConfig,
{
	fn from(memo: Lazy<'a, A, Config>) -> Self {
		TryThunk::new(move || Ok(memo.get().clone()))
	}
}

impl<'a, A, E, Config> From<TryLazy<'a, A, E, Config>> for TryThunk<'a, A, E>
where
	A: Clone + 'a,
	E: Clone + 'a,
	Config: LazyConfig,
{
	fn from(memo: TryLazy<'a, A, E, Config>) -> Self {
		TryThunk::new(move || memo.get().cloned().map_err(Clone::clone))
	}
}

impl<'a, A: 'a, E: 'a> From<Thunk<'a, A>> for TryThunk<'a, A, E> {
	fn from(eval: Thunk<'a, A>) -> Self {
		TryThunk::new(move || Ok(eval.run()))
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	/// Tests success path.
	///
	/// Verifies that `TryThunk::pure` creates a successful computation.
	#[test]
	fn test_success() {
		let try_eval: TryThunk<i32, ()> = TryThunk::pure(42);
		assert_eq!(try_eval.run(), Ok(42));
	}

	/// Tests failure path.
	///
	/// Verifies that `TryThunk::err` creates a failed computation.
	#[test]
	fn test_failure() {
		let try_eval: TryThunk<i32, &str> = TryThunk::err("error");
		assert_eq!(try_eval.run(), Err("error"));
	}

	/// Tests `TryThunk::map`.
	///
	/// Verifies that `map` transforms the success value.
	#[test]
	fn test_map() {
		let try_eval: TryThunk<i32, ()> = TryThunk::pure(21).map(|x| x * 2);
		assert_eq!(try_eval.run(), Ok(42));
	}

	/// Tests `TryThunk::map_err`.
	///
	/// Verifies that `map_err` transforms the error value.
	#[test]
	fn test_map_err() {
		let try_eval: TryThunk<i32, i32> = TryThunk::err(21).map_err(|x| x * 2);
		assert_eq!(try_eval.run(), Err(42));
	}

	/// Tests `TryThunk::bind`.
	///
	/// Verifies that `bind` chains computations.
	#[test]
	fn test_bind() {
		let try_eval: TryThunk<i32, ()> = TryThunk::pure(21).bind(|x| TryThunk::pure(x * 2));
		assert_eq!(try_eval.run(), Ok(42));
	}

	/// Tests borrowing in TryThunk.
	///
	/// Verifies that `TryThunk` can capture references.
	#[test]
	fn test_borrowing() {
		let x = 42;
		let try_eval: TryThunk<&i32, ()> = TryThunk::new(|| Ok(&x));
		assert_eq!(try_eval.run(), Ok(&42));
	}

	/// Tests `TryThunk::bind` failure propagation.
	///
	/// Verifies that if the first computation fails, the second one is not executed.
	#[test]
	fn test_bind_failure() {
		let try_eval = TryThunk::<i32, &str>::err("error").bind(|x| TryThunk::pure(x * 2));
		assert_eq!(try_eval.run(), Err("error"));
	}

	/// Tests `TryThunk::map` failure propagation.
	///
	/// Verifies that `map` is not executed if the computation fails.
	#[test]
	fn test_map_failure() {
		let try_eval = TryThunk::<i32, &str>::err("error").map(|x| x * 2);
		assert_eq!(try_eval.run(), Err("error"));
	}

	/// Tests `TryThunk::map_err` success propagation.
	///
	/// Verifies that `map_err` is not executed if the computation succeeds.
	#[test]
	fn test_map_err_success() {
		let try_eval = TryThunk::<i32, &str>::pure(42).map_err(|_| "new error");
		assert_eq!(try_eval.run(), Ok(42));
	}

	/// Tests `From<Lazy>`.
	#[test]
	fn test_try_eval_from_memo() {
		use crate::types::RcLazy;
		let memo = RcLazy::new(|| 42);
		let try_eval: TryThunk<i32, ()> = TryThunk::from(memo);
		assert_eq!(try_eval.run(), Ok(42));
	}

	/// Tests `From<TryLazy>`.
	#[test]
	fn test_try_eval_from_try_memo() {
		use crate::types::RcTryLazy;
		let memo = RcTryLazy::new(|| Ok(42));
		let try_eval: TryThunk<i32, ()> = TryThunk::from(memo);
		assert_eq!(try_eval.run(), Ok(42));
	}

	/// Tests `Thunk::into_try`.
	///
	/// Verifies that `From<Thunk>` converts an `Thunk` into a `TryThunk` that succeeds.
	#[test]
	fn test_try_eval_from_eval() {
		let eval = Thunk::pure(42);
		let try_eval: TryThunk<i32, ()> = TryThunk::from(eval);
		assert_eq!(try_eval.run(), Ok(42));
	}
}
