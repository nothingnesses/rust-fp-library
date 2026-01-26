//! Implementation of the `Step` type for tail-recursive computations.
//!
//! This module provides the [`Step`] enum, which represents a single step in a tail-recursive computation.
//! It is used by the [`MonadRec`](crate::classes::monad_rec::MonadRec) trait to ensure stack safety.
//!
//! ### Examples
//!
//! ```
//! use fp_library::types::Step;
//!
//! // Count down from n to 0, accumulating the sum
//! fn sum_to_zero(n: i32, acc: i32) -> Step<(i32, i32), i32> {
//!     if n <= 0 {
//!         Step::Done(acc)
//!     } else {
//!         Step::Loop((n - 1, acc + n))
//!     }
//! }
//! ```

/// Represents the result of a single step in a tail-recursive computation.
///
/// This type is fundamental to stack-safe recursion via `MonadRec`.
///
/// ### Type Parameters
///
/// * `A`: The "loop" type - when we return `Loop(a)`, we continue with `a`.
/// * `B`: The "done" type - when we return `Done(b)`, we're finished.
///
/// ### Variants
///
/// * `Loop(A)`: Continue the loop with a new value.
/// * `Done(B)`: Finish the computation with a final value.
///
/// ### Examples
///
/// ```
/// use fp_library::types::Step;
///
/// let loop_step: Step<i32, i32> = Step::Loop(10);
/// let done_step: Step<i32, i32> = Step::Done(20);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Step<A, B> {
	/// Continue the loop with a new value
	Loop(A),
	/// Finish the computation with a final value
	Done(B),
}

impl<A, B> Step<A, B> {
	/// Returns `true` if this is a `Loop` variant.
	///
	/// ### Type Signature
	///
	/// `forall b a. Step a b -> bool`
	///
	/// ### Returns
	///
	/// `true` if the step is a loop, `false` otherwise.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::types::Step;
	///
	/// let step: Step<i32, i32> = Step::Loop(1);
	/// assert!(step.is_loop());
	/// ```
	#[inline]
	pub fn is_loop(&self) -> bool {
		matches!(self, Step::Loop(_))
	}

	/// Returns `true` if this is a `Done` variant.
	///
	/// ### Type Signature
	///
	/// `forall b a. Step a b -> bool`
	///
	/// ### Returns
	///
	/// `true` if the step is done, `false` otherwise.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::types::Step;
	///
	/// let step: Step<i32, i32> = Step::Done(1);
	/// assert!(step.is_done());
	/// ```
	#[inline]
	pub fn is_done(&self) -> bool {
		matches!(self, Step::Done(_))
	}

	/// Maps a function over the `Loop` variant.
	///
	/// ### Type Signature
	///
	/// `forall c b a. (a -> c, Step a b) -> Step c b`
	///
	/// ### Type Parameters
	///
	/// * `C`: The new loop type.
	///
	/// ### Parameters
	///
	/// * `f`: The function to apply to the loop value.
	///
	/// ### Returns
	///
	/// A new `Step` with the loop value transformed.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::types::Step;
	///
	/// let step: Step<i32, i32> = Step::Loop(1);
	/// let mapped = step.map_loop(|x| x + 1);
	/// assert_eq!(mapped, Step::Loop(2));
	/// ```
	pub fn map_loop<C>(
		self,
		f: impl FnOnce(A) -> C,
	) -> Step<C, B> {
		match self {
			Step::Loop(a) => Step::Loop(f(a)),
			Step::Done(b) => Step::Done(b),
		}
	}

