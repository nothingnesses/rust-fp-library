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
//! the recovery program both produce the same next-program shape in
//! the same first-order and scoped rows.
//!
//! ## Three sibling types
//!
//! Mirroring the
//! [`BoxState`](crate::types::effects::state::BoxState) /
//! [`State`](crate::types::effects::state::State) /
//! [`SendState`](crate::types::effects::state::SendState) split),
//! `Catch` ships in three sibling flavours that thread different
//! per-pointer-brand closure-storage shapes for both fields. The
//! action and the recovery handler are stored as unit-arg thunks
//! / 1-arg closures behind the same per-pointer-brand pointer; the
//! unit-arg form lets the existing pointer-abstraction
//! [`ToDynFnOnce`](crate::classes::ToDynFnOnce) /
//! [`ToDynCloneFn`](crate::classes::ToDynCloneFn) /
//! [`ToDynSendFn`](crate::classes::ToDynSendFn) family construct
//! both fields.
//!
//! - [`BoxCatch<'a, P, E, A>`] for default `Run` / `RunExplicit`,
//!   `P: ToDynFnOnce` (`BoxBrand` only); action is
//!   `Box<dyn 'a + FnOnce(()) -> A>`, recovery handler is
//!   `Box<dyn 'a + FnOnce(E) -> A>` (both single-shot).
//! - [`Catch<'a, P, E, A>`] for `RcRun` / `RcRunExplicit`,
//!   `P: ToDynCloneFn` (typically `RcBrand`); action is
//!   `Rc<dyn 'a + Fn(()) -> A>`, recovery handler is
//!   `Rc<dyn 'a + Fn(E) -> A>` (both clone-able via Rc-bump).
//! - [`SendCatch<'a, P, E, A>`] for `ArcRun` / `ArcRunExplicit`,
//!   `P: ToDynSendFn` (typically `ArcBrand`); action is
//!   `Arc<dyn 'a + Fn(()) -> A + Send + Sync>`, recovery handler
//!   is `Arc<dyn 'a + Fn(E) -> A + Send + Sync>` (both clone-able
//!   via Arc-bump and thread-safe).
//!
//! ## Why the action is a thunk
//!
//! Storing the action directly as `A` would create a substrate-
//! level layout cycle when `A` resolves to `Free<NodeBrand<R, S>, ...>`
//! and `S` contains the Catch brand: Free's view holds a Node, the
//! Node::Scoped arm holds a Coproduct of scoped-effect cells, and
//! the Catch cell would carry an unboxed `A = Free<...>` field,
//! creating an infinite layout. The thunk indirection (Box / Rc /
//! Arc are pointer-sized regardless of the closure target's
//! layout) breaks the cycle; the unit-arg form fits the existing
//! pointer-abstraction matrix without a new construction path.

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
		},
		fp_macros::*,
		std::{
			rc::Rc,
			sync::Arc,
		},
	};

	// ===== BoxCatch (BoxBrand + FnOnce, single-shot) =====

	/// Scoped error-recovery effect for default `Run` / `RunExplicit`
	/// substrates. Both the action and the recovery handler are stored
	/// as `<P as Pointer>::Of<'a, dyn 'a + FnOnce(...) -> A>` (i.e.,
	/// `Box<dyn FnOnce>` projections): the action as a 0-arg thunk
	/// `Box<dyn FnOnce() -> A>` (single-shot, materialised once at
	/// dispatch time); the recovery handler as a 1-arg `Box<dyn FnOnce(E) -> A>`.
	/// The thunk-storage on the action breaks the substrate-level
	/// layout cycle (Free -> Node::Scoped -> Coproduct -> Catch.action ->
	/// Free) that an unboxed `action: A` field would create when `A`
	/// resolves to `Free<NodeBrand<R, S>, ...>` with `S` containing
	/// the Catch brand.
	#[document_type_parameters(
		"The lifetime of the action and recovery handler closures.",
		"The pointer brand storing the closures (BoxBrand only by structural bound).",
		"The error type recovered from.",
		"The result type of the action and recovery."
	)]
	pub enum BoxCatch<'a, P, E, A>
	where
		P: ToDynFnOnce,
		E: 'a,
		A: 'a, {
		/// Run the `action` thunk to materialise the protected program;
		/// if it throws an `E`, invoke `handler` with the error to
		/// produce a recovery program.
		Catch {
			/// The protected action program, stored as a unit-arg thunk
			/// (`<P as Pointer>::Of<'a, dyn 'a + FnOnce(()) -> A>`,
			/// single-shot via [`FnOnce`]). The unit-arg form lets the
			/// pointer-abstraction's [`ToDynFnOnce::new`](crate::classes::ToDynFnOnce)
			/// family construct it; the call site invokes it as
			/// `action(())`.
			action: <P as Pointer>::Of<'a, dyn 'a + FnOnce(()) -> A>,
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
		/// effect. Composes `f` with the action thunk and the recovery
		/// handler's return; both new closures are `Box<dyn FnOnce>`
		/// that share `f` via an internal `Rc<dyn Fn>` (so each
		/// single-shot closure can call `f` once via Rc-deref). The
		/// action thunk is invoked lazily at dispatch time.
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
		/// 	action: <BoxBrand as ToDynFnOnce>::new(|_: ()| 7),
		/// 	handler: <BoxBrand as ToDynFnOnce>::new(|_e: &'static str| 100),
		/// };
		/// let mapped = <BoxCatchBrand<BoxBrand, &'static str> as Functor>::map(|x: i32| x + 1, catch);
		/// match mapped {
		/// 	BoxCatch::Catch {
		/// 		action,
		/// 		handler,
		/// 	} => {
		/// 		assert_eq!(action(()), 8);
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
				} => {
					let f_rc: Rc<dyn 'a + Fn(A) -> B> = Rc::new(f);
					let f_for_action = Rc::clone(&f_rc);
					let f_for_handler = f_rc;
					BoxCatch::Catch {
						action: <BoxBrand as ToDynFnOnce>::new(move |()| f_for_action(action(()))),
						handler: <BoxBrand as ToDynFnOnce>::new(move |e: E| {
							f_for_handler(handler(e))
						}),
					}
				}
			}
		}
	}

	// ===== Catch (RcBrand + Fn, multi-shot clone-capable) =====

	/// Scoped error-recovery effect for `RcRun` / `RcRunExplicit`
	/// substrates. Both the action and the recovery handler are stored
	/// as `<P as RefCountedPointer>::Of<'a, dyn 'a + Fn(...) -> A>`
	/// (i.e., `Rc<dyn Fn>` projections): the action as a 0-arg thunk
	/// `Rc<dyn Fn() -> A>` (multi-shot, materialised on each call);
	/// the recovery handler as a 1-arg `Rc<dyn Fn(E) -> A>`. Both are
	/// clone-able (refcount-bump) along with the surrounding multi-shot
	/// program. The thunk-storage on the action breaks the substrate-
	/// level layout cycle (Free -> Node::Scoped -> Coproduct ->
	/// Catch.action -> Free) that an unboxed `action: A` field would
	/// create when `A` resolves to `Free<NodeBrand<R, S>, ...>` with
	/// `S` containing the Catch brand.
	#[document_type_parameters(
		"The lifetime of the action and recovery handler closures.",
		"The pointer brand storing the closures (RcBrand only).",
		"The error type recovered from.",
		"The result type of the action and recovery."
	)]
	pub enum Catch<'a, P, E, A>
	where
		P: ToDynCloneFn,
		E: 'a,
		A: 'a, {
		/// Run the `action` thunk to materialise the protected program;
		/// if it throws an `E`, invoke `handler` with the error to
		/// produce a recovery program.
		Catch {
			/// The protected action program, stored as a unit-arg thunk
			/// (`<P as RefCountedPointer>::Of<'a, dyn 'a + Fn(()) -> A>`,
			/// multi-shot via [`Fn`]; clone via Rc-bump). The unit-arg
			/// form lets the pointer-abstraction's
			/// [`ToDynCloneFn::new`](crate::classes::ToDynCloneFn) family
			/// construct it; the call site invokes it as `action(())`.
			action: <P as RefCountedPointer>::Of<'a, dyn 'a + Fn(()) -> A>,
			/// The recovery handler invoked on a thrown error.
			handler: <P as RefCountedPointer>::Of<'a, dyn 'a + Fn(E) -> A>,
		},
	}

	impl_kind! {
		impl<P: ToDynCloneFn, E: 'static> for CatchBrand<P, E> {
			type Of<'a, A: 'a>: 'a = Catch<'a, P, E, A>;
		}
	}

	#[document_type_parameters(
		"The lifetime of the action and recovery handler closures.",
		"The pointer brand storing the closures.",
		"The error type recovered from.",
		"The result type of the action and recovery."
	)]
	#[document_parameters("The catch effect to clone.")]
	impl<'a, P, E, A> Clone for Catch<'a, P, E, A>
	where
		P: ToDynCloneFn,
		E: 'a,
		A: 'a,
	{
		/// Clones the catch effect by refcount-bumping the stored
		/// action thunk and recovery handler pointers. Both are
		/// `<P as RefCountedPointer>::Of<...>`, which is unconditionally
		/// [`Clone`] per the trait's associated-type bound.
		#[document_signature]
		///
		#[document_returns("A new catch effect sharing the action and handler by refcount.")]
		///
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	core::ops::Deref,
		/// 	fp_library::{
		/// 		brands::*,
		/// 		classes::ToDynCloneFn,
		/// 		types::effects::catch::Catch,
		/// 	},
		/// };
		///
		/// let original: Catch<'static, RcBrand, &'static str, i32> = Catch::Catch {
		/// 	action: <RcBrand as ToDynCloneFn>::new(|_: ()| 42),
		/// 	handler: <RcBrand as ToDynCloneFn>::new(|_e: &'static str| 7),
		/// };
		/// let cloned = original.clone();
		/// match cloned {
		/// 	Catch::Catch {
		/// 		action,
		/// 		handler,
		/// 	} => {
		/// 		assert_eq!(action.deref()(()), 42);
		/// 		assert_eq!(handler.deref()("oops"), 7);
		/// 	}
		/// }
		/// ```
		fn clone(&self) -> Self {
			match self {
				Catch::Catch {
					action,
					handler,
				} => Catch::Catch {
					action: <P as RefCountedPointer>::Of::clone(action),
					handler: <P as RefCountedPointer>::Of::clone(handler),
				},
			}
		}
	}

	#[document_type_parameters("The error type recovered from.")]
	impl<E> Functor for CatchBrand<RcBrand, E>
	where
		E: 'static,
	{
		/// Maps `f` over the result type of this scoped recovery
		/// effect. Composes `f` with the action thunk and the recovery
		/// handler's return; both new closures are `Rc<dyn Fn>` that
		/// share `f` via an internal `Rc<dyn Fn>` clone (so the
		/// multi-shot semantics survive). Mirrors
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
		/// 	action: <RcBrand as ToDynCloneFn>::new(|_: ()| 7),
		/// 	handler: <RcBrand as ToDynCloneFn>::new(|_e: &'static str| 100),
		/// };
		/// let mapped = <CatchBrand<RcBrand, &'static str> as Functor>::map(|x: i32| x + 1, catch);
		/// match mapped {
		/// 	Catch::Catch {
		/// 		action,
		/// 		handler,
		/// 	} => {
		/// 		assert_eq!(action.deref()(()), 8);
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
				} => {
					let f_rc: Rc<dyn 'a + Fn(A) -> B> = Rc::new(f);
					let f_for_action = Rc::clone(&f_rc);
					let f_for_handler = f_rc;
					Catch::Catch {
						action: <RcBrand as ToDynCloneFn>::new(move |()| f_for_action(action(()))),
						handler: <RcBrand as ToDynCloneFn>::new(move |e: E| {
							f_for_handler(handler(e))
						}),
					}
				}
			}
		}
	}

	// ===== SendCatch (ArcBrand + Fn + Send + Sync, thread-safe) =====

	/// Scoped error-recovery effect for `ArcRun` / `ArcRunExplicit`
	/// substrates. Both the action and the recovery handler are stored
	/// as `<P as SendRefCountedPointer>::Of<'a, dyn 'a + Fn(...) -> A + Send + Sync>`
	/// (i.e., `Arc<dyn Fn + Send + Sync>` projections): the action as
	/// a 0-arg thunk `Arc<dyn Fn() -> A + Send + Sync>` (multi-shot,
	/// thread-safe); the recovery handler as a 1-arg
	/// `Arc<dyn Fn(E) -> A + Send + Sync>`. Both are thread-safe and
	/// clone-able (refcount-bump). The thunk-storage on the action
	/// breaks the substrate-level layout cycle described on
	/// [`BoxCatch`] / [`Catch`] for the same reason.
	#[document_type_parameters(
		"The lifetime of the action and recovery handler closures.",
		"The pointer brand storing the closures (ArcBrand only).",
		"The error type recovered from.",
		"The result type of the action and recovery."
	)]
	pub enum SendCatch<'a, P, E, A>
	where
		P: ToDynSendFn,
		E: 'a,
		A: 'a, {
		/// Run the `action` thunk to materialise the protected program;
		/// if it throws an `E`, invoke `handler` with the error to
		/// produce a recovery program.
		Catch {
			/// The protected action program, stored as a unit-arg thunk
			/// (`<P as SendRefCountedPointer>::Of<'a, dyn 'a + Fn(()) -> A + Send + Sync>`,
			/// multi-shot via [`Fn`]; thread-safe; clone via Arc-bump).
			/// The unit-arg form lets the pointer-abstraction's
			/// [`ToDynSendFn::new`](crate::classes::ToDynSendFn) family
			/// construct it; the call site invokes it as `action(())`.
			action: <P as SendRefCountedPointer>::Of<'a, dyn 'a + Fn(()) -> A + Send + Sync>,
			/// The recovery handler invoked on a thrown error.
			handler: <P as SendRefCountedPointer>::Of<'a, dyn 'a + Fn(E) -> A + Send + Sync>,
		},
	}

	impl_kind! {
		impl<P: ToDynSendFn, E: Send + Sync + 'static> for SendCatchBrand<P, E> {
			type Of<'a, A: 'a>: 'a = SendCatch<'a, P, E, A>;
		}
	}

	#[document_type_parameters(
		"The lifetime of the action and recovery handler closures.",
		"The pointer brand storing the closures.",
		"The error type recovered from.",
		"The result type of the action and recovery."
	)]
	#[document_parameters("The catch effect to clone.")]
	impl<'a, P, E, A> Clone for SendCatch<'a, P, E, A>
	where
		P: ToDynSendFn,
		E: 'a,
		A: 'a,
	{
		/// Clones the send-catch effect by refcount-bumping the stored
		/// action thunk and recovery handler pointers. Both are
		/// `<P as SendRefCountedPointer>::Of<...>`, which is
		/// unconditionally [`Clone`] per the trait's associated-type bound.
		#[document_signature]
		///
		#[document_returns("A new catch effect sharing the action and handler by refcount.")]
		///
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	core::ops::Deref,
		/// 	fp_library::{
		/// 		brands::*,
		/// 		classes::ToDynSendFn,
		/// 		types::effects::catch::SendCatch,
		/// 	},
		/// };
		///
		/// let original: SendCatch<'static, ArcBrand, &'static str, i32> = SendCatch::Catch {
		/// 	action: <ArcBrand as ToDynSendFn>::new(|_: ()| 42),
		/// 	handler: <ArcBrand as ToDynSendFn>::new(|_e: &'static str| 7),
		/// };
		/// let cloned = original.clone();
		/// match cloned {
		/// 	SendCatch::Catch {
		/// 		action,
		/// 		handler,
		/// 	} => {
		/// 		assert_eq!(action.deref()(()), 42);
		/// 		assert_eq!(handler.deref()("oops"), 7);
		/// 	}
		/// }
		/// ```
		fn clone(&self) -> Self {
			match self {
				SendCatch::Catch {
					action,
					handler,
				} => SendCatch::Catch {
					action: <P as SendRefCountedPointer>::Of::clone(action),
					handler: <P as SendRefCountedPointer>::Of::clone(handler),
				},
			}
		}
	}

	// SendCatchBrand does not implement Functor: the trait's
	// `f: impl Fn(A) -> B + 'a` lacks the `Send + Sync` bounds that
	// `<ArcBrand as ToDynSendFn>::new` requires for closure-storage in
	// the SendCatch handler cell. Mirrors the [`SendStateBrand`]
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
		/// 	action: <ArcBrand as ToDynSendFn>::new(|_: ()| 7),
		/// 	handler: <ArcBrand as ToDynSendFn>::new(|_e: &'static str| 100),
		/// };
		/// let mapped =
		/// 	<SendCatchBrand<ArcBrand, &'static str> as SendFunctor>::send_map(|x: i32| x + 1, catch);
		/// match mapped {
		/// 	SendCatch::Catch {
		/// 		action,
		/// 		handler,
		/// 	} => {
		/// 		assert_eq!(action.deref()(()), 8);
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
				} => {
					let f_arc: Arc<dyn 'a + Fn(A) -> B + Send + Sync> = Arc::new(f);
					let f_for_action = Arc::clone(&f_arc);
					let f_for_handler = f_arc;
					SendCatch::Catch {
						action: <ArcBrand as ToDynSendFn>::new(move |()| f_for_action(action(()))),
						handler: <ArcBrand as ToDynSendFn>::new(move |e: E| {
							f_for_handler(handler(e))
						}),
					}
				}
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
		/// 	action: <BoxBrand as ToDynFnOnce>::new(|_: ()| 7),
		/// 	handler: <BoxBrand as ToDynFnOnce>::new(|_e: &'static str| 100),
		/// };
		/// let mapped =
		/// 	<BoxCatchBrand<BoxBrand, &'static str> as SendFunctor>::send_map(|x: i32| x + 1, catch);
		/// match mapped {
		/// 	BoxCatch::Catch {
		/// 		action,
		/// 		handler,
		/// 	} => {
		/// 		assert_eq!(action(()), 8);
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
		/// 	action: <RcBrand as ToDynCloneFn>::new(|_: ()| 7),
		/// 	handler: <RcBrand as ToDynCloneFn>::new(|_e: &'static str| 100),
		/// };
		/// let mapped =
		/// 	<CatchBrand<RcBrand, &'static str> as SendFunctor>::send_map(|x: i32| x + 1, catch);
		/// match mapped {
		/// 	Catch::Catch {
		/// 		action,
		/// 		handler,
		/// 	} => {
		/// 		assert_eq!(action.deref()(()), 8);
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
		/// Drop-time decomposition for `BoxCatch`: invokes the action
		/// thunk to materialise the action program and returns
		/// `Some(action)` so the substrate's iterative `Drop` path can
		/// continue walking the program tree. The recovery handler
		/// closure is dropped silently.
		#[document_signature]
		///
		#[document_type_parameters("The lifetime.", "The result type.")]
		///
		#[document_parameters("The catch effect to decompose.")]
		///
		#[document_returns("`Some` of the materialised action; the recovery handler is dropped.")]
		///
		#[document_examples(skip_call_check)]
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
		/// 	action: <BoxBrand as ToDynFnOnce>::new(|_: ()| 42),
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
				} => Some(action(())),
			}
		}
	}

	#[document_type_parameters("The error type recovered from.")]
	impl<E> WrapDrop for CatchBrand<RcBrand, E>
	where
		E: 'static,
	{
		/// Drop-time decomposition for `Catch`. Same shape as
		/// [`BoxCatchBrand`'s drop](BoxCatchBrand): calls the action
		/// thunk via Rc-deref and returns `Some(action)`, drops the
		/// recovery handler silently.
		#[document_signature]
		///
		#[document_type_parameters("The lifetime.", "The result type.")]
		///
		#[document_parameters("The catch effect to decompose.")]
		///
		#[document_returns("`Some` of the materialised action; the recovery handler is dropped.")]
		///
		#[document_examples(skip_call_check)]
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
		/// 	action: <RcBrand as ToDynCloneFn>::new(|_: ()| 42),
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
				} => Some(action(())),
			}
		}
	}

	#[document_type_parameters("The error type recovered from.")]
	impl<E> WrapDrop for SendCatchBrand<ArcBrand, E>
	where
		E: Send + Sync + 'static,
	{
		/// Drop-time decomposition for `SendCatch`. Same shape as
		/// [`BoxCatchBrand`'s drop](BoxCatchBrand): calls the action
		/// thunk via Arc-deref and returns `Some(action)`, drops the
		/// recovery handler silently.
		#[document_signature]
		///
		#[document_type_parameters("The lifetime.", "The result type.")]
		///
		#[document_parameters("The catch effect to decompose.")]
		///
		#[document_returns("`Some` of the materialised action; the recovery handler is dropped.")]
		///
		#[document_examples(skip_call_check)]
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
		/// 	action: <ArcBrand as ToDynSendFn>::new(|_: ()| 42),
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
				} => Some(action(())),
			}
		}
	}

	// ===== Extract impls =====

	#[document_type_parameters("The error type recovered from.")]
	impl<E> Extract for BoxCatchBrand<BoxBrand, E>
	where
		E: 'static,
	{
		/// Extracts the action from a `BoxCatch` by invoking the
		/// action thunk. The recovery handler is dropped silently.
		#[document_signature]
		///
		#[document_type_parameters("The lifetime.", "The result type.")]
		///
		#[document_parameters("The catch effect.")]
		///
		#[document_returns("The materialised action.")]
		///
		#[document_examples(skip_call_check)]
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
		/// 	action: <BoxBrand as ToDynFnOnce>::new(|_: ()| 42),
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
				} => action(()),
			}
		}
	}

	#[document_type_parameters("The error type recovered from.")]
	impl<E> Extract for CatchBrand<RcBrand, E>
	where
		E: 'static,
	{
		/// Extracts the action from a `Catch` by invoking the action
		/// thunk via Rc-deref. Mirrors [`BoxCatchBrand::extract`]; the
		/// recovery handler is dropped.
		#[document_signature]
		///
		#[document_type_parameters("The lifetime.", "The result type.")]
		///
		#[document_parameters("The catch effect.")]
		///
		#[document_returns("The materialised action.")]
		///
		#[document_examples(skip_call_check)]
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
		/// 	action: <RcBrand as ToDynCloneFn>::new(|_: ()| 42),
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
				} => action(()),
			}
		}
	}

	#[document_type_parameters("The error type recovered from.")]
	impl<E> Extract for SendCatchBrand<ArcBrand, E>
	where
		E: Send + Sync + 'static,
	{
		/// Extracts the action from a `SendCatch` by invoking the
		/// action thunk via Arc-deref. Mirrors
		/// [`BoxCatchBrand::extract`]; the recovery handler is dropped.
		#[document_signature]
		///
		#[document_type_parameters("The lifetime.", "The result type.")]
		///
		#[document_parameters("The catch effect.")]
		///
		#[document_returns("The materialised action.")]
		///
		#[document_examples(skip_call_check)]
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
		/// 	action: <ArcBrand as ToDynSendFn>::new(|_: ()| 42),
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
				} => action(()),
			}
		}
	}

	// ===== Brand-projection helpers (RefFunctor support) =====
	//
	// `RefFunctor::ref_map` takes `fa: &<Self as Kind>::Of<'a, A>`; the
	// compiler refuses to unify that reference with the concrete
	// `&BoxCatch<...>` / `&Catch<...>` enum inside the impl scope, even
	// with explicit type annotations, because the GAT projection does
	// not normalize through reference parameters within the impl's
	// HRTB-bearing context. Free functions whose where-clauses carry
	// only `Kind` bounds normalize cleanly, mirroring the
	// [`unwrap_first`](crate::types::effects::arc_run::unwrap_first)
	// precedent.
	//
	// The `BoxCatch` variant's `action: Box<dyn FnOnce() -> A>` cannot
	// be invoked through a reference, so [`BoxCatchBrand`'s
	// `RefFunctor` impl] is a panicking stub and does not need an
	// action-projection helper. The Rc-flavoured `Catch` variant's
	// `action: Rc<dyn Fn() -> A>` is callable via Rc-deref, so its
	// `RefFunctor` impl meaningfully composes via two helpers
	// ([`catch_action_thunk_ref`] and [`catch_handler_ref`]) extracting
	// the Rc-shared cells.

	/// Projects an action-thunk reference out of a [`Catch`] GAT
	/// projection. Used inside [`CatchBrand`'s `RefFunctor` impl] to
	/// clone the `Rc`-shared action thunk without consuming the catch
	/// effect.
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the catch effect's contents.",
		"The borrow lifetime of the input projection.",
		"The pointer brand storing the action thunk (RcBrand only).",
		"The error type recovered from.",
		"The result type of the action."
	)]
	///
	#[document_parameters("The catch effect projection.")]
	///
	#[document_returns("A reference to the action-thunk pointer stored in the catch effect.")]
	///
	#[document_examples]
	///
	/// ```
	/// use {
	/// 	core::ops::Deref,
	/// 	fp_library::{
	/// 		brands::RcBrand,
	/// 		classes::ToDynCloneFn,
	/// 		types::effects::catch::{
	/// 			Catch,
	/// 			catch_action_thunk_ref,
	/// 		},
	/// 	},
	/// };
	///
	/// let catch: Catch<'static, RcBrand, &'static str, i32> = Catch::Catch {
	/// 	action: <RcBrand as ToDynCloneFn>::new(|_: ()| 42),
	/// 	handler: <RcBrand as ToDynCloneFn>::new(|_e: &'static str| 0),
	/// };
	/// let action_thunk = catch_action_thunk_ref::<RcBrand, &'static str, i32>(&catch);
	/// assert_eq!(action_thunk.deref()(()), 42);
	/// ```
	#[doc(hidden)]
	pub fn catch_action_thunk_ref<'a, 'b, P, E, A>(
		fa: &'b Apply!(<CatchBrand<P, E> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)
	) -> &'b <P as RefCountedPointer>::Of<'a, dyn 'a + Fn(()) -> A>
	where
		P: ToDynCloneFn,
		E: 'static,
		A: 'a, {
		match fa {
			Catch::Catch {
				action,
				handler: _,
			} => action,
		}
	}

	/// Projects a recovery-handler reference out of a [`Catch`] GAT
	/// projection. Used inside [`CatchBrand`'s `RefFunctor` impl] to
	/// clone the `Rc`-shared handler without consuming the catch effect.
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the catch effect's contents.",
		"The borrow lifetime of the input projection.",
		"The pointer brand storing the recovery handler (RcBrand only).",
		"The error type recovered from.",
		"The result type of the action and recovery."
	)]
	///
	#[document_parameters("The catch effect projection.")]
	///
	#[document_returns("A reference to the recovery-handler pointer stored in the catch effect.")]
	///
	#[document_examples]
	///
	/// ```
	/// use {
	/// 	core::ops::Deref,
	/// 	fp_library::{
	/// 		brands::RcBrand,
	/// 		classes::ToDynCloneFn,
	/// 		types::effects::catch::{
	/// 			Catch,
	/// 			catch_handler_ref,
	/// 		},
	/// 	},
	/// };
	///
	/// let catch: Catch<'static, RcBrand, &'static str, i32> = Catch::Catch {
	/// 	action: <RcBrand as ToDynCloneFn>::new(|_: ()| 42),
	/// 	handler: <RcBrand as ToDynCloneFn>::new(|_e: &'static str| 7),
	/// };
	/// let handler = catch_handler_ref::<RcBrand, &'static str, i32>(&catch);
	/// assert_eq!(handler.deref()("oops"), 7);
	/// ```
	#[doc(hidden)]
	pub fn catch_handler_ref<'a, 'b, P, E, A>(
		fa: &'b Apply!(<CatchBrand<P, E> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)
	) -> &'b <P as RefCountedPointer>::Of<'a, dyn 'a + Fn(E) -> A>
	where
		P: ToDynCloneFn,
		E: 'static,
		A: 'a, {
		match fa {
			Catch::Catch {
				action: _,
				handler,
			} => handler,
		}
	}

	// ===== RefFunctor impls =====

	#[document_type_parameters("The error type recovered from.")]
	impl<E> RefFunctor for BoxCatchBrand<BoxBrand, E>
	where
		E: 'static,
	{
		/// Maps `func` over the result type by reference. Both the
		/// action thunk (`Box<dyn FnOnce() -> A>`) and the recovery
		/// handler (`Box<dyn FnOnce(E) -> A>`) cannot be re-invoked
		/// from a reference (Box is not [`Clone`] and
		/// [`FnOnce::call_once`] requires owned `self`), so both new
		/// closures are panicking stubs. This entire impl is
		/// structurally unreachable in real programs because
		/// [`RunExplicitBrand`'s `RefFunctor` impl](crate::brands::RunExplicitBrand)
		/// is reachable only through synthetic non-Coyoneda first-order
		/// rows.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the continuations.",
			"The original result type.",
			"The new result type after applying `func`."
		)]
		///
		#[document_parameters(
			"The function to apply by reference (ignored; new closures are stubs).",
			"The catch effect projection (ignored; new closures are stubs)."
		)]
		///
		#[document_returns(
			"A new catch effect with both action and handler closures as panicking stubs."
		)]
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
		/// 		RefFunctor,
		/// 		ToDynFnOnce,
		/// 	},
		/// 	types::effects::catch::BoxCatch,
		/// };
		///
		/// let catch: BoxCatch<'static, BoxBrand, &'static str, i32> = BoxCatch::Catch {
		/// 	action: <BoxBrand as ToDynFnOnce>::new(|_: ()| 7),
		/// 	handler: <BoxBrand as ToDynFnOnce>::new(|_e: &'static str| 100),
		/// };
		/// let mapped =
		/// 	<BoxCatchBrand<BoxBrand, &'static str> as RefFunctor>::ref_map(|x: &i32| *x + 1, &catch);
		/// match mapped {
		/// 	BoxCatch::Catch {
		/// 		action,
		/// 		handler,
		/// 	} => {
		/// 		assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| action(()))).is_err());
		/// 		assert!(
		/// 			std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| handler("boom"))).is_err()
		/// 		);
		/// 	}
		/// }
		/// ```
		#[expect(
			clippy::unreachable,
			reason = "BoxCatchBrand::ref_map cannot replicate FnOnce thunks through a reference (Box<dyn FnOnce> is uncloneable and FnOnce::call_once requires owned self). The path is reachable only through synthetic non-Coyoneda first-order rows on RunExplicit, which real programs do not exercise."
		)]
		fn ref_map<'a, A: 'a, B: 'a>(
			_func: impl Fn(&A) -> B + 'a,
			_fa: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			BoxCatch::Catch {
				action: <BoxBrand as ToDynFnOnce>::new(|_: ()| -> B {
					unreachable!(
						"BoxCatchBrand::ref_map's stub action invoked; the FnOnce action thunk cannot be replicated through a reference"
					)
				}),
				handler: <BoxBrand as ToDynFnOnce>::new(|_e: E| -> B {
					unreachable!(
						"BoxCatchBrand::ref_map's stub handler invoked; the FnOnce recovery handler cannot be replicated through a reference"
					)
				}),
			}
		}
	}

	#[document_type_parameters("The error type recovered from.")]
	impl<E> RefFunctor for CatchBrand<RcBrand, E>
	where
		E: 'static,
	{
		/// Maps `func` over the result type by reference. The
		/// `Rc<dyn Fn>` recovery handler is cloned (refcount bump) and
		/// `func` is shared via [`Rc`](std::rc::Rc) so both the action
		/// projection and the new handler closure can call it. Mirrors
		/// [`Functor::map`] but threads through references.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the continuations.",
			"The original result type.",
			"The new result type after applying `func`."
		)]
		///
		#[document_parameters(
			"The function to apply by reference.",
			"The catch effect projection."
		)]
		///
		#[document_returns(
			"A new catch effect with `func` applied to the action and post-composed onto the recovery handler."
		)]
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
		/// 			RefFunctor,
		/// 			ToDynCloneFn,
		/// 		},
		/// 		types::effects::catch::Catch,
		/// 	},
		/// };
		///
		/// let catch: Catch<'static, RcBrand, &'static str, i32> = Catch::Catch {
		/// 	action: <RcBrand as ToDynCloneFn>::new(|_: ()| 7),
		/// 	handler: <RcBrand as ToDynCloneFn>::new(|_e: &'static str| 100),
		/// };
		/// let mapped =
		/// 	<CatchBrand<RcBrand, &'static str> as RefFunctor>::ref_map(|x: &i32| *x + 1, &catch);
		/// match mapped {
		/// 	Catch::Catch {
		/// 		action,
		/// 		handler,
		/// 	} => {
		/// 		assert_eq!(action.deref()(()), 8);
		/// 		assert_eq!(handler.deref()("oops"), 101);
		/// 	}
		/// }
		/// ```
		fn ref_map<'a, A: 'a, B: 'a>(
			func: impl Fn(&A) -> B + 'a,
			fa: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			let action_thunk_ref = catch_action_thunk_ref::<RcBrand, E, A>(fa);
			let handler_ref = catch_handler_ref::<RcBrand, E, A>(fa);
			let action_thunk_clone = <RcBrand as RefCountedPointer>::Of::clone(action_thunk_ref);
			let handler_clone = <RcBrand as RefCountedPointer>::Of::clone(handler_ref);
			let func_rc = <RcBrand as ToDynCloneFn>::ref_new::<A, B>(func);
			let func_for_action = <RcBrand as RefCountedPointer>::Of::clone(&func_rc);
			let new_action = <RcBrand as ToDynCloneFn>::new::<(), B>(move |()| -> B {
				let a: A = action_thunk_clone(());
				func_for_action(&a)
			});
			let new_handler = <RcBrand as ToDynCloneFn>::new::<E, B>(move |e: E| -> B {
				let a: A = handler_clone(e);
				func_rc(&a)
			});
			Catch::Catch {
				action: new_action,
				handler: new_handler,
			}
		}
	}

	// SendCatchBrand does not implement RefFunctor: the cascade through
	// [`ArcRunExplicitBrand`](crate::brands::ArcRunExplicitBrand) does not
	// require it (the brand-level `Ref`-family is not reachable through
	// [`ArcFreeExplicitBrand`](crate::brands::ArcFreeExplicitBrand), per
	// the brand's documentation), and a hypothetical impl would face the
	// same `Send + Sync` bound mismatch on `func` that prevents
	// [`Functor`] (the trait method's `func: impl Fn(&A) -> B + 'a` lacks
	// the `Send + Sync` bounds that [`<ArcBrand as ToDynSendFn>::new`]
	// requires for closure storage). Mirrors the [`SendStateBrand`] /
	// [`SendCatchBrand`-no-`Functor`] precedent.
}

pub use inner::*;
