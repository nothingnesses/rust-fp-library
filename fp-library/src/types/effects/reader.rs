//! Environment-reading first-order effect type with the `Ask`
//! operation. The corresponding brand is
//! [`ReaderBrand`](crate::brands::ReaderBrand).
//!
//! `Reader<'a, P, E, A>` mirrors PureScript Run's
//! `Reader e a = Reader (e -> a)` shape directly. The single `Ask` variant carries a continuation
//! (`P::Of<'a, dyn Fn(E) -> A>`); the `Functor` instance composes a
//! user-supplied `f: A -> B` with the stored continuation to produce
//! a new `Reader<'a, P, E, B>`.
//!
//! ## Wrapper parameterization
//!
//! The pointer brand `P` (typically [`RcBrand`](crate::brands::RcBrand)
//! for single-thread substrates, [`ArcBrand`](crate::brands::ArcBrand)
//! for thread-safe substrates) is threaded through `Reader`'s
//! continuation via [`ToDynCloneFn`](crate::classes::ToDynCloneFn)
//! so a single `Reader` type works across all six Run wrappers.
//! Per-wrapper `ask` smart constructors thread the substrate-
//! appropriate `P`. The Arc family uses the parallel
//! [`SendReader`] /
//! [`SendReaderBrand`](crate::brands::SendReaderBrand) below to
//! sidestep the `Arc<dyn Fn>: !Send + !Sync` structural problem.

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			Apply,
			brands::{
				ReaderBrand,
				SendReaderBrand,
			},
			classes::{
				Functor,
				RefCountedPointer,
				SendFunctor,
				SendRefCountedPointer,
				ToDynCloneFn,
				ToDynSendFn,
			},
			impl_kind,
			kinds::*,
		},
		fp_macros::*,
	};

	/// Environment-reading first-order effect type.
	///
	/// The single `Ask` variant carries a continuation in `P`'s
	/// pointer kind (`Rc<dyn Fn(E) -> A>` for
	/// [`RcBrand`](crate::brands::RcBrand);
	/// `Arc<dyn Fn(E) -> A + Send + Sync>` for
	/// [`ArcBrand`](crate::brands::ArcBrand)). Mirrors PureScript Run's `Reader e a`.
	#[document_type_parameters(
		"The lifetime of the continuation and any references it captures.",
		"The pointer brand used for the continuation (typically [`RcBrand`](crate::brands::RcBrand) or [`ArcBrand`](crate::brands::ArcBrand)).",
		"The environment type.",
		"The result type produced by running the effect."
	)]
	pub enum Reader<'a, P, E, A>
	where
		P: ToDynCloneFn,
		E: 'a,
		A: 'a, {
		/// Read the immutable environment. The continuation is
		/// applied to the current environment value to produce the
		/// result `A`.
		Ask(<P as RefCountedPointer>::Of<'a, dyn 'a + Fn(E) -> A>),
	}

	impl_kind! {
		impl<P: ToDynCloneFn, E: 'static> for ReaderBrand<P, E> {
			type Of<'a, A: 'a>: 'a = Reader<'a, P, E, A>;
		}
	}

	#[document_type_parameters(
		"The lifetime of the continuation.",
		"The pointer brand used for the continuation.",
		"The environment type.",
		"The result type."
	)]
	#[document_parameters("The reader effect to clone.")]
	impl<'a, P, E, A> Clone for Reader<'a, P, E, A>
	where
		P: ToDynCloneFn,
		E: 'a,
		A: 'a,
	{
		/// Clones the reader effect by refcount-bumping the stored
		/// continuation pointer. The continuation pointer is
		/// `<P as RefCountedPointer>::Of<...>`, which is
		/// unconditionally [`Clone`] per the trait's associated-type
		/// bound.
		#[document_signature]
		///
		#[document_returns("A new reader effect sharing the continuation by refcount.")]
		///
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	fp_library::{
		/// 		brands::*,
		/// 		types::effects::reader::Reader,
		/// 	},
		/// 	std::rc::Rc,
		/// };
		///
		/// let original: Reader<'static, RcBrand, i32, i32> =
		/// 	Reader::Ask(Rc::new(|e: i32| e + 1) as Rc<dyn Fn(i32) -> i32>);
		/// let cloned = original.clone();
		/// match cloned {
		/// 	Reader::Ask(k) => assert_eq!(k(7), 8),
		/// }
		/// ```
		fn clone(&self) -> Self {
			match self {
				Reader::Ask(k) => Reader::Ask(k.clone()),
			}
		}
	}

	#[document_type_parameters(
		"The pointer brand used for the continuation.",
		"The environment type."
	)]
	impl<P, E> Functor for ReaderBrand<P, E>
	where
		P: ToDynCloneFn,
		E: 'static,
	{
		/// Maps `f` over the result type of this reader effect.
		///
		/// Composes `f` with the stored continuation. The pointer
		/// kind `P` is preserved; the new continuation is
		/// constructed via [`ToDynCloneFn::new`].
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
			"The reader effect to map over."
		)]
		///
		#[document_returns("A new reader effect with `f` composed onto the continuation.")]
		///
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	fp_library::{
		/// 		brands::*,
		/// 		classes::*,
		/// 		types::effects::reader::Reader,
		/// 	},
		/// 	std::rc::Rc,
		/// };
		///
		/// // Build an `Ask` continuation that doubles the environment,
		/// // then map `+1` over it. After mapping, the continuation
		/// // produces `2 * e + 1`.
		/// let ask: Reader<'static, RcBrand, i32, i32> =
		/// 	Reader::Ask(Rc::new(|e: i32| e * 2) as Rc<dyn Fn(i32) -> i32>);
		/// let mapped = <ReaderBrand<RcBrand, i32> as Functor>::map(|x: i32| x + 1, ask);
		/// match mapped {
		/// 	Reader::Ask(k) => assert_eq!(k(3), 7),
		/// }
		/// ```
		fn map<'a, A: 'a, B: 'a>(
			f: impl Fn(A) -> B + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			match fa {
				Reader::Ask(k) => Reader::Ask(<P as ToDynCloneFn>::new(move |e: E| f((*k)(e)))),
			}
		}
	}

	// SendFunctor on ReaderBrand cannot ship: the projection
	// <P as RefCountedPointer>::Of<'_, dyn 'a + Fn(E) -> A>
	// is structurally !Send + !Sync because the trait object's bounds
	// don't include + Send + Sync. The Send-aware sibling
	// SendReaderBrand below uses
	// <P as SendRefCountedPointer>::Of<'_, dyn 'a + Fn(...) + Send + Sync>
	// to sidestep this; the Arc family smart constructors target
	// SendReaderBrand (parallel to SendStateBrand for State).

	/// Thread-safe sibling of [`Reader`] with `Send + Sync`-bounded
	/// continuation trait objects.
	///
	/// The `Ask` variant carries a continuation in `P`'s Send-aware
	/// pointer kind
	/// (`Arc<dyn Fn(E) -> A + Send + Sync>` for
	/// [`ArcBrand`](crate::brands::ArcBrand)). Mirrors PureScript Run's
	/// `Reader e a` shape directly, with the marker traits baked into
	/// the trait object's bounds so the projection is structurally
	/// `Send + Sync`.
	///
	/// Used by the Arc family `ask` smart constructors
	/// ([`ArcRun::ask`](crate::types::effects::arc_run::ArcRun) /
	/// [`ArcRunExplicit::ask`](crate::types::effects::arc_run_explicit::ArcRunExplicit));
	/// non-Arc smart constructors keep using [`Reader`].
	#[document_type_parameters(
		"The lifetime of the continuation and any references it captures.",
		"The pointer brand used for the continuation (typically [`ArcBrand`](crate::brands::ArcBrand)).",
		"The environment type.",
		"The result type produced by running the effect."
	)]
	pub enum SendReader<'a, P, E, A>
	where
		P: ToDynSendFn,
		E: 'a,
		A: 'a, {
		/// Read the immutable environment. The continuation is
		/// applied to the current environment value to produce the
		/// result `A`.
		Ask(<P as SendRefCountedPointer>::Of<'a, dyn 'a + Fn(E) -> A + Send + Sync>),
	}

	impl_kind! {
		impl<P: ToDynSendFn, E: 'static> for SendReaderBrand<P, E> {
			type Of<'a, A: 'a>: 'a = SendReader<'a, P, E, A>;
		}
	}

	#[document_type_parameters(
		"The lifetime of the continuation.",
		"The pointer brand used for the continuation.",
		"The environment type.",
		"The result type."
	)]
	#[document_parameters("The send-reader effect to clone.")]
	impl<'a, P, E, A> Clone for SendReader<'a, P, E, A>
	where
		P: ToDynSendFn,
		E: 'a,
		A: 'a,
	{
		/// Clones the send-reader effect by refcount-bumping the
		/// stored continuation pointer. The continuation pointer is
		/// `<P as SendRefCountedPointer>::Of<...>`, which is
		/// unconditionally [`Clone`] per the trait's associated-type
		/// bound.
		#[document_signature]
		///
		#[document_returns("A new send-reader effect sharing the continuation by refcount.")]
		///
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	fp_library::{
		/// 		brands::*,
		/// 		types::effects::reader::SendReader,
		/// 	},
		/// 	std::sync::Arc,
		/// };
		///
		/// let original: SendReader<'static, ArcBrand, i32, i32> =
		/// 	SendReader::Ask(Arc::new(|e: i32| e + 1) as Arc<dyn Fn(i32) -> i32 + Send + Sync>);
		/// let cloned = original.clone();
		/// match cloned {
		/// 	SendReader::Ask(k) => assert_eq!(k(7), 8),
		/// }
		/// ```
		fn clone(&self) -> Self {
			match self {
				SendReader::Ask(k) => SendReader::Ask(k.clone()),
			}
		}
	}

	// Functor (by-value `map`) is intentionally NOT implemented for
	// SendReaderBrand: the new continuation must be storable behind
	// <P as SendRefCountedPointer>::Of<'_, dyn Fn + Send + Sync>, but
	// Functor::map's signature only requires `f: Fn` (no Send + Sync),
	// so a Functor impl could not construct a Send-aware trait object.
	// SendFunctor is independent of Functor in fp-library (not a
	// supertrait), so this is sound; users reach for
	// SendFunctor::send_map directly.

	#[document_type_parameters(
		"The pointer brand used for the continuation.",
		"The environment type."
	)]
	impl<P, E> SendFunctor for SendReaderBrand<P, E>
	where
		P: ToDynSendFn,
		E: Send + Sync + 'static,
	{
		/// Maps `f` over the result type of this thread-safe
		/// reader effect, with `Send + Sync` bounds on `f` and
		/// the result types so the new continuation can be stored
		/// behind a thread-safe trait object.
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
			"The send-reader effect to map over."
		)]
		///
		#[document_returns("A new send-reader effect with `f` composed onto the continuation.")]
		///
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	fp_library::{
		/// 		brands::*,
		/// 		classes::*,
		/// 		types::effects::reader::SendReader,
		/// 	},
		/// 	std::sync::Arc,
		/// };
		///
		/// let ask: SendReader<'static, ArcBrand, i32, i32> =
		/// 	SendReader::Ask(Arc::new(|e: i32| e * 2) as Arc<dyn Fn(i32) -> i32 + Send + Sync>);
		/// let mapped = <SendReaderBrand<ArcBrand, i32> as SendFunctor>::send_map(|x: i32| x + 1, ask);
		/// match mapped {
		/// 	SendReader::Ask(k) => assert_eq!(k(3), 7),
		/// }
		/// ```
		fn send_map<'a, A: Send + Sync + 'a, B: Send + Sync + 'a>(
			f: impl Fn(A) -> B + Send + Sync + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			match fa {
				SendReader::Ask(k) =>
					SendReader::Ask(<P as ToDynSendFn>::new(move |e: E| f((*k)(e)))),
			}
		}
	}
}

pub use inner::*;
