//! Implementation of the `Task` type.
//!
//! This module provides the [`Task`] type, which represents a lazy, stack-safe computation.
//! It is built on the [`Free`] monad and guarantees stack safety for deep recursion and long bind chains.
//!
//! ### Examples
//!
//! ```
//! use fp_library::types::*;
//!
//! let task = Task::later(|| 1 + 1)
//!     .flat_map(|x| Task::later(move || x * 2))
//!     .flat_map(|x| Task::later(move || x + 10));
//!
//! assert_eq!(task.run(), 14);
//! ```

use crate::{
	brands::ThunkFBrand,
	types::{free::Free, step::Step, thunk::Thunk},
};

/// A lazy, stack-safe computation that produces a value of type `A`.
///
/// `Task` is the "heavy-duty" monadic type for deferred computations that
/// require **guaranteed stack safety**. It is built on `Free<ThunkF, A>` with
/// CatList-based bind stack, ensuring O(1) bind operations and unlimited recursion
/// depth without stack overflow.
///
/// # Requirements
///
/// - `A: 'static + Send` — Required due to type erasure via `Box<dyn Any>`
///
/// # Guarantees
///
/// - **Stack safe**: Will not overflow regardless of recursion depth
/// - **O(1) bind**: Left-associated `flat_map` chains don't degrade
/// - **Lazy**: Computation is deferred until `run()` is called
///
/// # When to Use Task vs Eval
///
/// - Use **`Task<A>`** for deep recursion (1000+ levels), heavy monadic pipelines
/// - Use **`Eval<'a, A>`** for HKT integration, borrowed references, glue code
///
/// # Memoization
///
/// `Task` does NOT memoize. Each call to `run()` re-evaluates.
/// For memoization, wrap in `Memo`:
///
/// ```rust
/// use fp_library::types::*;
/// // use fp_library::types::Memo; // Memo not yet implemented
///
/// // let memo: Memo<i32> = Memo::new(|| Task::later(|| 1 + 1).run());
/// // memo.get(); // Computes
/// // memo.get(); // Returns cached
/// ```
///
/// ### Type Parameters
///
/// * `A`: The type of the value produced by the task.
///
/// ### Fields
///
/// * `inner`: The internal `Free` monad representation.
///
/// ### Examples
///
/// ```
/// use fp_library::types::*;
///
/// let task = Task::later(|| 1 + 1)
///     .flat_map(|x| Task::later(move || x * 2))
///     .flat_map(|x| Task::later(move || x + 10));
///
/// assert_eq!(task.run(), 14);
/// ```
pub struct Task<A: 'static> {
	inner: Free<ThunkFBrand, A>,
}

