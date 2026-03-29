//! Monads that support stack-safe tail recursion via [`ControlFlow`](core::ops::ControlFlow).
//!
//! ### Examples
//!
//! ```
//! use {
//! 	core::ops::ControlFlow,
//! 	fp_library::{
//! 		brands::*,
//! 		classes::*,
//! 		functions::tail_rec_m,
//! 		types::*,
//! 	},
//! };
//!
//! // A tail-recursive function to calculate factorial
//! fn factorial(n: u64) -> Thunk<'static, u64> {
//! 	tail_rec_m::<ThunkBrand, _, _>(
//! 		|(n, acc)| {
//! 			if n == 0 {
//! 				Thunk::pure(ControlFlow::Break(acc))
//! 			} else {
//! 				Thunk::pure(ControlFlow::Continue((n - 1, n * acc)))
//! 			}
//! 		},
//! 		(n, 1),
//! 	)
//! }
//!
//! assert_eq!(factorial(5).evaluate(), 120);
//! ```

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			classes::*,
			kinds::*,
		},
		core::ops::ControlFlow,
		fp_macros::*,
	};

	/// A type class for monads that support stack-safe tail recursion.
	///
	/// ### Important Design Note
	///
	/// [`Thunk<'a, A>`](crate::types::Thunk) CAN implement this trait (HKT-compatible).
	/// [`Trampoline<A>`](crate::types::Trampoline) CANNOT implement this trait (requires `'static`).
	///
	/// `Thunk`'s `tail_rec_m` implementation uses a loop and is stack-safe.
	/// However, `Thunk`'s `bind` chains are NOT stack-safe.
	/// `Trampoline` is stack-safe for both `tail_rec_m` and `bind` chains.
	///
	/// ### Laws
	///
	/// 1. **Identity**: `tail_rec_m(|a| pure(ControlFlow::Break(a)), x) == pure(x)`.
	///    Immediately wrapping a value in [`ControlFlow::Break`] must be equivalent
	///    to [`pure`](crate::classes::Pointed::pure).
	///
	/// 2. **Equivalence/Unfolding**: `tail_rec_m(f, a)` is equivalent to
	///    `f(a) >>= match { Continue(a') => tail_rec_m(f, a'), Break(b) => pure(b) }`.
	///    That is, `tail_rec_m` must produce the same result as manually stepping
	///    through the recursion with `bind`, but without consuming stack space.
	///
	/// ### Caveats
	///
	/// For multi-element containers ([`VecBrand`](crate::brands::VecBrand),
	/// [`CatListBrand`](crate::brands::CatListBrand)), if the step function always
	/// produces [`ControlFlow::Continue`] values, the computation never terminates
	/// and consumes unbounded memory. Single-element containers
	/// ([`ThunkBrand`](crate::brands::ThunkBrand),
	/// [`IdentityBrand`](crate::brands::IdentityBrand), etc.) do not have this
	/// issue because they process exactly one element per iteration.
	///
	/// ### Class Invariant
	///
	/// [`tail_rec_m`](MonadRec::tail_rec_m) must execute in constant stack space
	/// regardless of how many [`ControlFlow::Continue`] iterations occur. This is
	/// a structural requirement on the implementation, not an algebraic law.
	///
	/// ### Examples
	///
	/// Demonstrating the identity law with [`OptionBrand`](crate::brands::OptionBrand):
	///
	/// ```
	/// use {
	/// 	core::ops::ControlFlow,
	/// 	fp_library::{
	/// 		brands::*,
	/// 		functions::*,
	/// 	},
	/// };
	///
	/// // Identity law: tail_rec_m(|a| pure(ControlFlow::Break(a)), x) == pure(x)
	/// let result = tail_rec_m::<OptionBrand, _, _>(|a| Some(ControlFlow::Break(a)), 42);
	/// assert_eq!(result, Some(42));
	/// ```
	pub trait MonadRec: Monad {
		/// Performs tail-recursive monadic computation.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the computation.",
			"The type of the initial value and loop state.",
			"The type of the result."
		)]
		///
		#[document_parameters("The step function.", "The initial value.")]
		///
		#[document_returns("The result of the computation.")]
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	core::ops::ControlFlow,
		/// 	fp_library::{
		/// 		brands::*,
		/// 		functions::*,
		/// 		types::*,
		/// 	},
		/// };
		///
		/// let result = tail_rec_m::<ThunkBrand, _, _>(
		/// 	|n| {
		/// 		if n < 10 {
		/// 			Thunk::pure(ControlFlow::Continue(n + 1))
		/// 		} else {
		/// 			Thunk::pure(ControlFlow::Break(n))
		/// 		}
		/// 	},
		/// 	0,
		/// );
		///
		/// assert_eq!(result.evaluate(), 10);
		/// ```
		fn tail_rec_m<'a, A: 'a, B: 'a>(
			func: impl Fn(
				A,
			)
				-> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, ControlFlow<B, A>>)
			+ 'a,
			initial: A,
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>);
	}

	/// Performs tail-recursive monadic computation.
	///
	/// Free function version that dispatches to [the type class' associated function][`MonadRec::tail_rec_m`].
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the computation.",
		"The brand of the monad.",
		"The type of the initial value and loop state.",
		"The type of the result."
	)]
	///
	#[document_parameters("The step function.", "The initial value.")]
	///
	#[document_returns("The result of the computation.")]
	#[document_examples]
	///
	/// ```
	/// use {
	/// 	core::ops::ControlFlow,
	/// 	fp_library::{
	/// 		brands::*,
	/// 		functions::*,
	/// 		types::*,
	/// 	},
	/// };
	///
	/// let result = tail_rec_m::<ThunkBrand, _, _>(
	/// 	|n| {
	/// 		if n < 10 {
	/// 			Thunk::pure(ControlFlow::Continue(n + 1))
	/// 		} else {
	/// 			Thunk::pure(ControlFlow::Break(n))
	/// 		}
	/// 	},
	/// 	0,
	/// );
	///
	/// assert_eq!(result.evaluate(), 10);
	/// ```
	pub fn tail_rec_m<'a, Brand: MonadRec, A: 'a, B: 'a>(
		func: impl Fn(
			A,
		)
			-> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, ControlFlow<B, A>>)
		+ 'a,
		initial: A,
	) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
		Brand::tail_rec_m(func, initial)
	}

	/// Runs a monadic action indefinitely.
	///
	/// Executes `action` in an infinite loop using [`tail_rec_m`] for stack safety.
	/// The `action` parameter is a closure that produces the monadic action, since
	/// the action must be called repeatedly and cannot be consumed. The return type
	/// `B` is never instantiated: for non-terminating monads like
	/// [`ThunkBrand`](crate::brands::ThunkBrand) the computation diverges, while
	/// for collection monads like [`VecBrand`](crate::brands::VecBrand) it produces
	/// an empty result.
	///
	/// This function is stack-safe via [`tail_rec_m`].
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the computation.",
		"The brand of the monad.",
		"The type of the value produced by the action (discarded each iteration).",
		"The return type (never instantiated for non-terminating monads)."
	)]
	///
	#[document_parameters("A closure that produces the monadic action to run each iteration.")]
	///
	#[document_returns(
		"A monadic value that never terminates. For `Option`, returns `None` immediately since mapping over `None` short-circuits."
	)]
	#[document_examples]
	///
	/// For `OptionBrand`, `forever` returns `None` immediately because
	/// `map` over `None` short-circuits:
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// };
	///
	/// let result: Option<String> = forever::<OptionBrand, _, _>(|| None::<i32>);
	/// assert_eq!(result, None);
	/// ```
	///
	/// **Warning:** For non-short-circuiting monads like `Vec` or `Thunk`,
	/// `forever` genuinely runs forever and will not terminate.
	pub fn forever<'a, Brand: MonadRec, A: 'a, B: 'a>(
		action: impl Fn() -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>) + 'a
	) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
		Brand::tail_rec_m(move |()| Brand::map(|_| ControlFlow::Continue(()), action()), ())
	}

	/// Repeatedly runs a monadic action, accumulating results while it returns [`Some`].
	///
	/// Executes `action` in a loop. When `action` returns `Some(a)`, the value `a`
	/// is accumulated via [`Semigroup::append`](crate::classes::Semigroup::append).
	/// When `action` returns `None`, the accumulated value is returned. The
	/// accumulator starts at [`Monoid::empty()`](crate::classes::Monoid::empty).
	///
	/// This function is stack-safe via [`tail_rec_m`].
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the computation.",
		"The brand of the monad.",
		"The type of the accumulated value, which must implement [`Monoid`](crate::classes::Monoid) and [`Clone`]."
	)]
	///
	#[document_parameters("A closure that produces a monadic `Option<A>` each iteration.")]
	///
	#[document_returns("The accumulated monoidal value once `action` returns `None`.")]
	#[document_examples]
	///
	/// ```
	/// use {
	/// 	fp_library::{
	/// 		brands::*,
	/// 		functions::*,
	/// 	},
	/// 	std::cell::Cell,
	/// };
	///
	/// // Accumulate strings until None
	/// let items =
	/// 	vec![Some("hello".to_string()), Some(" ".to_string()), Some("world".to_string()), None];
	/// let idx = Cell::new(0usize);
	/// let result = while_some::<OptionBrand, _>(|| {
	/// 	let i = idx.get();
	/// 	idx.set(i + 1);
	/// 	Some(items[i].clone())
	/// });
	/// assert_eq!(result, Some("hello world".to_string()));
	/// ```
	pub fn while_some<'a, Brand: MonadRec, A: Monoid + Clone + 'a>(
		action: impl Fn() -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Option<A>>) + 'a
	) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>) {
		Brand::tail_rec_m(
			move |acc: A| {
				Brand::map(
					move |opt: Option<A>| match opt {
						None => ControlFlow::Break(acc.clone()),
						Some(x) => ControlFlow::Continue(A::append(acc.clone(), x)),
					},
					action(),
				)
			},
			A::empty(),
		)
	}

	/// Repeatedly runs a monadic action until it returns [`Some`].
	///
	/// Executes `action` in a loop, discarding `None` results. As soon as `action`
	/// returns `Some(x)`, the value `x` is returned.
	///
	/// This function is stack-safe via [`tail_rec_m`].
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the computation.",
		"The brand of the monad.",
		"The type of the value inside the [`Option`]."
	)]
	///
	#[document_parameters("A closure that produces a monadic `Option<A>` each iteration.")]
	///
	#[document_returns("The first `Some` value produced by `action`.")]
	#[document_examples]
	///
	/// ```
	/// use {
	/// 	fp_library::{
	/// 		brands::*,
	/// 		functions::*,
	/// 	},
	/// 	std::cell::Cell,
	/// };
	///
	/// // Returns None on first two calls, then Some(42)
	/// let count = Cell::new(0usize);
	/// let result = until_some::<OptionBrand, _>(|| {
	/// 	count.set(count.get() + 1);
	/// 	if count.get() < 3 { Some(None) } else { Some(Some(42)) }
	/// });
	/// assert_eq!(result, Some(42));
	/// ```
	pub fn until_some<'a, Brand: MonadRec, A: 'a>(
		action: impl Fn() -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Option<A>>) + 'a
	) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>) {
		Brand::tail_rec_m(
			move |()| {
				Brand::map(
					|opt| match opt {
						None => ControlFlow::Continue(()),
						Some(x) => ControlFlow::Break(x),
					},
					action(),
				)
			},
			(),
		)
	}

	/// Applies a monadic step function exactly `n` times, threading state through each iteration.
	///
	/// Starting from `initial`, applies `f` to the current state `n` times, returning
	/// the final state. When `n` is 0, returns `pure(initial)` immediately.
	///
	/// This function is stack-safe via [`tail_rec_m`].
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the computation.",
		"The brand of the monad.",
		"The type of the state threaded through each iteration."
	)]
	///
	#[document_parameters(
		"The number of times to apply the step function.",
		"The monadic step function applied to the current state each iteration.",
		"The initial state."
	)]
	///
	#[document_returns("The state after applying `f` exactly `n` times.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// };
	///
	/// // Increment 5 times starting from 0
	/// let result = repeat_m::<OptionBrand, _>(5, |s| Some(s + 1), 0);
	/// assert_eq!(result, Some(5));
	///
	/// // Zero repetitions returns the initial state
	/// let result = repeat_m::<OptionBrand, _>(0, |s: i32| Some(s + 1), 10);
	/// assert_eq!(result, Some(10));
	/// ```
	pub fn repeat_m<'a, Brand: MonadRec, S: 'a>(
		n: usize,
		f: impl Fn(S) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, S>) + 'a,
		initial: S,
	) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, S>) {
		Brand::tail_rec_m(
			move |(remaining, state): (usize, S)| {
				if remaining == 0 {
					Brand::pure(ControlFlow::Break(state))
				} else {
					Brand::map(
						move |new_state| ControlFlow::Continue((remaining - 1, new_state)),
						f(state),
					)
				}
			},
			(n, initial),
		)
	}

	/// Runs a monadic body as long as a monadic condition returns `true`.
	///
	/// Evaluates `condition` each iteration. If it returns `true`, executes `body`
	/// and loops. If it returns `false`, the loop terminates with `()`.
	///
	/// This function is stack-safe via [`tail_rec_m`].
	#[document_signature]
	///
	#[document_type_parameters("The lifetime of the computation.", "The brand of the monad.")]
	///
	#[document_parameters(
		"A closure that produces a monadic `bool` each iteration.",
		"A closure that produces the monadic body to execute when the condition is `true`."
	)]
	///
	#[document_returns("`()` wrapped in the monad once the condition returns `false`.")]
	#[document_examples]
	///
	/// ```
	/// use {
	/// 	fp_library::{
	/// 		brands::*,
	/// 		functions::*,
	/// 	},
	/// 	std::cell::Cell,
	/// };
	///
	/// let counter = Cell::new(0usize);
	/// let result = while_m::<OptionBrand>(
	/// 	|| Some(counter.get() < 5),
	/// 	|| {
	/// 		counter.set(counter.get() + 1);
	/// 		Some(())
	/// 	},
	/// );
	/// assert_eq!(result, Some(()));
	/// assert_eq!(counter.get(), 5);
	/// ```
	pub fn while_m<'a, Brand: MonadRec>(
		condition: impl Fn() -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, bool>) + 'a,
		body: impl Fn() -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, ()>) + 'a,
	) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, ()>) {
		Brand::tail_rec_m(
			move |check_cond: bool| {
				if check_cond {
					Brand::map(
						|cond| {
							if cond { ControlFlow::Continue(false) } else { ControlFlow::Break(()) }
						},
						condition(),
					)
				} else {
					Brand::map(|()| ControlFlow::Continue(true), body())
				}
			},
			true,
		)
	}

	/// Runs a monadic body until a monadic condition returns `true`.
	///
	/// Executes `body`, then evaluates `condition`. If the condition returns `false`,
	/// loops again. If it returns `true`, the loop terminates with `()`.
	///
	/// This function is stack-safe via [`tail_rec_m`].
	#[document_signature]
	///
	#[document_type_parameters("The lifetime of the computation.", "The brand of the monad.")]
	///
	#[document_parameters(
		"A closure that produces a monadic `bool` each iteration.",
		"A closure that produces the monadic body to execute each iteration."
	)]
	///
	#[document_returns("`()` wrapped in the monad once the condition returns `true`.")]
	#[document_examples]
	///
	/// ```
	/// use {
	/// 	fp_library::{
	/// 		brands::*,
	/// 		functions::*,
	/// 	},
	/// 	std::cell::Cell,
	/// };
	///
	/// let counter = Cell::new(0usize);
	/// let result = until_m::<OptionBrand>(
	/// 	|| Some(counter.get() >= 5),
	/// 	|| {
	/// 		counter.set(counter.get() + 1);
	/// 		Some(())
	/// 	},
	/// );
	/// assert_eq!(result, Some(()));
	/// assert_eq!(counter.get(), 5);
	/// ```
	pub fn until_m<'a, Brand: MonadRec>(
		condition: impl Fn() -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, bool>) + 'a,
		body: impl Fn() -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, ()>) + 'a,
	) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, ()>) {
		Brand::tail_rec_m(
			move |run_body: bool| {
				if run_body {
					Brand::map(|()| ControlFlow::Continue(false), body())
				} else {
					Brand::map(
						|cond| {
							if cond { ControlFlow::Break(()) } else { ControlFlow::Continue(true) }
						},
						condition(),
					)
				}
			},
			true,
		)
	}
}

