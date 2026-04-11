//! Thread-safe deferred, non-memoized computation.
//!
//! Like [`Thunk`](crate::types::Thunk) but with a `Send` bound on the inner closure,
//! enabling thread-safe deferred computation chains and truly lazy
//! [`into_arc_lazy`](SendThunk::into_arc_lazy) without eager evaluation.
//!
//! Standard HKT traits (`Functor`, `Semimonad`, etc.) cannot be implemented because
//! their signatures do not require `Send` on mapping functions. Use the inherent
//! methods ([`map`](SendThunk::map), [`bind`](SendThunk::bind)) instead.

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			brands::SendThunkBrand,
			classes::{
				CloneFn,
				Foldable,
				FoldableWithIndex,
				LiftFn,
				Monoid,
				Semigroup,
				SendDeferrable,
				WithIndex,
			},
			impl_kind,
			kinds::*,
			types::{
				ArcLazy,
				Thunk,
			},
		},
		core::ops::ControlFlow,
		fp_macros::*,
		std::{
			fmt,
			sync::Arc,
		},
	};

	/// A thread-safe deferred computation that produces a value of type `A`.
	///
	/// `SendThunk` is the `Send`-capable counterpart of [`Thunk`]. It wraps a
	/// `Box<dyn FnOnce() -> A + Send + 'a>`, so it can be transferred across thread
	/// boundaries. Like `Thunk`, it is NOT memoized and does not cache results.
	///
	/// The key advantage over `Thunk` is that [`into_arc_lazy`](SendThunk::into_arc_lazy)
	/// can wrap the closure lazily in an [`ArcLazy`] without forcing evaluation
	/// first, because the inner closure satisfies `Send`.
	///
	/// ### Higher-Kinded Type Representation
	///
	/// The higher-kinded representation of this type constructor is
	/// [`SendThunkBrand`](crate::brands::SendThunkBrand), which is fully
	/// polymorphic over the result type.
	///
	/// ### Trade-offs vs Other Lazy Types
	///
	/// | Aspect         | `SendThunk<'a, A>`            | `Thunk<'a, A>`                | `Trampoline<A>`              | `ArcLazy<'a, A>`             |
	/// |----------------|-------------------------------|-------------------------------|------------------------------|------------------------------|
	/// | Thread safety  | `Send`                        | Not `Send`                    | Not `Send`                   | `Send + Sync`                |
	/// | HKT compatible | No (needs `Send` closures)    | Yes                           | No (requires `'static`)      | Partial (`SendRefFunctor`)   |
	/// | Stack-safe     | Partial (`tail_rec_m` only)   | Partial (`tail_rec_m` only)   | Yes (unlimited)              | N/A (memoized)               |
	/// | Memoized       | No                            | No                            | No                           | Yes                          |
	/// | Lifetime       | `'a` (can borrow)             | `'a` (can borrow)             | `'static` only               | `'a` (can borrow)            |
	/// | Use case       | Cross-thread lazy pipelines   | Glue code, composition        | Deep recursion, pipelines    | Shared cached values          |
	///
	/// ### HKT Trait Limitations
	///
	/// Standard HKT traits such as `Functor`, `Pointed`, `Semimonad`, and
	/// `Semiapplicative` cannot be implemented for `SendThunkBrand` because
	/// their signatures do not require `Send` on the mapping or binding
	/// functions. Since `SendThunk` stores a `Box<dyn FnOnce() -> A + Send>`,
	/// composing it with a non-`Send` closure would violate the `Send` invariant.
	///
	/// Use the inherent methods ([`map`](SendThunk::map),
	/// [`bind`](SendThunk::bind)) instead, which accept `Send` closures
	/// explicitly.
	///
	/// ### Algebraic Properties
	///
	/// `SendThunk` satisfies the monad laws through its inherent methods, even though
	/// it cannot implement the HKT `Monad` trait (due to the `Send` bound requirement):
	/// - `pure(a).bind(f) ≡ f(a)` (left identity).
	/// - `m.bind(|x| pure(x)) ≡ m` (right identity).
	/// - `m.bind(f).bind(g) ≡ m.bind(|x| f(x).bind(g))` (associativity).
	///
	/// ### Stack Safety
	///
	/// Like `Thunk`, `SendThunk::bind` chains are **not** stack-safe. Each nested
	/// [`bind`](SendThunk::bind) adds a frame to the call stack.
	#[document_type_parameters(
		"The lifetime of the computation.",
		"The type of the value produced by the computation."
	)]
	///
	pub struct SendThunk<'a, A>(
		/// The thread-safe closure that performs the computation.
		Box<dyn FnOnce() -> A + Send + 'a>,
	);

	// INVARIANT: SendThunk is Send because its inner closure is Send.
	// The Box<dyn FnOnce() -> A + Send + 'a> already guarantees Send on the closure.
	// Rust auto-derives Send for Box<dyn ... + Send>, so this is sound.

	#[document_type_parameters(
		"The lifetime of the computation.",
		"The type of the value produced by the computation."
	)]
	#[document_parameters("The send thunk instance.")]
	impl<'a, A: 'a> SendThunk<'a, A> {
		/// Returns the inner boxed closure, erasing the `Send` bound.
		///
		/// This is a crate-internal helper used by `From<SendThunk> for Thunk`
		/// to perform a zero-cost unsizing coercion.
		#[document_signature]
		#[document_returns("The inner boxed closure with the `Send` bound erased.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let send_thunk = SendThunk::new(|| 42);
		/// let thunk = Thunk::from(send_thunk);
		/// assert_eq!(thunk.evaluate(), 42);
		/// ```
		#[inline]
		pub(crate) fn into_inner(self) -> Box<dyn FnOnce() -> A + 'a> {
			self.0
		}

		/// Creates a new `SendThunk` from a thread-safe closure.
		#[document_signature]
		///
		#[document_parameters("The thread-safe closure to wrap.")]
		///
		#[document_returns("A new `SendThunk` instance.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let thunk = SendThunk::new(|| 42);
		/// assert_eq!(thunk.evaluate(), 42);
		/// ```
		#[inline]
		pub fn new(f: impl FnOnce() -> A + Send + 'a) -> Self {
			SendThunk(Box::new(f))
		}

		/// Returns a pure value (already computed).
		#[document_signature]
		///
		#[document_parameters("The value to wrap.")]
		///
		#[document_returns("A new `SendThunk` instance containing the value.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let thunk = SendThunk::pure(42);
		/// assert_eq!(thunk.evaluate(), 42);
		/// ```
		#[inline]
		pub fn pure(a: A) -> Self
		where
			A: Send + 'a, {
			SendThunk::new(move || a)
		}

		/// Defers a computation that returns a `SendThunk`.
		#[document_signature]
		///
		#[document_parameters("The thunk that returns a `SendThunk`.")]
		///
		#[document_returns("A new `SendThunk` instance.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let thunk = SendThunk::defer(|| SendThunk::pure(42));
		/// assert_eq!(thunk.evaluate(), 42);
		/// ```
		#[inline]
		pub fn defer(f: impl FnOnce() -> SendThunk<'a, A> + Send + 'a) -> Self {
			SendThunk::new(move || f().evaluate())
		}

		/// Monadic bind: chains computations.
		///
		/// Note: Each `bind` adds to the call stack. For deep recursion,
		/// consider converting to [`Trampoline`](crate::types::Trampoline).
		#[document_signature]
		///
		#[document_type_parameters("The type of the result of the new computation.")]
		///
		#[document_parameters("The function to apply to the result of the computation.")]
		///
		#[document_returns("A new `SendThunk` instance representing the chained computation.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let thunk = SendThunk::pure(21).bind(|x| SendThunk::pure(x * 2));
		/// assert_eq!(thunk.evaluate(), 42);
		/// ```
		#[inline]
		pub fn bind<B: 'a>(
			self,
			f: impl FnOnce(A) -> SendThunk<'a, B> + Send + 'a,
		) -> SendThunk<'a, B> {
			SendThunk::new(move || {
				let a = (self.0)();
				let thunk_b = f(a);
				(thunk_b.0)()
			})
		}

		/// Functor map: transforms the result.
		#[document_signature]
		///
		#[document_type_parameters("The type of the result of the transformation.")]
		///
		#[document_parameters("The function to apply to the result of the computation.")]
		///
		#[document_returns("A new `SendThunk` instance with the transformed result.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let thunk = SendThunk::pure(21).map(|x| x * 2);
		/// assert_eq!(thunk.evaluate(), 42);
		/// ```
		#[inline]
		pub fn map<B: 'a>(
			self,
			f: impl FnOnce(A) -> B + Send + 'a,
		) -> SendThunk<'a, B> {
			SendThunk::new(move || f((self.0)()))
		}

		/// Forces evaluation and returns the result.
		#[document_signature]
		///
		#[document_returns("The result of the computation.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let thunk = SendThunk::pure(42);
		/// assert_eq!(thunk.evaluate(), 42);
		/// ```
		#[inline]
		pub fn evaluate(self) -> A {
			(self.0)()
		}

		/// Performs tail-recursive monadic computation.
		///
		/// The step function `f` is called in a loop, avoiding stack growth.
		/// Each iteration evaluates `f(state)` and inspects the resulting
		/// [`ControlFlow`]: `ControlFlow::Continue(next)` continues with `next`, while
		/// `ControlFlow::Break(a)` breaks out and returns `a`.
		///
		/// # Step Function
		///
		/// The function `f` is bounded by `Fn`, so it is callable multiple
		/// times by shared reference. Each iteration of the loop calls `f`
		/// without consuming it, so no `Clone` bound is needed.
		#[document_signature]
		///
		#[document_type_parameters("The type of the loop state.")]
		///
		#[document_parameters(
			"The step function that produces the next state or the final result.",
			"The initial state."
		)]
		///
		#[document_returns("A `SendThunk` that, when evaluated, runs the tail-recursive loop.")]
		///
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	core::ops::ControlFlow,
		/// 	fp_library::types::*,
		/// };
		///
		/// let result = SendThunk::tail_rec_m(
		/// 	|x| {
		/// 		SendThunk::pure(
		/// 			if x < 1000 { ControlFlow::Continue(x + 1) } else { ControlFlow::Break(x) },
		/// 		)
		/// 	},
		/// 	0,
		/// );
		/// assert_eq!(result.evaluate(), 1000);
		/// ```
		pub fn tail_rec_m<S>(
			f: impl Fn(S) -> SendThunk<'a, ControlFlow<A, S>> + Send + 'a,
			initial: S,
		) -> Self
		where
			S: Send + 'a, {
			SendThunk::new(move || {
				let mut state = initial;
				loop {
					match f(state).evaluate() {
						ControlFlow::Break(a) => return a,
						ControlFlow::Continue(next) => state = next,
					}
				}
			})
		}

		/// Arc-wrapped version of [`tail_rec_m`](SendThunk::tail_rec_m).
		///
		/// Wraps the closure in [`Arc`] internally so it can be shared
		/// across thread boundaries. The step function must be `Send + Sync`
		/// (rather than just `Send` as in `tail_rec_m`).
		#[document_signature]
		///
		#[document_type_parameters("The type of the loop state.")]
		///
		#[document_parameters(
			"The step function that produces the next state or the final result.",
			"The initial state."
		)]
		///
		#[document_returns("A `SendThunk` that, when evaluated, runs the tail-recursive loop.")]
		///
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	core::ops::ControlFlow,
		/// 	fp_library::types::*,
		/// 	std::sync::{
		/// 		Arc,
		/// 		atomic::{
		/// 			AtomicUsize,
		/// 			Ordering,
		/// 		},
		/// 	},
		/// };
		///
		/// let counter = Arc::new(AtomicUsize::new(0));
		/// let counter_clone = Arc::clone(&counter);
		/// let result = SendThunk::arc_tail_rec_m(
		/// 	move |x| {
		/// 		counter_clone.fetch_add(1, Ordering::SeqCst);
		/// 		SendThunk::pure(
		/// 			if x < 100 { ControlFlow::Continue(x + 1) } else { ControlFlow::Break(x) },
		/// 		)
		/// 	},
		/// 	0,
		/// );
		/// assert_eq!(result.evaluate(), 100);
		/// assert_eq!(counter.load(Ordering::SeqCst), 101);
		/// ```
		pub fn arc_tail_rec_m<S>(
			f: impl Fn(S) -> SendThunk<'a, ControlFlow<A, S>> + Send + Sync + 'a,
			initial: S,
		) -> Self
		where
			S: Send + 'a, {
			let f = Arc::new(f);
			let wrapper = move |s: S| {
				let f = Arc::clone(&f);
				f(s)
			};
			Self::tail_rec_m(wrapper, initial)
		}

		/// Converts this `SendThunk` into a memoized [`ArcLazy`] value.
		///
		/// Unlike [`Thunk::into_arc_lazy`](crate::types::Thunk::into_arc_lazy), this
		/// does **not** evaluate eagerly. The inner `Send` closure is passed
		/// directly into `ArcLazy::new`, so evaluation is deferred until the
		/// `ArcLazy` is first accessed.
		#[document_signature]
		///
		#[document_returns("A thread-safe `ArcLazy` that evaluates this thunk on first access.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let thunk = SendThunk::new(|| 42);
		/// let lazy = thunk.into_arc_lazy();
		/// assert_eq!(*lazy.evaluate(), 42);
		/// ```
		#[inline]
		pub fn into_arc_lazy(self) -> ArcLazy<'a, A>
		where
			A: Send + Sync, {
			self.into()
		}
	}

	#[document_type_parameters("The lifetime of the computation.", "The type of the value.")]
	impl<'a, A: 'a> From<Thunk<'a, A>> for SendThunk<'a, A>
	where
		A: Send,
	{
		/// Converts a [`Thunk`] into a [`SendThunk`].
		///
		/// The `Thunk` closure is not `Send`, so the conversion eagerly
		/// evaluates it and wraps the owned result in a new `SendThunk`.
		#[document_signature]
		#[document_parameters("The thunk to convert.")]
		#[document_returns("A new `SendThunk` wrapping the evaluated result.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		/// let thunk = Thunk::pure(42);
		/// let send_thunk = SendThunk::from(thunk);
		/// assert_eq!(send_thunk.evaluate(), 42);
		/// ```
		fn from(thunk: Thunk<'a, A>) -> Self {
			SendThunk::pure(thunk.evaluate())
		}
	}

	impl_kind! {
		for SendThunkBrand {
			type Of<'a, A: 'a>: 'a = SendThunk<'a, A>;
		}
	}

	impl Foldable for SendThunkBrand {
		/// Folds the `SendThunk` from the right.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the computation.",
			"The brand of the cloneable function to use.",
			"The type of the elements in the structure.",
			"The type of the accumulator."
		)]
		///
		#[document_parameters(
			"The function to apply to each element and the accumulator.",
			"The initial value of the accumulator.",
			"The `SendThunk` to fold."
		)]
		///
		#[document_returns("The final accumulator value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let thunk = SendThunk::pure(10);
		/// let result =
		/// 	fold_right_explicit::<RcFnBrand, SendThunkBrand, _, _, _, _>(|a, b| a + b, 5, thunk);
		/// assert_eq!(result, 15);
		/// ```
		fn fold_right<'a, FnBrand, A: 'a + Clone, B: 'a>(
			func: impl Fn(A, B) -> B + 'a,
			initial: B,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> B
		where
			FnBrand: CloneFn + 'a, {
			func(fa.evaluate(), initial)
		}

		/// Folds the `SendThunk` from the left.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the computation.",
			"The brand of the cloneable function to use.",
			"The type of the elements in the structure.",
			"The type of the accumulator."
		)]
		///
		#[document_parameters(
			"The function to apply to the accumulator and each element.",
			"The initial value of the accumulator.",
			"The `SendThunk` to fold."
		)]
		///
		#[document_returns("The final accumulator value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let thunk = SendThunk::pure(10);
		/// let result =
		/// 	fold_left_explicit::<RcFnBrand, SendThunkBrand, _, _, _, _>(|b, a| b + a, 5, thunk);
		/// assert_eq!(result, 15);
		/// ```
		fn fold_left<'a, FnBrand, A: 'a + Clone, B: 'a>(
			func: impl Fn(B, A) -> B + 'a,
			initial: B,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> B
		where
			FnBrand: CloneFn + 'a, {
			func(initial, fa.evaluate())
		}

		/// Maps the value to a monoid and returns it.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the computation.",
			"The brand of the cloneable function to use.",
			"The type of the elements in the structure.",
			"The type of the monoid."
		)]
		///
		#[document_parameters("The mapping function.", "The `SendThunk` to fold.")]
		///
		#[document_returns("The monoid value.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let thunk = SendThunk::pure(10);
		/// let result =
		/// 	fold_map_explicit::<RcFnBrand, SendThunkBrand, _, _, _, _>(|a: i32| a.to_string(), thunk);
		/// assert_eq!(result, "10");
		/// ```
		fn fold_map<'a, FnBrand, A: 'a + Clone, M>(
			func: impl Fn(A) -> M + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> M
		where
			M: Monoid + 'a,
			FnBrand: CloneFn + 'a, {
			func(fa.evaluate())
		}
	}

	impl WithIndex for SendThunkBrand {
		type Index = ();
	}

	impl FoldableWithIndex for SendThunkBrand {
		/// Folds the send thunk using a monoid, providing the index `()`.
		#[document_signature]
		#[document_type_parameters(
			"The lifetime of the computation.",
			"The brand of the cloneable function to use.",
			"The type of the value inside the send thunk.",
			"The monoid type."
		)]
		#[document_parameters(
			"The function to apply to the value and its index.",
			"The send thunk to fold."
		)]
		#[document_returns("The monoid value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::foldable_with_index::FoldableWithIndex,
		/// 	types::*,
		/// };
		///
		/// let thunk = SendThunk::pure(5);
		/// let result = <SendThunkBrand as FoldableWithIndex>::fold_map_with_index::<RcFnBrand, _, _>(
		/// 	|_, x: i32| x.to_string(),
		/// 	thunk,
		/// );
		/// assert_eq!(result, "5");
		/// ```
		fn fold_map_with_index<'a, FnBrand, A: 'a + Clone, R: Monoid>(
			f: impl Fn((), A) -> R + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> R
		where
			FnBrand: LiftFn + 'a, {
			f((), fa.evaluate())
		}
	}

	#[document_type_parameters(
		"The lifetime of the computation.",
		"The type of the value produced by the computation."
	)]
	impl<'a, A: Send + 'a> SendDeferrable<'a> for SendThunk<'a, A> {
		/// Creates a `SendThunk` from a thread-safe computation that produces it.
		#[document_signature]
		///
		#[document_parameters("A thread-safe thunk that produces the send thunk.")]
		///
		#[document_returns("The deferred send thunk.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	classes::SendDeferrable,
		/// 	types::*,
		/// };
		///
		/// let task: SendThunk<i32> = SendDeferrable::send_defer(|| SendThunk::pure(42));
		/// assert_eq!(task.evaluate(), 42);
		/// ```
		fn send_defer(f: impl FnOnce() -> Self + Send + 'a) -> Self
		where
			Self: Sized, {
			SendThunk::defer(f)
		}
	}

	#[document_type_parameters(
		"The lifetime of the computation.",
		"The type of the value produced by the computation."
	)]
	impl<'a, A: Semigroup + Send + 'a> Semigroup for SendThunk<'a, A> {
		/// Combines two `SendThunk`s by combining their results.
		#[document_signature]
		///
		#[document_parameters("The first `SendThunk`.", "The second `SendThunk`.")]
		///
		#[document_returns("A new `SendThunk` containing the combined result.")]
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
		/// let t1 = SendThunk::pure("Hello".to_string());
		/// let t2 = SendThunk::pure(" World".to_string());
		/// let t3 = append::<_>(t1, t2);
		/// assert_eq!(t3.evaluate(), "Hello World");
		/// ```
		fn append(
			a: Self,
			b: Self,
		) -> Self {
			SendThunk::new(move || Semigroup::append(a.evaluate(), b.evaluate()))
		}
	}

	#[document_type_parameters(
		"The lifetime of the computation.",
		"The type of the value produced by the computation."
	)]
	impl<'a, A: Monoid + Send + 'a> Monoid for SendThunk<'a, A> {
		/// Returns the identity `SendThunk`.
		#[document_signature]
		///
		#[document_returns("A `SendThunk` producing the identity value of `A`.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	classes::*,
		/// 	types::*,
		/// };
		///
		/// let t: SendThunk<String> = SendThunk::empty();
		/// assert_eq!(t.evaluate(), "");
		/// ```
		fn empty() -> Self {
			SendThunk::new(|| Monoid::empty())
		}
	}

	#[document_type_parameters(
		"The lifetime of the computation.",
		"The type of the computed value."
	)]
	#[document_parameters("The send thunk to format.")]
	impl<'a, A> fmt::Debug for SendThunk<'a, A> {
		/// Formats the send thunk without evaluating it.
		#[document_signature]
		#[document_parameters("The formatter.")]
		#[document_returns("The formatting result.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		/// let thunk = SendThunk::pure(42);
		/// assert_eq!(format!("{:?}", thunk), "SendThunk(<unevaluated>)");
		/// ```
		fn fmt(
			&self,
			f: &mut fmt::Formatter<'_>,
		) -> fmt::Result {
			f.write_str("SendThunk(<unevaluated>)")
		}
	}
}
pub use inner::*;

