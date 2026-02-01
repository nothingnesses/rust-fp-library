use crate::types::{Lazy, LazyConfig, Thunk, TryLazy};
use fp_macros::doc_params;
use fp_macros::doc_type_params;

/// A deferred computation that may fail with error type `E`.
///
/// Like [`Thunk`], this is NOT memoized. Each [`TryThunk::evaluate`] re-executes.
/// Unlike [`Thunk`], the result is [`Result<A, E>`].
///
/// ### Type Parameters
///
/// * `A`: The type of the value produced by the computation on success.
/// * `E`: The type of the error produced by the computation on failure.
///
/// ### Fields
///
/// * `0`: The closure that performs the computation.
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
/// match computation.evaluate() {
///     Ok(val) => assert_eq!(val, 42),
///     Err(_) => panic!("Should not fail"),
/// }
/// ```
pub struct TryThunk<'a, A, E>(Box<dyn FnOnce() -> Result<A, E> + 'a>);

impl<'a, A: 'a, E: 'a> TryThunk<'a, A, E> {
	/// Creates a new `TryThunk` from a thunk.
	///
	/// ### Type Signature
	///
	/// `forall e a. (Unit -> Result a e) -> TryThunk a e`
	///
	/// ### Type Parameters
	///
	#[doc_type_params("The type of the thunk.")]
	///
	/// ### Parameters
	///
	#[doc_params("The thunk to wrap.")]
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
	/// let try_thunk: TryThunk<i32, ()> = TryThunk::new(|| Ok(42));
	/// assert_eq!(try_thunk.evaluate(), Ok(42));
	/// ```
	pub fn new<F>(f: F) -> Self
	where
		F: FnOnce() -> Result<A, E> + 'a,
	{
		TryThunk(Box::new(f))
	}

	/// Returns a pure value (already computed).
	///
	/// ### Type Signature
	///
	/// `forall e a. a -> TryThunk a e`
	///
	/// ### Parameters
	///
	#[doc_params("The value to wrap.")]
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
	/// let try_thunk: TryThunk<i32, ()> = TryThunk::pure(42);
	/// assert_eq!(try_thunk.evaluate(), Ok(42));
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
	#[doc_params("The value to wrap.")]
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
	/// let try_thunk: TryThunk<i32, ()> = TryThunk::ok(42);
	/// assert_eq!(try_thunk.evaluate(), Ok(42));
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
	#[doc_params("The error to wrap.")]
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
	/// let try_thunk: TryThunk<i32, &str> = TryThunk::err("error");
	/// assert_eq!(try_thunk.evaluate(), Err("error"));
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
	#[doc_type_params(
		"The type of the result of the new computation.",
		"The type of the function to apply."
	)]
	///
	/// ### Parameters
	///
	#[doc_params("The function to apply to the result of the computation.")]
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
	/// let try_thunk: TryThunk<i32, ()> = TryThunk::pure(21).bind(|x| TryThunk::pure(x * 2));
	/// assert_eq!(try_thunk.evaluate(), Ok(42));
	/// ```
	pub fn bind<B: 'a, F>(
		self,
		f: F,
	) -> TryThunk<'a, B, E>
	where
		F: FnOnce(A) -> TryThunk<'a, B, E> + 'a,
	{
		TryThunk::new(move || match (self.0)() {
			Ok(a) => (f(a).0)(),
			Err(e) => Err(e),
		})
	}

	/// Functor map: transforms the result.
	///
	/// ### Type Signature
	///
	/// `forall e b a. (a -> b, TryThunk a e) -> TryThunk b e`
	///
	/// ### Type Parameters
	///
	#[doc_type_params(
		"The type of the result of the transformation.",
		("F", "The type of the transformation function.")
	)]
	///
	/// ### Parameters
	///
	#[doc_params("The function to apply to the result of the computation.")]
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
	/// let try_thunk: TryThunk<i32, ()> = TryThunk::pure(21).map(|x| x * 2);
	/// assert_eq!(try_thunk.evaluate(), Ok(42));
	/// ```
	pub fn map<B: 'a, Func>(
		self,
		func: Func,
	) -> TryThunk<'a, B, E>
	where
		Func: FnOnce(A) -> B + 'a,
	{
		TryThunk::new(move || (self.0)().map(func))
	}

	/// Map error: transforms the error.
	///
	/// ### Type Signature
	///
	/// `forall e2 e a. (e -> e2, TryThunk a e) -> TryThunk a e2`
	///
	/// ### Type Parameters
	///
	#[doc_type_params("The type of the new error.", "The type of the transformation function.")]
	///
	/// ### Parameters
	///
	#[doc_params("The function to apply to the error.")]
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
	/// let try_thunk: TryThunk<i32, i32> = TryThunk::err(21).map_err(|x| x * 2);
	/// assert_eq!(try_thunk.evaluate(), Err(42));
	/// ```
	pub fn map_err<E2: 'a, F>(
		self,
		f: F,
	) -> TryThunk<'a, A, E2>
	where
		F: FnOnce(E) -> E2 + 'a,
	{
		TryThunk::new(move || (self.0)().map_err(f))
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
	/// let try_thunk: TryThunk<i32, ()> = TryThunk::pure(42);
	/// assert_eq!(try_thunk.evaluate(), Ok(42));
	/// ```
	pub fn evaluate(self) -> Result<A, E> {
		(self.0)()
	}
}

impl<'a, A, E, Config> From<Lazy<'a, A, Config>> for TryThunk<'a, A, E>
where
	A: Clone + 'a,
	E: 'a,
	Config: LazyConfig,
{
	fn from(memo: Lazy<'a, A, Config>) -> Self {
		TryThunk::new(move || Ok(memo.evaluate().clone()))
	}
}

impl<'a, A, E, Config> From<TryLazy<'a, A, E, Config>> for TryThunk<'a, A, E>
where
	A: Clone + 'a,
	E: Clone + 'a,
	Config: LazyConfig,
{
	fn from(memo: TryLazy<'a, A, E, Config>) -> Self {
		TryThunk::new(move || memo.evaluate().cloned().map_err(Clone::clone))
	}
}

impl<'a, A: 'a, E: 'a> From<Thunk<'a, A>> for TryThunk<'a, A, E> {
	fn from(eval: Thunk<'a, A>) -> Self {
		TryThunk::new(move || Ok(eval.evaluate()))
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
		let try_thunk: TryThunk<i32, ()> = TryThunk::pure(21).map(|x| x * 2);
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
		let try_thunk: TryThunk<i32, ()> = TryThunk::pure(21).bind(|x| TryThunk::pure(x * 2));
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
		let try_thunk = TryThunk::<i32, &str>::err("error").bind(|x| TryThunk::pure(x * 2));
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
}
