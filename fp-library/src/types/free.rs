//! Implementation of the `Free` monad.
//!
//! This module provides the [`Free`] struct, which represents a free monad over a functor `F`.
//! It uses a [`CatList`] to ensure O(1) bind operations, preventing stack overflow
//! during construction of deep computations.
//!
//! ## Comparison with PureScript
//!
//! This implementation is based on the PureScript [`Control.Monad.Free`](https://github.com/purescript/purescript-free/blob/master/src/Control/Monad/Free.purs) module
//! and the ["Reflection without Remorse"](http://okmij.org/ftp/Haskell/zseq.pdf) technique. It shares the same core algorithmic properties (O(1) bind, stack safety)
//! but differs significantly in its intended use case and API surface.
//!
//! ### Key Differences
//!
//! 1. **Interpretation Strategy**:
//!    * **PureScript**: Designed as a generic Abstract Syntax Tree (AST) that can be interpreted into *any* target
//!      monad using `runFree` or `foldFree` by providing a natural transformation at runtime.
//!    * **Rust**: Designed primarily for **stack-safe execution** of computations. The interpretation logic is
//!      baked into the [`Runnable`] trait implemented by the functor `F`.
//!      The [`Free::run`] method relies on `F` knowing how to "run" itself.
//!
//! 2. **API Surface**:
//!    * **PureScript**: Rich API including `liftF`, `hoistFree`, `resume`, `foldFree`.
//!    * **Rust**: Minimal API focused on construction (`pure`, `roll`, `bind`) and execution (`run`).
//!      * `liftF` is missing (use `roll` + `map`).
//!      * `resume` is missing (cannot inspect the computation step-by-step).
//!      * `hoistFree` is missing.
//!
//! 3. **Terminology**:
//!    * Rust's `Free::roll` corresponds to PureScript's `wrap`.
//!
//! ### Capabilities and Limitations
//!
//! **What it CAN do:**
//! * Provide stack-safe recursion for monadic computations (trampolining).
//! * Prevent stack overflows when chaining many `bind` operations.
//! * Execute self-describing effects (like [`Eval`](crate::types::Thunk)).
//!
//! **What it CANNOT do (easily):**
//! * Act as a generic DSL where the interpretation is decoupled from the operation type.
//!   * *Example*: You cannot easily define a `DatabaseOp` enum and interpret it differently for
//!     production (SQL) and testing (InMemory) using this `Free` implementation, because
//!     `DatabaseOp` must implement a single `Runnable` trait.
//! * Inspect the structure of the computation (introspection) via `resume`.
//!
//! ### Lifetimes and Memory Management
//!
//! * **PureScript**: Relies on a garbage collector and `unsafeCoerce`. This allows it to ignore
//!   lifetimes and ownership, enabling a simpler implementation that supports all types.
//! * **Rust**: Relies on ownership and `Box<dyn Any>` for type erasure. `Any` requires `'static`
//!   to ensure memory safety (preventing use-after-free of references). This forces `Free` to
//!   only work with `'static` types, preventing it from implementing the library's HKT traits
//!   which require lifetime polymorphism.
//!
//! ### Examples
//!
//! ```
//! use fp_library::{brands::*, types::*};
//!
//! // ✅ CAN DO: Stack-safe recursion
//! let free = Free::<ThunkBrand, _>::pure(42)
//!     .bind(|x| Free::pure(x + 1));
//! ```

use crate::{
	Apply,
	classes::{Functor, Runnable},
	kinds::*,
	types::cat_list::CatList,
};
use std::{any::Any, marker::PhantomData};

/// A type-erased value for internal use.
///
/// This type alias represents a value whose type has been erased to `Box<dyn Any>`.
/// It is used within the internal implementation of `Free` to allow for
/// heterogeneous chains of computations in the [`CatList`].
type Val = Box<dyn Any>;

/// A type-erased continuation.
///
/// This type alias represents a function that takes a type-erased value [`Val`]
/// and returns a new `Free` computation (also type-erased).
///
/// ### Type Parameters
///
/// * `F`: The base functor.
type Cont<F> = Box<dyn FnOnce(Val) -> Free<F, Val>>;