#[cfg(test)]
mod tests {
	use {
		super::*,
		crate::classes::{
			monoid::empty,
			semigroup::append,
		},
		quickcheck_macros::quickcheck,
	};

	#[test]
	fn test_send_thunk_pure_and_evaluate() {
		let thunk = SendThunk::pure(42);
		assert_eq!(thunk.evaluate(), 42);
	}

	#[test]
	fn test_send_thunk_new() {
		let thunk = SendThunk::new(|| 1 + 2);
		assert_eq!(thunk.evaluate(), 3);
	}

	#[test]
	fn test_send_thunk_map() {
		let thunk = SendThunk::pure(10).map(|x| x * 3);
		assert_eq!(thunk.evaluate(), 30);
	}

	#[test]
	fn test_send_thunk_bind() {
		let thunk = SendThunk::pure(5).bind(|x| SendThunk::pure(x + 10));
		assert_eq!(thunk.evaluate(), 15);
	}

	#[test]
	fn test_send_thunk_defer() {
		let thunk = SendThunk::defer(|| SendThunk::pure(99));
		assert_eq!(thunk.evaluate(), 99);
	}

	#[test]
	fn test_send_thunk_into_arc_lazy() {
		let thunk = SendThunk::new(|| 42);
		let lazy = thunk.into_arc_lazy();
		assert_eq!(*lazy.evaluate(), 42);
		// Second access returns cached value.
		assert_eq!(*lazy.evaluate(), 42);
	}

