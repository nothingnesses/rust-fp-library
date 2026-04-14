//! Stack-safe computation type with guaranteed safety for unlimited recursion depth.
//!
//! Trampolining converts stack-based recursion into heap-based iteration: instead
//! of each recursive call consuming a stack frame, each step returns a thunk that
//! the driver loop evaluates iteratively. This eliminates the risk of stack overflow
//! regardless of recursion depth.
//!
//! Built on the [`Free`](crate::types::Free) monad with O(1) [`bind`](Trampoline::bind) operations. Provides complete stack safety at the cost of requiring `'static` types. Use this for deep recursion and heavy monadic pipelines.
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
			classes::{
				Deferrable,
				LazyConfig,
				Monoid,
				Semigroup,
			},
			types::{
				ArcLazyConfig,
				Free,
				Lazy,
				RcLazyConfig,
				Thunk,
			},
		},
		core::ops::ControlFlow,
		fp_macros::*,
		std::fmt,
	};

	/// A lazy, stack-safe computation that produces a value of type `A`.
	///
	/// `Trampoline` is the "heavy-duty" monadic type for deferred computations that
	/// require **guaranteed stack safety**. It is built on [`Free<Thunk, A>`] with
	/// [`CatList`](crate::types::CatList)-based bind stack, ensuring O(1) [`bind`](Trampoline::bind)
	/// operations and unlimited recursion depth without stack overflow.
	///
	/// # Requirements
	///
	/// - `A: 'static` - Required due to type erasure via [`Box<dyn Any>`].
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
	/// `Trampoline` does NOT memoize. Each call to `evaluate` re-evaluates.
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
	/// # Drop behavior
	///
	/// Dropping a `Trampoline` dismantles its inner [`Free<ThunkBrand, A>`](Free)
	/// chain iteratively. Each suspended thunk in the chain is evaluated during drop
	/// to access the next node. Be aware that dropping a partially-evaluated
	/// `Trampoline` may trigger deferred computations.
	#[document_type_parameters("The type of the value produced by the task.")]
	///
	pub struct Trampoline<A: 'static>(
		/// The internal `Free` monad representation.
		Free<ThunkBrand, A>,
	);

	#[document_type_parameters("The type of the value produced by the task.")]
	#[document_parameters("The `Trampoline` instance.")]
	impl<A: 'static> Trampoline<A> {
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
		/// `Trampoline` does NOT memoize - each `evaluate()`
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
		/// // Nothing computed yet
		/// let result = task.evaluate(); // Now the closure runs
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
		pub fn bind<B: 'static>(
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
		pub fn map<B: 'static>(
			self,
			f: impl FnOnce(A) -> B + 'static,
		) -> Trampoline<B> {
			Trampoline(self.0.map(f))
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

		/// Converts this `Trampoline` into a memoized [`Lazy`](crate::types::Lazy) value.
		///
		/// The computation will be evaluated at most once; subsequent accesses
		/// return the cached result.
		#[document_signature]
		///
		#[document_returns(
			"A memoized `Lazy` value that evaluates this trampoline on first access."
		)]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let task = Trampoline::new(|| 42);
		/// let lazy = task.into_rc_lazy();
		/// // evaluate() returns &i32, so deref to get i32 for comparison
		/// assert_eq!(*lazy.evaluate(), 42);
		/// ```
		pub fn into_rc_lazy(self) -> Lazy<'static, A, RcLazyConfig> {
			Lazy::from(self)
		}

		/// Evaluates this `Trampoline` and wraps the result in a thread-safe [`ArcLazy`](crate::types::Lazy).
		///
		/// The trampoline is evaluated eagerly because its inner closures are
		/// `!Send` (they are stored as `Box<dyn FnOnce>` inside the underlying
		/// `Free` monad), so they cannot be placed inside an `Arc`-based lazy
		/// value that requires `Send`. By evaluating first, only the resulting
		/// `A` (which is `Send + Sync`) needs to cross the thread-safety boundary.
		#[document_signature]
		///
		#[document_returns("A thread-safe `ArcLazy` containing the eagerly evaluated result.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let task = Trampoline::new(|| 42);
		/// let lazy = task.into_arc_lazy();
		/// assert_eq!(*lazy.evaluate(), 42);
		/// ```
		pub fn into_arc_lazy(self) -> Lazy<'static, A, ArcLazyConfig>
		where
			A: Send + Sync, {
			Lazy::from(self)
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
		pub fn lift2<B: 'static, C: 'static>(
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
		pub fn then<B: 'static>(
			self,
			other: Trampoline<B>,
		) -> Trampoline<B> {
			self.bind(move |_| other)
		}

		/// Combines two `Trampoline` values using the `Semigroup` operation on their results.
		///
		/// Evaluates both trampolines and combines the results via [`Semigroup::append`].
		/// The combination itself is deferred and stack-safe.
		#[document_signature]
		///
		#[document_parameters(
			"The second `Trampoline` whose result will be combined with this one."
		)]
		///
		#[document_returns("A new `Trampoline` producing the combined result.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let t1 = Trampoline::pure(vec![1, 2]);
		/// let t2 = Trampoline::pure(vec![3, 4]);
		/// assert_eq!(t1.append(t2).evaluate(), vec![1, 2, 3, 4]);
		/// ```
		#[inline]
		pub fn append(
			self,
			other: Trampoline<A>,
		) -> Trampoline<A>
		where
			A: Semigroup + 'static, {
			self.lift2(other, Semigroup::append)
		}

		/// Creates a `Trampoline` that produces the identity element for the given `Monoid`.
		#[document_signature]
		///
		#[document_returns("A `Trampoline` producing the monoid identity element.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let t: Trampoline<Vec<i32>> = Trampoline::empty();
		/// assert_eq!(t.evaluate(), Vec::<i32>::new());
		/// ```
		#[inline]
		pub fn empty() -> Trampoline<A>
		where
			A: Monoid + 'static, {
			Trampoline::pure(Monoid::empty())
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
		/// 	core::ops::ControlFlow,
		/// 	fp_library::types::Trampoline,
		/// };
		///
		/// // Fibonacci using tail recursion
		/// fn fib(n: u64) -> Trampoline<u64> {
		/// 	Trampoline::tail_rec_m(
		/// 		|(n, a, b)| {
		/// 			if n == 0 {
		/// 				Trampoline::pure(ControlFlow::Break(a))
		/// 			} else {
		/// 				Trampoline::pure(ControlFlow::Continue((n - 1, b, a + b)))
		/// 			}
		/// 		},
		/// 		(n, 0u64, 1u64),
		/// 	)
		/// }
		///
		/// assert_eq!(fib(50).evaluate(), 12586269025);
		/// ```
		pub fn tail_rec_m<S: 'static>(
			f: impl Fn(S) -> Trampoline<ControlFlow<A, S>> + Clone + 'static,
			initial: S,
		) -> Self {
			// Use defer to ensure each step is trampolined.
			fn go<A: 'static, B: 'static, F>(
				f: F,
				a: A,
			) -> Trampoline<B>
			where
				F: Fn(A) -> Trampoline<ControlFlow<B, A>> + Clone + 'static, {
				Trampoline::defer(move || {
					let result = f(a);
					result.bind(move |step| match step {
						ControlFlow::Continue(next) => go(f, next),
						ControlFlow::Break(b) => Trampoline::pure(b),
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
		/// 	core::ops::ControlFlow,
		/// 	fp_library::types::Trampoline,
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
		/// 			Trampoline::pure(ControlFlow::Break(0))
		/// 		} else {
		/// 			Trampoline::pure(ControlFlow::Continue(n - 1))
		/// 		}
		/// 	},
		/// 	100,
		/// );
		/// assert_eq!(task.evaluate(), 0);
		/// assert_eq!(counter.load(Ordering::SeqCst), 101);
		/// ```
		pub fn arc_tail_rec_m<S: 'static>(
			f: impl Fn(S) -> Trampoline<ControlFlow<A, S>> + 'static,
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

		/// Peels off one layer of the trampoline.
		///
		/// Returns `Ok(a)` if the computation has already completed with value `a`,
		/// or `Err(thunk)` if the computation is suspended. Evaluating the returned
		/// [`Thunk`] yields the next `Trampoline` step.
		///
		/// This is useful for implementing custom interpreters or drivers that need
		/// to interleave trampoline steps with other logic (e.g., logging, resource
		/// cleanup, cooperative scheduling).
		#[document_signature]
		///
		#[document_returns(
			"`Ok(a)` if the computation is finished, `Err(thunk)` if it is suspended."
		)]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// // A pure trampoline resumes immediately.
		/// let t = Trampoline::pure(42);
		/// assert_eq!(t.resume().unwrap(), 42);
		///
		/// // A deferred trampoline is suspended.
		/// let t = Trampoline::defer(|| Trampoline::pure(99));
		/// match t.resume() {
		/// 	Ok(_) => panic!("expected suspension"),
		/// 	Err(thunk) => {
		/// 		let next = thunk.evaluate();
		/// 		assert_eq!(next.resume().unwrap(), 99);
		/// 	}
		/// }
		/// ```
		pub fn resume(self) -> Result<A, Thunk<'static, Trampoline<A>>> {
			match self.0.resume() {
				Ok(a) => Ok(a),
				Err(thunk_of_free) => Err(thunk_of_free.map(Trampoline)),
			}
		}
	}

	#[document_type_parameters(
		"The type of the value produced by the task.",
		"The memoization configuration."
	)]
	impl<A: 'static + Clone, Config: LazyConfig> From<Lazy<'static, A, Config>> for Trampoline<A> {
		/// Converts a [`Lazy`] value into a [`Trampoline`] by cloning the memoized value.
		///
		/// This conversion clones the cached value on each evaluation.
		/// The cost depends on the [`Clone`] implementation of `A`.
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
	impl<A: 'static> Deferrable<'static> for Trampoline<A> {
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

	#[document_type_parameters("The type of the value produced by the task.")]
	impl<A: Semigroup + 'static> Semigroup for Trampoline<A> {
		/// Combines two `Trampoline`s by combining their results via [`Semigroup::append`].
		#[document_signature]
		///
		#[document_parameters("The first `Trampoline`.", "The second `Trampoline`.")]
		///
		#[document_returns("A new `Trampoline` producing the combined result.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	classes::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let t1 = Trampoline::pure(vec![1, 2]);
		/// let t2 = Trampoline::pure(vec![3, 4]);
		/// let t3 = append::<_>(t1, t2);
		/// assert_eq!(t3.evaluate(), vec![1, 2, 3, 4]);
		/// ```
		fn append(
			a: Self,
			b: Self,
		) -> Self {
			a.lift2(b, Semigroup::append)
		}
	}

	#[document_type_parameters("The type of the value produced by the task.")]
	impl<A: Monoid + 'static> Monoid for Trampoline<A> {
		/// Returns a `Trampoline` producing the identity element for `A`.
		#[document_signature]
		///
		#[document_returns("A `Trampoline` producing the monoid identity element.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	classes::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let t: Trampoline<Vec<i32>> = empty::<Trampoline<Vec<i32>>>();
		/// assert_eq!(t.evaluate(), Vec::<i32>::new());
		/// ```
		fn empty() -> Self {
			Trampoline::pure(Monoid::empty())
		}
	}

	#[document_type_parameters("The type of the value produced by the task.")]
	#[document_parameters("The trampoline to format.")]
	impl<A: 'static> fmt::Debug for Trampoline<A> {
		/// Formats the trampoline without evaluating it.
		#[document_signature]
		#[document_parameters("The formatter.")]
		#[document_returns("The formatting result.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		/// let task = Trampoline::pure(42);
		/// assert_eq!(format!("{:?}", task), "Trampoline(<unevaluated>)");
		/// ```
		fn fmt(
			&self,
			f: &mut fmt::Formatter<'_>,
		) -> fmt::Result {
			f.write_str("Trampoline(<unevaluated>)")
		}
	}
}
pub use inner::*;

