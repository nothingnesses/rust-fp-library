//! Scoped error-recovery effect type with `Catch` (run an action
//! and recover from a thrown error of type `E`) as its sole
//! operation. The corresponding brands are
//! [`BoxCatchBrand`](crate::brands::BoxCatchBrand),
//! [`CatchBrand`](crate::brands::CatchBrand), and
//! [`SendCatchBrand`](crate::brands::SendCatchBrand).
//!
//! Mirrors heftia's
//! [`Catch`](https://github.com/sayo-hs/heftia/blob/master/heftia-effects/src/Control/Monad/Hefty/Except.hs)
//! and PureScript Run's `Run.Except.catch`. The recovery handler
//! is invoked at most once when the action throws; the action and
//! the recovery program both have the same row signature `A`
//! (loose notation for "next program in the same row" per the
//! [B1 resolution](../../../../docs/plans/effects/resolutions.md)).
//!
//! ## Three sibling types
//!
//! Per the Phase 4 step 3.1 design (mirroring Phase 3.5's
//! [`BoxState`](crate::types::effects::state::BoxState) /
//! [`State`](crate::types::effects::state::State) /
//! [`SendState`](crate::types::effects::state::SendState) split),
//! `Catch` ships in three sibling flavours that thread different
//! per-pointer-brand closure-storage shapes:
//!
//! - [`BoxCatch<'a, P, E, A>`] for default `Run` / `RunExplicit`,
//!   `P: ToDynFnOnce` (`BoxBrand` only), recovery handler is
//!   `Box<dyn 'a + FnOnce(E) -> A>` (single-shot).
//! - [`Catch<'a, P, E, A>`] for `RcRun` / `RcRunExplicit`,
//!   `P: ToDynCloneFn` (typically `RcBrand`), recovery handler is
//!   `Rc<dyn 'a + Fn(E) -> A>` (clone-able).
//! - [`SendCatch<'a, P, E, A>`] for `ArcRun` / `ArcRunExplicit`,
//!   `P: ToDynSendFn` (typically `ArcBrand`), recovery handler is
//!   `Arc<dyn 'a + Fn(E) -> A + Send + Sync>` (clone-able and
//!   thread-safe).

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			Apply,
			brands::{
				ArcBrand,
				BoxBrand,
				BoxCatchBrand,
				CatchBrand,
				RcBrand,
				SendCatchBrand,
			},
			classes::{
				Extract,
				Functor,
				Pointer,
				RefCountedPointer,
				SendFunctor,
				SendRefCountedPointer,
				ToDynCloneFn,
				ToDynFnOnce,
				ToDynSendFn,
				WrapDrop,
			},
			impl_kind,
			kinds::*,
		},
		fp_macros::*,
	};

	// ===== BoxCatch (BoxBrand + FnOnce, single-shot) =====

	/// Scoped error-recovery effect for default `Run` / `RunExplicit`
	/// substrates. The recovery handler is stored as
	/// `<P as Pointer>::Of<'a, dyn 'a + FnOnce(E) -> A>` (a `Box<dyn
	/// FnOnce>` projection by structural bound).
	#[document_type_parameters(
		"The lifetime of the recovery handler closure.",
		"The pointer brand storing the recovery handler (BoxBrand only by structural bound).",
		"The error type recovered from.",
		"The result type of the action and recovery."
	)]
	pub enum BoxCatch<'a, P, E, A>
	where
		P: ToDynFnOnce,
		E: 'a,
		A: 'a, {
		/// Run the `action` program; if it throws an `E`, invoke
		/// `handler` with the error to produce a recovery program.
		Catch {
			/// The protected action program.
			action: A,
			/// The recovery handler invoked on a thrown error.
			handler: <P as Pointer>::Of<'a, dyn 'a + FnOnce(E) -> A>,
		},
	}

	impl_kind! {
		impl<P: ToDynFnOnce, E: 'static> for BoxCatchBrand<P, E> {
			type Of<'a, A: 'a>: 'a = BoxCatch<'a, P, E, A>;
		}
	}

	#[document_type_parameters("The error type recovered from.")]
	impl<E> Functor for BoxCatchBrand<BoxBrand, E>
	where
		E: 'static,
	{
		/// Maps `f` over the result type of this scoped recovery
		/// effect. Composes `f` with the action and post-applies `f`
		/// to the recovery handler's return.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the continuations.",
			"The original result type.",
			"The new result type after applying `f`."
		)]
		///
		#[document_parameters("The function to apply.", "The catch effect.")]
		///
		#[document_returns("A new catch effect with `f` composed over its result type.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		BoxBrand,
		/// 		BoxCatchBrand,
		/// 	},
		/// 	classes::{
		/// 		Functor,
		/// 		ToDynFnOnce,
		/// 	},
		/// 	types::effects::catch::BoxCatch,
		/// };
		///
		/// let catch: BoxCatch<'static, BoxBrand, &'static str, i32> = BoxCatch::Catch {
		/// 	action: 7,
		/// 	handler: <BoxBrand as ToDynFnOnce>::new(|_e: &'static str| 100),
		/// };
		/// let mapped = <BoxCatchBrand<BoxBrand, &'static str> as Functor>::map(|x: i32| x + 1, catch);
		/// match mapped {
		/// 	BoxCatch::Catch {
		/// 		action,
		/// 		handler,
		/// 	} => {
		/// 		assert_eq!(action, 8);
		/// 		assert_eq!(handler("oops"), 101);
		/// 	}
		/// }
		/// ```
		fn map<'a, A: 'a, B: 'a>(
			f: impl Fn(A) -> B + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			match fa {
				BoxCatch::Catch {
					action,
					handler,
				} => BoxCatch::Catch {
					action: f(action),
					handler: <BoxBrand as ToDynFnOnce>::new(move |e: E| f(handler(e))),
				},
			}
		}
	}

	// ===== Catch (RcBrand + Fn, multi-shot clone-capable) =====

	/// Scoped error-recovery effect for `RcRun` / `RcRunExplicit`
	/// substrates. The recovery handler is stored as
	/// `<P as RefCountedPointer>::Of<'a, dyn 'a + Fn(E) -> A>` (an
	/// `Rc<dyn Fn>` projection), which is clone-able along with the
	/// surrounding multi-shot program.
	#[document_type_parameters(
		"The lifetime of the recovery handler closure.",
		"The pointer brand storing the recovery handler (RcBrand only).",
		"The error type recovered from.",
		"The result type of the action and recovery."
	)]
	pub enum Catch<'a, P, E, A>
	where
		P: ToDynCloneFn,
		E: 'a,
		A: 'a, {
		/// Run the `action` program; if it throws an `E`, invoke
		/// `handler` with the error to produce a recovery program.
		Catch {
			/// The protected action program.
			action: A,
			/// The recovery handler invoked on a thrown error.
			handler: <P as RefCountedPointer>::Of<'a, dyn 'a + Fn(E) -> A>,
		},
	}

	impl_kind! {
		impl<P: ToDynCloneFn, E: 'static> for CatchBrand<P, E> {
			type Of<'a, A: 'a>: 'a = Catch<'a, P, E, A>;
		}
	}

	#[document_type_parameters("The error type recovered from.")]
	impl<E> Functor for CatchBrand<RcBrand, E>
	where
		E: 'static,
	{
		/// Maps `f` over the result type of this scoped recovery
		/// effect. Specialised to `RcBrand` so the projection's
		/// `dyn Fn` closure trait permits `f`-post-composition into
		/// a new `Rc<dyn Fn>` cell. Mirrors Phase 3.5's
		/// [`StateBrand::map`](crate::types::effects::state::State).
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the continuations.",
			"The original result type.",
			"The new result type after applying `f`."
		)]
		///
		#[document_parameters("The function to apply.", "The catch effect.")]
		///
		#[document_returns("A new catch effect with `f` composed over its result type.")]
		///
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	core::ops::Deref,
		/// 	fp_library::{
		/// 		brands::{
		/// 			CatchBrand,
		/// 			RcBrand,
		/// 		},
		/// 		classes::{
		/// 			Functor,
		/// 			ToDynCloneFn,
		/// 		},
		/// 		types::effects::catch::Catch,
		/// 	},
		/// };
		///
		/// let catch: Catch<'static, RcBrand, &'static str, i32> = Catch::Catch {
		/// 	action: 7,
		/// 	handler: <RcBrand as ToDynCloneFn>::new(|_e: &'static str| 100),
		/// };
		/// let mapped = <CatchBrand<RcBrand, &'static str> as Functor>::map(|x: i32| x + 1, catch);
		/// match mapped {
		/// 	Catch::Catch {
		/// 		action,
		/// 		handler,
		/// 	} => {
		/// 		assert_eq!(action, 8);
		/// 		assert_eq!(handler.deref()("oops"), 101);
		/// 	}
		/// }
		/// ```
		fn map<'a, A: 'a, B: 'a>(
			f: impl Fn(A) -> B + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			match fa {
				Catch::Catch {
					action,
					handler,
				} => Catch::Catch {
					action: f(action),
					handler: <RcBrand as ToDynCloneFn>::new(move |e: E| f(handler(e))),
				},
			}
		}
	}

	// ===== SendCatch (ArcBrand + Fn + Send + Sync, thread-safe) =====

	/// Scoped error-recovery effect for `ArcRun` / `ArcRunExplicit`
	/// substrates. The recovery handler is stored as
	/// `<P as SendRefCountedPointer>::Of<'a, dyn 'a + Fn(E) -> A + Send + Sync>`
	/// (an `Arc<dyn Fn + Send + Sync>` projection), which is
	/// thread-safe and clone-able.
	#[document_type_parameters(
		"The lifetime of the recovery handler closure.",
		"The pointer brand storing the recovery handler (ArcBrand only).",
		"The error type recovered from.",
		"The result type of the action and recovery."
	)]
	pub enum SendCatch<'a, P, E, A>
	where
		P: ToDynSendFn,
		E: 'a,
		A: 'a, {
		/// Run the `action` program; if it throws an `E`, invoke
		/// `handler` with the error to produce a recovery program.
		Catch {
			/// The protected action program.
			action: A,
			/// The recovery handler invoked on a thrown error.
			handler: <P as SendRefCountedPointer>::Of<'a, dyn 'a + Fn(E) -> A + Send + Sync>,
		},
	}

	impl_kind! {
		impl<P: ToDynSendFn, E: Send + Sync + 'static> for SendCatchBrand<P, E> {
			type Of<'a, A: 'a>: 'a = SendCatch<'a, P, E, A>;
		}
	}

	// SendCatchBrand does not implement Functor: the trait's
	// `f: impl Fn(A) -> B + 'a` lacks the `Send + Sync` bounds that
	// `<ArcBrand as ToDynSendFn>::new` requires for closure-storage in
	// the SendCatch handler cell. Mirrors the Phase 3 [`SendStateBrand`]
	// precedent (only [`SendFunctor`] is implemented). Arc-family
	// substrates traverse the scoped row via [`SendFunctor::send_map`]
	// at [`NodeBrand`](crate::brands::NodeBrand), so the missing
	// [`Functor`] impl is unreachable.

	// ===== SendFunctor for SendCatchBrand =====

	#[document_type_parameters("The error type recovered from.")]
	impl<E> SendFunctor for SendCatchBrand<ArcBrand, E>
	where
		E: Send + Sync + 'static,
	{
		/// Thread-safe map over the result type. Mirrors [`Functor::map`]
		/// but threads `Send + Sync` bounds through `f`, the input,
		/// and the output, matching the `ArcCoyoneda` algebra's
		/// `SendFunctor`-only requirement on the Arc family.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the continuations.",
			"The original result type (`Send + Sync`).",
			"The new result type after applying `f` (`Send + Sync`)."
		)]
		///
		#[document_parameters(
			"The function to apply (must be `Send + Sync`).",
			"The catch effect."
		)]
		///
		#[document_returns("A new catch effect with `f` composed over its result type.")]
		///
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	core::ops::Deref,
		/// 	fp_library::{
		/// 		brands::{
		/// 			ArcBrand,
		/// 			SendCatchBrand,
		/// 		},
		/// 		classes::{
		/// 			SendFunctor,
		/// 			ToDynSendFn,
		/// 		},
		/// 		types::effects::catch::SendCatch,
		/// 	},
		/// };
		///
		/// let catch: SendCatch<'static, ArcBrand, &'static str, i32> = SendCatch::Catch {
		/// 	action: 7,
		/// 	handler: <ArcBrand as ToDynSendFn>::new(|_e: &'static str| 100),
		/// };
		/// let mapped =
		/// 	<SendCatchBrand<ArcBrand, &'static str> as SendFunctor>::send_map(|x: i32| x + 1, catch);
		/// match mapped {
		/// 	SendCatch::Catch {
		/// 		action,
		/// 		handler,
		/// 	} => {
		/// 		assert_eq!(action, 8);
		/// 		assert_eq!(handler.deref()("oops"), 101);
		/// 	}
		/// }
		/// ```
		fn send_map<'a, A: Send + Sync + 'a, B: Send + Sync + 'a>(
			f: impl Fn(A) -> B + Send + Sync + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			match fa {
				SendCatch::Catch {
					action,
					handler,
				} => SendCatch::Catch {
					action: f(action),
					handler: <ArcBrand as ToDynSendFn>::new(move |e: E| f(handler(e))),
				},
			}
		}
	}

	// SendFunctor stubs for non-Send brands. Required by the
	// substrate's `NodeBrand<R, S>: SendFunctor` bound when S contains
	// these brands; the stubs panic if invoked but cannot be reached
	// because the corresponding wrappers (Run / RunExplicit / RcRun /
	// RcRunExplicit) do not actually exercise the SendFunctor path.

	#[document_type_parameters("The error type recovered from.")]
	impl<E> SendFunctor for BoxCatchBrand<BoxBrand, E>
	where
		E: Send + Sync + 'static,
	{
		/// SendFunctor stub for the BoxBrand-flavoured Catch. The
		/// projection `Box<dyn FnOnce>` is not `Send + Sync`, so this
		/// brand does not appear in Arc-family substrates and the
		/// SendFunctor path is unreachable in practice. The body
		/// delegates to [`Functor::map`].
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the continuations.",
			"The original result type.",
			"The new result type."
		)]
		///
		#[document_parameters("The function to apply.", "The catch effect.")]
		///
		#[document_returns("A new catch effect with `f` composed over its result type.")]
		///
		#[document_examples]
		///
		/// ```
		/// // Exercised via brand-dispatch through NodeBrand<R, S>: SendFunctor
		/// // when S = CoproductBrand<BoxCatchBrand<BoxBrand, E>, _>; never
		/// // reached at runtime on default Run substrates.
		/// use fp_library::{
		/// 	brands::{
		/// 		BoxBrand,
		/// 		BoxCatchBrand,
		/// 	},
		/// 	classes::{
		/// 		SendFunctor,
		/// 		ToDynFnOnce,
		/// 	},
		/// 	types::effects::catch::BoxCatch,
		/// };
		///
		/// let catch: BoxCatch<'static, BoxBrand, &'static str, i32> = BoxCatch::Catch {
		/// 	action: 7,
		/// 	handler: <BoxBrand as ToDynFnOnce>::new(|_e: &'static str| 100),
		/// };
		/// let mapped =
		/// 	<BoxCatchBrand<BoxBrand, &'static str> as SendFunctor>::send_map(|x: i32| x + 1, catch);
		/// match mapped {
		/// 	BoxCatch::Catch {
		/// 		action,
		/// 		handler,
		/// 	} => {
		/// 		assert_eq!(action, 8);
		/// 		assert_eq!(handler("oops"), 101);
		/// 	}
		/// }
		/// ```
		fn send_map<'a, A: Send + Sync + 'a, B: Send + Sync + 'a>(
			f: impl Fn(A) -> B + Send + Sync + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			<Self as Functor>::map(f, fa)
		}
	}

	#[document_type_parameters("The error type recovered from.")]
	impl<E> SendFunctor for CatchBrand<RcBrand, E>
	where
		E: Send + Sync + 'static,
	{
		/// SendFunctor stub for the RcBrand-flavoured Catch. The
		/// projection `Rc<dyn Fn>` is not `Send + Sync`, so this brand
		/// does not appear in Arc-family substrates; the SendFunctor
		/// path is reachable only through brand-dispatch but never
		/// exercised at runtime. Delegates to [`Functor::map`].
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the continuations.",
			"The original result type.",
			"The new result type."
		)]
		///
		#[document_parameters("The function to apply.", "The catch effect.")]
		///
		#[document_returns("A new catch effect with `f` composed over its result type.")]
		///
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	core::ops::Deref,
		/// 	fp_library::{
		/// 		brands::{
		/// 			CatchBrand,
		/// 			RcBrand,
		/// 		},
		/// 		classes::{
		/// 			SendFunctor,
		/// 			ToDynCloneFn,
		/// 		},
		/// 		types::effects::catch::Catch,
		/// 	},
		/// };
		///
		/// let catch: Catch<'static, RcBrand, &'static str, i32> = Catch::Catch {
		/// 	action: 7,
		/// 	handler: <RcBrand as ToDynCloneFn>::new(|_e: &'static str| 100),
		/// };
		/// let mapped =
		/// 	<CatchBrand<RcBrand, &'static str> as SendFunctor>::send_map(|x: i32| x + 1, catch);
		/// match mapped {
		/// 	Catch::Catch {
		/// 		action,
		/// 		handler,
		/// 	} => {
		/// 		assert_eq!(action, 8);
		/// 		assert_eq!(handler.deref()("oops"), 101);
		/// 	}
		/// }
		/// ```
		fn send_map<'a, A: Send + Sync + 'a, B: Send + Sync + 'a>(
			f: impl Fn(A) -> B + Send + Sync + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			<Self as Functor>::map(f, fa)
		}
	}

	// ===== WrapDrop impls =====

	#[document_type_parameters("The error type recovered from.")]
	impl<E> WrapDrop for BoxCatchBrand<BoxBrand, E>
	where
		E: 'static,
	{
		/// Drop-time decomposition for `BoxCatch`: returns `Some(action)`
		/// so the substrate's iterative `Drop` path can continue
		/// walking the program tree. The recovery handler closure is
		/// dropped silently.
		#[document_signature]
		///
		#[document_type_parameters("The lifetime.", "The result type.")]
		///
		#[document_parameters("The catch effect to decompose.")]
		///
		#[document_returns("`Some` of the action; the recovery handler is dropped.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		BoxBrand,
		/// 		BoxCatchBrand,
		/// 	},
		/// 	classes::{
		/// 		ToDynFnOnce,
		/// 		WrapDrop,
		/// 	},
		/// 	types::effects::catch::BoxCatch,
		/// };
		///
		/// let catch: BoxCatch<'static, BoxBrand, &'static str, i32> = BoxCatch::Catch {
		/// 	action: 42,
		/// 	handler: <BoxBrand as ToDynFnOnce>::new(|_e: &'static str| 0),
		/// };
		/// assert_eq!(<BoxCatchBrand<BoxBrand, &'static str> as WrapDrop>::drop(catch), Some(42));
		/// ```
		fn drop<'a, X: 'a>(
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, X>)
		) -> Option<X> {
			match fa {
				BoxCatch::Catch {
					action,
					handler: _,
				} => Some(action),
			}
		}
	}

	#[document_type_parameters("The error type recovered from.")]
	impl<E> WrapDrop for CatchBrand<RcBrand, E>
	where
		E: 'static,
	{
		/// Drop-time decomposition for `Catch`. Same shape as
		/// [`BoxCatchBrand`'s drop](BoxCatchBrand): returns `Some(action)`,
		/// drops the recovery handler silently.
		#[document_signature]
		///
		#[document_type_parameters("The lifetime.", "The result type.")]
		///
		#[document_parameters("The catch effect to decompose.")]
		///
		#[document_returns("`Some` of the action; the recovery handler is dropped.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		CatchBrand,
		/// 		RcBrand,
		/// 	},
		/// 	classes::{
		/// 		ToDynCloneFn,
		/// 		WrapDrop,
		/// 	},
		/// 	types::effects::catch::Catch,
		/// };
		///
		/// let catch: Catch<'static, RcBrand, &'static str, i32> = Catch::Catch {
		/// 	action: 42,
		/// 	handler: <RcBrand as ToDynCloneFn>::new(|_e: &'static str| 0),
		/// };
		/// assert_eq!(<CatchBrand<RcBrand, &'static str> as WrapDrop>::drop(catch), Some(42));
		/// ```
		fn drop<'a, X: 'a>(
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, X>)
		) -> Option<X> {
			match fa {
				Catch::Catch {
					action,
					handler: _,
				} => Some(action),
			}
		}
	}

	#[document_type_parameters("The error type recovered from.")]
	impl<E> WrapDrop for SendCatchBrand<ArcBrand, E>
	where
		E: Send + Sync + 'static,
	{
		/// Drop-time decomposition for `SendCatch`. Same shape as
		/// [`BoxCatchBrand`'s drop](BoxCatchBrand): returns `Some(action)`,
		/// drops the recovery handler silently.
		#[document_signature]
		///
		#[document_type_parameters("The lifetime.", "The result type.")]
		///
		#[document_parameters("The catch effect to decompose.")]
		///
		#[document_returns("`Some` of the action; the recovery handler is dropped.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		ArcBrand,
		/// 		SendCatchBrand,
		/// 	},
		/// 	classes::{
		/// 		ToDynSendFn,
		/// 		WrapDrop,
		/// 	},
		/// 	types::effects::catch::SendCatch,
		/// };
		///
		/// let catch: SendCatch<'static, ArcBrand, &'static str, i32> = SendCatch::Catch {
		/// 	action: 42,
		/// 	handler: <ArcBrand as ToDynSendFn>::new(|_e: &'static str| 0),
		/// };
		/// assert_eq!(<SendCatchBrand<ArcBrand, &'static str> as WrapDrop>::drop(catch), Some(42),);
		/// ```
		fn drop<'a, X: 'a>(
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, X>)
		) -> Option<X> {
			match fa {
				SendCatch::Catch {
					action,
					handler: _,
				} => Some(action),
			}
		}
	}

	// ===== Extract impls =====

	#[document_type_parameters("The error type recovered from.")]
	impl<E> Extract for BoxCatchBrand<BoxBrand, E>
	where
		E: 'static,
	{
		/// Extracts the action from a `BoxCatch`. The recovery handler
		/// is dropped silently.
		#[document_signature]
		///
		#[document_type_parameters("The lifetime.", "The result type.")]
		///
		#[document_parameters("The catch effect.")]
		///
		#[document_returns("The action.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		BoxBrand,
		/// 		BoxCatchBrand,
		/// 	},
		/// 	classes::{
		/// 		Extract,
		/// 		ToDynFnOnce,
		/// 	},
		/// 	types::effects::catch::BoxCatch,
		/// };
		///
		/// let catch: BoxCatch<'static, BoxBrand, &'static str, i32> = BoxCatch::Catch {
		/// 	action: 42,
		/// 	handler: <BoxBrand as ToDynFnOnce>::new(|_e: &'static str| 0),
		/// };
		/// assert_eq!(<BoxCatchBrand<BoxBrand, &'static str> as Extract>::extract(catch), 42);
		/// ```
		fn extract<'a, A: 'a>(
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)
		) -> A {
			match fa {
				BoxCatch::Catch {
					action,
					handler: _,
				} => action,
			}
		}
	}

	#[document_type_parameters("The error type recovered from.")]
	impl<E> Extract for CatchBrand<RcBrand, E>
	where
		E: 'static,
	{
		/// Extracts the action from a `Catch`. Mirrors
		/// [`BoxCatchBrand::extract`]; the recovery handler is dropped.
		#[document_signature]
		///
		#[document_type_parameters("The lifetime.", "The result type.")]
		///
		#[document_parameters("The catch effect.")]
		///
		#[document_returns("The action.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		CatchBrand,
		/// 		RcBrand,
		/// 	},
		/// 	classes::{
		/// 		Extract,
		/// 		ToDynCloneFn,
		/// 	},
		/// 	types::effects::catch::Catch,
		/// };
		///
		/// let catch: Catch<'static, RcBrand, &'static str, i32> = Catch::Catch {
		/// 	action: 42,
		/// 	handler: <RcBrand as ToDynCloneFn>::new(|_e: &'static str| 0),
		/// };
		/// assert_eq!(<CatchBrand<RcBrand, &'static str> as Extract>::extract(catch), 42);
		/// ```
		fn extract<'a, A: 'a>(
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)
		) -> A {
			match fa {
				Catch::Catch {
					action,
					handler: _,
				} => action,
			}
		}
	}

	#[document_type_parameters("The error type recovered from.")]
	impl<E> Extract for SendCatchBrand<ArcBrand, E>
	where
		E: Send + Sync + 'static,
	{
		/// Extracts the action from a `SendCatch`. Mirrors
		/// [`BoxCatchBrand::extract`]; the recovery handler is dropped.
		#[document_signature]
		///
		#[document_type_parameters("The lifetime.", "The result type.")]
		///
		#[document_parameters("The catch effect.")]
		///
		#[document_returns("The action.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		ArcBrand,
		/// 		SendCatchBrand,
		/// 	},
		/// 	classes::{
		/// 		Extract,
		/// 		ToDynSendFn,
		/// 	},
		/// 	types::effects::catch::SendCatch,
		/// };
		///
		/// let catch: SendCatch<'static, ArcBrand, &'static str, i32> = SendCatch::Catch {
		/// 	action: 42,
		/// 	handler: <ArcBrand as ToDynSendFn>::new(|_e: &'static str| 0),
		/// };
		/// assert_eq!(<SendCatchBrand<ArcBrand, &'static str> as Extract>::extract(catch), 42);
		/// ```
		fn extract<'a, A: 'a>(
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)
		) -> A {
			match fa {
				SendCatch::Catch {
					action,
					handler: _,
				} => action,
			}
		}
	}
}

pub use inner::*;