	#[test]
	fn test_send_thunk_semigroup() {
		let t1 = SendThunk::pure("Hello".to_string());
		let t2 = SendThunk::pure(" World".to_string());
		let t3 = append(t1, t2);
		assert_eq!(t3.evaluate(), "Hello World");
	}

	#[test]
	fn test_send_thunk_monoid() {
		let t: SendThunk<String> = empty();
		assert_eq!(t.evaluate(), "");
	}

	#[test]
	fn test_send_thunk_from_thunk() {
		let thunk = crate::types::Thunk::pure(42);
		let send_thunk = SendThunk::from(thunk);
		assert_eq!(send_thunk.evaluate(), 42);
	}

	#[test]
	fn test_send_thunk_debug() {
		let thunk = SendThunk::pure(42);
		assert_eq!(format!("{:?}", thunk), "SendThunk(<unevaluated>)");
	}

	#[test]
	fn test_send_thunk_is_send() {
		fn assert_send<T: Send>() {}
		assert_send::<SendThunk<'static, i32>>();
	}

	#[test]
	fn test_send_thunk_send_deferrable() {
		use crate::classes::SendDeferrable;
		let task: SendThunk<i32> = SendDeferrable::send_defer(|| SendThunk::pure(42));
		assert_eq!(task.evaluate(), 42);
	}

