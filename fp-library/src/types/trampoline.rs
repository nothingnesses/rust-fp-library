//! Stack-safe computation type with guaranteed safety for unlimited recursion depth.
//!
//! Built on the [`Free`] monad with O(1) [`bind`](crate::functions::bind) operations. Provides complete stack safety at the cost of requiring `'static` types. Use this for deep recursion and heavy monadic pipelines.
//!
//! ### Examples
//!
//! ```
//! use fp_library::types::*;
//!
//! let task = Trampoline::new(|| 1 + 1)
//!     .bind(|x| Trampoline::new(move || x * 2))
//!     .bind(|x| Trampoline::new(move || x + 10));
//!
//! assert_eq!(task.evaluate(), 14);
//! ```

use crate::{
	brands::ThunkBrand,
	classes::Deferrable,
	types::{Free, Lazy, LazyConfig, Step, Thunk},
};
use fp_macros::{doc_params, doc_type_params, hm_signature};

/// A lazy, stack-safe computation that produces a value of type `A`.
///
/// `Trampoline` is the "heavy-duty" monadic type for deferred computations that
/// require **guaranteed stack safety**. It is built on [`Free<Thunk, A>`] with
/// [`CatList`](crate::types::CatList)-based bind stack, ensuring O(1) [`bind`](crate::functions::bind)
/// operations and unlimited recursion depth without stack overflow.
///
/// # Requirements
///
/// - `A: 'static + Send` — Required due to type erasure via [`Box<dyn Any>`].
///
/// # Guarantees
///
/// - **Stack safe**: Will not overflow regardless of recursion depth.
/// - **O(1) bind**: Left-associated `bind` chains don't degrade.
/// - **Lazy**: Computation is deferred until [`Trampoline::evaluate`] is called.
///
/// # When to Use `Trampoline` vs [`Thunk`]
///
/// - Use **`Trampoline<A>`** for deep recursion, heavy monadic pipelines.
/// - Use **`Thunk<'a, A>`** for HKT integration, borrowed references, glue code.
///
/// # Memoization
///
/// `Trampoline` does NOT memoize. Each call to `run` re-evaluates.
/// For memoization, wrap in [`Lazy`]:
///
/// ```rust
/// use fp_library::types::*;
///
/// let lazy: Lazy<i32> = Lazy::<_, RcLazyConfig>::new(|| Trampoline::new(|| 1 + 1).evaluate());
/// lazy.evaluate(); // Computes
/// lazy.evaluate(); // Returns cached
/// ```
///
/// ### Type Parameters
///
/// * `A`: The type of the value produced by the task.
///
/// ### Fields
///
/// * `0`: The internal `Free` monad representation.
///
/// ### Examples
///
/// ```
/// use fp_library::types::*;
///
/// let task = Trampoline::new(|| 1 + 1)
///     .bind(|x| Trampoline::new(move || x * 2))
///     .bind(|x| Trampoline::new(move || x + 10));
///
/// assert_eq!(task.evaluate(), 14);
/// ```
pub struct Trampoline<A: 'static>(Free<ThunkBrand, A>);

