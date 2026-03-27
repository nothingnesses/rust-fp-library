//! Stack-safe Free monad over a functor with O(1) [`bind`](crate::functions::bind) operations.
//!
//! Enables building computation chains without stack overflow by using a catenable list of continuations. Note: requires `'static` types and cannot implement the library's HKT traits due to type erasure.
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
//!      baked into the [`Evaluable`](crate::classes::Evaluable) trait implemented by the functor `F`.
//!      The [`Free::wrap`] method wraps a functor layer containing a Free computation.
//!
//! 2. **API Surface**:
//!    * **PureScript**: Rich API including `liftF`, `hoistFree`, `resume`, `foldFree`.
//!    * **Rust**: Focused API with construction (`pure`, `wrap`, `lift_f`, `bind`), execution (`evaluate`),
//!      introspection (`resume`), and interpretation (`fold_free`).
//!      * `hoistFree` is missing.
//!
//! 3. **Terminology**:
//!    * Rust's `Free::wrap` corresponds to PureScript's `wrap`.
//!
//! ### Capabilities and Limitations
//!
//! **What it CAN do:**
//! * Provide stack-safe recursion for monadic computations (trampolining).
//! * Prevent stack overflows when chaining many `bind` operations.
//! * Execute self-describing effects (like [`Thunk`](crate::types::Thunk)).
//!
//! **What it CANNOT do (easily):**
//! * Act as a generic DSL where the interpretation is decoupled from the operation type.
//!   * *Example*: You cannot easily define a `DatabaseOp` enum and interpret it differently for
//!     production (SQL) and testing (InMemory) using this `Free` implementation, because
//!     `DatabaseOp` must implement a single `Evaluable` trait.
//!   * Note: `fold_free` with `NaturalTransformation` does support this pattern for simple cases.
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
//! use fp_library::{
//! 	brands::*,
//! 	types::*,
//! };
//!
//! // ✅ CAN DO: Stack-safe recursion
//! let free = Free::<ThunkBrand, _>::pure(42).bind(|x| Free::pure(x + 1));
//! ```

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			Apply,
			brands::ThunkBrand,
			classes::{
				Deferrable,
				Evaluable,
				Monad,
				NaturalTransformation,
			},
			kinds::*,
			types::{
				CatList,
				Thunk,
			},
		},
		fp_macros::*,
		std::{
			any::Any,
			marker::PhantomData,
		},
	};

	/// A type-erased value for internal use.
	///
	/// This type alias represents a value whose type has been erased to [`Box<dyn Any>`].
	/// It is used within the internal implementation of [`Free`] to allow for
	/// heterogeneous chains of computations in the [`CatList`].
	pub type TypeErasedValue = Box<dyn Any>;

	/// A type-erased continuation.
	///
	/// This type alias represents a function that takes a [`TypeErasedValue`]
	/// and returns a new [`Free`] computation (also type-erased).
	#[document_type_parameters("The base functor.")]
	pub type Continuation<F> = Box<dyn FnOnce(TypeErasedValue) -> Free<F, TypeErasedValue>>;

	/// The internal representation of the [`Free`] monad.
	///
	/// This enum encodes the structure of the free monad, supporting
	/// pure values, suspended computations, and efficient concatenation of binds.
	#[document_type_parameters(
		"The base functor (must implement [`Evaluable`]).",
		"The result type."
	)]
	pub enum FreeInner<F, A>
	where
		F: Evaluable + 'static,
		A: 'static, {
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
		Bind {
			/// The initial computation.
			head: Box<Free<F, TypeErasedValue>>,
			/// The list of continuations to apply to the result of `head`.
			continuations: CatList<Continuation<F>>,
			/// Phantom data for type parameter `A`.
			_marker: PhantomData<A>,
		},
	}

	/// The Free monad with O(1) bind via [`CatList`].
	///
	/// This implementation follows ["Reflection without Remorse"](http://okmij.org/ftp/Haskell/zseq.pdf) to ensure
	/// that left-associated binds do not degrade performance.
	///
	/// # Linear consumption invariant
	///
	/// `Free` values must be consumed exactly once, either by calling [`evaluate`](Free::evaluate),
	/// [`bind`](Free::bind), [`erase_type`](Free::erase_type), or by being dropped. The inner
	/// `Option` wrapper enables a take-and-replace pattern (analogous to `Cell::take`) so that
	/// methods like `bind` and `erase_type` can move the `FreeInner` out of `&mut self` without
	/// leaving the struct in an invalid state. After the take, the `Option` is `None`, and any
	/// subsequent access will panic with "Free value already consumed." This invariant is relied
	/// upon by the `Drop` implementation to avoid double-freeing internal allocations.
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
	/// # Consuming a `Free`: `evaluate` vs `fold_free`
	///
	/// * [`evaluate`](Free::evaluate) runs the computation to completion using an iterative
	///   loop. It requires the base functor `F` to implement [`Evaluable`], meaning each
	///   suspended layer can be "unwrapped" to reveal the next step. Use this when you want
	///   the final `A` value directly.
	/// * [`fold_free`](Free::fold_free) interprets the `Free` into a different monad `G` via
	///   a [`NaturalTransformation`] from `F` to `G`. Each suspended `F` layer is transformed
	///   into `G`, and the results are threaded together with `G::bind`. Use this when you
	///   want to translate the free structure into another effect (e.g., `Option`, `Result`,
	///   or a custom interpreter).
	#[document_type_parameters(
		"The base functor (must implement [`Evaluable`]).",
		"The result type."
	)]
	///
	pub struct Free<F, A>(Option<FreeInner<F, A>>)
	where
		F: Evaluable + 'static,
		A: 'static;

	#[document_type_parameters("The base functor.", "The result type.")]
	#[document_parameters("The Free monad instance to operate on.")]
	impl<F, A> Free<F, A>
	where
		F: Evaluable + 'static,
		A: 'static,
	{
		/// Constructs a `Free` from a `FreeInner` value.
		///
		/// This is the sole internal constructor, ensuring all `Free` values
		/// start with `Some(inner)`.
		#[document_signature]
		#[document_parameters("The inner value to wrap.")]
		#[document_returns("A new `Free` containing the given inner value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::*,
		/// };
		///
		/// // `from_inner` is internal; `pure` is the public API that uses it.
		/// let free = Free::<ThunkBrand, _>::pure(42);
		/// assert_eq!(free.evaluate(), 42);
		/// ```
		fn from_inner(inner: FreeInner<F, A>) -> Self {
			Free(Some(inner))
		}

		/// Takes the inner value out of this `Free`, leaving `None` behind.
		///
		/// This implements the linear consumption pattern: each `Free` value
		/// is consumed exactly once. The `Option` wrapper allows moving the
		/// `FreeInner` out through `&mut self` (as required by `bind`, `erase_type`,
		/// `evaluate`, and `Drop`) without leaving the struct in an invalid state.
		/// After this call, the `Option` is `None`. Any subsequent call will panic
		/// with "Free value already consumed."
		///
		/// # Panics
		///
		/// Panics if the inner value has already been taken.
		#[document_signature]
		#[document_returns("The inner `FreeInner` value, consuming it from this `Free`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::*,
		/// };
		///
		/// // `take_inner` is internal; `evaluate` is the public API that uses it.
		/// let free = Free::<ThunkBrand, _>::pure(10);
		/// assert_eq!(free.evaluate(), 10);
		/// ```
		fn take_inner(&mut self) -> FreeInner<F, A> {
			// SAFETY: Free values are used exactly once; double consumption indicates a bug.
			#[allow(clippy::expect_used)]
			self.0.take().expect("Free value already consumed")
		}

		/// Creates a pure `Free` value.
		#[document_signature]
		///
		#[document_parameters("The value to wrap.")]
		///
		#[document_returns("A `Free` computation that produces `a`.")]
		///
		#[inline]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::*,
		/// };
		///
		/// let free = Free::<ThunkBrand, _>::pure(42);
		/// assert_eq!(free.evaluate(), 42);
		/// ```
		pub fn pure(a: A) -> Self {
			Free::from_inner(FreeInner::Pure(a))
		}

		/// Creates a suspended computation from a functor value.
		#[document_signature]
		///
		#[document_parameters("The functor value containing the next step.")]
		///
		#[document_returns("A `Free` computation that performs the effect `fa`.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::*,
		/// };
		///
		/// let eval = Thunk::new(|| Free::pure(42));
		/// let free = Free::<ThunkBrand, _>::wrap(eval);
		/// assert_eq!(free.evaluate(), 42);
		/// ```
		pub fn wrap(
			fa: Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, Free<F, A>>)
		) -> Self {
			Free::from_inner(FreeInner::Wrap(fa))
		}

		/// Lifts a functor value into the Free monad.
		///
		/// This is the primary way to inject effects into Free monad computations.
		/// Equivalent to PureScript's `liftF` and Haskell's `liftF`.
		#[document_signature]
		///
		/// ### Implementation
		///
		/// ```text
		/// liftF fa = wrap (map pure fa)
		/// ```
		#[document_parameters("The functor value to lift.")]
		///
		#[document_returns("A `Free` computation that performs the effect and returns the result.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::*,
		/// };
		///
		/// // Lift a simple computation
		/// let thunk = Thunk::new(|| 42);
		/// let free = Free::<ThunkBrand, _>::lift_f(thunk);
		/// assert_eq!(free.evaluate(), 42);
		///
		/// // Build a computation from raw effects
		/// let computation = Free::<ThunkBrand, _>::lift_f(Thunk::new(|| 10))
		/// 	.bind(|x| Free::lift_f(Thunk::new(move || x * 2)))
		/// 	.bind(|x| Free::lift_f(Thunk::new(move || x + 5)));
		/// assert_eq!(computation.evaluate(), 25);
		/// ```
		pub fn lift_f(fa: Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, A>)) -> Self {
			// Map the value to a pure Free, then wrap it
			Free::wrap(F::map(Free::pure, fa))
		}

		/// Monadic bind with O(1) complexity.
		#[document_signature]
		///
		#[document_type_parameters("The result type of the new computation.")]
		///
		#[document_parameters("The function to apply to the result of this computation.")]
		///
		#[document_returns("A new `Free` computation that chains `f` after this computation.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::*,
		/// };
		///
		/// let free = Free::<ThunkBrand, _>::pure(42).bind(|x| Free::pure(x + 1));
		/// assert_eq!(free.evaluate(), 43);
		/// ```
		pub fn bind<B: 'static>(
			mut self,
			f: impl FnOnce(A) -> Free<F, B> + 'static,
		) -> Free<F, B> {
			// Type-erase the continuation
			let erased_f: Continuation<F> = Box::new(move |val: TypeErasedValue| {
				// INVARIANT: type is maintained by internal invariant - mismatch indicates a bug
				#[allow(clippy::expect_used)]
				let a: A = *val.downcast().expect("Type mismatch in Free::bind");
				let free_b: Free<F, B> = f(a);
				free_b.erase_type()
			});

			let inner = self.take_inner();

			match inner {
				// Pure: create a Bind with this continuation
				FreeInner::Pure(a) => {
					let head: Free<F, TypeErasedValue> =
						Free::from_inner(FreeInner::Pure(Box::new(a) as TypeErasedValue));
					Free::from_inner(FreeInner::Bind {
						head: Box::new(head),
						continuations: CatList::singleton(erased_f),
						_marker: PhantomData,
					})
				}

				// Wrap: wrap in a Bind
				FreeInner::Wrap(fa) => {
					let head = Free::wrap(fa).boxed_erase_type();
					Free::from_inner(FreeInner::Bind {
						head,
						continuations: CatList::singleton(erased_f),
						_marker: PhantomData,
					})
				}

				// Bind: snoc the new continuation onto the CatList (O(1)!)
				FreeInner::Bind {
					head,
					continuations: conts,
					..
				} => Free::from_inner(FreeInner::Bind {
					head,
					continuations: conts.snoc(erased_f),
					_marker: PhantomData,
				}),
			}
		}

		/// Functor map: transforms the result without changing structure.
		///
		/// Implemented via [`bind`](Free::bind) and [`pure`](Free::pure),
		/// keeping the internal representation simple with fewer variants.
		#[document_signature]
		///
		#[document_type_parameters("The result type of the mapping function.")]
		///
		#[document_parameters("The function to apply to the result of this computation.")]
		///
		#[document_returns("A new `Free` computation with the transformed result.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::*,
		/// };
		///
		/// let free = Free::<ThunkBrand, _>::pure(10).map(|x| x * 2);
		/// assert_eq!(free.evaluate(), 20);
		/// ```
		pub fn map<B: 'static>(
			self,
			f: impl FnOnce(A) -> B + 'static,
		) -> Free<F, B> {
			self.bind(move |a| Free::pure(f(a)))
		}

		/// Decomposes this `Free` computation into one step.
		///
		/// Returns `Ok(a)` if the computation is a pure value, or
		/// `Err(f_free)` if the computation is suspended in the functor `F`,
		/// where `f_free` contains the next `Free` computation wrapped in `F`.
		///
		/// For `Bind` variants, the continuation chain is collapsed first by
		/// iteratively applying continuations until a `Pure` or `Wrap` is reached.
		#[document_signature]
		///
		#[document_returns(
			"`Ok(a)` if the computation is a pure value, `Err(fa)` if it is a suspended computation."
		)]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::*,
		/// };
		///
		/// // Pure returns Ok
		/// let free = Free::<ThunkBrand, _>::pure(42);
		/// assert!(matches!(free.resume(), Ok(42)));
		///
		/// // Wrap returns Err containing the functor layer
		/// let free = Free::<ThunkBrand, _>::wrap(Thunk::new(|| Free::pure(99)));
		/// assert!(free.resume().is_err());
		/// ```
		pub fn resume(
			self
		) -> Result<A, Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, Free<F, A>>)> {
			// We process the bind chain iteratively, similar to evaluate,
			// but stop when we reach a Pure or Wrap at the top level.
			let mut current: Free<F, TypeErasedValue> = self.erase_type();
			let mut continuations: CatList<Continuation<F>> = CatList::empty();

			loop {
				// INVARIANT: Free values are used exactly once - double consumption indicates a bug
				#[allow(clippy::expect_used)]
				let inner = current.0.take().expect("Free value already consumed");

				match inner {
					FreeInner::Pure(val) => {
						match continuations.uncons() {
							Some((continuation, rest)) => {
								current = continuation(val);
								continuations = rest;
							}
							None => {
								// No more continuations - we have a final pure value
								// INVARIANT: type is maintained by internal invariant
								#[allow(clippy::expect_used)]
								return Ok(*val
									.downcast::<A>()
									.expect("Type mismatch in Free::resume final downcast"));
							}
						}
					}

					FreeInner::Wrap(fa) => {
						// We have a suspended computation. We need to rebuild the
						// Free<F, A> from the type-erased Free<F, TypeErasedValue>
						// by re-attaching the remaining continuations.

						// Always append the downcast continuation that converts
						// TypeErasedValue back to the concrete type A.
						let downcast_cont: Continuation<F> =
							Box::new(move |val: TypeErasedValue| {
								// INVARIANT: type is maintained by internal invariant
								#[allow(clippy::expect_used)]
								let a: A = *val.downcast().expect("Type mismatch in Free::resume downcast");
								Free::<F, A>::pure(a).erase_type()
							});
						let all_conts = continuations.snoc(downcast_cont);

						// Use Cell to move the CatList into the Fn closure (called exactly once).
						let remaining = std::cell::Cell::new(Some(all_conts));
						let typed_fa = F::map(
							move |inner_free: Free<F, TypeErasedValue>| {
								// INVARIANT: functors call map exactly once per element
								#[allow(clippy::expect_used)]
								let conts = remaining.take().expect("Free::resume map called more than once");
								Free(Some(FreeInner::Bind {
									head: Box::new(inner_free),
									continuations: conts,
									_marker: PhantomData,
								}))
							},
							fa,
						);
						return Err(typed_fa);
					}

					FreeInner::Bind {
						head,
						continuations: inner_continuations,
						..
					} => {
						current = *head;
						continuations = inner_continuations.append(continuations);
					}
				}
			}
		}

		/// Interprets this `Free` monad into a target monad `G` using a natural transformation.
		///
		/// This is the standard `foldFree` operation from Haskell/PureScript. It recursively
		/// processes the free structure, applying the natural transformation at each suspended
		/// layer to convert from functor `F` into monad `G`.
		///
		/// For `Pure(a)`, returns `G::pure(a)`.
		/// For `Wrap(fa)`, applies the natural transformation to get `G<Free<F, A>>`,
		/// then binds recursively to interpret the rest.
		///
		/// ### Stack Safety
		///
		/// **Caveat:** `fold_free` is **not** stack-safe when interpreting into a
		/// strict (non-trampolined) target monad. Unlike [`Free::evaluate`], which
		/// uses an iterative loop, `fold_free` relies on actual recursion: each
		/// `Wrap` layer adds one stack frame via
		/// [`Semimonad::bind`](crate::classes::Semimonad::bind). For strict target
		/// monads (e.g., `OptionBrand`), deeply nested `Free` structures will
		/// overflow the stack.
		///
		/// Prefer [`Free::evaluate`] for stack-safe execution when you do not need
		/// to interpret into a different monad.
		#[document_signature]
		///
		#[document_type_parameters("The target monad brand.")]
		///
		#[document_parameters("The natural transformation from `F` to `G`.")]
		///
		#[document_returns("The result of interpreting this free computation in monad `G`.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	Apply,
		/// 	brands::*,
		/// 	classes::*,
		/// 	kinds::*,
		/// 	types::*,
		/// };
		///
		/// // Define a natural transformation from Thunk to Option
		/// #[derive(Clone)]
		/// struct ThunkToOption;
		/// impl NaturalTransformation<ThunkBrand, OptionBrand> for ThunkToOption {
		/// 	fn transform<'a, A: 'a>(
		/// 		&self,
		/// 		fa: Thunk<'a, A>,
		/// 	) -> Option<A> {
		/// 		Some(fa.evaluate())
		/// 	}
		/// }
		///
		/// let free = Free::<ThunkBrand, _>::pure(42);
		/// let result: Option<i32> = free.fold_free::<OptionBrand>(ThunkToOption);
		/// assert_eq!(result, Some(42));
		/// ```
		pub fn fold_free<G>(
			self,
			nt: impl NaturalTransformation<F, G> + Clone + 'static,
		) -> Apply!(<G as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, A>)
		where
			G: Monad + 'static, {
			match self.resume() {
				Ok(a) => G::pure(a),
				Err(fa) => {
					// fa: F<Free<F, A>>
					// Transform F<Free<F, A>> into G<Free<F, A>> using the natural transformation
					let nt2 = nt.clone();
					let ga: Apply!(
						<G as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, Free<F, A>>
					) = nt.transform(fa);
					// Bind over G to recursively fold the inner Free<F, A>
					G::bind(ga, move |inner_free: Free<F, A>| {
						inner_free.fold_free::<G>(nt2.clone())
					})
				}
			}
		}

		/// Converts to type-erased form.
		#[document_signature]
		#[document_returns(
			"A `Free` computation where the result type has been erased to `Box<dyn Any>`."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::*,
		/// };
		/// let free = Free::<ThunkBrand, _>::pure(42);
		/// let erased = free.erase_type();
		/// assert!(erased.evaluate().is::<i32>());
		/// ```
		pub fn erase_type(mut self) -> Free<F, TypeErasedValue> {
			let inner = self.take_inner();

			match inner {
				FreeInner::Pure(a) =>
					Free::from_inner(FreeInner::Pure(Box::new(a) as TypeErasedValue)),
				FreeInner::Wrap(fa) => {
					// Map over the functor to erase the inner type.
					// IMPORTANT: this relies on the invariant that `Functor::map` for `F`
					// calls the mapping function exactly once. If it were called zero times,
					// the inner `Free` value would leak. If called more than once, it would
					// be consumed multiple times, violating the linear consumption invariant.
					let erased = F::map(|inner: Free<F, A>| inner.erase_type(), fa);
					Free::from_inner(FreeInner::Wrap(erased))
				}
				FreeInner::Bind {
					head,
					continuations,
					..
				} => Free::from_inner(FreeInner::Bind {
					head,
					continuations,
					_marker: PhantomData,
				}),
			}
		}

		/// Converts to boxed type-erased form.
		#[document_signature]
		#[document_returns("A boxed `Free` computation where the result type has been erased.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::*,
		/// };
		/// let free = Free::<ThunkBrand, _>::pure(42);
		/// let boxed = free.boxed_erase_type();
		/// assert!(boxed.evaluate().is::<i32>());
		/// ```
		pub fn boxed_erase_type(self) -> Box<Free<F, TypeErasedValue>> {
			Box::new(self.erase_type())
		}

		/// Executes the Free computation, returning the final result.
		///
		/// This is the "trampoline" that iteratively processes the
		/// [`CatList`] of continuations without growing the stack.
		#[document_signature]
		///
		#[document_returns("The final result of the computation.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::*,
		/// };
		///
		/// let free = Free::<ThunkBrand, _>::pure(42);
		/// assert_eq!(free.evaluate(), 42);
		/// ```
		pub fn evaluate(self) -> A {
			// Start with a type-erased version
			let mut current: Free<F, TypeErasedValue> = self.erase_type();
			let mut continuations: CatList<Continuation<F>> = CatList::empty();

			loop {
				let inner = current.take_inner();

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
								// INVARIANT: type is maintained by internal invariant - mismatch indicates a bug
								#[allow(clippy::expect_used)]
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

					FreeInner::Bind {
						head,
						continuations: inner_continuations,
						..
					} => {
						// Merge the inner continuations with outer ones
						// This is where CatList's O(1) append shines!
						current = *head;
						continuations = inner_continuations.append(continuations);
					}
				}
			}
		}
	}

	#[document_type_parameters("The base functor.", "The result type.")]
	#[document_parameters("The free monad instance to drop.")]
	impl<F, A> Drop for Free<F, A>
	where
		F: Evaluable + 'static,
		A: 'static,
	{
		#[document_signature]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::*,
		/// };
		/// {
		/// 	let _free = Free::<ThunkBrand, _>::pure(42);
		/// } // drop called here
		/// assert!(true);
		/// ```
		fn drop(&mut self) {
			// Take the inner value out so we can iteratively dismantle the chain
			// instead of relying on recursive Drop, which would overflow the stack
			// for deep computations (both Bind chains and Wrap chains).
			let mut worklist: Vec<FreeInner<F, A>> = Vec::new();

			if let Some(inner) = self.0.take() {
				worklist.push(inner);
			}

			while let Some(node) = worklist.pop() {
				match node {
					FreeInner::Pure(_) => {
						// Trivially dropped, no nested Free values.
					}

					FreeInner::Wrap(fa) => {
						// The functor layer contains a `Free<F, A>` inside. If we
						// let it drop recursively, deeply nested Wrap chains will
						// overflow the stack. Instead, we use `Evaluable::evaluate`
						// to eagerly extract the inner `Free`, then push it onto
						// the worklist for iterative dismantling.
						let mut extracted = <F as Evaluable>::evaluate(fa);
						if let Some(next_inner) = extracted.0.take() {
							worklist.push(next_inner);
						}
					}

					FreeInner::Bind {
						head,
						continuations,
						..
					} => {
						// Drop the head computation. Since `head` is
						// `Box<Free<F, TypeErasedValue>>` (a different type
						// parameter than ours), we cannot fold it into our
						// worklist. Its own `Drop` impl will iteratively
						// dismantle any nested chains it contains.
						drop(head);

						// Drain the CatList of continuations iteratively. Each
						// continuation is a Box<dyn FnOnce> that may capture Free
						// values. By consuming them one at a time via uncons, we
						// let each boxed closure drop without building stack depth.
						let mut conts = continuations;
						while let Some((_continuation, rest)) = conts.uncons() {
							// _continuation (a Box<dyn FnOnce>) drops here, which
							// frees any captured Free values. Those Free values'
							// own Drop impls will re-enter this iterative loop.
							conts = rest;
						}
					}
				}
			}
		}
	}

	#[document_type_parameters("The result type.")]
	impl<A: 'static> Deferrable<'static> for Free<ThunkBrand, A> {
		/// Creates a `Free` computation from a thunk.
		///
		/// This delegates to `Free::wrap` and `Thunk::new`.
		#[document_signature]
		///
		#[document_parameters("A thunk that produces the free computation.")]
		///
		#[document_returns("The deferred free computation.")]
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
		/// let task: Free<ThunkBrand, i32> = Deferrable::defer(|| Free::pure(42));
		/// assert_eq!(task.evaluate(), 42);
		/// ```
		fn defer(f: impl FnOnce() -> Self + 'static) -> Self
		where
			Self: Sized, {
			Self::wrap(Thunk::new(f))
		}
	}
}
pub use inner::*;