	/// Tests that a `SendThunk` can be sent to another thread and evaluated there.
	///
	/// Verifies that `SendThunk` satisfies the `Send` bound by moving it across a
	/// thread boundary via `std::thread::spawn`.
	#[test]
	fn test_send_thunk_cross_thread() {
		let thunk = SendThunk::new(|| 42 * 2);
		let handle = std::thread::spawn(move || thunk.evaluate());
		let result = handle.join().expect("thread should not panic");
		assert_eq!(result, 84);
	}

	/// Tests that a mapped `SendThunk` evaluates correctly on another thread.
	#[test]
	fn test_send_thunk_cross_thread_with_map() {
		let thunk = SendThunk::pure(10).map(|x| x + 5).map(|x| x * 3);
		let handle = std::thread::spawn(move || thunk.evaluate());
		let result = handle.join().expect("thread should not panic");
		assert_eq!(result, 45);
	}

	/// Tests that a bound `SendThunk` evaluates correctly on another thread.
	#[test]
	fn test_send_thunk_cross_thread_with_bind() {
		let thunk = SendThunk::pure(7).bind(|x| SendThunk::pure(x * 6));
		let handle = std::thread::spawn(move || thunk.evaluate());
		let result = handle.join().expect("thread should not panic");
		assert_eq!(result, 42);
	}

