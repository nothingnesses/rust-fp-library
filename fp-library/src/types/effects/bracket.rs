//! Scoped resource-management effect type with `Bracket`
//! (acquire a resource, run a body that uses it, release the
//! resource) as its sole operation. The corresponding brands are
//! [`BoxBracketBrand`](crate::brands::BoxBracketBrand),
//! [`BracketBrand`](crate::brands::BracketBrand), and
//! [`SendBracketBrand`](crate::brands::SendBracketBrand).
//!
//! Mirrors PureScript Run's `Aff.bracket` and the Val flavour of
//! Heftia's higher-order resource-management operation. The cell
//! holds three closures with three differently-typed program
//! returns over a single substrate brand `Sub`: `acquire` returns
//! `Free<Sub, A>` (resource value type), `body` returns
//! `Free<Sub, (A, B)>` (paired resource and body result), `release`
//! returns `Free<Sub, ()>` (unit). The substrate brand `Sub` is
//! carried explicitly through the cell's struct because Rust's
//! well-formedness check on `<Self as Kind>::Of<'a, X>` rejects
//! extracting Sub from X via a substrate-side trait projection
//! (the variant attempted as Option C in the closed B17 entry).
//!
//! ## Three sibling types
//!
//! `Bracket` ships in three sibling flavours that thread different
//! per-pointer-brand closure-storage shapes for all three fields.
//! Acquire is a unit-arg B-thunk (single materialisation at
//! dispatch time, breaking the Free layout cycle); body and release
//! are 1-arg closures taking the resource as `<P>::Of<'a, A>`. The
//! per-pointer-brand pointer lets the existing pointer-abstraction
//! [`ToDynFnOnce`](crate::classes::ToDynFnOnce) /
//! [`ToDynCloneFn`](crate::classes::ToDynCloneFn) /
//! [`ToDynSendFn`](crate::classes::ToDynSendFn) family construct
//! the closures uniformly.
//!
//! - [`BoxBracket<'a, P, Sub, A, B>`] for default `Run` /
//!   `RunExplicit`, `P: ToDynFnOnce` (`BoxBrand` only); each field
//!   is `Box<dyn FnOnce(...)>` (single-shot).
//! - [`Bracket<'a, P, Sub, A, B>`] for `RcRun` / `RcRunExplicit`,
//!   `P: ToDynCloneFn` (typically `RcBrand`); each field is
//!   `Rc<dyn Fn(...)>` (clone-able via Rc-bump, multi-shot).
//! - [`SendBracket<'a, P, Sub, A, B>`] for `ArcRun` /
//!   `ArcRunExplicit`, `P: ToDynSendFn` (typically `ArcBrand`);
//!   each field is `Arc<dyn Fn(...) + Send + Sync>` (clone-able via
//!   Arc-bump, thread-safe, multi-shot).
//!
//! ## Why acquire is a thunk
//!
//! Storing `acquire` directly as `Free<Sub, A>` would create a
//! substrate-level layout cycle when `Sub` resolves to
//! `NodeBrand<R, S>` and `S` contains the `Bracket` brand:
//! `Free`'s view holds a `Node`, the `Node::Scoped` arm holds a
//! `Coproduct` of scoped-effect cells, and the `Bracket` cell would
//! carry an unboxed `acquire: Free<NodeBrand<R, S>, A>` field,
//! creating an infinite layout. The thunk indirection (Box / Rc /
//! Arc are pointer-sized regardless of the closure target's layout)
//! breaks the cycle; the unit-arg form fits the existing
//! pointer-abstraction matrix without a new construction path.
//! Body and release fields don't need this treatment because their
//! `Free<Sub, _>` returns live inside their closures (materialised
//! at call time when the dispatcher passes the resource, not stored
//! as direct fields). Mirrors the same fix applied to
//! [`Catch`](crate::types::effects::catch) (B7) and
//! [`Local`](crate::types::effects::local) (B9).
//!
//! ## Why the substrate brand is explicit
//!
//! The cell's three fields return programs of three different
//! result types over the same row-functor brand. The substrate's
//! GAT projection `Brand::Of<'a, X>` fills X with the body's
//! program type (`Free<Sub, (A, B)>` for Val); the cell needs
//! access to `Sub` to spell acquire's `Free<Sub, A>` and release's
//! `Free<Sub, ()>` types. Carrying `Sub` as an explicit cell
//! parameter avoids a substrate-side trait projection
//! `<X as ProjectSub>::F` whose well-formedness check would reject
//! `Of<'a, X: 'a>` for unbounded X. The recursive type cycle
//! (`Sub` references the row that contains the `Bracket` brand) is
//! broken at the value level by `PhantomData<Sub>` on the brand:
//! the brand is zero-sized regardless of `Sub`'s layout.

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			Apply,
			brands::{
				ArcBrand,
				BoxBracketBrand,
				BoxBracketExplicitBrand,
				BoxBrand,
				BracketBrand,
				BracketExplicitBrand,
				RcBrand,
				SendBracketBrand,
				SendBracketExplicitBrand,
			},
			classes::{
				Extract,
				Functor,
				Pointer,
				RefCountedPointer,
				RefFunctor,
				SendFunctor,
				SendRefCountedPointer,
				ToDynCloneFn,
				ToDynFnOnce,
				ToDynSendFn,
				WrapDrop,
			},
			impl_kind,
			kinds::*,
			types::{
				ArcFree,
				ArcFreeExplicit,
				ArcTypeErasedValue,
				Free,
				FreeExplicit,
				RcFree,
				RcFreeExplicit,
			},
		},
		fp_macros::*,
	};

	// ===== BoxBracket (BoxBrand + FnOnce, single-shot) =====

	/// Scoped resource-management effect for default `Run` /
	/// `RunExplicit` substrates. All three closures are stored as
	/// `<P as Pointer>::Of<'a, dyn 'a + FnOnce(...) -> _>` projections
	/// (`Box<dyn FnOnce>`): acquire as a unit-arg B-thunk
	/// `Box<dyn FnOnce(()) -> Free<Sub, A>>` (single-shot,
	/// materialised once at dispatch time, breaking the substrate
	/// layout cycle); body as
	/// `Box<dyn FnOnce(<P>::Of<'a, A>) -> Free<Sub, (A, B)>>`; release
	/// as `Box<dyn FnOnce(<P>::Of<'a, A>) -> Free<Sub, ()>>`.
	#[document_type_parameters(
		"The lifetime of the acquire / body / release closures.",
		"The pointer brand storing the closures (BoxBrand only by structural bound).",
		"The substrate brand (e.g., NodeBrand<R, S>) over which acquire / body / release return Free programs.",
		"The resource type produced by acquire and consumed by body / release.",
		"The body's result type."
	)]
	pub enum BoxBracket<'a, P, Sub, A, B>
	where
		P: ToDynFnOnce,
		Sub: WrapDrop + 'static,
		A: 'static,
		B: 'static, {
		/// Acquire a resource by running `acquire`, run `body` with
		/// the resource to produce a paired body result, then run
		/// `release` with the resource to clean up. The dispatcher
		/// orchestrates the three steps (step 6).
		Bracket {
			/// The acquire program, stored as a unit-arg B-thunk
			/// (`<P as Pointer>::Of<'a, dyn 'a + FnOnce(()) -> Free<Sub, A>>`,
			/// single-shot via [`FnOnce`]). The unit-arg form lets the
			/// pointer-abstraction's [`ToDynFnOnce::new`](crate::classes::ToDynFnOnce)
			/// family construct it; the dispatcher invokes it as
			/// `acquire(())` to materialise the resource program.
			acquire: <P as Pointer>::Of<'a, dyn 'a + FnOnce(()) -> Free<Sub, A>>,
			/// The body closure
			/// (`<P as Pointer>::Of<'a, dyn 'a + FnOnce(<P>::Of<'a, A>) -> Free<Sub, (A, B)>>`,
			/// single-shot via [`FnOnce`]). The dispatcher passes the
			/// acquired resource as `<P>::Of<'a, A>` and runs the
			/// returned program; the program's result pairs the resource
			/// with the body's result so the caller may see both.
			#[expect(
				clippy::type_complexity,
				reason = "Bracket cells store closures returning Free programs derived from the substrate brand Sub; the nested GAT and Free projections cannot be aliased without losing the per-pointer-brand structure ToDynFnOnce / ToDynCloneFn / ToDynSendFn dispatch over."
			)]
			body: <P as Pointer>::Of<
				'a,
				dyn 'a + FnOnce(<P as Pointer>::Of<'a, A>) -> Free<Sub, (A, B)>,
			>,
			/// The release closure
			/// (`<P as Pointer>::Of<'a, dyn 'a + FnOnce(<P>::Of<'a, A>) -> Free<Sub, ()>>`,
			/// single-shot via [`FnOnce`]). The dispatcher passes the
			/// acquired resource as `<P>::Of<'a, A>` and runs the
			/// returned program for its side effects; the result is unit.
			#[expect(
				clippy::type_complexity,
				reason = "Bracket cells store closures returning Free programs derived from the substrate brand Sub; the nested GAT and Free projections cannot be aliased without losing the per-pointer-brand structure ToDynFnOnce / ToDynCloneFn / ToDynSendFn dispatch over."
			)]
			release:
				<P as Pointer>::Of<'a, dyn 'a + FnOnce(<P as Pointer>::Of<'a, A>) -> Free<Sub, ()>>,
		},
	}

	impl_kind! {
		impl<P: ToDynFnOnce, Sub: WrapDrop + 'static, A: 'static, B: 'static> for BoxBracketBrand<P, Sub, A, B> {
			type Of<'a, X: 'a>: 'a = BoxBracket<'a, P, Sub, A, B>;
		}
	}

	#[document_type_parameters(
		"The substrate brand over which acquire / body / release return Free programs.",
		"The resource type.",
		"The body's result type."
	)]
	impl<Sub, A, B> Functor for BoxBracketBrand<BoxBrand, Sub, A, B>
	where
		Sub: WrapDrop + 'static,
		A: 'static,
		B: 'static,
	{
		/// Identity `map`. The cell's GAT projection `Of<'a, X>`
		/// erases X (the body's program type is fixed by the brand's
		/// `Sub`, `A`, `B` parameters), so post-composing `f` over
		/// the body would change the cell's brand identity. The
		/// substrate calls this when threading first-order
		/// continuations through the program tree; for scoped
		/// `Bracket` cells the relevant transformation is the
		/// dispatcher walking acquire / body / release in sequence,
		/// not a pointwise `f` post-composition.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the continuations.",
			"The original GAT-filled type.",
			"The new GAT-filled type."
		)]
		///
		#[document_parameters(
			"The function (ignored; the cell's structure is fixed by Sub / A / B).",
			"The bracket effect."
		)]
		///
		#[document_returns(
			"The bracket effect unchanged (the GAT projection erases the X parameter)."
		)]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		BoxBracketBrand,
		/// 		BoxBrand,
		/// 		ThunkBrand,
		/// 	},
		/// 	classes::{
		/// 		Functor,
		/// 		ToDynFnOnce,
		/// 	},
		/// 	types::{
		/// 		Free,
		/// 		effects::bracket::BoxBracket,
		/// 	},
		/// };
		///
		/// let bracket: BoxBracket<'static, BoxBrand, ThunkBrand, i32, i32> = BoxBracket::Bracket {
		/// 	acquire: <BoxBrand as ToDynFnOnce>::new(|_: ()| Free::<ThunkBrand, _>::pure(7)),
		/// 	body: <BoxBrand as ToDynFnOnce>::new(|_a: Box<i32>| Free::<ThunkBrand, _>::pure((7, 42))),
		/// 	release: <BoxBrand as ToDynFnOnce>::new(|_a: Box<i32>| Free::<ThunkBrand, _>::pure(())),
		/// };
		/// let mapped =
		/// 	<BoxBracketBrand<BoxBrand, ThunkBrand, i32, i32> as Functor>::map(|x: i32| x + 1, bracket);
		/// assert!(matches!(mapped, BoxBracket::Bracket { .. }));
		/// ```
		fn map<'a, X: 'a, Y: 'a>(
			_f: impl Fn(X) -> Y + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, X>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Y>) {
			fa
		}
	}

	// ===== Bracket (RcBrand + Fn, multi-shot clone-capable) =====

	/// Scoped resource-management effect for `RcRun` /
	/// `RcRunExplicit` substrates. All three closures are stored as
	/// `<P as RefCountedPointer>::Of<'a, dyn 'a + Fn(...) -> _>`
	/// projections (`Rc<dyn Fn>`): acquire as a unit-arg B-thunk
	/// (multi-shot, materialised on each call); body and release as
	/// resource-consuming closures. All three are clone-able
	/// (refcount-bump) along with the surrounding multi-shot program.
	#[document_type_parameters(
		"The lifetime of the acquire / body / release closures.",
		"The pointer brand storing the closures (RcBrand only).",
		"The substrate brand (e.g., NodeBrand<R, S>) over which acquire / body / release return Free programs.",
		"The resource type produced by acquire and consumed by body / release.",
		"The body's result type."
	)]
	pub enum Bracket<'a, P, Sub, A, B>
	where
		P: ToDynCloneFn,
		Sub: WrapDrop + 'static,
		A: 'static,
		B: 'static, {
		/// Acquire a resource by running `acquire`, run `body` with
		/// the resource to produce a paired body result, then run
		/// `release` with the resource to clean up. The dispatcher
		/// orchestrates the three steps (step 6).
		Bracket {
			/// The acquire program, stored as a unit-arg B-thunk
			/// (`<P as RefCountedPointer>::Of<'a, dyn 'a + Fn(()) -> RcFree<Sub, A>>`,
			/// multi-shot via [`Fn`]; clone via Rc-bump). The unit-arg
			/// form lets the pointer-abstraction's
			/// [`ToDynCloneFn::new`](crate::classes::ToDynCloneFn) family
			/// construct it; the dispatcher invokes it as `acquire(())`.
			acquire: <P as RefCountedPointer>::Of<'a, dyn 'a + Fn(()) -> RcFree<Sub, A>>,
			/// The body closure
			/// (`<P as RefCountedPointer>::Of<'a, dyn 'a + Fn(<P>::Of<'a, A>) -> RcFree<Sub, (A, B)>>`,
			/// multi-shot via [`Fn`]; clone via Rc-bump). The
			/// dispatcher passes the acquired resource as
			/// `<P>::Of<'a, A>`.
			#[expect(
				clippy::type_complexity,
				reason = "Bracket cells store closures returning RcFree programs derived from the substrate brand Sub; the nested GAT and RcFree projections cannot be aliased without losing the per-pointer-brand structure ToDynFnOnce / ToDynCloneFn / ToDynSendFn dispatch over."
			)]
			body: <P as RefCountedPointer>::Of<
				'a,
				dyn 'a + Fn(<P as RefCountedPointer>::Of<'a, A>) -> RcFree<Sub, (A, B)>,
			>,
			/// The release closure
			/// (`<P as RefCountedPointer>::Of<'a, dyn 'a + Fn(<P>::Of<'a, A>) -> RcFree<Sub, ()>>`,
			/// multi-shot via [`Fn`]; clone via Rc-bump).
			#[expect(
				clippy::type_complexity,
				reason = "Bracket cells store closures returning RcFree programs derived from the substrate brand Sub; the nested GAT and RcFree projections cannot be aliased without losing the per-pointer-brand structure ToDynFnOnce / ToDynCloneFn / ToDynSendFn dispatch over."
			)]
			release: <P as RefCountedPointer>::Of<
				'a,
				dyn 'a + Fn(<P as RefCountedPointer>::Of<'a, A>) -> RcFree<Sub, ()>,
			>,
		},
	}

	impl_kind! {
		impl<P: ToDynCloneFn, Sub: WrapDrop + 'static, A: 'static, B: 'static> for BracketBrand<P, Sub, A, B> {
			type Of<'a, X: 'a>: 'a = Bracket<'a, P, Sub, A, B>;
		}
	}

	#[document_type_parameters(
		"The lifetime of the closures.",
		"The pointer brand storing the closures.",
		"The substrate brand.",
		"The resource type.",
		"The body's result type."
	)]
	#[document_parameters("The bracket effect to clone.")]
	impl<'a, P, Sub, A, B> Clone for Bracket<'a, P, Sub, A, B>
	where
		P: ToDynCloneFn,
		Sub: WrapDrop + 'static,
		A: 'static,
		B: 'static,
	{
		/// Clones the bracket effect by refcount-bumping the stored
		/// acquire / body / release pointers. All three are
		/// `<P as RefCountedPointer>::Of<...>`, which is unconditionally
		/// [`Clone`] per the trait's associated-type bound.
		#[document_signature]
		///
		#[document_returns("A new bracket effect sharing acquire / body / release by refcount.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		RcBrand,
		/// 		ThunkBrand,
		/// 	},
		/// 	classes::ToDynCloneFn,
		/// 	types::{
		/// 		RcFree,
		/// 		effects::bracket::Bracket,
		/// 	},
		/// };
		///
		/// let original: Bracket<'static, RcBrand, ThunkBrand, i32, i32> = Bracket::Bracket {
		/// 	acquire: <RcBrand as ToDynCloneFn>::new(|_: ()| RcFree::<ThunkBrand, _>::pure(7)),
		/// 	body: <RcBrand as ToDynCloneFn>::new(|_a: std::rc::Rc<i32>| {
		/// 		RcFree::<ThunkBrand, _>::pure((7, 42))
		/// 	}),
		/// 	release: <RcBrand as ToDynCloneFn>::new(|_a: std::rc::Rc<i32>| {
		/// 		RcFree::<ThunkBrand, _>::pure(())
		/// 	}),
		/// };
		/// let cloned = original.clone();
		/// assert!(matches!(cloned, Bracket::Bracket { .. }));
		/// ```
		fn clone(&self) -> Self {
			match self {
				Bracket::Bracket {
					acquire,
					body,
					release,
				} => Bracket::Bracket {
					acquire: <P as RefCountedPointer>::Of::clone(acquire),
					body: <P as RefCountedPointer>::Of::clone(body),
					release: <P as RefCountedPointer>::Of::clone(release),
				},
			}
		}
	}

	#[document_type_parameters(
		"The substrate brand.",
		"The resource type.",
		"The body's result type."
	)]
	impl<Sub, A, B> Functor for BracketBrand<RcBrand, Sub, A, B>
	where
		Sub: WrapDrop + 'static,
		A: 'static,
		B: 'static,
	{
		/// Identity `map`. The cell's GAT projection `Of<'a, X>`
		/// erases X; mirroring [`BoxBracketBrand`'s `Functor`].
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the continuations.",
			"The original GAT-filled type.",
			"The new GAT-filled type."
		)]
		///
		#[document_parameters("The function (ignored).", "The bracket effect.")]
		///
		#[document_returns("The bracket effect unchanged.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		BracketBrand,
		/// 		RcBrand,
		/// 		ThunkBrand,
		/// 	},
		/// 	classes::{
		/// 		Functor,
		/// 		ToDynCloneFn,
		/// 	},
		/// 	types::{
		/// 		RcFree,
		/// 		effects::bracket::Bracket,
		/// 	},
		/// };
		///
		/// let bracket: Bracket<'static, RcBrand, ThunkBrand, i32, i32> = Bracket::Bracket {
		/// 	acquire: <RcBrand as ToDynCloneFn>::new(|_: ()| RcFree::<ThunkBrand, _>::pure(7)),
		/// 	body: <RcBrand as ToDynCloneFn>::new(|_a: std::rc::Rc<i32>| {
		/// 		RcFree::<ThunkBrand, _>::pure((7, 42))
		/// 	}),
		/// 	release: <RcBrand as ToDynCloneFn>::new(|_a: std::rc::Rc<i32>| {
		/// 		RcFree::<ThunkBrand, _>::pure(())
		/// 	}),
		/// };
		/// let mapped =
		/// 	<BracketBrand<RcBrand, ThunkBrand, i32, i32> as Functor>::map(|x: i32| x + 1, bracket);
		/// assert!(matches!(mapped, Bracket::Bracket { .. }));
		/// ```
		fn map<'a, X: 'a, Y: 'a>(
			_f: impl Fn(X) -> Y + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, X>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Y>) {
			fa
		}
	}

	// ===== SendBracket (ArcBrand + Fn + Send + Sync, thread-safe) =====

	/// Scoped resource-management effect for `ArcRun` /
	/// `ArcRunExplicit` substrates. All three closures are stored as
	/// `<P as SendRefCountedPointer>::Of<'a, dyn 'a + Fn(...) -> _ + Send + Sync>`
	/// projections (`Arc<dyn Fn + Send + Sync>`): acquire as a
	/// unit-arg B-thunk (multi-shot, thread-safe); body and release
	/// as resource-consuming closures. All three are clone-able
	/// (refcount-bump) and thread-safe.
	#[document_type_parameters(
		"The lifetime of the acquire / body / release closures.",
		"The pointer brand storing the closures (ArcBrand only).",
		"The substrate brand (e.g., NodeBrand<R, S>) over which acquire / body / release return Free programs.",
		"The resource type produced by acquire and consumed by body / release.",
		"The body's result type."
	)]
	pub enum SendBracket<'a, P, Sub, A, B>
	where
		P: ToDynSendFn,
		Sub: WrapDrop
			+ Kind_cdc7cd43dac7585f<Of<'static, ArcFree<Sub, ArcTypeErasedValue>>: Send + Sync>
			+ 'static,
		A: Send + Sync + 'static,
		B: Send + Sync + 'static, {
		/// Acquire a resource by running `acquire`, run `body` with
		/// the resource to produce a paired body result, then run
		/// `release` with the resource to clean up. The dispatcher
		/// orchestrates the three steps (step 6).
		Bracket {
			/// The acquire program, stored as a unit-arg B-thunk
			/// (`<P as SendRefCountedPointer>::Of<'a, dyn 'a + Fn(()) -> ArcFree<Sub, A> + Send + Sync>`,
			/// multi-shot via [`Fn`]; clone via Arc-bump; thread-safe).
			acquire: <P as SendRefCountedPointer>::Of<
				'a,
				dyn 'a + Fn(()) -> ArcFree<Sub, A> + Send + Sync,
			>,
			/// The body closure
			/// (`<P as SendRefCountedPointer>::Of<'a, dyn 'a + Fn(<P>::Of<'a, A>) -> ArcFree<Sub, (A, B)> + Send + Sync>`,
			/// multi-shot via [`Fn`]; clone via Arc-bump; thread-safe).
			#[expect(
				clippy::type_complexity,
				reason = "Bracket cells store closures returning ArcFree programs derived from the substrate brand Sub; the nested GAT and ArcFree projections cannot be aliased without losing the per-pointer-brand structure ToDynFnOnce / ToDynCloneFn / ToDynSendFn dispatch over."
			)]
			body: <P as SendRefCountedPointer>::Of<
				'a,
				dyn 'a
					+ Fn(<P as SendRefCountedPointer>::Of<'a, A>) -> ArcFree<Sub, (A, B)>
					+ Send
					+ Sync,
			>,
			/// The release closure
			/// (`<P as SendRefCountedPointer>::Of<'a, dyn 'a + Fn(<P>::Of<'a, A>) -> ArcFree<Sub, ()> + Send + Sync>`,
			/// multi-shot via [`Fn`]; clone via Arc-bump; thread-safe).
			#[expect(
				clippy::type_complexity,
				reason = "Bracket cells store closures returning ArcFree programs derived from the substrate brand Sub; the nested GAT and ArcFree projections cannot be aliased without losing the per-pointer-brand structure ToDynFnOnce / ToDynCloneFn / ToDynSendFn dispatch over."
			)]
			release: <P as SendRefCountedPointer>::Of<
				'a,
				dyn 'a
					+ Fn(<P as SendRefCountedPointer>::Of<'a, A>) -> ArcFree<Sub, ()>
					+ Send
					+ Sync,
			>,
		},
	}

	impl_kind! {
		impl<P: ToDynSendFn, Sub: WrapDrop + Kind_cdc7cd43dac7585f<Of<'static, ArcFree<Sub, ArcTypeErasedValue>>: Send + Sync> + 'static, A: Send + Sync + 'static, B: Send + Sync + 'static> for SendBracketBrand<P, Sub, A, B> {
			type Of<'a, X: 'a>: 'a = SendBracket<'a, P, Sub, A, B>;
		}
	}

	#[document_type_parameters(
		"The lifetime of the closures.",
		"The pointer brand storing the closures.",
		"The substrate brand.",
		"The resource type.",
		"The body's result type."
	)]
	#[document_parameters("The send-bracket effect to clone.")]
	impl<'a, P, Sub, A, B> Clone for SendBracket<'a, P, Sub, A, B>
	where
		P: ToDynSendFn,
		Sub: WrapDrop
			+ Kind_cdc7cd43dac7585f<Of<'static, ArcFree<Sub, ArcTypeErasedValue>>: Send + Sync>
			+ 'static,
		A: Send + Sync + 'static,
		B: Send + Sync + 'static,
	{
		/// Clones the send-bracket effect by refcount-bumping the
		/// stored acquire / body / release pointers. All three are
		/// `<P as SendRefCountedPointer>::Of<...>`, which is
		/// unconditionally [`Clone`] per the trait's associated-type
		/// bound.
		#[document_signature]
		///
		#[document_returns(
			"A new send-bracket effect sharing acquire / body / release by refcount."
		)]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		ArcBrand,
		/// 		IdentityBrand,
		/// 	},
		/// 	classes::ToDynSendFn,
		/// 	types::{
		/// 		ArcFree,
		/// 		effects::bracket::SendBracket,
		/// 	},
		/// };
		///
		/// let original: SendBracket<'static, ArcBrand, IdentityBrand, i32, i32> = SendBracket::Bracket {
		/// 	acquire: <ArcBrand as ToDynSendFn>::new(|_: ()| ArcFree::<IdentityBrand, _>::pure(7)),
		/// 	body: <ArcBrand as ToDynSendFn>::new(|_a: std::sync::Arc<i32>| {
		/// 		ArcFree::<IdentityBrand, _>::pure((7, 42))
		/// 	}),
		/// 	release: <ArcBrand as ToDynSendFn>::new(|_a: std::sync::Arc<i32>| {
		/// 		ArcFree::<IdentityBrand, _>::pure(())
		/// 	}),
		/// };
		/// let cloned = original.clone();
		/// assert!(matches!(cloned, SendBracket::Bracket { .. }));
		/// ```
		fn clone(&self) -> Self {
			match self {
				SendBracket::Bracket {
					acquire,
					body,
					release,
				} => SendBracket::Bracket {
					acquire: <P as SendRefCountedPointer>::Of::clone(acquire),
					body: <P as SendRefCountedPointer>::Of::clone(body),
					release: <P as SendRefCountedPointer>::Of::clone(release),
				},
			}
		}
	}

	// ===== SendFunctor for SendBracketBrand (load-bearing on Arc family) =====

	#[document_type_parameters(
		"The substrate brand.",
		"The resource type (Send + Sync).",
		"The body's result type (Send + Sync)."
	)]
	impl<Sub, A, B> SendFunctor for SendBracketBrand<ArcBrand, Sub, A, B>
	where
		Sub: WrapDrop
			+ Kind_cdc7cd43dac7585f<Of<'static, ArcFree<Sub, ArcTypeErasedValue>>: Send + Sync>
			+ 'static,
		A: Send + Sync + 'static,
		B: Send + Sync + 'static,
	{
		/// Identity `send_map`. The cell's GAT projection `Of<'a, X>`
		/// erases X; threading `Send + Sync` bounds preserves the
		/// thread-safe invariant on the unchanged cell.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the continuations.",
			"The original GAT-filled type (Send + Sync).",
			"The new GAT-filled type (Send + Sync)."
		)]
		///
		#[document_parameters(
			"The function (ignored; the cell's structure is fixed by Sub / A / B).",
			"The send-bracket effect."
		)]
		///
		#[document_returns("The send-bracket effect unchanged.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		ArcBrand,
		/// 		IdentityBrand,
		/// 		SendBracketBrand,
		/// 	},
		/// 	classes::{
		/// 		SendFunctor,
		/// 		ToDynSendFn,
		/// 	},
		/// 	types::{
		/// 		ArcFree,
		/// 		effects::bracket::SendBracket,
		/// 	},
		/// };
		///
		/// let bracket: SendBracket<'static, ArcBrand, IdentityBrand, i32, i32> = SendBracket::Bracket {
		/// 	acquire: <ArcBrand as ToDynSendFn>::new(|_: ()| ArcFree::<IdentityBrand, _>::pure(7)),
		/// 	body: <ArcBrand as ToDynSendFn>::new(|_a: std::sync::Arc<i32>| {
		/// 		ArcFree::<IdentityBrand, _>::pure((7, 42))
		/// 	}),
		/// 	release: <ArcBrand as ToDynSendFn>::new(|_a: std::sync::Arc<i32>| {
		/// 		ArcFree::<IdentityBrand, _>::pure(())
		/// 	}),
		/// };
		/// let mapped = <SendBracketBrand<ArcBrand, IdentityBrand, i32, i32> as SendFunctor>::send_map(
		/// 	|x: i32| x + 1,
		/// 	bracket,
		/// );
		/// assert!(matches!(mapped, SendBracket::Bracket { .. }));
		/// ```
		fn send_map<'a, X: Send + Sync + 'a, Y: Send + Sync + 'a>(
			_f: impl Fn(X) -> Y + Send + Sync + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, X>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Y>) {
			fa
		}
	}

	// SendFunctor stubs for non-Send brands. Required by the
	// substrate's `NodeBrand<R, S>: SendFunctor` bound when S contains
	// these brands; the stubs delegate to `Functor::map` and cannot be
	// reached at runtime because the corresponding wrappers (Run /
	// RunExplicit / RcRun / RcRunExplicit) do not exercise the
	// SendFunctor path. Mirrors the BoxLocalBrand / LocalBrand
	// SendFunctor-stub precedent.

	#[document_type_parameters(
		"The substrate brand.",
		"The resource type (Send + Sync stub bound).",
		"The body's result type (Send + Sync stub bound)."
	)]
	impl<Sub, A, B> SendFunctor for BoxBracketBrand<BoxBrand, Sub, A, B>
	where
		Sub: WrapDrop + 'static,
		A: Send + Sync + 'static,
		B: Send + Sync + 'static,
	{
		/// SendFunctor stub for the BoxBrand-flavoured Bracket. The
		/// projection `Box<dyn FnOnce>` is not `Send + Sync`, so this
		/// brand does not appear in Arc-family substrates and the
		/// SendFunctor path is unreachable in practice. The body
		/// delegates to [`Functor::map`].
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the continuations.",
			"The original GAT-filled type.",
			"The new GAT-filled type."
		)]
		///
		#[document_parameters("The function (ignored).", "The bracket effect.")]
		///
		#[document_returns("The bracket effect unchanged.")]
		///
		#[document_examples]
		///
		/// ```
		/// // Exercised via brand-dispatch through NodeBrand<R, S>: SendFunctor
		/// // when S = CoproductBrand<BoxBracketBrand<BoxBrand, Sub, A, B>, _>; never
		/// // reached at runtime on default Run substrates.
		/// use fp_library::{
		/// 	brands::{
		/// 		BoxBracketBrand,
		/// 		BoxBrand,
		/// 		ThunkBrand,
		/// 	},
		/// 	classes::{
		/// 		SendFunctor,
		/// 		ToDynFnOnce,
		/// 	},
		/// 	types::{
		/// 		Free,
		/// 		effects::bracket::BoxBracket,
		/// 	},
		/// };
		///
		/// let bracket: BoxBracket<'static, BoxBrand, ThunkBrand, i32, i32> = BoxBracket::Bracket {
		/// 	acquire: <BoxBrand as ToDynFnOnce>::new(|_: ()| Free::<ThunkBrand, _>::pure(7)),
		/// 	body: <BoxBrand as ToDynFnOnce>::new(|_a: Box<i32>| Free::<ThunkBrand, _>::pure((7, 42))),
		/// 	release: <BoxBrand as ToDynFnOnce>::new(|_a: Box<i32>| Free::<ThunkBrand, _>::pure(())),
		/// };
		/// let mapped = <BoxBracketBrand<BoxBrand, ThunkBrand, i32, i32> as SendFunctor>::send_map(
		/// 	|x: i32| x + 1,
		/// 	bracket,
		/// );
		/// assert!(matches!(mapped, BoxBracket::Bracket { .. }));
		/// ```
		fn send_map<'a, X: Send + Sync + 'a, Y: Send + Sync + 'a>(
			_f: impl Fn(X) -> Y + Send + Sync + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, X>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Y>) {
			fa
		}
	}

	#[document_type_parameters(
		"The substrate brand.",
		"The resource type (Send + Sync stub bound).",
		"The body's result type (Send + Sync stub bound)."
	)]
	impl<Sub, A, B> SendFunctor for BracketBrand<RcBrand, Sub, A, B>
	where
		Sub: WrapDrop + 'static,
		A: Send + Sync + 'static,
		B: Send + Sync + 'static,
	{
		/// SendFunctor stub for the RcBrand-flavoured Bracket. The
		/// projection `Rc<dyn Fn>` is not `Send + Sync`, so this brand
		/// does not appear in Arc-family substrates; the SendFunctor
		/// path is reachable only through brand-dispatch but never
		/// exercised at runtime. Delegates to [`Functor::map`].
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the continuations.",
			"The original GAT-filled type.",
			"The new GAT-filled type."
		)]
		///
		#[document_parameters("The function (ignored).", "The bracket effect.")]
		///
		#[document_returns("The bracket effect unchanged.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		BracketBrand,
		/// 		RcBrand,
		/// 		ThunkBrand,
		/// 	},
		/// 	classes::{
		/// 		SendFunctor,
		/// 		ToDynCloneFn,
		/// 	},
		/// 	types::{
		/// 		RcFree,
		/// 		effects::bracket::Bracket,
		/// 	},
		/// };
		///
		/// let bracket: Bracket<'static, RcBrand, ThunkBrand, i32, i32> = Bracket::Bracket {
		/// 	acquire: <RcBrand as ToDynCloneFn>::new(|_: ()| RcFree::<ThunkBrand, _>::pure(7)),
		/// 	body: <RcBrand as ToDynCloneFn>::new(|_a: std::rc::Rc<i32>| {
		/// 		RcFree::<ThunkBrand, _>::pure((7, 42))
		/// 	}),
		/// 	release: <RcBrand as ToDynCloneFn>::new(|_a: std::rc::Rc<i32>| {
		/// 		RcFree::<ThunkBrand, _>::pure(())
		/// 	}),
		/// };
		/// let mapped = <BracketBrand<RcBrand, ThunkBrand, i32, i32> as SendFunctor>::send_map(
		/// 	|x: i32| x + 1,
		/// 	bracket,
		/// );
		/// assert!(matches!(mapped, Bracket::Bracket { .. }));
		/// ```
		fn send_map<'a, X: Send + Sync + 'a, Y: Send + Sync + 'a>(
			_f: impl Fn(X) -> Y + Send + Sync + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, X>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Y>) {
			fa
		}
	}

	// ===== WrapDrop impls =====
	//
	// Bracket cells return None for WrapDrop because the cell's
	// "result" (the body's program result) cannot be materialised
	// without first running acquire to produce a resource and
	// invoking body with that resource. The substrate's iterative
	// Drop path falls through to recursive drop on the layer; this
	// is sound because Bracket cells appear at the Node::Scoped arm
	// and the depth that grows with chain length lives in the
	// Free family's CatList of continuations, which the iterative
	// drop loop already dismantles. Mirrors the CoyonedaBrand
	// WrapDrop-returns-None precedent for type-erasure brands.

	#[document_type_parameters(
		"The substrate brand.",
		"The resource type.",
		"The body's result type."
	)]
	impl<Sub, A, B> WrapDrop for BoxBracketBrand<BoxBrand, Sub, A, B>
	where
		Sub: WrapDrop + 'static,
		A: 'static,
		B: 'static,
	{
		/// Returns `None`; the cell's body program result cannot be
		/// materialised without running acquire and body. The
		/// substrate's iterative drop falls through to recursive drop.
		#[document_signature]
		///
		#[document_type_parameters("The lifetime.", "The result type.")]
		///
		#[document_parameters("The bracket effect (dropped silently).")]
		///
		#[document_returns("`None` always; cells require dispatch to materialise a result.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		BoxBracketBrand,
		/// 		BoxBrand,
		/// 		ThunkBrand,
		/// 	},
		/// 	classes::{
		/// 		ToDynFnOnce,
		/// 		WrapDrop,
		/// 	},
		/// 	types::{
		/// 		Free,
		/// 		effects::bracket::BoxBracket,
		/// 	},
		/// };
		///
		/// let bracket: BoxBracket<'static, BoxBrand, ThunkBrand, i32, i32> = BoxBracket::Bracket {
		/// 	acquire: <BoxBrand as ToDynFnOnce>::new(|_: ()| Free::<ThunkBrand, _>::pure(7)),
		/// 	body: <BoxBrand as ToDynFnOnce>::new(|_a: Box<i32>| Free::<ThunkBrand, _>::pure((7, 42))),
		/// 	release: <BoxBrand as ToDynFnOnce>::new(|_a: Box<i32>| Free::<ThunkBrand, _>::pure(())),
		/// };
		/// assert_eq!(
		/// 	<BoxBracketBrand<BoxBrand, ThunkBrand, i32, i32> as WrapDrop>::drop::<i32>(bracket),
		/// 	None
		/// );
		/// ```
		fn drop<'a, X: 'a>(
			_fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, X>)
		) -> Option<X> {
			None
		}
	}

	#[document_type_parameters(
		"The substrate brand.",
		"The resource type.",
		"The body's result type."
	)]
	impl<Sub, A, B> WrapDrop for BracketBrand<RcBrand, Sub, A, B>
	where
		Sub: WrapDrop + 'static,
		A: 'static,
		B: 'static,
	{
		/// Returns `None`; mirrors [`BoxBracketBrand`'s `WrapDrop`].
		#[document_signature]
		///
		#[document_type_parameters("The lifetime.", "The result type.")]
		///
		#[document_parameters("The bracket effect (dropped silently).")]
		///
		#[document_returns("`None` always.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		BracketBrand,
		/// 		RcBrand,
		/// 		ThunkBrand,
		/// 	},
		/// 	classes::{
		/// 		ToDynCloneFn,
		/// 		WrapDrop,
		/// 	},
		/// 	types::{
		/// 		RcFree,
		/// 		effects::bracket::Bracket,
		/// 	},
		/// };
		///
		/// let bracket: Bracket<'static, RcBrand, ThunkBrand, i32, i32> = Bracket::Bracket {
		/// 	acquire: <RcBrand as ToDynCloneFn>::new(|_: ()| RcFree::<ThunkBrand, _>::pure(7)),
		/// 	body: <RcBrand as ToDynCloneFn>::new(|_a: std::rc::Rc<i32>| {
		/// 		RcFree::<ThunkBrand, _>::pure((7, 42))
		/// 	}),
		/// 	release: <RcBrand as ToDynCloneFn>::new(|_a: std::rc::Rc<i32>| {
		/// 		RcFree::<ThunkBrand, _>::pure(())
		/// 	}),
		/// };
		/// assert_eq!(
		/// 	<BracketBrand<RcBrand, ThunkBrand, i32, i32> as WrapDrop>::drop::<i32>(bracket),
		/// 	None
		/// );
		/// ```
		fn drop<'a, X: 'a>(
			_fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, X>)
		) -> Option<X> {
			None
		}
	}

	#[document_type_parameters(
		"The substrate brand.",
		"The resource type (Send + Sync).",
		"The body's result type (Send + Sync)."
	)]
	impl<Sub, A, B> WrapDrop for SendBracketBrand<ArcBrand, Sub, A, B>
	where
		Sub: WrapDrop
			+ Kind_cdc7cd43dac7585f<Of<'static, ArcFree<Sub, ArcTypeErasedValue>>: Send + Sync>
			+ 'static,
		A: Send + Sync + 'static,
		B: Send + Sync + 'static,
	{
		/// Returns `None`; mirrors [`BoxBracketBrand`'s `WrapDrop`].
		#[document_signature]
		///
		#[document_type_parameters("The lifetime.", "The result type.")]
		///
		#[document_parameters("The send-bracket effect (dropped silently).")]
		///
		#[document_returns("`None` always.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		ArcBrand,
		/// 		IdentityBrand,
		/// 		SendBracketBrand,
		/// 	},
		/// 	classes::{
		/// 		ToDynSendFn,
		/// 		WrapDrop,
		/// 	},
		/// 	types::{
		/// 		ArcFree,
		/// 		effects::bracket::SendBracket,
		/// 	},
		/// };
		///
		/// let bracket: SendBracket<'static, ArcBrand, IdentityBrand, i32, i32> = SendBracket::Bracket {
		/// 	acquire: <ArcBrand as ToDynSendFn>::new(|_: ()| ArcFree::<IdentityBrand, _>::pure(7)),
		/// 	body: <ArcBrand as ToDynSendFn>::new(|_a: std::sync::Arc<i32>| {
		/// 		ArcFree::<IdentityBrand, _>::pure((7, 42))
		/// 	}),
		/// 	release: <ArcBrand as ToDynSendFn>::new(|_a: std::sync::Arc<i32>| {
		/// 		ArcFree::<IdentityBrand, _>::pure(())
		/// 	}),
		/// };
		/// assert_eq!(
		/// 	<SendBracketBrand<ArcBrand, IdentityBrand, i32, i32> as WrapDrop>::drop::<i32>(bracket),
		/// 	None
		/// );
		/// ```
		fn drop<'a, X: 'a>(
			_fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, X>)
		) -> Option<X> {
			None
		}
	}

	// ===== Extract impls =====
	//
	// Bracket cells cannot extract a result without first running
	// acquire and body via the dispatcher; Extract is implemented
	// as a panicking stub. The substrate-required Extract cascade
	// (NodeBrand: Extract requires S: Extract, etc.) reaches Bracket
	// only on synthetic paths that never hold a real Bracket cell at
	// runtime: production interpret routes scoped layers to the
	// bracket dispatcher (step 6), not Extract.

	#[document_type_parameters(
		"The substrate brand.",
		"The resource type.",
		"The body's result type."
	)]
	impl<Sub, A, B> Extract for BoxBracketBrand<BoxBrand, Sub, A, B>
	where
		Sub: WrapDrop + 'static,
		A: 'static,
		B: 'static,
	{
		/// Panicking stub. Bracket cells require dispatcher-driven
		/// evaluation (acquire then body then release); Extract is
		/// substrate-required but never invoked on real Bracket cells
		/// at runtime.
		#[document_signature]
		///
		#[document_type_parameters("The lifetime.", "The result type.")]
		///
		#[document_parameters("The bracket effect (ignored; the impl panics).")]
		///
		#[document_returns("Never returns; panics with an unreachable! message.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		BoxBrand,
		/// 		ThunkBrand,
		/// 	},
		/// 	classes::ToDynFnOnce,
		/// 	types::{
		/// 		Free,
		/// 		effects::bracket::BoxBracket,
		/// 	},
		/// };
		///
		/// // Construct a cell to demonstrate the Extract impl is wired (compilation
		/// // check); real interpret routes Bracket cells to the bracket dispatcher
		/// // (step 6), so extract is never invoked at runtime.
		/// let bracket: BoxBracket<'static, BoxBrand, ThunkBrand, i32, i32> = BoxBracket::Bracket {
		/// 	acquire: <BoxBrand as ToDynFnOnce>::new(|_: ()| Free::<ThunkBrand, _>::pure(7)),
		/// 	body: <BoxBrand as ToDynFnOnce>::new(|_a: Box<i32>| Free::<ThunkBrand, _>::pure((7, 42))),
		/// 	release: <BoxBrand as ToDynFnOnce>::new(|_a: Box<i32>| Free::<ThunkBrand, _>::pure(())),
		/// };
		/// assert!(matches!(bracket, BoxBracket::Bracket { .. }));
		/// ```
		#[expect(
			clippy::unreachable,
			reason = "Bracket cells cannot extract a result without first running acquire to materialise the resource and invoking body via the dispatcher; Extract is a substrate-required impl that the scoped dispatch path never invokes (interpret routes scoped layers to the bracket dispatcher in step 6, not Extract)."
		)]
		fn extract<'a, X: 'a>(
			_fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, X>)
		) -> X {
			unreachable!(
				"BoxBracketBrand::extract invoked; Bracket cells require dispatcher-driven evaluation (acquire then body then release) and have no projection-only result"
			)
		}
	}

	#[document_type_parameters(
		"The substrate brand.",
		"The resource type.",
		"The body's result type."
	)]
	impl<Sub, A, B> Extract for BracketBrand<RcBrand, Sub, A, B>
	where
		Sub: WrapDrop + 'static,
		A: 'static,
		B: 'static,
	{
		/// Panicking stub; mirrors [`BoxBracketBrand`'s `Extract`].
		#[document_signature]
		///
		#[document_type_parameters("The lifetime.", "The result type.")]
		///
		#[document_parameters("The bracket effect (ignored).")]
		///
		#[document_returns("Never returns; panics with an unreachable! message.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		RcBrand,
		/// 		ThunkBrand,
		/// 	},
		/// 	classes::ToDynCloneFn,
		/// 	types::{
		/// 		RcFree,
		/// 		effects::bracket::Bracket,
		/// 	},
		/// };
		///
		/// // Construct a cell to demonstrate the Extract impl is wired.
		/// let bracket: Bracket<'static, RcBrand, ThunkBrand, i32, i32> = Bracket::Bracket {
		/// 	acquire: <RcBrand as ToDynCloneFn>::new(|_: ()| RcFree::<ThunkBrand, _>::pure(7)),
		/// 	body: <RcBrand as ToDynCloneFn>::new(|_a: std::rc::Rc<i32>| {
		/// 		RcFree::<ThunkBrand, _>::pure((7, 42))
		/// 	}),
		/// 	release: <RcBrand as ToDynCloneFn>::new(|_a: std::rc::Rc<i32>| {
		/// 		RcFree::<ThunkBrand, _>::pure(())
		/// 	}),
		/// };
		/// assert!(matches!(bracket, Bracket::Bracket { .. }));
		/// ```
		#[expect(
			clippy::unreachable,
			reason = "Bracket cells cannot extract a result without dispatcher-driven evaluation; substrate-required impl, never reached on real Bracket cells at runtime."
		)]
		fn extract<'a, X: 'a>(
			_fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, X>)
		) -> X {
			unreachable!(
				"BracketBrand::extract invoked; Bracket cells require dispatcher-driven evaluation"
			)
		}
	}

	#[document_type_parameters(
		"The substrate brand.",
		"The resource type (Send + Sync).",
		"The body's result type (Send + Sync)."
	)]
	impl<Sub, A, B> Extract for SendBracketBrand<ArcBrand, Sub, A, B>
	where
		Sub: WrapDrop
			+ Kind_cdc7cd43dac7585f<Of<'static, ArcFree<Sub, ArcTypeErasedValue>>: Send + Sync>
			+ 'static,
		A: Send + Sync + 'static,
		B: Send + Sync + 'static,
	{
		/// Panicking stub; mirrors [`BoxBracketBrand`'s `Extract`].
		#[document_signature]
		///
		#[document_type_parameters("The lifetime.", "The result type.")]
		///
		#[document_parameters("The send-bracket effect (ignored).")]
		///
		#[document_returns("Never returns; panics with an unreachable! message.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		ArcBrand,
		/// 		IdentityBrand,
		/// 	},
		/// 	classes::ToDynSendFn,
		/// 	types::{
		/// 		ArcFree,
		/// 		effects::bracket::SendBracket,
		/// 	},
		/// };
		///
		/// // Construct a cell to demonstrate the Extract impl is wired.
		/// let bracket: SendBracket<'static, ArcBrand, IdentityBrand, i32, i32> = SendBracket::Bracket {
		/// 	acquire: <ArcBrand as ToDynSendFn>::new(|_: ()| ArcFree::<IdentityBrand, _>::pure(7)),
		/// 	body: <ArcBrand as ToDynSendFn>::new(|_a: std::sync::Arc<i32>| {
		/// 		ArcFree::<IdentityBrand, _>::pure((7, 42))
		/// 	}),
		/// 	release: <ArcBrand as ToDynSendFn>::new(|_a: std::sync::Arc<i32>| {
		/// 		ArcFree::<IdentityBrand, _>::pure(())
		/// 	}),
		/// };
		/// assert!(matches!(bracket, SendBracket::Bracket { .. }));
		/// ```
		#[expect(
			clippy::unreachable,
			reason = "SendBracket cells cannot extract a result without dispatcher-driven evaluation; substrate-required impl, never reached on real cells at runtime."
		)]
		fn extract<'a, X: 'a>(
			_fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, X>)
		) -> X {
			unreachable!(
				"SendBracketBrand::extract invoked; Bracket cells require dispatcher-driven evaluation"
			)
		}
	}

	// ===== RefFunctor impls =====
	//
	// Under Option A, the brand's GAT projection `Of<'a, X>` is
	// independent of `X` (resolves to `Bracket<'a, P, Sub, A, B>`
	// regardless), so `RefFunctor::ref_map` is identity-shaped on
	// the cell: post-composing `func` over the body's program
	// return cannot change the cell's brand identity (Sub / A / B
	// are baked into the brand). For the Rc / Arc family the impl
	// is `Clone::clone(fa)`; for the Box family the cell is
	// non-`Clone` so the impl constructs a new BoxBracket with
	// `unreachable!()` stub closures (the path is reachable only
	// through synthetic non-Coyoneda first-order rows on
	// `RunExplicit`'s `RefFunctor` cascade, which real programs do
	// not exercise).
	//
	// `SendBracketBrand` does not impl `RefFunctor` (mirrors
	// `SendCatchBrand` / `SendLocalBrand` / `SendRefLocalBrand`
	// precedents): the cascade through `ArcRunExplicitBrand:
	// RefFunctor` does not require it (the `ArcFreeExplicitBrand:
	// !RefFunctor` brand-level docstring records the structural
	// gap), and a hypothetical impl would face the same
	// `Send + Sync` bound mismatch on `func` that prevents
	// `SendBracketBrand: Functor`.

	#[document_type_parameters(
		"The substrate brand.",
		"The resource type.",
		"The body's result type."
	)]
	impl<Sub, A, B> RefFunctor for BoxBracketBrand<BoxBrand, Sub, A, B>
	where
		Sub: WrapDrop + 'static,
		A: 'static,
		B: 'static,
	{
		/// Maps `func` over the cell's body program type by
		/// reference. Under Option A the cell's brand is fixed by
		/// `Sub` / `A` / `B`; the GAT projection `Of<'a, X>`
		/// erases X so the returned `Bracket` has the same type
		/// as the input. `BoxBracket` is non-`Clone`, so the
		/// returned cell is constructed with `unreachable!()` stub
		/// closures rather than a clone of `fa`. The path is
		/// reachable only through synthetic non-Coyoneda first-order
		/// rows on `RunExplicit`'s `RefFunctor` cascade, which real
		/// programs do not exercise.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the continuations.",
			"The original GAT-filled type.",
			"The new GAT-filled type after applying `func`."
		)]
		///
		#[document_parameters(
			"The function to apply by reference (ignored; the impl returns a stub cell).",
			"The bracket effect projection (ignored; the impl returns a stub cell)."
		)]
		///
		#[document_returns("A new bracket effect with all three closures as panicking stubs.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		BoxBracketBrand,
		/// 		BoxBrand,
		/// 		ThunkBrand,
		/// 	},
		/// 	classes::{
		/// 		RefFunctor,
		/// 		ToDynFnOnce,
		/// 	},
		/// 	types::{
		/// 		Free,
		/// 		effects::bracket::BoxBracket,
		/// 	},
		/// };
		///
		/// let bracket: BoxBracket<'static, BoxBrand, ThunkBrand, i32, i32> = BoxBracket::Bracket {
		/// 	acquire: <BoxBrand as ToDynFnOnce>::new(|_: ()| Free::<ThunkBrand, _>::pure(7)),
		/// 	body: <BoxBrand as ToDynFnOnce>::new(|_a: Box<i32>| Free::<ThunkBrand, _>::pure((7, 42))),
		/// 	release: <BoxBrand as ToDynFnOnce>::new(|_a: Box<i32>| Free::<ThunkBrand, _>::pure(())),
		/// };
		/// // Stub-only impl: the returned BoxBracket's closures are panicking
		/// // thunks; we only verify the variant tag here.
		/// let mapped = <BoxBracketBrand<BoxBrand, ThunkBrand, i32, i32> as RefFunctor>::ref_map(
		/// 	|x: &i32| *x + 1,
		/// 	&bracket,
		/// );
		/// assert!(matches!(mapped, BoxBracket::Bracket { .. }));
		/// ```
		#[expect(
			clippy::unreachable,
			reason = "BoxBracketBrand::ref_map cannot replicate the cell from a reference because BoxBracket is non-Clone (Box<dyn FnOnce> is uncloneable). The path is reachable only through synthetic non-Coyoneda first-order rows on RunExplicit, which real programs do not exercise."
		)]
		fn ref_map<'a, X: 'a, Y: 'a>(
			_func: impl Fn(&X) -> Y + 'a,
			_fa: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, X>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Y>) {
			BoxBracket::Bracket {
				acquire: <BoxBrand as ToDynFnOnce>::new(|_: ()| -> Free<Sub, A> {
					unreachable!(
						"BoxBracketBrand::ref_map's stub acquire invoked; BoxBracket is non-Clone and the impl is reachable only through synthetic substrate paths"
					)
				}),
				body: <BoxBrand as ToDynFnOnce>::new(
					|_a: <BoxBrand as Pointer>::Of<'a, A>| -> Free<Sub, (A, B)> {
						unreachable!(
							"BoxBracketBrand::ref_map's stub body invoked; BoxBracket is non-Clone and the impl is reachable only through synthetic substrate paths"
						)
					},
				),
				release: <BoxBrand as ToDynFnOnce>::new(
					|_a: <BoxBrand as Pointer>::Of<'a, A>| -> Free<Sub, ()> {
						unreachable!(
							"BoxBracketBrand::ref_map's stub release invoked; BoxBracket is non-Clone and the impl is reachable only through synthetic substrate paths"
						)
					},
				),
			}
		}
	}

	#[document_type_parameters(
		"The substrate brand.",
		"The resource type.",
		"The body's result type."
	)]
	impl<Sub, A, B> RefFunctor for BracketBrand<RcBrand, Sub, A, B>
	where
		Sub: WrapDrop + 'static,
		A: 'static,
		B: 'static,
	{
		/// Maps `func` over the cell's body program type by
		/// reference. Under Option A the cell's brand is fixed by
		/// `Sub` / `A` / `B`; the GAT projection `Of<'a, X>`
		/// erases X so the returned `Bracket` has the same type
		/// as the input. The impl is `Clone::clone(fa)` (Rc-bumps
		/// the three stored closure pointers); `func` is unused.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the continuations.",
			"The original GAT-filled type.",
			"The new GAT-filled type."
		)]
		///
		#[document_parameters(
			"The function to apply by reference (ignored).",
			"The bracket effect projection."
		)]
		///
		#[document_returns("A clone of the bracket effect (acquire / body / release Rc-bumped).")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		BracketBrand,
		/// 		RcBrand,
		/// 		ThunkBrand,
		/// 	},
		/// 	classes::{
		/// 		RefFunctor,
		/// 		ToDynCloneFn,
		/// 	},
		/// 	types::{
		/// 		RcFree,
		/// 		effects::bracket::Bracket,
		/// 	},
		/// };
		///
		/// let bracket: Bracket<'static, RcBrand, ThunkBrand, i32, i32> = Bracket::Bracket {
		/// 	acquire: <RcBrand as ToDynCloneFn>::new(|_: ()| RcFree::<ThunkBrand, _>::pure(7)),
		/// 	body: <RcBrand as ToDynCloneFn>::new(|_a: std::rc::Rc<i32>| {
		/// 		RcFree::<ThunkBrand, _>::pure((7, 42))
		/// 	}),
		/// 	release: <RcBrand as ToDynCloneFn>::new(|_a: std::rc::Rc<i32>| {
		/// 		RcFree::<ThunkBrand, _>::pure(())
		/// 	}),
		/// };
		/// let mapped = <BracketBrand<RcBrand, ThunkBrand, i32, i32> as RefFunctor>::ref_map(
		/// 	|x: &i32| *x + 1,
		/// 	&bracket,
		/// );
		/// assert!(matches!(mapped, Bracket::Bracket { .. }));
		/// ```
		fn ref_map<'a, X: 'a, Y: 'a>(
			_func: impl Fn(&X) -> Y + 'a,
			fa: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, X>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Y>) {
			fa.clone()
		}
	}

	// ===== BoxBracketExplicit (BoxBrand + FnOnce, single-shot, FreeExplicit substrate) =====

	/// Scoped resource-management effect for `RunExplicit` substrates.
	/// Mirrors [`BoxBracket`] structurally with one substrate swap:
	/// stored closures return [`FreeExplicit<'a, Sub, _>`] programs
	/// instead of [`Free<Sub, _>`]. Used by `RunExplicit::bracket` so
	/// the cell's substrate matches the wrapper's underlying Free
	/// family.
	#[document_type_parameters(
		"The lifetime of the acquire / body / release closures and the FreeExplicit programs.",
		"The pointer brand storing the closures (BoxBrand only by structural bound).",
		"The substrate brand over which acquire / body / release return FreeExplicit programs.",
		"The resource type produced by acquire and consumed by body / release.",
		"The body's result type."
	)]
	pub enum BoxBracketExplicit<'a, P, Sub, A, B>
	where
		P: ToDynFnOnce,
		Sub: WrapDrop + 'a,
		A: 'a,
		B: 'a, {
		/// Acquire a resource, run `body` with it to produce a paired
		/// body result, then run `release` to clean up. Mirrors
		/// [`BoxBracket::Bracket`] over the FreeExplicit substrate.
		Bracket {
			/// The acquire program, stored as a unit-arg B-thunk
			/// returning `FreeExplicit<'a, Sub, A>`.
			acquire: <P as Pointer>::Of<'a, dyn 'a + FnOnce(()) -> FreeExplicit<'a, Sub, A>>,
			/// The body closure returning `FreeExplicit<'a, Sub, (A, B)>`.
			#[expect(
				clippy::type_complexity,
				reason = "BracketExplicit cells store closures returning FreeExplicit programs derived from the substrate brand Sub; the nested GAT and FreeExplicit projections cannot be aliased without losing the per-pointer-brand structure ToDynFnOnce / ToDynCloneFn / ToDynSendFn dispatch over."
			)]
			body: <P as Pointer>::Of<
				'a,
				dyn 'a + FnOnce(<P as Pointer>::Of<'a, A>) -> FreeExplicit<'a, Sub, (A, B)>,
			>,
			/// The release closure returning `FreeExplicit<'a, Sub, ()>`.
			#[expect(
				clippy::type_complexity,
				reason = "BracketExplicit cells store closures returning FreeExplicit programs derived from the substrate brand Sub; the nested GAT and FreeExplicit projections cannot be aliased without losing the per-pointer-brand structure ToDynFnOnce / ToDynCloneFn / ToDynSendFn dispatch over."
			)]
			release: <P as Pointer>::Of<
				'a,
				dyn 'a + FnOnce(<P as Pointer>::Of<'a, A>) -> FreeExplicit<'a, Sub, ()>,
			>,
		},
	}

	impl_kind! {
		impl<P: ToDynFnOnce, Sub: WrapDrop + 'static, A: 'static, B: 'static> for BoxBracketExplicitBrand<P, Sub, A, B> {
			type Of<'a, X: 'a>: 'a = BoxBracketExplicit<'a, P, Sub, A, B>;
		}
	}

	#[document_type_parameters(
		"The substrate brand.",
		"The resource type.",
		"The body's result type."
	)]
	impl<Sub, A, B> Functor for BoxBracketExplicitBrand<BoxBrand, Sub, A, B>
	where
		Sub: WrapDrop + 'static,
		A: 'static,
		B: 'static,
	{
		/// Identity `map`; mirrors [`BoxBracketBrand`'s `Functor`].
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime.",
			"The original GAT-filled type.",
			"The new GAT-filled type."
		)]
		///
		#[document_parameters("The function (ignored).", "The bracket effect.")]
		///
		#[document_returns("The bracket effect unchanged.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		BoxBracketExplicitBrand,
		/// 		BoxBrand,
		/// 		IdentityBrand,
		/// 	},
		/// 	classes::{
		/// 		Functor,
		/// 		ToDynFnOnce,
		/// 	},
		/// 	types::{
		/// 		FreeExplicit,
		/// 		effects::bracket::BoxBracketExplicit,
		/// 	},
		/// };
		///
		/// let bracket: BoxBracketExplicit<'static, BoxBrand, IdentityBrand, i32, i32> =
		/// 	BoxBracketExplicit::Bracket {
		/// 		acquire: <BoxBrand as ToDynFnOnce>::new(|_: ()| {
		/// 			FreeExplicit::<IdentityBrand, _>::pure(7)
		/// 		}),
		/// 		body: <BoxBrand as ToDynFnOnce>::new(|_a: Box<i32>| {
		/// 			FreeExplicit::<IdentityBrand, _>::pure((7, 42))
		/// 		}),
		/// 		release: <BoxBrand as ToDynFnOnce>::new(|_a: Box<i32>| {
		/// 			FreeExplicit::<IdentityBrand, _>::pure(())
		/// 		}),
		/// 	};
		/// let mapped = <BoxBracketExplicitBrand<BoxBrand, IdentityBrand, i32, i32> as Functor>::map(
		/// 	|x: i32| x + 1,
		/// 	bracket,
		/// );
		/// assert!(matches!(mapped, BoxBracketExplicit::Bracket { .. }));
		/// ```
		fn map<'a, X: 'a, Y: 'a>(
			_f: impl Fn(X) -> Y + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, X>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Y>) {
			fa
		}
	}

	// ===== BracketExplicit (RcBrand + Fn, multi-shot, RcFreeExplicit substrate) =====

	/// Scoped resource-management effect for `RcRunExplicit` substrates.
	/// Mirrors [`Bracket`] structurally with one substrate swap:
	/// stored closures return [`RcFreeExplicit<'a, Sub, _>`] programs
	/// instead of [`RcFree<Sub, _>`].
	#[document_type_parameters(
		"The lifetime of the acquire / body / release closures and the RcFreeExplicit programs.",
		"The pointer brand storing the closures (RcBrand only).",
		"The substrate brand over which acquire / body / release return RcFreeExplicit programs.",
		"The resource type.",
		"The body's result type."
	)]
	pub enum BracketExplicit<'a, P, Sub, A, B>
	where
		P: ToDynCloneFn,
		Sub: WrapDrop + 'a,
		A: 'a,
		B: 'a, {
		/// Mirrors [`Bracket::Bracket`] over the RcFreeExplicit substrate.
		Bracket {
			/// The acquire program returning `RcFreeExplicit<'a, Sub, A>`.
			acquire:
				<P as RefCountedPointer>::Of<'a, dyn 'a + Fn(()) -> RcFreeExplicit<'a, Sub, A>>,
			/// The body closure returning `RcFreeExplicit<'a, Sub, (A, B)>`.
			#[expect(
				clippy::type_complexity,
				reason = "BracketExplicit cells store closures returning RcFreeExplicit programs derived from the substrate brand Sub; the nested GAT and RcFreeExplicit projections cannot be aliased without losing the per-pointer-brand structure ToDynFnOnce / ToDynCloneFn / ToDynSendFn dispatch over."
			)]
			body: <P as RefCountedPointer>::Of<
				'a,
				dyn 'a + Fn(<P as RefCountedPointer>::Of<'a, A>) -> RcFreeExplicit<'a, Sub, (A, B)>,
			>,
			/// The release closure returning `RcFreeExplicit<'a, Sub, ()>`.
			#[expect(
				clippy::type_complexity,
				reason = "BracketExplicit cells store closures returning RcFreeExplicit programs derived from the substrate brand Sub; the nested GAT and RcFreeExplicit projections cannot be aliased without losing the per-pointer-brand structure ToDynFnOnce / ToDynCloneFn / ToDynSendFn dispatch over."
			)]
			release: <P as RefCountedPointer>::Of<
				'a,
				dyn 'a + Fn(<P as RefCountedPointer>::Of<'a, A>) -> RcFreeExplicit<'a, Sub, ()>,
			>,
		},
	}

	impl_kind! {
		impl<P: ToDynCloneFn, Sub: WrapDrop + 'static, A: 'static, B: 'static> for BracketExplicitBrand<P, Sub, A, B> {
			type Of<'a, X: 'a>: 'a = BracketExplicit<'a, P, Sub, A, B>;
		}
	}

	#[document_type_parameters(
		"The lifetime.",
		"The pointer brand.",
		"The substrate brand.",
		"The resource type.",
		"The body's result type."
	)]
	#[document_parameters("The bracket-explicit effect to clone.")]
	impl<'a, P, Sub, A, B> Clone for BracketExplicit<'a, P, Sub, A, B>
	where
		P: ToDynCloneFn,
		Sub: WrapDrop + 'a,
		A: 'a,
		B: 'a,
	{
		/// Clones by refcount-bumping the stored pointers.
		#[document_signature]
		///
		#[document_returns(
			"A new bracket-explicit effect sharing acquire / body / release by refcount."
		)]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		IdentityBrand,
		/// 		RcBrand,
		/// 	},
		/// 	classes::ToDynCloneFn,
		/// 	types::{
		/// 		RcFreeExplicit,
		/// 		effects::bracket::BracketExplicit,
		/// 	},
		/// };
		///
		/// let original: BracketExplicit<'static, RcBrand, IdentityBrand, i32, i32> =
		/// 	BracketExplicit::Bracket {
		/// 		acquire: <RcBrand as ToDynCloneFn>::new(|_: ()| {
		/// 			RcFreeExplicit::<IdentityBrand, _>::pure(7)
		/// 		}),
		/// 		body: <RcBrand as ToDynCloneFn>::new(|_a: std::rc::Rc<i32>| {
		/// 			RcFreeExplicit::<IdentityBrand, _>::pure((7, 42))
		/// 		}),
		/// 		release: <RcBrand as ToDynCloneFn>::new(|_a: std::rc::Rc<i32>| {
		/// 			RcFreeExplicit::<IdentityBrand, _>::pure(())
		/// 		}),
		/// 	};
		/// let cloned = original.clone();
		/// assert!(matches!(cloned, BracketExplicit::Bracket { .. }));
		/// ```
		fn clone(&self) -> Self {
			match self {
				BracketExplicit::Bracket {
					acquire,
					body,
					release,
				} => BracketExplicit::Bracket {
					acquire: <P as RefCountedPointer>::Of::clone(acquire),
					body: <P as RefCountedPointer>::Of::clone(body),
					release: <P as RefCountedPointer>::Of::clone(release),
				},
			}
		}
	}

	#[document_type_parameters(
		"The substrate brand.",
		"The resource type.",
		"The body's result type."
	)]
	impl<Sub, A, B> Functor for BracketExplicitBrand<RcBrand, Sub, A, B>
	where
		Sub: WrapDrop + 'static,
		A: 'static,
		B: 'static,
	{
		/// Identity `map`; mirrors [`BracketBrand`'s `Functor`].
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime.",
			"The original GAT-filled type.",
			"The new GAT-filled type."
		)]
		///
		#[document_parameters("The function (ignored).", "The bracket-explicit effect.")]
		///
		#[document_returns("The bracket-explicit effect unchanged.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		BracketExplicitBrand,
		/// 		IdentityBrand,
		/// 		RcBrand,
		/// 	},
		/// 	classes::{
		/// 		Functor,
		/// 		ToDynCloneFn,
		/// 	},
		/// 	types::{
		/// 		RcFreeExplicit,
		/// 		effects::bracket::BracketExplicit,
		/// 	},
		/// };
		///
		/// let bracket: BracketExplicit<'static, RcBrand, IdentityBrand, i32, i32> =
		/// 	BracketExplicit::Bracket {
		/// 		acquire: <RcBrand as ToDynCloneFn>::new(|_: ()| {
		/// 			RcFreeExplicit::<IdentityBrand, _>::pure(7)
		/// 		}),
		/// 		body: <RcBrand as ToDynCloneFn>::new(|_a: std::rc::Rc<i32>| {
		/// 			RcFreeExplicit::<IdentityBrand, _>::pure((7, 42))
		/// 		}),
		/// 		release: <RcBrand as ToDynCloneFn>::new(|_a: std::rc::Rc<i32>| {
		/// 			RcFreeExplicit::<IdentityBrand, _>::pure(())
		/// 		}),
		/// 	};
		/// let mapped = <BracketExplicitBrand<RcBrand, IdentityBrand, i32, i32> as Functor>::map(
		/// 	|x: i32| x + 1,
		/// 	bracket,
		/// );
		/// assert!(matches!(mapped, BracketExplicit::Bracket { .. }));
		/// ```
		fn map<'a, X: 'a, Y: 'a>(
			_f: impl Fn(X) -> Y + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, X>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Y>) {
			fa
		}
	}

	// ===== SendBracketExplicit (ArcBrand + Fn + Send + Sync, ArcFreeExplicit substrate) =====

	/// Thread-safe scoped resource-management effect for
	/// `ArcRunExplicit` substrates. Mirrors [`SendBracket`] structurally
	/// with one substrate swap: stored closures return
	/// [`ArcFreeExplicit<'a, Sub, _>`] programs instead of
	/// [`ArcFree<Sub, _>`].
	#[document_type_parameters(
		"The lifetime of the acquire / body / release closures and the ArcFreeExplicit programs.",
		"The pointer brand storing the closures (ArcBrand only).",
		"The substrate brand over which acquire / body / release return ArcFreeExplicit programs.",
		"The resource type (Send + Sync).",
		"The body's result type (Send + Sync)."
	)]
	pub enum SendBracketExplicit<'a, P, Sub, A, B>
	where
		P: ToDynSendFn,
		Sub: WrapDrop + 'a,
		A: Send + Sync + 'a,
		B: Send + Sync + 'a, {
		/// Mirrors [`SendBracket::Bracket`] over the ArcFreeExplicit substrate.
		Bracket {
			/// The acquire program returning `ArcFreeExplicit<'a, Sub, A>`.
			acquire: <P as SendRefCountedPointer>::Of<
				'a,
				dyn 'a + Fn(()) -> ArcFreeExplicit<'a, Sub, A> + Send + Sync,
			>,
			/// The body closure returning `ArcFreeExplicit<'a, Sub, (A, B)>`.
			#[expect(
				clippy::type_complexity,
				reason = "SendBracketExplicit cells store closures returning ArcFreeExplicit programs derived from the substrate brand Sub; the nested GAT and ArcFreeExplicit projections cannot be aliased without losing the per-pointer-brand structure ToDynFnOnce / ToDynCloneFn / ToDynSendFn dispatch over."
			)]
			body: <P as SendRefCountedPointer>::Of<
				'a,
				dyn 'a
					+ Fn(<P as SendRefCountedPointer>::Of<'a, A>) -> ArcFreeExplicit<'a, Sub, (A, B)>
					+ Send
					+ Sync,
			>,
			/// The release closure returning `ArcFreeExplicit<'a, Sub, ()>`.
			#[expect(
				clippy::type_complexity,
				reason = "SendBracketExplicit cells store closures returning ArcFreeExplicit programs derived from the substrate brand Sub; the nested GAT and ArcFreeExplicit projections cannot be aliased without losing the per-pointer-brand structure ToDynFnOnce / ToDynCloneFn / ToDynSendFn dispatch over."
			)]
			release: <P as SendRefCountedPointer>::Of<
				'a,
				dyn 'a
					+ Fn(<P as SendRefCountedPointer>::Of<'a, A>) -> ArcFreeExplicit<'a, Sub, ()>
					+ Send
					+ Sync,
			>,
		},
	}

	impl_kind! {
		impl<P: ToDynSendFn, Sub: WrapDrop + 'static, A: Send + Sync + 'static, B: Send + Sync + 'static> for SendBracketExplicitBrand<P, Sub, A, B> {
			type Of<'a, X: 'a>: 'a = SendBracketExplicit<'a, P, Sub, A, B>;
		}
	}

	#[document_type_parameters(
		"The lifetime.",
		"The pointer brand.",
		"The substrate brand.",
		"The resource type.",
		"The body's result type."
	)]
	#[document_parameters("The send-bracket-explicit effect to clone.")]
	impl<'a, P, Sub, A, B> Clone for SendBracketExplicit<'a, P, Sub, A, B>
	where
		P: ToDynSendFn,
		Sub: WrapDrop + 'a,
		A: Send + Sync + 'a,
		B: Send + Sync + 'a,
	{
		/// Clones by refcount-bumping the stored pointers.
		#[document_signature]
		///
		#[document_returns(
			"A new send-bracket-explicit effect sharing acquire / body / release by refcount."
		)]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		ArcBrand,
		/// 		IdentityBrand,
		/// 	},
		/// 	classes::ToDynSendFn,
		/// 	types::{
		/// 		ArcFreeExplicit,
		/// 		effects::bracket::SendBracketExplicit,
		/// 	},
		/// };
		///
		/// let original: SendBracketExplicit<'static, ArcBrand, IdentityBrand, i32, i32> =
		/// 	SendBracketExplicit::Bracket {
		/// 		acquire: <ArcBrand as ToDynSendFn>::new(|_: ()| {
		/// 			ArcFreeExplicit::<IdentityBrand, _>::pure(7)
		/// 		}),
		/// 		body: <ArcBrand as ToDynSendFn>::new(|_a: std::sync::Arc<i32>| {
		/// 			ArcFreeExplicit::<IdentityBrand, _>::pure((7, 42))
		/// 		}),
		/// 		release: <ArcBrand as ToDynSendFn>::new(|_a: std::sync::Arc<i32>| {
		/// 			ArcFreeExplicit::<IdentityBrand, _>::pure(())
		/// 		}),
		/// 	};
		/// let cloned = original.clone();
		/// assert!(matches!(cloned, SendBracketExplicit::Bracket { .. }));
		/// ```
		fn clone(&self) -> Self {
			match self {
				SendBracketExplicit::Bracket {
					acquire,
					body,
					release,
				} => SendBracketExplicit::Bracket {
					acquire: <P as SendRefCountedPointer>::Of::clone(acquire),
					body: <P as SendRefCountedPointer>::Of::clone(body),
					release: <P as SendRefCountedPointer>::Of::clone(release),
				},
			}
		}
	}

	// ===== SendFunctor for SendBracketExplicitBrand =====

	#[document_type_parameters(
		"The substrate brand.",
		"The resource type (Send + Sync).",
		"The body's result type (Send + Sync)."
	)]
	impl<Sub, A, B> SendFunctor for SendBracketExplicitBrand<ArcBrand, Sub, A, B>
	where
		Sub: WrapDrop + 'static,
		A: Send + Sync + 'static,
		B: Send + Sync + 'static,
	{
		/// Identity `send_map`; mirrors [`SendBracketBrand`'s `SendFunctor`].
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime.",
			"The original GAT-filled type.",
			"The new GAT-filled type."
		)]
		///
		#[document_parameters("The function (ignored).", "The send-bracket-explicit effect.")]
		///
		#[document_returns("The send-bracket-explicit effect unchanged.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		ArcBrand,
		/// 		IdentityBrand,
		/// 		SendBracketExplicitBrand,
		/// 	},
		/// 	classes::{
		/// 		SendFunctor,
		/// 		ToDynSendFn,
		/// 	},
		/// 	types::{
		/// 		ArcFreeExplicit,
		/// 		effects::bracket::SendBracketExplicit,
		/// 	},
		/// };
		///
		/// let bracket: SendBracketExplicit<'static, ArcBrand, IdentityBrand, i32, i32> =
		/// 	SendBracketExplicit::Bracket {
		/// 		acquire: <ArcBrand as ToDynSendFn>::new(|_: ()| {
		/// 			ArcFreeExplicit::<IdentityBrand, _>::pure(7)
		/// 		}),
		/// 		body: <ArcBrand as ToDynSendFn>::new(|_a: std::sync::Arc<i32>| {
		/// 			ArcFreeExplicit::<IdentityBrand, _>::pure((7, 42))
		/// 		}),
		/// 		release: <ArcBrand as ToDynSendFn>::new(|_a: std::sync::Arc<i32>| {
		/// 			ArcFreeExplicit::<IdentityBrand, _>::pure(())
		/// 		}),
		/// 	};
		/// let mapped =
		/// 	<SendBracketExplicitBrand<ArcBrand, IdentityBrand, i32, i32> as SendFunctor>::send_map(
		/// 		|x: i32| x + 1,
		/// 		bracket,
		/// 	);
		/// assert!(matches!(mapped, SendBracketExplicit::Bracket { .. }));
		/// ```
		fn send_map<'a, X: Send + Sync + 'a, Y: Send + Sync + 'a>(
			_f: impl Fn(X) -> Y + Send + Sync + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, X>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Y>) {
			fa
		}
	}

	// SendFunctor stubs for the non-Send Explicit-family brands. Mirror
	// the Erased-family stub precedent: required by the substrate's
	// `NodeBrand<R, S>: SendFunctor` cascade when S contains these
	// brands; never reached at runtime because the corresponding
	// wrappers (RunExplicit / RcRunExplicit) do not exercise the
	// SendFunctor path.

	#[document_type_parameters(
		"The substrate brand.",
		"The resource type (Send + Sync stub bound).",
		"The body's result type (Send + Sync stub bound)."
	)]
	impl<Sub, A, B> SendFunctor for BoxBracketExplicitBrand<BoxBrand, Sub, A, B>
	where
		Sub: WrapDrop + 'static,
		A: Send + Sync + 'static,
		B: Send + Sync + 'static,
	{
		/// SendFunctor stub; mirrors [`BoxBracketBrand`'s `SendFunctor`] stub.
		#[document_signature]
		///
		#[document_type_parameters("The lifetime.", "The original type.", "The new type.")]
		///
		#[document_parameters("The function (ignored).", "The bracket-explicit effect.")]
		///
		#[document_returns("The bracket-explicit effect unchanged.")]
		///
		#[document_examples]
		///
		/// ```
		/// // Reachable only through brand-dispatch via NodeBrand<R, S>: SendFunctor;
		/// // never invoked at runtime on RunExplicit substrates.
		/// use fp_library::{
		/// 	brands::{
		/// 		BoxBracketExplicitBrand,
		/// 		BoxBrand,
		/// 		IdentityBrand,
		/// 	},
		/// 	classes::{
		/// 		SendFunctor,
		/// 		ToDynFnOnce,
		/// 	},
		/// 	types::{
		/// 		FreeExplicit,
		/// 		effects::bracket::BoxBracketExplicit,
		/// 	},
		/// };
		///
		/// let bracket: BoxBracketExplicit<'static, BoxBrand, IdentityBrand, i32, i32> =
		/// 	BoxBracketExplicit::Bracket {
		/// 		acquire: <BoxBrand as ToDynFnOnce>::new(|_: ()| {
		/// 			FreeExplicit::<IdentityBrand, _>::pure(7)
		/// 		}),
		/// 		body: <BoxBrand as ToDynFnOnce>::new(|_a: Box<i32>| {
		/// 			FreeExplicit::<IdentityBrand, _>::pure((7, 42))
		/// 		}),
		/// 		release: <BoxBrand as ToDynFnOnce>::new(|_a: Box<i32>| {
		/// 			FreeExplicit::<IdentityBrand, _>::pure(())
		/// 		}),
		/// 	};
		/// let mapped =
		/// 	<BoxBracketExplicitBrand<BoxBrand, IdentityBrand, i32, i32> as SendFunctor>::send_map(
		/// 		|x: i32| x + 1,
		/// 		bracket,
		/// 	);
		/// assert!(matches!(mapped, BoxBracketExplicit::Bracket { .. }));
		/// ```
		fn send_map<'a, X: Send + Sync + 'a, Y: Send + Sync + 'a>(
			_f: impl Fn(X) -> Y + Send + Sync + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, X>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Y>) {
			fa
		}
	}

	#[document_type_parameters(
		"The substrate brand.",
		"The resource type (Send + Sync stub bound).",
		"The body's result type (Send + Sync stub bound)."
	)]
	impl<Sub, A, B> SendFunctor for BracketExplicitBrand<RcBrand, Sub, A, B>
	where
		Sub: WrapDrop + 'static,
		A: Send + Sync + 'static,
		B: Send + Sync + 'static,
	{
		/// SendFunctor stub; mirrors [`BracketBrand`'s `SendFunctor`] stub.
		#[document_signature]
		///
		#[document_type_parameters("The lifetime.", "The original type.", "The new type.")]
		///
		#[document_parameters("The function (ignored).", "The bracket-explicit effect.")]
		///
		#[document_returns("The bracket-explicit effect unchanged.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		BracketExplicitBrand,
		/// 		IdentityBrand,
		/// 		RcBrand,
		/// 	},
		/// 	classes::{
		/// 		SendFunctor,
		/// 		ToDynCloneFn,
		/// 	},
		/// 	types::{
		/// 		RcFreeExplicit,
		/// 		effects::bracket::BracketExplicit,
		/// 	},
		/// };
		///
		/// let bracket: BracketExplicit<'static, RcBrand, IdentityBrand, i32, i32> =
		/// 	BracketExplicit::Bracket {
		/// 		acquire: <RcBrand as ToDynCloneFn>::new(|_: ()| {
		/// 			RcFreeExplicit::<IdentityBrand, _>::pure(7)
		/// 		}),
		/// 		body: <RcBrand as ToDynCloneFn>::new(|_a: std::rc::Rc<i32>| {
		/// 			RcFreeExplicit::<IdentityBrand, _>::pure((7, 42))
		/// 		}),
		/// 		release: <RcBrand as ToDynCloneFn>::new(|_a: std::rc::Rc<i32>| {
		/// 			RcFreeExplicit::<IdentityBrand, _>::pure(())
		/// 		}),
		/// 	};
		/// let mapped = <BracketExplicitBrand<RcBrand, IdentityBrand, i32, i32> as SendFunctor>::send_map(
		/// 	|x: i32| x + 1,
		/// 	bracket,
		/// );
		/// assert!(matches!(mapped, BracketExplicit::Bracket { .. }));
		/// ```
		fn send_map<'a, X: Send + Sync + 'a, Y: Send + Sync + 'a>(
			_f: impl Fn(X) -> Y + Send + Sync + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, X>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Y>) {
			fa
		}
	}

	// ===== WrapDrop impls for Explicit-family brands =====

	#[document_type_parameters(
		"The substrate brand.",
		"The resource type.",
		"The body's result type."
	)]
	impl<Sub, A, B> WrapDrop for BoxBracketExplicitBrand<BoxBrand, Sub, A, B>
	where
		Sub: WrapDrop + 'static,
		A: 'static,
		B: 'static,
	{
		/// Returns `None`; mirrors [`BoxBracketBrand`'s `WrapDrop`].
		#[document_signature]
		///
		#[document_type_parameters("The lifetime.", "The result type.")]
		///
		#[document_parameters("The bracket-explicit effect (dropped silently).")]
		///
		#[document_returns("`None` always.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		BoxBracketExplicitBrand,
		/// 		BoxBrand,
		/// 		IdentityBrand,
		/// 	},
		/// 	classes::{
		/// 		ToDynFnOnce,
		/// 		WrapDrop,
		/// 	},
		/// 	types::{
		/// 		FreeExplicit,
		/// 		effects::bracket::BoxBracketExplicit,
		/// 	},
		/// };
		///
		/// let bracket: BoxBracketExplicit<'static, BoxBrand, IdentityBrand, i32, i32> =
		/// 	BoxBracketExplicit::Bracket {
		/// 		acquire: <BoxBrand as ToDynFnOnce>::new(|_: ()| {
		/// 			FreeExplicit::<IdentityBrand, _>::pure(7)
		/// 		}),
		/// 		body: <BoxBrand as ToDynFnOnce>::new(|_a: Box<i32>| {
		/// 			FreeExplicit::<IdentityBrand, _>::pure((7, 42))
		/// 		}),
		/// 		release: <BoxBrand as ToDynFnOnce>::new(|_a: Box<i32>| {
		/// 			FreeExplicit::<IdentityBrand, _>::pure(())
		/// 		}),
		/// 	};
		/// assert_eq!(
		/// 	<BoxBracketExplicitBrand<BoxBrand, IdentityBrand, i32, i32> as WrapDrop>::drop::<i32>(
		/// 		bracket
		/// 	),
		/// 	None
		/// );
		/// ```
		fn drop<'a, X: 'a>(
			_fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, X>)
		) -> Option<X> {
			None
		}
	}

	#[document_type_parameters(
		"The substrate brand.",
		"The resource type.",
		"The body's result type."
	)]
	impl<Sub, A, B> WrapDrop for BracketExplicitBrand<RcBrand, Sub, A, B>
	where
		Sub: WrapDrop + 'static,
		A: 'static,
		B: 'static,
	{
		/// Returns `None`; mirrors [`BoxBracketBrand`'s `WrapDrop`].
		#[document_signature]
		///
		#[document_type_parameters("The lifetime.", "The result type.")]
		///
		#[document_parameters("The bracket-explicit effect (dropped silently).")]
		///
		#[document_returns("`None` always.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		BracketExplicitBrand,
		/// 		IdentityBrand,
		/// 		RcBrand,
		/// 	},
		/// 	classes::{
		/// 		ToDynCloneFn,
		/// 		WrapDrop,
		/// 	},
		/// 	types::{
		/// 		RcFreeExplicit,
		/// 		effects::bracket::BracketExplicit,
		/// 	},
		/// };
		///
		/// let bracket: BracketExplicit<'static, RcBrand, IdentityBrand, i32, i32> =
		/// 	BracketExplicit::Bracket {
		/// 		acquire: <RcBrand as ToDynCloneFn>::new(|_: ()| {
		/// 			RcFreeExplicit::<IdentityBrand, _>::pure(7)
		/// 		}),
		/// 		body: <RcBrand as ToDynCloneFn>::new(|_a: std::rc::Rc<i32>| {
		/// 			RcFreeExplicit::<IdentityBrand, _>::pure((7, 42))
		/// 		}),
		/// 		release: <RcBrand as ToDynCloneFn>::new(|_a: std::rc::Rc<i32>| {
		/// 			RcFreeExplicit::<IdentityBrand, _>::pure(())
		/// 		}),
		/// 	};
		/// assert_eq!(
		/// 	<BracketExplicitBrand<RcBrand, IdentityBrand, i32, i32> as WrapDrop>::drop::<i32>(bracket),
		/// 	None
		/// );
		/// ```
		fn drop<'a, X: 'a>(
			_fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, X>)
		) -> Option<X> {
			None
		}
	}

	#[document_type_parameters(
		"The substrate brand.",
		"The resource type (Send + Sync).",
		"The body's result type (Send + Sync)."
	)]
	impl<Sub, A, B> WrapDrop for SendBracketExplicitBrand<ArcBrand, Sub, A, B>
	where
		Sub: WrapDrop + 'static,
		A: Send + Sync + 'static,
		B: Send + Sync + 'static,
	{
		/// Returns `None`; mirrors [`BoxBracketBrand`'s `WrapDrop`].
		#[document_signature]
		///
		#[document_type_parameters("The lifetime.", "The result type.")]
		///
		#[document_parameters("The send-bracket-explicit effect (dropped silently).")]
		///
		#[document_returns("`None` always.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		ArcBrand,
		/// 		IdentityBrand,
		/// 		SendBracketExplicitBrand,
		/// 	},
		/// 	classes::{
		/// 		ToDynSendFn,
		/// 		WrapDrop,
		/// 	},
		/// 	types::{
		/// 		ArcFreeExplicit,
		/// 		effects::bracket::SendBracketExplicit,
		/// 	},
		/// };
		///
		/// let bracket: SendBracketExplicit<'static, ArcBrand, IdentityBrand, i32, i32> =
		/// 	SendBracketExplicit::Bracket {
		/// 		acquire: <ArcBrand as ToDynSendFn>::new(|_: ()| {
		/// 			ArcFreeExplicit::<IdentityBrand, _>::pure(7)
		/// 		}),
		/// 		body: <ArcBrand as ToDynSendFn>::new(|_a: std::sync::Arc<i32>| {
		/// 			ArcFreeExplicit::<IdentityBrand, _>::pure((7, 42))
		/// 		}),
		/// 		release: <ArcBrand as ToDynSendFn>::new(|_a: std::sync::Arc<i32>| {
		/// 			ArcFreeExplicit::<IdentityBrand, _>::pure(())
		/// 		}),
		/// 	};
		/// assert_eq!(
		/// 	<SendBracketExplicitBrand<ArcBrand, IdentityBrand, i32, i32> as WrapDrop>::drop::<i32>(
		/// 		bracket
		/// 	),
		/// 	None
		/// );
		/// ```
		fn drop<'a, X: 'a>(
			_fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, X>)
		) -> Option<X> {
			None
		}
	}

	// ===== Extract impls for Explicit-family brands =====

	#[document_type_parameters(
		"The substrate brand.",
		"The resource type.",
		"The body's result type."
	)]
	impl<Sub, A, B> Extract for BoxBracketExplicitBrand<BoxBrand, Sub, A, B>
	where
		Sub: WrapDrop + 'static,
		A: 'static,
		B: 'static,
	{
		/// Panicking stub; mirrors [`BoxBracketBrand`'s `Extract`].
		#[document_signature]
		///
		#[document_type_parameters("The lifetime.", "The result type.")]
		///
		#[document_parameters("The bracket-explicit effect (ignored).")]
		///
		#[document_returns("Never returns; panics with an unreachable! message.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		BoxBrand,
		/// 		IdentityBrand,
		/// 	},
		/// 	classes::ToDynFnOnce,
		/// 	types::{
		/// 		FreeExplicit,
		/// 		effects::bracket::BoxBracketExplicit,
		/// 	},
		/// };
		///
		/// let bracket: BoxBracketExplicit<'static, BoxBrand, IdentityBrand, i32, i32> =
		/// 	BoxBracketExplicit::Bracket {
		/// 		acquire: <BoxBrand as ToDynFnOnce>::new(|_: ()| {
		/// 			FreeExplicit::<IdentityBrand, _>::pure(7)
		/// 		}),
		/// 		body: <BoxBrand as ToDynFnOnce>::new(|_a: Box<i32>| {
		/// 			FreeExplicit::<IdentityBrand, _>::pure((7, 42))
		/// 		}),
		/// 		release: <BoxBrand as ToDynFnOnce>::new(|_a: Box<i32>| {
		/// 			FreeExplicit::<IdentityBrand, _>::pure(())
		/// 		}),
		/// 	};
		/// assert!(matches!(bracket, BoxBracketExplicit::Bracket { .. }));
		/// ```
		#[expect(
			clippy::unreachable,
			reason = "BracketExplicit cells require dispatcher-driven evaluation; substrate-required Extract is unreachable on real cells at runtime."
		)]
		fn extract<'a, X: 'a>(
			_fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, X>)
		) -> X {
			unreachable!(
				"BoxBracketExplicitBrand::extract invoked; BracketExplicit cells require dispatcher-driven evaluation"
			)
		}
	}

	#[document_type_parameters(
		"The substrate brand.",
		"The resource type.",
		"The body's result type."
	)]
	impl<Sub, A, B> Extract for BracketExplicitBrand<RcBrand, Sub, A, B>
	where
		Sub: WrapDrop + 'static,
		A: 'static,
		B: 'static,
	{
		/// Panicking stub; mirrors [`BoxBracketBrand`'s `Extract`].
		#[document_signature]
		///
		#[document_type_parameters("The lifetime.", "The result type.")]
		///
		#[document_parameters("The bracket-explicit effect (ignored).")]
		///
		#[document_returns("Never returns; panics with an unreachable! message.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		IdentityBrand,
		/// 		RcBrand,
		/// 	},
		/// 	classes::ToDynCloneFn,
		/// 	types::{
		/// 		RcFreeExplicit,
		/// 		effects::bracket::BracketExplicit,
		/// 	},
		/// };
		///
		/// let bracket: BracketExplicit<'static, RcBrand, IdentityBrand, i32, i32> =
		/// 	BracketExplicit::Bracket {
		/// 		acquire: <RcBrand as ToDynCloneFn>::new(|_: ()| {
		/// 			RcFreeExplicit::<IdentityBrand, _>::pure(7)
		/// 		}),
		/// 		body: <RcBrand as ToDynCloneFn>::new(|_a: std::rc::Rc<i32>| {
		/// 			RcFreeExplicit::<IdentityBrand, _>::pure((7, 42))
		/// 		}),
		/// 		release: <RcBrand as ToDynCloneFn>::new(|_a: std::rc::Rc<i32>| {
		/// 			RcFreeExplicit::<IdentityBrand, _>::pure(())
		/// 		}),
		/// 	};
		/// assert!(matches!(bracket, BracketExplicit::Bracket { .. }));
		/// ```
		#[expect(
			clippy::unreachable,
			reason = "BracketExplicit cells require dispatcher-driven evaluation; substrate-required Extract is unreachable on real cells at runtime."
		)]
		fn extract<'a, X: 'a>(
			_fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, X>)
		) -> X {
			unreachable!(
				"BracketExplicitBrand::extract invoked; BracketExplicit cells require dispatcher-driven evaluation"
			)
		}
	}

	#[document_type_parameters(
		"The substrate brand.",
		"The resource type (Send + Sync).",
		"The body's result type (Send + Sync)."
	)]
	impl<Sub, A, B> Extract for SendBracketExplicitBrand<ArcBrand, Sub, A, B>
	where
		Sub: WrapDrop + 'static,
		A: Send + Sync + 'static,
		B: Send + Sync + 'static,
	{
		/// Panicking stub; mirrors [`BoxBracketBrand`'s `Extract`].
		#[document_signature]
		///
		#[document_type_parameters("The lifetime.", "The result type.")]
		///
		#[document_parameters("The send-bracket-explicit effect (ignored).")]
		///
		#[document_returns("Never returns; panics with an unreachable! message.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		ArcBrand,
		/// 		IdentityBrand,
		/// 	},
		/// 	classes::ToDynSendFn,
		/// 	types::{
		/// 		ArcFreeExplicit,
		/// 		effects::bracket::SendBracketExplicit,
		/// 	},
		/// };
		///
		/// let bracket: SendBracketExplicit<'static, ArcBrand, IdentityBrand, i32, i32> =
		/// 	SendBracketExplicit::Bracket {
		/// 		acquire: <ArcBrand as ToDynSendFn>::new(|_: ()| {
		/// 			ArcFreeExplicit::<IdentityBrand, _>::pure(7)
		/// 		}),
		/// 		body: <ArcBrand as ToDynSendFn>::new(|_a: std::sync::Arc<i32>| {
		/// 			ArcFreeExplicit::<IdentityBrand, _>::pure((7, 42))
		/// 		}),
		/// 		release: <ArcBrand as ToDynSendFn>::new(|_a: std::sync::Arc<i32>| {
		/// 			ArcFreeExplicit::<IdentityBrand, _>::pure(())
		/// 		}),
		/// 	};
		/// assert!(matches!(bracket, SendBracketExplicit::Bracket { .. }));
		/// ```
		#[expect(
			clippy::unreachable,
			reason = "SendBracketExplicit cells require dispatcher-driven evaluation; substrate-required Extract is unreachable on real cells at runtime."
		)]
		fn extract<'a, X: 'a>(
			_fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, X>)
		) -> X {
			unreachable!(
				"SendBracketExplicitBrand::extract invoked; SendBracketExplicit cells require dispatcher-driven evaluation"
			)
		}
	}

	// ===== RefFunctor impls for Explicit-family brands =====

	#[document_type_parameters(
		"The substrate brand.",
		"The resource type.",
		"The body's result type."
	)]
	impl<Sub, A, B> RefFunctor for BoxBracketExplicitBrand<BoxBrand, Sub, A, B>
	where
		Sub: WrapDrop + 'static,
		A: 'static,
		B: 'static,
	{
		/// Stub-everywhere `ref_map`; mirrors [`BoxBracketBrand`'s
		/// `RefFunctor`]. `BoxBracketExplicit` is non-`Clone`; the
		/// returned cell is constructed with `unreachable!()` stub
		/// closures.
		#[document_signature]
		///
		#[document_type_parameters("The lifetime.", "The original type.", "The new type.")]
		///
		#[document_parameters(
			"The function (ignored).",
			"The bracket-explicit effect projection (ignored)."
		)]
		///
		#[document_returns(
			"A new bracket-explicit effect with all three closures as panicking stubs."
		)]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		BoxBracketExplicitBrand,
		/// 		BoxBrand,
		/// 		IdentityBrand,
		/// 	},
		/// 	classes::{
		/// 		RefFunctor,
		/// 		ToDynFnOnce,
		/// 	},
		/// 	types::{
		/// 		FreeExplicit,
		/// 		effects::bracket::BoxBracketExplicit,
		/// 	},
		/// };
		///
		/// let bracket: BoxBracketExplicit<'static, BoxBrand, IdentityBrand, i32, i32> =
		/// 	BoxBracketExplicit::Bracket {
		/// 		acquire: <BoxBrand as ToDynFnOnce>::new(|_: ()| {
		/// 			FreeExplicit::<IdentityBrand, _>::pure(7)
		/// 		}),
		/// 		body: <BoxBrand as ToDynFnOnce>::new(|_a: Box<i32>| {
		/// 			FreeExplicit::<IdentityBrand, _>::pure((7, 42))
		/// 		}),
		/// 		release: <BoxBrand as ToDynFnOnce>::new(|_a: Box<i32>| {
		/// 			FreeExplicit::<IdentityBrand, _>::pure(())
		/// 		}),
		/// 	};
		/// let mapped =
		/// 	<BoxBracketExplicitBrand<BoxBrand, IdentityBrand, i32, i32> as RefFunctor>::ref_map(
		/// 		|x: &i32| *x + 1,
		/// 		&bracket,
		/// 	);
		/// assert!(matches!(mapped, BoxBracketExplicit::Bracket { .. }));
		/// ```
		#[expect(
			clippy::unreachable,
			reason = "BoxBracketExplicitBrand::ref_map cannot replicate the cell from a reference because BoxBracketExplicit is non-Clone (Box<dyn FnOnce> is uncloneable). The path is reachable only through synthetic non-Coyoneda first-order rows on RunExplicit, which real programs do not exercise."
		)]
		fn ref_map<'a, X: 'a, Y: 'a>(
			_func: impl Fn(&X) -> Y + 'a,
			_fa: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, X>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Y>) {
			BoxBracketExplicit::Bracket {
				acquire: <BoxBrand as ToDynFnOnce>::new(|_: ()| -> FreeExplicit<'a, Sub, A> {
					unreachable!(
						"BoxBracketExplicitBrand::ref_map's stub acquire invoked; BoxBracketExplicit is non-Clone and the impl is reachable only through synthetic substrate paths"
					)
				}),
				body: <BoxBrand as ToDynFnOnce>::new(
					|_a: <BoxBrand as Pointer>::Of<'a, A>| -> FreeExplicit<'a, Sub, (A, B)> {
						unreachable!(
							"BoxBracketExplicitBrand::ref_map's stub body invoked; BoxBracketExplicit is non-Clone and the impl is reachable only through synthetic substrate paths"
						)
					},
				),
				release: <BoxBrand as ToDynFnOnce>::new(
					|_a: <BoxBrand as Pointer>::Of<'a, A>| -> FreeExplicit<'a, Sub, ()> {
						unreachable!(
							"BoxBracketExplicitBrand::ref_map's stub release invoked; BoxBracketExplicit is non-Clone and the impl is reachable only through synthetic substrate paths"
						)
					},
				),
			}
		}
	}

	#[document_type_parameters(
		"The substrate brand.",
		"The resource type.",
		"The body's result type."
	)]
	impl<Sub, A, B> RefFunctor for BracketExplicitBrand<RcBrand, Sub, A, B>
	where
		Sub: WrapDrop + 'static,
		A: 'static,
		B: 'static,
	{
		/// `Clone::clone(fa)`; mirrors [`BracketBrand`'s `RefFunctor`].
		#[document_signature]
		///
		#[document_type_parameters("The lifetime.", "The original type.", "The new type.")]
		///
		#[document_parameters("The function (ignored).", "The bracket-explicit effect projection.")]
		///
		#[document_returns(
			"A clone of the bracket-explicit effect (acquire / body / release Rc-bumped)."
		)]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		BracketExplicitBrand,
		/// 		IdentityBrand,
		/// 		RcBrand,
		/// 	},
		/// 	classes::{
		/// 		RefFunctor,
		/// 		ToDynCloneFn,
		/// 	},
		/// 	types::{
		/// 		RcFreeExplicit,
		/// 		effects::bracket::BracketExplicit,
		/// 	},
		/// };
		///
		/// let bracket: BracketExplicit<'static, RcBrand, IdentityBrand, i32, i32> =
		/// 	BracketExplicit::Bracket {
		/// 		acquire: <RcBrand as ToDynCloneFn>::new(|_: ()| {
		/// 			RcFreeExplicit::<IdentityBrand, _>::pure(7)
		/// 		}),
		/// 		body: <RcBrand as ToDynCloneFn>::new(|_a: std::rc::Rc<i32>| {
		/// 			RcFreeExplicit::<IdentityBrand, _>::pure((7, 42))
		/// 		}),
		/// 		release: <RcBrand as ToDynCloneFn>::new(|_a: std::rc::Rc<i32>| {
		/// 			RcFreeExplicit::<IdentityBrand, _>::pure(())
		/// 		}),
		/// 	};
		/// let mapped = <BracketExplicitBrand<RcBrand, IdentityBrand, i32, i32> as RefFunctor>::ref_map(
		/// 	|x: &i32| *x + 1,
		/// 	&bracket,
		/// );
		/// assert!(matches!(mapped, BracketExplicit::Bracket { .. }));
		/// ```
		fn ref_map<'a, X: 'a, Y: 'a>(
			_func: impl Fn(&X) -> Y + 'a,
			fa: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, X>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Y>) {
			fa.clone()
		}
	}
}

pub use inner::*;
