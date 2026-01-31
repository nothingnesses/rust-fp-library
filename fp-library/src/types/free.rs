use crate::{
	Apply,
	classes::{Evaluable, Functor},
	kinds::*,
	types::CatList,
};
use std::{any::Any, marker::PhantomData};

/// A type-erased value for internal use.
///
/// This type alias represents a value whose type has been erased to [`Box<dyn Any>`].
/// It is used within the internal implementation of [`Free`] to allow for
/// heterogeneous chains of computations in the [`CatList`].
type TypeErasedValue = Box<dyn Any>;

/// A type-erased continuation.
///
/// This type alias represents a function that takes a [`TypeErasedValue`]
/// and returns a new [`Free`] computation (also type-erased).
///
/// ### Type Parameters
///
/// * `F`: The base functor.
type Continuation<F> = Box<dyn FnOnce(TypeErasedValue) -> Free<F, TypeErasedValue>>;

/// The internal representation of the [`Free`] monad.
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
	Wrap(Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, Free<F, A>>)),

	/// A bind operation.
	///
	/// This variant represents a computation followed by a sequence of continuations.
	/// It uses a [`CatList`] to store continuations, ensuring O(1) append complexity
	/// for left-associated binds.
	///
	/// ### Fields
	///
	/// * `head`: The initial computation.
	/// * `continuations`: The list of continuations to apply to the result of `head`.
	Bind {
		head: Box<Free<F, TypeErasedValue>>,
		continuations: CatList<Continuation<F>>,
		_marker: PhantomData<A>,
	},
}