	#[test]
	fn test_send_thunk_tail_rec_m() {
		use core::ops::ControlFlow;
		let result = SendThunk::tail_rec_m(
			|x| {
				SendThunk::pure(
					if x < 1000 { ControlFlow::Continue(x + 1) } else { ControlFlow::Break(x) },
				)
			},
			0,
		);
		assert_eq!(result.evaluate(), 1000);
	}

	#[test]
	fn test_send_thunk_tail_rec_m_stack_safety() {
		use core::ops::ControlFlow;
		let iterations: i64 = 200_000;
		let result = SendThunk::tail_rec_m(
			|acc| {
				SendThunk::pure(
					if acc < iterations {
						ControlFlow::Continue(acc + 1)
					} else {
						ControlFlow::Break(acc)
					},
				)
			},
			0i64,
		);
		assert_eq!(result.evaluate(), iterations);
	}

	#[test]
	fn test_send_thunk_arc_tail_rec_m() {
		use {
			core::ops::ControlFlow,
			std::sync::{
				Arc,
				atomic::{
					AtomicUsize,
					Ordering,
				},
			},
		};
		let counter = Arc::new(AtomicUsize::new(0));
		let counter_clone = Arc::clone(&counter);
		let result = SendThunk::arc_tail_rec_m(
			move |x| {
				counter_clone.fetch_add(1, Ordering::SeqCst);
				SendThunk::pure(
					if x < 100 { ControlFlow::Continue(x + 1) } else { ControlFlow::Break(x) },
				)
			},
			0,
		);
		assert_eq!(result.evaluate(), 100);
		assert_eq!(counter.load(Ordering::SeqCst), 101);
	}