/// The internal representation of the `Free` monad.
///
/// This enum encodes the structure of the free monad, supporting
/// pure values, suspended computations, and efficient concatenation of binds.
///
/// ### Type Parameters
///
/// * `F`: The base functor (must implement [`Functor`]).
/// * `A`: The result type.
///
enum FreeInner<F, A>
where
	F: Functor + 'static,
	A: 'static,
{
	/// A pure value.
	///
	/// This variant represents a computation that has finished and produced a value.
	Pure(A),

	/// A suspended computation.
	///
	/// This variant represents a computation that is suspended in the functor `F`.
	/// The functor contains the next step of the computation.
	Roll(Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, Free<F, A>>)),

	/// A bind operation.
	///
	/// This variant represents a computation followed by a sequence of continuations.
	/// It uses a [`CatList`] to store continuations, ensuring O(1) append complexity
	/// for left-associated binds.
	///
	/// ### Fields
	///
	/// * `head`: The initial computation.
	/// * `conts`: The list of continuations to apply to the result of `head`.
	Bind { head: Box<Free<F, Val>>, conts: CatList<Cont<F>>, _marker: PhantomData<A> },
}

/// The Free monad with O(1) bind via CatList.
///
/// This implementation follows "Reflection without Remorse" to ensure
/// that left-associated binds do not degrade performance.
///
/// # HKT and Lifetime Limitations
///
/// `Free` does not implement HKT traits (like `Functor`, `Monad`) from this library.
///
/// ## The Conflict
/// * **The Traits**: The `Kind` trait implemented by the `Functor` hierarchy requires the type
///   constructor to accept *any* lifetime `'a` (e.g., `type Of<'a, A> = Free<F, A>`).
/// * **The Implementation**: This implementation uses `Box<dyn Any>` to type-erase continuations
///   for the "Reflection without Remorse" optimization. `dyn Any` strictly requires `A: 'static`.
///
/// This creates an unresolvable conflict: `Free` cannot support non-static references (like `&'a str`),
/// so it cannot satisfy the `Kind` signature.
///
/// ## Why not use the "Naive" Recursive Definition?
///
/// A naive definition (`enum Free { Pure(A), Roll(F<Box<Free<F, A>>>) }`) would support lifetimes
/// and HKT traits. However, it was rejected because:
/// 1.  **Stack Safety**: `run` would not be stack-safe for deep computations.
/// 2.  **Performance**: `bind` would be O(N), leading to quadratic complexity for sequences of binds.
///
/// This implementation prioritizes **stack safety** and **O(1) bind** over HKT trait compatibility.
///
/// ### Type Parameters
///
/// * `F`: The base functor (must implement [`Functor`]).
/// * `A`: The result type.
///
/// ### Examples
///
/// ```
/// use fp_library::{brands::*, types::*};
///
/// let free = Free::<ThunkBrand, _>::pure(42);
/// ```
pub struct Free<F, A>(Option<FreeInner<F, A>>)
where
	F: Functor + 'static,
	A: 'static;