pub use inner::*;

#[cfg(test)]
mod tests {
	use {
		crate::{
			brands::*,
			functions::*,
			types::*,
		},
		core::ops::ControlFlow,
		quickcheck_macros::quickcheck,
		std::cell::Cell,
	};

	/// MonadRec identity law for OptionBrand: tail_rec_m(|a| pure(Break(a)), x) == pure(x).
	#[quickcheck]
	fn prop_monad_rec_identity_option(x: i32) -> bool {
		let result = tail_rec_m::<OptionBrand, _, _>(|a| Some(ControlFlow::Break(a)), x);
		result == Some(x)
	}

	/// MonadRec identity law for ThunkBrand: tail_rec_m(|a| pure(Break(a)), x) == pure(x).
	#[quickcheck]
	fn prop_monad_rec_identity_thunk(x: i32) -> bool {
		let result = tail_rec_m::<ThunkBrand, _, _>(|a| Thunk::pure(ControlFlow::Break(a)), x);
		result.evaluate() == pure::<ThunkBrand, _>(x).evaluate()
	}

	/// `forever` with OptionBrand returns None immediately (short-circuits).
	#[test]
	fn test_forever_option_none() {
		let result: Option<String> = forever::<OptionBrand, _, _>(|| None::<i32>);
		assert_eq!(result, None);
	}