	#[test]
	fn test_send_thunk_fold_right() {
		use crate::{
			brands::{
				RcFnBrand,
				SendThunkBrand,
			},
			functions::fold_right_explicit,
		};
		let thunk = SendThunk::pure(10);
		let result =
			fold_right_explicit::<RcFnBrand, SendThunkBrand, _, _, _, _>(|a, b| a + b, 5, thunk);
		assert_eq!(result, 15);
	}

	#[test]
	fn test_send_thunk_fold_left() {
		use crate::{
			brands::{
				RcFnBrand,
				SendThunkBrand,
			},
			functions::fold_left_explicit,
		};
		let thunk = SendThunk::pure(10);
		let result =
			fold_left_explicit::<RcFnBrand, SendThunkBrand, _, _, _, _>(|b, a| b + a, 5, thunk);
		assert_eq!(result, 15);
	}

	#[test]
	fn test_send_thunk_fold_map() {
		use crate::{
			brands::{
				RcFnBrand,
				SendThunkBrand,
			},
			functions::fold_map_explicit,
		};
		let thunk = SendThunk::pure(10);
		let result = fold_map_explicit::<RcFnBrand, SendThunkBrand, _, _, _, _>(
			|a: i32| a.to_string(),
			thunk,
		);
		assert_eq!(result, "10");
	}