impl<A: 'static + Send> Trampoline<A> {
	/// Creates a `Trampoline` from an already-computed value.
	///
	/// ### Complexity
	///
	/// O(1) creation, O(1) evaluation
	///
	/// ### Type Signature
	///
	#[hm_signature]
	///
	/// ### Type Parameters
	///
	/// * `A`: The type of the value.
	///
	/// ### Parameters
	///
	#[doc_params("The value to wrap.")]
	///
	/// ### Returns
	///
	/// A `Trampoline` that produces the value `a`.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::types::*;
	///
	/// let task = Trampoline::pure(42);
	/// assert_eq!(task.evaluate(), 42);
	/// ```
	#[inline]
	pub fn pure(a: A) -> Self {
		Trampoline(Free::pure(a))
	}

	/// Creates a lazy `Trampoline` that computes `f` on first evaluation.
	///
	/// `Trampoline` does NOT memoize — each `evaluate()`
	/// re-evaluates. Use [`Lazy`] for caching.
	///
	/// # Complexity
	/// O(1) creation
	///
	/// ### Type Signature
	///
	#[hm_signature]
	///
	/// ### Type Parameters
	///
	#[doc_type_params("The type of the closure.")]
	///
	/// ### Parameters
	///
	#[doc_params("The closure to execute.")]
	///
	/// ### Returns
	///
	/// A `Trampoline` that executes `f` when run.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::types::*;
	///
	/// let task = Trampoline::new(|| {
	///     // println!("Computing!");
	///     1 + 1
	/// });
	///
	/// // Nothing printed yet
	/// let result = task.evaluate(); // Prints "Computing!"
	/// ```
	#[inline]
	pub fn new<F>(f: F) -> Self
	where
		F: FnOnce() -> A + 'static,
	{
		Trampoline(Free::wrap(Thunk::new(move || Free::pure(f()))))
	}

	/// Defers the construction of a `Trampoline` itself.
	///
	/// This is critical for stack-safe recursion: instead of
	/// building a chain of `Trampoline`s directly (which grows the stack),
	/// we defer the construction.
	///
	/// ### Type Signature
	///
	#[hm_signature]
	///
	/// ### Type Parameters
	///
	#[doc_type_params("The type of the closure.")]
	///
	/// ### Parameters
	///
	#[doc_params("The closure that produces a `Trampoline`.")]
	///
	/// ### Returns
	///
	/// A `Trampoline` that defers the creation of the inner task.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::types::*;
	///
	/// fn recursive_sum(n: u64, acc: u64) -> Trampoline<u64> {
	///     if n == 0 {
	///         Trampoline::pure(acc)
	///     } else {
	///         // Defer construction to avoid stack growth
	///         Trampoline::defer(move || recursive_sum(n - 1, acc + n))
	///     }
	/// }
	///
	/// // This works for n = 1_000_000 without stack overflow!
	/// let result = recursive_sum(1_000, 0).evaluate();
	/// ```
	#[inline]
	pub fn defer<F>(f: F) -> Self
	where
		F: FnOnce() -> Trampoline<A> + 'static,
	{
		Trampoline(Free::wrap(Thunk::new(move || f().0)))
	}

	/// Monadic bind with O(1) complexity.
	///
	/// Chains computations together. The key property is that
	/// left-associated chains don't degrade to O(n²).
	///
	/// ### Type Signature
	///
	#[hm_signature]
	///
	/// ### Type Parameters
	///
	#[doc_type_params(
		"The type of the result of the new task.",
		"The type of the binding function."
	)]
	///
	/// ### Parameters
	///
	#[doc_params("The function to apply to the result of this task.")]
	///
	/// ### Returns
	///
	/// A new `Trampoline` that chains `f` after this task.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::types::*;
	///
	/// // This is O(n), not O(n²)
	/// let mut task = Trampoline::pure(0);
	/// for i in 0..100 {
	///     task = task.bind(move |x| Trampoline::pure(x + i));
	/// }
	/// ```
	#[inline]
	pub fn bind<B: 'static + Send, F>(
		self,
		f: F,
	) -> Trampoline<B>
	where
		F: FnOnce(A) -> Trampoline<B> + 'static,
	{
		Trampoline(self.0.bind(move |a| f(a).0))
	}

	/// Functor map: transforms the result without changing structure.
	///
	/// ### Type Signature
	///
	#[hm_signature]
	///
	/// ### Type Parameters
	///
	#[doc_type_params(
		"The type of the result of the mapping function.",
		"The type of the mapping function."
	)]
	///
	/// ### Parameters
	///
	#[doc_params("The function to apply to the result of this task.")]
	///
	/// ### Returns
	///
	/// A new `Trampoline` with the transformed result.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::types::*;
	///
	/// let task = Trampoline::pure(10).map(|x| x * 2);
	/// assert_eq!(task.evaluate(), 20);
	/// ```
	#[inline]
	pub fn map<B: 'static + Send, F>(
		self,
		f: F,
	) -> Trampoline<B>
	where
		F: FnOnce(A) -> B + 'static,
	{
		self.bind(move |a| Trampoline::pure(f(a)))
	}

	/// Forces evaluation and returns the result.
	///
	/// This runs the trampoline loop, iteratively processing
	/// the CatList of continuations without growing the stack.
	///
	/// ### Type Signature
	///
	#[hm_signature]
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
	/// let task = Trampoline::new(|| 1 + 1);
	/// assert_eq!(task.evaluate(), 2);
	/// ```
	pub fn evaluate(self) -> A {
		self.0.evaluate()
	}

	/// Combines two `Trampoline`s, running both and combining results.
	///
	/// ### Type Signature
	///
	#[hm_signature]
	///
	/// ### Type Parameters
	///
	#[doc_type_params(
		"The type of the second task's result.",
		"The type of the combined result.",
		"The type of the combining function."
	)]
	///
	/// ### Parameters
	///
	#[doc_params("The second task.", "The function to combine the results.")]
	///
	/// ### Returns
	///
	/// A new `Trampoline` producing the combined result.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::types::*;
	///
	/// let t1 = Trampoline::pure(10);
	/// let t2 = Trampoline::pure(20);
	/// let t3 = t1.lift2(t2, |a, b| a + b);
	/// assert_eq!(t3.evaluate(), 30);
	/// ```
	pub fn lift2<B: 'static + Send, C: 'static + Send, F>(
		self,
		other: Trampoline<B>,
		f: F,
	) -> Trampoline<C>
	where
		F: FnOnce(A, B) -> C + 'static,
	{
		self.bind(move |a| other.map(move |b| f(a, b)))
	}

	/// Sequences two `Trampoline`s, discarding the first result.
	///
	/// ### Type Signature
	///
	#[hm_signature]
	///
	/// ### Type Parameters
	///
	#[doc_type_params("The type of the second task's result.")]
	///
	/// ### Parameters
	///
	#[doc_params("The second task.")]
	///
	/// ### Returns
	///
	/// A new `Trampoline` that runs both tasks and returns the result of the second.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::types::*;
	///
	/// let t1 = Trampoline::pure(10);
	/// let t2 = Trampoline::pure(20);
	/// let t3 = t1.then(t2);
	/// assert_eq!(t3.evaluate(), 20);
	/// ```
	pub fn then<B: 'static + Send>(
		self,
		other: Trampoline<B>,
	) -> Trampoline<B> {
		self.bind(move |_| other)
	}

	/// Stack-safe tail recursion within Trampoline.
	///
	/// # Clone Bound
	///
	/// The function `f` must implement `Clone` because each iteration
	/// of the recursion may need its own copy. Most closures naturally
	/// implement `Clone` when all their captures implement `Clone`.
	///
	/// For closures that don't implement `Clone`, use `arc_tail_rec_m`
	/// which wraps the closure in `Arc` internally.
	///
	/// ### Type Signature
	///
	#[hm_signature]
	///
	/// ### Type Parameters
	///
	#[doc_type_params("The type of the state.", "The type of the step function.")]
	///
	/// ### Parameters
	///
	#[doc_params("The function that performs one step of the recursion.", "The initial state.")]
	///
	/// ### Returns
	///
	/// A `Trampoline` that performs the recursion.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::types::{Trampoline, Step};
	///
	/// // Fibonacci using tail recursion
	/// fn fib(n: u64) -> Trampoline<u64> {
	///     Trampoline::tail_rec_m(|(n, a, b)| {
	///         if n == 0 {
	///             Trampoline::pure(Step::Done(a))
	///         } else {
	///             Trampoline::pure(Step::Loop((n - 1, b, a + b)))
	///         }
	///     }, (n, 0u64, 1u64))
	/// }
	///
	/// assert_eq!(fib(50).evaluate(), 12586269025);
	/// ```
	pub fn tail_rec_m<S: 'static + Send, F>(
		f: F,
		initial: S,
	) -> Self
	where
		F: Fn(S) -> Trampoline<Step<S, A>> + Clone + 'static,
	{
		// Use defer to ensure each step is trampolined.
		fn go<A: 'static + Send, B: 'static + Send, F>(
			f: F,
			a: A,
		) -> Trampoline<B>
		where
			F: Fn(A) -> Trampoline<Step<A, B>> + Clone + 'static,
		{
			let f_clone = f.clone();
			Trampoline::defer(move || {
				f(a).bind(move |step| match step {
					Step::Loop(next) => go(f_clone.clone(), next),
					Step::Done(b) => Trampoline::pure(b),
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
	#[hm_signature]
	///
	/// ### Type Parameters
	///
	#[doc_type_params("The type of the state.", "The type of the step function.")]
	///
	/// ### Parameters
	///
	#[doc_params("The function that performs one step of the recursion.", "The initial state.")]
	///
	/// ### Returns
	///
	/// A `Trampoline` that performs the recursion.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::types::{Trampoline, Step};
	/// use std::sync::{Arc, atomic::{AtomicUsize, Ordering}};
	///
	/// // Closure captures non-Clone state
	/// let counter = Arc::new(AtomicUsize::new(0));
	/// Trampoline::arc_tail_rec_m(move |n| {
	///     counter.fetch_add(1, Ordering::SeqCst);
	///     if n == 0 {
	///         Trampoline::pure(Step::Done(0))
	///     } else {
	///         Trampoline::pure(Step::Loop(n - 1))
	///     }
	/// }, 100);
	/// ```
	pub fn arc_tail_rec_m<S: 'static + Send, F>(
		f: F,
		initial: S,
	) -> Self
	where
		F: Fn(S) -> Trampoline<Step<S, A>> + 'static,
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

impl<A: 'static + Send + Clone, Config: LazyConfig> From<Lazy<'static, A, Config>>
	for Trampoline<A>
{
	fn from(lazy: Lazy<'static, A, Config>) -> Self {
		Trampoline::new(move || lazy.evaluate().clone())
	}
}

impl<A: 'static + Send> Deferrable<'static> for Trampoline<A> {
	/// Creates a `Trampoline` from a computation that produces it.
	///
	/// ### Type Signature
	///
	#[hm_signature(Deferrable)]
	///
	/// ### Type Parameters
	///
	#[doc_type_params("The type of the thunk.")]
	///
	/// ### Parameters
	///
	#[doc_params("A thunk that produces the trampoline.")]
	///
	/// ### Returns
	///
	/// The deferred trampoline.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, functions::*, types::*, classes::Deferrable};
	///
	/// let task: Trampoline<i32> = Deferrable::defer(|| Trampoline::pure(42));
	/// assert_eq!(task.evaluate(), 42);
	/// ```
	fn defer<F>(f: F) -> Self
	where
		F: FnOnce() -> Self + 'static,
		Self: Sized,
	{
		Trampoline::defer(f)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::types::step::Step;

	/// Tests `Trampoline::pure`.
	///
	/// Verifies that `pure` creates a task that returns the value immediately.
	#[test]
	fn test_task_pure() {
		let task = Trampoline::pure(42);
		assert_eq!(task.evaluate(), 42);
	}

	/// Tests `Trampoline::new`.
	///
	/// Verifies that `new` creates a task that computes the value when run.
	#[test]
	fn test_task_new() {
		let task = Trampoline::new(|| 42);
		assert_eq!(task.evaluate(), 42);
	}

	/// Tests `Trampoline::bind`.
	///
	/// Verifies that `bind` chains computations correctly.
	#[test]
	fn test_task_bind() {
		let task = Trampoline::pure(10).bind(|x| Trampoline::pure(x * 2));
		assert_eq!(task.evaluate(), 20);
	}

	/// Tests `Trampoline::map`.
	///
	/// Verifies that `map` transforms the result.
	#[test]
	fn test_task_map() {
		let task = Trampoline::pure(10).map(|x| x * 2);
		assert_eq!(task.evaluate(), 20);
	}

	/// Tests `Trampoline::defer`.
	///
	/// Verifies that `defer` delays the creation of the task.
	#[test]
	fn test_task_defer() {
		let task = Trampoline::defer(|| Trampoline::pure(42));
		assert_eq!(task.evaluate(), 42);
	}

	/// Tests `Trampoline::tail_rec_m`.
	///
	/// Verifies that `tail_rec_m` performs tail recursion correctly.
	#[test]
	fn test_task_tail_rec_m() {
		fn factorial(n: u64) -> Trampoline<u64> {
			Trampoline::tail_rec_m(
				|(n, acc)| {
					if n <= 1 {
						Trampoline::pure(Step::Done(acc))
					} else {
						Trampoline::pure(Step::Loop((n - 1, n * acc)))
					}
				},
				(n, 1u64),
			)
		}

		assert_eq!(factorial(5).evaluate(), 120);
	}

	/// Tests `Trampoline::map2`.
	///
	/// Verifies that `map2` combines two tasks.
	#[test]
	fn test_task_map2() {
		let t1 = Trampoline::pure(10);
		let t2 = Trampoline::pure(20);
		let t3 = t1.lift2(t2, |a, b| a + b);
		assert_eq!(t3.evaluate(), 30);
	}

	/// Tests `Trampoline::and_then`.
	///
	/// Verifies that `and_then` sequences two tasks.
	#[test]
	fn test_task_and_then() {
		let t1 = Trampoline::pure(10);
		let t2 = Trampoline::pure(20);
		let t3 = t1.then(t2);
		assert_eq!(t3.evaluate(), 20);
	}

	/// Tests `Trampoline::arc_tail_rec_m`.
	///
	/// Verifies that `arc_tail_rec_m` works with non-Clone closures.
	#[test]
	fn test_task_arc_tail_rec_m() {
		use std::sync::{
			Arc,
			atomic::{AtomicUsize, Ordering},
		};

		let counter = Arc::new(AtomicUsize::new(0));
		let counter_clone = Arc::clone(&counter);

		let task = Trampoline::arc_tail_rec_m(
			move |n| {
				counter_clone.fetch_add(1, Ordering::SeqCst);
				if n == 0 {
					Trampoline::pure(Step::Done(0))
				} else {
					Trampoline::pure(Step::Loop(n - 1))
				}
			},
			10,
		);

		assert_eq!(task.evaluate(), 0);
		assert_eq!(counter.load(Ordering::SeqCst), 11);
	}

	/// Tests `Trampoline::from_memo`.
	///
	/// Verifies that `From<Lazy>` creates a task that retrieves the memoized value lazily.
	#[test]
	fn test_task_from_memo() {
		use crate::types::{Lazy, RcLazyConfig};
		use std::cell::RefCell;
		use std::rc::Rc;

		let counter = Rc::new(RefCell::new(0));
		let counter_clone = counter.clone();
		let memo = Lazy::<_, RcLazyConfig>::new(move || {
			*counter_clone.borrow_mut() += 1;
			42
		});

		let task = Trampoline::from(memo.clone());

		// Should not have computed yet (lazy creation)
		assert_eq!(*counter.borrow(), 0);

		assert_eq!(task.evaluate(), 42);
		assert_eq!(*counter.borrow(), 1);

		// Run again, should use cached value
		let task2 = Trampoline::from(memo);
		assert_eq!(task2.evaluate(), 42);
		assert_eq!(*counter.borrow(), 1);
	}

	/// Tests `Trampoline::from` with `ArcLazy`.
	#[test]
	fn test_task_from_arc_memo() {
		use crate::types::{ArcLazyConfig, Lazy};
		use std::sync::{Arc, Mutex};

		let counter = Arc::new(Mutex::new(0));
		let counter_clone = counter.clone();
		let memo = Lazy::<_, ArcLazyConfig>::new(move || {
			*counter_clone.lock().unwrap() += 1;
			42
		});

		let task = Trampoline::from(memo.clone());

		// Should not have computed yet (lazy creation)
		assert_eq!(*counter.lock().unwrap(), 0);

		assert_eq!(task.evaluate(), 42);
		assert_eq!(*counter.lock().unwrap(), 1);

		// Run again, should use cached value
		let task2 = Trampoline::from(memo);
		assert_eq!(task2.evaluate(), 42);
		assert_eq!(*counter.lock().unwrap(), 1);
	}
}