#[cfg(test)]
mod tests {
	use {
		super::*,
		crate::{
			brands::{
				OptionBrand,
				ThunkBrand,
			},
			classes::natural_transformation::NaturalTransformation,
			types::thunk::Thunk,
		},
	};

	/// Tests `Free::pure`.
	///
	/// **What it tests:** Verifies that `pure` creates a computation that simply returns the provided value.
	/// **How it tests:** Constructs a `Free::pure(42)` and runs it, asserting the result is 42.
	#[test]
	fn test_free_pure() {
		let free = Free::<ThunkBrand, _>::pure(42);
		assert_eq!(free.evaluate(), 42);
	}

	/// Tests `Free::wrap`.
	///
	/// **What it tests:** Verifies that `wrap` creates a computation from a suspended effect.
	/// **How it tests:** Wraps a `Free::pure(42)` inside a `Thunk`, wraps it into a `Free`, and runs it to ensure it unwraps correctly.
	#[test]
	fn test_free_wrap() {
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

	/// Tests `Free::resume` on a `Pure` variant.
	///
	/// **What it tests:** Verifies that `resume` on a pure value returns `Ok(value)`.
	/// **How it tests:** Creates a `Free::pure(42)` and asserts `resume()` returns `Ok(42)`.
	#[test]
	fn test_free_resume_pure() {
		let free = Free::<ThunkBrand, _>::pure(42);
		assert!(matches!(free.resume(), Ok(42)));
	}

	/// Tests `Free::resume` on a `Wrap` variant.
	///
	/// **What it tests:** Verifies that `resume` on a suspended computation returns `Err(functor_layer)`.
	/// **How it tests:** Creates a `Free::wrap(...)` and asserts `resume()` returns `Err(_)`,
	/// then evaluates the inner `Free` to verify the value is preserved.
	#[test]
	fn test_free_resume_wrap() {
		let free = Free::<ThunkBrand, _>::wrap(Thunk::new(|| Free::pure(99)));
		let result = free.resume();
		assert!(result.is_err());
		// Evaluate the inner thunk to verify the value
		let thunk = result.unwrap_err();
		let inner_free: Free<ThunkBrand, i32> = thunk.evaluate();
		assert_eq!(inner_free.evaluate(), 99);
	}

	/// Tests `Free::resume` on a `Bind` variant.
	///
	/// **What it tests:** Verifies that `resume` correctly collapses bind chains.
	/// **How it tests:** Creates a `pure(10).bind(|x| pure(x + 5))` and checks
	/// that resume returns `Ok(15)` after collapsing the bind.
	#[test]
	fn test_free_resume_bind() {
		let free = Free::<ThunkBrand, _>::pure(10).bind(|x: i32| Free::pure(x + 5));
		assert!(matches!(free.resume(), Ok(15)));
	}

	/// Tests `Free::resume` on a `Bind` with `Wrap`.
	///
	/// **What it tests:** Verifies that `resume` correctly handles bind chains that end in a `Wrap`.
	/// **How it tests:** Creates a `wrap(...).bind(...)` and checks that resume returns `Err(_)`.
	#[test]
	fn test_free_resume_bind_on_wrap() {
		let free = Free::<ThunkBrand, _>::wrap(Thunk::new(|| Free::pure(10)))
			.bind(|x: i32| Free::pure(x + 5));
		let result = free.resume();
		assert!(result.is_err());
		// The inner computation, when evaluated, should produce 15
		let thunk = result.unwrap_err();
		let inner_free: Free<ThunkBrand, i32> = thunk.evaluate();
		assert_eq!(inner_free.evaluate(), 15);
	}

	/// A natural transformation from `ThunkBrand` to `OptionBrand`.
	///
	/// Evaluates the thunk and wraps the result in `Some`.
	#[derive(Clone)]
	struct ThunkToOption;
	impl NaturalTransformation<ThunkBrand, OptionBrand> for ThunkToOption {
		fn transform<'a, A: 'a>(
			&self,
			fa: Thunk<'a, A>,
		) -> Option<A> {
			Some(fa.evaluate())
		}
	}

	/// Tests `Free::fold_free` on a `Pure` variant.
	///
	/// **What it tests:** Verifies that `fold_free` on a pure value wraps it in the target monad.
	/// **How it tests:** Folds `Free::pure(42)` with `ThunkToOption` and asserts it returns `Some(42)`.
	#[test]
	fn test_free_fold_free_pure() {
		let free = Free::<ThunkBrand, _>::pure(42);
		let result: Option<i32> = free.fold_free::<OptionBrand>(ThunkToOption);
		assert_eq!(result, Some(42));
	}

	/// Tests `Free::fold_free` on a suspended computation.
	///
	/// **What it tests:** Verifies that `fold_free` correctly interprets a computation with effects.
	/// **How it tests:** Lifts a thunk into Free, then folds with `ThunkToOption`.
	#[test]
	fn test_free_fold_free_wrap() {
		let free = Free::<ThunkBrand, _>::lift_f(Thunk::new(|| 42));
		let result: Option<i32> = free.fold_free::<OptionBrand>(ThunkToOption);
		assert_eq!(result, Some(42));
	}

	/// Tests `Free::fold_free` on a chained computation.
	///
	/// **What it tests:** Verifies that `fold_free` correctly interprets a multi-step computation.
	/// **How it tests:** Chains several binds and folds with `ThunkToOption`.
	#[test]
	fn test_free_fold_free_chain() {
		let free = Free::<ThunkBrand, _>::lift_f(Thunk::new(|| 10))
			.bind(|x| Free::<ThunkBrand, _>::lift_f(Thunk::new(move || x * 2)))
			.bind(|x| Free::pure(x + 5));
		let result: Option<i32> = free.fold_free::<OptionBrand>(ThunkToOption);
		assert_eq!(result, Some(25));
	}

	/// Tests `Free::map` on a pure value.
	///
	/// **What it tests:** Verifies that `map` correctly transforms a pure value.
	/// **How it tests:** Creates `Free::pure(10).map(|x| x * 2)` and evaluates.
	#[test]
	fn test_free_map_pure() {
		let free = Free::<ThunkBrand, _>::pure(10).map(|x| x * 2);
		assert_eq!(free.evaluate(), 20);
	}

	/// Tests chained `Free::map` operations.
	///
	/// **What it tests:** Verifies that multiple map operations compose correctly.
	/// **How it tests:** Chains three maps and verifies the final result.
	#[test]
	fn test_free_map_chain() {
		let free = Free::<ThunkBrand, _>::pure(5).map(|x| x + 1).map(|x| x * 3).map(|x| x - 2);
		assert_eq!(free.evaluate(), 16);
	}

	/// Tests `Free::map` on a wrapped computation.
	///
	/// **What it tests:** Verifies that map works on suspended computations.
	/// **How it tests:** Wraps a thunk in Free, maps over it, and evaluates.
	#[test]
	fn test_free_map_wrap() {
		let free = Free::<ThunkBrand, _>::lift_f(Thunk::new(|| 7)).map(|x| x * 3);
		assert_eq!(free.evaluate(), 21);
	}

	/// Tests `Free::map` followed by `Free::bind`.
	///
	/// **What it tests:** Verifies that map and bind interoperate correctly.
	/// **How it tests:** Maps, then binds, and checks the result.
	#[test]
	fn test_free_map_then_bind() {
		let free = Free::<ThunkBrand, _>::pure(10).map(|x| x + 5).bind(|x| Free::pure(x * 2));
		assert_eq!(free.evaluate(), 30);
	}

	/// Tests `Free::bind` followed by `Free::map`.
	///
	/// **What it tests:** Verifies that bind and map interoperate correctly.
	/// **How it tests:** Binds, then maps, and checks the result.
	#[test]
	fn test_free_bind_then_map() {
		let free = Free::<ThunkBrand, _>::pure(10).bind(|x| Free::pure(x + 5)).map(|x| x * 2);
		assert_eq!(free.evaluate(), 30);
	}

	/// Tests stack safety of deep `Free::map` chains.
	///
	/// **What it tests:** Verifies that deeply nested map chains do not overflow the stack.
	/// **How it tests:** Creates a chain of 10,000 map operations.
	#[test]
	fn test_free_map_deep_chain() {
		let mut free = Free::<ThunkBrand, _>::pure(0i64);
		for _ in 0 .. 10_000 {
			free = free.map(|x| x + 1);
		}
		assert_eq!(free.evaluate(), 10_000);
	}

	// ── Monad law tests (Task 6.2h) ──

	/// Tests monad left identity law for `Free`.
	///
	/// **What it tests:** Left identity: `pure(a).bind(f)` equals `f(a)`.
	/// **How it tests:** Applies `bind` to a `pure` value and compares the result
	/// to calling `f` directly on the same value.
	#[test]
	fn test_free_monad_left_identity() {
		let a = 42_i32;
		let f = |x: i32| Free::<ThunkBrand, _>::pure(x * 2 + 1);

		let lhs = Free::<ThunkBrand, _>::pure(a).bind(f).evaluate();
		let rhs = f(a).evaluate();
		assert_eq!(lhs, rhs);
	}

	/// Tests monad left identity law with `lift_f`.
	///
	/// **What it tests:** Left identity with an effectful continuation: `pure(a).bind(f)`
	/// equals `f(a)` when `f` uses `lift_f`.
	/// **How it tests:** Uses `lift_f` inside the continuation to introduce a functor layer.
	#[test]
	fn test_free_monad_left_identity_with_lift_f() {
		let a = 10_i32;
		let f = |x: i32| Free::<ThunkBrand, _>::lift_f(Thunk::new(move || x + 5));

		let lhs = Free::<ThunkBrand, _>::pure(a).bind(f).evaluate();
		let rhs = f(a).evaluate();
		assert_eq!(lhs, rhs);
	}

	/// Tests monad right identity law for `Free`.
	///
	/// **What it tests:** Right identity: `m.bind(pure)` equals `m`.
	/// **How it tests:** Binds `pure` to various `Free` computations and checks that
	/// the result is unchanged.
	#[test]
	fn test_free_monad_right_identity_pure() {
		let m = Free::<ThunkBrand, _>::pure(42);
		let result = m.bind(Free::pure).evaluate();
		assert_eq!(result, 42);
	}

	/// Tests monad right identity law with a bind chain.
	///
	/// **What it tests:** Right identity on a computation that already has binds:
	/// `m.bind(|x| Free::pure(x))` produces the same result as `m`.
	/// **How it tests:** Constructs a chain of binds and appends `pure` at the end.
	#[test]
	fn test_free_monad_right_identity_chain() {
		let m =
			Free::<ThunkBrand, _>::pure(10).bind(|x| Free::pure(x + 5)).bind(|x| Free::pure(x * 3));

		let expected = Free::<ThunkBrand, _>::pure(10)
			.bind(|x| Free::pure(x + 5))
			.bind(|x| Free::pure(x * 3))
			.evaluate();

		let result = m.bind(Free::pure).evaluate();
		assert_eq!(result, expected);
	}

	/// Tests monad right identity law with a `Wrap` variant.
	///
	/// **What it tests:** Right identity when the initial computation is a `Wrap`.
	/// **How it tests:** Wraps a thunk, binds `pure`, and checks the result.
	#[test]
	fn test_free_monad_right_identity_wrap() {
		let m = Free::<ThunkBrand, _>::wrap(Thunk::new(|| Free::pure(99)));
		let result = m.bind(Free::pure).evaluate();
		assert_eq!(result, 99);
	}

	/// Tests monad associativity law for `Free`.
	///
	/// **What it tests:** Associativity: `m.bind(f).bind(g)` equals
	/// `m.bind(|x| f(x).bind(g))`.
	/// **How it tests:** Evaluates both sides of the law and checks equality.
	#[test]
	fn test_free_monad_associativity() {
		let f = |x: i32| Free::<ThunkBrand, _>::pure(x + 10);
		let g = |x: i32| Free::<ThunkBrand, _>::pure(x * 2);

		// Left grouping: (m >>= f) >>= g
		let lhs = Free::<ThunkBrand, _>::pure(5).bind(f).bind(g).evaluate();

		// Right grouping: m >>= (\x -> f x >>= g)
		let rhs = Free::<ThunkBrand, _>::pure(5).bind(move |x| f(x).bind(g)).evaluate();

		assert_eq!(lhs, rhs);
		assert_eq!(lhs, 30); // (5 + 10) * 2
	}

	/// Tests monad associativity law with effectful continuations.
	///
	/// **What it tests:** Associativity when continuations use `lift_f` for real effects.
	/// **How it tests:** Both groupings produce the same result when `lift_f` is involved.
	#[test]
	fn test_free_monad_associativity_with_effects() {
		let f = |x: i32| Free::<ThunkBrand, _>::lift_f(Thunk::new(move || x + 3));
		let g = |x: i32| Free::<ThunkBrand, _>::lift_f(Thunk::new(move || x * 4));

		let lhs = Free::<ThunkBrand, _>::pure(7).bind(f).bind(g).evaluate();
		let rhs = Free::<ThunkBrand, _>::pure(7).bind(move |x| f(x).bind(g)).evaluate();

		assert_eq!(lhs, rhs);
		assert_eq!(lhs, 40); // (7 + 3) * 4
	}

	/// Tests monad associativity law with three continuations.
	///
	/// **What it tests:** Associativity holds for longer chains where multiple
	/// groupings are possible.
	/// **How it tests:** Compares sequential bind with nested bind for three functions.
	#[test]
	fn test_free_monad_associativity_three_functions() {
		let f = |x: i32| Free::<ThunkBrand, _>::pure(x + 1);
		let g = |x: i32| Free::<ThunkBrand, _>::pure(x * 2);
		let h = |x: i32| Free::<ThunkBrand, _>::pure(x - 3);

		// ((m >>= f) >>= g) >>= h
		let lhs = Free::<ThunkBrand, _>::pure(10).bind(f).bind(g).bind(h).evaluate();

		// m >>= (\x -> f(x) >>= (\y -> g(y) >>= h))
		let rhs = Free::<ThunkBrand, _>::pure(10)
			.bind(move |x| f(x).bind(move |y| g(y).bind(h)))
			.evaluate();

		assert_eq!(lhs, rhs);
		assert_eq!(lhs, 19); // ((10 + 1) * 2) - 3
	}

	// ── Mixed deep chain tests (Task 6.2i) ──

	/// Tests interleaved `bind` and `wrap` in a deep chain.
	///
	/// **What it tests:** Verifies that alternating `bind` and `wrap` operations
	/// compose correctly and evaluate to the expected result.
	/// **How it tests:** Builds a chain that alternates between `bind` (increment)
	/// and `wrap` (deferred computation) for 1,000 iterations.
	#[test]
	fn test_free_mixed_bind_and_wrap() {
		let mut free = Free::<ThunkBrand, _>::pure(0_i32);
		for _ in 0 .. 1_000 {
			free = free.bind(|x| {
				let inner = Free::pure(x + 1);
				Free::wrap(Thunk::new(move || inner))
			});
		}
		assert_eq!(free.evaluate(), 1_000);
	}

	/// Tests interleaved `bind` and `lift_f` in a deep chain.
	///
	/// **What it tests:** Verifies that alternating `bind` and `lift_f` operations
	/// compose correctly.
	/// **How it tests:** Builds a chain of 1,000 iterations mixing `bind` with `lift_f`.
	#[test]
	fn test_free_mixed_bind_and_lift_f() {
		let mut free = Free::<ThunkBrand, _>::pure(0_i32);
		for i in 0 .. 1_000 {
			if i % 2 == 0 {
				free = free.bind(|x| Free::pure(x + 1));
			} else {
				free = free.bind(|x| Free::<ThunkBrand, _>::lift_f(Thunk::new(move || x + 1)));
			}
		}
		assert_eq!(free.evaluate(), 1_000);
	}

	/// Tests a deep chain mixing all four construction methods.
	///
	/// **What it tests:** Verifies that `pure`, `bind`, `wrap`, and `lift_f` can be
	/// freely interleaved in a single computation chain.
	/// **How it tests:** Applies a rotating pattern of operations across 400 iterations
	/// (100 of each type) and checks the final accumulated value.
	#[test]
	fn test_free_mixed_all_constructors() {
		let mut free = Free::<ThunkBrand, _>::pure(0_i32);
		for i in 0 .. 400 {
			match i % 4 {
				0 => {
					// bind with pure
					free = free.bind(|x| Free::pure(x + 1));
				}
				1 => {
					// bind with lift_f
					free = free.bind(|x| Free::<ThunkBrand, _>::lift_f(Thunk::new(move || x + 1)));
				}
				2 => {
					// bind with wrap
					free = free.bind(|x| {
						let inner = Free::pure(x + 1);
						Free::wrap(Thunk::new(move || inner))
					});
				}
				3 => {
					// nested bind
					free =
						free.bind(|x| Free::<ThunkBrand, _>::pure(x).bind(|y| Free::pure(y + 1)));
				}
				_ => unreachable!(),
			}
		}
		assert_eq!(free.evaluate(), 400);
	}

	/// Tests deep chain with `lift_f` at the root.
	///
	/// **What it tests:** Verifies that starting a chain from `lift_f` (a `Wrap` variant)
	/// and then applying many `bind` operations works correctly.
	/// **How it tests:** Starts with `lift_f` and chains 10,000 binds.
	#[test]
	fn test_free_deep_chain_from_lift_f() {
		let mut free = Free::<ThunkBrand, _>::lift_f(Thunk::new(|| 0_i32));
		for _ in 0 .. 10_000 {
			free = free.bind(|x| Free::pure(x + 1));
		}
		assert_eq!(free.evaluate(), 10_000);
	}

	/// Tests drop safety of deep `Free::map` chains.
	///
	/// **What it tests:** Verifies that dropping deeply nested map chains does not overflow the stack.
	/// **How it tests:** Creates a deep map chain and drops it without evaluating.
	#[test]
	fn test_free_map_drop_safety() {
		let mut free = Free::<ThunkBrand, _>::pure(0i64);
		for _ in 0 .. 10_000 {
			free = free.map(|x| x + 1);
		}
		drop(free);
	}

	/// Tests deep chain with nested wraps.
	///
	/// **What it tests:** Verifies that deeply nested `wrap` operations evaluate correctly.
	/// **How it tests:** Creates 1,000 nested wraps around a pure value.
	#[test]
	fn test_free_deep_nested_wraps() {
		let mut free = Free::<ThunkBrand, _>::pure(42_i32);
		for _ in 0 .. 1_000 {
			let inner = free;
			free = Free::wrap(Thunk::new(move || inner));
		}
		assert_eq!(free.evaluate(), 42);
	}

	/// Tests dropping a deeply mixed chain without evaluating.
	///
	/// **What it tests:** Verifies that Drop handles deep chains with interleaved
	/// `bind`, `wrap`, and `lift_f` without stack overflow.
	/// **How it tests:** Builds a mixed chain of 50,000 operations and drops it.
	#[test]
	fn test_free_drop_deep_mixed_chain() {
		let mut free = Free::<ThunkBrand, _>::pure(0_i32);
		for i in 0 .. 50_000 {
			if i % 3 == 0 {
				free = free.bind(|x| Free::pure(x + 1));
			} else if i % 3 == 1 {
				free = free.bind(|x| Free::<ThunkBrand, _>::lift_f(Thunk::new(move || x + 1)));
			} else {
				free = free.bind(|x| {
					let inner = Free::pure(x + 1);
					Free::wrap(Thunk::new(move || inner))
				});
			}
		}
		// Drop without evaluating; should not overflow the stack.

		drop(free);
	}
}