/// The Free monad with O(1) bind via [`CatList`].
///
/// This implementation follows ["Reflection without Remorse"](http://okmij.org/ftp/Haskell/zseq.pdf) to ensure
/// that left-associated binds do not degrade performance.
///
/// # HKT and Lifetime Limitations
///
/// `Free` does not implement HKT traits (like `Functor`, `Monad`) from this library.
///
/// ## The Conflict
/// * **The Traits**: The `Kind` trait implemented by the `Functor` hierarchy requires the type
///   constructor to accept *any* lifetime `'a` (e.g., `type Of<'a, A> = Free<F, A>`).
/// * **The Implementation**: This implementation uses [`Box<dyn Any>`] to type-erase continuations
///   for the "Reflection without Remorse" optimization. `dyn Any` strictly requires `A: 'static`.
///
/// This creates an unresolvable conflict: `Free` cannot support non-static references (like `&'a str`),
/// so it cannot satisfy the `Kind` signature.
///
/// ## Why not use the "Naive" Recursive Definition?
///
/// A naive definition (`enum Free { Pure(A), Wrap(F<Box<Free<F, A>>>) }`) would support lifetimes
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
	/// Creates a pure `Free` value.
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
	/// let free = Free::<ThunkBrand, _>::wrap(eval);
	/// ```
	pub fn wrap(
		fa: Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, Free<F, A>>)
	) -> Self {
		Free(Some(FreeInner::Wrap(fa)))
	}

	/// Lifts a functor value into the Free monad.
	///
	/// This is the primary way to inject effects into Free monad computations.
	/// Equivalent to PureScript's `liftF` and Haskell's `liftF`.
	///
	/// ### Type Signature
	///
	/// `forall f a. Functor f => f a -> Free f a`
	///
	/// ### Implementation
	///
	/// ```text
	/// liftF fa = wrap (map pure fa)
	/// ```
	///
	/// ### Parameters
	///
	/// * `fa`: The functor value to lift.
	///
	/// ### Returns
	///
	/// A `Free` computation that performs the effect and returns the result.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, types::*};
	///
	/// // Lift a simple computation
	/// let thunk = Thunk::new(|| 42);
	/// let free = Free::<ThunkBrand, _>::lift_f(thunk);
	/// assert_eq!(free.evaluate(), 42);
	///
	/// // Build a computation from raw effects
	/// let computation = Free::lift_f(Thunk::new(|| 10))
	///     .bind(|x| Free::lift_f(Thunk::new(move || x * 2)))
	///     .bind(|x| Free::lift_f(Thunk::new(move || x + 5)));
	/// assert_eq!(computation.evaluate(), 25);
	/// ```
	pub fn lift_f(fa: Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, A>)) -> Self {
		// Map the value to a pure Free, then wrap it
		Free::wrap(F::map(Free::pure, fa))
	}

	/// Monadic bind with O(1) complexity.
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
		let erased_f: Continuation<F> = Box::new(move |val: TypeErasedValue| {
			let a: A = *val.downcast().expect("Type mismatch in Free::bind");
			let free_b: Free<F, B> = f(a);
			free_b.erase_type()
		});

		// Extract inner safely
		let inner = self.0.take().expect("Free value already consumed");

		match inner {
			// Pure: create a Bind with this continuation
			FreeInner::Pure(a) => {
				let head: Free<F, TypeErasedValue> = Free::pure(a).erase_type();
				Free(Some(FreeInner::Bind {
					head: Box::new(head),
					continuations: CatList::singleton(erased_f),
					_marker: PhantomData,
				}))
			}

			// Wrap: wrap in a Bind
			FreeInner::Wrap(fa) => {
				let head = Free::wrap(fa).boxed_erase_type();
				Free(Some(FreeInner::Bind {
					head,
					continuations: CatList::singleton(erased_f),
					_marker: PhantomData,
				}))
			}

			// Bind: snoc the new continuation onto the CatList (O(1)!)
			FreeInner::Bind { head, continuations: conts, .. } => Free(Some(FreeInner::Bind {
				head,
				continuations: conts.snoc(erased_f),
				_marker: PhantomData,
			})),
		}
	}

	/// Converts to type-erased form.
	fn erase_type(mut self) -> Free<F, TypeErasedValue> {
		let inner = self.0.take().expect("Free value already consumed");

		match inner {
			FreeInner::Pure(a) => Free(Some(FreeInner::Pure(Box::new(a) as TypeErasedValue))),
			FreeInner::Wrap(fa) => {
				// Map over the functor to erase the inner type
				let erased = F::map(|inner: Free<F, A>| inner.erase_type(), fa);
				Free(Some(FreeInner::Wrap(erased)))
			}
			FreeInner::Bind { head, continuations, .. } => {
				Free(Some(FreeInner::Bind { head, continuations, _marker: PhantomData }))
			}
		}
	}

	/// Converts to boxed type-erased form.
	fn boxed_erase_type(self) -> Box<Free<F, TypeErasedValue>> {
		Box::new(self.erase_type())
	}

	/// Executes the Free computation, returning the final result.
	///
	/// This is the "trampoline" that iteratively processes the
	/// [`CatList`] of continuations without growing the stack.
	///
	/// ### Type Signature
	///
	/// `forall f a. Evaluable f => Free f a -> a`
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
	/// assert_eq!(free.evaluate(), 42);
	/// ```
	pub fn evaluate(self) -> A
	where
		F: Evaluable,
	{
		// Start with a type-erased version
		let mut current: Free<F, TypeErasedValue> = self.erase_type();
		let mut continuations: CatList<Continuation<F>> = CatList::empty();

		loop {
			let inner = current.0.take().expect("Free value already consumed");

			match inner {
				FreeInner::Pure(val) => {
					// Try to apply the next continuation
					match continuations.uncons() {
						Some((continuation, rest)) => {
							current = continuation(val);
							continuations = rest;
						}
						None => {
							// No more continuations - we're done!
							return *val
								.downcast::<A>()
								.expect("Type mismatch in Free::evaluate final downcast");
						}
					}
				}

				FreeInner::Wrap(fa) => {
					// Run the effect to get the inner Free
					current = <F as Evaluable>::evaluate(fa);
				}

				FreeInner::Bind { head, continuations: inner_continuations, .. } => {
					// Merge the inner continuations with outer ones
					// This is where CatList's O(1) append shines!
					current = *head;
					continuations = inner_continuations.append(continuations);
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
			// head is Box<Free<F, TypeEraseValue>>.
			// We take its inner value to continue the chain.
			// From now on, everything is typed as FreeInner<F, TypeEraseValue>.
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
		assert_eq!(free.evaluate(), 42);
	}

	/// Tests `Free::roll`.
	///
	/// **What it tests:** Verifies that `roll` creates a computation from a suspended effect.
	/// **How it tests:** Wraps a `Free::pure(42)` inside a `Thunk`, rolls it into a `Free`, and runs it to ensure it unwraps correctly.
	#[test]
	fn test_free_roll() {
		let eval = Thunk::new(|| Free::pure(42));
		let free = Free::<ThunkBrand, _>::wrap(eval);
		assert_eq!(free.evaluate(), 42);
	}

	/// Tests `Free::bind`.
	///
	/// **What it tests:** Verifies that `bind` correctly chains computations and passes values between them.
	/// **How it tests:** Chains `pure(42) -> bind(+1) -> bind(*2)` and asserts the result is (42+1)*2 = 86.
	#[test]
	fn test_free_bind() {
		let free =
			Free::<ThunkBrand, _>::pure(42).bind(|x| Free::pure(x + 1)).bind(|x| Free::pure(x * 2));
		assert_eq!(free.evaluate(), 86);
	}

	/// Tests stack safety of `Free::evaluate`.
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
		assert_eq!(free.evaluate(), 0);
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

	/// Tests `Free::bind` on a `Wrap` variant.
	///
	/// **What it tests:** Verifies that `bind` works correctly when applied to a suspended computation (`Wrap`).
	/// **How it tests:** Creates a `Wrap` (via `wrap`) and `bind`s it.
	#[test]
	fn test_free_bind_on_wrap() {
		let eval = Thunk::new(|| Free::pure(42));
		let free = Free::<ThunkBrand, _>::wrap(eval).bind(|x| Free::pure(x + 1));
		assert_eq!(free.evaluate(), 43);
	}

	/// Tests `Free::lift_f`.
	///
	/// **What it tests:** Verifies that `lift_f` correctly lifts a functor value into the Free monad.
	/// **How it tests:** Lifts a simple thunk and verifies the result.
	#[test]
	fn test_free_lift_f() {
		let thunk = Thunk::new(|| 42);
		let free = Free::<ThunkBrand, _>::lift_f(thunk);
		assert_eq!(free.evaluate(), 42);
	}

	/// Tests `Free::lift_f` with bind.
	///
	/// **What it tests:** Verifies that `lift_f` can be used to build computations with `bind`.
	/// **How it tests:** Chains multiple `lift_f` calls with `bind`.
	#[test]
	fn test_free_lift_f_with_bind() {
		let computation = Free::<ThunkBrand, _>::lift_f(Thunk::new(|| 10))
			.bind(|x| Free::<ThunkBrand, _>::lift_f(Thunk::new(move || x * 2)))
			.bind(|x| Free::<ThunkBrand, _>::lift_f(Thunk::new(move || x + 5)));
		assert_eq!(computation.evaluate(), 25);
	}
}
