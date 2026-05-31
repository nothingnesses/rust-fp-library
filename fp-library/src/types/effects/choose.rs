//! Nondeterministic-branching first-order effect type with the
//! `Alt` operation. The corresponding brand is
//! [`ChooseBrand`](crate::brands::ChooseBrand).
//!
//! `Choose<'a, P, A>` mirrors PureScript Run's
//! `Choose a = Alt (Boolean -> a)` shape directly. The single
//! `Alt` variant carries a continuation `bool -> A`; the `Functor`
//! instance composes a user-supplied `f: A -> B` with the stored
//! continuation to produce a new `Choose<'a, P, B>`.
//!
//! ## Single-shot wrappers cannot host this effect
//!
//! A `Choose` handler runs the continuation twice (once for the
//! `true` branch, once for the `false` branch) to capture the
//! nondeterministic semantics, which requires the underlying
//! continuation pointer to be cloneable. The single-shot wrappers
//! [`Run`](crate::types::effects::run::Run) and
//! [`RunExplicit`](crate::types::effects::run_explicit::RunExplicit)
//! consume their continuations once; only the four multi-shot
//! wrappers ([`RcRun`](crate::types::effects::rc_run::RcRun) /
//! [`RcRunExplicit`](crate::types::effects::rc_run_explicit::RcRunExplicit) /
//! [`ArcRun`](crate::types::effects::arc_run::ArcRun) /
//! [`ArcRunExplicit`](crate::types::effects::arc_run_explicit::ArcRunExplicit))
//! ship the `choose` smart constructor.
//!
//! ## Wrapper parameterization
//!
//! The pointer brand `P` (typically [`RcBrand`](crate::brands::RcBrand)
//! for single-thread substrates, [`ArcBrand`](crate::brands::ArcBrand)
//! for thread-safe substrates) is threaded through `Choose`'s
//! continuation via [`ToDynCloneFn`](crate::classes::ToDynCloneFn)
//! so a single `Choose` type works across the four multi-shot
//! Run wrappers. The Arc family uses the parallel
//! [`SendChoose`] / [`SendChooseBrand`](crate::brands::SendChooseBrand)
//! below to sidestep the `Arc<dyn Fn>: !Send + !Sync` structural
//! problem.
//!
//! See the parent [`effects`](crate::types::effects) guide for the
//! consolidated wrapper and pointer-brand conventions.

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			Apply,
			brands::{
				BoxBrand,
				BoxChooseBrand,
				ChooseBrand,
				SendChooseBrand,
			},
			classes::{
				Functor,
				Pointer,
				RefCountedPointer,
				SendFunctor,
				SendRefCountedPointer,
				ToDynCloneFn,
				ToDynFnOnce,
				ToDynSendFn,
			},
			impl_kind,
			kinds::*,
		},
		fp_macros::*,
	};

	/// Nondeterministic-branching first-order effect type.
	///
	/// The single `Alt` variant carries a continuation in `P`'s
	/// pointer kind (`Rc<dyn Fn(bool) -> A>` for
	/// [`RcBrand`](crate::brands::RcBrand)). Mirrors PureScript
	/// Run's `Choose a`.
	#[document_type_parameters(
		"The lifetime of the continuation and any references it captures.",
		"The pointer brand used for the continuation (typically [`RcBrand`](crate::brands::RcBrand)).",
		"The result type produced by running the effect."
	)]
	pub enum Choose<'a, P, A>
	where
		P: ToDynCloneFn,
		A: 'a, {
		/// Run the continuation for both `true` and `false` branches.
		/// The handler invokes the continuation twice (once per
		/// branch) and combines the two results per its semantics.
		Alt(<P as RefCountedPointer>::Of<'a, dyn 'a + Fn(bool) -> A>),
	}

	impl_kind! {
		impl<P: ToDynCloneFn> for ChooseBrand<P> {
			type Of<'a, A: 'a>: 'a = Choose<'a, P, A>;
		}
	}

	#[document_type_parameters(
		"The lifetime of the continuation.",
		"The pointer brand used for the continuation.",
		"The result type."
	)]
	#[document_parameters("The choose effect to clone.")]
	impl<'a, P, A> Clone for Choose<'a, P, A>
	where
		P: ToDynCloneFn,
		A: 'a,
	{
		/// Clones the choose effect by refcount-bumping the stored
		/// continuation pointer.
		#[document_signature]
		///
		#[document_returns("A new choose effect sharing the continuation by refcount.")]
		///
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	fp_library::{
		/// 		brands::*,
		/// 		types::effects::choose::Choose,
		/// 	},
		/// 	std::rc::Rc,
		/// };
		///
		/// let original: Choose<'static, RcBrand, i32> =
		/// 	Choose::Alt(Rc::new(|b: bool| if b { 1 } else { 0 }) as Rc<dyn Fn(bool) -> i32>);
		/// let cloned = original.clone();
		/// match cloned {
		/// 	Choose::Alt(k) => {
		/// 		assert_eq!(k(true), 1);
		/// 		assert_eq!(k(false), 0);
		/// 	}
		/// }
		/// ```
		fn clone(&self) -> Self {
			match self {
				Choose::Alt(k) => Choose::Alt(k.clone()),
			}
		}
	}

	#[document_type_parameters("The pointer brand used for the continuation.")]
	impl<P> Functor for ChooseBrand<P>
	where
		P: ToDynCloneFn,
	{
		/// Maps `f` over the result type of this choose effect.
		///
		/// Composes `f` with the stored continuation; the new
		/// continuation is constructed via [`ToDynCloneFn::new`].
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the continuation.",
			"The original result type.",
			"The new result type after applying `f`."
		)]
		///
		#[document_parameters(
			"The function to compose with the continuation.",
			"The choose effect to map over."
		)]
		///
		#[document_returns("A new choose effect with `f` composed onto the continuation.")]
		///
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	fp_library::{
		/// 		brands::*,
		/// 		classes::*,
		/// 		types::effects::choose::Choose,
		/// 	},
		/// 	std::rc::Rc,
		/// };
		///
		/// let alt: Choose<'static, RcBrand, i32> =
		/// 	Choose::Alt(Rc::new(|b: bool| if b { 10 } else { 20 }) as Rc<dyn Fn(bool) -> i32>);
		/// let mapped = <ChooseBrand<RcBrand> as Functor>::map(|x: i32| x + 1, alt);
		/// match mapped {
		/// 	Choose::Alt(k) => {
		/// 		assert_eq!(k(true), 11);
		/// 		assert_eq!(k(false), 21);
		/// 	}
		/// }
		/// ```
		fn map<'a, A: 'a, B: 'a>(
			f: impl Fn(A) -> B + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			match fa {
				Choose::Alt(k) => Choose::Alt(<P as ToDynCloneFn>::new(move |b: bool| f((*k)(b)))),
			}
		}
	}

	// SendFunctor on ChooseBrand cannot ship: the projection
	// <P as RefCountedPointer>::Of<'_, dyn 'a + Fn(bool) -> A>
	// is structurally !Send + !Sync because the trait object's bounds
	// don't include + Send + Sync. The Send-aware sibling
	// SendChooseBrand below uses
	// <P as SendRefCountedPointer>::Of<'_, dyn 'a + Fn(bool) -> A + Send + Sync>
	// to sidestep this.

	/// Thread-safe sibling of [`Choose`] with `Send + Sync`-bounded
	/// continuation trait objects.
	///
	/// The `Alt` variant carries a continuation in `P`'s Send-aware
	/// pointer kind (`Arc<dyn Fn(bool) -> A + Send + Sync>` for
	/// [`ArcBrand`](crate::brands::ArcBrand)). Mirrors PureScript
	/// Run's `Choose a` shape directly, with the marker traits baked
	/// into the trait object's bounds so the projection is
	/// structurally `Send + Sync`.
	///
	/// Used by the Arc family `choose` smart constructors
	/// ([`ArcRun::choose`](crate::types::effects::arc_run::ArcRun) /
	/// [`ArcRunExplicit::choose`](crate::types::effects::arc_run_explicit::ArcRunExplicit));
	/// non-Arc smart constructors keep using [`Choose`].
	#[document_type_parameters(
		"The lifetime of the continuation and any references it captures.",
		"The pointer brand used for the continuation (typically [`ArcBrand`](crate::brands::ArcBrand)).",
		"The result type produced by running the effect."
	)]
	pub enum SendChoose<'a, P, A>
	where
		P: ToDynSendFn,
		A: 'a, {
		/// Run the continuation for both `true` and `false` branches
		/// (thread-safe variant). Handler invokes the continuation
		/// twice (once per branch).
		Alt(<P as SendRefCountedPointer>::Of<'a, dyn 'a + Fn(bool) -> A + Send + Sync>),
	}

	impl_kind! {
		impl<P: ToDynSendFn> for SendChooseBrand<P> {
			type Of<'a, A: 'a>: 'a = SendChoose<'a, P, A>;
		}
	}

	#[document_type_parameters(
		"The lifetime of the continuation.",
		"The pointer brand used for the continuation.",
		"The result type."
	)]
	#[document_parameters("The send-choose effect to clone.")]
	impl<'a, P, A> Clone for SendChoose<'a, P, A>
	where
		P: ToDynSendFn,
		A: 'a,
	{
		/// Clones the send-choose effect by refcount-bumping the
		/// stored continuation pointer.
		#[document_signature]
		///
		#[document_returns("A new send-choose effect sharing the continuation by refcount.")]
		///
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	fp_library::{
		/// 		brands::*,
		/// 		types::effects::choose::SendChoose,
		/// 	},
		/// 	std::sync::Arc,
		/// };
		///
		/// let original: SendChoose<'static, ArcBrand, i32> =
		/// 	SendChoose::Alt(
		/// 		Arc::new(|b: bool| if b { 1 } else { 0 }) as Arc<dyn Fn(bool) -> i32 + Send + Sync>
		/// 	);
		/// let cloned = original.clone();
		/// match cloned {
		/// 	SendChoose::Alt(k) => {
		/// 		assert_eq!(k(true), 1);
		/// 		assert_eq!(k(false), 0);
		/// 	}
		/// }
		/// ```
		fn clone(&self) -> Self {
			match self {
				SendChoose::Alt(k) => SendChoose::Alt(k.clone()),
			}
		}
	}

	// Functor (by-value `map`) is intentionally NOT implemented for
	// SendChooseBrand: the new continuation must be storable behind
	// <P as SendRefCountedPointer>::Of<'_, dyn Fn + Send + Sync>, but
	// Functor::map's signature only requires `f: Fn` (no Send + Sync),
	// so a Functor impl could not construct a Send-aware trait object.
	// SendFunctor is independent of Functor in fp-library (not a
	// supertrait), so this is sound; users reach for
	// SendFunctor::send_map directly.

	#[document_type_parameters("The pointer brand used for the continuation.")]
	impl<P> SendFunctor for SendChooseBrand<P>
	where
		P: ToDynSendFn,
	{
		/// Maps `f` over the result type of this thread-safe choose
		/// effect, with `Send + Sync` bounds on `f` and the result
		/// types so the new continuation can be stored behind a
		/// thread-safe trait object.
		///
		/// Composes `f` with the stored continuation via
		/// [`ToDynSendFn::new`].
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the continuation.",
			"The original result type.",
			"The new result type after applying `f`."
		)]
		///
		#[document_parameters(
			"The function to compose with the continuation.",
			"The send-choose effect to map over."
		)]
		///
		#[document_returns("A new send-choose effect with `f` composed onto the continuation.")]
		///
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	fp_library::{
		/// 		brands::*,
		/// 		classes::*,
		/// 		types::effects::choose::SendChoose,
		/// 	},
		/// 	std::sync::Arc,
		/// };
		///
		/// let alt: SendChoose<'static, ArcBrand, i32> = SendChoose::Alt(Arc::new(
		/// 	|b: bool| if b { 10 } else { 20 },
		/// )
		/// 	as Arc<dyn Fn(bool) -> i32 + Send + Sync>);
		/// let mapped = <SendChooseBrand<ArcBrand> as SendFunctor>::send_map(|x: i32| x + 1, alt);
		/// match mapped {
		/// 	SendChoose::Alt(k) => {
		/// 		assert_eq!(k(true), 11);
		/// 		assert_eq!(k(false), 21);
		/// 	}
		/// }
		/// ```
		fn send_map<'a, A: Send + Sync + 'a, B: Send + Sync + 'a>(
			f: impl Fn(A) -> B + Send + Sync + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			match fa {
				SendChoose::Alt(k) =>
					SendChoose::Alt(<P as ToDynSendFn>::new(move |b: bool| f((*k)(b)))),
			}
		}
	}

	/// Single-shot sibling of [`Choose`] with `dyn FnOnce`-bounded
	/// continuation trait objects. Defined for substrate
	/// uniformity but not exposed via any smart constructor.
	///
	/// `Choose` ships only on the four multi-shot wrappers because a
	/// `Choose` handler runs the continuation twice (once per branch),
	/// which a single-shot `dyn FnOnce` continuation cannot host.
	/// `BoxChoose` exists so that the substrate trait family
	/// (`Choose` / `SendChoose` / `BoxChoose` parallel to
	/// `State` / `SendState` / `BoxState` and
	/// `Reader` / `SendReader` / `BoxReader`) is structurally
	/// uniform; no [`Run::choose`](crate::types::effects::run::Run)
	/// or
	/// [`RunExplicit::choose`](crate::types::effects::run_explicit::RunExplicit)
	/// smart constructor exists, so the type is reachable only
	/// from user code.
	#[document_type_parameters(
		"The lifetime of the continuation and any references it captures.",
		"The pointer brand used for the continuation (necessarily [`BoxBrand`](crate::brands::BoxBrand) since that is the only brand implementing [`ToDynFnOnce`](crate::classes::ToDynFnOnce)).",
		"The result type produced by running the effect."
	)]
	pub enum BoxChoose<'a, P, A>
	where
		P: ToDynFnOnce,
		A: 'a, {
		/// Run the continuation once for the chosen branch.
		/// Unlike [`Choose::Alt`], this single-shot variant
		/// invokes the continuation at most once.
		Alt(<P as Pointer>::Of<'a, dyn 'a + FnOnce(bool) -> A>),
	}

	impl_kind! {
		impl<P: ToDynFnOnce> for BoxChooseBrand<P> {
			type Of<'a, A: 'a>: 'a = BoxChoose<'a, P, A>;
		}
	}

	// No Clone impl: Box<dyn FnOnce> is not Clone.
	// No SendFunctor impl: BoxBrand is single-thread by design.

	impl Functor for BoxChooseBrand<BoxBrand> {
		/// Maps `f` over the result type of this single-shot
		/// choose effect.
		///
		/// Composes `f` with the stored continuation. The new
		/// continuation is constructed via
		/// [`ToDynFnOnce::new`]. Specialised to
		/// [`BoxBrand`](crate::brands::BoxBrand) for the same
		/// reason
		/// [`BoxStateBrand`](crate::brands::BoxStateBrand)'s
		/// [`Functor`](crate::classes::Functor) is specialised:
		/// `Box<dyn FnOnce(...)>` itself implements
		/// [`FnOnce`] via the standard library's
		/// blanket impl on [`Box`].
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the continuation.",
			"The original result type.",
			"The new result type after applying `f`."
		)]
		///
		#[document_parameters(
			"The function to compose with the continuation.",
			"The choose effect to map over."
		)]
		///
		#[document_returns("A new choose effect with `f` composed onto the continuation.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::*,
		/// 	types::effects::choose::BoxChoose,
		/// };
		///
		/// let alt: BoxChoose<'static, BoxBrand, i32> =
		/// 	BoxChoose::Alt(Box::new(|b: bool| if b { 10 } else { 20 }) as Box<dyn FnOnce(bool) -> i32>);
		/// let mapped = <BoxChooseBrand<BoxBrand> as Functor>::map(|x: i32| x + 1, alt);
		/// match mapped {
		/// 	BoxChoose::Alt(k) => assert_eq!(k(true), 11),
		/// }
		/// ```
		fn map<'a, A: 'a, B: 'a>(
			f: impl Fn(A) -> B + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			match fa {
				BoxChoose::Alt(k) =>
					BoxChoose::Alt(<BoxBrand as ToDynFnOnce>::new(move |b: bool| f(k(b)))),
			}
		}
	}
}

pub use inner::*;