impl<F, A> Free<F, A>
where
	F: Functor + 'static,
	A: 'static,
{
	/// Creates a pure Free value.
	///
	/// ### Type Signature
	///
	/// `forall f a. a -> Free f a`
	///
	/// ### Parameters
	///
	/// * `a`: The value to wrap.
	///
	/// ### Returns
	///
	/// A `Free` computation that produces `a`.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, types::*};
	///
	/// let free = Free::<ThunkBrand, _>::pure(42);
	/// ```
	#[inline]
	pub fn pure(a: A) -> Self {
		Free(Some(FreeInner::Pure(a)))
	}

	/// Creates a suspended computation from a functor value.
	///
	/// ### Type Signature
	///
	/// `forall f a. f (Free f a) -> Free f a`
	///
	/// ### Parameters
	///
	/// * `fa`: The functor value containing the next step.
	///
	/// ### Returns
	///
	/// A `Free` computation that performs the effect `fa`.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, types::*};
	///
	/// let eval = Thunk::new(|| Free::pure(42));
	/// let free = Free::<ThunkBrand, _>::roll(eval);
	/// ```
	pub fn roll(
		fa: Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, Free<F, A>>)
	) -> Self {
		Free(Some(FreeInner::Roll(fa)))
	}

	/// Monadic bind (flatMap) with O(1) complexity.
	///
	/// ### Type Signature
	///
	/// `forall f b a. (a -> Free f b, Free f a) -> Free f b`
	///
	/// ### Type Parameters
	///
	/// * `B`: The result type of the new computation.
	///
	/// ### Parameters
	///
	/// * `f`: The function to apply to the result of this computation.
	///
	/// ### Returns
	///
	/// A new `Free` computation that chains `f` after this computation.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, types::*};
	///
	/// let free = Free::<ThunkBrand, _>::pure(42)
	///     .bind(|x| Free::pure(x + 1));
	/// ```
	pub fn bind<B: 'static>(
		mut self,
		f: impl FnOnce(A) -> Free<F, B> + 'static,
	) -> Free<F, B> {
		// Type-erase the continuation
		let erased_f: Cont<F> = Box::new(move |val: Val| {
			let a: A = *val.downcast().expect("Type mismatch in Free::bind");
			let free_b: Free<F, B> = f(a);
			free_b.erase_type()
		});

		// Extract inner safely
		let inner = self.0.take().expect("Free value already consumed");

		match inner {
			// Pure: create a Bind with this continuation
			FreeInner::Pure(a) => {
				let head: Free<F, Val> = Free::pure(a).erase_type();
				Free(Some(FreeInner::Bind {
					head: Box::new(head),
					conts: CatList::singleton(erased_f),
					_marker: PhantomData,
				}))
			}

			// Roll: wrap in a Bind
			FreeInner::Roll(fa) => {
				let head = Free::roll(fa).erase_type_boxed();
				Free(Some(FreeInner::Bind {
					head,
					conts: CatList::singleton(erased_f),
					_marker: PhantomData,
				}))
			}

			// Bind: snoc the new continuation onto the CatList (O(1)!)
			FreeInner::Bind { head, conts, .. } => Free(Some(FreeInner::Bind {
				head,
				conts: conts.snoc(erased_f),
				_marker: PhantomData,
			})),
		}
	}

	/// Converts to type-erased form.
	fn erase_type(mut self) -> Free<F, Val> {
		let inner = self.0.take().expect("Free value already consumed");

		match inner {
			FreeInner::Pure(a) => Free(Some(FreeInner::Pure(Box::new(a) as Val))),
			FreeInner::Roll(fa) => {
				// Map over the functor to erase the inner type
				let erased = F::map(|inner: Free<F, A>| inner.erase_type(), fa);
				Free(Some(FreeInner::Roll(erased)))
			}
			FreeInner::Bind { head, conts, .. } => {
				Free(Some(FreeInner::Bind { head, conts, _marker: PhantomData }))
			}
		}
	}

	/// Converts to boxed type-erased form.
	fn erase_type_boxed(self) -> Box<Free<F, Val>> {
		Box::new(self.erase_type())
	}

	/// Executes the Free computation, returning the final result.
	///
	/// This is the "trampoline" that iteratively processes the
	/// CatList of continuations without growing the stack.
	///
	/// ### Type Signature
	///
	/// `forall f a. Runnable f => Free f a -> a`
	///
	/// ### Returns
	///
	/// The final result of the computation.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, types::*};
	///
	/// let free = Free::<ThunkBrand, _>::pure(42);
	/// assert_eq!(free.run(), 42);
	/// ```
	pub fn run(self) -> A
	where
		F: Runnable,
	{
		// Start with a type-erased version
		let mut current: Free<F, Val> = self.erase_type();
		let mut conts: CatList<Cont<F>> = CatList::empty();

		loop {
			let inner = current.0.take().expect("Free value already consumed");

			match inner {
				FreeInner::Pure(val) => {
					// Try to apply the next continuation
					match conts.uncons() {
						Some((cont, rest)) => {
							current = cont(val);
							conts = rest;
						}
						None => {
							// No more continuations - we're done!
							return *val
								.downcast::<A>()
								.expect("Type mismatch in Free::run final downcast");
						}
					}
				}

				FreeInner::Roll(fa) => {
					// Run the effect to get the inner Free
					current = <F as Runnable>::run(fa);
				}

				FreeInner::Bind { head, conts: inner_conts, .. } => {
					// Merge the inner continuations with outer ones
					// This is where CatList's O(1) append shines!
					current = *head;
					conts = inner_conts.append(conts);
				}
			}
		}
	}
}

