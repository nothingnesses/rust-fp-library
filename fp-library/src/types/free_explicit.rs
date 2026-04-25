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
//! `FreeExplicit<'a, F, A>` requires `F: Extract + Functor + 'a`. The
//! [`Extract`](crate::classes::Extract) bound is needed by the custom
//! iterative [`Drop`] impl so deep `Wrap` chains can be dismantled without
//! overflowing the stack. Functors whose payload is a continuation function
//! rather than an extractable value cannot be used directly with
//! `FreeExplicit`; route through
//! [`Free::fold_free`](crate::types::Free::fold_free) into a
//! [`MonadRec`](crate::classes::MonadRec) target instead.

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			Apply,
			brands::FreeExplicitBrand,
			classes::{
				Extract,
				Functor,
			},
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
		F: Extract + Functor + 'a, {
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
	/// [`Extract::extract`], mirroring the strategy used by
	/// [`Free::drop`](crate::types::Free). Without this, a 100 000-deep
	/// `Wrap` chain stack-overflows during cleanup.
	#[document_type_parameters(
		"The lifetime that bounds the payload and the functor.",
		"The base functor (must implement [`Extract`] and [`Functor`]).",
		"The result type."
	)]
	pub struct FreeExplicit<'a, F, A: 'a>
	where
		F: Extract + Functor + 'a, {
		view: Option<FreeExplicitView<'a, F, A>>,
	}

	impl_kind! {
		impl<F: Extract + Functor + 'static> for FreeExplicitBrand<F> {
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
		F: Extract + Functor + 'a,
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
		pub fn evaluate(self) -> A {
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
		F: Extract + Functor + 'a,
	{
		/// Iteratively dismantles a deep `Wrap` chain via [`Extract::extract`].
		///
		/// Naive recursive `Drop` would overflow the stack for chains beyond
		/// a few thousand layers. This impl walks the chain in a loop: at
		/// each `Wrap`, it extracts the inner `Box<FreeExplicit>`, takes its
		/// view (leaving `None`), and continues. The just-emptied
		/// `Box<FreeExplicit>` then drops with `view: None`, so its own
		/// `Drop` becomes a no-op and never recurses.
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
						let mut extracted: Box<FreeExplicit<'a, F, A>> = F::extract(fa);
						current_view = extracted.view.take();
					}
				}
			}
		}
	}
}

pub use inner::*;