#[cfg(test)]
#[expect(
	clippy::unwrap_used,
	clippy::panic,
	reason = "Tests use panicking operations for brevity and clarity"
)]
mod tests {
	use {
		super::*,
		core::ops::ControlFlow,
		quickcheck_macros::quickcheck,
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
						Trampoline::pure(ControlFlow::Break(acc))
					} else {
						Trampoline::pure(ControlFlow::Continue((n - 1, n * acc)))
					}
				},
				(n, 1u64),
			)
		}

		assert_eq!(factorial(5).evaluate(), 120);
	}

	/// Tests `Trampoline::lift2`.
	///
	/// Verifies that `lift2` combines two tasks.
	#[test]
	fn test_task_lift2() {
		let t1 = Trampoline::pure(10);
		let t2 = Trampoline::pure(20);
		let t3 = t1.lift2(t2, |a, b| a + b);
		assert_eq!(t3.evaluate(), 30);
	}

	/// Tests `Trampoline::then`.
	///
	/// Verifies that `then` sequences two tasks.
	#[test]
	fn test_task_then() {
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
					Trampoline::pure(ControlFlow::Break(0))
				} else {
					Trampoline::pure(ControlFlow::Continue(n - 1))
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

	/// Tests `From<Thunk>` for `Trampoline`.
	///
	/// Verifies that converting a `Thunk` to a `Trampoline` preserves the computed value.
	#[test]
	fn test_task_from_thunk() {
		use crate::types::Thunk;

		let thunk = Thunk::pure(42);
		let task = Trampoline::from(thunk);
		assert_eq!(task.evaluate(), 42);
	}

	/// Tests roundtrip `Thunk` -> `Trampoline` -> evaluate.
	///
	/// Verifies that a lazy thunk is correctly evaluated when converted to a trampoline.
	#[test]
	fn test_task_from_thunk_lazy() {
		use crate::types::Thunk;

		let thunk = Thunk::new(|| 21 * 2);
		let task = Trampoline::from(thunk);
		assert_eq!(task.evaluate(), 42);
	}

	// QuickCheck Law Tests

	// Functor Laws (via inherent methods)

	/// Functor identity: `pure(a).map(identity) == a`.
	#[quickcheck]
	fn functor_identity(x: i32) -> bool {
		Trampoline::pure(x).map(|a| a).evaluate() == x
	}

	/// Functor composition: `fa.map(f . g) == fa.map(g).map(f)`.
	#[quickcheck]
	fn functor_composition(x: i32) -> bool {
		let f = |a: i32| a.wrapping_add(1);
		let g = |a: i32| a.wrapping_mul(2);
		let lhs = Trampoline::pure(x).map(move |a| f(g(a))).evaluate();
		let rhs = Trampoline::pure(x).map(g).map(f).evaluate();
		lhs == rhs
	}

	// Monad Laws (via inherent methods)

	/// Monad left identity: `pure(a).bind(f) == f(a)`.
	#[quickcheck]
	fn monad_left_identity(a: i32) -> bool {
		let f = |x: i32| Trampoline::pure(x.wrapping_mul(2));
		Trampoline::pure(a).bind(f).evaluate() == f(a).evaluate()
	}

	/// Monad right identity: `m.bind(pure) == m`.
	#[quickcheck]
	fn monad_right_identity(x: i32) -> bool {
		Trampoline::pure(x).bind(Trampoline::pure).evaluate() == x
	}

	/// Monad associativity: `m.bind(f).bind(g) == m.bind(|a| f(a).bind(g))`.
	#[quickcheck]
	fn monad_associativity(x: i32) -> bool {
		let f = |a: i32| Trampoline::pure(a.wrapping_add(1));
		let g = |a: i32| Trampoline::pure(a.wrapping_mul(3));
		let lhs = Trampoline::pure(x).bind(f).bind(g).evaluate();
		let rhs = Trampoline::pure(x).bind(move |a| f(a).bind(g)).evaluate();
		lhs == rhs
	}

	// Tests for !Send types (Rc)

	/// Tests that `Trampoline` works with `Rc<T>`, a `!Send` type.
	///
	/// This verifies that the `Send` bound relaxation allows single-threaded
	/// stack-safe recursion with reference-counted types.
	#[test]
	fn test_trampoline_with_rc() {
		use std::rc::Rc;

		let rc_val = Rc::new(42);
		let task = Trampoline::pure(rc_val);
		let result = task.evaluate();
		assert_eq!(*result, 42);
	}

	/// Tests `Trampoline::bind` with `Rc<T>`.
	///
	/// Verifies that `bind` works correctly when the value type is `!Send`.
	#[test]
	fn test_trampoline_bind_with_rc() {
		use std::rc::Rc;

		let task = Trampoline::pure(Rc::new(10)).bind(|rc| {
			let val = *rc;
			Trampoline::pure(Rc::new(val * 2))
		});
		assert_eq!(*task.evaluate(), 20);
	}

	/// Tests `Trampoline::map` with `Rc<T>`.
	///
	/// Verifies that `map` works correctly with `!Send` types.
	#[test]
	fn test_trampoline_map_with_rc() {
		use std::rc::Rc;

		let task = Trampoline::pure(Rc::new(10)).map(|rc| Rc::new(*rc * 3));
		assert_eq!(*task.evaluate(), 30);
	}

	/// Tests `Trampoline::defer` with `Rc<T>`.
	///
	/// Verifies that deferred construction works with `!Send` types.
	#[test]
	fn test_trampoline_defer_with_rc() {
		use std::rc::Rc;

		let task = Trampoline::defer(|| Trampoline::pure(Rc::new(42)));
		assert_eq!(*task.evaluate(), 42);
	}

	/// Tests `Trampoline::tail_rec_m` with `Rc<T>`.
	///
	/// Verifies that stack-safe recursion works with `!Send` types.
	#[test]
	fn test_trampoline_tail_rec_m_with_rc() {
		use std::rc::Rc;

		let task = Trampoline::tail_rec_m(
			|(n, acc): (u64, Rc<u64>)| {
				if n == 0 {
					Trampoline::pure(ControlFlow::Break(acc))
				} else {
					Trampoline::pure(ControlFlow::Continue((n - 1, Rc::new(*acc + n))))
				}
			},
			(100u64, Rc::new(0u64)),
		);
		assert_eq!(*task.evaluate(), 5050);
	}

	#[test]
	fn test_trampoline_append() {
		let t1 = Trampoline::pure(vec![1, 2]);
		let t2 = Trampoline::pure(vec![3, 4]);
		assert_eq!(t1.append(t2).evaluate(), vec![1, 2, 3, 4]);
	}

	#[test]
	fn test_trampoline_append_strings() {
		let t1 = Trampoline::pure("hello".to_string());
		let t2 = Trampoline::pure(" world".to_string());
		assert_eq!(t1.append(t2).evaluate(), "hello world");
	}

	#[test]
	fn test_trampoline_empty() {
		let t: Trampoline<Vec<i32>> = Trampoline::empty();
		assert_eq!(t.evaluate(), Vec::<i32>::new());
	}

	#[test]
	fn test_trampoline_append_with_empty() {
		let t1 = Trampoline::pure(vec![1, 2, 3]);
		let t2: Trampoline<Vec<i32>> = Trampoline::empty();
		assert_eq!(t1.append(t2).evaluate(), vec![1, 2, 3]);
	}

	// 7.7: Deeper stack safety stress test for tail_rec_m

	/// Stress test for `Trampoline::tail_rec_m` with 100,000+ iterations.
	///
	/// Verifies that stack safety holds at depths far exceeding typical stack limits.
	#[test]
	fn test_tail_rec_m_deep_stack_safety() {
		let n: u64 = 200_000;
		let result = Trampoline::tail_rec_m(
			move |acc: u64| {
				if acc >= n {
					Trampoline::pure(ControlFlow::Break(acc))
				} else {
					Trampoline::pure(ControlFlow::Continue(acc + 1))
				}
			},
			0u64,
		);
		assert_eq!(result.evaluate(), n);
	}

	/// Stress test for `Trampoline::arc_tail_rec_m` with 100,000+ iterations.
	///
	/// Verifies that the `Arc`-based variant is also stack-safe at high depth.
	#[test]
	fn test_arc_tail_rec_m_deep_stack_safety() {
		let n: u64 = 200_000;
		let result = Trampoline::arc_tail_rec_m(
			move |acc: u64| {
				if acc >= n {
					Trampoline::pure(ControlFlow::Break(acc))
				} else {
					Trampoline::pure(ControlFlow::Continue(acc + 1))
				}
			},
			0u64,
		);
		assert_eq!(result.evaluate(), n);
	}

	/// Tests `Trampoline::resume` on a pure value.
	///
	/// Verifies that resuming a pure trampoline returns `Ok(value)`.
	#[test]
	fn test_resume_pure() {
		let t = Trampoline::pure(42);
		assert_eq!(t.resume().unwrap(), 42);
	}

	/// Tests `Trampoline::resume` on a deferred computation.
	///
	/// Verifies that resuming a deferred trampoline returns `Err(thunk)`,
	/// and evaluating the thunk yields another trampoline that can be resumed.
	#[test]
	fn test_resume_deferred() {
		let t = Trampoline::defer(|| Trampoline::pure(99));
		match t.resume() {
			Ok(_) => panic!("expected suspension"),
			Err(thunk) => {
				let next = thunk.evaluate();
				assert_eq!(next.resume().unwrap(), 99);
			}
		}
	}

	/// Tests that resuming through a chain of deferred steps eventually reaches `Ok`.
	///
	/// Builds a chain of three deferred steps and manually drives it to completion.
	#[test]
	fn test_resume_chain_reaches_ok() {
		let t =
			Trampoline::defer(|| Trampoline::defer(|| Trampoline::defer(|| Trampoline::pure(7))));

		let mut current = t;
		let mut steps = 0;
		loop {
			match current.resume() {
				Ok(value) => {
					assert_eq!(value, 7);
					break;
				}
				Err(thunk) => {
					current = thunk.evaluate();
					steps += 1;
				}
			}
		}
		assert!(steps > 0, "expected at least one suspension step");
	}
}