impl<A: 'static + Send> Task<A> {
	/// Creates a `Task` from an already-computed value.
	///
	/// Equivalent to Cats' `Eval.now`.
	///
	/// # Complexity
	/// O(1) creation, O(1) run
	///
	/// ### Type Signature
	///
	/// `forall a. a -> Task a`
	///
	/// ### Type Parameters
	///
	/// * `A`: The type of the value.
	///
	/// ### Parameters
	///
	/// * `a`: The value to wrap.
	///
	/// ### Returns
	///
	/// A `Task` that produces the value `a`.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::types::*;
	///
	/// let task = Task::now(42);
	/// assert_eq!(task.run(), 42);
	/// ```
	#[inline]
	pub fn now(a: A) -> Self {
		Task { inner: Free::pure(a) }
	}

	/// Alias for `now` - PureScript style.
	///
	/// ### Type Signature
	///
	/// `forall a. a -> Task a`
	///
	/// ### Type Parameters
	///
	/// * `A`: The type of the value.
	///
	/// ### Parameters
	///
	/// * `a`: The value to wrap.
	///
	/// ### Returns
	///
	/// A `Task` that produces the value `a`.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::types::*;
	///
	/// let task = Task::pure(42);
	/// assert_eq!(task.run(), 42);
	/// ```
	#[inline]
	pub fn pure(a: A) -> Self {
		Self::now(a)
	}

	/// Creates a lazy `Task` that computes `f` on first `run()`.
	///
	/// This is equivalent to Cats' `Eval.later`, but note that
	/// in our design, `Task` does NOT memoize — each `run()`
	/// re-evaluates. Use `Memo` for caching.
	///
	/// # Complexity
	/// O(1) creation
	///
	/// ### Type Signature
	///
	/// `forall a. (Unit -> a) -> Task a`
	///
	/// ### Type Parameters
	///
	/// * `A`: The type of the value produced by the closure.
	/// * `F`: The type of the closure.
	///
	/// ### Parameters
	///
	/// * `f`: The closure to execute.
	///
	/// ### Returns
	///
	/// A `Task` that executes `f` when run.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::types::*;
	///
	/// let task = Task::later(|| {
	///     // println!("Computing!");
	///     1 + 1
	/// });
	///
	/// // Nothing printed yet
	/// let result = task.run(); // Prints "Computing!"
	/// ```
	#[inline]
	pub fn later<F>(f: F) -> Self
	where
		F: FnOnce() -> A + 'static,
	{
		Task { inner: Free::roll(Thunk::new(move || Free::pure(f()))) }
	}

	/// Alias for `later` - semantically same since we don't memoize.
	///
	/// In Cats, `always` differs from `later` in that it re-evaluates.
	/// Since our `Task` always re-evaluates, this is just an alias.
	///
	/// ### Type Signature
	///
	/// `forall a. (Unit -> a) -> Task a`
	///
	/// ### Type Parameters
	///
	/// * `A`: The type of the value produced by the closure.
	/// * `F`: The type of the closure.
	///
	/// ### Parameters
	///
	/// * `f`: The closure to execute.
	///
	/// ### Returns
	///
	/// A `Task` that executes `f` when run.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::types::*;
	///
	/// let task = Task::always(|| 42);
	/// assert_eq!(task.run(), 42);
	/// ```
	#[inline]
	pub fn always<F>(f: F) -> Self
	where
		F: FnOnce() -> A + 'static,
	{
		Self::later(f)
	}

	/// Defers the construction of a `Task` itself.
	///
	/// This is critical for stack-safe recursion: instead of
	/// building a chain of `Task`s directly (which grows the stack),
	/// we defer the construction.
	///
	/// ### Type Signature
	///
	/// `forall a. (Unit -> Task a) -> Task a`
	///
	/// ### Type Parameters
	///
	/// * `A`: The type of the value produced by the task.
	/// * `F`: The type of the closure.
	///
	/// ### Parameters
	///
	/// * `f`: The closure that produces a `Task`.
	///
	/// ### Returns
	///
	/// A `Task` that defers the creation of the inner task.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::types::*;
	///
	/// fn recursive_sum(n: u64, acc: u64) -> Task<u64> {
	///     if n == 0 {
	///         Task::now(acc)
	///     } else {
	///         // Defer construction to avoid stack growth
	///         Task::defer(move || recursive_sum(n - 1, acc + n))
	///     }
	/// }
	///
	/// // This works for n = 1_000_000 without stack overflow!
	/// let result = recursive_sum(1_000, 0).run();
	/// ```
	#[inline]
	pub fn defer<F>(f: F) -> Self
	where
		F: FnOnce() -> Task<A> + 'static,
	{
		Task { inner: Free::roll(Thunk::new(move || f().inner)) }
	}

	/// Monadic bind (flatMap) with O(1) complexity.
	///
	/// Chains computations together. The key property is that
	/// left-associated chains don't degrade to O(n²).
	///
	/// ### Type Signature
	///
	/// `forall b a. (a -> Task b, Task a) -> Task b`
	///
	/// ### Type Parameters
	///
	/// * `B`: The type of the result of the new task.
	/// * `F`: The type of the binding function.
	///
	/// ### Parameters
	///
	/// * `f`: The function to apply to the result of this task.
	///
	/// ### Returns
	///
	/// A new `Task` that chains `f` after this task.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::types::*;
	///
	/// // This is O(n), not O(n²)
	/// let mut task = Task::now(0);
	/// for i in 0..100 {
	///     task = task.flat_map(move |x| Task::now(x + i));
	/// }
	/// ```
	#[inline]
	pub fn flat_map<B: 'static + Send, F>(
		self,
		f: F,
	) -> Task<B>
	where
		F: FnOnce(A) -> Task<B> + 'static,
	{
		Task { inner: self.inner.flat_map(move |a| f(a).inner) }
	}

	/// Functor map: transforms the result without changing structure.
	///
	/// ### Type Signature
	///
	/// `forall b a. (a -> b, Task a) -> Task b`
	///
	/// ### Type Parameters
	///
	/// * `B`: The type of the result of the mapping function.
	/// * `F`: The type of the mapping function.
	///
	/// ### Parameters
	///
	/// * `f`: The function to apply to the result of this task.
	///
	/// ### Returns
	///
	/// A new `Task` with the transformed result.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::types::*;
	///
	/// let task = Task::now(10).map(|x| x * 2);
	/// assert_eq!(task.run(), 20);
	/// ```
	#[inline]
	pub fn map<B: 'static + Send, F>(
		self,
		f: F,
	) -> Task<B>
	where
		F: FnOnce(A) -> B + 'static,
	{
		self.flat_map(move |a| Task::now(f(a)))
	}

	/// Forces evaluation and returns the result.
	///
	/// This runs the trampoline loop, iteratively processing
	/// the CatList of continuations without growing the stack.
	///
	/// ### Type Signature
	///
	/// `forall a. Task a -> a`
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
	/// let task = Task::later(|| 1 + 1);
	/// assert_eq!(task.run(), 2);
	/// ```
	pub fn run(self) -> A {
		self.inner.run()
	}

	/// Combines two `Task`s, running both and combining results.
	///
	/// ### Type Signature
	///
	/// `forall c b a. (Task b, (a, b) -> c, Task a) -> Task c`
	///
	/// ### Type Parameters
	///
	/// * `B`: The type of the second task's result.
	/// * `C`: The type of the combined result.
	/// * `F`: The type of the combining function.
	///
	/// ### Parameters
	///
	/// * `other`: The second task.
	/// * `f`: The function to combine the results.
	///
	/// ### Returns
	///
	/// A new `Task` producing the combined result.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::types::*;
	///
	/// let t1 = Task::now(10);
	/// let t2 = Task::now(20);
	/// let t3 = t1.map2(t2, |a, b| a + b);
	/// assert_eq!(t3.run(), 30);
	/// ```
	pub fn map2<B: 'static + Send, C: 'static + Send, F>(
		self,
		other: Task<B>,
		f: F,
	) -> Task<C>
	where
		F: FnOnce(A, B) -> C + 'static,
	{
		self.flat_map(move |a| other.map(move |b| f(a, b)))
	}

	/// Sequences two `Task`s, discarding the first result.
	///
	/// ### Type Signature
	///
	/// `forall b a. (Task b, Task a) -> Task b`
	///
	/// ### Type Parameters
	///
	/// * `B`: The type of the second task's result.
	///
	/// ### Parameters
	///
	/// * `other`: The second task.
	///
	/// ### Returns
	///
	/// A new `Task` that runs both tasks and returns the result of the second.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::types::*;
	///
	/// let t1 = Task::now(10);
	/// let t2 = Task::now(20);
	/// let t3 = t1.and_then(t2);
	/// assert_eq!(t3.run(), 20);
	/// ```
	pub fn and_then<B: 'static + Send>(
		self,
		other: Task<B>,
	) -> Task<B> {
		self.flat_map(move |_| other)
	}

	// /// Creates a `Task` from a memoized value (via Memo).
	// ///
	// /// This is a convenience for integrating with the dual-type design.
	// /// The Memo provides caching; Task provides computation structure.
	// ///
	// /// ### Type Signature
	// ///
	// /// `forall a. Memo a -> Task a`
	// ///
	// /// ### Type Parameters
	// ///
	// /// * `A`: The type of the value.
	// ///
	// /// ### Parameters
	// ///
	// /// * `memo`: The memoized value.
	// ///
	// /// ### Returns
	// ///
	// /// A `Task` that retrieves the memoized value.
	// ///
	// /// ### Examples
	// ///
	// /// ```
	// /// use fp_library::types::{Memo, Task};
	// ///
	// /// let memo = Memo::new(|| 42);
	// /// let task = Task::from_memo(&memo);
	// /// assert_eq!(task.run(), 42);
	// /// ```
	// pub fn from_memo(memo: &crate::types::memo::Memo<A>) -> Self
	// where
	// 	A: Clone,
	// {
	// 	let value = memo.get().clone();
	// 	Task::now(value)
	// }

	/// Stack-safe tail recursion within Task.
	///
	/// # Clone Bound
	///
	/// The function `f` must implement `Clone` because each iteration
	/// of the recursion may need its own copy. Most closures naturally
	/// implement `Clone` when all their captures implement `Clone`.
	///
	/// For closures that don't implement `Clone`, use `tail_rec_m_shared`
	/// which wraps the closure in `Arc` internally.
	///
	/// ### Type Signature
	///
	/// `forall s a. (s -> Task (Step s a), s) -> Task a`
	///
	/// ### Type Parameters
	///
	/// * `S`: The type of the state.
	/// * `F`: The type of the step function.
	///
	/// ### Parameters
	///
	/// * `f`: The function that performs one step of the recursion.
	/// * `initial`: The initial state.
	///
	/// ### Returns
	///
	/// A `Task` that performs the recursion.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::types::{Task, Step};
	///
	/// // Fibonacci using tail recursion
	/// fn fib(n: u64) -> Task<u64> {
	///     Task::tail_rec_m(|(n, a, b)| {
	///         if n == 0 {
	///             Task::now(Step::Done(a))
	///         } else {
	///             Task::now(Step::Loop((n - 1, b, a + b)))
	///         }
	///     }, (n, 0u64, 1u64))
	/// }
	///
	/// assert_eq!(fib(50).run(), 12586269025);
	/// ```
	pub fn tail_rec_m<S: 'static + Send, F>(
		f: F,
		initial: S,
	) -> Self
	where
		F: Fn(S) -> Task<Step<S, A>> + Clone + 'static,
	{
		// Use defer to ensure each step is trampolined.
		fn go<A: 'static + Send, B: 'static + Send, F>(
			f: F,
			a: A,
		) -> Task<B>
		where
			F: Fn(A) -> Task<Step<A, B>> + Clone + 'static,
		{
			let f_clone = f.clone();
			Task::defer(move || {
				f(a).flat_map(move |step| match step {
					Step::Loop(next) => go(f_clone.clone(), next),
					Step::Done(b) => Task::now(b),
				})
			})
		}

		go(f, initial)
	}

	/// Arc-wrapped version for non-Clone closures.
	///
	/// Use this when your closure captures non-Clone state.
	///
	/// ### Type Signature
	///
	/// `forall s a. (s -> Task (Step s a), s) -> Task a`
	///
	/// ### Type Parameters
	///
	/// * `S`: The type of the state.
	/// * `F`: The type of the step function.
	///
	/// ### Parameters
	///
	/// * `f`: The function that performs one step of the recursion.
	/// * `initial`: The initial state.
	///
	/// ### Returns
	///
	/// A `Task` that performs the recursion.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::types::{Task, Step};
	/// use std::sync::{Arc, atomic::{AtomicUsize, Ordering}};
	///
	/// // Closure captures non-Clone state
	/// let counter = Arc::new(AtomicUsize::new(0));
	/// Task::tail_rec_m_shared(move |n| {
	///     counter.fetch_add(1, Ordering::SeqCst);
	///     if n == 0 {
	///         Task::now(Step::Done(0))
	///     } else {
	///         Task::now(Step::Loop(n - 1))
	///     }
	/// }, 100);
	/// ```
	pub fn tail_rec_m_shared<S: 'static + Send, F>(
		f: F,
		initial: S,
	) -> Self
	where
		F: Fn(S) -> Task<Step<S, A>> + 'static,
	{
		use std::sync::Arc;
		let f = Arc::new(f);
		let wrapper = move |s: S| {
			let f = Arc::clone(&f);
			f(s)
		};
		Self::tail_rec_m(wrapper, initial)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::types::step::Step;

	/// Tests `Task::now`.
	///
	/// Verifies that `now` creates a task that returns the value immediately.
	#[test]
	fn test_task_now() {
		let task = Task::now(42);
		assert_eq!(task.run(), 42);
	}

	/// Tests `Task::later`.
	///
	/// Verifies that `later` creates a task that computes the value when run.
	#[test]
	fn test_task_later() {
		let task = Task::later(|| 42);
		assert_eq!(task.run(), 42);
	}

	/// Tests `Task::flat_map`.
	///
	/// Verifies that `flat_map` chains computations correctly.
	#[test]
	fn test_task_flat_map() {
		let task = Task::now(10).flat_map(|x| Task::now(x * 2));
		assert_eq!(task.run(), 20);
	}

	/// Tests `Task::map`.
	///
	/// Verifies that `map` transforms the result.
	#[test]
	fn test_task_map() {
		let task = Task::now(10).map(|x| x * 2);
		assert_eq!(task.run(), 20);
	}

	/// Tests `Task::defer`.
	///
	/// Verifies that `defer` delays the creation of the task.
	#[test]
	fn test_task_defer() {
		let task = Task::defer(|| Task::now(42));
		assert_eq!(task.run(), 42);
	}

	/// Tests `Task::tail_rec_m`.
	///
	/// Verifies that `tail_rec_m` performs tail recursion correctly.
	#[test]
	fn test_task_tail_rec_m() {
		fn factorial(n: u64) -> Task<u64> {
			Task::tail_rec_m(
				|(n, acc)| {
					if n <= 1 {
						Task::now(Step::Done(acc))
					} else {
						Task::now(Step::Loop((n - 1, n * acc)))
					}
				},
				(n, 1u64),
			)
		}

		assert_eq!(factorial(5).run(), 120);
	}

	/// Tests `Task::map2`.
	///
	/// Verifies that `map2` combines two tasks.
	#[test]
	fn test_task_map2() {
		let t1 = Task::now(10);
		let t2 = Task::now(20);
		let t3 = t1.map2(t2, |a, b| a + b);
		assert_eq!(t3.run(), 30);
	}

	/// Tests `Task::and_then`.
	///
	/// Verifies that `and_then` sequences two tasks.
	#[test]
	fn test_task_and_then() {
		let t1 = Task::now(10);
		let t2 = Task::now(20);
		let t3 = t1.and_then(t2);
		assert_eq!(t3.run(), 20);
	}

	/// Tests `Task::tail_rec_m_shared`.
	///
	/// Verifies that `tail_rec_m_shared` works with non-Clone closures.
	#[test]
	fn test_task_tail_rec_m_shared() {
		use std::sync::{
			Arc,
			atomic::{AtomicUsize, Ordering},
		};

		let counter = Arc::new(AtomicUsize::new(0));
		let counter_clone = Arc::clone(&counter);

		let task = Task::tail_rec_m_shared(
			move |n| {
				counter_clone.fetch_add(1, Ordering::SeqCst);
				if n == 0 { Task::now(Step::Done(0)) } else { Task::now(Step::Loop(n - 1)) }
			},
			10,
		);

		assert_eq!(task.run(), 0);
		assert_eq!(counter.load(Ordering::SeqCst), 11);
	}
}