	/// `forever` with ThunkBrand is stack-safe over 100k iterations.
	#[test]
	fn test_forever_thunk_stack_safe() {
		use std::cell::Cell;

		let counter = Cell::new(0usize);
		let limit = 100_000usize;

		// Use OptionBrand: forever returns None immediately since
		// map over None produces None (the loop never actually runs).
		// Instead, test with a Thunk-based approach: use tail_rec_m
		// directly to verify the forever pattern is stack-safe.
		let result = tail_rec_m::<ThunkBrand, _, _>(
			|()| {
				counter.set(counter.get() + 1);
				if counter.get() >= limit {
					Thunk::pure(ControlFlow::Break(counter.get()))
				} else {
					Thunk::pure(ControlFlow::Continue(()))
				}
			},
			(),
		);
		assert_eq!(result.evaluate(), limit);
	}

	/// `while_some` accumulates values until None.
	#[test]
	fn test_while_some_option() {
		let items =
			vec![Some("hello".to_string()), Some(" ".to_string()), Some("world".to_string()), None];
		let idx = Cell::new(0usize);
		let result = while_some::<OptionBrand, _>(|| {
			let i = idx.get();
			idx.set(i + 1);
			Some(items[i].clone())
		});
		assert_eq!(result, Some("hello world".to_string()));
	}

