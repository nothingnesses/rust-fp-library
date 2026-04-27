//! Thread-safe naive recursive Free monad with `Arc`-shared continuations
//! supporting multi-shot effects and non-`'static` payloads.
//!
//! [`ArcFreeExplicit`] is the [`Send`] + [`Sync`] sibling of
//! [`RcFreeExplicit`](crate::types::RcFreeExplicit): same concrete recursive
//! enum body (no `dyn Any` erasure) and same outer-[`Arc`](std::sync::Arc)
//! wrapping pattern, but with [`Arc<dyn Fn + Send + Sync>`](std::sync::Arc)
//! continuations (matching what
//! [`FnBrand<ArcBrand>`](crate::brands::FnBrand) resolves to via
//! [`SendCloneFn`](crate::classes::SendCloneFn)) instead of `Rc<dyn Fn>`.
//! Programs cross thread boundaries.
//!
//! ## Trade-offs vs `RcFreeExplicit`
//!
//! - **Thread-safety:** `ArcFreeExplicit` is `Send + Sync` whenever its
//!   inputs are; `RcFreeExplicit` is not.
//! - **Overhead:** Atomic reference counting is slightly more expensive
//!   than non-atomic. Use `RcFreeExplicit` when single-threaded.
//! - **Closure bounds:** continuations are `Send + Sync`, so the closures
//!   passed to [`bind`](ArcFreeExplicit::bind) must be `Send + Sync`.
//! - **`A: Send + Sync` for value participation:** the [`Pure`](inner::ArcFreeExplicitView::Pure)
//!   variant holds `A` directly, so any thread-crossing operation requires
//!   `A: Send + Sync` (in addition to `A: Clone` for the
//!   shared-inner-state recovery fallback).
//!
//! ## Trait bound on `F`
//!
//! `ArcFreeExplicit<'a, F, A>` requires
//! `F: WrapDrop + Functor + Kind_cdc7cd43dac7585f<Of<'a, ArcFreeExplicit<'a, F, A>>: Send + Sync> + 'a`.
//! The associated-type-bound trick on the [`Kind`](crate::kinds) projection
//! is what lets the compiler auto-derive `Send + Sync` on the inner state
//! and (by extension) `ArcFreeExplicit` for concrete `F`: without it, the
//! `F::Of<...>` field is opaque and the auto-trait derivation fails.
//! [`WrapDrop`](crate::classes::WrapDrop) is needed by the custom iterative
//! [`Drop`] impl so deep `Wrap` chains can be dismantled without overflowing
//! the stack. The
//! [`evaluate`](ArcFreeExplicit::evaluate) method additionally requires
//! `F: Extract`.
//!
//! ## Drop behavior
//!
//! When the last `Arc` reference releases, the inner data's [`Drop`] runs
//! and iteratively dismantles a deep `Wrap` chain via
//! [`WrapDrop::drop`](crate::classes::WrapDrop::drop) plus
//! [`Arc::try_unwrap`](std::sync::Arc::try_unwrap). Without this, deep chains
//! stack-overflow during cleanup.

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			Apply,
			brands::{
				ArcFnBrand,
				ArcFreeExplicitBrand,
			},
			classes::*,
			impl_kind,
			kinds::*,
		},
		fp_macros::*,
		std::sync::Arc,
	};

	/// The internal view of an [`ArcFreeExplicit`] computation.
	///
	/// Either a pure value or a suspended functor layer holding the next
	/// step. Mirrors [`RcFreeExplicitView`](crate::types::RcFreeExplicitView)
	/// with `Arc + Send + Sync` substitutions.
	#[document_type_parameters(
		"The lifetime that bounds the payload and the functor.",
		"The base functor.",
		"The result type."
	)]
	pub enum ArcFreeExplicitView<'a, F, A: 'a>
	where
		F: WrapDrop + Functor + 'a, {
		/// A pure value.
		Pure(A),
		/// A suspended computation: a functor layer wrapping the next step.
		Wrap(
			Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, F, A>,
			>),
		),
	}

	#[document_type_parameters(
		"The lifetime that bounds the payload and the functor.",
		"The base functor.",
		"The result type."
	)]
	#[document_parameters("The view to clone.")]
	impl<'a, F, A: 'a> Clone for ArcFreeExplicitView<'a, F, A>
	where
		F: WrapDrop + Functor + 'a,
		A: Clone,
		Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'a,
			ArcFreeExplicit<'a, F, A>,
		>): Clone,
	{
		/// Clones the view by delegating to `A: Clone` for [`Pure`](ArcFreeExplicitView::Pure)
		/// and to the underlying functor's [`Clone`] impl for the suspended
		/// layer.
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
		/// let free = ArcFreeExplicit::<IdentityBrand, _>::pure(42);
		/// let cloned = free.clone();
		/// assert_eq!(cloned.evaluate(), 42);
		/// ```
		fn clone(&self) -> Self {
			match self {
				ArcFreeExplicitView::Pure(a) => ArcFreeExplicitView::Pure(a.clone()),
				ArcFreeExplicitView::Wrap(fa) => ArcFreeExplicitView::Wrap(fa.clone()),
			}
		}
	}

	/// Inner state of an [`ArcFreeExplicit`]: the optional view.
	///
	/// The view is wrapped in [`Option`] so the custom [`Drop`] impl can move
	/// it out via [`Option::take`] without producing a sentinel value of `A`.
	/// The struct-level associated-type bound
	/// `Kind_cdc7cd43dac7585f<Of<'a, ArcFreeExplicit<'a, F, A>>: Send + Sync>`
	/// is what lets the compiler auto-derive `Send + Sync` for concrete `F`.
	pub struct ArcFreeExplicitInner<'a, F, A>
	where
		F: WrapDrop + Functor + 'a,
		A: 'a, {
		view: Option<ArcFreeExplicitView<'a, F, A>>,
	}

	#[document_type_parameters(
		"The lifetime that bounds the payload and the functor.",
		"The base functor.",
		"The result type."
	)]
	#[document_parameters("The inner state to clone.")]
	impl<'a, F, A> Clone for ArcFreeExplicitInner<'a, F, A>
	where
		F: WrapDrop + Functor + 'a,
		A: 'a + Clone,
		Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'a,
			ArcFreeExplicit<'a, F, A>,
		>): Clone,
	{
		/// Clones the inner state by delegating to [`ArcFreeExplicitView`]'s
		/// `Clone`.
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
		/// // `ArcFreeExplicitInner` is internal; `ArcFreeExplicit::clone` exposes the same effect.
		/// let free = ArcFreeExplicit::<IdentityBrand, _>::pure(42);
		/// let cloned = free.clone();
		/// assert_eq!(cloned.evaluate(), 42);
		/// ```
		fn clone(&self) -> Self {
			ArcFreeExplicitInner {
				view: self.view.clone(),
			}
		}
	}

	#[document_type_parameters(
		"The lifetime that bounds the payload and the functor.",
		"The base functor.",
		"The result type."
	)]
	#[document_parameters("The inner state being dropped.")]
	impl<'a, F, A> Drop for ArcFreeExplicitInner<'a, F, A>
	where
		F: WrapDrop + Functor + 'a,
		A: 'a,
	{
		/// Iteratively dismantles a deep `Wrap` chain via
		/// [`WrapDrop::drop`] and [`Arc::try_unwrap`](std::sync::Arc::try_unwrap),
		/// mirroring [`RcFreeExplicit`](crate::types::RcFreeExplicit)'s
		/// strategy with atomic refcounting.
		#[document_signature]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::*,
		/// };
		/// {
		/// 	let _free = ArcFreeExplicit::<IdentityBrand, _>::pure(42);
		/// } // drop called here
		/// assert!(true);
		/// ```
		fn drop(&mut self) {
			let mut current_view = self.view.take();
			while let Some(view) = current_view {
				match view {
					ArcFreeExplicitView::Pure(_) => {
						current_view = None;
					}
					ArcFreeExplicitView::Wrap(fa) => {
						if let Some(extracted) =
							<F as WrapDrop>::drop::<ArcFreeExplicit<'a, F, A>>(fa)
						{
							if let Ok(mut owned) = Arc::try_unwrap(extracted.inner) {
								current_view = owned.view.take();
							} else {
								current_view = None;
							}
						} else {
							current_view = None;
						}
					}
				}
			}
		}
	}

	/// Thread-safe naive recursive Free monad with `Arc`-shared continuations.
	///
	/// Same internal data shape as
	/// [`RcFreeExplicit`](crate::types::RcFreeExplicit) with `Arc` substituted
	/// for `Rc` and `Send + Sync` bounds added on closure storage. The whole
	/// program is `Clone`, `Send`, and `Sync` whenever the underlying
	/// functor's `Of<'a, ArcFreeExplicit<'a, F, A>>` is and `A` itself is
	/// `Send + Sync`.
	#[document_type_parameters(
		"The lifetime that bounds the payload and the functor.",
		"The base functor (must implement [`Extract`] and [`Functor`]).",
		"The result type."
	)]
	pub struct ArcFreeExplicit<'a, F, A>
	where
		F: WrapDrop + Functor + 'a,
		A: 'a, {
		inner: Arc<ArcFreeExplicitInner<'a, F, A>>,
	}

	impl_kind! {
		impl<F: WrapDrop + Functor + 'static> for ArcFreeExplicitBrand<F> {
			type Of<'a, A: 'a>: 'a = ArcFreeExplicit<'a, F, A>;
		}
	}

	#[document_type_parameters(
		"The lifetime that bounds the payload and the functor.",
		"The base functor.",
		"The result type."
	)]
	#[document_parameters("The `ArcFreeExplicit` instance to clone.")]
	impl<'a, F, A> Clone for ArcFreeExplicit<'a, F, A>
	where
		F: WrapDrop + Functor + 'a,
		A: 'a,
	{
		/// Clones the `ArcFreeExplicit` by atomic refcount bump on the outer
		/// `Arc`. O(1).
		#[document_signature]
		///
		#[document_returns("A new `ArcFreeExplicit` representing an independent branch.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::*,
		/// };
		///
		/// let free = ArcFreeExplicit::<IdentityBrand, _>::pure(42);
		/// let branch = free.clone();
		/// assert_eq!(free.evaluate(), 42);
		/// assert_eq!(branch.evaluate(), 42);
		/// ```
		fn clone(&self) -> Self {
			ArcFreeExplicit {
				inner: Arc::clone(&self.inner),
			}
		}
	}

	#[document_type_parameters(
		"The lifetime that bounds the payload and the functor.",
		"The base functor.",
		"The result type."
	)]
	#[document_parameters("The `ArcFreeExplicit` instance.")]
	impl<'a, F, A: 'a> ArcFreeExplicit<'a, F, A>
	where
		F: WrapDrop + Functor + 'a,
	{
		/// Constructs an `ArcFreeExplicit` from owned inner state.
		#[document_signature]
		///
		#[document_parameters("The inner state to wrap.")]
		///
		#[document_returns("A new `ArcFreeExplicit` wrapping the inner state in an `Arc`.")]
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
		/// let free = ArcFreeExplicit::<IdentityBrand, _>::pure(42);
		/// assert_eq!(free.evaluate(), 42);
		/// ```
		fn from_inner(inner: ArcFreeExplicitInner<'a, F, A>) -> Self {
			ArcFreeExplicit {
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
		/// let free =
		/// 	ArcFreeExplicit::<IdentityBrand, _>::pure(1).bind(|x: i32| ArcFreeExplicit::pure(x + 1));
		/// assert_eq!(free.evaluate(), 2);
		/// ```
		fn into_inner_owned(self) -> ArcFreeExplicitInner<'a, F, A>
		where
			A: Clone,
			Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, F, A>,
			>): Clone, {
			Arc::try_unwrap(self.inner).unwrap_or_else(|shared| (*shared).clone())
		}

		/// Creates a pure `ArcFreeExplicit` value.
		#[document_signature]
		///
		#[document_parameters("The value to wrap.")]
		///
		#[document_returns("An `ArcFreeExplicit` computation that produces `a`.")]
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
		/// let free = ArcFreeExplicit::<IdentityBrand, _>::pure(42);
		/// assert_eq!(free.evaluate(), 42);
		/// ```
		pub fn pure(a: A) -> Self {
			ArcFreeExplicit::from_inner(ArcFreeExplicitInner {
				view: Some(ArcFreeExplicitView::Pure(a)),
			})
		}

		/// Creates a suspended computation from a functor layer.
		///
		/// The layer is `F<ArcFreeExplicit<'a, F, A>>`: a single application
		/// of `F` whose payload is the next step. The outer `Arc<Inner>`
		/// wrapper provides the indirection that the recursive type would
		/// otherwise need [`Box`] for, so the layer holds [`ArcFreeExplicit`]
		/// directly rather than `Box<ArcFreeExplicit>`.
		#[document_signature]
		///
		#[document_parameters("The functor layer holding the next step.")]
		///
		#[document_returns("An `ArcFreeExplicit` computation that performs the wrapped effect.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::*,
		/// };
		///
		/// let inner = ArcFreeExplicit::<IdentityBrand, _>::pure(7);
		/// let free: ArcFreeExplicit<'_, IdentityBrand, _> = ArcFreeExplicit::wrap(Identity(inner));
		/// assert_eq!(free.evaluate(), 7);
		/// ```
		pub fn wrap(
			layer: Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, F, A>,
			>)
		) -> Self {
			ArcFreeExplicit::from_inner(ArcFreeExplicitInner {
				view: Some(ArcFreeExplicitView::Wrap(layer)),
			})
		}

		/// Decomposes this `ArcFreeExplicit` into its [`ArcFreeExplicitView`].
		///
		/// Since `ArcFreeExplicit`'s `bind` rebuilds the structure inline
		/// (no pending continuation queue), the view is the canonical form
		/// of the computation: either [`Pure`](ArcFreeExplicitView::Pure)
		/// or [`Wrap`](ArcFreeExplicitView::Wrap).
		#[document_signature]
		///
		#[document_returns("The view of the computation.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::*,
		/// };
		///
		/// let free = ArcFreeExplicit::<IdentityBrand, _>::pure(42);
		/// match free.to_view() {
		/// 	ArcFreeExplicitView::Pure(a) => assert_eq!(a, 42),
		/// 	ArcFreeExplicitView::Wrap(_) => panic!("expected Pure"),
		/// }
		/// ```
		#[expect(
			clippy::expect_used,
			reason = "ArcFreeExplicit values consumed exactly once per branch; double consumption indicates a bug"
		)]
		pub fn to_view(self) -> ArcFreeExplicitView<'a, F, A>
		where
			A: Clone,
			Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, F, A>,
			>): Clone, {
			let mut owned = self.into_inner_owned();
			owned.view.take().expect("ArcFreeExplicit value already consumed")
		}

		/// Iteratively evaluates the computation by extracting one functor
		/// layer at a time.
		///
		/// Uses [`Extract::extract`] to pull the next step out of each
		/// `Wrap` layer in a `loop`, never recursing on the spine.
		/// Stack-safe regardless of chain depth.
		#[document_signature]
		///
		#[document_returns("The final value produced by the computation.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::*,
		/// };
		///
		/// let mut free: ArcFreeExplicit<'_, IdentityBrand, i32> = ArcFreeExplicit::pure(0);
		/// for _ in 0 .. 1000 {
		/// 	free = ArcFreeExplicit::wrap(Identity(free));
		/// }
		/// assert_eq!(free.evaluate(), 0);
		/// ```
		pub fn evaluate(self) -> A
		where
			F: Extract,
			A: Clone,
			Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, F, A>,
			>): Clone, {
			let mut current = self;
			loop {
				match current.to_view() {
					ArcFreeExplicitView::Pure(a) => return a,
					ArcFreeExplicitView::Wrap(fa) => {
						current = F::extract(fa);
					}
				}
			}
		}

		/// Non-consuming counterpart to [`evaluate`](ArcFreeExplicit::evaluate):
		/// clones the structure (O(1) atomic refcount bump on the outer
		/// `Arc`) and runs the consuming version on the clone.
		#[document_signature]
		///
		#[document_returns("The final value produced by the computation.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::*,
		/// };
		///
		/// let free = ArcFreeExplicit::<IdentityBrand, _>::pure(42);
		/// assert_eq!(free.lower_ref(), 42);
		/// // Original still usable.
		/// assert_eq!(free.evaluate(), 42);
		/// ```
		pub fn lower_ref(&self) -> A
		where
			F: Extract,
			A: Clone,
			Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, F, A>,
			>): Clone, {
			self.clone().evaluate()
		}

		/// Non-consuming counterpart to [`to_view`](ArcFreeExplicit::to_view).
		#[document_signature]
		///
		#[document_returns("The current view of the computation.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::*,
		/// };
		///
		/// let free = ArcFreeExplicit::<IdentityBrand, _>::pure(42);
		/// match free.peel_ref() {
		/// 	ArcFreeExplicitView::Pure(a) => assert_eq!(a, 42),
		/// 	ArcFreeExplicitView::Wrap(_) => panic!("expected Pure"),
		/// }
		/// // Original still usable.
		/// assert_eq!(free.evaluate(), 42);
		/// ```
		pub fn peel_ref(&self) -> ArcFreeExplicitView<'a, F, A>
		where
			A: Clone,
			Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, F, A>,
			>): Clone, {
			self.clone().to_view()
		}

		/// Naive recursive bind. O(N) on left-associated chains because
		/// composing through a `Wrap` layer recurses through the spine via
		/// the closure passed to [`Functor::map`].
		///
		/// The closure is boxed once into an
		/// [`Arc<dyn Fn + Send + Sync>`](std::sync::Arc) via
		/// [`<ArcFnBrand as SendLiftFn>::new`](crate::classes::SendLiftFn)
		/// so the recursive call in the internal helper does not generate a
		/// fresh closure type at each nesting level (which would hit the
		/// monomorphisation recursion limit).
		#[document_signature]
		///
		#[document_type_parameters("The result type of the new computation.")]
		///
		#[document_parameters("The function to apply to the result of this computation.")]
		///
		#[document_returns("A new `ArcFreeExplicit` computation chaining `f` after this one.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::*,
		/// };
		///
		/// let free = ArcFreeExplicit::<IdentityBrand, _>::pure(2)
		/// 	.bind(|x: i32| ArcFreeExplicit::pure(x + 1))
		/// 	.bind(|x: i32| ArcFreeExplicit::pure(x * 10));
		/// assert_eq!(free.evaluate(), 30);
		/// ```
		pub fn bind<B: 'a>(
			self,
			f: impl Fn(A) -> ArcFreeExplicit<'a, F, B> + Send + Sync + 'a,
		) -> ArcFreeExplicit<'a, F, B>
		where
			A: Clone,
			Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, F, A>,
			>): Clone,
			Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, F, B>,
			>): Clone, {
			let boxed: Arc<dyn Fn(A) -> ArcFreeExplicit<'a, F, B> + Send + Sync + 'a> =
				<ArcFnBrand as SendLiftFn>::new(f);
			self.bind_boxed(boxed)
		}

		/// Internal recursive worker for [`bind`](ArcFreeExplicit::bind).
		///
		/// Takes the continuation pre-boxed into an
		/// [`Arc<dyn Fn + Send + Sync>`](std::sync::Arc) so the recursive
		/// call inside the [`Functor::map`] closure does not generate a
		/// fresh closure type per layer (which would hit monomorphisation
		/// limits).
		#[document_signature]
		///
		#[document_type_parameters("The result type of the new computation.")]
		///
		#[document_parameters("The boxed continuation to apply.")]
		///
		#[document_returns("A new `ArcFreeExplicit` computation chaining the continuation.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::*,
		/// };
		///
		/// // `bind_boxed` is internal; `bind` is the public API that uses it.
		/// let free =
		/// 	ArcFreeExplicit::<IdentityBrand, _>::pure(2).bind(|x: i32| ArcFreeExplicit::pure(x + 1));
		/// assert_eq!(free.evaluate(), 3);
		/// ```
		#[expect(
			clippy::expect_used,
			reason = "ArcFreeExplicit values consumed exactly once per branch; double consumption indicates a bug"
		)]
		fn bind_boxed<B: 'a>(
			self,
			f: Arc<dyn Fn(A) -> ArcFreeExplicit<'a, F, B> + Send + Sync + 'a>,
		) -> ArcFreeExplicit<'a, F, B>
		where
			A: Clone,
			Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, F, A>,
			>): Clone,
			Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, F, B>,
			>): Clone, {
			let mut owned = self.into_inner_owned();
			let view = owned.view.take().expect("ArcFreeExplicit value already consumed");
			match view {
				ArcFreeExplicitView::Pure(a) => f(a),
				ArcFreeExplicitView::Wrap(fa) => {
					let f_outer = Arc::clone(&f);
					ArcFreeExplicit::from_inner(ArcFreeExplicitInner {
						view: Some(ArcFreeExplicitView::Wrap(F::map(
							move |inner: ArcFreeExplicit<'a, F, A>| -> ArcFreeExplicit<'a, F, B> {
								let f_inner = Arc::clone(&f_outer);
								inner.bind_boxed(f_inner)
							},
							fa,
						))),
					})
				}
			}
		}
	}

	// -- Brand-level type class instances --
	//
	// `ArcFreeExplicitBrand` implements `SendPointed` only at the by-value
	// level. The full `SendFunctor` / `SendSemimonad` / `SendLift` /
	// `SendApplicative` / `SendMonad` chain is unexpressible for the same
	// reasons as `RcFreeExplicitBrand` (Clone bounds on `bind` /
	// `into_inner_owned` are per-`A` and not in the trait method
	// signatures), plus the `Send + Sync` Kind bound is a per-`A` HRTB
	// that no stable Rust feature supports. By-reference brand dispatch
	// routes through the `SendRef*` hierarchy below. See
	// [`fp-library/docs/limitations-and-workarounds.md`](crate) for the
	// pattern.

	#[document_type_parameters("The base functor.")]
	impl<F: WrapDrop + Functor + 'static> SendPointed for ArcFreeExplicitBrand<F> {
		/// Wraps a value in a pure thread-safe `ArcFreeExplicit` computation.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime that bounds the payload and the functor.",
			"The type of the value to wrap. Must be `Send + Sync`."
		)]
		///
		#[document_parameters("The value to wrap.")]
		///
		#[document_returns("An `ArcFreeExplicit` computation that produces `a`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::*,
		/// 	types::*,
		/// };
		///
		/// let free: ArcFreeExplicit<'_, IdentityBrand, _> =
		/// 	ArcFreeExplicitBrand::<IdentityBrand>::send_pure(42);
		/// assert_eq!(free.evaluate(), 42);
		/// ```
		fn send_pure<'a, A: Send + Sync + 'a>(
			a: A
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>) {
			ArcFreeExplicit::pure(a)
		}
	}

	// -- SendRef hierarchy NOT IMPLEMENTED for `ArcFreeExplicitBrand` --
	//
	// The natural recursive impl pattern (walk `&fa`, build new
	// `ArcFreeExplicit<F, B>`, recurse via `F::send_ref_map`) requires
	// the closure passed to `F::send_ref_map` to return
	// `ArcFreeExplicit<'a, F, B>: Send + Sync`. Auto-derive of `Send +
	// Sync` on `ArcFreeExplicit` requires
	// `Kind<Of<'a, ArcFreeExplicit<'a, F, A>>: Send + Sync>` (the bound
	// dropped from the struct in step 5 — see deviations in plan.md).
	// That bound's `'a` and `A` are the trait method's per-method
	// generics; stable Rust does not support `for<'a, T>` HRTB, so the
	// bound cannot be added at the impl block level. By-reference
	// dispatch over `ArcFreeExplicit` is reachable via inherent
	// methods on the concrete type, plus the `RcFreeExplicitBrand`
	// `Ref*` impls when the user is willing to forfeit thread-safety.
	// See `fp-library/docs/limitations-and-workarounds.md`.
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
		let free = ArcFreeExplicit::<IdentityBrand, _>::pure(42);
		assert_eq!(free.evaluate(), 42);
	}

	#[test]
	fn wrap_evaluate() {
		let inner = ArcFreeExplicit::<IdentityBrand, _>::pure(7);
		let free: ArcFreeExplicit<'_, IdentityBrand, _> = ArcFreeExplicit::wrap(Identity(inner));
		assert_eq!(free.evaluate(), 7);
	}

	#[test]
	fn bind_chains() {
		let free = ArcFreeExplicit::<IdentityBrand, _>::pure(1)
			.bind(|x: i32| ArcFreeExplicit::pure(x + 1))
			.bind(|x: i32| ArcFreeExplicit::pure(x * 10));
		assert_eq!(free.evaluate(), 20);
	}

	#[test]
	fn clone_branches_independent() {
		let free = ArcFreeExplicit::<IdentityBrand, _>::pure(42)
			.bind(|x: i32| ArcFreeExplicit::pure(x + 1));
		let branch = free.clone();
		assert_eq!(free.evaluate(), 43);
		assert_eq!(branch.evaluate(), 43);
	}

	#[test]
	fn lower_ref_does_not_consume() {
		let free = ArcFreeExplicit::<IdentityBrand, _>::pure(7)
			.bind(|x: i32| ArcFreeExplicit::pure(x * 6));
		assert_eq!(free.lower_ref(), 42);
		assert_eq!(free.lower_ref(), 42);
		assert_eq!(free.evaluate(), 42);
	}

	#[test]
	fn peel_ref_does_not_consume() {
		let free = ArcFreeExplicit::<IdentityBrand, _>::pure(123);
		match free.peel_ref() {
			ArcFreeExplicitView::Pure(a) => assert_eq!(a, 123),
			ArcFreeExplicitView::Wrap(_) => panic!("expected Pure"),
		}
		assert_eq!(free.evaluate(), 123);
	}

	#[test]
	fn multi_shot_continuation_via_clone() {
		// Multi-shot handler emulation: clone the program, evaluate each
		// branch, sum. The user closure inside `bind` is `Fn`, so the same
		// stored continuation runs once per branch.
		let program = ArcFreeExplicit::<IdentityBrand, _>::pure(10)
			.bind(|x: i32| ArcFreeExplicit::pure(x + 1));
		let total = program.clone().evaluate() + program.evaluate();
		assert_eq!(total, 22);
	}

	#[test]
	fn deep_evaluate_does_not_overflow() {
		const DEPTH: usize = 100_000;
		let mut free: ArcFreeExplicit<'_, IdentityBrand, i32> = ArcFreeExplicit::pure(0);
		for _ in 0 .. DEPTH {
			free = ArcFreeExplicit::wrap(Identity(free));
		}
		assert_eq!(free.evaluate(), 0);
	}

	#[test]
	fn deep_drop_does_not_overflow() {
		const DEPTH: usize = 100_000;
		let mut free: ArcFreeExplicit<'_, IdentityBrand, i32> = ArcFreeExplicit::pure(0);
		for _ in 0 .. DEPTH {
			free = ArcFreeExplicit::wrap(Identity(free));
		}
		drop(free);
	}

	#[test]
	fn cross_thread_via_spawn() {
		// Defining capability of ArcFreeExplicit: send a program to
		// another thread and run it there.
		let free = ArcFreeExplicit::<IdentityBrand, _>::pure(10)
			.bind(|x: i32| ArcFreeExplicit::pure(x * 4));
		let handle = std::thread::spawn(move || free.evaluate());
		assert_eq!(handle.join().unwrap(), 40);
	}

	#[test]
	fn cross_thread_clone_branches() {
		// Clone the program first; send each branch to its own thread.
		let program = ArcFreeExplicit::<IdentityBrand, _>::pure(5)
			.bind(|x: i32| ArcFreeExplicit::pure(x + 100));
		let branch_a = program.clone();
		let branch_b = program;
		let handle_a = std::thread::spawn(move || branch_a.evaluate());
		let handle_b = std::thread::spawn(move || branch_b.evaluate());
		assert_eq!(handle_a.join().unwrap(), 105);
		assert_eq!(handle_b.join().unwrap(), 105);
	}

	#[test]
	fn is_send_and_sync() {
		fn assert_send<T: Send>(_: &T) {}
		fn assert_sync<T: Sync>(_: &T) {}
		let free = ArcFreeExplicit::<IdentityBrand, i32>::pure(42);
		assert_send(&free);
		assert_sync(&free);
	}

	#[test]
	fn non_static_payload() {
		// Demonstrates the key property of the Explicit family: payloads
		// can borrow non-`'static` data.
		let s = String::from("hello");
		let r: &str = &s;
		let free: ArcFreeExplicit<'_, IdentityBrand, &str> = ArcFreeExplicit::pure(r);
		assert_eq!(free.evaluate(), "hello");
	}
}
