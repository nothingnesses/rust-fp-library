//! Stack-safe Free monad with `Rc`-shared continuations supporting multi-shot effects.
//!
//! [`RcFree`] mirrors [`Free`](crate::types::Free)'s "Reflection without Remorse"
//! structure (a [`CatList`](crate::types::CatList) of pending continuations
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
//!   [`CatList`](crate::types::CatList), so the per-bind cost is one `Rc`
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
//! [`Extract::extract`](crate::classes::Extract::extract), mirroring
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
			},
			kinds::*,
			types::CatList,
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

	/// Type-erased continuation stored in the [`CatList`](crate::types::CatList)
	/// queue, equivalent to
	/// [`<RcFnBrand as CloneFn>::Of<'static, RcTypeErasedValue, RcFree<F, RcTypeErasedValue>>`](crate::brands::FnBrand).
	pub struct RcContinuation<F>(Rc<dyn Fn(RcTypeErasedValue) -> RcFree<F, RcTypeErasedValue>>)
	where
		F: Extract + Functor + 'static;

	#[document_type_parameters("The base functor.")]
	#[document_parameters("The continuation to clone.")]
	impl<F> Clone for RcContinuation<F>
	where
		F: Extract + Functor + 'static,
	{
		/// Clones the continuation by bumping the refcount on its `Rc`.
		#[document_signature]
		///
		#[document_returns("A clone of the continuation.")]
		#[document_examples]
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

	/// The internal view of an [`RcFree`] computation.
	///
	/// Mirrors [`FreeView`](crate::types::free::FreeView): either a pure
	/// value or a single suspended functor layer holding the next step.
	#[document_type_parameters("The base functor (must implement [`Extract`] and [`Functor`]).")]
	pub enum RcFreeView<F>
	where
		F: Extract + Functor + 'static, {
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

	#[document_type_parameters("The base functor (must implement [`Extract`] and [`Functor`]).")]
	#[document_parameters("The view to clone.")]
	impl<F> Clone for RcFreeView<F>
	where
		F: Extract + Functor + 'static,
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
		F: Extract + Functor + 'static,
		A: 'static, {
		/// The computation completed with a final value.
		Done(A),
		/// The computation is suspended in the functor `F`. The inner
		/// `RcFree` values have all pending continuations reattached.
		Suspended(Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, RcFree<F, A>>)),
	}

	/// Inner state of an [`RcFree`]: view plus pending continuations.
	struct RcFreeInner<F, A>
	where
		F: Extract + Functor + 'static,
		A: 'static, {
		view: Option<RcFreeView<F>>,
		continuations: CatList<RcContinuation<F>>,
		_marker: PhantomData<A>,
	}

	#[document_type_parameters("The base functor.", "The result type.")]
	#[document_parameters("The inner state to clone.")]
	impl<F, A> Clone for RcFreeInner<F, A>
	where
		F: Extract + Functor + 'static,
		A: 'static,
		Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'static,
			RcFree<F, RcTypeErasedValue>,
		>): Clone,
	{
		/// Clones the inner state: the view via [`RcFreeView`]'s `Clone`,
		/// the continuation queue via [`CatList`](crate::types::CatList)'s
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
		F: Extract + Functor + 'static,
		A: 'static,
	{
		/// Iteratively dismantles deep `Suspend` chains via
		/// [`Extract::extract`], mirroring [`Free::drop`](crate::types::Free).
		#[document_signature]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::*,
		/// };
		/// {
		/// 	let _free = RcFree::<IdentityBrand, _>::pure(42);
		/// } // drop called here
		/// assert!(true);
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
						let extracted: RcFree<F, RcTypeErasedValue> = <F as Extract>::extract(fa);
						// If we hold the last reference, peel its view and
						// continue the worklist; otherwise leave the other
						// holders to dismantle when they release.
						if let Ok(mut owned) = Rc::try_unwrap(extracted.inner) {
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
		"The base functor (must implement [`Extract`] and [`Functor`]).",
		"The result type."
	)]
	pub struct RcFree<F, A>
	where
		F: Extract + Functor + 'static,
		A: 'static, {
		inner: Rc<RcFreeInner<F, A>>,
	}

	#[document_type_parameters("The base functor.", "The result type.")]
	#[document_parameters("The `RcFree` instance to clone.")]
	impl<F, A> Clone for RcFree<F, A>
	where
		F: Extract + Functor + 'static,
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
		F: Extract + Functor + 'static,
		A: 'static,
	{
		/// Constructs an `RcFree` from owned inner state.
		#[document_signature]
		///
		#[document_parameters("The inner state to wrap.")]
		///
		#[document_returns("A new `RcFree` wrapping the inner state in an `Rc`.")]
		///
		#[document_examples]
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
		#[document_examples]
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
				continuations: CatList::empty(),
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
		#[document_examples]
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

		/// Monadic bind with O(1) per-call cost.
		///
		/// Wraps the user closure into an `Rc<dyn Fn>` and snocs onto the
		/// continuation [`CatList`](crate::types::CatList). Requires
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
			let erased_f =
				RcContinuation(<RcFnBrand as LiftFn>::new(move |val: RcTypeErasedValue| {
					let rc_a: Rc<A> = val.downcast::<A>().expect("Type mismatch in RcFree::bind");
					let a: A = Rc::try_unwrap(rc_a).unwrap_or_else(|shared| (*shared).clone());
					f(a).cast_phantom()
				}));
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
				continuations: CatList::empty(),
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
			reason = "RcFree values consumed exactly once per branch; double consumption indicates a bug"
		)]
		pub fn to_view(self) -> RcFreeStep<F, A>
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
							let next = (continuation.0)(val);
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
						let downcast_cont = RcContinuation(<RcFnBrand as LiftFn>::new(
							move |val: RcTypeErasedValue| {
								let rc_a: Rc<A> = val
									.downcast::<A>()
									.expect("Type mismatch in RcFree::to_view downcast");
								let a: A =
									Rc::try_unwrap(rc_a).unwrap_or_else(|shared| (*shared).clone());
								RcFree::<F, A>::pure(a).cast_phantom()
							},
						));
						let all_conts = conts.snoc(downcast_cont);
						let remaining = std::cell::Cell::new(Some(all_conts));
						let typed_fa = F::map(
							move |inner_free: RcFree<F, RcTypeErasedValue>| {
								let conts_for_inner = remaining
									.take()
									.expect("RcFree::to_view map called more than once");
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
		pub fn hoist_free<G: Extract + Functor + 'static>(
			self,
			nt: impl NaturalTransformation<F, G> + Clone + 'static,
		) -> RcFree<G, A>
		where
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
			types::Identity,
		},
	};

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