	#[test]
	fn test_send_thunk_fold_map_with_index() {
		use crate::{
			brands::{
				RcFnBrand,
				SendThunkBrand,
			},
			classes::foldable_with_index::FoldableWithIndex,
		};
		let thunk = SendThunk::pure(5);
		let result = <SendThunkBrand as FoldableWithIndex>::fold_map_with_index::<RcFnBrand, _, _>(
			|_, x: i32| x.to_string(),
			thunk,
		);
		assert_eq!(result, "5");
	}

	#[test]
	fn test_send_thunk_foldable_with_index_receives_unit_index() {
		use crate::{
			brands::{
				RcFnBrand,
				SendThunkBrand,
			},
			classes::foldable_with_index::FoldableWithIndex,
		};
		let thunk = SendThunk::pure(42);
		let result = <SendThunkBrand as FoldableWithIndex>::fold_map_with_index::<RcFnBrand, _, _>(
			|idx, x: i32| {
				assert_eq!(idx, ());
				vec![x]
			},
			thunk,
		);
		assert_eq!(result, vec![42]);
	}

	#[test]
	fn test_send_thunk_foldable_consistency() {
		use crate::{
			brands::{
				RcFnBrand,
				SendThunkBrand,
			},
			classes::foldable_with_index::FoldableWithIndex,
			functions::fold_map_explicit,
		};
		let f = |a: i32| a.to_string();
		let t1 = SendThunk::pure(7);
		let t2 = SendThunk::pure(7);
		// fold_map(f, fa) = fold_map_with_index(|_, a| f(a), fa)
		assert_eq!(
			fold_map_explicit::<RcFnBrand, SendThunkBrand, _, _, _, _>(f, t1),
			<SendThunkBrand as FoldableWithIndex>::fold_map_with_index::<RcFnBrand, _, _>(
				|_, a| f(a),
				t2
			),
		);
	}

