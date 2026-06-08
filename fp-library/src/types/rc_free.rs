//! Stack-safe Free monad with `Rc`-shared continuations supporting multi-shot effects.
//!
//! [`RcFree`] mirrors [`Free`](crate::types::Free)'s "Reflection without Remorse"
//! structure (a [`RcCatList`](crate::types::RcCatList) of pending continuations
//! sitting beside a single type-erased view) but swaps the closure storage
//! from `Box<dyn FnOnce>` to `Rc<dyn Fn>` (matching what
//! [`FnBrand<RcBrand>`](crate::brands::FnBrand) resolves to). The `Fn`-shape
//! lets each stored continuation run more than once, which is the property
//! multi-shot effects like `Choose` and `Amb` need.
//!
//! The whole substrate lives behind an outer [`Rc`](std::rc::Rc) so cloning a program is
//! O(1) (refcount bump), matching the
//! [`RcCoyoneda`](crate::types::RcCoyoneda) cloning pattern. Operations
//! that extend the structure (`bind`, `map`, `lift_f`, ...) consume `self`
//! and either move out of the `Rc` (when uniquely owned) or clone the inner
//! state (when shared from a prior `clone()`).
//!
//! ## Trade-offs vs `Free`
//!
//! - **Multi-shot:** `RcFree`'s continuations are `Fn`, so a handler can
//!   invoke the same suspended continuation multiple times. `Free`'s
//!   `FnOnce` continuations cannot.
//! - **Clone:** `RcFree` is `Clone` in O(1).
//! - **Bind requires `A: Clone`:** the continuation queue feeds each
//!   intermediate result into the next stored `Fn`, but a single shared
//!   value cannot be moved twice. `bind` recovers an owned `A` from the
//!   stored `Rc<A>` cell on each call, which falls back to `Clone` when the
//!   cell is shared.
//! - **Allocation per bind:** [`bind`](RcFree::bind) wraps each user
//!   continuation in `Rc<dyn Fn(...)>` and snocs onto the
//!   [`RcCatList`](crate::types::RcCatList), so the per-bind cost is one `Rc`
//!   allocation plus the queue snoc.
//! - **Thread-safety:** `RcFree` is `!Send`. Use `ArcFree` for thread-safe
//!   contexts.
//!
//! ## When to use which
//!
//! Use [`Free`](crate::types::Free) when payloads are `'static` and effect
//! continuations are single-shot (the common case). Use `RcFree` when an
//! effect needs to drive its continuation more than once: `Choose`, `Amb`,
//! probabilistic / non-deterministic search, backtracking parsers.
//!
//! ## Drop behavior
//!
//! When the last `Rc` reference releases, the inner data's [`Drop`] runs
//! and iteratively dismantles a deep `Wrap` chain via
//! [`WrapDrop::drop`](crate::classes::WrapDrop::drop), mirroring
//! [`Free`](crate::types::Free)'s strategy.
//! Without this, deep `Suspend` chains stack-overflow during cleanup.

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			Apply,
			brands::RcFnBrand,
			classes::{
				Extract,
				Functor,
				LiftFn,
				NaturalTransformation,
				WrapDrop,
			},
			kinds::*,
			types::RcCatList,
		},
		fp_macros::*,
		std::{
			any::Any,
			marker::PhantomData,
			rc::Rc,
		},
	};

	/// Type-erased value carrying its concrete type at runtime via [`Any`].
	///
	/// `Rc<dyn Any>` (rather than `Box<dyn Any>`) so the inner state can
	/// participate in [`Clone`] without deep-copying the payload.
	pub type RcTypeErasedValue = Rc<dyn Any>;

	/// Type-erased continuation stored in the [`RcCatList`](crate::types::RcCatList)
	/// queue, equivalent to
	/// [`<RcFnBrand as CloneFn>::Of<'static, RcTypeErasedValue, RcFree<F, RcTypeErasedValue>>`](crate::brands::FnBrand).
	pub struct RcContinuation<F>(Rc<dyn Fn(RcTypeErasedValue) -> RcFree<F, RcTypeErasedValue>>)
	where
		F: WrapDrop + 'static;

	#[document_type_parameters("The base functor.")]
	#[document_parameters("The continuation to clone.")]
	impl<F> Clone for RcContinuation<F>
	where
		F: WrapDrop + 'static,
	{
		/// Clones the continuation by bumping the refcount on its `Rc`.
		#[document_signature]
		///
		#[document_returns("A clone of the continuation.")]
		#[document_examples(
			skip_call_check,
			reason = "Direct-call validation is skipped because RcContinuation is internal continuation storage; public bind constructs it and clone-based evaluation exercises it."
		)]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::*,
		/// };
		///
		/// // `RcContinuation` is internal; `bind` is the public API that constructs it.
		/// let free = RcFree::<IdentityBrand, _>::pure(1).bind(|x: i32| RcFree::pure(x + 1));
		/// assert_eq!(free.evaluate(), 2);
		/// ```
		fn clone(&self) -> Self {
			RcContinuation(Rc::clone(&self.0))
		}
	}

	#[document_type_parameters("The base functor.")]
	#[document_parameters("The continuation storage.")]
	impl<F> RcContinuation<F>
	where
		F: WrapDrop + 'static,
	{
		/// Creates an `Rc`-shared type-erased continuation.
		#[document_signature]
		#[document_parameters("The continuation function to store.")]
		#[document_returns("A type-erased `RcFree` continuation.")]
		#[document_examples(
			skip_call_check,
			reason = "Direct-call validation is skipped because RcContinuation is crate-private continuation storage; public bind and raw-transform tests exercise construction and invocation."
		)]
		///
		/// ```
		/// let value = 42;
		/// assert_eq!(value, 42);
		/// ```
		pub(crate) fn new(
			continuation: impl Fn(RcTypeErasedValue) -> RcFree<F, RcTypeErasedValue> + 'static
		) -> Self {
			RcContinuation(<RcFnBrand as LiftFn>::new(continuation))
		}

		/// Runs the stored continuation.
		#[document_signature]
		#[document_parameters("The type-erased value to pass to the continuation.")]
		#[document_returns("The next type-erased `RcFree` computation.")]
		#[document_examples(
			skip_call_check,
			reason = "Direct-call validation is skipped because RcContinuation is crate-private continuation storage; public bind and raw-transform tests exercise construction and invocation."
		)]
		///
		/// ```
		/// let value = 42;
		/// assert_eq!(value, 42);
		/// ```
		pub(crate) fn call(
			&self,
			value: RcTypeErasedValue,
		) -> RcFree<F, RcTypeErasedValue> {
			(self.0)(value)
		}
	}

	/// The internal view of an [`RcFree`] computation.
	///
	/// Mirrors [`FreeView`](crate::types::free::FreeView): either a pure
	/// value or a single suspended functor layer holding the next step.
	#[document_type_parameters("The base functor (must implement [`WrapDrop`]).")]
	pub enum RcFreeView<F>
	where
		F: WrapDrop + 'static, {
		/// A pure value (type-erased).
		Return(RcTypeErasedValue),
		/// A suspended functor layer holding the next step.
		Suspend(
			Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<F, RcTypeErasedValue>,
			>),
		),
	}

	#[document_type_parameters("The base functor (must implement [`WrapDrop`]).")]
	#[document_parameters("The view to clone.")]
	impl<F> Clone for RcFreeView<F>
	where
		F: WrapDrop + 'static,
		Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'static,
			RcFree<F, RcTypeErasedValue>,
		>): Clone,
	{
		/// Clones the view, sharing the type-erased value via `Rc::clone`
		/// and delegating to the underlying functor's `Clone` impl for the
		/// suspended layer.
		#[document_signature]
		///
		#[document_returns("A clone of the view.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::*,
		/// };
		///
		/// let free = RcFree::<IdentityBrand, _>::pure(42);
		/// let cloned = free.clone();
		/// assert_eq!(cloned.evaluate(), 42);
		/// ```
		fn clone(&self) -> Self {
			match self {
				RcFreeView::Return(val) => RcFreeView::Return(Rc::clone(val)),
				RcFreeView::Suspend(fa) => RcFreeView::Suspend(fa.clone()),
			}
		}
	}

	/// The result of stepping through an [`RcFree`] computation.
	///
	/// Mirror of [`FreeStep`](crate::types::FreeStep) for the `Rc`-shared
	/// substrate. Returned by [`RcFree::to_view`] and [`RcFree::peel_ref`].
	#[document_type_parameters("The base functor.", "The result type.")]
	pub enum RcFreeStep<F, A>
	where
		F: WrapDrop + 'static,
		A: 'static, {
		/// The computation completed with a final value.
		Done(A),
		/// The computation is suspended in the functor `F`. The inner
		/// `RcFree` values have all pending continuations reattached.
		Suspended(Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, RcFree<F, A>>)),
	}

	/// Raw single-step decomposition of an [`RcFree`] computation.
	///
	/// Unlike [`RcFreeStep`], the suspended layer carries
	/// `RcFree<F, RcTypeErasedValue>` payloads and keeps the shared
	/// continuation queue separate. This is for internal interpreters
	/// that need to select one branch before reattaching multi-shot
	/// continuations.
	#[document_type_parameters(
		"The base functor (must implement [`WrapDrop`]).",
		"The result type of the computation."
	)]
	#[cfg_attr(not(feature = "effects"), allow(dead_code))]
	pub(crate) enum RcFreeRawStep<F, A>
	where
		F: WrapDrop + 'static,
		A: 'static, {
		/// The computation completed with a final value.
		Done(A),
		/// The computation is suspended in the functor `F`, with
		/// pending continuations kept outside the layer.
		Suspended {
			/// The suspended functor layer with type-erased inner programs.
			layer: Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<F, RcTypeErasedValue>,
			>),
			/// The pending continuations that must be attached to the
			/// selected branch.
			continuations: RcCatList<RcContinuation<F>>,
		},
	}

	/// Inner state of an [`RcFree`]: view plus pending continuations.
	struct RcFreeInner<F, A>
	where
		F: WrapDrop + 'static,
		A: 'static, {
		view: Option<RcFreeView<F>>,
		continuations: RcCatList<RcContinuation<F>>,
		_marker: PhantomData<A>,
	}

	#[document_type_parameters("The base functor.", "The result type.")]
	#[document_parameters("The inner state to clone.")]
	impl<F, A> Clone for RcFreeInner<F, A>
	where
		F: WrapDrop + 'static,
		A: 'static,
		Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'static,
			RcFree<F, RcTypeErasedValue>,
		>): Clone,
	{
		/// Clones the inner state: the view via [`RcFreeView`]'s `Clone`,
		/// the continuation queue via [`RcCatList`](crate::types::RcCatList)'s
		/// `Clone` (each `Rc<dyn Fn>` cell becomes a refcount bump).
		#[document_signature]
		///
		#[document_returns("A clone of the inner state.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::*,
		/// };
		///
		/// // `RcFreeInner` is internal; `RcFree::clone` exposes the same effect.
		/// let free = RcFree::<IdentityBrand, _>::pure(42);
		/// let cloned = free.clone();
		/// assert_eq!(cloned.evaluate(), 42);
		/// ```
		fn clone(&self) -> Self {
			RcFreeInner {
				view: self.view.clone(),
				continuations: self.continuations.clone(),
				_marker: PhantomData,
			}
		}
	}

	#[document_type_parameters("The base functor.", "The result type.")]
	#[document_parameters("The inner state being dropped.")]
	impl<F, A> Drop for RcFreeInner<F, A>
	where
		F: WrapDrop + 'static,
		A: 'static,
	{
		/// Iteratively dismantles deep `Suspend` chains via
		/// [`WrapDrop::drop`], mirroring [`Free::drop`](crate::types::Free).
		#[document_signature]
		#[document_examples(
			skip_call_check,
			reason = "Direct-call validation is skipped because Drop::drop cannot be called directly from public examples; leaving the value to go out of scope exercises the destructor."
		)]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::*,
		/// };
		/// {
		/// 	let _free = RcFree::<IdentityBrand, _>::pure(42);
		/// } // drop called here
		/// // Reaching this point without panicking proves drop completed.
		/// let post_drop = RcFree::<IdentityBrand, _>::pure(7);
		/// assert!(matches!(post_drop.resume(), Ok(7)));
		/// ```
		fn drop(&mut self) {
			let mut worklist: Vec<RcFreeView<F>> = Vec::new();

			if let Some(view) = self.view.take() {
				worklist.push(view);
			}

			let mut top_conts = std::mem::take(&mut self.continuations);
			while let Some((_continuation, rest)) = top_conts.uncons() {
				top_conts = rest;
			}

			while let Some(view) = worklist.pop() {
				match view {
					RcFreeView::Return(_) => {
						// Trivially dropped, no nested `RcFree` values.
					}
					RcFreeView::Suspend(fa) => {
						// Consult `WrapDrop::drop` on the base functor to decide how
						// to dismantle the layer. `Some(extracted)` means F materially
						// holds an inner `RcFree`; if we hold the last reference, peel
						// its view and continue the worklist (otherwise leave other
						// holders to dismantle when they release). `None` lets the
						// layer drop in place, which is sound for brands that do not
						// materially store the inner `RcFree`.
						if let Some(extracted) =
							<F as WrapDrop>::drop::<RcFree<F, RcTypeErasedValue>>(fa)
							&& let Ok(mut owned) = Rc::try_unwrap(extracted.inner)
						{
							if let Some(inner_view) = owned.view.take() {
								worklist.push(inner_view);
							}
							let mut inner_conts = std::mem::take(&mut owned.continuations);
							while let Some((_continuation, rest)) = inner_conts.uncons() {
								inner_conts = rest;
							}
						}
					}
				}
			}
		}
	}

	/// Stack-safe Free monad with `Rc`-shared continuations.
	///
	/// Same internal shape as [`Free`](crate::types::Free) but with
	/// `Rc<dyn Fn>` continuations (matching what
	/// [`FnBrand<RcBrand>`](crate::brands::FnBrand) resolves to) instead of
	/// `Box<dyn FnOnce>`, plus an outer [`Rc`] wrapper so the whole program
	/// is cheaply cloneable. Multi-shot effects (`Choose`, `Amb`) drive the
	/// stored continuations more than once, with `Clone` exposing the
	/// program independently to each handler branch.
	#[document_type_parameters(
		"The base functor (must implement [`WrapDrop`]; methods that walk the spine additionally require [`Functor`], and `evaluate` / `lower_ref` additionally require [`Extract`]).",
		"The result type."
	)]
	pub struct RcFree<F, A>
	where
		F: WrapDrop + 'static,
		A: 'static, {
		inner: Rc<RcFreeInner<F, A>>,
	}

	#[document_type_parameters("The base functor.", "The result type.")]
	#[document_parameters("The `RcFree` instance to clone.")]
	impl<F, A> Clone for RcFree<F, A>
	where
		F: WrapDrop + 'static,
		A: 'static,
	{
		/// Clones the `RcFree` by bumping the refcount on the outer `Rc`.
		/// O(1).
		#[document_signature]
		///
		#[document_returns("A new `RcFree` representing an independent branch.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::*,
		/// };
		///
		/// let free = RcFree::<IdentityBrand, _>::pure(42);
		/// let branch = free.clone();
		/// assert_eq!(free.evaluate(), 42);
		/// assert_eq!(branch.evaluate(), 42);
		/// ```
		fn clone(&self) -> Self {
			RcFree {
				inner: Rc::clone(&self.inner),
			}
		}
	}

	#[document_type_parameters("The base functor.", "The result type.")]
	#[document_parameters("The `RcFree` instance.")]
	impl<F, A> RcFree<F, A>
	where
		F: WrapDrop + 'static,
		A: 'static,
	{
		/// Constructs an `RcFree` from owned inner state.
		#[document_signature]
		///
		#[document_parameters("The inner state to wrap.")]
		///
		#[document_returns("A new `RcFree` wrapping the inner state in an `Rc`.")]
		///
		#[document_examples(
			skip_call_check,
			reason = "Direct-call validation is skipped because from_inner is a private constructor helper; public pure and wrap exercise it while keeping inner-state construction internal."
		)]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::*,
		/// };
		///
		/// // `from_inner` is internal; `pure` is the public API that uses it.
		/// let free = RcFree::<IdentityBrand, _>::pure(42);
		/// assert_eq!(free.evaluate(), 42);
		/// ```
		fn from_inner(inner: RcFreeInner<F, A>) -> Self {
			RcFree {
				inner: Rc::new(inner),
			}
		}

		/// Acquires owned access to the inner state, cloning the shared
		/// state when the outer `Rc` is not unique.
		#[document_signature]
		///
		#[document_returns("Owned inner state, either moved out or cloned.")]
		///
		#[document_examples(
			skip_call_check,
			reason = "Direct-call validation is skipped because into_inner_owned is a private ownership helper; public bind, to_view, evaluate, and lower_ref exercise it."
		)]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::*,
		/// };
		///
		/// // `into_inner_owned` is internal; `bind` is the public API that uses it.
		/// let free = RcFree::<IdentityBrand, _>::pure(1).bind(|x: i32| RcFree::pure(x + 1));
		/// assert_eq!(free.evaluate(), 2);
		/// ```
		fn into_inner_owned(self) -> RcFreeInner<F, A>
		where
			Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<F, RcTypeErasedValue>,
			>): Clone, {
			Rc::try_unwrap(self.inner).unwrap_or_else(|shared| (*shared).clone())
		}

		/// Creates a pure `RcFree` value.
		#[document_signature]
		///
		#[document_parameters("The value to wrap.")]
		///
		#[document_returns("An `RcFree` computation that produces `a`.")]
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
		/// let free = RcFree::<IdentityBrand, _>::pure(42);
		/// assert_eq!(free.evaluate(), 42);
		/// ```
		pub fn pure(a: A) -> Self {
			RcFree::from_inner(RcFreeInner {
				view: Some(RcFreeView::Return(Rc::new(a) as RcTypeErasedValue)),
				continuations: RcCatList::empty(),
				_marker: PhantomData,
			})
		}

		/// Changes the phantom type parameter without adding any
		/// continuations. Internal use only; the caller guarantees the
		/// stored type matches the new phantom.
		#[document_signature]
		///
		#[document_type_parameters("The target phantom type.")]
		///
		#[document_returns("The same `RcFree` with a different phantom type parameter.")]
		///
		#[document_examples(
			skip_call_check,
			reason = "Direct-call validation is skipped because cast_phantom is a private type-erasure helper; public bind and wrap exercise it while preserving the internal type invariant."
		)]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::*,
		/// };
		///
		/// // `cast_phantom` is internal; `bind` is the public API that uses it.
		/// let free = RcFree::<IdentityBrand, _>::pure(2).bind(|x: i32| RcFree::pure(x + 1));
		/// assert_eq!(free.evaluate(), 3);
		/// ```
		fn cast_phantom<B: 'static>(self) -> RcFree<F, B>
		where
			Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<F, RcTypeErasedValue>,
			>): Clone, {
			let mut owned = self.into_inner_owned();
			let view = owned.view.take();
			let continuations = std::mem::take(&mut owned.continuations);
			RcFree::from_inner(RcFreeInner {
				view,
				continuations,
				_marker: PhantomData,
			})
		}

		/// Erases the result type and adds a rebox continuation so typed
		/// operations can safely treat the result as [`RcTypeErasedValue`].
		#[document_signature]
		///
		#[document_returns(
			"An `RcFree` computation where the result type has been reboxed as erased."
		)]
		#[document_examples(
			skip_call_check,
			reason = "Direct-call validation is skipped because erase_type is crate-private raw-erasure plumbing; public bind, map, to_view, and evaluate exercise the continuation invariant."
		)]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::*,
		/// };
		///
		/// let free = RcFree::<IdentityBrand, _>::pure(42);
		/// assert_eq!(free.evaluate(), 42);
		/// ```
		#[cfg_attr(not(feature = "effects"), allow(dead_code))]
		pub(crate) fn erase_type(self) -> RcFree<F, RcTypeErasedValue>
		where
			Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<F, RcTypeErasedValue>,
			>): Clone, {
			let mut owned = self.into_inner_owned();
			let view = owned.view.take();
			let continuations = std::mem::take(&mut owned.continuations);
			let rebox_continuation = RcContinuation::new(|value: RcTypeErasedValue| {
				RcFree::from_inner(RcFreeInner {
					view: Some(RcFreeView::Return(Rc::new(value) as RcTypeErasedValue)),
					continuations: RcCatList::empty(),
					_marker: PhantomData,
				})
			});
			RcFree::from_inner(RcFreeInner {
				view,
				continuations: continuations.snoc(rebox_continuation),
				_marker: PhantomData,
			})
		}

		/// Casts this computation to its type-erased result form without
		/// changing the stored view or continuation queue.
		///
		/// This is used by continuation-aware scoped interpreters after
		/// they build a typed branch result and before they reattach the
		/// suspended outer continuation queue.
		#[document_signature]
		///
		#[document_returns("The same `RcFree` with a type-erased result parameter.")]
		#[document_examples(
			skip_call_check,
			reason = "Direct-call validation is skipped because cast_erased is crate-private raw-erasure plumbing; public bind, map, and evaluation paths cover the exposed behavior."
		)]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::*,
		/// };
		///
		/// let free = RcFree::<IdentityBrand, _>::pure(41).map(|value: i32| value + 1);
		/// assert_eq!(free.evaluate(), 42);
		/// ```
		#[cfg_attr(not(feature = "effects"), allow(dead_code))]
		pub(crate) fn cast_erased(self) -> RcFree<F, RcTypeErasedValue>
		where
			Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<F, RcTypeErasedValue>,
			>): Clone, {
			self.cast_phantom()
		}

		/// Appends pending continuations to a type-erased suspended branch
		/// and restores the concrete result type.
		///
		/// The appended downcast continuation is what makes the returned
		/// `RcFree<F, A>` type-correct after the branch has been stepped in
		/// type-erased form.
		#[document_signature]
		///
		#[document_parameters(
			"The type-erased branch selected by the interpreter.",
			"The pending continuation queue to append to that branch."
		)]
		#[document_returns(
			"An `RcFree` value whose selected branch will run the pending continuations."
		)]
		#[document_examples(
			skip_call_check,
			reason = "Direct-call validation is skipped because continue_from_erased is crate-private continuation plumbing; public resume and evaluate exercise the same reattachment path."
		)]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::*,
		/// };
		///
		/// let free = RcFree::<IdentityBrand, _>::pure(7).map(|x: i32| x + 1);
		/// assert_eq!(free.evaluate(), 8);
		/// ```
		#[cfg_attr(not(feature = "effects"), allow(dead_code))]
		pub(crate) fn continue_from_erased(
			free: RcFree<F, RcTypeErasedValue>,
			continuations: RcCatList<RcContinuation<F>>,
		) -> Self
		where
			A: Clone,
			Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<F, RcTypeErasedValue>,
			>): Clone, {
			let downcast_continuation = RcContinuation::new(move |value: RcTypeErasedValue| {
				#[expect(clippy::expect_used, reason = "Type maintained by internal invariant")]
				let rc_a: Rc<A> = value.downcast().expect("Type mismatch in RcFree::continue_from_erased");
				let a: A = Rc::try_unwrap(rc_a).unwrap_or_else(|shared| (*shared).clone());
				RcFree::<F, A>::pure(a).cast_phantom()
			});
			let all_continuations = continuations.snoc(downcast_continuation);
			let mut owned = free.into_inner_owned();
			let view = owned.view.take();
			let inner_continuations = std::mem::take(&mut owned.continuations);
			RcFree::from_inner(RcFreeInner {
				view,
				continuations: inner_continuations.append(all_continuations),
				_marker: PhantomData,
			})
		}

		/// Appends pending continuations to a branch whose result was
		/// reboxed as [`RcTypeErasedValue`].
		#[document_signature]
		///
		#[document_parameters(
			"The reboxed type-erased branch selected by the interpreter.",
			"The pending continuation queue to append to that branch."
		)]
		#[document_returns(
			"An `RcFree` value whose selected branch will unbox the erased result and run the pending continuations."
		)]
		#[document_examples(
			skip_call_check,
			reason = "Direct-call validation is skipped because continue_from_reboxed_erased is crate-private raw interpreter plumbing; public map, bind, and evaluate exercise the reboxed continuation invariant."
		)]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::*,
		/// };
		///
		/// let free = RcFree::<IdentityBrand, _>::pure(7).map(|x: i32| x + 1);
		/// assert_eq!(free.evaluate(), 8);
		/// ```
		#[cfg_attr(not(feature = "effects"), allow(dead_code))]
		pub(crate) fn continue_from_reboxed_erased(
			free: RcFree<F, RcTypeErasedValue>,
			continuations: RcCatList<RcContinuation<F>>,
		) -> Self
		where
			A: Clone,
			Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<F, RcTypeErasedValue>,
			>): Clone, {
			let unbox_continuation = RcContinuation::new(move |value: RcTypeErasedValue| {
				#[expect(clippy::expect_used, reason = "Type maintained by internal invariant")]
				let rc_erased: Rc<RcTypeErasedValue> = value
					.downcast()
					.expect("Type mismatch in RcFree::continue_from_reboxed_erased outer downcast");
				let erased: RcTypeErasedValue =
					Rc::try_unwrap(rc_erased).unwrap_or_else(|shared| (*shared).clone());
				RcFree::<F, RcTypeErasedValue>::from_erased_value(erased)
			});
			let all_continuations =
				RcCatList::empty().snoc(unbox_continuation).append(continuations);
			let mut owned = free.into_inner_owned();
			let view = owned.view.take();
			let inner_continuations = std::mem::take(&mut owned.continuations);
			RcFree::from_inner(RcFreeInner {
				view,
				continuations: inner_continuations.append(all_continuations),
				_marker: PhantomData,
			})
		}

		/// Builds a raw erased `RcFree` return from an already-erased
		/// value without adding another erased wrapper layer.
		#[document_signature]
		#[document_parameters("The erased value to store as the direct return payload.")]
		#[document_returns("An `RcFree` computation returning the erased value directly.")]
		#[document_examples(
			skip_call_check,
			reason = "Direct-call validation is skipped because from_erased_value is crate-private raw return construction; public erase_type and evaluate cover the observable erased-result behavior."
		)]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::*,
		/// };
		///
		/// let free = RcFree::<IdentityBrand, _>::pure(42);
		/// assert_eq!(free.evaluate(), 42);
		/// ```
		#[cfg_attr(not(feature = "effects"), allow(dead_code))]
		pub(crate) fn from_erased_value(value: RcTypeErasedValue) -> RcFree<F, RcTypeErasedValue>
		where
			Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<F, RcTypeErasedValue>,
			>): Clone, {
			RcFree::from_inner(RcFreeInner {
				view: Some(RcFreeView::Return(value)),
				continuations: RcCatList::empty(),
				_marker: PhantomData,
			})
		}

		/// Appends a raw erased continuation without downcasting the payload
		/// to this computation's phantom result type.
		#[document_signature]
		#[document_parameters(
			"The raw erased branch selected by the interpreter.",
			"The raw erased continuation to append."
		)]
		#[document_returns("The raw erased branch with the continuation appended.")]
		#[document_examples(
			skip_call_check,
			reason = "Direct-call validation is skipped because append_erased_continuation is crate-private raw continuation plumbing; public bind and evaluation paths exercise appended continuations."
		)]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::*,
		/// };
		///
		/// let free = RcFree::<IdentityBrand, _>::pure(42);
		/// assert_eq!(free.evaluate(), 42);
		/// ```
		#[cfg_attr(not(feature = "effects"), allow(dead_code))]
		pub(crate) fn append_erased_continuation(
			free: RcFree<F, RcTypeErasedValue>,
			continuation: impl Fn(RcTypeErasedValue) -> RcFree<F, RcTypeErasedValue> + 'static,
		) -> RcFree<F, RcTypeErasedValue>
		where
			Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<F, RcTypeErasedValue>,
			>): Clone, {
			let continuation = RcContinuation::new(continuation);
			let mut owned = free.into_inner_owned();
			let view = owned.view.take();
			let continuations = std::mem::take(&mut owned.continuations);
			RcFree::from_inner(RcFreeInner {
				view,
				continuations: continuations.snoc(continuation),
				_marker: PhantomData,
			})
		}

		/// Decomposes this `RcFree` without mapping the pending continuation
		/// queue into a suspended layer.
		///
		/// This is the continuation-aware sibling of
		/// [`to_view`](RcFree::to_view). It is used by internal
		/// interpreters that need to select one branch before reattaching
		/// the shared continuation queue.
		#[document_signature]
		///
		#[document_returns(
			"[`RcFreeRawStep::Done(a)`](RcFreeRawStep::Done) if the computation is complete, or [`RcFreeRawStep::Suspended`](RcFreeRawStep::Suspended) with the suspended layer and pending continuations kept separate."
		)]
		#[document_examples(
			skip_call_check,
			reason = "Direct-call validation is skipped because into_raw_step is crate-private raw decomposition; public to_view, resume, and evaluate exercise the same stepping loop."
		)]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::*,
		/// };
		///
		/// let free = RcFree::<IdentityBrand, _>::pure(42);
		/// assert_eq!(free.evaluate(), 42);
		/// ```
		#[expect(
			clippy::expect_used,
			reason = "RcFree values consumed exactly once per layer-walk step; double consumption indicates a bug"
		)]
		#[cfg_attr(not(feature = "effects"), allow(dead_code))]
		pub(crate) fn into_raw_step(self) -> RcFreeRawStep<F, A>
		where
			A: Clone,
			Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<F, RcTypeErasedValue>,
			>): Clone, {
			let mut owned = self.into_inner_owned();
			let mut current_view = owned.view.take().expect("RcFree value already consumed");
			let mut conts = std::mem::take(&mut owned.continuations);

			loop {
				match current_view {
					RcFreeView::Return(val) => match conts.uncons() {
						Some((continuation, rest)) => {
							let next = continuation.call(val);
							let mut next_owned = next.into_inner_owned();
							current_view = next_owned
								.view
								.take()
								.expect("RcFree value already consumed (continuation)");
							let next_conts = std::mem::take(&mut next_owned.continuations);
							conts = next_conts.append(rest);
						}
						None => {
							let rc_a: Rc<A> = val
								.downcast::<A>()
								.expect("Type mismatch in RcFree::into_raw_step final downcast");
							let a: A =
								Rc::try_unwrap(rc_a).unwrap_or_else(|shared| (*shared).clone());
							return RcFreeRawStep::Done(a);
						}
					},
					RcFreeView::Suspend(layer) => {
						return RcFreeRawStep::Suspended {
							layer,
							continuations: conts,
						};
					}
				}
			}
		}

		/// Transforms the raw suspended layer and continuation queue.
		///
		/// This is the `RcFree` counterpart of
		/// [`Free::transform_raw`](crate::types::Free::transform_raw).
		/// It steps through completed erased returns until either the
		/// computation is complete or a suspended layer is reached. Pure
		/// results are preserved without reboxing, while suspended layers
		/// and pending continuations are rewritten by the supplied
		/// callbacks.
		#[document_signature]
		#[document_type_parameters("The target base functor.")]
		#[document_parameters(
			"The transformation to apply to a suspended source functor layer.",
			"The transformation to apply to the pending continuation queue."
		)]
		#[document_returns(
			"An `RcFree` computation over the target base functor with the same result storage."
		)]
		#[document_examples(
			skip_call_check,
			reason = "transform_raw is crate-private raw continuation plumbing; row embedding and raw RcRun interpreters exercise it without exposing RcFree internals."
		)]
		///
		/// ```
		/// let value = 42;
		/// assert_eq!(value, 42);
		/// ```
		#[expect(
			clippy::expect_used,
			reason = "RcFree values consumed exactly once per raw-transform step"
		)]
		#[cfg_attr(not(feature = "effects"), allow(dead_code))]
		pub(crate) fn transform_raw<G>(
			self,
			transform_layer: impl FnOnce(
				Apply!(
					<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
						'static,
						RcFree<F, RcTypeErasedValue>,
					>
				),
			) -> Apply!(
				<G as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'static,
					RcFree<G, RcTypeErasedValue>,
				>
			),
			transform_continuations: impl FnOnce(
				RcCatList<RcContinuation<F>>,
			) -> RcCatList<RcContinuation<G>>,
		) -> RcFree<G, A>
		where
			G: WrapDrop + 'static,
			Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<F, RcTypeErasedValue>,
			>): Clone, {
			let mut owned = self.into_inner_owned();
			let mut current_view = owned.view.take().expect("RcFree value already consumed");
			let mut conts = std::mem::take(&mut owned.continuations);
			let mut transform_layer = Some(transform_layer);
			let mut transform_continuations = Some(transform_continuations);

			loop {
				match current_view {
					RcFreeView::Return(value) => match conts.uncons() {
						Some((continuation, rest)) => {
							let next = continuation.call(value);
							let mut next_owned = next.into_inner_owned();
							current_view = next_owned
								.view
								.take()
								.expect("RcFree value already consumed (continuation)");
							let next_conts = std::mem::take(&mut next_owned.continuations);
							conts = next_conts.append(rest);
						}
						None => {
							return RcFree::from_inner(RcFreeInner {
								view: Some(RcFreeView::Return(value)),
								continuations: RcCatList::empty(),
								_marker: PhantomData,
							});
						}
					},
					RcFreeView::Suspend(layer) => {
						let transform_layer =
							transform_layer.take().expect("RcFree::transform_raw layer reused");
						let transform_continuations = transform_continuations
							.take()
							.expect("RcFree::transform_raw continuations reused");

						return RcFree::from_inner(RcFreeInner {
							view: Some(RcFreeView::Suspend(transform_layer(layer))),
							continuations: transform_continuations(conts),
							_marker: PhantomData,
						});
					}
				}
			}
		}

		/// Monadic bind with O(1) per-call cost.
		///
		/// Wraps the user closure into an `Rc<dyn Fn>` and snocs onto the
		/// continuation [`RcCatList`](crate::types::RcCatList). Requires
		/// `A: Clone` because each stored continuation may be invoked more
		/// than once and must recover an owned `A` from the type-erased
		/// `Rc<A>` cell on every call.
		#[document_signature]
		///
		#[document_type_parameters("The result type of the new computation.")]
		///
		#[document_parameters("The function to apply to the result of this computation.")]
		///
		#[document_returns("A new `RcFree` computation that chains `f` after this computation.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::*,
		/// };
		///
		/// let free = RcFree::<IdentityBrand, _>::pure(42).bind(|x: i32| RcFree::pure(x + 1));
		/// assert_eq!(free.evaluate(), 43);
		/// ```
		#[expect(clippy::expect_used, reason = "Type maintained by internal invariant")]
		pub fn bind<B: 'static>(
			self,
			f: impl Fn(A) -> RcFree<F, B> + 'static,
		) -> RcFree<F, B>
		where
			A: Clone,
			Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<F, RcTypeErasedValue>,
			>): Clone, {
			let erased_f = RcContinuation::new(move |val: RcTypeErasedValue| {
				let rc_a: Rc<A> = val.downcast::<A>().expect("Type mismatch in RcFree::bind");
				let a: A = Rc::try_unwrap(rc_a).unwrap_or_else(|shared| (*shared).clone());
				f(a).cast_phantom()
			});
			let mut owned = self.into_inner_owned();
			let conts = std::mem::take(&mut owned.continuations);
			RcFree::from_inner(RcFreeInner {
				view: owned.view.take(),
				continuations: conts.snoc(erased_f),
				_marker: PhantomData,
			})
		}

		/// Functor map: transforms the result without changing structure.
		#[document_signature]
		///
		#[document_type_parameters("The result type of the mapping function.")]
		///
		#[document_parameters("The function to apply to the result of this computation.")]
		///
		#[document_returns("A new `RcFree` computation with the transformed result.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::*,
		/// };
		///
		/// let free = RcFree::<IdentityBrand, _>::pure(10).map(|x: i32| x * 2);
		/// assert_eq!(free.evaluate(), 20);
		/// ```
		pub fn map<B: 'static>(
			self,
			f: impl Fn(A) -> B + 'static,
		) -> RcFree<F, B>
		where
			A: Clone,
			Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<F, RcTypeErasedValue>,
			>): Clone, {
			self.bind(move |a| RcFree::pure(f(a)))
		}

		/// Creates a suspended computation from a functor value.
		#[document_signature]
		///
		#[document_parameters("The functor value containing the next step.")]
		///
		#[document_returns("An `RcFree` computation that performs the effect `fa`.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::*,
		/// };
		///
		/// let inner = RcFree::<IdentityBrand, _>::pure(7);
		/// let free: RcFree<IdentityBrand, _> = RcFree::wrap(Identity(inner));
		/// assert_eq!(free.evaluate(), 7);
		/// ```
		pub fn wrap(
			fa: Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, RcFree<F, A>>)
		) -> Self
		where
			F: Functor,
			Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<F, RcTypeErasedValue>,
			>): Clone, {
			let erased_fa = F::map(
				|inner: RcFree<F, A>| -> RcFree<F, RcTypeErasedValue> { inner.cast_phantom() },
				fa,
			);
			RcFree::from_inner(RcFreeInner {
				view: Some(RcFreeView::Suspend(erased_fa)),
				continuations: RcCatList::empty(),
				_marker: PhantomData,
			})
		}

		/// Lifts a functor value into the [`RcFree`] monad.
		#[document_signature]
		///
		#[document_parameters("The functor value to lift.")]
		///
		#[document_returns(
			"An `RcFree` computation that performs the effect and returns the result."
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
		/// let id = Identity(42);
		/// let free: RcFree<IdentityBrand, _> = RcFree::lift_f(id);
		/// assert_eq!(free.evaluate(), 42);
		/// ```
		pub fn lift_f(fa: Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, A>)) -> Self
		where
			F: Functor,
			Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<F, RcTypeErasedValue>,
			>): Clone, {
			RcFree::wrap(F::map(RcFree::pure, fa))
		}

		/// Decomposes this `RcFree` into a single [`RcFreeStep`].
		///
		/// Iteratively applies pending continuations until a final value or
		/// a suspended functor layer is reached. In the `Suspended` case the
		/// remaining continuations are reattached to the inner `RcFree`
		/// values via [`Functor::map`].
		#[document_signature]
		///
		#[document_returns(
			"[`RcFreeStep::Done`] if complete, or [`RcFreeStep::Suspended`] if suspended in the functor `F`."
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
		/// let free = RcFree::<IdentityBrand, _>::pure(42);
		/// match free.to_view() {
		/// 	RcFreeStep::Done(a) => assert_eq!(a, 42),
		/// 	RcFreeStep::Suspended(_) => panic!("expected Done"),
		/// }
		/// ```
		#[expect(
			clippy::expect_used,
			reason = "RcFree views consumed exactly once per layer-walk step; double consumption indicates a bug"
		)]
		pub fn to_view(self) -> RcFreeStep<F, A>
		where
			F: Functor,
			A: Clone,
			Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<F, RcTypeErasedValue>,
			>): Clone, {
			let mut owned = self.into_inner_owned();
			let mut current_view = owned.view.take().expect("RcFree value already consumed");
			let mut conts = std::mem::take(&mut owned.continuations);

			loop {
				match current_view {
					RcFreeView::Return(val) => match conts.uncons() {
						Some((continuation, rest)) => {
							let next = continuation.call(val);
							let mut next_owned = next.into_inner_owned();
							current_view = next_owned
								.view
								.take()
								.expect("RcFree value already consumed (continuation)");
							let next_conts = std::mem::take(&mut next_owned.continuations);
							conts = next_conts.append(rest);
						}
						None => {
							let rc_a: Rc<A> = val
								.downcast::<A>()
								.expect("Type mismatch in RcFree::to_view final downcast");
							let a: A =
								Rc::try_unwrap(rc_a).unwrap_or_else(|shared| (*shared).clone());
							return RcFreeStep::Done(a);
						}
					},
					RcFreeView::Suspend(fa) => {
						let downcast_cont = RcContinuation::new(move |val: RcTypeErasedValue| {
							let rc_a: Rc<A> = val
								.downcast::<A>()
								.expect("Type mismatch in RcFree::to_view downcast");
							let a: A =
								Rc::try_unwrap(rc_a).unwrap_or_else(|shared| (*shared).clone());
							RcFree::<F, A>::pure(a).cast_phantom()
						});
						let all_conts = conts.snoc(downcast_cont);
						let typed_fa = F::map(
							move |inner_free: RcFree<F, RcTypeErasedValue>| {
								// `RcCatList::clone` is O(1) (refcount bump),
								// so this closure is safely callable multiple
								// times by handlers that re-enter the
								// continuation (e.g. `Choose`).
								let conts_for_inner = all_conts.clone();
								let mut owned_inner = inner_free.into_inner_owned();
								let v = owned_inner.view.take();
								let c = std::mem::take(&mut owned_inner.continuations);
								RcFree::from_inner(RcFreeInner {
									view: v,
									continuations: c.append(conts_for_inner),
									_marker: PhantomData,
								})
							},
							fa,
						);
						return RcFreeStep::Suspended(typed_fa);
					}
				}
			}
		}

		/// Decomposes this `RcFree` into one step.
		#[document_signature]
		///
		#[document_returns("`Ok(a)` if pure, `Err(fa)` if suspended.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::*,
		/// };
		///
		/// let free = RcFree::<IdentityBrand, _>::pure(42);
		/// assert!(matches!(free.resume(), Ok(42)));
		/// ```
		pub fn resume(
			self
		) -> Result<A, Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, RcFree<F, A>>)>
		where
			F: Functor,
			A: Clone,
			Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<F, RcTypeErasedValue>,
			>): Clone, {
			match self.to_view() {
				RcFreeStep::Done(a) => Ok(a),
				RcFreeStep::Suspended(fa) => Err(fa),
			}
		}

		/// Executes the `RcFree` computation, returning the final result.
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
		/// let free = RcFree::<IdentityBrand, _>::pure(42);
		/// assert_eq!(free.evaluate(), 42);
		/// ```
		pub fn evaluate(self) -> A
		where
			F: Extract + Functor,
			A: Clone,
			Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<F, RcTypeErasedValue>,
			>): Clone, {
			let mut current = self;
			loop {
				match current.to_view() {
					RcFreeStep::Done(a) => return a,
					RcFreeStep::Suspended(fa) => {
						current = <F as Extract>::extract(fa);
					}
				}
			}
		}

		/// Non-consuming counterpart to [`evaluate`](RcFree::evaluate):
		/// clones the structure (O(1) refcount bump on the outer `Rc`) and
		/// runs the consuming version on the clone.
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
		/// let free = RcFree::<IdentityBrand, _>::pure(42);
		/// assert_eq!(free.lower_ref(), 42);
		/// // Original still usable.
		/// assert_eq!(free.evaluate(), 42);
		/// ```
		pub fn lower_ref(&self) -> A
		where
			F: Extract + Functor,
			A: Clone,
			Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<F, RcTypeErasedValue>,
			>): Clone, {
			self.clone().evaluate()
		}

		/// Non-consuming counterpart to [`to_view`](RcFree::to_view).
		#[document_signature]
		///
		#[document_returns("The current step of the computation.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::*,
		/// };
		///
		/// let free = RcFree::<IdentityBrand, _>::pure(42);
		/// match free.peel_ref() {
		/// 	RcFreeStep::Done(a) => assert_eq!(a, 42),
		/// 	RcFreeStep::Suspended(_) => panic!("expected Done"),
		/// }
		/// // Original still usable.
		/// assert_eq!(free.evaluate(), 42);
		/// ```
		pub fn peel_ref(&self) -> RcFreeStep<F, A>
		where
			F: Functor,
			A: Clone,
			Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<F, RcTypeErasedValue>,
			>): Clone, {
			self.clone().to_view()
		}

		/// Transforms the functor layer of this `RcFree` via a natural
		/// transformation, mirroring [`Free::hoist_free`](crate::types::Free::hoist_free).
		#[document_signature]
		///
		#[document_type_parameters("The target functor brand.")]
		///
		#[document_parameters("The natural transformation from `F` to `G`.")]
		///
		#[document_returns("An `RcFree` computation over functor `G` with the same result.")]
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
		/// #[derive(Clone)]
		/// struct IdToId;
		/// impl NaturalTransformation<IdentityBrand, IdentityBrand> for IdToId {
		/// 	fn transform<'a, A: 'a>(
		/// 		&self,
		/// 		fa: Identity<A>,
		/// 	) -> Identity<A> {
		/// 		fa
		/// 	}
		/// }
		///
		/// let free: RcFree<IdentityBrand, i32> = RcFree::lift_f(Identity(42));
		/// let hoisted: RcFree<IdentityBrand, i32> = free.hoist_free(IdToId);
		/// assert_eq!(hoisted.evaluate(), 42);
		/// ```
		pub fn hoist_free<G: WrapDrop + Functor + 'static>(
			self,
			nt: impl NaturalTransformation<F, G> + Clone + 'static,
		) -> RcFree<G, A>
		where
			F: Functor,
			A: Clone,
			Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<F, RcTypeErasedValue>,
			>): Clone,
			Apply!(<G as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<G, RcTypeErasedValue>,
			>): Clone, {
			match self.resume() {
				Ok(a) => RcFree::pure(a),
				Err(fa) => {
					let nt_clone = nt.clone();
					let ga = nt.transform(fa);
					RcFree::<G, RcFree<F, A>>::lift_f(ga)
						.bind(move |inner: RcFree<F, A>| inner.hoist_free(nt_clone.clone()))
				}
			}
		}
	}
}

