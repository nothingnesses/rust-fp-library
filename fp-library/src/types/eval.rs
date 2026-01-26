//! Implementations for [`Eval`], a deferred computation type.
//!
//! This module provides the [`Eval`] type, which represents a deferred computation that produces a value.
//! Unlike [`Task`](crate::types::Task), `Eval` is not stack-safe for deep recursion but supports higher-kinded types and borrowing.

use crate::types::TryEval;

/// A deferred computation that produces a value of type `A`.
///
/// `Eval` is NOT memoized - each call to `run()` re-executes the computation.
/// This type exists to build computation chains without allocation overhead.
///
/// Unlike [`Task`](crate::types::Task), `Eval` does NOT require `'static` and CAN implement
/// HKT traits like [`Functor`](crate::classes::Functor), [`Semimonad`](crate::classes::Semimonad), etc.
///
/// ### Trade-offs vs Task
///
/// | Aspect         | Eval<'a, A>               | Task<A>                    |
/// |----------------|---------------------------|----------------------------|
/// | HKT compatible | ✅ Yes                    | ❌ No (requires `'static`) |
/// | Stack-safe     | ❌ No (~8000 calls limit) | ✅ Yes (unlimited)         |
/// | Lifetime       | `'a` (can borrow)         | `'static` only             |
/// | Use case       | Glue code, composition    | Deep recursion, pipelines  |
///
/// ### Algebraic Properties
///
/// `Eval` is a proper Monad:
/// - `pure(a).run() == a` (left identity)
/// - `eval.bind(pure) == eval` (right identity)
/// - `eval.bind(f).bind(g) == eval.bind(|a| f(a).bind(g))` (associativity)
///
/// ### Type Parameters
///
/// * `A`: The type of the value produced by the computation.
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
/// let computation = Eval::new(|| 5)
///     .map(|x| x * 2)
///     .map(|x| x + 1);
///
/// // No computation has happened yet!
/// // Only when we call run() does it execute:
/// let result = computation.run();
/// assert_eq!(result, 11);
/// ```
pub struct Eval<'a, A> {
	thunk: Box<dyn FnOnce() -> A + 'a>,
}

