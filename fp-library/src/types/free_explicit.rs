//! Naive recursive Free monad supporting non-`'static` payloads.
//!
//! [`FreeExplicit`] is the existential-free sibling of [`Free`](crate::types::Free):
//! the functor structure is kept as a concrete recursive enum rather than
//! erased through `Box<dyn Any>`, which lifts the `'static` requirement at
//! the cost of [`bind`](FreeExplicit::bind) walking the spine in O(N).
//!
//! ## When to use which
//!
//! Use [`Free`](crate::types::Free) when payloads are `'static` and stack-safe
//! O(1) bind matters. Use `FreeExplicit` when effect payloads need to borrow
//! non-`'static` data (`Reader<&str>`, `State<&'a mut T>`, handlers that
//! close over scoped environment data).
//!
//! ## Trait bound on `F`
//!
//! `FreeExplicit<'a, F, A>` requires `F: WrapDrop + Functor + 'a`. The
//! [`WrapDrop`](crate::classes::WrapDrop) bound is needed by the custom
//! iterative [`Drop`] impl so deep `Wrap` chains can be dismantled without
//! overflowing the stack. Functors whose payload is a continuation function
//! rather than an extractable value can opt into `WrapDrop::drop = None` so
//! that recursive drop runs in place; this is sound for the patterns
//! documented on `WrapDrop`. The
//! [`evaluate`](FreeExplicit::evaluate) method additionally requires
//! `F: Extract`. Functors whose payload doesn't yield a value at all should
//! route through [`Free::fold_free`](crate::types::Free::fold_free) into a
//! [`MonadRec`](crate::classes::MonadRec) target instead of relying on
//! `evaluate`.

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			Apply,
			brands::FreeExplicitBrand,
			classes::*,
			impl_kind,
			kinds::*,
		},
		fp_macros::*,
		std::rc::Rc,
	};

	/// The internal view of a [`FreeExplicit`] computation.
	///
	/// Either a pure value or a suspended functor layer holding the next
	/// step. The variants are exposed so callers building custom interpreters
	/// can pattern-match, but typical use goes through
	/// [`pure`](FreeExplicit::pure), [`wrap`](FreeExplicit::wrap),
	/// [`bind`](FreeExplicit::bind), and [`evaluate`](FreeExplicit::evaluate).
	#[document_type_parameters(
		"The lifetime that bounds the payload and the functor.",
		"The base functor.",
		"The result type."
	)]
	pub enum FreeExplicitView<'a, F, A: 'a>
	where
		F: WrapDrop + 'a, {
		/// A pure value.
		Pure(A),
		/// A suspended computation: a functor layer wrapping the next step.
		Wrap(
			Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				Box<FreeExplicit<'a, F, A>>,
			>),
		),
	}

	/// Naive recursive Free monad over a functor `F`, supporting non-`'static`
	/// payloads.
	///
	/// The internal view is wrapped in [`Option`] so the custom [`Drop`] impl
	/// can move it out via [`Option::take`] without producing a sentinel
	/// value of `A`. After `Drop` runs, or after a method consumes the view,
	/// `view` is `None`; subsequent access panics with "FreeExplicit value
	/// already consumed."
	///
	/// # Linear consumption invariant
	///
	/// `FreeExplicit` values must be consumed exactly once, either by calling
	/// [`evaluate`](FreeExplicit::evaluate), [`bind`](FreeExplicit::bind), or
	/// by being dropped. The `view` field is wrapped in `Option` to enable a
	/// take-and-replace pattern (analogous to `Cell::take`) so that
	/// `evaluate`, `bind`, and `Drop` can move the view out without leaving
	/// the struct in an invalid state.
	///
	/// # Drop behavior
	///
	/// Dropping iteratively dismantles a deep `Wrap` chain via
	/// [`WrapDrop::drop`], mirroring the strategy used by
	/// [`Free::drop`](crate::types::Free). Without this, a 100 000-deep
	/// `Wrap` chain stack-overflows during cleanup.
	#[document_type_parameters(
		"The lifetime that bounds the payload and the functor.",
		"The base functor (must implement [`WrapDrop`]; the inherent methods additionally require [`Functor`], and `evaluate` additionally requires [`Extract`]).",
		"The result type."
	)]
	pub struct FreeExplicit<'a, F, A: 'a>
	where
		F: WrapDrop + 'a, {
		view: Option<FreeExplicitView<'a, F, A>>,
	}

	impl_kind! {
		impl<F: WrapDrop + 'static> for FreeExplicitBrand<F> {
			type Of<'a, A: 'a>: 'a = FreeExplicit<'a, F, A>;
		}
	}

	#[document_type_parameters(
		"The lifetime that bounds the payload and the functor.",
		"The base functor.",
		"The result type."
	)]
	#[document_parameters("The `FreeExplicit` instance.")]
	impl<'a, F, A: 'a> FreeExplicit<'a, F, A>
	where
		F: WrapDrop + Functor + 'a,
	{
		/// Creates a pure `FreeExplicit` value.
		#[document_signature]
		///
		#[document_parameters("The value to wrap.")]
		///
		#[document_returns("A `FreeExplicit` computation that produces `a`.")]
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
		/// let free = FreeExplicit::<IdentityBrand, _>::pure(42);
		/// assert_eq!(free.evaluate(), 42);
		/// ```
		pub fn pure(a: A) -> Self {
			FreeExplicit {
				view: Some(FreeExplicitView::Pure(a)),
			}
		}

		/// Creates a suspended computation from a functor layer.
		///
		/// The layer is `F<Box<FreeExplicit<'a, F, A>>>`: a single application
		/// of `F` whose payload is the next step in the computation.
		#[document_signature]
		///
		#[document_parameters("The functor layer holding the next step.")]
		///
		#[document_returns("A `FreeExplicit` computation that performs the wrapped effect.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::*,
		/// };
		///
		/// let inner = FreeExplicit::<IdentityBrand, _>::pure(7);
		/// let free: FreeExplicit<'_, IdentityBrand, _> = FreeExplicit::wrap(Identity(Box::new(inner)));
		/// assert_eq!(free.evaluate(), 7);
		/// ```
		pub fn wrap(
			layer: Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				Box<FreeExplicit<'a, F, A>>,
			>)
		) -> Self {
			FreeExplicit {
				view: Some(FreeExplicitView::Wrap(layer)),
			}
		}

		/// Decomposes this `FreeExplicit` into its [`FreeExplicitView`].
		///
		/// Mirrors [`RcFreeExplicit::to_view`](crate::types::RcFreeExplicit)
		/// and [`ArcFreeExplicit::to_view`](crate::types::ArcFreeExplicit)
		/// at the by-value level, with no `Clone` bound (the unboxed outer
		/// struct does not have a refcount to recover).
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
		/// let free = FreeExplicit::<IdentityBrand, _>::pure(42);
		/// match free.to_view() {
		/// 	FreeExplicitView::Pure(a) => assert_eq!(a, 42),
		/// 	FreeExplicitView::Wrap(_) => panic!("expected Pure"),
		/// }
		/// ```
		#[expect(
			clippy::expect_used,
			reason = "FreeExplicit values consumed exactly once; double consumption is a bug"
		)]
		pub fn to_view(mut self) -> FreeExplicitView<'a, F, A> {
			self.view.take().expect("FreeExplicit value already consumed")
		}

		/// Iteratively evaluates the computation by extracting one functor
		/// layer at a time.
		///
		/// Uses [`Extract::extract`] to pull the next step out of each `Wrap`
		/// layer in a `loop`, never recursing. Stack-safe regardless of
		/// chain depth.
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
		/// let mut free: FreeExplicit<'_, IdentityBrand, i32> = FreeExplicit::pure(0);
		/// for _ in 0 .. 1000 {
		/// 	free = FreeExplicit::wrap(Identity(Box::new(free)));
		/// }
		/// assert_eq!(free.evaluate(), 0);
		/// ```
		#[expect(
			clippy::expect_used,
			reason = "FreeExplicit values consumed exactly once; double consumption is a bug"
		)]
		pub fn evaluate(self) -> A
		where
			F: Extract, {
			let mut current = self;
			loop {
				let view = current.view.take().expect("FreeExplicit value already consumed");
				match view {
					FreeExplicitView::Pure(a) => return a,
					FreeExplicitView::Wrap(fa) => {
						let extracted: Box<FreeExplicit<'a, F, A>> = F::extract(fa);
						current = *extracted;
					}
				}
			}
		}

		/// Naive recursive bind. O(N) on left-associated chains because
		/// composing through a `Wrap` layer recurses through the spine via
		/// the closure passed to [`Functor::map`].
		///
		/// The closure is boxed once into an [`Rc`] so the recursive call in
		/// the internal helper does not generate a fresh closure type at each
		/// nesting level (which would hit the monomorphisation recursion
		/// limit).
		#[document_signature]
		///
		#[document_type_parameters("The result type of the new computation.")]
		///
		#[document_parameters("The function to apply to the result of this computation.")]
		///
		#[document_returns("A new `FreeExplicit` computation chaining `f` after this one.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::*,
		/// };
		///
		/// let free = FreeExplicit::<IdentityBrand, _>::pure(2)
		/// 	.bind(|x| FreeExplicit::pure(x + 1))
		/// 	.bind(|x| FreeExplicit::pure(x * 10));
		/// assert_eq!(free.evaluate(), 30);
		/// ```
		pub fn bind<B: 'a>(
			self,
			f: impl Fn(A) -> FreeExplicit<'a, F, B> + 'a,
		) -> FreeExplicit<'a, F, B> {
			let boxed: Rc<dyn Fn(A) -> FreeExplicit<'a, F, B> + 'a> = Rc::new(f);
			self.bind_boxed(boxed)
		}

		/// Internal recursive worker for [`bind`](FreeExplicit::bind).
		///
		/// Takes the continuation pre-boxed into an [`Rc`] so the recursive
		/// call inside the [`Functor::map`] closure does not generate a fresh
		/// closure type per layer (which would hit monomorphisation limits).
		#[document_signature]
		///
		#[document_type_parameters("The result type of the new computation.")]
		///
		#[document_parameters("The boxed continuation to apply.")]
		///
		#[document_returns("A new `FreeExplicit` computation chaining the continuation.")]
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
		/// let free = FreeExplicit::<IdentityBrand, _>::pure(2).bind(|x| FreeExplicit::pure(x + 1));
		/// assert_eq!(free.evaluate(), 3);
		/// ```
		#[expect(
			clippy::expect_used,
			reason = "FreeExplicit values consumed exactly once; double consumption is a bug"
		)]
		fn bind_boxed<B: 'a>(
			mut self,
			f: Rc<dyn Fn(A) -> FreeExplicit<'a, F, B> + 'a>,
		) -> FreeExplicit<'a, F, B> {
			let view = self.view.take().expect("FreeExplicit value already consumed");
			match view {
				FreeExplicitView::Pure(a) => f(a),
				FreeExplicitView::Wrap(fa) => {
					let f_outer = Rc::clone(&f);
					FreeExplicit {
						view: Some(FreeExplicitView::Wrap(F::map(
							move |inner: Box<FreeExplicit<'a, F, A>>|
							-> Box<FreeExplicit<'a, F, B>> {
								let f_inner = Rc::clone(&f_outer);
								Box::new((*inner).bind_boxed(f_inner))
							},
							fa,
						))),
					}
				}
			}
		}
	}

	#[document_type_parameters(
		"The lifetime that bounds the payload and the functor.",
		"The base functor.",
		"The result type."
	)]
	#[document_parameters("The `FreeExplicit` instance being dropped.")]
	impl<'a, F, A: 'a> Drop for FreeExplicit<'a, F, A>
	where
		F: WrapDrop + 'a,
	{
		/// Iteratively dismantles a deep `Wrap` chain via [`WrapDrop::drop`].
		///
		/// Naive recursive `Drop` would overflow the stack for chains beyond
		/// a few thousand layers. This impl walks the chain in a loop: at
		/// each `Wrap`, it consults `WrapDrop::drop` on the base functor to
		/// decide how to dismantle the layer. `Some(extracted)` lets the
		/// loop take the extracted node's view (leaving `None`) and
		/// continue (the just-emptied `Box<FreeExplicit>` drops with
		/// `view: None`, so its own `Drop` becomes a no-op and never
		/// recurses). `None` lets the layer drop in place, which is sound
		/// for brands that do not materially store the inner
		/// `Box<FreeExplicit>`.
		#[document_signature]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::*,
		/// };
		/// {
		/// 	let _free = FreeExplicit::<IdentityBrand, _>::pure(42);
		/// } // drop called here
		/// assert!(true);
		/// ```
		fn drop(&mut self) {
			let mut current_view = self.view.take();
			while let Some(view) = current_view {
				match view {
					FreeExplicitView::Pure(_) => {
						current_view = None;
					}
					FreeExplicitView::Wrap(fa) => {
						if let Some(mut extracted) =
							<F as WrapDrop>::drop::<Box<FreeExplicit<'a, F, A>>>(fa)
						{
							current_view = extracted.view.take();
						} else {
							current_view = None;
						}
					}
				}
			}
		}
	}

	// -- Brand-level type class instances --
	//
	// FreeExplicitBrand implements Functor, Pointed, and Semimonad. It does
	// NOT implement Lift, Semiapplicative, Applicative, ApplyFirst,
	// ApplySecond, or Monad because their natural implementation pattern
	// (`lift2 = bind(fa, |a| map(fb, |b| f(a, b)))`) requires `fb` to be
	// usable across multiple invocations of the bind closure. `FreeExplicit`
	// is not `Clone` (no outer `Rc`/`Arc` wrapper, and the embedded
	// `F::Of<'a, ...>` field's `Clone`-ness is not expressible per-`A` in
	// trait method signatures without HRTB-over-types). The remaining
	// applicative-family operations are reachable through:
	//
	// 1. Inherent methods on `FreeExplicit` (`bind`, `wrap`, `evaluate`).
	// 2. The Ref hierarchy (`RefFunctor` / `RefPointed` / `RefSemimonad`)
	//    impls below, which take `&self` and so don't have the consume-
	//    multiple-times issue.
	//
	// This matches the `RcCoyoneda`/`ArcCoyoneda` precedent: brand-level
	// coverage is whatever the trait signatures admit; the rest is
	// inherent-only.

	#[document_type_parameters("The base functor.")]
	impl<F: WrapDrop + Functor + 'static> Pointed for FreeExplicitBrand<F> {
		/// Wraps a value in a pure `FreeExplicit` computation.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime that bounds the payload and the functor.",
			"The type of the value to wrap."
		)]
		///
		#[document_parameters("The value to wrap.")]
		///
		#[document_returns("A `FreeExplicit` computation that produces `a`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::*,
		/// 	types::*,
		/// };
		///
		/// let free: FreeExplicit<'_, IdentityBrand, _> = FreeExplicitBrand::<IdentityBrand>::pure(42);
		/// assert_eq!(free.evaluate(), 42);
		/// ```
		fn pure<'a, A: 'a>(a: A) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>) {
			FreeExplicit::pure(a)
		}
	}

	#[document_type_parameters("The base functor.")]
	impl<F: WrapDrop + Functor + 'static> Functor for FreeExplicitBrand<F> {
		/// Maps a function over the result of a `FreeExplicit` computation
		/// by composing it with [`pure`](FreeExplicit::pure) under
		/// [`bind`](FreeExplicit::bind).
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime that bounds the payload and the functor.",
			"The original result type.",
			"The new result type."
		)]
		///
		#[document_parameters(
			"The function to apply to the result.",
			"The `FreeExplicit` computation."
		)]
		///
		#[document_returns("A new `FreeExplicit` with the function applied to its result.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::*,
		/// 	types::*,
		/// };
		///
		/// let free = FreeExplicit::<IdentityBrand, _>::pure(10);
		/// let mapped = FreeExplicitBrand::<IdentityBrand>::map(|x: i32| x * 2, free);
		/// assert_eq!(mapped.evaluate(), 20);
		/// ```
		fn map<'a, A: 'a, B: 'a>(
			f: impl Fn(A) -> B + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			fa.bind(move |a| FreeExplicit::pure(f(a)))
		}
	}

	#[document_type_parameters("The base functor.")]
	impl<F: WrapDrop + Functor + 'static> Semimonad for FreeExplicitBrand<F> {
		/// Sequences `FreeExplicit` computations.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime that bounds the payload and the functor.",
			"The type of the result of the first computation.",
			"The type of the result of the second computation."
		)]
		///
		#[document_parameters(
			"The first `FreeExplicit` computation.",
			"The function to chain after the first computation."
		)]
		///
		#[document_returns("A new `FreeExplicit` chaining the function after `ma`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::*,
		/// 	types::*,
		/// };
		///
		/// let free = FreeExplicit::<IdentityBrand, _>::pure(2);
		/// let chained =
		/// 	FreeExplicitBrand::<IdentityBrand>::bind(free, |x: i32| FreeExplicit::pure(x + 1));
		/// assert_eq!(chained.evaluate(), 3);
		/// ```
		fn bind<'a, A: 'a, B: 'a>(
			ma: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			func: impl Fn(A) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) + 'a,
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			ma.bind(func)
		}
	}

	// -- Ref hierarchy --
	//
	// The Ref hierarchy takes `&self` rather than `self`, so the
	// consume-or-clone issues that block `Lift` / `Semiapplicative` for
	// the by-value side don't apply here. Recursive helpers use
	// `Rc<dyn Fn>` boxing to avoid monomorphisation blow-up at each
	// spine layer (analogous to `bind_boxed`).

	/// Internal recursive worker for [`RefFunctor::ref_map`] over
	/// [`FreeExplicit`]. Takes the user closure pre-boxed into an [`Rc`]
	/// so the recursive call inside `F::ref_map`'s closure does not
	/// generate a fresh closure type per layer.
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	classes::*,
	/// 	types::*,
	/// };
	///
	/// let free = FreeExplicit::<IdentityBrand, _>::pure(10);
	/// let mapped = FreeExplicitBrand::<IdentityBrand>::ref_map(|x: &i32| *x + 1, &free);
	/// assert_eq!(mapped.evaluate(), 11);
	/// ```
	#[expect(
		clippy::expect_used,
		clippy::borrowed_box,
		reason = "FreeExplicit values consumed exactly once; double consumption is a bug. The `&Box<FreeExplicit>` shape is dictated by the F::Of<'a, Box<FreeExplicit>>: F::ref_map dispatch."
	)]
	fn free_explicit_ref_map<'a, F, A: 'a, B: 'a>(
		func: Rc<dyn Fn(&A) -> B + 'a>,
		fa: &FreeExplicit<'a, F, A>,
	) -> FreeExplicit<'a, F, B>
	where
		F: WrapDrop + Functor + RefFunctor + 'a, {
		let view = fa.view.as_ref().expect("FreeExplicit value already consumed");
		match view {
			FreeExplicitView::Pure(a) => FreeExplicit::pure(func(a)),
			FreeExplicitView::Wrap(fa_inner) => {
				let func_outer = Rc::clone(&func);
				FreeExplicit::wrap(F::ref_map(
					move |inner: &Box<FreeExplicit<'a, F, A>>| -> Box<FreeExplicit<'a, F, B>> {
						let func_inner = Rc::clone(&func_outer);
						Box::new(free_explicit_ref_map(func_inner, &**inner))
					},
					fa_inner,
				))
			}
		}
	}

	/// Internal recursive worker for [`RefSemimonad::ref_bind`] over
	/// [`FreeExplicit`]. Mirrors `free_explicit_ref_map`'s shape but the
	/// `Pure` arm uses the produced `FreeExplicit<F, B>` directly
	/// (rather than wrapping `f(a)` in `pure`).
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	classes::*,
	/// 	types::*,
	/// };
	///
	/// let free = FreeExplicit::<IdentityBrand, _>::pure(2);
	/// let chained =
	/// 	FreeExplicitBrand::<IdentityBrand>::ref_bind(&free, |x: &i32| FreeExplicit::pure(x + 1));
	/// assert_eq!(chained.evaluate(), 3);
	/// ```
	#[expect(
		clippy::expect_used,
		clippy::borrowed_box,
		clippy::type_complexity,
		reason = "FreeExplicit values consumed exactly once; double consumption is a bug. The `&Box<FreeExplicit>` shape is dictated by the F::Of<'a, Box<FreeExplicit>>: F::ref_map dispatch. Boxed closure type complexity is dictated by the recursive helper signature."
	)]
	fn free_explicit_ref_bind<'a, F, A: 'a, B: 'a>(
		f: Rc<dyn Fn(&A) -> FreeExplicit<'a, F, B> + 'a>,
		fa: &FreeExplicit<'a, F, A>,
	) -> FreeExplicit<'a, F, B>
	where
		F: WrapDrop + Functor + RefFunctor + 'a, {
		let view = fa.view.as_ref().expect("FreeExplicit value already consumed");
		match view {
			FreeExplicitView::Pure(a) => f(a),
			FreeExplicitView::Wrap(fa_inner) => {
				let f_outer = Rc::clone(&f);
				FreeExplicit::wrap(F::ref_map(
					move |inner: &Box<FreeExplicit<'a, F, A>>| -> Box<FreeExplicit<'a, F, B>> {
						let f_inner = Rc::clone(&f_outer);
						Box::new(free_explicit_ref_bind(f_inner, &**inner))
					},
					fa_inner,
				))
			}
		}
	}

	#[document_type_parameters("The base functor.")]
	impl<F: WrapDrop + Functor + RefFunctor + 'static> RefFunctor for FreeExplicitBrand<F> {
		/// Maps a function over the result of a `FreeExplicit` computation
		/// using a reference to the value, walking the structure
		/// recursively via `F::ref_map`.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime that bounds the payload and the functor.",
			"The original result type.",
			"The new result type."
		)]
		///
		#[document_parameters(
			"The function to apply to the result by reference.",
			"The `FreeExplicit` computation."
		)]
		///
		#[document_returns("A new `FreeExplicit` with the function applied to its result.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::*,
		/// 	types::*,
		/// };
		///
		/// let free = FreeExplicit::<IdentityBrand, _>::pure(10);
		/// let mapped = FreeExplicitBrand::<IdentityBrand>::ref_map(|x: &i32| *x * 2, &free);
		/// assert_eq!(mapped.evaluate(), 20);
		/// ```
		fn ref_map<'a, A: 'a, B: 'a>(
			func: impl Fn(&A) -> B + 'a,
			fa: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			let func_rc: Rc<dyn Fn(&A) -> B + 'a> = Rc::new(func);
			free_explicit_ref_map(func_rc, fa)
		}
	}

	#[document_type_parameters("The base functor.")]
	impl<F: WrapDrop + Functor + 'static> RefPointed for FreeExplicitBrand<F> {
		/// Wraps a cloned value in a pure `FreeExplicit` computation.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime that bounds the payload and the functor.",
			"The type of the value to wrap. Must be `Clone`."
		)]
		///
		#[document_parameters("A reference to the value to wrap.")]
		///
		#[document_returns("A `FreeExplicit` computation that produces a clone of `a`.")]
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
		/// let free: FreeExplicit<'_, IdentityBrand, _> =
		/// 	FreeExplicitBrand::<IdentityBrand>::ref_pure(&value);
		/// assert_eq!(free.evaluate(), 42);
		/// ```
		fn ref_pure<'a, A: Clone + 'a>(
			a: &A
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>) {
			FreeExplicit::pure(a.clone())
		}
	}

	#[document_type_parameters("The base functor.")]
	impl<F: WrapDrop + Functor + RefFunctor + 'static> RefSemimonad for FreeExplicitBrand<F> {
		/// Sequences `FreeExplicit` computations using a reference to the
		/// intermediate value, walking the structure recursively via
		/// `F::ref_map`.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime that bounds the payload and the functor.",
			"The type of the result of the first computation.",
			"The type of the result of the second computation."
		)]
		///
		#[document_parameters(
			"The first `FreeExplicit` computation.",
			"The function to chain after the first computation."
		)]
		///
		#[document_returns("A new `FreeExplicit` chaining the function after `ma`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::*,
		/// 	types::*,
		/// };
		///
		/// let free = FreeExplicit::<IdentityBrand, _>::pure(2);
		/// let chained =
		/// 	FreeExplicitBrand::<IdentityBrand>::ref_bind(&free, |x: &i32| FreeExplicit::pure(x + 1));
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
			let f_rc: Rc<dyn Fn(&A) -> FreeExplicit<'a, F, B> + 'a> = Rc::new(f);
			free_explicit_ref_bind(f_rc, ma)
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
				FreeExplicitBrand,
				IdentityBrand,
			},
			classes::{
				RefFunctor,
				RefPointed,
				RefSemimonad,
			},
			types::Identity,
		},
	};

	#[test]
	fn pure_evaluate() {
		let free = FreeExplicit::<IdentityBrand, _>::pure(42);
		assert_eq!(free.evaluate(), 42);
	}

	#[test]
	fn wrap_evaluate() {
		let inner = FreeExplicit::<IdentityBrand, _>::pure(7);
		let free: FreeExplicit<'_, IdentityBrand, _> =
			FreeExplicit::wrap(Identity(Box::new(inner)));
		assert_eq!(free.evaluate(), 7);
	}

	#[test]
	fn bind_chains() {
		let free = FreeExplicit::<IdentityBrand, _>::pure(1)
			.bind(|x: i32| FreeExplicit::pure(x + 1))
			.bind(|x: i32| FreeExplicit::pure(x * 10));
		assert_eq!(free.evaluate(), 20);
	}

	#[test]
	fn deep_evaluate_does_not_overflow() {
		const DEPTH: usize = 100_000;
		let mut free: FreeExplicit<'_, IdentityBrand, i32> = FreeExplicit::pure(0);
		for _ in 0 .. DEPTH {
			free = FreeExplicit::wrap(Identity(Box::new(free)));
		}
		assert_eq!(free.evaluate(), 0);
	}

	#[test]
	fn deep_drop_does_not_overflow() {
		const DEPTH: usize = 100_000;
		let mut free: FreeExplicit<'_, IdentityBrand, i32> = FreeExplicit::pure(0);
		for _ in 0 .. DEPTH {
			free = FreeExplicit::wrap(Identity(Box::new(free)));
		}
		drop(free);
	}

	#[test]
	fn non_static_payload() {
		// Defining property of the Explicit family: payloads can borrow
		// non-`'static` data.
		let s = String::from("hello");
		let r: &str = &s;
		let free: FreeExplicit<'_, IdentityBrand, &str> = FreeExplicit::pure(r);
		assert_eq!(free.evaluate(), "hello");
	}

	#[test]
	fn brand_dispatched_pointed() {
		// Brand-dispatched `pure` via `RefPointed`.
		let value = 7;
		let free: FreeExplicit<'_, IdentityBrand, _> =
			FreeExplicitBrand::<IdentityBrand>::ref_pure(&value);
		assert_eq!(free.evaluate(), 7);
	}

	#[test]
	fn brand_dispatched_functor() {
		// Brand-dispatched `ref_map` via `RefFunctor`.
		let free = FreeExplicit::<IdentityBrand, _>::pure(10);
		let mapped = FreeExplicitBrand::<IdentityBrand>::ref_map(|x: &i32| *x * 2, &free);
		assert_eq!(mapped.evaluate(), 20);
	}

	#[test]
	fn brand_dispatched_semimonad() {
		// Brand-dispatched `ref_bind` via `RefSemimonad`.
		let free = FreeExplicit::<IdentityBrand, _>::pure(2);
		let chained = FreeExplicitBrand::<IdentityBrand>::ref_bind(&free, |x: &i32| {
			FreeExplicit::pure(x + 1)
		});
		assert_eq!(chained.evaluate(), 3);
	}
}
