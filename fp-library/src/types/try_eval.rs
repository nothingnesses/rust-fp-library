//! Implementations for [`TryEval`], a fallible deferred computation type.
//!
//! This module provides the [`TryEval`] type, which represents a deferred computation that may fail.
//! It is the fallible counterpart to [`Eval`](crate::types::Eval).

use crate::types::{Eval, Memo, MemoConfig, TryMemo};

/// A deferred computation that may fail with error type `E`.
///
/// Like [`Eval`](crate::types::Eval), this is NOT memoized. Each `run()` re-executes.
/// Unlike [`Eval`](crate::types::Eval), the result is `Result<A, E>`.
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
/// let computation: TryEval<i32, &str> = TryEval::new(|| {
///     Ok(42)
/// });
///
/// match computation.run() {
///     Ok(val) => assert_eq!(val, 42),
///     Err(_) => panic!("Should not fail"),
/// }
/// ```
pub struct TryEval<'a, A, E> {
	thunk: Box<dyn FnOnce() -> Result<A, E> + 'a>,
}

impl<'a, A: 'a, E: 'a> TryEval<'a, A, E> {
	/// Creates a new TryEval from a thunk.
	///
	/// ### Type Signature
	///
	/// `forall e a. (Unit -> Result a e) -> TryEval a e`
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
	/// A new `TryEval` instance.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::types::*;
	///
	/// let try_eval: TryEval<i32, ()> = TryEval::new(|| Ok(42));
	/// assert_eq!(try_eval.run(), Ok(42));
	/// ```
	pub fn new<F>(f: F) -> Self
	where
		F: FnOnce() -> Result<A, E> + 'a,
	{
		TryEval { thunk: Box::new(f) }
	}

	/// Returns a pure value (already computed).
	///
	/// ### Type Signature
	///
	/// `forall e a. a -> TryEval a e`
	///
	/// ### Parameters
	///
	/// * `a`: The value to wrap.
	///
	/// ### Returns
	///
	/// A new `TryEval` instance containing the value.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::types::*;
	///
	/// let try_eval: TryEval<i32, ()> = TryEval::pure(42);
	/// assert_eq!(try_eval.run(), Ok(42));
	/// ```
	pub fn pure(a: A) -> Self
	where
		A: 'a,
	{
		TryEval::new(move || Ok(a))
	}

	/// Alias for [`pure`](Self::pure).
	///
	/// Creates a successful computation.
	///
	/// ### Type Signature
	///
	/// `forall e a. a -> TryEval a e`
	///
	/// ### Parameters
	///
	/// * `a`: The value to wrap.
	///
	/// ### Returns
	///
	/// A new `TryEval` instance containing the value.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::types::*;
	///
	/// let try_eval: TryEval<i32, ()> = TryEval::ok(42);
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
	/// `forall e a. e -> TryEval a e`
	///
	/// ### Parameters
	///
	/// * `e`: The error to wrap.
	///
	/// ### Returns
	///
	/// A new `TryEval` instance containing the error.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::types::*;
	///
	/// let try_eval: TryEval<i32, &str> = TryEval::err("error");
	/// assert_eq!(try_eval.run(), Err("error"));
	/// ```
	pub fn err(e: E) -> Self
	where
		E: 'a,
	{
		TryEval::new(move || Err(e))
	}

	/// Monadic bind: chains computations.
	///
	/// ### Type Signature
	///
	/// `forall e b a. (a -> TryEval b e, TryEval a e) -> TryEval b e`
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
	/// A new `TryEval` instance representing the chained computation.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::types::*;
	///
	/// let try_eval: TryEval<i32, ()> = TryEval::pure(21).bind(|x| TryEval::pure(x * 2));
	/// assert_eq!(try_eval.run(), Ok(42));
	/// ```
	pub fn bind<B: 'a, F>(
		self,
		f: F,
	) -> TryEval<'a, B, E>
	where
		F: FnOnce(A) -> TryEval<'a, B, E> + 'a,
	{
		TryEval::new(move || match (self.thunk)() {
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
	/// `forall e b a. (a -> TryEval b e, TryEval a e) -> TryEval b e`
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
	/// A new `TryEval` instance representing the chained computation.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::types::*;
	///
	/// let try_eval: TryEval<i32, ()> = TryEval::ok(21).and_then(|x| TryEval::ok(x * 2));
	/// assert_eq!(try_eval.run(), Ok(42));
	/// ```
	pub fn and_then<B: 'a, F>(
		self,
		f: F,
	) -> TryEval<'a, B, E>
	where
		F: FnOnce(A) -> TryEval<'a, B, E> + 'a,
	{
		self.bind(f)
	}

	/// Functor map: transforms the result.
	///
	/// ### Type Signature
	///
	/// `forall e b a. (a -> b, TryEval a e) -> TryEval b e`
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
	/// A new `TryEval` instance with the transformed result.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::types::*;
	///
	/// let try_eval: TryEval<i32, ()> = TryEval::pure(21).map(|x| x * 2);
	/// assert_eq!(try_eval.run(), Ok(42));
	/// ```
	pub fn map<B: 'a, F>(
		self,
		f: F,
	) -> TryEval<'a, B, E>
	where
		F: FnOnce(A) -> B + 'a,
	{
		TryEval::new(move || (self.thunk)().map(f))
	}

	/// Map error: transforms the error.
	///
	/// ### Type Signature
	///
	/// `forall e2 e a. (e -> e2, TryEval a e) -> TryEval a e2`
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
	/// A new `TryEval` instance with the transformed error.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::types::*;
	///
	/// let try_eval: TryEval<i32, i32> = TryEval::err(21).map_err(|x| x * 2);
	/// assert_eq!(try_eval.run(), Err(42));
	/// ```
	pub fn map_err<E2: 'a, F>(
		self,
		f: F,
	) -> TryEval<'a, A, E2>
	where
		F: FnOnce(E) -> E2 + 'a,
	{
		TryEval::new(move || (self.thunk)().map_err(f))
	}

	/// Forces evaluation and returns the result.
	///
	/// ### Type Signature
	///
	/// `forall e a. TryEval a e -> Result a e`
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
	/// let try_eval: TryEval<i32, ()> = TryEval::pure(42);
	/// assert_eq!(try_eval.run(), Ok(42));
	/// ```
	pub fn run(self) -> Result<A, E> {
		(self.thunk)()
	}
}

