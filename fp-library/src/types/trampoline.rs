//! Stack-safe computation type with guaranteed safety for unlimited recursion depth.
//!
//! Built on the [`Free`](crate::types::Free) monad with O(1) [`bind`](crate::functions::bind) operations. Provides complete stack safety at the cost of requiring `'static` types. Use this for deep recursion and heavy monadic pipelines.
//!
//! ### Examples
//!
//! ```
//! use fp_library::types::*;
//!
//! let task = Trampoline::new(|| 1 + 1)
//! 	.bind(|x| Trampoline::new(move || x * 2))
//! 	.bind(|x| Trampoline::new(move || x + 10));
//!
//! assert_eq!(task.evaluate(), 14);
//! ```

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			brands::ThunkBrand,
			classes::Deferrable,
			types::{
				Free,
				Lazy,
				LazyConfig,
				Step,
				Thunk,
			},
		},
		fp_macros::*,
	};

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
	#[document_type_parameters("The type of the value produced by the task.")]
	///
	#[document_fields("The internal `Free` monad representation.")]
	///
	pub struct Trampoline<A: 'static>(Free<ThunkBrand, A>);

	#[document_type_parameters("The type of the value produced by the task.")]
	#[document_parameters("The `Trampoline` instance.")]
	impl<A: 'static + Send> Trampoline<A> {
		/// Creates a `Trampoline` from an already-computed value.
		///
		/// ### Complexity
		///
		/// O(1) creation, O(1) evaluation
		#[document_signature]
		///
		#[document_parameters("The value to wrap.")]
		///
		#[document_returns("A `Trampoline` that produces the value `a`.")]
		///
		#[inline]
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let task = Trampoline::pure(42);
		/// assert_eq!(task.evaluate(), 42);
		/// ```
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
		#[document_signature]
		///
		#[document_parameters("The closure to execute.")]
		///
		#[document_returns("A `Trampoline` that executes `f` when run.")]
		///
		#[inline]
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let task = Trampoline::new(|| {
		/// 	// println!("Computing!");
		/// 	1 + 1
		/// });
		///
		/// // Nothing printed yet
		/// let result = task.evaluate(); // Prints "Computing!"
		/// assert_eq!(result, 2);
		/// ```
		pub fn new(f: impl FnOnce() -> A + 'static) -> Self {
			Trampoline(Free::wrap(Thunk::new(move || Free::pure(f()))))
		}

		/// Defers the construction of a `Trampoline` itself.
		///
		/// This is critical for stack-safe recursion: instead of
		/// building a chain of `Trampoline`s directly (which grows the stack),
		/// we defer the construction.
		#[document_signature]
		///
		#[document_parameters("The closure that produces a `Trampoline`.")]
		///
		#[document_returns("A `Trampoline` that defers the creation of the inner task.")]
		///
		#[inline]
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// fn recursive_sum(
		/// 	n: u64,
		/// 	acc: u64,
		/// ) -> Trampoline<u64> {
		/// 	if n == 0 {
		/// 		Trampoline::pure(acc)
		/// 	} else {
		/// 		// Defer construction to avoid stack growth
		/// 		Trampoline::defer(move || recursive_sum(n - 1, acc + n))
		/// 	}
		/// }
		///
		/// // This works for n = 1_000_000 without stack overflow!
		/// let result = recursive_sum(1_000, 0).evaluate();
		/// assert_eq!(result, 500500);
		/// ```
		pub fn defer(f: impl FnOnce() -> Trampoline<A> + 'static) -> Self {
			Trampoline(Free::wrap(Thunk::new(move || f().0)))
		}

		/// Monadic bind with O(1) complexity.
		///
		/// Chains computations together. The key property is that
		/// left-associated chains don't degrade to O(n²).
		#[document_signature]
		///
		#[document_type_parameters("The type of the result of the new task.")]
		///
		#[document_parameters("The function to apply to the result of this task.")]
		///
		#[document_returns("A new `Trampoline` that chains `f` after this task.")]
		///
		#[inline]
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// // This is O(n), not O(n²)
		/// let mut task = Trampoline::pure(0);
		/// for i in 0 .. 100 {
		/// 	task = task.bind(move |x| Trampoline::pure(x + i));
		/// }
		/// assert_eq!(task.evaluate(), 4950);
		/// ```
		pub fn bind<B: 'static + Send>(
			self,
			f: impl FnOnce(A) -> Trampoline<B> + 'static,
		) -> Trampoline<B> {
			Trampoline(self.0.bind(move |a| f(a).0))
		}

		/// Functor map: transforms the result without changing structure.
		#[document_signature]
		///
		#[document_type_parameters("The type of the result of the mapping function.")]
		///
		#[document_parameters("The function to apply to the result of this task.")]
		///
		#[document_returns("A new `Trampoline` with the transformed result.")]
		///
		#[inline]
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let task = Trampoline::pure(10).map(|x| x * 2);
		/// assert_eq!(task.evaluate(), 20);
		/// ```
		pub fn map<B: 'static + Send>(
			self,
			f: impl FnOnce(A) -> B + 'static,
		) -> Trampoline<B> {
			self.bind(move |a| Trampoline::pure(f(a)))
		}

		/// Forces evaluation and returns the result.
		///
		/// This runs the trampoline loop, iteratively processing
		/// the CatList of continuations without growing the stack.
		#[document_signature]
		///
		#[document_parameters]
		///
		#[document_returns("The result of the computation.")]
		///
		#[document_examples]
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
		#[document_signature]
		///
		#[document_type_parameters(
			"The type of the second task's result.",
			"The type of the combined result."
		)]
		///
		#[document_parameters("The second task.", "The function to combine the results.")]
		///
		#[document_returns("A new `Trampoline` producing the combined result.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let t1 = Trampoline::pure(10);
		/// let t2 = Trampoline::pure(20);
		/// let t3 = t1.lift2(t2, |a, b| a + b);
		/// assert_eq!(t3.evaluate(), 30);
		/// ```
		pub fn lift2<B: 'static + Send, C: 'static + Send>(
			self,
			other: Trampoline<B>,
			f: impl FnOnce(A, B) -> C + 'static,
		) -> Trampoline<C> {
			self.bind(move |a| other.map(move |b| f(a, b)))
		}

		/// Sequences two `Trampoline`s, discarding the first result.
		#[document_signature]
		///
		#[document_type_parameters("The type of the second task's result.")]
		///
		#[document_parameters("The second task.")]
		///
		#[document_returns(
			"A new `Trampoline` that runs both tasks and returns the result of the second."
		)]
		///
		#[document_examples]
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
		#[document_signature]
		///
		#[document_type_parameters("The type of the state.", "The type of the step function.")]
		///
		#[document_parameters(
			"The function that performs one step of the recursion.",
			"The initial state."
		)]
		///
		#[document_returns("A `Trampoline` that performs the recursion.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::{
		/// 	Step,
		/// 	Trampoline,
		/// };
		///
		/// // Fibonacci using tail recursion
		/// fn fib(n: u64) -> Trampoline<u64> {
		/// 	Trampoline::tail_rec_m(
		/// 		|(n, a, b)| {
		/// 			if n == 0 {
		/// 				Trampoline::pure(Step::Done(a))
		/// 			} else {
		/// 				Trampoline::pure(Step::Loop((n - 1, b, a + b)))
		/// 			}
		/// 		},
		/// 		(n, 0u64, 1u64),
		/// 	)
		/// }
		///
		/// assert_eq!(fib(50).evaluate(), 12586269025);
		/// ```
		pub fn tail_rec_m<S: 'static + Send, F>(
			f: F,
			initial: S,
		) -> Self
		where
			F: Fn(S) -> Trampoline<Step<S, A>> + Clone + 'static, {
			// Use defer to ensure each step is trampolined.
			fn go<A: 'static + Send, B: 'static + Send, F>(
				f: F,
				a: A,
			) -> Trampoline<B>
			where
				F: Fn(A) -> Trampoline<Step<A, B>> + Clone + 'static, {
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
		#[document_signature]
		///
		#[document_type_parameters("The type of the state.")]
		///
		#[document_parameters(
			"The function that performs one step of the recursion.",
			"The initial state."
		)]
		///
		#[document_returns("A `Trampoline` that performs the recursion.")]
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	fp_library::types::{
		/// 		Step,
		/// 		Trampoline,
		/// 	},
		/// 	std::sync::{
		/// 		Arc,
		/// 		atomic::{
		/// 			AtomicUsize,
		/// 			Ordering,
		/// 		},
		/// 	},
		/// };
		///
		/// // Closure captures non-Clone state
		/// let counter = Arc::new(AtomicUsize::new(0));
		/// let counter_clone = Arc::clone(&counter);
		/// let task = Trampoline::arc_tail_rec_m(
		/// 	move |n| {
		/// 		counter_clone.fetch_add(1, Ordering::SeqCst);
		/// 		if n == 0 {
		/// 			Trampoline::pure(Step::Done(0))
		/// 		} else {
		/// 			Trampoline::pure(Step::Loop(n - 1))
		/// 		}
		/// 	},
		/// 	100,
		/// );
		/// assert_eq!(task.evaluate(), 0);
		/// assert_eq!(counter.load(Ordering::SeqCst), 101);
		/// ```
		pub fn arc_tail_rec_m<S: 'static + Send>(
			f: impl Fn(S) -> Trampoline<Step<S, A>> + 'static,
			initial: S,
		) -> Self {
			use std::sync::Arc;
			let f = Arc::new(f);
			let wrapper = move |s: S| {
				let f = Arc::clone(&f);
				f(s)
			};
			Self::tail_rec_m(wrapper, initial)
		}
	}

	#[document_type_parameters(
		"The type of the value produced by the task.",
		"The memoization configuration."
	)]
	impl<A: 'static + Send + Clone, Config: LazyConfig> From<Lazy<'static, A, Config>>
		for Trampoline<A>
	{
		#[document_signature]
		#[document_parameters("The lazy value to convert.")]
		#[document_returns("A trampoline that evaluates the lazy value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		/// let lazy = Lazy::<_, RcLazyConfig>::pure(42);
		/// let task = Trampoline::from(lazy);
		/// assert_eq!(task.evaluate(), 42);
		/// ```
		fn from(lazy: Lazy<'static, A, Config>) -> Self {
			Trampoline::new(move || lazy.evaluate().clone())
		}
	}

	#[document_type_parameters("The type of the value produced by the task.")]
	impl<A: 'static + Send> Deferrable<'static> for Trampoline<A> {
		/// Creates a `Trampoline` from a computation that produces it.
		#[document_signature]
		///
		#[document_parameters("A thunk that produces the trampoline.")]
		///
		#[document_returns("The deferred trampoline.")]
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
		/// let task: Trampoline<i32> = Deferrable::defer(|| Trampoline::pure(42));
		/// assert_eq!(task.evaluate(), 42);
		/// ```
		fn defer(f: impl FnOnce() -> Self + 'static) -> Self
		where
			Self: Sized, {
			Trampoline::defer(f)
		}
	}
}
pub use inner::*;

#[cfg(test)]
mod tests {
	use {
		super::*,
		crate::types::step::Step,
	};

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
			atomic::{
				AtomicUsize,
				Ordering,
			},
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
		use {
			crate::types::{
				Lazy,
				RcLazyConfig,
			},
			std::{
				cell::RefCell,
				rc::Rc,
			},
		};

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
		use {
			crate::types::{
				ArcLazyConfig,
				Lazy,
			},
			std::sync::{
				Arc,
				Mutex,
			},
		};

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
