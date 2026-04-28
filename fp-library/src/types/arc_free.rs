//! Thread-safe Free monad with `Arc`-shared continuations.
//!
//! [`ArcFree`] is the [`Send`] + [`Sync`] sibling of
//! [`RcFree`](crate::types::RcFree): the same "Reflection without Remorse"
//! structure (a [`CatList`](crate::types::CatList) of pending continuations
//! sitting beside a single type-erased view), but with [`Arc<dyn Fn + Send + Sync>`](std::sync::Arc)
//! continuations (matching what
//! [`FnBrand<ArcBrand>`](crate::brands::FnBrand) resolves to via
//! [`SendCloneFn`](crate::classes::SendCloneFn)) instead of `Rc<dyn Fn>`.
//! Programs cross thread boundaries.
//!
//! The whole substrate lives behind an outer [`Arc`](std::sync::Arc) so
//! cloning a program is O(1) (atomic refcount bump). Operations that
//! extend the structure (`bind`, `map`, `lift_f`, ...) consume `self` and
//! either move out of the `Arc` (when uniquely owned) or clone the inner
//! state (when shared from a prior `clone()` or send across threads).
//!
//! ## Trade-offs vs `RcFree`
//!
//! - **Thread-safety:** `ArcFree` is `Send + Sync`. `RcFree` is not.
//! - **Overhead:** Atomic reference counting is slightly more expensive
//!   than non-atomic. Use `RcFree` when single-threaded.
//! - **Closure bounds:** continuations are `Send + Sync`, so the closures
//!   passed to `bind` / `map` / `wrap` must be `Send + Sync`.
//! - **`A: Send + Sync + Clone` on downcast:** the type-erased value cell
//!   is `Arc<dyn Any + Send + Sync>`, so the downcast requires
//!   `A: Send + Sync` (in addition to `A: Clone` from the shared-cell
//!   recovery fallback).
//!
//! ## Drop behavior
//!
//! When the last `Arc` reference releases, the inner data's [`Drop`] runs
//! and iteratively dismantles a deep `Wrap` chain via
//! [`WrapDrop::drop`](crate::classes::WrapDrop::drop), mirroring
//! [`RcFree`](crate::types::RcFree) and [`Free`](crate::types::Free).

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			Apply,
			brands::ArcFnBrand,
			classes::{
				Extract,
				NaturalTransformation,
				SendFunctor,
				SendLiftFn,
				WrapDrop,
			},
			kinds::*,
			types::CatList,
		},
		fp_macros::*,
		std::{
			any::Any,
			marker::PhantomData,
			sync::Arc,
		},
	};

	/// Type-erased value carrying its concrete type at runtime via [`Any`],
	/// constrained to be `Send + Sync` so the wrapping `Arc` is also
	/// `Send + Sync`.
	pub type ArcTypeErasedValue = Arc<dyn Any + Send + Sync>;

	/// Type-erased continuation stored in the [`CatList`](crate::types::CatList)
	/// queue, equivalent to
	/// [`<ArcFnBrand as SendCloneFn>::Of<'static, ArcTypeErasedValue, ArcFree<F, ArcTypeErasedValue>>`](crate::brands::FnBrand).
	pub struct ArcContinuation<F>(
		Arc<dyn Fn(ArcTypeErasedValue) -> ArcFree<F, ArcTypeErasedValue> + Send + Sync>,
	)
	where
		F: WrapDrop
			+ Kind_cdc7cd43dac7585f<Of<'static, ArcFree<F, ArcTypeErasedValue>>: Send + Sync>
			+ 'static;

	#[document_type_parameters("The base functor.")]
	#[document_parameters("The continuation to clone.")]
	impl<F> Clone for ArcContinuation<F>
	where
		F: WrapDrop
			+ Kind_cdc7cd43dac7585f<Of<'static, ArcFree<F, ArcTypeErasedValue>>: Send + Sync>
			+ 'static,
	{
		/// Clones the continuation by atomic refcount bump.
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
		/// // `ArcContinuation` is internal; `bind` is the public API that constructs it.
		/// let free = ArcFree::<IdentityBrand, _>::pure(1).bind(|x: i32| ArcFree::pure(x + 1));
		/// assert_eq!(free.evaluate(), 2);
		/// ```
		fn clone(&self) -> Self {
			ArcContinuation(Arc::clone(&self.0))
		}
	}

	/// The internal view of an [`ArcFree`] computation.
	///
	/// Either a pure value or a single suspended functor layer holding the
	/// next step. Mirrors [`RcFreeView`](crate::types::RcFreeView) with
	/// `Arc + Send + Sync` substitutions.
	#[document_type_parameters("The base functor (must implement [`WrapDrop`]).")]
	pub enum ArcFreeView<F>
	where
		F: WrapDrop
			+ Kind_cdc7cd43dac7585f<Of<'static, ArcFree<F, ArcTypeErasedValue>>: Send + Sync>
			+ 'static, {
		/// A pure value (type-erased, `Send + Sync`).
		Return(ArcTypeErasedValue),
		/// A suspended functor layer holding the next step.
		Suspend(
			Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<F, ArcTypeErasedValue>,
			>),
		),
	}

	#[document_type_parameters("The base functor.")]
	#[document_parameters("The view to clone.")]
	impl<F> Clone for ArcFreeView<F>
	where
		F: WrapDrop
			+ Kind_cdc7cd43dac7585f<Of<'static, ArcFree<F, ArcTypeErasedValue>>: Send + Sync>
			+ 'static,
		Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'static,
			ArcFree<F, ArcTypeErasedValue>,
		>): Clone,
	{
		/// Clones the view, sharing the type-erased value via `Arc::clone`
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
		/// let free = ArcFree::<IdentityBrand, _>::pure(42);
		/// let cloned = free.clone();
		/// assert_eq!(cloned.evaluate(), 42);
		/// ```
		fn clone(&self) -> Self {
			match self {
				ArcFreeView::Return(val) => ArcFreeView::Return(Arc::clone(val)),
				ArcFreeView::Suspend(fa) => ArcFreeView::Suspend(fa.clone()),
			}
		}
	}

	/// The result of stepping through an [`ArcFree`] computation.
	///
	/// Mirror of [`RcFreeStep`](crate::types::RcFreeStep) for the
	/// thread-safe substrate.
	#[document_type_parameters("The base functor.", "The result type.")]
	pub enum ArcFreeStep<F, A>
	where
		F: WrapDrop
			+ Kind_cdc7cd43dac7585f<Of<'static, ArcFree<F, ArcTypeErasedValue>>: Send + Sync>
			+ 'static,
		A: 'static, {
		/// The computation completed with a final value.
		Done(A),
		/// The computation is suspended in the functor `F`.
		Suspended(Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, ArcFree<F, A>>)),
	}

	/// Inner state of an [`ArcFree`]: view plus pending continuations.
	///
	/// The struct-level associated-type bound
	/// `Kind_cdc7cd43dac7585f<Of<'static, ArcFree<F, ArcTypeErasedValue>>: Send + Sync>`
	/// is what lets the compiler auto-derive `Send + Sync` on
	/// `ArcFreeInner` and (by extension) `ArcFree` for concrete `F` like
	/// [`IdentityBrand`](crate::brands::IdentityBrand): without it the
	/// `F::Of<...>` field is opaque and the auto-trait derivation fails.
	struct ArcFreeInner<F, A>
	where
		F: WrapDrop
			+ Kind_cdc7cd43dac7585f<Of<'static, ArcFree<F, ArcTypeErasedValue>>: Send + Sync>
			+ 'static,
		A: 'static, {
		view: Option<ArcFreeView<F>>,
		continuations: CatList<ArcContinuation<F>>,
		_marker: PhantomData<A>,
	}

	#[document_type_parameters("The base functor.", "The result type.")]
	#[document_parameters("The inner state to clone.")]
	impl<F, A> Clone for ArcFreeInner<F, A>
	where
		F: WrapDrop
			+ Kind_cdc7cd43dac7585f<Of<'static, ArcFree<F, ArcTypeErasedValue>>: Send + Sync>
			+ 'static,
		A: 'static,
		Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'static,
			ArcFree<F, ArcTypeErasedValue>,
		>): Clone,
	{
		/// Clones the inner state: view via [`ArcFreeView`]'s `Clone`, the
		/// continuation queue via [`CatList`](crate::types::CatList)'s
		/// `Clone` (each `Arc<dyn Fn + Send + Sync>` cell becomes an
		/// atomic refcount bump).
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
		/// // `ArcFreeInner` is internal; `ArcFree::clone` exposes the same effect.
		/// let free = ArcFree::<IdentityBrand, _>::pure(42);
		/// let cloned = free.clone();
		/// assert_eq!(cloned.evaluate(), 42);
		/// ```
		fn clone(&self) -> Self {
			ArcFreeInner {
				view: self.view.clone(),
				continuations: self.continuations.clone(),
				_marker: PhantomData,
			}
		}
	}

	#[document_type_parameters("The base functor.", "The result type.")]
	#[document_parameters("The inner state being dropped.")]
	impl<F, A> Drop for ArcFreeInner<F, A>
	where
		F: WrapDrop
			+ Kind_cdc7cd43dac7585f<Of<'static, ArcFree<F, ArcTypeErasedValue>>: Send + Sync>
			+ 'static,
		A: 'static,
	{
		/// Iteratively dismantles deep `Suspend` chains via
		/// [`WrapDrop::drop`](crate::classes::WrapDrop::drop), mirroring
		/// [`RcFree`](crate::types::RcFree)'s `Drop` strategy.
		#[document_signature]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::*,
		/// };
		/// {
		/// 	let _free = ArcFree::<IdentityBrand, _>::pure(42);
		/// } // drop called here
		/// assert!(true);
		/// ```
		fn drop(&mut self) {
			let mut worklist: Vec<ArcFreeView<F>> = Vec::new();

			if let Some(view) = self.view.take() {
				worklist.push(view);
			}

			let mut top_conts = std::mem::take(&mut self.continuations);
			while let Some((_continuation, rest)) = top_conts.uncons() {
				top_conts = rest;
			}

			while let Some(view) = worklist.pop() {
				match view {
					ArcFreeView::Return(_) => {
						// Trivially dropped, no nested `ArcFree` values.
					}
					ArcFreeView::Suspend(fa) => {
						// `Some(extracted)` means F materially holds an inner
						// `ArcFree`; if we hold the last reference, peel and
						// continue the worklist iteratively. `None` lets the
						// layer drop in place (sound for brands that do not
						// materially store the inner `ArcFree`).
						if let Some(extracted) =
							<F as WrapDrop>::drop::<ArcFree<F, ArcTypeErasedValue>>(fa)
							&& let Ok(mut owned) = Arc::try_unwrap(extracted.inner)
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

	/// Thread-safe Free monad with `Arc`-shared continuations.
	///
	/// Same internal shape as [`RcFree`](crate::types::RcFree) with `Arc`
	/// substituted for `Rc` and `Send + Sync` bounds added on closure
	/// storage. The whole program is `Clone`, `Send`, and `Sync` whenever
	/// the underlying functor's `Of<'static, ArcFree<...>>` is.
	#[document_type_parameters(
		"The base functor (must implement [`WrapDrop`]; methods that walk the spine additionally require [`Functor`], and `evaluate` / `lower_ref` additionally require [`Extract`]).",
		"The result type."
	)]
	pub struct ArcFree<F, A>
	where
		F: WrapDrop
			+ Kind_cdc7cd43dac7585f<Of<'static, ArcFree<F, ArcTypeErasedValue>>: Send + Sync>
			+ 'static,
		A: 'static, {
		inner: Arc<ArcFreeInner<F, A>>,
	}

	#[document_type_parameters("The base functor.", "The result type.")]
	#[document_parameters("The `ArcFree` instance to clone.")]
	impl<F, A> Clone for ArcFree<F, A>
	where
		F: WrapDrop
			+ Kind_cdc7cd43dac7585f<Of<'static, ArcFree<F, ArcTypeErasedValue>>: Send + Sync>
			+ 'static,
		A: 'static,
	{
		/// Clones the `ArcFree` by atomic refcount bump on the outer `Arc`.
		/// O(1).
		#[document_signature]
		///
		#[document_returns("A new `ArcFree` representing an independent branch.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::*,
		/// };
		///
		/// let free = ArcFree::<IdentityBrand, _>::pure(42);
		/// let branch = free.clone();
		/// assert_eq!(free.evaluate(), 42);
		/// assert_eq!(branch.evaluate(), 42);
		/// ```
		fn clone(&self) -> Self {
			ArcFree {
				inner: Arc::clone(&self.inner),
			}
		}
	}

	#[document_type_parameters("The base functor.", "The result type.")]
	#[document_parameters("The `ArcFree` instance.")]
	impl<F, A> ArcFree<F, A>
	where
		F: WrapDrop
			+ Kind_cdc7cd43dac7585f<Of<'static, ArcFree<F, ArcTypeErasedValue>>: Send + Sync>
			+ 'static,
		A: 'static,
	{
		/// Constructs an `ArcFree` from owned inner state.
		#[document_signature]
		///
		#[document_parameters("The inner state to wrap.")]
		///
		#[document_returns("A new `ArcFree` wrapping the inner state in an `Arc`.")]
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
		/// let free = ArcFree::<IdentityBrand, _>::pure(42);
		/// assert_eq!(free.evaluate(), 42);
		/// ```
		fn from_inner(inner: ArcFreeInner<F, A>) -> Self {
			ArcFree {
				inner: Arc::new(inner),
			}
		}

		/// Acquires owned access to the inner state, cloning the shared
		/// state when the outer `Arc` is not unique.
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
		/// let free = ArcFree::<IdentityBrand, _>::pure(1).bind(|x: i32| ArcFree::pure(x + 1));
		/// assert_eq!(free.evaluate(), 2);
		/// ```
		fn into_inner_owned(self) -> ArcFreeInner<F, A>
		where
			Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<F, ArcTypeErasedValue>,
			>): Clone, {
			Arc::try_unwrap(self.inner).unwrap_or_else(|shared| (*shared).clone())
		}

		/// Creates a pure `ArcFree` value.
		#[document_signature]
		///
		#[document_parameters("The value to wrap.")]
		///
		#[document_returns("An `ArcFree` computation that produces `a`.")]
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
		/// let free = ArcFree::<IdentityBrand, _>::pure(42);
		/// assert_eq!(free.evaluate(), 42);
		/// ```
		pub fn pure(a: A) -> Self
		where
			A: Send + Sync, {
			ArcFree::from_inner(ArcFreeInner {
				view: Some(ArcFreeView::Return(Arc::new(a) as ArcTypeErasedValue)),
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
		#[document_returns("The same `ArcFree` with a different phantom type parameter.")]
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
		/// let free = ArcFree::<IdentityBrand, _>::pure(2).bind(|x: i32| ArcFree::pure(x + 1));
		/// assert_eq!(free.evaluate(), 3);
		/// ```
		fn cast_phantom<B: 'static>(self) -> ArcFree<F, B>
		where
			Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<F, ArcTypeErasedValue>,
			>): Clone, {
			let mut owned = self.into_inner_owned();
			let view = owned.view.take();
			let continuations = std::mem::take(&mut owned.continuations);
			ArcFree::from_inner(ArcFreeInner {
				view,
				continuations,
				_marker: PhantomData,
			})
		}

		/// Monadic bind with O(1) per-call cost.
		///
		/// Wraps the user closure into an `Arc<dyn Fn + Send + Sync>` (via
		/// [`<ArcFnBrand as SendLiftFn>::new`](crate::classes::SendLiftFn))
		/// and snocs onto the [`CatList`](crate::types::CatList) queue.
		/// Requires `A: Clone + Send + Sync` because the continuation
		/// recovers an owned `A` from a shared `Arc<dyn Any + Send + Sync>`
		/// cell on each call, with `Clone` as the fallback when the cell is
		/// shared between branches.
		#[document_signature]
		///
		#[document_type_parameters("The result type of the new computation.")]
		///
		#[document_parameters("The function to apply to the result of this computation.")]
		///
		#[document_returns("A new `ArcFree` computation that chains `f` after this computation.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::*,
		/// };
		///
		/// let free = ArcFree::<IdentityBrand, _>::pure(42).bind(|x: i32| ArcFree::pure(x + 1));
		/// assert_eq!(free.evaluate(), 43);
		/// ```
		#[expect(clippy::expect_used, reason = "Type maintained by internal invariant")]
		pub fn bind<B: 'static + Send + Sync>(
			self,
			f: impl Fn(A) -> ArcFree<F, B> + Send + Sync + 'static,
		) -> ArcFree<F, B>
		where
			A: Clone + Send + Sync,
			Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<F, ArcTypeErasedValue>,
			>): Clone, {
			let erased_f =
				ArcContinuation(<ArcFnBrand as SendLiftFn>::new(move |val: ArcTypeErasedValue| {
					let arc_a: Arc<A> =
						val.downcast::<A>().expect("Type mismatch in ArcFree::bind");
					let a: A = Arc::try_unwrap(arc_a).unwrap_or_else(|shared| (*shared).clone());
					f(a).cast_phantom()
				}));
			let mut owned = self.into_inner_owned();
			let conts = std::mem::take(&mut owned.continuations);
			ArcFree::from_inner(ArcFreeInner {
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
		#[document_returns("A new `ArcFree` computation with the transformed result.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::*,
		/// };
		///
		/// let free = ArcFree::<IdentityBrand, _>::pure(10).map(|x: i32| x * 2);
		/// assert_eq!(free.evaluate(), 20);
		/// ```
		pub fn map<B: 'static + Send + Sync>(
			self,
			f: impl Fn(A) -> B + Send + Sync + 'static,
		) -> ArcFree<F, B>
		where
			A: Clone + Send + Sync,
			Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<F, ArcTypeErasedValue>,
			>): Clone, {
			self.bind(move |a| ArcFree::pure(f(a)))
		}

		/// Creates a suspended computation from a functor value.
		#[document_signature]
		///
		#[document_parameters("The functor value containing the next step.")]
		///
		#[document_returns("An `ArcFree` computation that performs the effect `fa`.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::*,
		/// };
		///
		/// let inner = ArcFree::<IdentityBrand, _>::pure(7);
		/// let free: ArcFree<IdentityBrand, _> = ArcFree::wrap(Identity(inner));
		/// assert_eq!(free.evaluate(), 7);
		/// ```
		pub fn wrap(
			fa: Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, ArcFree<F, A>>)
		) -> Self
		where
			F: SendFunctor,
			A: Send + Sync,
			Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<F, ArcTypeErasedValue>,
			>): Clone, {
			let erased_fa = F::send_map(
				|inner: ArcFree<F, A>| -> ArcFree<F, ArcTypeErasedValue> { inner.cast_phantom() },
				fa,
			);
			ArcFree::from_inner(ArcFreeInner {
				view: Some(ArcFreeView::Suspend(erased_fa)),
				continuations: CatList::empty(),
				_marker: PhantomData,
			})
		}

		/// Lifts a functor value into the [`ArcFree`] monad.
		#[document_signature]
		///
		#[document_parameters("The functor value to lift.")]
		///
		#[document_returns(
			"An `ArcFree` computation that performs the effect and returns the result."
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
		/// let free: ArcFree<IdentityBrand, _> = ArcFree::lift_f(id);
		/// assert_eq!(free.evaluate(), 42);
		/// ```
		pub fn lift_f(fa: Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, A>)) -> Self
		where
			F: SendFunctor,
			A: Send + Sync,
			Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<F, ArcTypeErasedValue>,
			>): Clone, {
			ArcFree::wrap(F::send_map(ArcFree::pure, fa))
		}

		/// Decomposes this `ArcFree` into a single [`ArcFreeStep`].
		///
		/// Iteratively applies pending continuations until a final value or
		/// a suspended functor layer is reached.
		#[document_signature]
		///
		#[document_returns(
			"[`ArcFreeStep::Done`] if complete, or [`ArcFreeStep::Suspended`] if suspended in the functor `F`."
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
		/// let free = ArcFree::<IdentityBrand, _>::pure(42);
		/// match free.to_view() {
		/// 	ArcFreeStep::Done(a) => assert_eq!(a, 42),
		/// 	ArcFreeStep::Suspended(_) => panic!("expected Done"),
		/// }
		/// ```
		#[expect(
			clippy::expect_used,
			reason = "ArcFree values consumed exactly once per branch; double consumption indicates a bug"
		)]
		pub fn to_view(self) -> ArcFreeStep<F, A>
		where
			F: SendFunctor,
			A: Clone + Send + Sync,
			Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<F, ArcTypeErasedValue>,
			>): Clone, {
			let mut owned = self.into_inner_owned();
			let mut current_view = owned.view.take().expect("ArcFree value already consumed");
			let mut conts = std::mem::take(&mut owned.continuations);

			loop {
				match current_view {
					ArcFreeView::Return(val) => match conts.uncons() {
						Some((continuation, rest)) => {
							let next = (continuation.0)(val);
							let mut next_owned = next.into_inner_owned();
							current_view = next_owned
								.view
								.take()
								.expect("ArcFree value already consumed (continuation)");
							let next_conts = std::mem::take(&mut next_owned.continuations);
							conts = next_conts.append(rest);
						}
						None => {
							let arc_a: Arc<A> = val
								.downcast::<A>()
								.expect("Type mismatch in ArcFree::to_view final downcast");
							let a: A =
								Arc::try_unwrap(arc_a).unwrap_or_else(|shared| (*shared).clone());
							return ArcFreeStep::Done(a);
						}
					},
					ArcFreeView::Suspend(fa) => {
						let downcast_cont = ArcContinuation(<ArcFnBrand as SendLiftFn>::new(
							move |val: ArcTypeErasedValue| {
								let arc_a: Arc<A> = val
									.downcast::<A>()
									.expect("Type mismatch in ArcFree::to_view downcast");
								let a: A = Arc::try_unwrap(arc_a)
									.unwrap_or_else(|shared| (*shared).clone());
								ArcFree::<F, A>::pure(a).cast_phantom()
							},
						));
						let all_conts = conts.snoc(downcast_cont);
						let remaining = std::sync::Mutex::new(Some(all_conts));
						let typed_fa = F::send_map(
							move |inner_free: ArcFree<F, ArcTypeErasedValue>| {
								let conts_for_inner = remaining
									.lock()
									.expect("ArcFree::to_view mutex poisoned")
									.take()
									.expect("ArcFree::to_view map called more than once");
								let mut owned_inner = inner_free.into_inner_owned();
								let v = owned_inner.view.take();
								let c = std::mem::take(&mut owned_inner.continuations);
								ArcFree::from_inner(ArcFreeInner {
									view: v,
									continuations: c.append(conts_for_inner),
									_marker: PhantomData,
								})
							},
							fa,
						);
						return ArcFreeStep::Suspended(typed_fa);
					}
				}
			}
		}

		/// Decomposes this `ArcFree` into one step.
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
		/// let free = ArcFree::<IdentityBrand, _>::pure(42);
		/// assert!(matches!(free.resume(), Ok(42)));
		/// ```
		pub fn resume(
			self
		) -> Result<A, Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, ArcFree<F, A>>)>
		where
			F: SendFunctor,
			A: Clone + Send + Sync,
			Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<F, ArcTypeErasedValue>,
			>): Clone, {
			match self.to_view() {
				ArcFreeStep::Done(a) => Ok(a),
				ArcFreeStep::Suspended(fa) => Err(fa),
			}
		}

		/// Executes the `ArcFree` computation, returning the final result.
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
		/// let free = ArcFree::<IdentityBrand, _>::pure(42);
		/// assert_eq!(free.evaluate(), 42);
		/// ```
		pub fn evaluate(self) -> A
		where
			F: Extract + SendFunctor,
			A: Clone + Send + Sync,
			Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<F, ArcTypeErasedValue>,
			>): Clone, {
			let mut current = self;
			loop {
				match current.to_view() {
					ArcFreeStep::Done(a) => return a,
					ArcFreeStep::Suspended(fa) => {
						current = <F as Extract>::extract(fa);
					}
				}
			}
		}

		/// Non-consuming counterpart to [`evaluate`](ArcFree::evaluate):
		/// clones the structure (O(1) atomic refcount bump) and runs the
		/// consuming version on the clone.
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
		/// let free = ArcFree::<IdentityBrand, _>::pure(42);
		/// assert_eq!(free.lower_ref(), 42);
		/// // Original still usable.
		/// assert_eq!(free.evaluate(), 42);
		/// ```
		pub fn lower_ref(&self) -> A
		where
			F: Extract + SendFunctor,
			A: Clone + Send + Sync,
			Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<F, ArcTypeErasedValue>,
			>): Clone, {
			self.clone().evaluate()
		}

		/// Non-consuming counterpart to [`to_view`](ArcFree::to_view).
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
		/// let free = ArcFree::<IdentityBrand, _>::pure(42);
		/// match free.peel_ref() {
		/// 	ArcFreeStep::Done(a) => assert_eq!(a, 42),
		/// 	ArcFreeStep::Suspended(_) => panic!("expected Done"),
		/// }
		/// // Original still usable.
		/// assert_eq!(free.evaluate(), 42);
		/// ```
		pub fn peel_ref(&self) -> ArcFreeStep<F, A>
		where
			F: SendFunctor,
			A: Clone + Send + Sync,
			Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<F, ArcTypeErasedValue>,
			>): Clone, {
			self.clone().to_view()
		}

		/// Transforms the functor layer of this `ArcFree` via a natural
		/// transformation, mirroring
		/// [`RcFree::hoist_free`](crate::types::RcFree::hoist_free).
		#[document_signature]
		///
		#[document_type_parameters("The target functor brand.")]
		///
		#[document_parameters("The natural transformation from `F` to `G`.")]
		///
		#[document_returns("An `ArcFree` computation over functor `G` with the same result.")]
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
		/// let free: ArcFree<IdentityBrand, i32> = ArcFree::lift_f(Identity(42));
		/// let hoisted: ArcFree<IdentityBrand, i32> = free.hoist_free(IdToId);
		/// assert_eq!(hoisted.evaluate(), 42);
		/// ```
		pub fn hoist_free<G>(
			self,
			nt: impl NaturalTransformation<F, G> + Clone + Send + Sync + 'static,
		) -> ArcFree<G, A>
		where
			F: SendFunctor,
			G: WrapDrop
				+ SendFunctor
				+ Kind_cdc7cd43dac7585f<Of<'static, ArcFree<G, ArcTypeErasedValue>>: Send + Sync>
				+ 'static,
			A: Clone + Send + Sync,
			Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<F, ArcTypeErasedValue>,
			>): Clone,
			Apply!(<G as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<G, ArcTypeErasedValue>,
			>): Clone, {
			match self.resume() {
				Ok(a) => ArcFree::pure(a),
				Err(fa) => {
					let nt_clone = nt.clone();
					let ga = nt.transform(fa);
					ArcFree::<G, ArcFree<F, A>>::lift_f(ga)
						.bind(move |inner: ArcFree<F, A>| inner.hoist_free(nt_clone.clone()))
				}
			}
		}
	}
}