	/// Maps a function over the `Done` variant.
	///
	/// ### Type Signature
	///
	/// `forall c b a. (b -> c, Step a b) -> Step a c`
	///
	/// ### Type Parameters
	///
	/// * `C`: The new done type.
	///
	/// ### Parameters
	///
	/// * `f`: The function to apply to the done value.
	///
	/// ### Returns
	///
	/// A new `Step` with the done value transformed.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::types::Step;
	///
	/// let step: Step<i32, i32> = Step::Done(1);
	/// let mapped = step.map_done(|x| x + 1);
	/// assert_eq!(mapped, Step::Done(2));
	/// ```
	pub fn map_done<C>(
		self,
		f: impl FnOnce(B) -> C,
	) -> Step<A, C> {
		match self {
			Step::Loop(a) => Step::Loop(a),
			Step::Done(b) => Step::Done(f(b)),
		}
	}

	/// Applies functions to both variants (bifunctor map).
	///
	/// ### Type Signature
	///
	/// `forall d c b a. (a -> c, b -> d, Step a b) -> Step c d`
	///
	/// ### Type Parameters
	///
	/// * `C`: The new loop type.
	/// * `D`: The new done type.
	///
	/// ### Parameters
	///
	/// * `f`: The function to apply to the loop value.
	/// * `g`: The function to apply to the done value.
	///
	/// ### Returns
	///
	/// A new `Step` with both values transformed.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::types::Step;
	///
	/// let step: Step<i32, i32> = Step::Loop(1);
	/// let mapped = step.bimap(|x| x + 1, |x| x * 2);
	/// assert_eq!(mapped, Step::Loop(2));
	/// ```
	pub fn bimap<C, D>(
		self,
		f: impl FnOnce(A) -> C,
		g: impl FnOnce(B) -> D,
	) -> Step<C, D> {
		match self {
			Step::Loop(a) => Step::Loop(f(a)),
			Step::Done(b) => Step::Done(g(b)),
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	/// Tests the `is_loop` method.
	///
	/// Verifies that `is_loop` returns true for `Loop` variants and false for `Done` variants.
	#[test]
	fn test_is_loop() {
		let step: Step<i32, i32> = Step::Loop(1);
		assert!(step.is_loop());
		assert!(!step.is_done());
	}

	/// Tests the `is_done` method.
	///
	/// Verifies that `is_done` returns true for `Done` variants and false for `Loop` variants.
	#[test]
	fn test_is_done() {
		let step: Step<i32, i32> = Step::Done(1);
		assert!(step.is_done());
		assert!(!step.is_loop());
	}

	/// Tests the `map_loop` method.
	///
	/// Verifies that `map_loop` transforms the value inside a `Loop` variant and leaves a `Done` variant unchanged.
	#[test]
	fn test_map_loop() {
		let step: Step<i32, i32> = Step::Loop(1);
		let mapped = step.map_loop(|x| x + 1);
		assert_eq!(mapped, Step::Loop(2));

		let done: Step<i32, i32> = Step::Done(1);
		let mapped_done = done.map_loop(|x| x + 1);
		assert_eq!(mapped_done, Step::Done(1));
	}

	/// Tests the `map_done` method.
	///
	/// Verifies that `map_done` transforms the value inside a `Done` variant and leaves a `Loop` variant unchanged.
	#[test]
	fn test_map_done() {
		let step: Step<i32, i32> = Step::Done(1);
		let mapped = step.map_done(|x| x + 1);
		assert_eq!(mapped, Step::Done(2));

		let loop_step: Step<i32, i32> = Step::Loop(1);
		let mapped_loop = loop_step.map_done(|x| x + 1);
		assert_eq!(mapped_loop, Step::Loop(1));
	}

	/// Tests the `bimap` method.
	///
	/// Verifies that `bimap` transforms the value inside both `Loop` and `Done` variants using the appropriate function.
	#[test]
	fn test_bimap() {
		let step: Step<i32, i32> = Step::Loop(1);
		let mapped = step.bimap(|x| x + 1, |x| x * 2);
		assert_eq!(mapped, Step::Loop(2));

		let done: Step<i32, i32> = Step::Done(1);
		let mapped_done = done.bimap(|x| x + 1, |x| x * 2);
		assert_eq!(mapped_done, Step::Done(2));
	}
}