	// QuickCheck Law Tests

	// Functor Laws

	/// Functor identity: `send_thunk.map(identity).evaluate() == send_thunk.evaluate()`.
	#[quickcheck]
	fn functor_identity(x: i32) -> bool {
		SendThunk::pure(x).map(|a| a).evaluate() == x
	}

	/// Functor composition: `send_thunk.map(f).map(g) == send_thunk.map(|x| g(f(x)))`.
	#[quickcheck]
	fn functor_composition(x: i32) -> bool {
		let f = |a: i32| a.wrapping_add(1);
		let g = |a: i32| a.wrapping_mul(2);
		let lhs = SendThunk::pure(x).map(f).map(g).evaluate();
		let rhs = SendThunk::pure(x).map(move |a| g(f(a))).evaluate();
		lhs == rhs
	}

	// Monad Laws

	/// Monad left identity: `SendThunk::pure(a).bind(f) == f(a)`.
	#[quickcheck]
	fn monad_left_identity(a: i32) -> bool {
		let f = |x: i32| SendThunk::pure(x.wrapping_mul(2));
		let lhs = SendThunk::pure(a).bind(f).evaluate();
		let rhs = f(a).evaluate();
		lhs == rhs
	}

	/// Monad right identity: `send_thunk.bind(SendThunk::pure) == send_thunk`.
	#[quickcheck]
	fn monad_right_identity(x: i32) -> bool {
		let lhs = SendThunk::pure(x).bind(SendThunk::pure).evaluate();
		lhs == x
	}

	/// Monad associativity: `m.bind(f).bind(g) == m.bind(|x| f(x).bind(g))`.
	#[quickcheck]
	fn monad_associativity(x: i32) -> bool {
		let f = |a: i32| SendThunk::pure(a.wrapping_add(1));
		let g = |a: i32| SendThunk::pure(a.wrapping_mul(3));
		let lhs = SendThunk::pure(x).bind(f).bind(g).evaluate();
		let rhs = SendThunk::pure(x).bind(move |a| f(a).bind(g)).evaluate();
		lhs == rhs
	}
}