impl<'a, A, E, Config> From<Memo<'a, A, Config>> for TryEval<'a, A, E>
where
	A: Clone + 'a,
	E: 'a,
	Config: MemoConfig,
{
	fn from(memo: Memo<'a, A, Config>) -> Self {
		TryEval::new(move || Ok(memo.get().clone()))
	}
}

impl<'a, A, E, Config> From<TryMemo<'a, A, E, Config>> for TryEval<'a, A, E>
where
	A: Clone + 'a,
	E: Clone + 'a,
	Config: MemoConfig,
{
	fn from(memo: TryMemo<'a, A, E, Config>) -> Self {
		TryEval::new(move || memo.get().cloned().map_err(Clone::clone))
	}
}

impl<'a, A: 'a, E: 'a> From<Eval<'a, A>> for TryEval<'a, A, E> {
	fn from(eval: Eval<'a, A>) -> Self {
		TryEval::new(move || Ok(eval.run()))
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	/// Tests success path.
	///
	/// Verifies that `TryEval::pure` creates a successful computation.
	#[test]
	fn test_success() {
		let try_eval: TryEval<i32, ()> = TryEval::pure(42);
		assert_eq!(try_eval.run(), Ok(42));
	}

	/// Tests failure path.
	///
	/// Verifies that `TryEval::err` creates a failed computation.
	#[test]
	fn test_failure() {
		let try_eval: TryEval<i32, &str> = TryEval::err("error");
		assert_eq!(try_eval.run(), Err("error"));
	}

	/// Tests `TryEval::map`.
	///
	/// Verifies that `map` transforms the success value.
	#[test]
	fn test_map() {
		let try_eval: TryEval<i32, ()> = TryEval::pure(21).map(|x| x * 2);
		assert_eq!(try_eval.run(), Ok(42));
	}

	/// Tests `TryEval::map_err`.
	///
	/// Verifies that `map_err` transforms the error value.
	#[test]
	fn test_map_err() {
		let try_eval: TryEval<i32, i32> = TryEval::err(21).map_err(|x| x * 2);
		assert_eq!(try_eval.run(), Err(42));
	}

	/// Tests `TryEval::bind`.
	///
	/// Verifies that `bind` chains computations.
	#[test]
	fn test_bind() {
		let try_eval: TryEval<i32, ()> = TryEval::pure(21).bind(|x| TryEval::pure(x * 2));
		assert_eq!(try_eval.run(), Ok(42));
	}

	/// Tests borrowing in TryEval.
	///
	/// Verifies that `TryEval` can capture references.
	#[test]
	fn test_borrowing() {
		let x = 42;
		let try_eval: TryEval<&i32, ()> = TryEval::new(|| Ok(&x));
		assert_eq!(try_eval.run(), Ok(&42));
	}

	/// Tests `TryEval::bind` failure propagation.
	///
	/// Verifies that if the first computation fails, the second one is not executed.
	#[test]
	fn test_bind_failure() {
		let try_eval = TryEval::<i32, &str>::err("error").bind(|x| TryEval::pure(x * 2));
		assert_eq!(try_eval.run(), Err("error"));
	}

	/// Tests `TryEval::map` failure propagation.
	///
	/// Verifies that `map` is not executed if the computation fails.
	#[test]
	fn test_map_failure() {
		let try_eval = TryEval::<i32, &str>::err("error").map(|x| x * 2);
		assert_eq!(try_eval.run(), Err("error"));
	}

	/// Tests `TryEval::map_err` success propagation.
	///
	/// Verifies that `map_err` is not executed if the computation succeeds.
	#[test]
	fn test_map_err_success() {
		let try_eval = TryEval::<i32, &str>::pure(42).map_err(|_| "new error");
		assert_eq!(try_eval.run(), Ok(42));
	}

	/// Tests `From<Memo>`.
	#[test]
	fn test_try_eval_from_memo() {
		use crate::types::RcMemo;
		let memo = RcMemo::new(|| 42);
		let try_eval: TryEval<i32, ()> = TryEval::from(memo);
		assert_eq!(try_eval.run(), Ok(42));
	}

	/// Tests `From<TryMemo>`.
	#[test]
	fn test_try_eval_from_try_memo() {
		use crate::types::RcTryMemo;
		let memo = RcTryMemo::new(|| Ok(42));
		let try_eval: TryEval<i32, ()> = TryEval::from(memo);
		assert_eq!(try_eval.run(), Ok(42));
	}

	/// Tests `Eval::into_try`.
	///
	/// Verifies that `From<Eval>` converts an `Eval` into a `TryEval` that succeeds.
	#[test]
	fn test_try_eval_from_eval() {
		let eval = Eval::pure(42);
		let try_eval: TryEval<i32, ()> = TryEval::from(eval);
		assert_eq!(try_eval.run(), Ok(42));
	}
}
