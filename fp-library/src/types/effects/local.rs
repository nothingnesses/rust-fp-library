//! Scoped environment-modification effect type with `Local`
//! (run an action under a transformed environment value `E`)
//! as its sole operation. The corresponding brands are
//! [`BoxLocalBrand`](crate::brands::BoxLocalBrand),
//! [`LocalBrand`](crate::brands::LocalBrand), and
//! [`SendLocalBrand`](crate::brands::SendLocalBrand).
//!
//! Mirrors PureScript Run's `Reader.local` and the Val flavour
//! of Heftia's higher-order Reader operation. The
//! environment-transform closure is invoked at most once per
//! scoped layer at dispatch time; the action and the recovery
//! environment-transform run in the same row signature `A`.
//!
//! ## Three sibling types
//!
//! `Local` ships in three sibling flavours that thread different
//! per-pointer-brand closure-storage shapes for both fields. The
//! action is a unit-arg thunk and the environment-transform is a
//! 1-arg closure stored behind the same per-pointer-brand pointer;
//! the unit-arg form lets the existing pointer-abstraction
//! [`ToDynFnOnce`](crate::classes::ToDynFnOnce) /
//! [`ToDynCloneFn`](crate::classes::ToDynCloneFn) /
//! [`ToDynSendFn`](crate::classes::ToDynSendFn) family construct
//! both fields uniformly.
//!
//! - [`BoxLocal<'a, P, E, A>`] for default `Run` / `RunExplicit`,
//!   `P: ToDynFnOnce` (`BoxBrand` only); modify is
//!   `Box<dyn 'a + FnOnce(E) -> E>`, action is
//!   `Box<dyn 'a + FnOnce(()) -> A>` (both single-shot).
//! - [`Local<'a, P, E, A>`] for `RcRun` / `RcRunExplicit`,
//!   `P: ToDynCloneFn` (typically `RcBrand`); modify is
//!   `Rc<dyn 'a + Fn(E) -> E>`, action is
//!   `Rc<dyn 'a + Fn(()) -> A>` (both clone-able via Rc-bump).
//! - [`SendLocal<'a, P, E, A>`] for `ArcRun` / `ArcRunExplicit`,
//!   `P: ToDynSendFn` (typically `ArcBrand`); modify is
//!   `Arc<dyn 'a + Fn(E) -> E + Send + Sync>`, action is
//!   `Arc<dyn 'a + Fn(()) -> A + Send + Sync>` (both clone-able
//!   via Arc-bump and thread-safe).
//!
//! ## Why the action is a thunk
//!
//! Storing the action directly as `A` would create a substrate-
//! level layout cycle when `A` resolves to `Free<NodeBrand<R, S>, ...>`
//! and `S` contains the `Local` brand: `Free`'s view holds a
//! `Node`, the `Node::Scoped` arm holds a `Coproduct` of scoped-
//! effect cells, and the `Local` cell would carry an unboxed
//! `A = Free<...>` field, creating an infinite layout. The thunk
//! indirection (Box / Rc / Arc are pointer-sized regardless of
//! the closure target's layout) breaks the cycle; the unit-arg
//! form fits the existing pointer-abstraction matrix without a
//! new construction path. Mirrors the same fix applied to
//! [`Catch`](crate::types::effects::catch).

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			Apply,
			brands::{
				ArcBrand,
				BoxBrand,
				BoxLocalBrand,
				LocalBrand,
				RcBrand,
				SendLocalBrand,
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

	// ===== BoxLocal (BoxBrand + FnOnce, single-shot) =====

	/// Scoped environment-modification effect for default `Run` /
	/// `RunExplicit` substrates. Both the environment-transform and
	/// the action are stored as `<P as Pointer>::Of<'a, dyn 'a + FnOnce(...) -> _>`
	/// projections (`Box<dyn FnOnce>`): the environment-transform as a
	/// 1-arg `Box<dyn FnOnce(E) -> E>`; the action as a unit-arg thunk
	/// `Box<dyn FnOnce(()) -> A>` (single-shot, materialised once at
	/// dispatch time). The thunk-storage on the action breaks the
	/// substrate-level layout cycle (`Free` -> `Node::Scoped` ->
	/// `Coproduct` -> `BoxLocal.action` -> `Free`) that an unboxed
	/// `action: A` field would create when `A` resolves to
	/// `Free<NodeBrand<R, S>, ...>` with `S` containing the `BoxLocal`
	/// brand.
	#[document_type_parameters(
		"The lifetime of the modify and action closures.",
		"The pointer brand storing the closures (BoxBrand only by structural bound).",
		"The environment type transformed by `modify`.",
		"The result type of the action."
	)]
	pub enum BoxLocal<'a, P, E, A>
	where
		P: ToDynFnOnce,
		E: 'a,
		A: 'a, {
		/// Run the `action` thunk under an environment transformed by
		/// `modify`; the dispatcher applies `modify` to the inherited
		/// environment value once before invoking `action`.
		Local {
			/// The environment-transform closure (`<P as Pointer>::Of<'a, dyn 'a + FnOnce(E) -> E>`,
			/// single-shot via [`FnOnce`]).
			modify: <P as Pointer>::Of<'a, dyn 'a + FnOnce(E) -> E>,
			/// The protected action program, stored as a unit-arg thunk
			/// (`<P as Pointer>::Of<'a, dyn 'a + FnOnce(()) -> A>`,
			/// single-shot via [`FnOnce`]). The unit-arg form lets the
			/// pointer-abstraction's [`ToDynFnOnce::new`](crate::classes::ToDynFnOnce)
			/// family construct it; the call site invokes it as
			/// `action(())`.
			action: <P as Pointer>::Of<'a, dyn 'a + FnOnce(()) -> A>,
		},
	}

	impl_kind! {
		impl<P: ToDynFnOnce, E: 'static> for BoxLocalBrand<P, E> {
			type Of<'a, A: 'a>: 'a = BoxLocal<'a, P, E, A>;
		}
	}

	#[document_type_parameters("The environment type transformed by `modify`.")]
	impl<E> Functor for BoxLocalBrand<BoxBrand, E>
	where
		E: 'static,
	{
		/// Maps `f` over the result type of this scoped effect.
		/// Composes `f` with the action thunk's return; the
		/// environment-transform is unchanged because it does not
		/// depend on the result type. The new action thunk is
		/// `Box<dyn FnOnce>` and consumes both the original action
		/// and `f` once.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the continuations.",
			"The original result type.",
			"The new result type after applying `f`."
		)]
		///
		#[document_parameters("The function to apply.", "The local effect.")]
		///
		#[document_returns("A new local effect with `f` composed over its result type.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		BoxBrand,
		/// 		BoxLocalBrand,
		/// 	},
		/// 	classes::{
		/// 		Functor,
		/// 		ToDynFnOnce,
		/// 	},
		/// 	types::effects::local::BoxLocal,
		/// };
		///
		/// let local: BoxLocal<'static, BoxBrand, i32, i32> = BoxLocal::Local {
		/// 	modify: <BoxBrand as ToDynFnOnce>::new(|e: i32| e + 1),
		/// 	action: <BoxBrand as ToDynFnOnce>::new(|_: ()| 7),
		/// };
		/// let mapped = <BoxLocalBrand<BoxBrand, i32> as Functor>::map(|x: i32| x + 1, local);
		/// match mapped {
		/// 	BoxLocal::Local {
		/// 		modify,
		/// 		action,
		/// 	} => {
		/// 		assert_eq!(modify(10), 11);
		/// 		assert_eq!(action(()), 8);
		/// 	}
		/// }
		/// ```
		fn map<'a, A: 'a, B: 'a>(
			f: impl Fn(A) -> B + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			match fa {
				BoxLocal::Local {
					modify,
					action,
				} => BoxLocal::Local {
					modify,
					action: <BoxBrand as ToDynFnOnce>::new(move |()| f(action(()))),
				},
			}
		}
	}

	// ===== Local (RcBrand + Fn, multi-shot clone-capable) =====

	/// Scoped environment-modification effect for `RcRun` /
	/// `RcRunExplicit` substrates. Both the environment-transform and
	/// the action are stored as
	/// `<P as RefCountedPointer>::Of<'a, dyn 'a + Fn(...) -> _>`
	/// projections (`Rc<dyn Fn>`): the environment-transform as a
	/// 1-arg `Rc<dyn Fn(E) -> E>`; the action as a unit-arg thunk
	/// `Rc<dyn Fn(()) -> A>` (multi-shot, materialised on each call).
	/// Both are clone-able (refcount-bump) along with the surrounding
	/// multi-shot program. The thunk-storage on the action breaks the
	/// substrate-level layout cycle described on [`BoxLocal`] for the
	/// same reason.
	#[document_type_parameters(
		"The lifetime of the modify and action closures.",
		"The pointer brand storing the closures (RcBrand only).",
		"The environment type transformed by `modify`.",
		"The result type of the action."
	)]
	pub enum Local<'a, P, E, A>
	where
		P: ToDynCloneFn,
		E: 'a,
		A: 'a, {
		/// Run the `action` thunk under an environment transformed by
		/// `modify`; the dispatcher applies `modify` to the inherited
		/// environment value once before invoking `action`.
		Local {
			/// The environment-transform closure (`<P as RefCountedPointer>::Of<'a, dyn 'a + Fn(E) -> E>`,
			/// multi-shot via [`Fn`]; clone via Rc-bump).
			modify: <P as RefCountedPointer>::Of<'a, dyn 'a + Fn(E) -> E>,
			/// The protected action program, stored as a unit-arg thunk
			/// (`<P as RefCountedPointer>::Of<'a, dyn 'a + Fn(()) -> A>`,
			/// multi-shot via [`Fn`]; clone via Rc-bump). The unit-arg
			/// form lets the pointer-abstraction's
			/// [`ToDynCloneFn::new`](crate::classes::ToDynCloneFn) family
			/// construct it; the call site invokes it as `action(())`.
			action: <P as RefCountedPointer>::Of<'a, dyn 'a + Fn(()) -> A>,
		},
	}

	impl_kind! {
		impl<P: ToDynCloneFn, E: 'static> for LocalBrand<P, E> {
			type Of<'a, A: 'a>: 'a = Local<'a, P, E, A>;
		}
	}

	#[document_type_parameters(
		"The lifetime of the modify and action closures.",
		"The pointer brand storing the closures.",
		"The environment type transformed by `modify`.",
		"The result type of the action."
	)]
	#[document_parameters("The local effect to clone.")]
	impl<'a, P, E, A> Clone for Local<'a, P, E, A>
	where
		P: ToDynCloneFn,
		E: 'a,
		A: 'a,
	{
		/// Clones the local effect by refcount-bumping the stored
		/// environment-transform and action thunk pointers. Both are
		/// `<P as RefCountedPointer>::Of<...>`, which is unconditionally
		/// [`Clone`] per the trait's associated-type bound.
		#[document_signature]
		///
		#[document_returns("A new local effect sharing modify and action by refcount.")]
		///
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	core::ops::Deref,
		/// 	fp_library::{
		/// 		brands::*,
		/// 		classes::ToDynCloneFn,
		/// 		types::effects::local::Local,
		/// 	},
		/// };
		///
		/// let original: Local<'static, RcBrand, i32, i32> = Local::Local {
		/// 	modify: <RcBrand as ToDynCloneFn>::new(|e: i32| e + 1),
		/// 	action: <RcBrand as ToDynCloneFn>::new(|_: ()| 42),
		/// };
		/// let cloned = original.clone();
		/// match cloned {
		/// 	Local::Local {
		/// 		modify,
		/// 		action,
		/// 	} => {
		/// 		assert_eq!(modify.deref()(10), 11);
		/// 		assert_eq!(action.deref()(()), 42);
		/// 	}
		/// }
		/// ```
		fn clone(&self) -> Self {
			match self {
				Local::Local {
					modify,
					action,
				} => Local::Local {
					modify: <P as RefCountedPointer>::Of::clone(modify),
					action: <P as RefCountedPointer>::Of::clone(action),
				},
			}
		}
	}

	#[document_type_parameters("The environment type transformed by `modify`.")]
	impl<E> Functor for LocalBrand<RcBrand, E>
	where
		E: 'static,
	{
		/// Maps `f` over the result type of this scoped effect.
		/// Composes `f` with the action thunk's return; the
		/// environment-transform is unchanged because it does not
		/// depend on the result type. The new action thunk is
		/// `Rc<dyn Fn>` and shares both the original action and `f` by
		/// move (multi-shot semantics survive).
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the continuations.",
			"The original result type.",
			"The new result type after applying `f`."
		)]
		///
		#[document_parameters("The function to apply.", "The local effect.")]
		///
		#[document_returns("A new local effect with `f` composed over its result type.")]
		///
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	core::ops::Deref,
		/// 	fp_library::{
		/// 		brands::{
		/// 			LocalBrand,
		/// 			RcBrand,
		/// 		},
		/// 		classes::{
		/// 			Functor,
		/// 			ToDynCloneFn,
		/// 		},
		/// 		types::effects::local::Local,
		/// 	},
		/// };
		///
		/// let local: Local<'static, RcBrand, i32, i32> = Local::Local {
		/// 	modify: <RcBrand as ToDynCloneFn>::new(|e: i32| e + 1),
		/// 	action: <RcBrand as ToDynCloneFn>::new(|_: ()| 7),
		/// };
		/// let mapped = <LocalBrand<RcBrand, i32> as Functor>::map(|x: i32| x + 1, local);
		/// match mapped {
		/// 	Local::Local {
		/// 		modify,
		/// 		action,
		/// 	} => {
		/// 		assert_eq!(modify.deref()(10), 11);
		/// 		assert_eq!(action.deref()(()), 8);
		/// 	}
		/// }
		/// ```
		fn map<'a, A: 'a, B: 'a>(
			f: impl Fn(A) -> B + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			match fa {
				Local::Local {
					modify,
					action,
				} => Local::Local {
					modify,
					action: <RcBrand as ToDynCloneFn>::new(move |()| f(action(()))),
				},
			}
		}
	}

	// ===== SendLocal (ArcBrand + Fn + Send + Sync, thread-safe) =====

	/// Scoped environment-modification effect for `ArcRun` /
	/// `ArcRunExplicit` substrates. Both the environment-transform and
	/// the action are stored as
	/// `<P as SendRefCountedPointer>::Of<'a, dyn 'a + Fn(...) -> _ + Send + Sync>`
	/// projections (`Arc<dyn Fn + Send + Sync>`): the environment-
	/// transform as a 1-arg `Arc<dyn Fn(E) -> E + Send + Sync>`; the
	/// action as a unit-arg thunk
	/// `Arc<dyn Fn(()) -> A + Send + Sync>` (multi-shot, thread-safe).
	/// Both are clone-able (refcount-bump) and thread-safe. The
	/// thunk-storage on the action breaks the substrate-level layout
	/// cycle described on [`BoxLocal`] / [`Local`] for the same reason.
	#[document_type_parameters(
		"The lifetime of the modify and action closures.",
		"The pointer brand storing the closures (ArcBrand only).",
		"The environment type transformed by `modify`.",
		"The result type of the action."
	)]
	pub enum SendLocal<'a, P, E, A>
	where
		P: ToDynSendFn,
		E: 'a,
		A: 'a, {
		/// Run the `action` thunk under an environment transformed by
		/// `modify`; the dispatcher applies `modify` to the inherited
		/// environment value once before invoking `action`.
		Local {
			/// The environment-transform closure
			/// (`<P as SendRefCountedPointer>::Of<'a, dyn 'a + Fn(E) -> E + Send + Sync>`,
			/// multi-shot via [`Fn`]; clone via Arc-bump; thread-safe).
			modify: <P as SendRefCountedPointer>::Of<'a, dyn 'a + Fn(E) -> E + Send + Sync>,
			/// The protected action program, stored as a unit-arg thunk
			/// (`<P as SendRefCountedPointer>::Of<'a, dyn 'a + Fn(()) -> A + Send + Sync>`,
			/// multi-shot via [`Fn`]; clone via Arc-bump; thread-safe).
			action: <P as SendRefCountedPointer>::Of<'a, dyn 'a + Fn(()) -> A + Send + Sync>,
		},
	}

	impl_kind! {
		impl<P: ToDynSendFn, E: Send + Sync + 'static> for SendLocalBrand<P, E> {
			type Of<'a, A: 'a>: 'a = SendLocal<'a, P, E, A>;
		}
	}

	#[document_type_parameters(
		"The lifetime of the modify and action closures.",
		"The pointer brand storing the closures.",
		"The environment type transformed by `modify`.",
		"The result type of the action."
	)]
	#[document_parameters("The send-local effect to clone.")]
	impl<'a, P, E, A> Clone for SendLocal<'a, P, E, A>
	where
		P: ToDynSendFn,
		E: 'a,
		A: 'a,
	{
		/// Clones the send-local effect by refcount-bumping the stored
		/// environment-transform and action thunk pointers. Both are
		/// `<P as SendRefCountedPointer>::Of<...>`, which is
		/// unconditionally [`Clone`] per the trait's associated-type
		/// bound.
		#[document_signature]
		///
		#[document_returns("A new send-local effect sharing modify and action by refcount.")]
		///
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	core::ops::Deref,
		/// 	fp_library::{
		/// 		brands::*,
		/// 		classes::ToDynSendFn,
		/// 		types::effects::local::SendLocal,
		/// 	},
		/// };
		///
		/// let original: SendLocal<'static, ArcBrand, i32, i32> = SendLocal::Local {
		/// 	modify: <ArcBrand as ToDynSendFn>::new(|e: i32| e + 1),
		/// 	action: <ArcBrand as ToDynSendFn>::new(|_: ()| 42),
		/// };
		/// let cloned = original.clone();
		/// match cloned {
		/// 	SendLocal::Local {
		/// 		modify,
		/// 		action,
		/// 	} => {
		/// 		assert_eq!(modify.deref()(10), 11);
		/// 		assert_eq!(action.deref()(()), 42);
		/// 	}
		/// }
		/// ```
		fn clone(&self) -> Self {
			match self {
				SendLocal::Local {
					modify,
					action,
				} => SendLocal::Local {
					modify: <P as SendRefCountedPointer>::Of::clone(modify),
					action: <P as SendRefCountedPointer>::Of::clone(action),
				},
			}
		}
	}

	// ===== SendFunctor for SendLocalBrand (load-bearing on Arc family) =====

	#[document_type_parameters("The environment type transformed by `modify`.")]
	impl<E> SendFunctor for SendLocalBrand<ArcBrand, E>
	where
		E: Send + Sync + 'static,
	{
		/// Thread-safe map over the result type. Mirrors [`Functor::map`]
		/// but threads `Send + Sync` bounds through `f`, the input,
		/// and the output, matching the `ArcCoyoneda` algebra's
		/// `SendFunctor`-only requirement on the Arc family. The
		/// environment-transform is unchanged; only the action thunk
		/// is composed with `f`.
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
			"The send-local effect."
		)]
		///
		#[document_returns("A new send-local effect with `f` composed over its result type.")]
		///
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	core::ops::Deref,
		/// 	fp_library::{
		/// 		brands::{
		/// 			ArcBrand,
		/// 			SendLocalBrand,
		/// 		},
		/// 		classes::{
		/// 			SendFunctor,
		/// 			ToDynSendFn,
		/// 		},
		/// 		types::effects::local::SendLocal,
		/// 	},
		/// };
		///
		/// let local: SendLocal<'static, ArcBrand, i32, i32> = SendLocal::Local {
		/// 	modify: <ArcBrand as ToDynSendFn>::new(|e: i32| e + 1),
		/// 	action: <ArcBrand as ToDynSendFn>::new(|_: ()| 7),
		/// };
		/// let mapped = <SendLocalBrand<ArcBrand, i32> as SendFunctor>::send_map(|x: i32| x + 1, local);
		/// match mapped {
		/// 	SendLocal::Local {
		/// 		modify,
		/// 		action,
		/// 	} => {
		/// 		assert_eq!(modify.deref()(10), 11);
		/// 		assert_eq!(action.deref()(()), 8);
		/// 	}
		/// }
		/// ```
		fn send_map<'a, A: Send + Sync + 'a, B: Send + Sync + 'a>(
			f: impl Fn(A) -> B + Send + Sync + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			match fa {
				SendLocal::Local {
					modify,
					action,
				} => SendLocal::Local {
					modify,
					action: <ArcBrand as ToDynSendFn>::new(move |()| f(action(()))),
				},
			}
		}
	}

	// SendFunctor stubs for non-Send brands. Required by the
	// substrate's `NodeBrand<R, S>: SendFunctor` bound when S contains
	// these brands; the stubs delegate to `Functor::map` and cannot be
	// reached at runtime because the corresponding wrappers (Run /
	// RunExplicit / RcRun / RcRunExplicit) do not exercise the
	// SendFunctor path.

	#[document_type_parameters("The environment type transformed by `modify`.")]
	impl<E> SendFunctor for BoxLocalBrand<BoxBrand, E>
	where
		E: Send + Sync + 'static,
	{
		/// SendFunctor stub for the BoxBrand-flavoured Local. The
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
		#[document_parameters("The function to apply.", "The local effect.")]
		///
		#[document_returns("A new local effect with `f` composed over its result type.")]
		///
		#[document_examples]
		///
		/// ```
		/// // Exercised via brand-dispatch through NodeBrand<R, S>: SendFunctor
		/// // when S = CoproductBrand<BoxLocalBrand<BoxBrand, E>, _>; never
		/// // reached at runtime on default Run substrates.
		/// use fp_library::{
		/// 	brands::{
		/// 		BoxBrand,
		/// 		BoxLocalBrand,
		/// 	},
		/// 	classes::{
		/// 		SendFunctor,
		/// 		ToDynFnOnce,
		/// 	},
		/// 	types::effects::local::BoxLocal,
		/// };
		///
		/// let local: BoxLocal<'static, BoxBrand, i32, i32> = BoxLocal::Local {
		/// 	modify: <BoxBrand as ToDynFnOnce>::new(|e: i32| e + 1),
		/// 	action: <BoxBrand as ToDynFnOnce>::new(|_: ()| 7),
		/// };
		/// let mapped = <BoxLocalBrand<BoxBrand, i32> as SendFunctor>::send_map(|x: i32| x + 1, local);
		/// match mapped {
		/// 	BoxLocal::Local {
		/// 		modify,
		/// 		action,
		/// 	} => {
		/// 		assert_eq!(modify(10), 11);
		/// 		assert_eq!(action(()), 8);
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

	#[document_type_parameters("The environment type transformed by `modify`.")]
	impl<E> SendFunctor for LocalBrand<RcBrand, E>
	where
		E: Send + Sync + 'static,
	{
		/// SendFunctor stub for the RcBrand-flavoured Local. The
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
		#[document_parameters("The function to apply.", "The local effect.")]
		///
		#[document_returns("A new local effect with `f` composed over its result type.")]
		///
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	core::ops::Deref,
		/// 	fp_library::{
		/// 		brands::{
		/// 			LocalBrand,
		/// 			RcBrand,
		/// 		},
		/// 		classes::{
		/// 			SendFunctor,
		/// 			ToDynCloneFn,
		/// 		},
		/// 		types::effects::local::Local,
		/// 	},
		/// };
		///
		/// let local: Local<'static, RcBrand, i32, i32> = Local::Local {
		/// 	modify: <RcBrand as ToDynCloneFn>::new(|e: i32| e + 1),
		/// 	action: <RcBrand as ToDynCloneFn>::new(|_: ()| 7),
		/// };
		/// let mapped = <LocalBrand<RcBrand, i32> as SendFunctor>::send_map(|x: i32| x + 1, local);
		/// match mapped {
		/// 	Local::Local {
		/// 		modify,
		/// 		action,
		/// 	} => {
		/// 		assert_eq!(modify.deref()(10), 11);
		/// 		assert_eq!(action.deref()(()), 8);
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

	#[document_type_parameters("The environment type transformed by `modify`.")]
	impl<E> WrapDrop for BoxLocalBrand<BoxBrand, E>
	where
		E: 'static,
	{
		/// Drop-time decomposition for `BoxLocal`: invokes the action
		/// thunk to materialise the action program and returns
		/// `Some(action)` so the substrate's iterative `Drop` path can
		/// continue walking the program tree. The
		/// environment-transform closure is dropped silently.
		#[document_signature]
		///
		#[document_type_parameters("The lifetime.", "The result type.")]
		///
		#[document_parameters("The local effect to decompose.")]
		///
		#[document_returns("`Some` of the materialised action; the modify closure is dropped.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		BoxBrand,
		/// 		BoxLocalBrand,
		/// 	},
		/// 	classes::{
		/// 		ToDynFnOnce,
		/// 		WrapDrop,
		/// 	},
		/// 	types::effects::local::BoxLocal,
		/// };
		///
		/// let local: BoxLocal<'static, BoxBrand, i32, i32> = BoxLocal::Local {
		/// 	modify: <BoxBrand as ToDynFnOnce>::new(|e: i32| e + 1),
		/// 	action: <BoxBrand as ToDynFnOnce>::new(|_: ()| 42),
		/// };
		/// assert_eq!(<BoxLocalBrand<BoxBrand, i32> as WrapDrop>::drop(local), Some(42));
		/// ```
		fn drop<'a, X: 'a>(
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, X>)
		) -> Option<X> {
			match fa {
				BoxLocal::Local {
					modify: _,
					action,
				} => Some(action(())),
			}
		}
	}

	#[document_type_parameters("The environment type transformed by `modify`.")]
	impl<E> WrapDrop for LocalBrand<RcBrand, E>
	where
		E: 'static,
	{
		/// Drop-time decomposition for `Local`. Same shape as
		/// [`BoxLocalBrand`'s drop](BoxLocalBrand): calls the action
		/// thunk via Rc-deref and returns `Some(action)`, drops the
		/// environment-transform closure silently.
		#[document_signature]
		///
		#[document_type_parameters("The lifetime.", "The result type.")]
		///
		#[document_parameters("The local effect to decompose.")]
		///
		#[document_returns("`Some` of the materialised action; the modify closure is dropped.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		LocalBrand,
		/// 		RcBrand,
		/// 	},
		/// 	classes::{
		/// 		ToDynCloneFn,
		/// 		WrapDrop,
		/// 	},
		/// 	types::effects::local::Local,
		/// };
		///
		/// let local: Local<'static, RcBrand, i32, i32> = Local::Local {
		/// 	modify: <RcBrand as ToDynCloneFn>::new(|e: i32| e + 1),
		/// 	action: <RcBrand as ToDynCloneFn>::new(|_: ()| 42),
		/// };
		/// assert_eq!(<LocalBrand<RcBrand, i32> as WrapDrop>::drop(local), Some(42));
		/// ```
		fn drop<'a, X: 'a>(
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, X>)
		) -> Option<X> {
			match fa {
				Local::Local {
					modify: _,
					action,
				} => Some(action(())),
			}
		}
	}

	#[document_type_parameters("The environment type transformed by `modify`.")]
	impl<E> WrapDrop for SendLocalBrand<ArcBrand, E>
	where
		E: Send + Sync + 'static,
	{
		/// Drop-time decomposition for `SendLocal`. Same shape as
		/// [`BoxLocalBrand`'s drop](BoxLocalBrand): calls the action
		/// thunk via Arc-deref and returns `Some(action)`, drops the
		/// environment-transform closure silently.
		#[document_signature]
		///
		#[document_type_parameters("The lifetime.", "The result type.")]
		///
		#[document_parameters("The send-local effect to decompose.")]
		///
		#[document_returns("`Some` of the materialised action; the modify closure is dropped.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		ArcBrand,
		/// 		SendLocalBrand,
		/// 	},
		/// 	classes::{
		/// 		ToDynSendFn,
		/// 		WrapDrop,
		/// 	},
		/// 	types::effects::local::SendLocal,
		/// };
		///
		/// let local: SendLocal<'static, ArcBrand, i32, i32> = SendLocal::Local {
		/// 	modify: <ArcBrand as ToDynSendFn>::new(|e: i32| e + 1),
		/// 	action: <ArcBrand as ToDynSendFn>::new(|_: ()| 42),
		/// };
		/// assert_eq!(<SendLocalBrand<ArcBrand, i32> as WrapDrop>::drop(local), Some(42));
		/// ```
		fn drop<'a, X: 'a>(
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, X>)
		) -> Option<X> {
			match fa {
				SendLocal::Local {
					modify: _,
					action,
				} => Some(action(())),
			}
		}
	}

	// ===== Extract impls =====

	#[document_type_parameters("The environment type transformed by `modify`.")]
	impl<E> Extract for BoxLocalBrand<BoxBrand, E>
	where
		E: 'static,
	{
		/// Extracts the action from a `BoxLocal` by invoking the action
		/// thunk. The environment-transform closure is dropped silently.
		#[document_signature]
		///
		#[document_type_parameters("The lifetime.", "The result type.")]
		///
		#[document_parameters("The local effect.")]
		///
		#[document_returns("The materialised action.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		BoxBrand,
		/// 		BoxLocalBrand,
		/// 	},
		/// 	classes::{
		/// 		Extract,
		/// 		ToDynFnOnce,
		/// 	},
		/// 	types::effects::local::BoxLocal,
		/// };
		///
		/// let local: BoxLocal<'static, BoxBrand, i32, i32> = BoxLocal::Local {
		/// 	modify: <BoxBrand as ToDynFnOnce>::new(|e: i32| e + 1),
		/// 	action: <BoxBrand as ToDynFnOnce>::new(|_: ()| 42),
		/// };
		/// assert_eq!(<BoxLocalBrand<BoxBrand, i32> as Extract>::extract(local), 42);
		/// ```
		fn extract<'a, A: 'a>(
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)
		) -> A {
			match fa {
				BoxLocal::Local {
					modify: _,
					action,
				} => action(()),
			}
		}
	}

	#[document_type_parameters("The environment type transformed by `modify`.")]
	impl<E> Extract for LocalBrand<RcBrand, E>
	where
		E: 'static,
	{
		/// Extracts the action from a `Local` by invoking the action
		/// thunk via Rc-deref. The environment-transform closure is
		/// dropped silently.
		#[document_signature]
		///
		#[document_type_parameters("The lifetime.", "The result type.")]
		///
		#[document_parameters("The local effect.")]
		///
		#[document_returns("The materialised action.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		LocalBrand,
		/// 		RcBrand,
		/// 	},
		/// 	classes::{
		/// 		Extract,
		/// 		ToDynCloneFn,
		/// 	},
		/// 	types::effects::local::Local,
		/// };
		///
		/// let local: Local<'static, RcBrand, i32, i32> = Local::Local {
		/// 	modify: <RcBrand as ToDynCloneFn>::new(|e: i32| e + 1),
		/// 	action: <RcBrand as ToDynCloneFn>::new(|_: ()| 42),
		/// };
		/// assert_eq!(<LocalBrand<RcBrand, i32> as Extract>::extract(local), 42);
		/// ```
		fn extract<'a, A: 'a>(
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)
		) -> A {
			match fa {
				Local::Local {
					modify: _,
					action,
				} => action(()),
			}
		}
	}

	#[document_type_parameters("The environment type transformed by `modify`.")]
	impl<E> Extract for SendLocalBrand<ArcBrand, E>
	where
		E: Send + Sync + 'static,
	{
		/// Extracts the action from a `SendLocal` by invoking the action
		/// thunk via Arc-deref. The environment-transform closure is
		/// dropped silently.
		#[document_signature]
		///
		#[document_type_parameters("The lifetime.", "The result type.")]
		///
		#[document_parameters("The send-local effect.")]
		///
		#[document_returns("The materialised action.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		ArcBrand,
		/// 		SendLocalBrand,
		/// 	},
		/// 	classes::{
		/// 		Extract,
		/// 		ToDynSendFn,
		/// 	},
		/// 	types::effects::local::SendLocal,
		/// };
		///
		/// let local: SendLocal<'static, ArcBrand, i32, i32> = SendLocal::Local {
		/// 	modify: <ArcBrand as ToDynSendFn>::new(|e: i32| e + 1),
		/// 	action: <ArcBrand as ToDynSendFn>::new(|_: ()| 42),
		/// };
		/// assert_eq!(<SendLocalBrand<ArcBrand, i32> as Extract>::extract(local), 42);
		/// ```
		fn extract<'a, A: 'a>(
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)
		) -> A {
			match fa {
				SendLocal::Local {
					modify: _,
					action,
				} => action(()),
			}
		}
	}
}

pub use inner::*;