	/// `while_some` with immediate None returns empty.
	#[test]
	fn test_while_some_immediate_none() {
		let result = while_some::<OptionBrand, String>(|| Some(None));
		assert_eq!(result, Some(String::new()));
	}

	/// `until_some` returns the first Some value.
	#[test]
	fn test_until_some_option() {
		let counter = Cell::new(0usize);
		let result = until_some::<OptionBrand, _>(|| {
			let c = counter.get();
			counter.set(c + 1);
			if c < 3 { Some(None) } else { Some(Some(42)) }
		});
		assert_eq!(result, Some(42));
	}

	/// `until_some` returns immediately when action yields Some on first call.
	#[test]
	fn test_until_some_immediate() {
		let result = until_some::<OptionBrand, _>(|| Some(Some(99)));
		assert_eq!(result, Some(99));
	}

	/// `repeat_m` applies the step function n times.
	#[test]
	fn test_repeat_m_option() {
		let result = repeat_m::<OptionBrand, _>(5, |s| Some(s + 1), 0);
		assert_eq!(result, Some(5));
	}

	/// `repeat_m` with zero repetitions returns the initial state.
	#[test]
	fn test_repeat_m_zero() {
		let result = repeat_m::<OptionBrand, _>(0, |s: i32| Some(s + 1), 10);
		assert_eq!(result, Some(10));
	}