impl<F, A> Drop for Free<F, A>
where
	F: Functor + 'static,
	A: 'static,
{
	fn drop(&mut self) {
		// We take the inner value out.
		let inner = self.0.take();

		// If the top level is a Bind, we need to start the iterative drop chain.
		if let Some(FreeInner::Bind { mut head, .. }) = inner {
			// head is Box<Free<F, Val>>.
			// We take its inner value to continue the chain.
			// From now on, everything is typed as FreeInner<F, Val>.
			let mut current = head.0.take();

			while let Some(FreeInner::Bind { mut head, .. }) = current {
				current = head.0.take();
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{brands::ThunkBrand, types::thunk::Thunk};

	/// Tests `Free::pure`.
	///
	/// **What it tests:** Verifies that `pure` creates a computation that simply returns the provided value.
	/// **How it tests:** Constructs a `Free::pure(42)` and runs it, asserting the result is 42.
	#[test]
	fn test_free_pure() {
		let free = Free::<ThunkBrand, _>::pure(42);
		assert_eq!(free.run(), 42);
	}

	/// Tests `Free::roll`.
	///
	/// **What it tests:** Verifies that `roll` creates a computation from a suspended effect.
	/// **How it tests:** Wraps a `Free::pure(42)` inside a `Eval`, rolls it into a `Free`, and runs it to ensure it unwraps correctly.
	#[test]
	fn test_free_roll() {
		let eval = Thunk::new(|| Free::pure(42));
		let free = Free::<ThunkBrand, _>::roll(eval);
		assert_eq!(free.run(), 42);
	}

	/// Tests `Free::bind`.
	///
	/// **What it tests:** Verifies that `bind` correctly chains computations and passes values between them.
	/// **How it tests:** Chains `pure(42) -> bind(+1) -> bind(*2)` and asserts the result is (42+1)*2 = 86.
	#[test]
	fn test_free_bind() {
		let free =
			Free::<ThunkBrand, _>::pure(42).bind(|x| Free::pure(x + 1)).bind(|x| Free::pure(x * 2));
		assert_eq!(free.run(), 86);
	}

	/// Tests stack safety of `Free::run`.
	///
	/// **What it tests:** Verifies that `run` can handle deep recursion without stack overflow (trampolining).
	/// **How it tests:** Creates a recursive `count_down` function that builds a chain of 100,000 `bind` calls.
	/// If the implementation were not stack-safe, this would crash with a stack overflow.
	#[test]
	fn test_free_stack_safety() {
		fn count_down(n: i32) -> Free<ThunkBrand, i32> {
			if n == 0 { Free::pure(0) } else { Free::pure(n).bind(|n| count_down(n - 1)) }
		}

		// 100,000 iterations should overflow stack if not safe
		let free = count_down(100_000);
		assert_eq!(free.run(), 0);
	}

	/// Tests stack safety of `Free::drop`.
	///
	/// **What it tests:** Verifies that dropping a deep `Free` computation does not cause a stack overflow.
	/// **How it tests:** Constructs a deep `Free` chain (similar to `test_free_stack_safety`) and lets it go out of scope.
	#[test]
	fn test_free_drop_safety() {
		fn count_down(n: i32) -> Free<ThunkBrand, i32> {
			if n == 0 { Free::pure(0) } else { Free::pure(n).bind(|n| count_down(n - 1)) }
		}

		// Construct a deep chain but DO NOT run it.
		// When `free` goes out of scope, `Drop` should handle it iteratively.
		let _free = count_down(100_000);
	}

	/// Tests `Free::bind` on a `Roll` variant.
	///
	/// **What it tests:** Verifies that `bind` works correctly when applied to a suspended computation (`Roll`).
	/// **How it tests:** Creates a `Roll` (via `roll`) and `bind`s it.
	#[test]
	fn test_free_bind_on_roll() {
		let eval = Thunk::new(|| Free::pure(42));
		let free = Free::<ThunkBrand, _>::roll(eval).bind(|x| Free::pure(x + 1));
		assert_eq!(free.run(), 43);
	}
}
