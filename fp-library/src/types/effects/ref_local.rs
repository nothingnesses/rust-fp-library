//! Scoped environment-modification effect type with `Local`
//! (run an action under a transformed environment value `&E`)
//! as its sole operation. The Ref flavour: the modify closure
//! borrows the environment value (`&E -> E`) rather than
//! consuming it (the Val flavour
//! [`local`](crate::types::effects::local) ships separately with
//! `E -> E`). Removes the `E: Clone` requirement that the Val
//! flavour imposes on users who want to derive a sub-scope
//! environment from the parent without owning it.
//!
//! The corresponding brands are
//! [`BoxRefLocalBrand`](crate::brands::BoxRefLocalBrand),
//! [`RefLocalBrand`](crate::brands::RefLocalBrand), and
//! [`SendRefLocalBrand`](crate::brands::SendRefLocalBrand).
//!
//! ## Three sibling types
//!
//! `RefLocal` ships in three sibling flavours that thread different
//! per-pointer-brand closure-storage shapes for both fields. The
//! action is a unit-arg thunk and the environment-transform is a
//! 1-arg closure stored behind the same per-pointer-brand pointer;
//! the unit-arg form lets the existing pointer-abstraction
//! [`ToDynFnOnce`](crate::classes::ToDynFnOnce) /
//! [`ToDynCloneFn`](crate::classes::ToDynCloneFn) /
//! [`ToDynSendFn`](crate::classes::ToDynSendFn) family construct
//! both fields uniformly.
//!
//! - [`BoxRefLocal<'a, P, E, A>`] for default `Run` / `RunExplicit`,
//!   `P: ToDynFnOnce` (`BoxBrand` only); modify is
//!   `Box<dyn 'a + FnOnce(&E) -> E>`, action is
//!   `Box<dyn 'a + FnOnce(()) -> A>` (both single-shot).
//! - [`RefLocal<'a, P, E, A>`] for `RcRun` / `RcRunExplicit`,
//!   `P: ToDynCloneFn` (typically `RcBrand`); modify is
//!   `Rc<dyn 'a + Fn(&E) -> E>`, action is
//!   `Rc<dyn 'a + Fn(()) -> A>` (both clone-able via Rc-bump).
//! - [`SendRefLocal<'a, P, E, A>`] for `ArcRun` / `ArcRunExplicit`,
//!   `P: ToDynSendFn` (typically `ArcBrand`); modify is
//!   `Arc<dyn 'a + Fn(&E) -> E + Send + Sync>`, action is
//!   `Arc<dyn 'a + Fn(()) -> A + Send + Sync>` (both clone-able
//!   via Arc-bump and thread-safe).
//!
//! ## Why the action is a thunk
//!
//! Storing the action directly as `A` would create a substrate-
//! level layout cycle when `A` resolves to `Free<NodeBrand<R, S>, ...>`
//! and `S` contains the `RefLocal` brand: `Free`'s view holds a
//! `Node`, the `Node::Scoped` arm holds a `Coproduct` of scoped-
//! effect cells, and the `RefLocal` cell would carry an unboxed
//! `A = Free<...>` field, creating an infinite layout. The thunk
//! indirection (Box / Rc / Arc are pointer-sized regardless of
//! the closure target's layout) breaks the cycle; the unit-arg
//! form fits the existing pointer-abstraction matrix without a
//! new construction path. Mirrors the same fix applied to
//! [`Catch`](crate::types::effects::catch) and
//! [`Local`](crate::types::effects::local).

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			Apply,
			brands::{
				ArcBrand,
				BoxBrand,
				BoxRefLocalBrand,
				RcBrand,
				RefLocalBrand,
				SendRefLocalBrand,
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
	};

	// ===== BoxRefLocal (BoxBrand + FnOnce, single-shot) =====

	/// Scoped environment-modification effect (Ref flavour) for
	/// default `Run` / `RunExplicit` substrates. Both the
	/// environment-transform and the action are stored as
	/// `<P as Pointer>::Of<'a, dyn 'a + FnOnce(...) -> _>` projections
	/// (`Box<dyn FnOnce>`): the environment-transform as a 1-arg
	/// `Box<dyn FnOnce(&E) -> E>` (borrows `E`, returns owned `E`);
	/// the action as a unit-arg thunk `Box<dyn FnOnce(()) -> A>`
	/// (single-shot). The thunk-storage on the action breaks the
	/// substrate-level layout cycle that an unboxed `action: A` field
	/// would create; mirrors the [`BoxLocal`](crate::types::effects::local::BoxLocal)
	/// fix.
	#[document_type_parameters(
		"The lifetime of the modify and action closures.",
		"The pointer brand storing the closures (BoxBrand only by structural bound).",
		"The environment type borrowed by `modify`.",
		"The result type of the action."
	)]
	pub enum BoxRefLocal<'a, P, E, A>
	where
		P: ToDynFnOnce,
		E: 'a,
		A: 'a, {
		/// Run the `action` thunk under an environment transformed by
		/// `modify`; the dispatcher applies `modify` to a borrow of the
		/// inherited environment value once before invoking `action`.
		Local {
			/// The environment-transform closure
			/// (`<P as Pointer>::Of<'a, dyn 'a + FnOnce(&E) -> E>`,
			/// single-shot via [`FnOnce`]; borrows `E`).
			modify: <P as Pointer>::Of<'a, dyn 'a + FnOnce(&E) -> E>,
			/// The protected action program, stored as a unit-arg thunk
			/// (`<P as Pointer>::Of<'a, dyn 'a + FnOnce(()) -> A>`,
			/// single-shot via [`FnOnce`]).
			action: <P as Pointer>::Of<'a, dyn 'a + FnOnce(()) -> A>,
		},
	}

	impl_kind! {
		impl<P: ToDynFnOnce, E: 'static> for BoxRefLocalBrand<P, E> {
			type Of<'a, A: 'a>: 'a = BoxRefLocal<'a, P, E, A>;
		}
	}

	#[document_type_parameters("The environment type borrowed by `modify`.")]
	impl<E> Functor for BoxRefLocalBrand<BoxBrand, E>
	where
		E: 'static,
	{
		/// Maps `f` over the result type of this scoped effect.
		/// Composes `f` with the action thunk's return; the
		/// environment-transform is unchanged because it does not
		/// depend on the result type.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the continuations.",
			"The original result type.",
			"The new result type after applying `f`."
		)]
		///
		#[document_parameters("The function to apply.", "The ref-local effect.")]
		///
		#[document_returns("A new ref-local effect with `f` composed over its result type.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		BoxBrand,
		/// 		BoxRefLocalBrand,
		/// 	},
		/// 	classes::{
		/// 		Functor,
		/// 		ToDynFnOnce,
		/// 	},
		/// 	types::effects::ref_local::BoxRefLocal,
		/// };
		///
		/// let local: BoxRefLocal<'static, BoxBrand, i32, i32> = BoxRefLocal::Local {
		/// 	modify: <BoxBrand as ToDynFnOnce>::ref_new(|e: &i32| *e + 1),
		/// 	action: <BoxBrand as ToDynFnOnce>::new(|_: ()| 7),
		/// };
		/// let mapped = <BoxRefLocalBrand<BoxBrand, i32> as Functor>::map(|x: i32| x + 1, local);
		/// match mapped {
		/// 	BoxRefLocal::Local {
		/// 		modify,
		/// 		action,
		/// 	} => {
		/// 		assert_eq!(modify(&10), 11);
		/// 		assert_eq!(action(()), 8);
		/// 	}
		/// }
		/// ```
		fn map<'a, A: 'a, B: 'a>(
			f: impl Fn(A) -> B + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			match fa {
				BoxRefLocal::Local {
					modify,
					action,
				} => BoxRefLocal::Local {
					modify,
					action: <BoxBrand as ToDynFnOnce>::new(move |()| f(action(()))),
				},
			}
		}
	}

	// ===== RefLocal (RcBrand + Fn, multi-shot clone-capable) =====

	/// Scoped environment-modification effect (Ref flavour) for
	/// `RcRun` / `RcRunExplicit` substrates. Both the
	/// environment-transform and the action are stored as
	/// `<P as RefCountedPointer>::Of<'a, dyn 'a + Fn(...) -> _>`
	/// projections (`Rc<dyn Fn>`): the environment-transform as a
	/// 1-arg `Rc<dyn Fn(&E) -> E>` (borrows `E`, returns owned `E`);
	/// the action as a unit-arg thunk `Rc<dyn Fn(()) -> A>`
	/// (multi-shot). Both are clone-able (refcount-bump). The
	/// thunk-storage on the action breaks the substrate-level layout
	/// cycle described on [`BoxRefLocal`] for the same reason.
	#[document_type_parameters(
		"The lifetime of the modify and action closures.",
		"The pointer brand storing the closures (RcBrand only).",
		"The environment type borrowed by `modify`.",
		"The result type of the action."
	)]
	pub enum RefLocal<'a, P, E, A>
	where
		P: ToDynCloneFn,
		E: 'a,
		A: 'a, {
		/// Run the `action` thunk under an environment transformed by
		/// `modify`; the dispatcher applies `modify` to a borrow of the
		/// inherited environment value once before invoking `action`.
		Local {
			/// The environment-transform closure
			/// (`<P as RefCountedPointer>::Of<'a, dyn 'a + Fn(&E) -> E>`,
			/// multi-shot via [`Fn`]; clone via Rc-bump; borrows `E`).
			modify: <P as RefCountedPointer>::Of<'a, dyn 'a + Fn(&E) -> E>,
			/// The protected action program, stored as a unit-arg thunk
			/// (`<P as RefCountedPointer>::Of<'a, dyn 'a + Fn(()) -> A>`,
			/// multi-shot via [`Fn`]; clone via Rc-bump).
			action: <P as RefCountedPointer>::Of<'a, dyn 'a + Fn(()) -> A>,
		},
	}

	impl_kind! {
		impl<P: ToDynCloneFn, E: 'static> for RefLocalBrand<P, E> {
			type Of<'a, A: 'a>: 'a = RefLocal<'a, P, E, A>;
		}
	}

	#[document_type_parameters(
		"The lifetime of the modify and action closures.",
		"The pointer brand storing the closures.",
		"The environment type borrowed by `modify`.",
		"The result type of the action."
	)]
	#[document_parameters("The ref-local effect to clone.")]
	impl<'a, P, E, A> Clone for RefLocal<'a, P, E, A>
	where
		P: ToDynCloneFn,
		E: 'a,
		A: 'a,
	{
		/// Clones the ref-local effect by refcount-bumping the stored
		/// environment-transform and action thunk pointers.
		#[document_signature]
		///
		#[document_returns("A new ref-local effect sharing modify and action by refcount.")]
		///
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	core::ops::Deref,
		/// 	fp_library::{
		/// 		brands::*,
		/// 		classes::ToDynCloneFn,
		/// 		types::effects::ref_local::RefLocal,
		/// 	},
		/// };
		///
		/// let original: RefLocal<'static, RcBrand, i32, i32> = RefLocal::Local {
		/// 	modify: <RcBrand as ToDynCloneFn>::ref_new(|e: &i32| *e + 1),
		/// 	action: <RcBrand as ToDynCloneFn>::new(|_: ()| 42),
		/// };
		/// let cloned = original.clone();
		/// match cloned {
		/// 	RefLocal::Local {
		/// 		modify,
		/// 		action,
		/// 	} => {
		/// 		assert_eq!(modify.deref()(&10), 11);
		/// 		assert_eq!(action.deref()(()), 42);
		/// 	}
		/// }
		/// ```
		fn clone(&self) -> Self {
			match self {
				RefLocal::Local {
					modify,
					action,
				} => RefLocal::Local {
					modify: <P as RefCountedPointer>::Of::clone(modify),
					action: <P as RefCountedPointer>::Of::clone(action),
				},
			}
		}
	}

	#[document_type_parameters("The environment type borrowed by `modify`.")]
	impl<E> Functor for RefLocalBrand<RcBrand, E>
	where
		E: 'static,
	{
		/// Maps `f` over the result type of this scoped effect.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the continuations.",
			"The original result type.",
			"The new result type after applying `f`."
		)]
		///
		#[document_parameters("The function to apply.", "The ref-local effect.")]
		///
		#[document_returns("A new ref-local effect with `f` composed over its result type.")]
		///
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	core::ops::Deref,
		/// 	fp_library::{
		/// 		brands::{
		/// 			RcBrand,
		/// 			RefLocalBrand,
		/// 		},
		/// 		classes::{
		/// 			Functor,
		/// 			ToDynCloneFn,
		/// 		},
		/// 		types::effects::ref_local::RefLocal,
		/// 	},
		/// };
		///
		/// let local: RefLocal<'static, RcBrand, i32, i32> = RefLocal::Local {
		/// 	modify: <RcBrand as ToDynCloneFn>::ref_new(|e: &i32| *e + 1),
		/// 	action: <RcBrand as ToDynCloneFn>::new(|_: ()| 7),
		/// };
		/// let mapped = <RefLocalBrand<RcBrand, i32> as Functor>::map(|x: i32| x + 1, local);
		/// match mapped {
		/// 	RefLocal::Local {
		/// 		modify,
		/// 		action,
		/// 	} => {
		/// 		assert_eq!(modify.deref()(&10), 11);
		/// 		assert_eq!(action.deref()(()), 8);
		/// 	}
		/// }
		/// ```
		fn map<'a, A: 'a, B: 'a>(
			f: impl Fn(A) -> B + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			match fa {
				RefLocal::Local {
					modify,
					action,
				} => RefLocal::Local {
					modify,
					action: <RcBrand as ToDynCloneFn>::new(move |()| f(action(()))),
				},
			}
		}
	}

	// ===== SendRefLocal (ArcBrand + Fn + Send + Sync, thread-safe) =====

	/// Scoped environment-modification effect (Ref flavour) for
	/// `ArcRun` / `ArcRunExplicit` substrates. Both fields are stored
	/// as `<P as SendRefCountedPointer>::Of<'a, dyn 'a + Fn(...) -> _ + Send + Sync>`
	/// projections (`Arc<dyn Fn + Send + Sync>`): modify as
	/// `Arc<dyn Fn(&E) -> E + Send + Sync>`; action as
	/// `Arc<dyn Fn(()) -> A + Send + Sync>` (both clone-able and
	/// thread-safe). The thunk-storage on the action breaks the
	/// substrate-level layout cycle described on [`BoxRefLocal`] /
	/// [`RefLocal`] for the same reason.
	#[document_type_parameters(
		"The lifetime of the modify and action closures.",
		"The pointer brand storing the closures (ArcBrand only).",
		"The environment type borrowed by `modify`.",
		"The result type of the action."
	)]
	pub enum SendRefLocal<'a, P, E, A>
	where
		P: ToDynSendFn,
		E: 'a,
		A: 'a, {
		/// Run the `action` thunk under an environment transformed by
		/// `modify`; the dispatcher applies `modify` to a borrow of the
		/// inherited environment value once before invoking `action`.
		Local {
			/// The environment-transform closure
			/// (`<P as SendRefCountedPointer>::Of<'a, dyn 'a + Fn(&E) -> E + Send + Sync>`,
			/// multi-shot via [`Fn`]; clone via Arc-bump; thread-safe).
			modify: <P as SendRefCountedPointer>::Of<'a, dyn 'a + Fn(&E) -> E + Send + Sync>,
			/// The protected action program, stored as a unit-arg thunk
			/// (`<P as SendRefCountedPointer>::Of<'a, dyn 'a + Fn(()) -> A + Send + Sync>`,
			/// multi-shot via [`Fn`]; clone via Arc-bump; thread-safe).
			action: <P as SendRefCountedPointer>::Of<'a, dyn 'a + Fn(()) -> A + Send + Sync>,
		},
	}

	impl_kind! {
		impl<P: ToDynSendFn, E: Send + Sync + 'static> for SendRefLocalBrand<P, E> {
			type Of<'a, A: 'a>: 'a = SendRefLocal<'a, P, E, A>;
		}
	}

	#[document_type_parameters(
		"The lifetime of the modify and action closures.",
		"The pointer brand storing the closures.",
		"The environment type borrowed by `modify`.",
		"The result type of the action."
	)]
	#[document_parameters("The send-ref-local effect to clone.")]
	impl<'a, P, E, A> Clone for SendRefLocal<'a, P, E, A>
	where
		P: ToDynSendFn,
		E: 'a,
		A: 'a,
	{
		/// Clones the send-ref-local effect by refcount-bumping the
		/// stored environment-transform and action thunk pointers.
		#[document_signature]
		///
		#[document_returns("A new send-ref-local effect sharing modify and action by refcount.")]
		///
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	core::ops::Deref,
		/// 	fp_library::{
		/// 		brands::*,
		/// 		classes::ToDynSendFn,
		/// 		types::effects::ref_local::SendRefLocal,
		/// 	},
		/// };
		///
		/// let original: SendRefLocal<'static, ArcBrand, i32, i32> = SendRefLocal::Local {
		/// 	modify: <ArcBrand as ToDynSendFn>::ref_new(|e: &i32| *e + 1),
		/// 	action: <ArcBrand as ToDynSendFn>::new(|_: ()| 42),
		/// };
		/// let cloned = original.clone();
		/// match cloned {
		/// 	SendRefLocal::Local {
		/// 		modify,
		/// 		action,
		/// 	} => {
		/// 		assert_eq!(modify.deref()(&10), 11);
		/// 		assert_eq!(action.deref()(()), 42);
		/// 	}
		/// }
		/// ```
		fn clone(&self) -> Self {
			match self {
				SendRefLocal::Local {
					modify,
					action,
				} => SendRefLocal::Local {
					modify: <P as SendRefCountedPointer>::Of::clone(modify),
					action: <P as SendRefCountedPointer>::Of::clone(action),
				},
			}
		}
	}

	// ===== SendFunctor for SendRefLocalBrand (load-bearing on Arc family) =====

	#[document_type_parameters("The environment type borrowed by `modify`.")]
	impl<E> SendFunctor for SendRefLocalBrand<ArcBrand, E>
	where
		E: Send + Sync + 'static,
	{
		/// Thread-safe map over the result type. The
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
			"The send-ref-local effect."
		)]
		///
		#[document_returns("A new send-ref-local effect with `f` composed over its result type.")]
		///
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	core::ops::Deref,
		/// 	fp_library::{
		/// 		brands::{
		/// 			ArcBrand,
		/// 			SendRefLocalBrand,
		/// 		},
		/// 		classes::{
		/// 			SendFunctor,
		/// 			ToDynSendFn,
		/// 		},
		/// 		types::effects::ref_local::SendRefLocal,
		/// 	},
		/// };
		///
		/// let local: SendRefLocal<'static, ArcBrand, i32, i32> = SendRefLocal::Local {
		/// 	modify: <ArcBrand as ToDynSendFn>::ref_new(|e: &i32| *e + 1),
		/// 	action: <ArcBrand as ToDynSendFn>::new(|_: ()| 7),
		/// };
		/// let mapped = <SendRefLocalBrand<ArcBrand, i32> as SendFunctor>::send_map(|x: i32| x + 1, local);
		/// match mapped {
		/// 	SendRefLocal::Local {
		/// 		modify,
		/// 		action,
		/// 	} => {
		/// 		assert_eq!(modify.deref()(&10), 11);
		/// 		assert_eq!(action.deref()(()), 8);
		/// 	}
		/// }
		/// ```
		fn send_map<'a, A: Send + Sync + 'a, B: Send + Sync + 'a>(
			f: impl Fn(A) -> B + Send + Sync + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			match fa {
				SendRefLocal::Local {
					modify,
					action,
				} => SendRefLocal::Local {
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

	#[document_type_parameters("The environment type borrowed by `modify`.")]
	impl<E> SendFunctor for BoxRefLocalBrand<BoxBrand, E>
	where
		E: Send + Sync + 'static,
	{
		/// SendFunctor stub for the BoxBrand-flavoured RefLocal.
		/// The projection `Box<dyn FnOnce>` is not `Send + Sync`, so
		/// this brand does not appear in Arc-family substrates and the
		/// SendFunctor path is unreachable in practice. Delegates to
		/// [`Functor::map`].
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the continuations.",
			"The original result type.",
			"The new result type."
		)]
		///
		#[document_parameters("The function to apply.", "The ref-local effect.")]
		///
		#[document_returns("A new ref-local effect with `f` composed over its result type.")]
		///
		#[document_examples]
		///
		/// ```
		/// // Exercised via brand-dispatch through NodeBrand<R, S>: SendFunctor
		/// // when S = CoproductBrand<BoxRefLocalBrand<BoxBrand, E>, _>; never
		/// // reached at runtime on default Run substrates.
		/// use fp_library::{
		/// 	brands::{
		/// 		BoxBrand,
		/// 		BoxRefLocalBrand,
		/// 	},
		/// 	classes::{
		/// 		SendFunctor,
		/// 		ToDynFnOnce,
		/// 	},
		/// 	types::effects::ref_local::BoxRefLocal,
		/// };
		///
		/// let local: BoxRefLocal<'static, BoxBrand, i32, i32> = BoxRefLocal::Local {
		/// 	modify: <BoxBrand as ToDynFnOnce>::ref_new(|e: &i32| *e + 1),
		/// 	action: <BoxBrand as ToDynFnOnce>::new(|_: ()| 7),
		/// };
		/// let mapped = <BoxRefLocalBrand<BoxBrand, i32> as SendFunctor>::send_map(|x: i32| x + 1, local);
		/// match mapped {
		/// 	BoxRefLocal::Local {
		/// 		modify,
		/// 		action,
		/// 	} => {
		/// 		assert_eq!(modify(&10), 11);
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

	#[document_type_parameters("The environment type borrowed by `modify`.")]
	impl<E> SendFunctor for RefLocalBrand<RcBrand, E>
	where
		E: Send + Sync + 'static,
	{
		/// SendFunctor stub for the RcBrand-flavoured RefLocal.
		/// The projection `Rc<dyn Fn>` is not `Send + Sync`, so this
		/// brand does not appear in Arc-family substrates; the
		/// SendFunctor path is reachable only through brand-dispatch
		/// but never exercised at runtime. Delegates to
		/// [`Functor::map`].
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the continuations.",
			"The original result type.",
			"The new result type."
		)]
		///
		#[document_parameters("The function to apply.", "The ref-local effect.")]
		///
		#[document_returns("A new ref-local effect with `f` composed over its result type.")]
		///
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	core::ops::Deref,
		/// 	fp_library::{
		/// 		brands::{
		/// 			RcBrand,
		/// 			RefLocalBrand,
		/// 		},
		/// 		classes::{
		/// 			SendFunctor,
		/// 			ToDynCloneFn,
		/// 		},
		/// 		types::effects::ref_local::RefLocal,
		/// 	},
		/// };
		///
		/// let local: RefLocal<'static, RcBrand, i32, i32> = RefLocal::Local {
		/// 	modify: <RcBrand as ToDynCloneFn>::ref_new(|e: &i32| *e + 1),
		/// 	action: <RcBrand as ToDynCloneFn>::new(|_: ()| 7),
		/// };
		/// let mapped = <RefLocalBrand<RcBrand, i32> as SendFunctor>::send_map(|x: i32| x + 1, local);
		/// match mapped {
		/// 	RefLocal::Local {
		/// 		modify,
		/// 		action,
		/// 	} => {
		/// 		assert_eq!(modify.deref()(&10), 11);
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

	#[document_type_parameters("The environment type borrowed by `modify`.")]
	impl<E> WrapDrop for BoxRefLocalBrand<BoxBrand, E>
	where
		E: 'static,
	{
		/// Drop-time decomposition for `BoxRefLocal`: invokes the
		/// action thunk to materialise the action program and returns
		/// `Some(action)` so the substrate's iterative `Drop` path can
		/// continue walking the program tree. The
		/// environment-transform closure is dropped silently.
		#[document_signature]
		///
		#[document_type_parameters("The lifetime.", "The result type.")]
		///
		#[document_parameters("The ref-local effect to decompose.")]
		///
		#[document_returns("`Some` of the materialised action; the modify closure is dropped.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		BoxBrand,
		/// 		BoxRefLocalBrand,
		/// 	},
		/// 	classes::{
		/// 		ToDynFnOnce,
		/// 		WrapDrop,
		/// 	},
		/// 	types::effects::ref_local::BoxRefLocal,
		/// };
		///
		/// let local: BoxRefLocal<'static, BoxBrand, i32, i32> = BoxRefLocal::Local {
		/// 	modify: <BoxBrand as ToDynFnOnce>::ref_new(|e: &i32| *e + 1),
		/// 	action: <BoxBrand as ToDynFnOnce>::new(|_: ()| 42),
		/// };
		/// assert_eq!(<BoxRefLocalBrand<BoxBrand, i32> as WrapDrop>::drop(local), Some(42));
		/// ```
		fn drop<'a, X: 'a>(
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, X>)
		) -> Option<X> {
			match fa {
				BoxRefLocal::Local {
					modify: _,
					action,
				} => Some(action(())),
			}
		}
	}

	#[document_type_parameters("The environment type borrowed by `modify`.")]
	impl<E> WrapDrop for RefLocalBrand<RcBrand, E>
	where
		E: 'static,
	{
		/// Drop-time decomposition for `RefLocal`. Same shape as
		/// [`BoxRefLocalBrand`'s drop](BoxRefLocalBrand): calls the
		/// action thunk via Rc-deref and returns `Some(action)`,
		/// drops the environment-transform closure silently.
		#[document_signature]
		///
		#[document_type_parameters("The lifetime.", "The result type.")]
		///
		#[document_parameters("The ref-local effect to decompose.")]
		///
		#[document_returns("`Some` of the materialised action; the modify closure is dropped.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		RcBrand,
		/// 		RefLocalBrand,
		/// 	},
		/// 	classes::{
		/// 		ToDynCloneFn,
		/// 		WrapDrop,
		/// 	},
		/// 	types::effects::ref_local::RefLocal,
		/// };
		///
		/// let local: RefLocal<'static, RcBrand, i32, i32> = RefLocal::Local {
		/// 	modify: <RcBrand as ToDynCloneFn>::ref_new(|e: &i32| *e + 1),
		/// 	action: <RcBrand as ToDynCloneFn>::new(|_: ()| 42),
		/// };
		/// assert_eq!(<RefLocalBrand<RcBrand, i32> as WrapDrop>::drop(local), Some(42));
		/// ```
		fn drop<'a, X: 'a>(
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, X>)
		) -> Option<X> {
			match fa {
				RefLocal::Local {
					modify: _,
					action,
				} => Some(action(())),
			}
		}
	}

	#[document_type_parameters("The environment type borrowed by `modify`.")]
	impl<E> WrapDrop for SendRefLocalBrand<ArcBrand, E>
	where
		E: Send + Sync + 'static,
	{
		/// Drop-time decomposition for `SendRefLocal`. Same shape as
		/// [`BoxRefLocalBrand`'s drop](BoxRefLocalBrand): calls the
		/// action thunk via Arc-deref and returns `Some(action)`,
		/// drops the environment-transform closure silently.
		#[document_signature]
		///
		#[document_type_parameters("The lifetime.", "The result type.")]
		///
		#[document_parameters("The send-ref-local effect to decompose.")]
		///
		#[document_returns("`Some` of the materialised action; the modify closure is dropped.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		ArcBrand,
		/// 		SendRefLocalBrand,
		/// 	},
		/// 	classes::{
		/// 		ToDynSendFn,
		/// 		WrapDrop,
		/// 	},
		/// 	types::effects::ref_local::SendRefLocal,
		/// };
		///
		/// let local: SendRefLocal<'static, ArcBrand, i32, i32> = SendRefLocal::Local {
		/// 	modify: <ArcBrand as ToDynSendFn>::ref_new(|e: &i32| *e + 1),
		/// 	action: <ArcBrand as ToDynSendFn>::new(|_: ()| 42),
		/// };
		/// assert_eq!(<SendRefLocalBrand<ArcBrand, i32> as WrapDrop>::drop(local), Some(42));
		/// ```
		fn drop<'a, X: 'a>(
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, X>)
		) -> Option<X> {
			match fa {
				SendRefLocal::Local {
					modify: _,
					action,
				} => Some(action(())),
			}
		}
	}

	// ===== Extract impls =====

	#[document_type_parameters("The environment type borrowed by `modify`.")]
	impl<E> Extract for BoxRefLocalBrand<BoxBrand, E>
	where
		E: 'static,
	{
		/// Extracts the action from a `BoxRefLocal` by invoking the
		/// action thunk. The environment-transform closure is dropped
		/// silently.
		#[document_signature]
		///
		#[document_type_parameters("The lifetime.", "The result type.")]
		///
		#[document_parameters("The ref-local effect.")]
		///
		#[document_returns("The materialised action.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		BoxBrand,
		/// 		BoxRefLocalBrand,
		/// 	},
		/// 	classes::{
		/// 		Extract,
		/// 		ToDynFnOnce,
		/// 	},
		/// 	types::effects::ref_local::BoxRefLocal,
		/// };
		///
		/// let local: BoxRefLocal<'static, BoxBrand, i32, i32> = BoxRefLocal::Local {
		/// 	modify: <BoxBrand as ToDynFnOnce>::ref_new(|e: &i32| *e + 1),
		/// 	action: <BoxBrand as ToDynFnOnce>::new(|_: ()| 42),
		/// };
		/// assert_eq!(<BoxRefLocalBrand<BoxBrand, i32> as Extract>::extract(local), 42);
		/// ```
		fn extract<'a, A: 'a>(
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)
		) -> A {
			match fa {
				BoxRefLocal::Local {
					modify: _,
					action,
				} => action(()),
			}
		}
	}

	#[document_type_parameters("The environment type borrowed by `modify`.")]
	impl<E> Extract for RefLocalBrand<RcBrand, E>
	where
		E: 'static,
	{
		/// Extracts the action from a `RefLocal` by invoking the
		/// action thunk via Rc-deref. The environment-transform
		/// closure is dropped silently.
		#[document_signature]
		///
		#[document_type_parameters("The lifetime.", "The result type.")]
		///
		#[document_parameters("The ref-local effect.")]
		///
		#[document_returns("The materialised action.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		RcBrand,
		/// 		RefLocalBrand,
		/// 	},
		/// 	classes::{
		/// 		Extract,
		/// 		ToDynCloneFn,
		/// 	},
		/// 	types::effects::ref_local::RefLocal,
		/// };
		///
		/// let local: RefLocal<'static, RcBrand, i32, i32> = RefLocal::Local {
		/// 	modify: <RcBrand as ToDynCloneFn>::ref_new(|e: &i32| *e + 1),
		/// 	action: <RcBrand as ToDynCloneFn>::new(|_: ()| 42),
		/// };
		/// assert_eq!(<RefLocalBrand<RcBrand, i32> as Extract>::extract(local), 42);
		/// ```
		fn extract<'a, A: 'a>(
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)
		) -> A {
			match fa {
				RefLocal::Local {
					modify: _,
					action,
				} => action(()),
			}
		}
	}

	#[document_type_parameters("The environment type borrowed by `modify`.")]
	impl<E> Extract for SendRefLocalBrand<ArcBrand, E>
	where
		E: Send + Sync + 'static,
	{
		/// Extracts the action from a `SendRefLocal` by invoking the
		/// action thunk via Arc-deref. The environment-transform
		/// closure is dropped silently.
		#[document_signature]
		///
		#[document_type_parameters("The lifetime.", "The result type.")]
		///
		#[document_parameters("The send-ref-local effect.")]
		///
		#[document_returns("The materialised action.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		ArcBrand,
		/// 		SendRefLocalBrand,
		/// 	},
		/// 	classes::{
		/// 		Extract,
		/// 		ToDynSendFn,
		/// 	},
		/// 	types::effects::ref_local::SendRefLocal,
		/// };
		///
		/// let local: SendRefLocal<'static, ArcBrand, i32, i32> = SendRefLocal::Local {
		/// 	modify: <ArcBrand as ToDynSendFn>::ref_new(|e: &i32| *e + 1),
		/// 	action: <ArcBrand as ToDynSendFn>::new(|_: ()| 42),
		/// };
		/// assert_eq!(<SendRefLocalBrand<ArcBrand, i32> as Extract>::extract(local), 42);
		/// ```
		fn extract<'a, A: 'a>(
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)
		) -> A {
			match fa {
				SendRefLocal::Local {
					modify: _,
					action,
				} => action(()),
			}
		}
	}

	// ===== Brand-projection helpers (RefFunctor support) =====
	//
	// Mirrors the [`local_modify_ref`](crate::types::effects::local::local_modify_ref)
	// and [`local_action_thunk_ref`](crate::types::effects::local::local_action_thunk_ref)
	// pattern from the Val cycle: `RefFunctor::ref_map` takes
	// `fa: &<Self as Kind>::Of<'a, A>`; the compiler refuses to unify
	// that reference with the concrete `&BoxRefLocal<...>` /
	// `&RefLocal<...>` enum inside the impl's HRTB-bearing scope.
	// Free functions whose where-clauses carry only `Kind` bounds
	// normalize cleanly.
	//
	// Per the B-thunk action representation, the `BoxRefLocal`
	// variant's `action: Box<dyn FnOnce(()) -> A>` cannot be invoked
	// through a reference and the `modify: Box<dyn FnOnce(&E) -> E>`
	// cannot be cloned through a reference either, so
	// [`BoxRefLocalBrand`'s `RefFunctor` impl] is a panicking stub on
	// both fields and does not need projection helpers. The
	// Rc-flavoured `RefLocal` variant's
	// `action: Rc<dyn Fn(()) -> A>` is callable via Rc-deref and its
	// `modify: Rc<dyn Fn(&E) -> E>` is cloneable via Rc-bump, so its
	// `RefFunctor` impl meaningfully composes via two helpers
	// ([`ref_local_modify_ref`] and [`ref_local_action_thunk_ref`])
	// extracting the Rc-shared cells.

	/// Projects a modify-closure reference out of a [`RefLocal`] GAT
	/// projection. Used inside [`RefLocalBrand`'s `RefFunctor` impl]
	/// to clone the `Rc`-shared modify closure without consuming the
	/// ref-local effect.
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the ref-local effect's contents.",
		"The borrow lifetime of the input projection.",
		"The pointer brand storing the modify closure (RcBrand only).",
		"The environment type borrowed by `modify`.",
		"The result type of the action."
	)]
	///
	#[document_parameters("The ref-local effect projection.")]
	///
	#[document_returns("A reference to the modify-closure pointer stored in the ref-local effect.")]
	///
	#[document_examples]
	///
	/// ```
	/// use {
	/// 	core::ops::Deref,
	/// 	fp_library::{
	/// 		brands::RcBrand,
	/// 		classes::ToDynCloneFn,
	/// 		types::effects::ref_local::{
	/// 			RefLocal,
	/// 			ref_local_modify_ref,
	/// 		},
	/// 	},
	/// };
	///
	/// let local: RefLocal<'static, RcBrand, i32, i32> = RefLocal::Local {
	/// 	modify: <RcBrand as ToDynCloneFn>::ref_new(|e: &i32| *e + 1),
	/// 	action: <RcBrand as ToDynCloneFn>::new(|_: ()| 42),
	/// };
	/// let modify = ref_local_modify_ref::<RcBrand, i32, i32>(&local);
	/// assert_eq!(modify.deref()(&10), 11);
	/// ```
	#[doc(hidden)]
	pub fn ref_local_modify_ref<'a, 'b, P, E, A>(
		fa: &'b Apply!(<RefLocalBrand<P, E> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)
	) -> &'b <P as RefCountedPointer>::Of<'a, dyn 'a + Fn(&E) -> E>
	where
		P: ToDynCloneFn,
		E: 'static,
		A: 'a, {
		match fa {
			RefLocal::Local {
				modify,
				action: _,
			} => modify,
		}
	}

	/// Projects an action-thunk reference out of a [`RefLocal`] GAT
	/// projection. Used inside [`RefLocalBrand`'s `RefFunctor` impl]
	/// to clone the `Rc`-shared action thunk without consuming the
	/// ref-local effect.
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the ref-local effect's contents.",
		"The borrow lifetime of the input projection.",
		"The pointer brand storing the action thunk (RcBrand only).",
		"The environment type borrowed by `modify`.",
		"The result type of the action."
	)]
	///
	#[document_parameters("The ref-local effect projection.")]
	///
	#[document_returns("A reference to the action-thunk pointer stored in the ref-local effect.")]
	///
	#[document_examples]
	///
	/// ```
	/// use {
	/// 	core::ops::Deref,
	/// 	fp_library::{
	/// 		brands::RcBrand,
	/// 		classes::ToDynCloneFn,
	/// 		types::effects::ref_local::{
	/// 			RefLocal,
	/// 			ref_local_action_thunk_ref,
	/// 		},
	/// 	},
	/// };
	///
	/// let local: RefLocal<'static, RcBrand, i32, i32> = RefLocal::Local {
	/// 	modify: <RcBrand as ToDynCloneFn>::ref_new(|e: &i32| *e + 1),
	/// 	action: <RcBrand as ToDynCloneFn>::new(|_: ()| 42),
	/// };
	/// let action_thunk = ref_local_action_thunk_ref::<RcBrand, i32, i32>(&local);
	/// assert_eq!(action_thunk.deref()(()), 42);
	/// ```
	#[doc(hidden)]
	pub fn ref_local_action_thunk_ref<'a, 'b, P, E, A>(
		fa: &'b Apply!(<RefLocalBrand<P, E> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)
	) -> &'b <P as RefCountedPointer>::Of<'a, dyn 'a + Fn(()) -> A>
	where
		P: ToDynCloneFn,
		E: 'static,
		A: 'a, {
		match fa {
			RefLocal::Local {
				modify: _,
				action,
			} => action,
		}
	}

	// ===== RefFunctor impls =====

	#[document_type_parameters("The environment type borrowed by `modify`.")]
	impl<E> RefFunctor for BoxRefLocalBrand<BoxBrand, E>
	where
		E: 'static,
	{
		/// Maps `func` over the result type by reference. Both the
		/// modify closure (`Box<dyn FnOnce(&E) -> E>`) and the action
		/// thunk (`Box<dyn FnOnce(()) -> A>`) cannot be re-built from a
		/// reference (Box is not [`Clone`] and [`FnOnce::call_once`]
		/// requires owned `self`), so both new closures are panicking
		/// stubs. This entire impl is structurally unreachable in real
		/// programs because
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
			"The ref-local effect projection (ignored; new closures are stubs)."
		)]
		///
		#[document_returns(
			"A new ref-local effect with both modify and action closures as panicking stubs."
		)]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		BoxBrand,
		/// 		BoxRefLocalBrand,
		/// 	},
		/// 	classes::{
		/// 		RefFunctor,
		/// 		ToDynFnOnce,
		/// 	},
		/// 	types::effects::ref_local::BoxRefLocal,
		/// };
		///
		/// let local: BoxRefLocal<'static, BoxBrand, i32, i32> = BoxRefLocal::Local {
		/// 	modify: <BoxBrand as ToDynFnOnce>::ref_new(|e: &i32| *e + 1),
		/// 	action: <BoxBrand as ToDynFnOnce>::new(|_: ()| 7),
		/// };
		/// // Stub-only impl: the returned BoxRefLocal's modify and action are
		/// // panicking thunks; we only verify the variant tag here.
		/// let mapped = <BoxRefLocalBrand<BoxBrand, i32> as RefFunctor>::ref_map(|x: &i32| *x + 1, &local);
		/// assert!(matches!(mapped, BoxRefLocal::Local { .. }));
		/// ```
		#[expect(
			clippy::unreachable,
			reason = "BoxRefLocalBrand::ref_map cannot replicate FnOnce closures through a reference (Box<dyn FnOnce> is uncloneable and FnOnce::call_once requires owned self). The path is reachable only through synthetic non-Coyoneda first-order rows on RunExplicit, which real programs do not exercise."
		)]
		fn ref_map<'a, A: 'a, B: 'a>(
			_func: impl Fn(&A) -> B + 'a,
			_fa: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			BoxRefLocal::Local {
				modify: <BoxBrand as ToDynFnOnce>::ref_new(|_e: &E| -> E {
					unreachable!(
						"BoxRefLocalBrand::ref_map's stub modify invoked; the FnOnce modify closure cannot be replicated through a reference"
					)
				}),
				action: <BoxBrand as ToDynFnOnce>::new(|_: ()| -> B {
					unreachable!(
						"BoxRefLocalBrand::ref_map's stub action invoked; the FnOnce action thunk cannot be replicated through a reference"
					)
				}),
			}
		}
	}

	#[document_type_parameters("The environment type borrowed by `modify`.")]
	impl<E> RefFunctor for RefLocalBrand<RcBrand, E>
	where
		E: 'static,
	{
		/// Maps `func` over the result type by reference. The
		/// `Rc<dyn Fn>` modify closure is cloned (refcount bump) and
		/// preserved unchanged in the new ref-local effect (modify's
		/// signature `&E -> E` does not depend on the result type).
		/// The action thunk is similarly cloned, then composed with
		/// `func` to produce a new action thunk that calls the
		/// original action and applies `func` to the result by
		/// reference.
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
			"The ref-local effect projection."
		)]
		///
		#[document_returns(
			"A new ref-local effect with the original modify (refcount-shared) and `func` post-composed onto the action thunk."
		)]
		///
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	core::ops::Deref,
		/// 	fp_library::{
		/// 		brands::{
		/// 			RcBrand,
		/// 			RefLocalBrand,
		/// 		},
		/// 		classes::{
		/// 			RefFunctor,
		/// 			ToDynCloneFn,
		/// 		},
		/// 		types::effects::ref_local::RefLocal,
		/// 	},
		/// };
		///
		/// let local: RefLocal<'static, RcBrand, i32, i32> = RefLocal::Local {
		/// 	modify: <RcBrand as ToDynCloneFn>::ref_new(|e: &i32| *e + 1),
		/// 	action: <RcBrand as ToDynCloneFn>::new(|_: ()| 7),
		/// };
		/// let mapped = <RefLocalBrand<RcBrand, i32> as RefFunctor>::ref_map(|x: &i32| *x + 1, &local);
		/// match mapped {
		/// 	RefLocal::Local {
		/// 		modify,
		/// 		action,
		/// 	} => {
		/// 		assert_eq!(modify.deref()(&10), 11);
		/// 		assert_eq!(action.deref()(()), 8);
		/// 	}
		/// }
		/// ```
		fn ref_map<'a, A: 'a, B: 'a>(
			func: impl Fn(&A) -> B + 'a,
			fa: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			let modify_ref = ref_local_modify_ref::<RcBrand, E, A>(fa);
			let action_thunk_ref = ref_local_action_thunk_ref::<RcBrand, E, A>(fa);
			let modify_clone = <RcBrand as RefCountedPointer>::Of::clone(modify_ref);
			let action_thunk_clone = <RcBrand as RefCountedPointer>::Of::clone(action_thunk_ref);
			let new_action = <RcBrand as ToDynCloneFn>::new::<(), B>(move |()| -> B {
				let a: A = action_thunk_clone(());
				func(&a)
			});
			RefLocal::Local {
				modify: modify_clone,
				action: new_action,
			}
		}
	}

	// SendRefLocalBrand does not implement RefFunctor: the cascade
	// through `ArcRunExplicitBrand: RefFunctor` does not require it
	// (`ArcFreeExplicitBrand: !RefFunctor` per the brand's docs), and
	// a hypothetical impl would face the same `Send + Sync` bound
	// mismatch on `func` that prevents `Functor`. Mirrors the
	// `SendLocalBrand`-no-`RefFunctor` and
	// `SendCatchBrand`-no-`RefFunctor` precedents.
}

pub use inner::*;