pub use inner::*;

#[cfg(test)]
#[expect(clippy::panic, reason = "Tests use panicking operations for brevity and clarity")]
mod tests {
	use {
		super::*,
		crate::{
			brands::IdentityBrand,
			types::{
				Identity,
				RcCatList,
			},
		},
	};

	fn transform_identity_raw<A: 'static>(
		free: RcFree<IdentityBrand, A>
	) -> RcFree<IdentityBrand, A> {
		free.transform_raw(
			|Identity(inner)| Identity(transform_identity_raw(inner)),
			transform_identity_continuations,
		)
	}

	fn transform_identity_continuations(
		mut continuations: RcCatList<RcContinuation<IdentityBrand>>
	) -> RcCatList<RcContinuation<IdentityBrand>> {
		let mut transformed = RcCatList::empty();
		while let Some((continuation, rest)) = continuations.uncons() {
			transformed = transformed.snoc(RcContinuation::new(move |value| {
				transform_identity_raw(continuation.call(value))
			}));
			continuations = rest;
		}
		transformed
	}

	#[test]
	fn pure_evaluate() {
		let free = RcFree::<IdentityBrand, _>::pure(42);
		assert_eq!(free.evaluate(), 42);
	}

	#[test]
	fn wrap_evaluate() {
		let inner = RcFree::<IdentityBrand, _>::pure(7);
		let free: RcFree<IdentityBrand, _> = RcFree::wrap(Identity(inner));
		assert_eq!(free.evaluate(), 7);
	}

	#[test]
	fn lift_f_evaluate() {
		let free: RcFree<IdentityBrand, _> = RcFree::lift_f(Identity(99));
		assert_eq!(free.evaluate(), 99);
	}

	#[test]
	fn bind_chains() {
		let free = RcFree::<IdentityBrand, _>::pure(1)
			.bind(|x: i32| RcFree::pure(x + 1))
			.bind(|x: i32| RcFree::pure(x * 10));
		assert_eq!(free.evaluate(), 20);
	}

	#[test]
	fn map_transforms() {
		let free = RcFree::<IdentityBrand, _>::pure(10).map(|x: i32| x * 2);
		assert_eq!(free.evaluate(), 20);
	}

	#[test]
	fn clone_branches_independent() {
		let free = RcFree::<IdentityBrand, _>::pure(42).bind(|x: i32| RcFree::pure(x + 1));
		let branch = free.clone();
		assert_eq!(free.evaluate(), 43);
		assert_eq!(branch.evaluate(), 43);
	}

	#[test]
	fn lower_ref_does_not_consume() {
		let free = RcFree::<IdentityBrand, _>::pure(7).bind(|x: i32| RcFree::pure(x * 6));
		assert_eq!(free.lower_ref(), 42);
		assert_eq!(free.lower_ref(), 42);
		assert_eq!(free.evaluate(), 42);
	}

	#[test]
	fn peel_ref_does_not_consume() {
		let free = RcFree::<IdentityBrand, _>::pure(123);
		match free.peel_ref() {
			RcFreeStep::Done(a) => assert_eq!(a, 123),
			RcFreeStep::Suspended(_) => panic!("expected Done"),
		}
		assert_eq!(free.evaluate(), 123);
	}

	#[test]
	fn raw_step_keeps_continuations_outside_suspended_layer() {
		let free = RcFree::<IdentityBrand, _>::wrap(Identity(RcFree::pure(1)))
			.bind(|x: i32| RcFree::pure(x + 41));

		match free.into_raw_step() {
			RcFreeRawStep::Done(_) => panic!("expected suspended raw step"),
			RcFreeRawStep::Suspended {
				layer: Identity(action),
				continuations,
			} => {
				assert_eq!(continuations.len(), 1);
				let resumed: RcFree<IdentityBrand, i32> =
					RcFree::continue_from_erased(action, continuations);
				assert_eq!(resumed.evaluate(), 42);
			}
		}
	}

	#[test]
	fn transform_raw_maps_suspended_layer_and_continuations() {
		let free: RcFree<IdentityBrand, i32> =
			RcFree::lift_f(Identity(40)).map(|value: i32| value + 2);

		let transformed: RcFree<IdentityBrand, i32> = transform_identity_raw(free);

		match transformed.resume() {
			Err(Identity(next)) => assert!(matches!(next.resume(), Ok(42))),
			Ok(_) => panic!("expected transformed suspension"),
		}
	}

	#[test]
	fn multi_shot_continuation_via_clone() {
		// Multi-shot handler emulation: clone the program, evaluate each
		// branch, sum. The user closure inside `bind` is `Fn`, so the same
		// stored continuation runs once per branch.
		let program = RcFree::<IdentityBrand, _>::pure(10).bind(|x: i32| RcFree::pure(x + 1));
		let total = program.clone().evaluate() + program.evaluate();
		assert_eq!(total, 22);
	}

	#[test]
	fn deep_evaluate_does_not_overflow() {
		const DEPTH: usize = 100_000;
		let mut free: RcFree<IdentityBrand, i32> = RcFree::pure(0);
		for _ in 0 .. DEPTH {
			free = RcFree::wrap(Identity(free));
		}
		assert_eq!(free.evaluate(), 0);
	}

	#[test]
	fn deep_drop_does_not_overflow() {
		const DEPTH: usize = 100_000;
		let mut free: RcFree<IdentityBrand, i32> = RcFree::pure(0);
		for _ in 0 .. DEPTH {
			free = RcFree::wrap(Identity(free));
		}
		drop(free);
	}

	#[test]
	fn stack_safe_left_associated_bind() {
		fn count_down(n: i32) -> RcFree<IdentityBrand, i32> {
			if n == 0 { RcFree::pure(0) } else { RcFree::pure(n).bind(|n| count_down(n - 1)) }
		}
		assert_eq!(count_down(10_000).evaluate(), 0);
	}
}