impl<'a, A: 'a> Eval<'a, A> {
	/// Creates a new Eval from a thunk.
	///
	/// ### Type Signature
	///
	/// `forall a. (Unit -> a) -> Eval a`
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
	/// A new `Eval` instance.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::types::*;
	///
	/// let eval = Eval::new(|| 42);
	/// assert_eq!(eval.run(), 42);
	/// ```
	pub fn new<F>(f: F) -> Self
	where
		F: FnOnce() -> A + 'a,
	{
		Eval { thunk: Box::new(f) }
	}

	/// Returns a pure value (already computed).
	///
	/// ### Type Signature
	///
	/// `forall a. a -> Eval a`
	///
	/// ### Parameters
	///
	/// * `a`: The value to wrap.
	///
	/// ### Returns
	///
	/// A new `Eval` instance containing the value.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::types::*;
	///
	/// let eval = Eval::pure(42);
	/// assert_eq!(eval.run(), 42);
	/// ```
	pub fn pure(a: A) -> Self
	where
		A: 'a,
	{
		Eval::new(move || a)
	}

	/// Defers a computation that returns an Eval.
	///
	/// ### Type Signature
	///
	/// `forall a. (Unit -> Eval a) -> Eval a`
	///
	/// ### Type Parameters
	///
	/// * `F`: The type of the thunk.
	///
	/// ### Parameters
	///
	/// * `f`: The thunk that returns an `Eval`.
	///
	/// ### Returns
	///
	/// A new `Eval` instance.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::types::*;
	///
	/// let eval = Eval::defer(|| Eval::pure(42));
	/// assert_eq!(eval.run(), 42);
	/// ```
	pub fn defer<F>(f: F) -> Self
	where
		F: FnOnce() -> Eval<'a, A> + 'a,
	{
		Eval::new(move || f().run())
	}

	/// Monadic bind: chains computations.
	///
	/// Note: Each `flat_map` adds to the call stack. For deep recursion
	/// (>1000 levels), use [`Task`](crate::types::Task) instead.
	///
	/// ### Type Signature
	///
	/// `forall b a. (a -> Eval b, Eval a) -> Eval b`
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
	/// A new `Eval` instance representing the chained computation.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::types::*;
	///
	/// let eval = Eval::pure(21).flat_map(|x| Eval::pure(x * 2));
	/// assert_eq!(eval.run(), 42);
	/// ```
	pub fn flat_map<B: 'a, F>(
		self,
		f: F,
	) -> Eval<'a, B>
	where
		F: FnOnce(A) -> Eval<'a, B> + 'a,
	{
		Eval::new(move || {
			let a = (self.thunk)();
			let eval_b = f(a);
			(eval_b.thunk)()
		})
	}

	/// Functor map: transforms the result.
	///
	/// ### Type Signature
	///
	/// `forall b a. (a -> b, Eval a) -> Eval b`
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
	/// A new `Eval` instance with the transformed result.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::types::*;
	///
	/// let eval = Eval::pure(21).map(|x| x * 2);
	/// assert_eq!(eval.run(), 42);
	/// ```
	pub fn map<B: 'a, F>(
		self,
		f: F,
	) -> Eval<'a, B>
	where
		F: FnOnce(A) -> B + 'a,
	{
		Eval::new(move || f((self.thunk)()))
	}

	/// Forces evaluation and returns the result.
	///
	/// ### Type Signature
	///
	/// `forall a. Eval a -> a`
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
	/// let eval = Eval::pure(42);
	/// assert_eq!(eval.run(), 42);
	/// ```
	pub fn run(self) -> A {
		(self.thunk)()
	}

	/// Converts to a TryEval that always succeeds.
	///
	/// ### Type Signature
	///
	/// `forall e a. Eval a -> TryEval a e`
	///
	/// ### Type Parameters
	///
	/// * `E`: The error type of the resulting `TryEval`.
	///
	/// ### Returns
	///
	/// A `TryEval` that produces `Ok(value)`.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::types::*;
	///
	/// let eval = Eval::pure(42);
	/// let try_eval: TryEval<i32, ()> = eval.into_try();
	/// assert_eq!(try_eval.run(), Ok(42));
	/// ```
	pub fn into_try<E: 'a>(self) -> TryEval<'a, A, E> {
		TryEval::new(move || Ok(self.run()))
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	/// Tests basic execution of Eval.
	///
	/// Verifies that `Eval::new` creates a computation that can be run to produce the expected value.
	#[test]
	fn test_basic_execution() {
		let eval = Eval::new(|| 42);
		assert_eq!(eval.run(), 42);
	}

	/// Tests `Eval::pure`.
	///
	/// Verifies that `Eval::pure` creates a computation that returns the provided value.
	#[test]
	fn test_pure() {
		let eval = Eval::pure(42);
		assert_eq!(eval.run(), 42);
	}

	/// Tests borrowing in Eval.
	///
	/// Verifies that `Eval` can capture references to values on the stack.
	#[test]
	fn test_borrowing() {
		let x = 42;
		let eval = Eval::new(|| &x);
		assert_eq!(eval.run(), &42);
	}

	/// Tests `Eval::map`.
	///
	/// Verifies that `map` transforms the result of the computation.
	#[test]
	fn test_map() {
		let eval = Eval::pure(21).map(|x| x * 2);
		assert_eq!(eval.run(), 42);
	}

	/// Tests `Eval::flat_map`.
	///
	/// Verifies that `flat_map` chains computations correctly.
	#[test]
	fn test_flat_map() {
		let eval = Eval::pure(21).flat_map(|x| Eval::pure(x * 2));
		assert_eq!(eval.run(), 42);
	}

	/// Tests `Eval::defer`.
	///
	/// Verifies that `defer` allows creating an `Eval` from a thunk that returns an `Eval`.
	#[test]
	fn test_defer() {
		let eval = Eval::defer(|| Eval::pure(42));
		assert_eq!(eval.run(), 42);
	}

	/// Tests `Eval::into_try`.
	///
	/// Verifies that `into_try` converts an `Eval` into a `TryEval` that succeeds.
	#[test]
	fn test_into_try() {
		let eval = Eval::pure(42);
		let try_eval: TryEval<i32, ()> = eval.into_try();
		assert_eq!(try_eval.run(), Ok(42));
	}
}
