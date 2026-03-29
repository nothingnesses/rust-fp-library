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
//!      baked into the [`Extract`](crate::classes::Extract) trait implemented by the functor `F`.
//!      The [`Free::wrap`] method wraps a functor layer containing a Free computation.
//!
//! 2. **API Surface**:
//!    * **PureScript**: Rich API including `liftF`, `hoistFree`, `resume`, `foldFree`.
//!    * **Rust**: Focused API with construction (`pure`, `wrap`, `lift_f`, `bind`), execution (`evaluate`),
//!      introspection (`resume`), and interpretation (`fold_free`).
//!      * `hoistFree` is provided as [`hoist_free`](Free::hoist_free).
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
//!     `DatabaseOp` must implement a single `Extract` trait.
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
				Extract,
				Functor,
				MonadRec,
				NaturalTransformation,
			},
			kinds::*,
			types::{
				CatList,
				Thunk,
			},
		},
		core::ops::ControlFlow,
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

	/// The internal view of the [`Free`] monad.
	///
	/// This enum encodes the current step of the free monad computation. Both
	/// variants hold type-erased values; the concrete type `A` is tracked by
	/// [`PhantomData`] on the outer [`Free`] struct. The CatList of continuations
	/// lives at the top level in [`Free`], not inside any variant.
	#[document_type_parameters(
		"The base functor. Requires [`Extract`] and [`Functor`] to match the struct-level bounds on [`Free`]; the `Suspend` variant itself only uses the [`Kind`](crate::kinds) trait (implied by `Functor`) for type application, and the `Extract` bound is needed for stack-safe `Drop`."
	)]
	pub enum FreeView<F>
	where
		F: Extract + Functor + 'static, {
		/// A pure value (type-erased).
		///
		/// This variant represents a computation that has finished and produced a value.
		/// The actual type is tracked by `PhantomData<A>` on the enclosing [`Free`].
		Return(TypeErasedValue),

		/// A suspended computation (type-erased).
		///
		/// This variant represents a computation that is suspended in the functor `F`.
		/// The functor contains `Free<F, TypeErasedValue>` as the next step.
		Suspend(
			Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, Free<F, TypeErasedValue>>),
		),
	}

	/// The result of stepping through a [`Free`] computation.
	///
	/// Produced by [`Free::to_view`], this decomposes a `Free` into either
	/// a final value or a suspended functor layer with all pending
	/// continuations reattached. This provides a unified, single-step
	/// decomposition that both [`Free::evaluate`] and [`Free::resume`]
	/// delegate to.
	#[document_type_parameters(
		"The base functor. Requires [`Extract`] and [`Functor`] to match the struct-level bounds on [`Free`].",
		"The result type of the computation."
	)]
	pub enum FreeStep<F, A>
	where
		F: Extract + Functor + 'static,
		A: 'static, {
		/// The computation completed with a final value.
		Done(A),
		/// The computation is suspended in the functor `F`.
		/// The inner `Free` values have all pending continuations reattached.
		Suspended(Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, Free<F, A>>)),
	}

	/// The Free monad with O(1) bind via [`CatList`].
	///
	/// This implementation follows ["Reflection without Remorse"](http://okmij.org/ftp/Haskell/zseq.pdf) to ensure
	/// that left-associated binds do not degrade performance.
	///
	/// # Internal representation
	///
	/// A `Free` value consists of:
	/// * A **view** (`FreeView<F>`) that is either a pure value (`Return`) or a suspended
	///   computation (`Suspend`). Both variants hold type-erased values.
	/// * A **continuation queue** (`CatList<Continuation<F>>`) that stores the chain of
	///   pending `bind` operations with O(1) snoc and O(1) amortized uncons.
	/// * A `PhantomData<A>` that tracks the concrete result type at the type level.
	///
	/// Because the view is fully type-erased, `bind` is uniformly O(1) for all cases:
	/// it simply appends the new continuation to the CatList without inspecting the view.
	///
	/// # Linear consumption invariant
	///
	/// `Free` values must be consumed exactly once, either by calling [`evaluate`](Free::evaluate),
	/// [`bind`](Free::bind), [`erase_type`](Free::erase_type), or by being dropped. The `view`
	/// field is wrapped in `Option` to enable a take-and-replace pattern (analogous to
	/// `Cell::take`) so that methods like `evaluate` and `Drop` can move the view out without
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
	///   loop. It requires the base functor `F` to implement [`Extract`], meaning each
	///   suspended layer can be "unwrapped" to reveal the next step. Use this when you want
	///   the final `A` value directly.
	/// * [`fold_free`](Free::fold_free) interprets the `Free` into a different monad `G` via
	///   a [`NaturalTransformation`] from `F` to `G`. Each suspended `F` layer is transformed
	///   into `G`, and the results are threaded together using [`MonadRec::tail_rec_m`] for
	///   stack-safe iteration. Use this when you want to translate the free structure into
	///   another effect (e.g., `Option`, `Result`, or a custom interpreter).
	#[document_type_parameters(
		"The base functor (must implement [`Extract`] and [`Functor`]). Many construction methods (`pure`, `bind`, `map`) only need `F: 'static` in principle, and functor-dependent methods (`wrap`, `lift_f`, `resume`, `fold_free`, `hoist_free`) only need `Functor`. The `Extract` bound is required at the struct level because the custom `Drop` implementation calls [`Extract::extract`] to iteratively dismantle `Suspend` nodes without overflowing the stack.",
		"The result type."
	)]
	///
	pub struct Free<F, A>
	where
		F: Extract + Functor + 'static,
		A: 'static, {
		/// The current step of the computation (type-erased).
		view: Option<FreeView<F>>,
		/// The queue of pending continuations.
		continuations: CatList<Continuation<F>>,
		/// Phantom data tracking the concrete result type.
		_marker: PhantomData<A>,
	}

	// ── Construction and composition ──────────────────────────────────
	//
	// Methods in this block only need `F: 'static` in principle; they
	// never call `Functor::map` or `Extract::extract`. The `Extract +
	// Functor` bounds are inherited from the struct definition, which
	// requires them for stack-safe `Drop` of `Suspend` nodes.
	//
	// Relaxing these bounds was investigated and is not feasible: Rust
	// requires `Drop` impl bounds to match the struct bounds exactly,
	// and the `Drop` impl needs `Extract` to iteratively dismantle
	// `Suspend` nodes without stack overflow. Alternative approaches
	// (ManuallyDrop with leaks, type-erased drop functions, split
	// structs) all introduce unsoundness, leaks, or significant
	// per-node overhead for marginal gain. This is a Rust-specific
	// limitation; PureScript/Haskell avoid it via GC. The workaround
	// for non-Extract functors is `fold_free`, which interprets into
	// any `MonadRec` target.

	#[document_type_parameters("The base functor.", "The result type.")]
	#[document_parameters("The Free monad instance to operate on.")]
	impl<F, A> Free<F, A>
	where
		F: Extract + Functor + 'static,
		A: 'static,
	{
		/// Extracts the view and continuations, leaving `self` in a consumed
		/// state (view `None`, continuations empty).
		///
		/// Uses `Option::take` for the view and `std::mem::take` for the
		/// continuations, both of which leave valid sentinel values behind.
		/// The custom `Drop` implementation handles the consumed state correctly.
		#[document_signature]
		#[document_returns("A tuple of the view and continuation queue, moved out of this `Free`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::*,
		/// };
		///
		/// // `take_parts` is internal; `evaluate` is the public API that uses it.
		/// let free = Free::<ThunkBrand, _>::pure(42);
		/// assert_eq!(free.evaluate(), 42);
		/// ```
		fn take_parts(&mut self) -> (Option<FreeView<F>>, CatList<Continuation<F>>) {
			let view = self.view.take();
			let conts = std::mem::take(&mut self.continuations);
			(view, conts)
		}

		/// Changes the phantom type parameter without adding any continuations.
		///
		/// This is an internal-only operation used by `bind` continuations where
		/// the caller guarantees the stored type matches the new phantom. Unlike
		/// the public [`erase_type`](Free::erase_type), this does not append a
		/// rebox continuation, so the caller must ensure type safety.
		#[document_signature]
		#[document_type_parameters("The target phantom type.")]
		#[document_returns("The same `Free` with a different phantom type parameter.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::*,
		/// };
		///
		/// // `cast_phantom` is internal; `bind` is the public API that uses it.
		/// let free = Free::<ThunkBrand, _>::pure(42).bind(|x| Free::pure(x + 1));
		/// assert_eq!(free.evaluate(), 43);
		/// ```
		fn cast_phantom<B: 'static>(mut self) -> Free<F, B> {
			let (view, conts) = self.take_parts();
			Free {
				view,
				continuations: conts,
				_marker: PhantomData,
			}
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
			Free {
				view: Some(FreeView::Return(Box::new(a) as TypeErasedValue)),
				continuations: CatList::empty(),
				_marker: PhantomData,
			}
		}

		/// Monadic bind with O(1) complexity.
		///
		/// This is uniformly O(1) for all cases: the new continuation is simply
		/// appended to the CatList without inspecting the view.
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
				// INVARIANT: type is maintained by internal invariant; mismatch indicates a bug
				#[allow(clippy::expect_used)]
				let a: A = *val.downcast().expect("Type mismatch in Free::bind");
				let free_b: Free<F, B> = f(a);
				// Use cast_phantom (not erase_type) to avoid adding a rebox
				// continuation. The view already stores TypeErasedValue, and
				// the next continuation in the chain knows the correct type to
				// downcast to.
				free_b.cast_phantom()
			});

			// Uniformly O(1): just move the view and snoc the continuation.
			// The view is already type-erased, so it can be moved directly.
			let (view, conts) = self.take_parts();
			Free {
				view,
				continuations: conts.snoc(erased_f),
				_marker: PhantomData,
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

		/// Converts to type-erased form.
		///
		/// With the CatList-paired representation, the view is already type-erased,
		/// so this operation simply changes the phantom type parameter.
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
			let (view, conts) = self.take_parts();
			// Append a continuation that re-boxes the value so that the outer
			// phantom type (TypeErasedValue = Box<dyn Any>) matches the stored
			// type. Without this, evaluate would try to downcast Box<dyn Any>
			// containing A to Box<dyn Any>, which fails when A != Box<dyn Any>.
			let rebox_cont: Continuation<F> = Box::new(|val: TypeErasedValue| Free {
				view: Some(FreeView::Return(Box::new(val) as TypeErasedValue)),
				continuations: CatList::empty(),
				_marker: PhantomData,
			});
			Free {
				view,
				continuations: conts.snoc(rebox_cont),
				_marker: PhantomData,
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
	}

	// ── Functor-dependent operations ──────────────────────────────────
	//
	// Methods in this block call `Functor::map` (`F::map`) and thus
	// require `F: Functor`. They do NOT call `Extract::extract`; the
	// `Extract` bound is inherited from the struct definition.

	#[document_type_parameters("The base functor.", "The result type.")]
	#[document_parameters("The Free monad instance to operate on.")]
	impl<F, A> Free<F, A>
	where
		F: Extract + Functor + 'static,
		A: 'static,
	{
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
			// Type-erase the inner Free values in the functor using F::map.
			let erased_fa = F::map(
				|inner: Free<F, A>| -> Free<F, TypeErasedValue> { inner.cast_phantom() },
				fa,
			);
			Free {
				view: Some(FreeView::Suspend(erased_fa)),
				continuations: CatList::empty(),
				_marker: PhantomData,
			}
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

		/// Decomposes this `Free` computation into a single [`FreeStep`].
		///
		/// Iteratively applies pending continuations until the computation
		/// reaches either a final value ([`FreeStep::Done`]) or a suspended
		/// functor layer ([`FreeStep::Suspended`]). In the `Suspended` case,
		/// all remaining continuations are reattached to the inner `Free`
		/// values via [`Functor::map`], so the caller receives a fully
		/// self-contained layer that can be further interpreted.
		///
		/// This is the shared core that both [`evaluate`](Free::evaluate) and
		/// [`resume`](Free::resume) delegate to.
		#[document_signature]
		///
		#[document_returns(
			"[`FreeStep::Done(a)`](FreeStep::Done) if the computation is complete, or [`FreeStep::Suspended(fa)`](FreeStep::Suspended) if it is suspended in the functor `F`."
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
		/// // Pure value yields Done
		/// let free = Free::<ThunkBrand, _>::pure(42);
		/// match free.to_view() {
		/// 	FreeStep::Done(a) => assert_eq!(a, 42),
		/// 	FreeStep::Suspended(_) => panic!("expected Done"),
		/// }
		///
		/// // Wrapped value yields Suspended
		/// let free = Free::<ThunkBrand, _>::wrap(Thunk::new(|| Free::pure(99)));
		/// assert!(matches!(free.to_view(), FreeStep::Suspended(_)));
		/// ```
		#[allow(clippy::expect_used)]
		pub fn to_view(mut self) -> FreeStep<F, A> {
			let (view, continuations) = self.take_parts();

			// INVARIANT: Free values are used exactly once; double consumption indicates a bug
			let mut current_view = view.expect("Free value already consumed");
			let mut conts = continuations;

			loop {
				match current_view {
					FreeView::Return(val) => {
						match conts.uncons() {
							Some((continuation, rest)) => {
								let mut next = continuation(val);
								let (next_view, next_conts) = next.take_parts();
								// INVARIANT: continuation returns a valid Free
								current_view =
									next_view.expect("Free value already consumed (continuation)");
								conts = next_conts.append(rest);
							}
							None => {
								// No more continuations; we have a final pure value.
								// INVARIANT: type is maintained by internal invariant
								return FreeStep::Done(
									*val.downcast::<A>()
										.expect("Type mismatch in Free::to_view final downcast"),
								);
							}
						}
					}

					FreeView::Suspend(fa) => {
						// We have a suspended computation. Rebuild the
						// Free<F, A> from the type-erased Free<F, TypeErasedValue>
						// by re-attaching the remaining continuations.

						// Append a final downcast continuation that converts
						// TypeErasedValue back to the concrete type A.
						let downcast_cont: Continuation<F> =
							Box::new(move |val: TypeErasedValue| {
								// INVARIANT: type is maintained by internal invariant
								let a: A = *val
									.downcast()
									.expect("Type mismatch in Free::to_view downcast");
								Free::<F, A>::pure(a).cast_phantom()
							});
						let all_conts = conts.snoc(downcast_cont);

						// Use Cell to move the CatList into the Fn closure (called exactly once).
						let remaining = std::cell::Cell::new(Some(all_conts));
						let typed_fa = F::map(
							move |mut inner_free: Free<F, TypeErasedValue>| {
								// INVARIANT: functors call map exactly once per element
								let conts_for_inner = remaining
									.take()
									.expect("Free::to_view map called more than once");
								let (v, c) = inner_free.take_parts();
								Free {
									view: v,
									continuations: c.append(conts_for_inner),
									_marker: PhantomData,
								}
							},
							fa,
						);
						return FreeStep::Suspended(typed_fa);
					}
				}
			}
		}

		/// Decomposes this `Free` computation into one step.
		///
		/// Returns `Ok(a)` if the computation is a pure value, or
		/// `Err(f_free)` if the computation is suspended in the functor `F`,
		/// where `f_free` contains the next `Free` computation wrapped in `F`.
		///
		/// Delegates to [`to_view`](Free::to_view) and converts the resulting
		/// [`FreeStep`] into a `Result`.
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
			match self.to_view() {
				FreeStep::Done(a) => Ok(a),
				FreeStep::Suspended(fa) => Err(fa),
			}
		}

		/// Interprets this `Free` monad into a target monad `G` using a natural transformation.
		///
		/// This is the standard `foldFree` operation from Haskell/PureScript. It uses
		/// [`MonadRec::tail_rec_m`] to iteratively process the free structure, applying the
		/// natural transformation at each suspended layer to convert from functor `F` into
		/// monad `G`.
		///
		/// For `Pure(a)`, returns `G::pure(ControlFlow::Break(a))`.
		/// For `Wrap(fa)`, applies the natural transformation to get `G<Free<F, A>>`,
		/// then maps to `ControlFlow::Continue` to continue the iteration.
		///
		/// ### Stack Safety
		///
		/// This implementation is stack-safe as long as the target monad `G` implements
		/// [`MonadRec`] with a stack-safe `tail_rec_m`. The iteration is driven by
		/// `G::tail_rec_m` rather than actual recursion, so deeply nested `Free`
		/// structures will not overflow the stack.
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
			G: MonadRec + 'static, {
			G::tail_rec_m(
				move |free: Free<F, A>| match free.resume() {
					Ok(a) => G::pure(ControlFlow::Break(a)),
					Err(fa) => {
						// fa: F<Free<F, A>>
						// Transform F<Free<F, A>> into G<Free<F, A>> using the natural
						// transformation, then map to ControlFlow::Continue to continue iteration.
						let ga: Apply!(
							<G as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
								'static,
								Free<F, A>,
							>
						) = nt.transform(fa);
						G::map(|inner_free: Free<F, A>| ControlFlow::Continue(inner_free), ga)
					}
				},
				self,
			)
		}

		/// Transforms the functor layer of this `Free` monad using a natural transformation.
		///
		/// Converts `Free<F, A>` into `Free<G, A>` by applying a natural transformation
		/// to each suspended functor layer. This is the standard `hoistFree` operation from
		/// PureScript/Haskell.
		///
		/// ### Stack Safety
		///
		/// This method is stack-safe for arbitrarily deep `Suspend` chains. Rather than
		/// recursively mapping `hoist_free` over inner layers during construction, it uses
		/// [`lift_f`](Free::lift_f) and [`bind`](Free::bind) to defer each layer's
		/// transformation into a continuation stored in the `CatList`. The actual work
		/// happens inside [`evaluate`](Free::evaluate)'s iterative loop, so the call
		/// stack depth is O(1) regardless of how many `Suspend` layers the `Free` contains.
		#[document_signature]
		///
		#[document_type_parameters("The target functor brand.")]
		///
		#[document_parameters("The natural transformation from `F` to `G`.")]
		///
		#[document_returns("A `Free` computation over functor `G` with the same result.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::NaturalTransformation,
		/// 	types::*,
		/// };
		///
		/// // Identity natural transformation (Thunk to Thunk)
		/// #[derive(Clone)]
		/// struct ThunkId;
		/// impl NaturalTransformation<ThunkBrand, ThunkBrand> for ThunkId {
		/// 	fn transform<'a, A: 'a>(
		/// 		&self,
		/// 		fa: Thunk<'a, A>,
		/// 	) -> Thunk<'a, A> {
		/// 		fa
		/// 	}
		/// }
		///
		/// let free = Free::<ThunkBrand, _>::lift_f(Thunk::new(|| 42));
		/// let hoisted: Free<ThunkBrand, i32> = free.hoist_free(ThunkId);
		/// assert_eq!(hoisted.evaluate(), 42);
		/// ```
		pub fn hoist_free<G: Extract + Functor + 'static>(
			self,
			nt: impl NaturalTransformation<F, G> + Clone + 'static,
		) -> Free<G, A> {
			match self.resume() {
				Ok(a) => Free::pure(a),
				Err(fa) => {
					// fa: F<Free<F, A>>
					// Transform F<Free<F, A>> into G<Free<F, A>>
					let nt_clone = nt.clone();
					let ga = nt.transform(fa);
					// Lift G<Free<F, A>> into Free<G, Free<F, A>> using lift_f,
					// then bind to recursively hoist. The bind closure is stored
					// in the CatList and only executed during evaluate's iterative
					// loop, so this does not grow the call stack.
					Free::<G, Free<F, A>>::lift_f(ga)
						.bind(move |inner: Free<F, A>| inner.hoist_free(nt_clone))
				}
			}
		}

		/// Interprets `Free<F, A>` into `Free<G, A>` by substituting each
		/// suspended `F` layer with a `Free<G, _>` computation.
		///
		/// This is the standard `substFree` from PureScript. It is similar to
		/// [`hoist_free`](Free::hoist_free) but more powerful: instead of a
		/// plain natural transformation `F ~> G`, the callback returns
		/// `Free<G, _>`, allowing each `F` layer to expand into an entire
		/// `Free<G>` sub-computation.
		///
		/// ### Stack Safety
		///
		/// This method is stack-safe for arbitrarily deep `Suspend` chains,
		/// following the same pattern as [`hoist_free`](Free::hoist_free). The
		/// recursive `substitute_free` call is inside a [`bind`](Free::bind)
		/// closure deferred into the CatList. Each invocation does O(1) work
		/// (one [`to_view`](Free::to_view), one natural transformation, one
		/// `bind`) and returns immediately. The actual work happens inside
		/// [`evaluate`](Free::evaluate)'s iterative loop, so the call stack
		/// depth is O(1) regardless of `Suspend` depth.
		#[document_signature]
		///
		#[document_type_parameters("The target functor brand.")]
		///
		#[document_parameters(
			"A function that transforms each suspended `F` layer into a `Free<G, _>` computation."
		)]
		///
		#[document_returns(
			"A `Free<G, A>` computation where every `F` layer has been substituted."
		)]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::NaturalTransformation,
		/// 	types::*,
		/// };
		///
		/// // Identity substitution: each Thunk layer becomes a Free<ThunkBrand, _>
		/// let free = Free::<ThunkBrand, _>::lift_f(Thunk::new(|| 42));
		/// let substituted: Free<ThunkBrand, i32> =
		/// 	free.substitute_free(|thunk: Thunk<'static, Free<ThunkBrand, i32>>| {
		/// 		Free::<ThunkBrand, _>::lift_f(thunk)
		/// 	});
		/// assert_eq!(substituted.evaluate(), 42);
		/// ```
		pub fn substitute_free<G: Extract + Functor + 'static>(
			self,
			nt: impl Fn(
				Apply!(
					<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, Free<F, A>>
				),
			) -> Free<G, Free<F, A>>
			+ Clone
			+ 'static,
		) -> Free<G, A> {
			match self.to_view() {
				FreeStep::Done(a) => Free::pure(a),
				FreeStep::Suspended(fa) => {
					// nt transforms the F-layer into Free<G, Free<F, A>>
					let free_g: Free<G, Free<F, A>> = nt.clone()(fa);
					// bind to recurse: for each inner Free<F, A>, continue substituting
					let nt_clone = nt;
					free_g.bind(move |inner| inner.substitute_free(nt_clone.clone()))
				}
			}
		}
	}

	// ── Evaluation (requires Extract) ─────────────────────────────────
	//
	// This method calls `Extract::extract` and thus genuinely requires
	// `F: Extract + Functor`. The `Drop` implementation also needs
	// `Extract` for the same reason (stack-safe dismantling of `Suspend`
	// nodes).

	#[document_type_parameters("The base functor.", "The result type.")]
	#[document_parameters("The Free monad instance to operate on.")]
	impl<F, A> Free<F, A>
	where
		F: Extract + Functor + 'static,
		A: 'static,
	{
		/// Executes the Free computation, returning the final result.
		///
		/// Uses [`to_view`](Free::to_view) to iteratively step through the
		/// computation: each `Suspended` layer is unwrapped via
		/// [`Extract::extract`] and fed back into the loop, while `Done`
		/// returns the final value. The continuation chain is collapsed by
		/// `to_view`, so this outer loop only sees functor layers.
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
			let mut current = self;
			loop {
				match current.to_view() {
					FreeStep::Done(a) => return a,
					FreeStep::Suspended(fa) => {
						current = <F as Extract>::extract(fa);
					}
				}
			}
		}
	}

	#[document_type_parameters("The base functor.", "The result type.")]
	#[document_parameters("The free monad instance to drop.")]
	impl<F, A> Drop for Free<F, A>
	where
		F: Extract + Functor + 'static,
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
			// Take the view out so we can iteratively dismantle the chain
			// instead of relying on recursive Drop, which would overflow the stack
			// for deep computations (both continuation chains and Suspend chains).
			let mut worklist: Vec<FreeView<F>> = Vec::new();

			if let Some(view) = self.view.take() {
				worklist.push(view);
			}

			// Drain the top-level continuations iteratively. Each
			// continuation is a Box<dyn FnOnce> that may capture Free
			// values. By consuming them one at a time via uncons, we
			// let each boxed closure drop without building stack depth.
			let mut top_conts = std::mem::take(&mut self.continuations);
			while let Some((_continuation, rest)) = top_conts.uncons() {
				top_conts = rest;
			}

			while let Some(view) = worklist.pop() {
				match view {
					FreeView::Return(_) => {
						// Trivially dropped, no nested Free values.
					}

					FreeView::Suspend(fa) => {
						// The functor layer contains a `Free<F, TypeErasedValue>` inside.
						// If we let it drop recursively, deeply nested Suspend chains will
						// overflow the stack. Instead, we use `Extract::extract`
						// to eagerly extract the inner `Free`, then push its view onto
						// the worklist for iterative dismantling.
						let mut extracted: Free<F, TypeErasedValue> = <F as Extract>::extract(fa);
						if let Some(inner_view) = extracted.view.take() {
							worklist.push(inner_view);
						}
						// Drain the extracted node's continuations iteratively.
						let mut inner_conts = std::mem::take(&mut extracted.continuations);
						while let Some((_continuation, rest)) = inner_conts.uncons() {
							inner_conts = rest;
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

	/// Tests `Free::fold_free` stack safety with a deeply nested `Wrap` chain.
	///
	/// **What it tests:** Verifies that `fold_free` does not overflow the stack on deep structures.
	/// **How it tests:** Builds a chain of 100,000 `Wrap` layers and folds with `ThunkToOption`.
	#[test]
	fn test_free_fold_free_stack_safety() {
		let depth = 100_000;
		let mut free = Free::<ThunkBrand, i32>::pure(0);
		for _ in 0 .. depth {
			free = Free::wrap(Thunk::new(move || free));
		}
		let result: Option<i32> = free.fold_free::<OptionBrand>(ThunkToOption);
		assert_eq!(result, Some(0));
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

	// ── hoist_free tests ──

	/// An identity natural transformation from `ThunkBrand` to `ThunkBrand`.
	///
	/// Used to test `hoist_free` with the same source and target functor.
	#[derive(Clone)]
	struct ThunkId;
	impl NaturalTransformation<ThunkBrand, ThunkBrand> for ThunkId {
		fn transform<'a, A: 'a>(
			&self,
			fa: Thunk<'a, A>,
		) -> Thunk<'a, A> {
			fa
		}
	}

	/// A natural transformation that wraps a thunk in an extra layer.
	///
	/// Evaluates the original thunk eagerly and returns a new thunk producing
	/// that value, demonstrating the transformation is actually invoked.
	#[derive(Clone)]
	struct ThunkEager;
	impl NaturalTransformation<ThunkBrand, ThunkBrand> for ThunkEager {
		fn transform<'a, A: 'a>(
			&self,
			fa: Thunk<'a, A>,
		) -> Thunk<'a, A> {
			let val = fa.evaluate();
			Thunk::new(move || val)
		}
	}

	/// Tests `Free::hoist_free` on a pure value.
	///
	/// **What it tests:** Verifies that hoisting a pure computation preserves the value,
	/// since there are no `Wrap` layers for the natural transformation to act on.
	/// **How it tests:** Creates `Free::pure(42)`, hoists with `ThunkId`, evaluates.
	#[test]
	fn test_free_hoist_free_pure() {
		let free = Free::<ThunkBrand, _>::pure(42);
		let hoisted = free.hoist_free(ThunkId);
		assert_eq!(hoisted.evaluate(), 42);
	}

	/// Tests `Free::hoist_free` on a single `Wrap` layer.
	///
	/// **What it tests:** Verifies that the natural transformation is applied to the functor layer.
	/// **How it tests:** Lifts a thunk into Free, hoists with `ThunkId`, and evaluates.
	#[test]
	fn test_free_hoist_free_single_wrap() {
		let free = Free::<ThunkBrand, _>::lift_f(Thunk::new(|| 42));
		let hoisted = free.hoist_free(ThunkId);
		assert_eq!(hoisted.evaluate(), 42);
	}

	/// Tests `Free::hoist_free` with a transformation that eagerly evaluates.
	///
	/// **What it tests:** Verifies the natural transformation is actually invoked on each layer.
	/// **How it tests:** Uses `ThunkEager` which evaluates the thunk during transformation,
	/// proving the transformation runs rather than being a no-op.
	#[test]
	fn test_free_hoist_free_transformation_applied() {
		let free = Free::<ThunkBrand, _>::lift_f(Thunk::new(|| 42));
		let hoisted = free.hoist_free(ThunkEager);
		assert_eq!(hoisted.evaluate(), 42);
	}

	/// Tests `Free::hoist_free` with multiple `Wrap` layers.
	///
	/// **What it tests:** Verifies that each functor layer is transformed.
	/// **How it tests:** Builds a chain of three nested `Wrap` layers and hoists all of them.
	#[test]
	fn test_free_hoist_free_multiple_wraps() {
		let free = Free::<ThunkBrand, _>::wrap(Thunk::new(|| {
			Free::wrap(Thunk::new(|| Free::wrap(Thunk::new(|| Free::pure(7)))))
		}));
		let hoisted = free.hoist_free(ThunkId);
		assert_eq!(hoisted.evaluate(), 7);
	}

	/// Tests `Free::hoist_free` with bind chains.
	///
	/// **What it tests:** Verifies that bind chains are preserved through hoisting.
	/// **How it tests:** Chains `lift_f` and `bind`, hoists, and checks the final result.
	#[test]
	fn test_free_hoist_free_with_binds() {
		let free = Free::<ThunkBrand, _>::lift_f(Thunk::new(|| 10))
			.bind(|x| Free::<ThunkBrand, _>::lift_f(Thunk::new(move || x * 2)))
			.bind(|x| Free::pure(x + 5));
		let hoisted = free.hoist_free(ThunkId);
		assert_eq!(hoisted.evaluate(), 25);
	}

	/// Tests `Free::hoist_free` preserves values across a deep wrap chain.
	///
	/// **What it tests:** Verifies hoisting works on moderately deep wrap chains.
	/// **How it tests:** Builds 100 nested `Wrap` layers and hoists them all.
	#[test]
	fn test_free_hoist_free_deep_wraps() {
		let depth = 100;
		let mut free = Free::<ThunkBrand, i32>::pure(99);
		for _ in 0 .. depth {
			free = Free::wrap(Thunk::new(move || free));
		}
		let hoisted = free.hoist_free(ThunkId);
		assert_eq!(hoisted.evaluate(), 99);
	}

	/// Tests `Free::hoist_free` is stack-safe with 100k nested `Wrap` layers.
	///
	/// **What it tests:** Verifies that `hoist_free` does not overflow the stack on deeply
	/// nested `Suspend` chains. The old recursive implementation would overflow at this depth.
	/// **How it tests:** Builds 100,000 nested `Wrap` layers, hoists with `ThunkId`, and evaluates.
	#[test]
	fn test_free_hoist_free_stack_safety() {
		let depth = 100_000;
		let mut free = Free::<ThunkBrand, i32>::pure(42);
		for _ in 0 .. depth {
			free = Free::wrap(Thunk::new(move || free));
		}
		let hoisted = free.hoist_free(ThunkId);
		assert_eq!(hoisted.evaluate(), 42);
	}

	// ── to_view tests ──

	/// Tests `Free::to_view` on a pure value.
	///
	/// **What it tests:** Verifies that `to_view` on a pure computation returns `FreeStep::Done`.
	/// **How it tests:** Creates `Free::pure(42)` and checks that `to_view` produces `Done(42)`.
	#[test]
	fn test_free_to_view_pure() {
		let free = Free::<ThunkBrand, _>::pure(42);
		match free.to_view() {
			FreeStep::Done(a) => assert_eq!(a, 42),
			FreeStep::Suspended(_) => panic!("expected Done"),
		}
	}

	/// Tests `Free::to_view` on a wrapped value.
	///
	/// **What it tests:** Verifies that `to_view` on a suspended computation returns
	/// `FreeStep::Suspended`.
	/// **How it tests:** Wraps a `Free::pure(99)` in a `Thunk` and checks that `to_view`
	/// produces `Suspended`, then evaluates the inner layer to verify the value.
	#[test]
	fn test_free_to_view_wrapped() {
		let free = Free::<ThunkBrand, _>::wrap(Thunk::new(|| Free::pure(99)));
		match free.to_view() {
			FreeStep::Done(_) => panic!("expected Suspended"),
			FreeStep::Suspended(thunk) => {
				let inner: Free<ThunkBrand, i32> = thunk.evaluate();
				assert_eq!(inner.evaluate(), 99);
			}
		}
	}

	/// Tests `Free::to_view` on a bind chain that resolves to a pure value.
	///
	/// **What it tests:** Verifies that `to_view` collapses a chain of `bind` operations
	/// and returns `Done` when all continuations produce pure values.
	/// **How it tests:** Creates `pure(10).bind(|x| pure(x + 5))` and checks that `to_view`
	/// returns `Done(15)`.
	#[test]
	fn test_free_to_view_bind_chain_done() {
		let free = Free::<ThunkBrand, _>::pure(10).bind(|x: i32| Free::pure(x + 5));
		match free.to_view() {
			FreeStep::Done(a) => assert_eq!(a, 15),
			FreeStep::Suspended(_) => panic!("expected Done"),
		}
	}

	/// Tests `Free::to_view` on a bind chain that resolves to a suspended layer.
	///
	/// **What it tests:** Verifies that `to_view` collapses bind continuations until it
	/// reaches a `Suspend`, then returns `Suspended` with continuations reattached.
	/// **How it tests:** Creates `wrap(...).bind(|x| pure(x + 5))` and checks that `to_view`
	/// returns `Suspended`, then evaluates the result to verify the value is correct.
	#[test]
	fn test_free_to_view_bind_chain_suspended() {
		let free = Free::<ThunkBrand, _>::wrap(Thunk::new(|| Free::pure(10)))
			.bind(|x: i32| Free::pure(x + 5));
		match free.to_view() {
			FreeStep::Done(_) => panic!("expected Suspended"),
			FreeStep::Suspended(thunk) => {
				// The inner Free should have the bind continuation reattached,
				// so evaluating the full thing should yield 15.
				let inner: Free<ThunkBrand, i32> = thunk.evaluate();
				assert_eq!(inner.evaluate(), 15);
			}
		}
	}

	/// Tests that `resume` delegates correctly to `to_view` after refactoring.
	///
	/// **What it tests:** Verifies that `resume` still produces the same results as before
	/// the refactoring to use `to_view`.
	/// **How it tests:** Exercises both `Ok` and `Err` paths and verifies values.
	#[test]
	fn test_free_resume_delegates_to_view() {
		// Pure -> Ok
		let free = Free::<ThunkBrand, _>::pure(42);
		assert!(matches!(free.resume(), Ok(42)));

		// Wrap -> Err
		let free = Free::<ThunkBrand, _>::wrap(Thunk::new(|| Free::pure(99)));
		let err = free.resume().unwrap_err();
		let inner: Free<ThunkBrand, i32> = err.evaluate();
		assert_eq!(inner.evaluate(), 99);

		// Bind chain -> Ok
		let free = Free::<ThunkBrand, _>::pure(10).bind(|x: i32| Free::pure(x + 5));
		assert!(matches!(free.resume(), Ok(15)));
	}

	/// Tests that `evaluate` delegates correctly to `to_view` after refactoring.
	///
	/// **What it tests:** Verifies that `evaluate` still produces the same results as before
	/// the refactoring to use `to_view`.
	/// **How it tests:** Exercises pure, wrapped, and bind-chain computations.
	#[test]
	fn test_free_evaluate_delegates_to_view() {
		assert_eq!(Free::<ThunkBrand, _>::pure(42).evaluate(), 42);

		let free = Free::<ThunkBrand, _>::wrap(Thunk::new(|| Free::pure(99)));
		assert_eq!(free.evaluate(), 99);

		let free =
			Free::<ThunkBrand, _>::pure(10).bind(|x| Free::pure(x + 5)).bind(|x| Free::pure(x * 2));
		assert_eq!(free.evaluate(), 30);
	}

	// ── substitute_free tests ──

	/// Tests `substitute_free` with an identity-like substitution.
	///
	/// **What it tests:** Verifies that substituting each `F` layer with `lift_f` (wrapping it
	/// back into `Free<F, _>`) preserves the computation result.
	/// **How it tests:** Lifts a thunk into Free, substitutes with `lift_f`, and evaluates.
	#[test]
	fn test_free_substitute_free_identity() {
		let free = Free::<ThunkBrand, _>::lift_f(Thunk::new(|| 42));
		let substituted = free.substitute_free(|thunk: Thunk<'static, Free<ThunkBrand, i32>>| {
			Free::<ThunkBrand, _>::lift_f(thunk)
		});
		assert_eq!(substituted.evaluate(), 42);
	}

	/// Tests `substitute_free` on a pure value.
	///
	/// **What it tests:** Verifies that `substitute_free` on a pure computation returns
	/// the same value without invoking the natural transformation.
	/// **How it tests:** Creates `Free::pure(42)` and substitutes, checking the result.
	#[test]
	fn test_free_substitute_free_pure() {
		let free = Free::<ThunkBrand, _>::pure(42);
		let substituted: Free<ThunkBrand, i32> =
			free.substitute_free(|_thunk: Thunk<'static, Free<ThunkBrand, i32>>| {
				panic!("should not be called on a pure value")
			});
		assert_eq!(substituted.evaluate(), 42);
	}

	/// Tests `substitute_free` with a chain of binds.
	///
	/// **What it tests:** Verifies that `substitute_free` correctly handles computations
	/// with bind chains.
	/// **How it tests:** Creates a computation with `lift_f` and `bind`, substitutes, and
	/// checks the final result.
	#[test]
	fn test_free_substitute_free_with_binds() {
		let free = Free::<ThunkBrand, _>::lift_f(Thunk::new(|| 10))
			.bind(|x| Free::<ThunkBrand, _>::lift_f(Thunk::new(move || x * 2)))
			.bind(|x| Free::pure(x + 5));
		let substituted: Free<ThunkBrand, i32> =
			free.substitute_free(|thunk: Thunk<'static, Free<ThunkBrand, i32>>| {
				Free::<ThunkBrand, _>::lift_f(thunk)
			});
		assert_eq!(substituted.evaluate(), 25);
	}

	/// Tests `substitute_free` that expands each layer into multiple layers.
	///
	/// **What it tests:** Verifies that `substitute_free` can expand each `F` layer into
	/// an entire `Free<G, _>` sub-computation (not just a single layer).
	/// **How it tests:** Substitutes each thunk layer with a double-wrapped computation
	/// and verifies the final result is correct.
	#[test]
	fn test_free_substitute_free_expands_layers() {
		let free = Free::<ThunkBrand, _>::lift_f(Thunk::new(|| 7));
		// Each thunk layer becomes a double-wrapped Free
		let substituted: Free<ThunkBrand, i32> =
			free.substitute_free(|thunk: Thunk<'static, Free<ThunkBrand, i32>>| {
				// Wrap the original thunk layer, then add an extra wrap layer
				let inner = Free::<ThunkBrand, _>::lift_f(thunk);
				Free::wrap(Thunk::new(move || inner))
			});
		assert_eq!(substituted.evaluate(), 7);
	}
}