pub use inner::*;

#[cfg(test)]
#[expect(
	clippy::panic,
	clippy::unwrap_used,
	reason = "Tests use panicking operations for brevity and clarity"
)]
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
		let free = ArcFree::<IdentityBrand, _>::pure(42);
		assert_eq!(free.evaluate(), 42);
	}

	#[test]
	fn wrap_evaluate() {
		let inner = ArcFree::<IdentityBrand, _>::pure(7);
		let free: ArcFree<IdentityBrand, _> = ArcFree::wrap(Identity(inner));
		assert_eq!(free.evaluate(), 7);
	}

	#[test]
	fn lift_f_evaluate() {
		let free: ArcFree<IdentityBrand, _> = ArcFree::lift_f(Identity(99));
		assert_eq!(free.evaluate(), 99);
	}

	#[test]
	fn bind_chains() {
		let free = ArcFree::<IdentityBrand, _>::pure(1)
			.bind(|x: i32| ArcFree::pure(x + 1))
			.bind(|x: i32| ArcFree::pure(x * 10));
		assert_eq!(free.evaluate(), 20);
	}

	#[test]
	fn map_transforms() {
		let free = ArcFree::<IdentityBrand, _>::pure(10).map(|x: i32| x * 2);
		assert_eq!(free.evaluate(), 20);
	}

	#[test]
	fn clone_branches_independent() {
		let free = ArcFree::<IdentityBrand, _>::pure(42).bind(|x: i32| ArcFree::pure(x + 1));
		let branch = free.clone();
		assert_eq!(free.evaluate(), 43);
		assert_eq!(branch.evaluate(), 43);
	}

	#[test]
	fn lower_ref_does_not_consume() {
		let free = ArcFree::<IdentityBrand, _>::pure(7).bind(|x: i32| ArcFree::pure(x * 6));
		assert_eq!(free.lower_ref(), 42);
		assert_eq!(free.lower_ref(), 42);
		assert_eq!(free.evaluate(), 42);
	}

	#[test]
	fn peel_ref_does_not_consume() {
		let free = ArcFree::<IdentityBrand, _>::pure(123);
		match free.peel_ref() {
			ArcFreeStep::Done(a) => assert_eq!(a, 123),
			ArcFreeStep::Suspended(_) => panic!("expected Done"),
		}
		assert_eq!(free.evaluate(), 123);
	}

	#[test]
	fn cross_thread_via_spawn() {
		// The defining capability of ArcFree: send a program to another
		// thread and run it there.
		let free = ArcFree::<IdentityBrand, _>::pure(10).bind(|x: i32| ArcFree::pure(x * 4));
		let handle = std::thread::spawn(move || free.evaluate());
		assert_eq!(handle.join().unwrap(), 40);
	}

	#[test]
	fn cross_thread_clone_branches() {
		// Clone the program first; send each branch to its own thread.
		let program = ArcFree::<IdentityBrand, _>::pure(5).bind(|x: i32| ArcFree::pure(x + 100));
		let branch_a = program.clone();
		let branch_b = program;
		let handle_a = std::thread::spawn(move || branch_a.evaluate());
		let handle_b = std::thread::spawn(move || branch_b.evaluate());
		assert_eq!(handle_a.join().unwrap(), 105);
		assert_eq!(handle_b.join().unwrap(), 105);
	}

	#[test]
	fn deep_evaluate_does_not_overflow() {
		const DEPTH: usize = 100_000;
		let mut free: ArcFree<IdentityBrand, i32> = ArcFree::pure(0);
		for _ in 0 .. DEPTH {
			free = ArcFree::wrap(Identity(free));
		}
		assert_eq!(free.evaluate(), 0);
	}

	#[test]
	fn deep_drop_does_not_overflow() {
		const DEPTH: usize = 100_000;
		let mut free: ArcFree<IdentityBrand, i32> = ArcFree::pure(0);
		for _ in 0 .. DEPTH {
			free = ArcFree::wrap(Identity(free));
		}
		drop(free);
	}

	#[test]
	fn stack_safe_left_associated_bind() {
		fn count_down(n: i32) -> ArcFree<IdentityBrand, i32> {
			if n == 0 { ArcFree::pure(0) } else { ArcFree::pure(n).bind(|n| count_down(n - 1)) }
		}
		assert_eq!(count_down(10_000).evaluate(), 0);
	}

	#[test]
	fn is_send_and_sync() {
		fn assert_send<T: Send>(_: &T) {}
		fn assert_sync<T: Sync>(_: &T) {}
		let free = ArcFree::<IdentityBrand, i32>::pure(42);
		assert_send(&free);
		assert_sync(&free);
	}
}
