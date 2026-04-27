//! Naive recursive Free monad with `Rc`-shared continuations supporting
//! multi-shot effects and non-`'static` payloads.
//!
//! [`RcFreeExplicit`] is the multi-shot, [`Clone`]-cheap sibling of
//! [`FreeExplicit`](crate::types::FreeExplicit): same concrete recursive enum
//! body (no `dyn Any` erasure), but the whole substrate lives behind an outer
//! [`Rc<Inner>`](std::rc::Rc) so cloning the program is O(1), and the
//! [`bind`](RcFreeExplicit::bind) worker boxes its continuation through
//! [`<RcFnBrand as LiftFn>::new`](crate::classes::LiftFn) so the unified
//! function-pointer abstraction is used on the construction path.
//!
//! ## When to use which
//!
//! Use [`FreeExplicit`](crate::types::FreeExplicit) when single-shot effects
//! over non-`'static` payloads suffice. Use `RcFreeExplicit` when an effect
//! needs to drive its continuation more than once (`Choose`, `Amb`,
//! probabilistic / non-deterministic search, backtracking parsers) and the
//! payload borrows non-`'static` data. Use [`RcFree`](crate::types::RcFree)
//! when the payload is `'static` and stack-safe O(1) bind matters; the cost
//! is `dyn Any` erasure, which forces `A: 'static` and opts out of Brand
//! dispatch.
//!
//! ## Trade-offs vs `FreeExplicit`
//!
//! - **Multi-shot:** the per-step continuation is `Rc<dyn Fn>`, so a handler
//!   can invoke the same suspended program multiple times via [`Clone`].
//!   `FreeExplicit` builds its bind worker on the same `Rc<dyn Fn>` shape but
//!   does not expose the program for cheap cloning.
//! - **Clone:** `RcFreeExplicit` is unconditionally [`Clone`] in O(1)
//!   (refcount bump on the outer `Rc`).
//! - **Bind requires `Clone` bounds:** every operation that walks the spine
//!   may go through [`Rc::try_unwrap`](std::rc::Rc::try_unwrap) and fall back
//!   to cloning the inner state, which propagates `A: Clone` and a `Clone`
//!   bound on the suspended functor layer.
//! - **Allocation per spine step:** [`bind`](RcFreeExplicit::bind) walks the
//!   spine via [`Functor::map`](crate::classes::Functor::map), allocating one
//!   `Rc` per layer (O(N) on left-associated chains). This is the same
//!   asymptotic cost as `FreeExplicit::bind` plus the extra `Rc` allocation.
//! - **Thread-safety:** `RcFreeExplicit` is `!Send`. Use
//!   `ArcFreeExplicit` for thread-safe contexts.
//!
//! ## Trait bound on `F`
//!
//! `RcFreeExplicit<'a, F, A>` requires `F: WrapDrop + Functor + 'a`. The
//! [`WrapDrop`](crate::classes::WrapDrop) bound is needed by the custom
//! iterative [`Drop`] impl so deep `Wrap` chains can be dismantled without
//! overflowing the stack. The
//! [`evaluate`](RcFreeExplicit::evaluate) method additionally requires
//! `F: Extract`.
//!
//! ## Drop behavior
//!
//! When the last `Rc` reference releases, the inner data's [`Drop`] runs and
//! iteratively dismantles a deep `Wrap` chain via
//! [`WrapDrop::drop`](crate::classes::WrapDrop::drop) plus
//! [`Rc::try_unwrap`](std::rc::Rc::try_unwrap). Without this, deep chains
//! stack-overflow during cleanup.

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			Apply,
			brands::{
				RcFnBrand,
				RcFreeExplicitBrand,
			},
			classes::*,
			impl_kind,
			kinds::*,
		},
		fp_macros::*,
		std::rc::Rc,
	};

	/// The internal view of an [`RcFreeExplicit`] computation.
	///
	/// Either a pure value or a suspended functor layer holding the next
	/// step. The variants are exposed so callers building custom interpreters
	/// can pattern-match, but typical use goes through
	/// [`pure`](RcFreeExplicit::pure), [`wrap`](RcFreeExplicit::wrap),
	/// [`bind`](RcFreeExplicit::bind), and [`evaluate`](RcFreeExplicit::evaluate).
	#[document_type_parameters(
		"The lifetime that bounds the payload and the functor.",
		"The base functor.",
		"The result type."
	)]
	pub enum RcFreeExplicitView<'a, F, A: 'a>
	where
		F: WrapDrop + 'a, {
		/// A pure value.
		Pure(A),
		/// A suspended computation: a functor layer wrapping the next step.
		Wrap(
			Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, F, A>,
			>),
		),
	}

	#[document_type_parameters(
		"The lifetime that bounds the payload and the functor.",
		"The base functor.",
		"The result type."
	)]
	#[document_parameters("The view to clone.")]
	impl<'a, F, A: 'a> Clone for RcFreeExplicitView<'a, F, A>
	where
		F: WrapDrop + 'a,
		A: Clone,
		Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'a,
			RcFreeExplicit<'a, F, A>,
		>): Clone,
	{
		/// Clones the view by delegating to `A: Clone` for [`Pure`](RcFreeExplicitView::Pure)
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
		/// let free = RcFreeExplicit::<IdentityBrand, _>::pure(42);
		/// let cloned = free.clone();
		/// assert_eq!(cloned.evaluate(), 42);
		/// ```
		fn clone(&self) -> Self {
			match self {
				RcFreeExplicitView::Pure(a) => RcFreeExplicitView::Pure(a.clone()),
				RcFreeExplicitView::Wrap(fa) => RcFreeExplicitView::Wrap(fa.clone()),
			}
		}
	}

	/// Inner state of an [`RcFreeExplicit`]: the optional view.
	///
	/// The view is wrapped in [`Option`] so the custom [`Drop`] impl can move
	/// it out via [`Option::take`] without producing a sentinel value of `A`.
	struct RcFreeExplicitInner<'a, F, A>
	where
		F: WrapDrop + 'a,
		A: 'a, {
		view: Option<RcFreeExplicitView<'a, F, A>>,
	}

	#[document_type_parameters(
		"The lifetime that bounds the payload and the functor.",
		"The base functor.",
		"The result type."
	)]
	#[document_parameters("The inner state to clone.")]
	impl<'a, F, A> Clone for RcFreeExplicitInner<'a, F, A>
	where
		F: WrapDrop + 'a,
		A: 'a + Clone,
		Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'a,
			RcFreeExplicit<'a, F, A>,
		>): Clone,
	{
		/// Clones the inner state by delegating to [`RcFreeExplicitView`]'s
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
		/// // `RcFreeExplicitInner` is internal; `RcFreeExplicit::clone` exposes the same effect.
		/// let free = RcFreeExplicit::<IdentityBrand, _>::pure(42);
		/// let cloned = free.clone();
		/// assert_eq!(cloned.evaluate(), 42);
		/// ```
		fn clone(&self) -> Self {
			RcFreeExplicitInner {
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
	impl<'a, F, A> Drop for RcFreeExplicitInner<'a, F, A>
	where
		F: WrapDrop + 'a,
		A: 'a,
	{
		/// Iteratively dismantles a deep `Wrap` chain via
		/// [`WrapDrop::drop`] and [`Rc::try_unwrap`](std::rc::Rc::try_unwrap).
		///
		/// At each `Wrap`, consult `WrapDrop::drop`. `Some(extracted)` lets
		/// the loop try to unwrap the inner [`RcFreeExplicit`]'s outer `Rc`;
		/// if unique, take its view (leaving `None`) and continue the loop
		/// (the just-emptied inner drops with `view: None`, so its own
		/// `Drop` becomes a no-op and never recurses). If shared, leave
		/// other holders to dismantle. `None` lets the layer drop in place,
		/// which is sound for brands that do not materially store the inner
		/// `RcFreeExplicit`.
		#[document_signature]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::*,
		/// };
		/// {
		/// 	let _free = RcFreeExplicit::<IdentityBrand, _>::pure(42);
		/// } // drop called here
		/// assert!(true);
		/// ```
		fn drop(&mut self) {
			let mut current_view = self.view.take();
			while let Some(view) = current_view {
				match view {
					RcFreeExplicitView::Pure(_) => {
						current_view = None;
					}
					RcFreeExplicitView::Wrap(fa) => {
						if let Some(extracted) =
							<F as WrapDrop>::drop::<RcFreeExplicit<'a, F, A>>(fa)
						{
							if let Ok(mut owned) = Rc::try_unwrap(extracted.inner) {
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

	/// Naive recursive Free monad with `Rc`-shared continuations.
	///
	/// Same internal data shape as
	/// [`FreeExplicit`](crate::types::FreeExplicit) but with the whole
	/// substrate behind an outer [`Rc`] wrapper so the program is cheaply
	/// cloneable, and the [`bind`](RcFreeExplicit::bind) worker boxes its
	/// continuation through
	/// [`<RcFnBrand as LiftFn>::new`](crate::classes::LiftFn) so the unified
	/// function-pointer abstraction is on the construction path. Multi-shot
	/// effects (`Choose`, `Amb`) drive the same stored program more than
	/// once, with [`Clone`] exposing the program independently to each
	/// handler branch.
	#[document_type_parameters(
		"The lifetime that bounds the payload and the functor.",
		"The base functor (must implement [`WrapDrop`]; the inherent methods additionally require [`Functor`], and `evaluate` / `lower_ref` additionally require [`Extract`]).",
		"The result type."
	)]
	pub struct RcFreeExplicit<'a, F, A>
	where
		F: WrapDrop + 'a,
		A: 'a, {
		inner: Rc<RcFreeExplicitInner<'a, F, A>>,
	}

	impl_kind! {
		impl<F: WrapDrop + 'static> for RcFreeExplicitBrand<F> {
			type Of<'a, A: 'a>: 'a = RcFreeExplicit<'a, F, A>;
		}
	}

	#[document_type_parameters(
		"The lifetime that bounds the payload and the functor.",
		"The base functor.",
		"The result type."
	)]
	#[document_parameters("The `RcFreeExplicit` instance to clone.")]
	impl<'a, F, A> Clone for RcFreeExplicit<'a, F, A>
	where
		F: WrapDrop + 'a,
		A: 'a,
	{
		/// Clones the `RcFreeExplicit` by bumping the refcount on the outer
		/// `Rc`. O(1).
		#[document_signature]
		///
		#[document_returns("A new `RcFreeExplicit` representing an independent branch.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::*,
		/// };
		///
		/// let free = RcFreeExplicit::<IdentityBrand, _>::pure(42);
		/// let branch = free.clone();
		/// assert_eq!(free.evaluate(), 42);
		/// assert_eq!(branch.evaluate(), 42);
		/// ```
		fn clone(&self) -> Self {
			RcFreeExplicit {
				inner: Rc::clone(&self.inner),
			}
		}
	}

	#[document_type_parameters(
		"The lifetime that bounds the payload and the functor.",
		"The base functor.",
		"The result type."
	)]
	#[document_parameters("The `RcFreeExplicit` instance.")]
	impl<'a, F, A: 'a> RcFreeExplicit<'a, F, A>
	where
		F: WrapDrop + Functor + 'a,
	{
		/// Constructs an `RcFreeExplicit` from owned inner state.
		#[document_signature]
		///
		#[document_parameters("The inner state to wrap.")]
		///
		#[document_returns("A new `RcFreeExplicit` wrapping the inner state in an `Rc`.")]
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
		/// let free = RcFreeExplicit::<IdentityBrand, _>::pure(42);
		/// assert_eq!(free.evaluate(), 42);
		/// ```
		fn from_inner(inner: RcFreeExplicitInner<'a, F, A>) -> Self {
			RcFreeExplicit {
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
		/// let free =
		/// 	RcFreeExplicit::<IdentityBrand, _>::pure(1).bind(|x: i32| RcFreeExplicit::pure(x + 1));
		/// assert_eq!(free.evaluate(), 2);
		/// ```
		fn into_inner_owned(self) -> RcFreeExplicitInner<'a, F, A>
		where
			A: Clone,
			Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, F, A>,
			>): Clone, {
			Rc::try_unwrap(self.inner).unwrap_or_else(|shared| (*shared).clone())
		}

		/// Creates a pure `RcFreeExplicit` value.
		#[document_signature]
		///
		#[document_parameters("The value to wrap.")]
		///
		#[document_returns("An `RcFreeExplicit` computation that produces `a`.")]
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
		/// let free = RcFreeExplicit::<IdentityBrand, _>::pure(42);
		/// assert_eq!(free.evaluate(), 42);
		/// ```
		pub fn pure(a: A) -> Self {
			RcFreeExplicit::from_inner(RcFreeExplicitInner {
				view: Some(RcFreeExplicitView::Pure(a)),
			})
		}

		/// Creates a suspended computation from a functor layer.
		///
		/// The layer is `F<RcFreeExplicit<'a, F, A>>`: a single application
		/// of `F` whose payload is the next step in the computation. The
		/// outer `Rc<Inner>` wrapper provides the indirection that the
		/// recursive type would otherwise need [`Box`] for, so the layer
		/// holds [`RcFreeExplicit`] directly rather than `Box<RcFreeExplicit>`.
		#[document_signature]
		///
		#[document_parameters("The functor layer holding the next step.")]
		///
		#[document_returns("An `RcFreeExplicit` computation that performs the wrapped effect.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::*,
		/// };
		///
		/// let inner = RcFreeExplicit::<IdentityBrand, _>::pure(7);
		/// let free: RcFreeExplicit<'_, IdentityBrand, _> = RcFreeExplicit::wrap(Identity(inner));
		/// assert_eq!(free.evaluate(), 7);
		/// ```
		pub fn wrap(
			layer: Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, F, A>,
			>)
		) -> Self {
			RcFreeExplicit::from_inner(RcFreeExplicitInner {
				view: Some(RcFreeExplicitView::Wrap(layer)),
			})
		}

		/// Decomposes this `RcFreeExplicit` into its [`RcFreeExplicitView`].
		///
		/// Since `RcFreeExplicit`'s `bind` rebuilds the structure inline
		/// (no pending continuation queue), the view is the canonical form
		/// of the computation: either [`Pure`](RcFreeExplicitView::Pure) or
		/// [`Wrap`](RcFreeExplicitView::Wrap).
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
		/// let free = RcFreeExplicit::<IdentityBrand, _>::pure(42);
		/// match free.to_view() {
		/// 	RcFreeExplicitView::Pure(a) => assert_eq!(a, 42),
		/// 	RcFreeExplicitView::Wrap(_) => panic!("expected Pure"),
		/// }
		/// ```
		#[expect(
			clippy::expect_used,
			reason = "RcFreeExplicit values consumed exactly once per branch; double consumption indicates a bug"
		)]
		pub fn to_view(self) -> RcFreeExplicitView<'a, F, A>
		where
			A: Clone,
			Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, F, A>,
			>): Clone, {
			let mut owned = self.into_inner_owned();
			owned.view.take().expect("RcFreeExplicit value already consumed")
		}

		/// Iteratively evaluates the computation by extracting one functor
		/// layer at a time.
		///
		/// Uses [`Extract::extract`] to pull the next step out of each `Wrap`
		/// layer in a `loop`, never recursing on the spine. Stack-safe
		/// regardless of chain depth.
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
		/// let mut free: RcFreeExplicit<'_, IdentityBrand, i32> = RcFreeExplicit::pure(0);
		/// for _ in 0 .. 1000 {
		/// 	free = RcFreeExplicit::wrap(Identity(free));
		/// }
		/// assert_eq!(free.evaluate(), 0);
		/// ```
		pub fn evaluate(self) -> A
		where
			F: Extract,
			A: Clone,
			Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, F, A>,
			>): Clone, {
			let mut current = self;
			loop {
				match current.to_view() {
					RcFreeExplicitView::Pure(a) => return a,
					RcFreeExplicitView::Wrap(fa) => {
						current = F::extract(fa);
					}
				}
			}
		}

		/// Non-consuming counterpart to [`evaluate`](RcFreeExplicit::evaluate):
		/// clones the structure (O(1) refcount bump on the outer `Rc`) and
		/// runs the consuming version on the clone.
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
		/// let free = RcFreeExplicit::<IdentityBrand, _>::pure(42);
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
				RcFreeExplicit<'a, F, A>,
			>): Clone, {
			self.clone().evaluate()
		}

		/// Non-consuming counterpart to [`to_view`](RcFreeExplicit::to_view).
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
		/// let free = RcFreeExplicit::<IdentityBrand, _>::pure(42);
		/// match free.peel_ref() {
		/// 	RcFreeExplicitView::Pure(a) => assert_eq!(a, 42),
		/// 	RcFreeExplicitView::Wrap(_) => panic!("expected Pure"),
		/// }
		/// // Original still usable.
		/// assert_eq!(free.evaluate(), 42);
		/// ```
		pub fn peel_ref(&self) -> RcFreeExplicitView<'a, F, A>
		where
			A: Clone,
			Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, F, A>,
			>): Clone, {
			self.clone().to_view()
		}

		/// Naive recursive bind. O(N) on left-associated chains because
		/// composing through a `Wrap` layer recurses through the spine via
		/// the closure passed to [`Functor::map`].
		///
		/// The closure is boxed once into an [`Rc<dyn Fn>`](std::rc::Rc) via
		/// [`<RcFnBrand as LiftFn>::new`](crate::classes::LiftFn) so the
		/// recursive call in the internal helper does not generate a fresh
		/// closure type at each nesting level (which would hit the
		/// monomorphisation recursion limit).
		#[document_signature]
		///
		#[document_type_parameters("The result type of the new computation.")]
		///
		#[document_parameters("The function to apply to the result of this computation.")]
		///
		#[document_returns("A new `RcFreeExplicit` computation chaining `f` after this one.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::*,
		/// };
		///
		/// let free = RcFreeExplicit::<IdentityBrand, _>::pure(2)
		/// 	.bind(|x: i32| RcFreeExplicit::pure(x + 1))
		/// 	.bind(|x: i32| RcFreeExplicit::pure(x * 10));
		/// assert_eq!(free.evaluate(), 30);
		/// ```
		pub fn bind<B: 'a>(
			self,
			f: impl Fn(A) -> RcFreeExplicit<'a, F, B> + 'a,
		) -> RcFreeExplicit<'a, F, B>
		where
			A: Clone,
			Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, F, A>,
			>): Clone,
			Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, F, B>,
			>): Clone, {
			let boxed: Rc<dyn Fn(A) -> RcFreeExplicit<'a, F, B> + 'a> =
				<RcFnBrand as LiftFn>::new(f);
			self.bind_boxed(boxed)
		}

		/// Internal recursive worker for [`bind`](RcFreeExplicit::bind).
		///
		/// Takes the continuation pre-boxed into an [`Rc<dyn Fn>`](std::rc::Rc)
		/// so the recursive call inside the [`Functor::map`] closure does not
		/// generate a fresh closure type per layer (which would hit
		/// monomorphisation limits).
		#[document_signature]
		///
		#[document_type_parameters("The result type of the new computation.")]
		///
		#[document_parameters("The boxed continuation to apply.")]
		///
		#[document_returns("A new `RcFreeExplicit` computation chaining the continuation.")]
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
		/// 	RcFreeExplicit::<IdentityBrand, _>::pure(2).bind(|x: i32| RcFreeExplicit::pure(x + 1));
		/// assert_eq!(free.evaluate(), 3);
		/// ```
		#[expect(
			clippy::expect_used,
			reason = "RcFreeExplicit values consumed exactly once per branch; double consumption indicates a bug"
		)]
		fn bind_boxed<B: 'a>(
			self,
			f: Rc<dyn Fn(A) -> RcFreeExplicit<'a, F, B> + 'a>,
		) -> RcFreeExplicit<'a, F, B>
		where
			A: Clone,
			Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, F, A>,
			>): Clone,
			Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, F, B>,
			>): Clone, {
			let mut owned = self.into_inner_owned();
			let view = owned.view.take().expect("RcFreeExplicit value already consumed");
			match view {
				RcFreeExplicitView::Pure(a) => f(a),
				RcFreeExplicitView::Wrap(fa) => {
					let f_outer = Rc::clone(&f);
					RcFreeExplicit::from_inner(RcFreeExplicitInner {
						view: Some(RcFreeExplicitView::Wrap(F::map(
							move |inner: RcFreeExplicit<'a, F, A>| -> RcFreeExplicit<'a, F, B> {
								let f_inner = Rc::clone(&f_outer);
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
	// `RcFreeExplicitBrand` implements `Pointed` only at the by-value level.
	// `Functor`, `Semimonad`, and the rest of the by-value applicative
	// chain are unexpressible because:
	//
	// 1. `RcFreeExplicit::bind` requires `A: Clone` and a per-`A` Clone
	//    bound on the suspended `F::Of` layer (because `into_inner_owned`
	//    falls back to cloning when the outer `Rc` is shared). The
	//    `Functor::map` / `Semimonad::bind` trait method signatures cannot
	//    add per-method bounds in stable Rust (no HRTB-over-types).
	//
	// 2. `Lift::lift2` natural impl `bind(fa, |a| map(fb, |b| f(a, b)))`
	//    requires `fb` to be reusable across closure invocations. Even
	//    though `RcFreeExplicit` is unconditionally `Clone`, the inherent
	//    `bind`/`map` it would dispatch through still carry the per-`A`
	//    bound from issue 1.
	//
	// `Pointed::pure` is unaffected because `RcFreeExplicit::pure` has no
	// Clone bound. By-reference dispatch routes through the `Ref*`
	// hierarchy below (which takes `&self` and avoids the consume-or-clone
	// issue). See [`fp-library/docs/limitations-and-workarounds.md`](crate)
	// for the broader pattern.

	#[document_type_parameters("The base functor.")]
	impl<F: WrapDrop + Functor + 'static> Pointed for RcFreeExplicitBrand<F> {
		/// Wraps a value in a pure `RcFreeExplicit` computation.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime that bounds the payload and the functor.",
			"The type of the value to wrap."
		)]
		///
		#[document_parameters("The value to wrap.")]
		///
		#[document_returns("An `RcFreeExplicit` computation that produces `a`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::*,
		/// 	types::*,
		/// };
		///
		/// let free: RcFreeExplicit<'_, IdentityBrand, _> = RcFreeExplicitBrand::<IdentityBrand>::pure(42);
		/// assert_eq!(free.evaluate(), 42);
		/// ```
		fn pure<'a, A: 'a>(a: A) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>) {
			RcFreeExplicit::pure(a)
		}
	}

	// -- Ref hierarchy --
	//
	// Walk the structure via `Rc::deref` and rebuild a fresh
	// `RcFreeExplicit<F, B>`. The recursive helper boxes the user closure
	// in `Rc<dyn Fn>` to avoid monomorphisation blow-up at each layer.

	/// Internal recursive worker for [`RefFunctor::ref_map`] over
	/// [`RcFreeExplicit`].
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	classes::*,
	/// 	types::*,
	/// };
	///
	/// let free = RcFreeExplicit::<IdentityBrand, _>::pure(10);
	/// let mapped = RcFreeExplicitBrand::<IdentityBrand>::ref_map(|x: &i32| *x + 1, &free);
	/// assert_eq!(mapped.evaluate(), 11);
	/// ```
	#[expect(
		clippy::expect_used,
		reason = "RcFreeExplicit values consumed exactly once per branch; double consumption is a bug"
	)]
	fn rc_free_explicit_ref_map<'a, F, A: 'a, B: 'a>(
		func: Rc<dyn Fn(&A) -> B + 'a>,
		fa: &RcFreeExplicit<'a, F, A>,
	) -> RcFreeExplicit<'a, F, B>
	where
		F: WrapDrop + Functor + RefFunctor + 'a, {
		let view = fa.inner.view.as_ref().expect("RcFreeExplicit value already consumed");
		match view {
			RcFreeExplicitView::Pure(a) => RcFreeExplicit::pure(func(a)),
			RcFreeExplicitView::Wrap(fa_inner) => {
				let func_outer = Rc::clone(&func);
				RcFreeExplicit::wrap(F::ref_map(
					move |inner: &RcFreeExplicit<'a, F, A>| -> RcFreeExplicit<'a, F, B> {
						let func_inner = Rc::clone(&func_outer);
						rc_free_explicit_ref_map(func_inner, inner)
					},
					fa_inner,
				))
			}
		}
	}

	/// Internal recursive worker for [`RefSemimonad::ref_bind`] over
	/// [`RcFreeExplicit`].
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	classes::*,
	/// 	types::*,
	/// };
	///
	/// let free = RcFreeExplicit::<IdentityBrand, _>::pure(2);
	/// let chained = RcFreeExplicitBrand::<IdentityBrand>::ref_bind(&free, |x: &i32| {
	/// 	RcFreeExplicit::pure(x + 1)
	/// });
	/// assert_eq!(chained.evaluate(), 3);
	/// ```
	#[expect(
		clippy::expect_used,
		clippy::type_complexity,
		reason = "RcFreeExplicit values consumed exactly once per branch; double consumption is a bug. Boxed closure type complexity is dictated by the recursive helper shape."
	)]
	fn rc_free_explicit_ref_bind<'a, F, A: 'a, B: 'a>(
		f: Rc<dyn Fn(&A) -> RcFreeExplicit<'a, F, B> + 'a>,
		fa: &RcFreeExplicit<'a, F, A>,
	) -> RcFreeExplicit<'a, F, B>
	where
		F: WrapDrop + Functor + RefFunctor + 'a, {
		let view = fa.inner.view.as_ref().expect("RcFreeExplicit value already consumed");
		match view {
			RcFreeExplicitView::Pure(a) => f(a),
			RcFreeExplicitView::Wrap(fa_inner) => {
				let f_outer = Rc::clone(&f);
				RcFreeExplicit::wrap(F::ref_map(
					move |inner: &RcFreeExplicit<'a, F, A>| -> RcFreeExplicit<'a, F, B> {
						let f_inner = Rc::clone(&f_outer);
						rc_free_explicit_ref_bind(f_inner, inner)
					},
					fa_inner,
				))
			}
		}
	}

	#[document_type_parameters("The base functor.")]
	impl<F: WrapDrop + Functor + RefFunctor + 'static> RefFunctor for RcFreeExplicitBrand<F> {
		/// Maps a function over the result of an `RcFreeExplicit`
		/// computation by reference.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime that bounds the payload and the functor.",
			"The original result type.",
			"The new result type."
		)]
		///
		#[document_parameters(
			"The function to apply by reference.",
			"The `RcFreeExplicit` computation."
		)]
		///
		#[document_returns("A new `RcFreeExplicit` with the function applied to its result.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::*,
		/// 	types::*,
		/// };
		///
		/// let free = RcFreeExplicit::<IdentityBrand, _>::pure(10);
		/// let mapped = RcFreeExplicitBrand::<IdentityBrand>::ref_map(|x: &i32| *x * 2, &free);
		/// assert_eq!(mapped.evaluate(), 20);
		/// ```
		fn ref_map<'a, A: 'a, B: 'a>(
			func: impl Fn(&A) -> B + 'a,
			fa: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			let func_rc: Rc<dyn Fn(&A) -> B + 'a> = Rc::new(func);
			rc_free_explicit_ref_map(func_rc, fa)
		}
	}

	#[document_type_parameters("The base functor.")]
	impl<F: WrapDrop + Functor + 'static> RefPointed for RcFreeExplicitBrand<F> {
		/// Wraps a cloned value in a pure `RcFreeExplicit` computation.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime that bounds the payload and the functor.",
			"The type of the value to wrap. Must be `Clone`."
		)]
		///
		#[document_parameters("A reference to the value to wrap.")]
		///
		#[document_returns("An `RcFreeExplicit` computation that produces a clone of `a`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::*,
		/// 	types::*,
		/// };
		///
		/// let value = 42;
		/// let free: RcFreeExplicit<'_, IdentityBrand, _> =
		/// 	RcFreeExplicitBrand::<IdentityBrand>::ref_pure(&value);
		/// assert_eq!(free.evaluate(), 42);
		/// ```
		fn ref_pure<'a, A: Clone + 'a>(
			a: &A
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>) {
			RcFreeExplicit::pure(a.clone())
		}
	}

	#[document_type_parameters("The base functor.")]
	impl<F: WrapDrop + Functor + RefFunctor + 'static> RefSemimonad for RcFreeExplicitBrand<F> {
		/// Sequences `RcFreeExplicit` computations using a reference to
		/// the intermediate value.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime that bounds the payload and the functor.",
			"The type of the result of the first computation.",
			"The type of the result of the second computation."
		)]
		///
		#[document_parameters(
			"The first `RcFreeExplicit` computation.",
			"The function to chain after the first computation."
		)]
		///
		#[document_returns("A new `RcFreeExplicit` chaining the function after `ma`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::*,
		/// 	types::*,
		/// };
		///
		/// let free = RcFreeExplicit::<IdentityBrand, _>::pure(2);
		/// let chained = RcFreeExplicitBrand::<IdentityBrand>::ref_bind(&free, |x: &i32| {
		/// 	RcFreeExplicit::pure(x + 1)
		/// });
		/// assert_eq!(chained.evaluate(), 3);
		/// ```
		#[expect(
			clippy::type_complexity,
			reason = "Boxed closure type is dictated by the recursive helper signature"
		)]
		fn ref_bind<'a, A: 'a, B: 'a>(
			ma: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			f: impl Fn(&A) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) + 'a,
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			let f_rc: Rc<dyn Fn(&A) -> RcFreeExplicit<'a, F, B> + 'a> = Rc::new(f);
			rc_free_explicit_ref_bind(f_rc, ma)
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
		let free = RcFreeExplicit::<IdentityBrand, _>::pure(42);
		assert_eq!(free.evaluate(), 42);
	}

	#[test]
	fn wrap_evaluate() {
		let inner = RcFreeExplicit::<IdentityBrand, _>::pure(7);
		let free: RcFreeExplicit<'_, IdentityBrand, _> = RcFreeExplicit::wrap(Identity(inner));
		assert_eq!(free.evaluate(), 7);
	}

	#[test]
	fn bind_chains() {
		let free = RcFreeExplicit::<IdentityBrand, _>::pure(1)
			.bind(|x: i32| RcFreeExplicit::pure(x + 1))
			.bind(|x: i32| RcFreeExplicit::pure(x * 10));
		assert_eq!(free.evaluate(), 20);
	}

	#[test]
	fn clone_branches_independent() {
		let free =
			RcFreeExplicit::<IdentityBrand, _>::pure(42).bind(|x: i32| RcFreeExplicit::pure(x + 1));
		let branch = free.clone();
		assert_eq!(free.evaluate(), 43);
		assert_eq!(branch.evaluate(), 43);
	}

	#[test]
	fn lower_ref_does_not_consume() {
		let free =
			RcFreeExplicit::<IdentityBrand, _>::pure(7).bind(|x: i32| RcFreeExplicit::pure(x * 6));
		assert_eq!(free.lower_ref(), 42);
		assert_eq!(free.lower_ref(), 42);
		assert_eq!(free.evaluate(), 42);
	}

	#[test]
	fn peel_ref_does_not_consume() {
		let free = RcFreeExplicit::<IdentityBrand, _>::pure(123);
		match free.peel_ref() {
			RcFreeExplicitView::Pure(a) => assert_eq!(a, 123),
			RcFreeExplicitView::Wrap(_) => panic!("expected Pure"),
		}
		assert_eq!(free.evaluate(), 123);
	}

	#[test]
	fn multi_shot_continuation_via_clone() {
		// Multi-shot handler emulation: clone the program, evaluate each
		// branch, sum. The user closure inside `bind` is `Fn`, so the same
		// stored continuation runs once per branch.
		let program =
			RcFreeExplicit::<IdentityBrand, _>::pure(10).bind(|x: i32| RcFreeExplicit::pure(x + 1));
		let total = program.clone().evaluate() + program.evaluate();
		assert_eq!(total, 22);
	}

	#[test]
	fn deep_evaluate_does_not_overflow() {
		const DEPTH: usize = 100_000;
		let mut free: RcFreeExplicit<'_, IdentityBrand, i32> = RcFreeExplicit::pure(0);
		for _ in 0 .. DEPTH {
			free = RcFreeExplicit::wrap(Identity(free));
		}
		assert_eq!(free.evaluate(), 0);
	}

	#[test]
	fn deep_drop_does_not_overflow() {
		const DEPTH: usize = 100_000;
		let mut free: RcFreeExplicit<'_, IdentityBrand, i32> = RcFreeExplicit::pure(0);
		for _ in 0 .. DEPTH {
			free = RcFreeExplicit::wrap(Identity(free));
		}
		drop(free);
	}

	#[test]
	fn non_static_payload() {
		// Demonstrates the key property of the Explicit family: payloads
		// can borrow non-`'static` data.
		let s = String::from("hello");
		let r: &str = &s;
		let free: RcFreeExplicit<'_, IdentityBrand, &str> = RcFreeExplicit::pure(r);
		assert_eq!(free.evaluate(), "hello");
	}
}