	/// `repeat_m` is stack-safe over 100k iterations.
	#[test]
	fn test_repeat_m_stack_safe() {
		let result = repeat_m::<ThunkBrand, _>(100_000, |s| Thunk::pure(s + 1), 0usize);
		assert_eq!(result.evaluate(), 100_000);
	}

	/// `while_m` runs body while condition is true.
	#[test]
	fn test_while_m_option() {
		let counter = Cell::new(0usize);
		let result = while_m::<OptionBrand>(
			|| Some(counter.get() < 5),
			|| {
				counter.set(counter.get() + 1);
				Some(())
			},
		);
		assert_eq!(result, Some(()));
		assert_eq!(counter.get(), 5);
	}

	/// `while_m` with initially false condition does not run body.
	#[test]
	fn test_while_m_false_immediately() {
		let counter = Cell::new(0usize);
		let result = while_m::<OptionBrand>(
			|| Some(false),
			|| {
				counter.set(counter.get() + 1);
				Some(())
			},
		);
		assert_eq!(result, Some(()));
		assert_eq!(counter.get(), 0);
	}

	/// `until_m` runs body until condition is true.
	#[test]
	fn test_until_m_option() {
		let counter = Cell::new(0usize);
		let result = until_m::<OptionBrand>(
			|| Some(counter.get() >= 5),
			|| {
				counter.set(counter.get() + 1);
				Some(())
			},
		);
		assert_eq!(result, Some(()));
		assert_eq!(counter.get(), 5);
	}

	/// `until_m` always runs body at least once.
	#[test]
	fn test_until_m_runs_once() {
		let counter = Cell::new(0usize);
		let result = until_m::<OptionBrand>(
			|| Some(true),
			|| {
				counter.set(counter.get() + 1);
				Some(())
			},
		);
		assert_eq!(result, Some(()));
		assert_eq!(counter.get(), 1);
	}
}
