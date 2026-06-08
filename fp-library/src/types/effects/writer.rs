//! Log-emitting first-order effect type with the `Tell`
//! operation. The corresponding brand is
//! [`WriterBrand`](crate::brands::WriterBrand).
//!
//! `Writer<'a, W, A>` mirrors PureScript Run's
//! `Writer w a = Writer w a` shape directly. The single `Tell`
//! variant carries a log value `W` and the next program's value
//! `A` together; the `Functor` instance composes a user-supplied
//! `f: A -> B` with the stored `A` to produce a new
//! `Writer<'a, W, B>`, leaving the log value untouched.
//!
//! ## No pointer-brand parameter
//!
//! Unlike [`State`](crate::types::effects::state::State) or
//! [`Reader`](crate::types::effects::reader::Reader), `Writer`
//! has no `dyn Fn` continuation, so it does not parameterise over
//! a pointer brand `P` and does not need a parallel `SendWriter`
//! for the Arc family. The same `Writer<'a, W, A>` type serves
//! all six Run wrappers; per-wrapper smart constructors handle
//! the `Send + Sync` cascade on `W` alone.
//!
//! See the parent [`effects`](crate::types::effects) guide for the
//! consolidated wrapper and pointer-brand conventions.

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			Apply,
			brands::{
				ArcBrand,
				BoxBrand,
				BoxWriterCensorBrand,
				BoxWriterListenBrand,
				RcBrand,
				SendWriterCensorBrand,
				SendWriterListenBrand,
				WriterBrand,
				WriterCensorBrand,
				WriterListenBrand,
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
				ToDynFn,
				ToDynFnOnce,
				ToDynSendFn,
				WrapDrop,
			},
			impl_kind,
			kinds::*,
		},
		core::marker::PhantomData,
		fp_macros::*,
	};

	/// Log-emitting first-order effect type.
	///
	/// The single `Tell` variant carries a log value `W` and the
	/// next program's value `A` together. Mirrors PureScript Run's
	/// `Writer w a`.
	#[document_type_parameters(
		"The lifetime of the effect (carried for `Kind`-projection purposes only; nothing in the variants borrows from it).",
		"The log type.",
		"The result type produced by running the effect."
	)]
	pub enum Writer<'a, W, A: 'a> {
		/// Emit a log value of type `W` and continue with the next
		/// program's value `A`. The `Functor` instance composes a
		/// user-supplied `f: A -> B` with the stored `A`; the log
		/// value is carried unchanged.
		Tell(W, A, PhantomData<&'a ()>),
	}

	impl_kind! {
		impl<W: 'static> for WriterBrand<W> {
			type Of<'a, A: 'a>: 'a = Writer<'a, W, A>;
		}
	}

	#[document_type_parameters("The lifetime of the effect.", "The log type.", "The result type.")]
	#[document_parameters("The writer effect to clone.")]
	impl<'a, W, A> Clone for Writer<'a, W, A>
	where
		W: Clone,
		A: Clone + 'a,
	{
		/// Clones the writer effect by cloning the carried log
		/// value and the next program's value.
		#[document_signature]
		///
		#[document_returns("A new writer effect carrying clones of the log and the next value.")]
		///
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	core::marker::PhantomData,
		/// 	fp_library::types::effects::writer::Writer,
		/// };
		///
		/// let original: Writer<'static, &'static str, i32> = Writer::Tell("logged", 42, PhantomData);
		/// let cloned = original.clone();
		/// match cloned {
		/// 	Writer::Tell(log, next, _) => {
		/// 		assert_eq!(log, "logged");
		/// 		assert_eq!(next, 42);
		/// 	}
		/// }
		/// ```
		fn clone(&self) -> Self {
			match self {
				Writer::Tell(log, next, _) => Writer::Tell(log.clone(), next.clone(), PhantomData),
			}
		}
	}

	#[document_type_parameters("The log type.")]
	impl<W> Functor for WriterBrand<W>
	where
		W: 'static,
	{
		/// Maps `f` over the next-program value of this writer
		/// effect.
		///
		/// The log value is carried unchanged; only the stored `A`
		/// is replaced with `f(A)`.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the effect.",
			"The original next-program type.",
			"The new next-program type after applying `f`."
		)]
		///
		#[document_parameters(
			"The function to apply to the next-program value.",
			"The writer effect to map over."
		)]
		///
		#[document_returns(
			"A new writer effect with the same log value and `f` applied to the next-program value."
		)]
		///
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	core::marker::PhantomData,
		/// 	fp_library::{
		/// 		brands::*,
		/// 		classes::*,
		/// 		types::effects::writer::Writer,
		/// 	},
		/// };
		///
		/// let tell: Writer<'static, &'static str, i32> = Writer::Tell("logged", 7, PhantomData);
		/// let mapped: Writer<'static, &'static str, i32> =
		/// 	<WriterBrand<&'static str> as Functor>::map(|x: i32| x * 2, tell);
		/// match mapped {
		/// 	Writer::Tell(log, next, _) => {
		/// 		assert_eq!(log, "logged");
		/// 		assert_eq!(next, 14);
		/// 	}
		/// }
		/// ```
		fn map<'a, A: 'a, B: 'a>(
			f: impl Fn(A) -> B + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			match fa {
				Writer::Tell(log, next, _) => Writer::Tell(log, f(next), PhantomData),
			}
		}
	}

	#[document_type_parameters("The log type.")]
	impl<W> SendFunctor for WriterBrand<W>
	where
		W: Send + Sync + 'static,
	{
		/// Maps `f` over the next-program value of this writer
		/// effect, with `Send + Sync` bounds so the operation
		/// composes inside thread-safe contexts.
		///
		/// Body is structurally identical to [`Functor::map`]'s; the
		/// log value is carried unchanged.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the effect.",
			"The original next-program type.",
			"The new next-program type after applying `f`."
		)]
		///
		#[document_parameters(
			"The function to apply to the next-program value. Must be `Send + Sync`.",
			"The writer effect to map over."
		)]
		///
		#[document_returns(
			"A new writer effect with the same log value and `f` applied to the next-program value."
		)]
		///
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	core::marker::PhantomData,
		/// 	fp_library::{
		/// 		brands::*,
		/// 		classes::*,
		/// 		types::effects::writer::Writer,
		/// 	},
		/// };
		///
		/// let tell: Writer<'static, &'static str, i32> = Writer::Tell("logged", 7, PhantomData);
		/// let mapped: Writer<'static, &'static str, i32> =
		/// 	<WriterBrand<&'static str> as SendFunctor>::send_map(|x: i32| x * 2, tell);
		/// match mapped {
		/// 	Writer::Tell(log, next, _) => {
		/// 		assert_eq!(log, "logged");
		/// 		assert_eq!(next, 14);
		/// 	}
		/// }
		/// ```
		fn send_map<'a, A: Send + Sync + 'a, B: Send + Sync + 'a>(
			f: impl Fn(A) -> B + Send + Sync + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			match fa {
				Writer::Tell(log, next, _) => Writer::Tell(log, f(next), PhantomData),
			}
		}
	}

	// ===== Scoped Writer: BoxWriterCensor / WriterCensor / SendWriterCensor =====

	/// Scoped Writer `censor` effect for default `Run` /
	/// `RunExplicit` substrates.
	///
	/// The cell stores the selected action as a unit-argument
	/// single-shot thunk and stores a neutral log transformation. The
	/// standard handler decides whether that transformation is applied
	/// before each `Tell` is accumulated or after the selected action's
	/// log is accumulated.
	#[document_type_parameters(
		"The lifetime of the stored closures.",
		"The pointer brand storing the closures (BoxBrand only by structural bound).",
		"The log type transformed by the censor function.",
		"The selected action program type."
	)]
	pub enum BoxWriterCensor<'a, P, W, A>
	where
		P: ToDynFn + ToDynFnOnce,
		W: 'a,
		A: 'a, {
		/// Run `action` under a neutral Writer log transformation.
		Censor {
			/// The log transformation. Handler selection determines
			/// whether it applies before or after accumulation.
			censor: <P as Pointer>::Of<'a, dyn 'a + Fn(W) -> W>,
			/// The selected action program, stored as a single-shot thunk.
			action: <P as Pointer>::Of<'a, dyn 'a + FnOnce(()) -> A>,
		},
	}

	impl_kind! {
		impl<P: ToDynFn + ToDynFnOnce, W: 'static> for BoxWriterCensorBrand<P, W> {
			type Of<'a, A: 'a>: 'a = BoxWriterCensor<'a, P, W, A>;
		}
	}

	/// Scoped Writer `censor` effect for `RcRun` /
	/// `RcRunExplicit` substrates.
	#[document_type_parameters(
		"The lifetime of the stored closures.",
		"The pointer brand storing the closures (RcBrand only).",
		"The log type transformed by the censor function.",
		"The selected action program type."
	)]
	pub enum WriterCensor<'a, P, W, A>
	where
		P: ToDynCloneFn,
		W: 'a,
		A: 'a, {
		/// Run `action` under a neutral Writer log transformation.
		Censor {
			/// The log transformation. Handler selection determines
			/// whether it applies before or after accumulation.
			censor: <P as RefCountedPointer>::Of<'a, dyn 'a + Fn(W) -> W>,
			/// The selected action program, stored as a multi-shot thunk.
			action: <P as RefCountedPointer>::Of<'a, dyn 'a + Fn(()) -> A>,
		},
	}

	impl_kind! {
		impl<P: ToDynCloneFn, W: 'static> for WriterCensorBrand<P, W> {
			type Of<'a, A: 'a>: 'a = WriterCensor<'a, P, W, A>;
		}
	}

	#[document_type_parameters(
		"The lifetime of the stored closures.",
		"The pointer brand storing the closures.",
		"The log type transformed by the censor function.",
		"The selected action program type."
	)]
	#[document_parameters("The scoped Writer censor cell to clone.")]
	impl<'a, P, W, A> Clone for WriterCensor<'a, P, W, A>
	where
		P: ToDynCloneFn,
		W: 'a,
		A: 'a,
	{
		/// Clones the censor cell by refcount-bumping the stored
		/// transformation and action thunks.
		#[document_signature]
		#[document_returns("A new censor cell sharing the stored closures by refcount.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::RcBrand,
		/// 	classes::ToDynCloneFn,
		/// 	types::effects::writer::WriterCensor,
		/// };
		///
		/// let cell: WriterCensor<'static, RcBrand, String, i32> = WriterCensor::Censor {
		/// 	censor: <RcBrand as ToDynCloneFn>::new(|log: String| format!("{log}!")),
		/// 	action: <RcBrand as ToDynCloneFn>::new(|_: ()| 41),
		/// };
		/// let cloned = cell.clone();
		/// match cloned {
		/// 	WriterCensor::Censor {
		/// 		censor,
		/// 		action,
		/// 	} => {
		/// 		assert_eq!(censor("hello".to_owned()), "hello!");
		/// 		assert_eq!(action(()), 41);
		/// 	}
		/// }
		/// ```
		fn clone(&self) -> Self {
			match self {
				WriterCensor::Censor {
					censor,
					action,
				} => WriterCensor::Censor {
					censor: <P as RefCountedPointer>::Of::clone(censor),
					action: <P as RefCountedPointer>::Of::clone(action),
				},
			}
		}
	}

	/// Scoped Writer `censor` effect for `ArcRun` /
	/// `ArcRunExplicit` substrates.
	#[document_type_parameters(
		"The lifetime of the stored closures.",
		"The pointer brand storing the closures (ArcBrand only).",
		"The log type transformed by the censor function.",
		"The selected action program type."
	)]
	pub enum SendWriterCensor<'a, P, W, A>
	where
		P: ToDynSendFn,
		W: Send + Sync + 'a,
		A: 'a, {
		/// Run `action` under a neutral Writer log transformation.
		Censor {
			/// The log transformation. Handler selection determines
			/// whether it applies before or after accumulation.
			censor: <P as SendRefCountedPointer>::Of<'a, dyn 'a + Fn(W) -> W + Send + Sync>,
			/// The selected action program, stored as a thread-safe
			/// multi-shot thunk.
			action: <P as SendRefCountedPointer>::Of<'a, dyn 'a + Fn(()) -> A + Send + Sync>,
		},
	}

	impl_kind! {
		impl<P: ToDynSendFn, W: Send + Sync + 'static> for SendWriterCensorBrand<P, W> {
			type Of<'a, A: 'a>: 'a = SendWriterCensor<'a, P, W, A>;
		}
	}

	#[document_type_parameters(
		"The lifetime of the stored closures.",
		"The pointer brand storing the closures.",
		"The log type transformed by the censor function.",
		"The selected action program type."
	)]
	#[document_parameters("The scoped Writer censor cell to clone.")]
	impl<'a, P, W, A> Clone for SendWriterCensor<'a, P, W, A>
	where
		P: ToDynSendFn,
		W: Send + Sync + 'a,
		A: 'a,
	{
		/// Clones the censor cell by refcount-bumping the stored
		/// transformation and action thunks.
		#[document_signature]
		#[document_returns("A new censor cell sharing the stored closures by refcount.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::ArcBrand,
		/// 	classes::ToDynSendFn,
		/// 	types::effects::writer::SendWriterCensor,
		/// };
		///
		/// let cell: SendWriterCensor<'static, ArcBrand, String, i32> = SendWriterCensor::Censor {
		/// 	censor: <ArcBrand as ToDynSendFn>::new(|log: String| format!("{log}!")),
		/// 	action: <ArcBrand as ToDynSendFn>::new(|_: ()| 41),
		/// };
		/// let cloned = cell.clone();
		/// match cloned {
		/// 	SendWriterCensor::Censor {
		/// 		censor,
		/// 		action,
		/// 	} => {
		/// 		assert_eq!(censor("hello".to_owned()), "hello!");
		/// 		assert_eq!(action(()), 41);
		/// 	}
		/// }
		/// ```
		fn clone(&self) -> Self {
			match self {
				SendWriterCensor::Censor {
					censor,
					action,
				} => SendWriterCensor::Censor {
					censor: <P as SendRefCountedPointer>::Of::clone(censor),
					action: <P as SendRefCountedPointer>::Of::clone(action),
				},
			}
		}
	}

	// ===== Scoped Writer: BoxWriterListen / WriterListen / SendWriterListen =====

	/// Scoped Writer `listen` effect for default `Run` /
	/// `RunExplicit` substrates.
	///
	/// The selected action program is stored in the ordinary GAT result
	/// slot, while `Action` records the value type the handler must pair
	/// with the observed log `W` before resuming the wrapper-owned outer
	/// continuation.
	#[document_type_parameters(
		"The lifetime of the stored action closure.",
		"The pointer brand storing the action closure (BoxBrand only by structural bound).",
		"The log type observed by listen.",
		"The selected action value type.",
		"The selected action program type."
	)]
	pub enum BoxWriterListen<'a, P, W, Action, A>
	where
		P: ToDynFnOnce,
		W: 'a,
		Action: 'a,
		A: 'a, {
		/// Run the selected action and make its produced log visible to
		/// the standard Writer handler.
		Listen {
			/// The selected action program, stored as a single-shot thunk.
			action: <P as Pointer>::Of<'a, dyn 'a + FnOnce(()) -> A>,
			/// Carries the selected action value type and log type
			/// without storing values of either type.
			result: PhantomData<fn(Action) -> W>,
		},
	}

	impl_kind! {
		impl<P: ToDynFnOnce, W: 'static, Action: 'static> for BoxWriterListenBrand<P, W, Action> {
			type Of<'a, A: 'a>: 'a = BoxWriterListen<'a, P, W, Action, A>;
		}
	}

	/// Scoped Writer `listen` effect for `RcRun` /
	/// `RcRunExplicit` substrates.
	#[document_type_parameters(
		"The lifetime of the stored action closure.",
		"The pointer brand storing the action closure (RcBrand only).",
		"The log type observed by listen.",
		"The selected action value type.",
		"The selected action program type."
	)]
	pub enum WriterListen<'a, P, W, Action, A>
	where
		P: ToDynCloneFn,
		W: 'a,
		Action: 'a,
		A: 'a, {
		/// Run the selected action and make its produced log visible to
		/// the standard Writer handler.
		Listen {
			/// The selected action program, stored as a multi-shot thunk.
			action: <P as RefCountedPointer>::Of<'a, dyn 'a + Fn(()) -> A>,
			/// Carries the selected action value type and log type
			/// without storing values of either type.
			result: PhantomData<fn(Action) -> W>,
		},
	}

	impl_kind! {
		impl<P: ToDynCloneFn, W: 'static, Action: 'static> for WriterListenBrand<P, W, Action> {
			type Of<'a, A: 'a>: 'a = WriterListen<'a, P, W, Action, A>;
		}
	}

	#[document_type_parameters(
		"The lifetime of the stored action closure.",
		"The pointer brand storing the action closure.",
		"The log type observed by listen.",
		"The selected action value type.",
		"The selected action program type."
	)]
	#[document_parameters("The scoped Writer listen cell to clone.")]
	impl<'a, P, W, Action, A> Clone for WriterListen<'a, P, W, Action, A>
	where
		P: ToDynCloneFn,
		W: 'a,
		Action: 'a,
		A: 'a,
	{
		/// Clones the listen cell by refcount-bumping the stored action thunk.
		#[document_signature]
		#[document_returns("A new listen cell sharing the stored action by refcount.")]
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	core::marker::PhantomData,
		/// 	fp_library::{
		/// 		brands::RcBrand,
		/// 		classes::ToDynCloneFn,
		/// 		types::effects::writer::WriterListen,
		/// 	},
		/// };
		///
		/// let cell: WriterListen<'static, RcBrand, String, i32, i32> = WriterListen::Listen {
		/// 	action: <RcBrand as ToDynCloneFn>::new(|_: ()| 41),
		/// 	result: PhantomData,
		/// };
		/// let cloned = cell.clone();
		/// match cloned {
		/// 	WriterListen::Listen {
		/// 		action,
		/// 		result: _,
		/// 	} => assert_eq!(action(()), 41),
		/// }
		/// ```
		fn clone(&self) -> Self {
			match self {
				WriterListen::Listen {
					action,
					result: _,
				} => WriterListen::Listen {
					action: <P as RefCountedPointer>::Of::clone(action),
					result: PhantomData,
				},
			}
		}
	}

	/// Scoped Writer `listen` effect for `ArcRun` /
	/// `ArcRunExplicit` substrates.
	#[document_type_parameters(
		"The lifetime of the stored action closure.",
		"The pointer brand storing the action closure (ArcBrand only).",
		"The log type observed by listen.",
		"The selected action value type.",
		"The selected action program type."
	)]
	pub enum SendWriterListen<'a, P, W, Action, A>
	where
		P: ToDynSendFn,
		W: Send + Sync + 'a,
		Action: Send + Sync + 'a,
		A: 'a, {
		/// Run the selected action and make its produced log visible to
		/// the standard Writer handler.
		Listen {
			/// The selected action program, stored as a thread-safe
			/// multi-shot thunk.
			action: <P as SendRefCountedPointer>::Of<'a, dyn 'a + Fn(()) -> A + Send + Sync>,
			/// Carries the selected action value type and log type
			/// without storing values of either type.
			result: PhantomData<fn(Action) -> W>,
		},
	}

	impl_kind! {
		impl<P: ToDynSendFn, W: Send + Sync + 'static, Action: Send + Sync + 'static>
			for SendWriterListenBrand<P, W, Action>
		{
			type Of<'a, A: 'a>: 'a = SendWriterListen<'a, P, W, Action, A>;
		}
	}

	#[document_type_parameters(
		"The lifetime of the stored action closure.",
		"The pointer brand storing the action closure.",
		"The log type observed by listen.",
		"The selected action value type.",
		"The selected action program type."
	)]
	#[document_parameters("The scoped Writer listen cell to clone.")]
	impl<'a, P, W, Action, A> Clone for SendWriterListen<'a, P, W, Action, A>
	where
		P: ToDynSendFn,
		W: Send + Sync + 'a,
		Action: Send + Sync + 'a,
		A: 'a,
	{
		/// Clones the listen cell by refcount-bumping the stored action thunk.
		#[document_signature]
		#[document_returns("A new listen cell sharing the stored action by refcount.")]
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	core::marker::PhantomData,
		/// 	fp_library::{
		/// 		brands::ArcBrand,
		/// 		classes::ToDynSendFn,
		/// 		types::effects::writer::SendWriterListen,
		/// 	},
		/// };
		///
		/// let cell: SendWriterListen<'static, ArcBrand, String, i32, i32> = SendWriterListen::Listen {
		/// 	action: <ArcBrand as ToDynSendFn>::new(|_: ()| 41),
		/// 	result: PhantomData,
		/// };
		/// let cloned = cell.clone();
		/// match cloned {
		/// 	SendWriterListen::Listen {
		/// 		action,
		/// 		result: _,
		/// 	} => assert_eq!(action(()), 41),
		/// }
		/// ```
		fn clone(&self) -> Self {
			match self {
				SendWriterListen::Listen {
					action,
					result: _,
				} => SendWriterListen::Listen {
					action: <P as SendRefCountedPointer>::Of::clone(action),
					result: PhantomData,
				},
			}
		}
	}

	// ===== Scoped Writer Functor impls =====

	#[document_type_parameters("The log type transformed by censor.")]
	impl<W> Functor for BoxWriterCensorBrand<BoxBrand, W>
	where
		W: 'static,
	{
		/// Maps over the selected action program while preserving the
		/// neutral censor transformation.
		#[document_signature]
		#[document_type_parameters(
			"The lifetime.",
			"The original action program type.",
			"The mapped action program type."
		)]
		#[document_parameters("The function to apply to the action program.", "The censor cell.")]
		#[document_returns("A censor cell with the action thunk mapped by `f`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		BoxBrand,
		/// 		BoxWriterCensorBrand,
		/// 	},
		/// 	classes::{
		/// 		Functor,
		/// 		ToDynFn,
		/// 		ToDynFnOnce,
		/// 	},
		/// 	types::effects::writer::BoxWriterCensor,
		/// };
		///
		/// let cell: BoxWriterCensor<'static, BoxBrand, String, i32> = BoxWriterCensor::Censor {
		/// 	censor: <BoxBrand as ToDynFn>::new(|log: String| format!("{log}!")),
		/// 	action: <BoxBrand as ToDynFnOnce>::new(|_: ()| 41),
		/// };
		/// let mapped = <BoxWriterCensorBrand<BoxBrand, String> as Functor>::map(|x| x + 1, cell);
		/// match mapped {
		/// 	BoxWriterCensor::Censor {
		/// 		censor,
		/// 		action,
		/// 	} => {
		/// 		assert_eq!(censor("hello".to_owned()), "hello!");
		/// 		assert_eq!(action(()), 42);
		/// 	}
		/// }
		/// ```
		fn map<'a, A: 'a, B: 'a>(
			f: impl Fn(A) -> B + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			match fa {
				BoxWriterCensor::Censor {
					censor,
					action,
				} => BoxWriterCensor::Censor {
					censor,
					action: <BoxBrand as ToDynFnOnce>::new(move |()| f(action(()))),
				},
			}
		}
	}

	#[document_type_parameters("The log type transformed by censor.")]
	impl<W> Functor for WriterCensorBrand<RcBrand, W>
	where
		W: 'static,
	{
		/// Maps over the selected action program while preserving the
		/// neutral censor transformation.
		#[document_signature]
		#[document_type_parameters(
			"The lifetime.",
			"The original action program type.",
			"The mapped action program type."
		)]
		#[document_parameters("The function to apply to the action program.", "The censor cell.")]
		#[document_returns("A censor cell with the action thunk mapped by `f`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		RcBrand,
		/// 		WriterCensorBrand,
		/// 	},
		/// 	classes::{
		/// 		Functor,
		/// 		ToDynCloneFn,
		/// 	},
		/// 	types::effects::writer::WriterCensor,
		/// };
		///
		/// let cell: WriterCensor<'static, RcBrand, String, i32> = WriterCensor::Censor {
		/// 	censor: <RcBrand as ToDynCloneFn>::new(|log: String| format!("{log}!")),
		/// 	action: <RcBrand as ToDynCloneFn>::new(|_: ()| 41),
		/// };
		/// let mapped = <WriterCensorBrand<RcBrand, String> as Functor>::map(|x| x + 1, cell);
		/// match mapped {
		/// 	WriterCensor::Censor {
		/// 		censor,
		/// 		action,
		/// 	} => {
		/// 		assert_eq!(censor("hello".to_owned()), "hello!");
		/// 		assert_eq!(action(()), 42);
		/// 	}
		/// }
		/// ```
		fn map<'a, A: 'a, B: 'a>(
			f: impl Fn(A) -> B + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			match fa {
				WriterCensor::Censor {
					censor,
					action,
				} => WriterCensor::Censor {
					censor,
					action: <RcBrand as ToDynCloneFn>::new(move |()| f(action(()))),
				},
			}
		}
	}

	#[document_type_parameters(
		"The log type observed by listen.",
		"The selected action value type."
	)]
	impl<W, Action> Functor for BoxWriterListenBrand<BoxBrand, W, Action>
	where
		W: 'static,
		Action: 'static,
	{
		/// Maps over the selected action program while preserving the
		/// `Action` value type recorded by the brand.
		#[document_signature]
		#[document_type_parameters(
			"The lifetime.",
			"The original action program type.",
			"The mapped action program type."
		)]
		#[document_parameters("The function to apply to the action program.", "The listen cell.")]
		#[document_returns("A listen cell with the action thunk mapped by `f`.")]
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	core::marker::PhantomData,
		/// 	fp_library::{
		/// 		brands::{
		/// 			BoxBrand,
		/// 			BoxWriterListenBrand,
		/// 		},
		/// 		classes::{
		/// 			Functor,
		/// 			ToDynFnOnce,
		/// 		},
		/// 		types::effects::writer::BoxWriterListen,
		/// 	},
		/// };
		///
		/// let cell: BoxWriterListen<'static, BoxBrand, String, i32, i32> = BoxWriterListen::Listen {
		/// 	action: <BoxBrand as ToDynFnOnce>::new(|_: ()| 41),
		/// 	result: PhantomData,
		/// };
		/// let mapped = <BoxWriterListenBrand<BoxBrand, String, i32> as Functor>::map(|x| x + 1, cell);
		/// match mapped {
		/// 	BoxWriterListen::Listen {
		/// 		action,
		/// 		result: _,
		/// 	} => assert_eq!(action(()), 42),
		/// }
		/// ```
		fn map<'a, A: 'a, B: 'a>(
			f: impl Fn(A) -> B + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			match fa {
				BoxWriterListen::Listen {
					action,
					result: _,
				} => BoxWriterListen::Listen {
					action: <BoxBrand as ToDynFnOnce>::new(move |()| f(action(()))),
					result: PhantomData,
				},
			}
		}
	}

	#[document_type_parameters(
		"The log type observed by listen.",
		"The selected action value type."
	)]
	impl<W, Action> Functor for WriterListenBrand<RcBrand, W, Action>
	where
		W: 'static,
		Action: 'static,
	{
		/// Maps over the selected action program while preserving the
		/// `Action` value type recorded by the brand.
		#[document_signature]
		#[document_type_parameters(
			"The lifetime.",
			"The original action program type.",
			"The mapped action program type."
		)]
		#[document_parameters("The function to apply to the action program.", "The listen cell.")]
		#[document_returns("A listen cell with the action thunk mapped by `f`.")]
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	core::marker::PhantomData,
		/// 	fp_library::{
		/// 		brands::{
		/// 			RcBrand,
		/// 			WriterListenBrand,
		/// 		},
		/// 		classes::{
		/// 			Functor,
		/// 			ToDynCloneFn,
		/// 		},
		/// 		types::effects::writer::WriterListen,
		/// 	},
		/// };
		///
		/// let cell: WriterListen<'static, RcBrand, String, i32, i32> = WriterListen::Listen {
		/// 	action: <RcBrand as ToDynCloneFn>::new(|_: ()| 41),
		/// 	result: PhantomData,
		/// };
		/// let mapped = <WriterListenBrand<RcBrand, String, i32> as Functor>::map(|x| x + 1, cell);
		/// match mapped {
		/// 	WriterListen::Listen {
		/// 		action,
		/// 		result: _,
		/// 	} => assert_eq!(action(()), 42),
		/// }
		/// ```
		fn map<'a, A: 'a, B: 'a>(
			f: impl Fn(A) -> B + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			match fa {
				WriterListen::Listen {
					action,
					result: _,
				} => WriterListen::Listen {
					action: <RcBrand as ToDynCloneFn>::new(move |()| f(action(()))),
					result: PhantomData,
				},
			}
		}
	}

	// ===== Scoped Writer SendFunctor impls =====

	#[document_type_parameters("The log type transformed by censor.")]
	impl<W> SendFunctor for SendWriterCensorBrand<ArcBrand, W>
	where
		W: Send + Sync + 'static,
	{
		/// Thread-safe map over the selected action program while
		/// preserving the neutral censor transformation.
		#[document_signature]
		#[document_type_parameters(
			"The lifetime.",
			"The original action program type.",
			"The mapped action program type."
		)]
		#[document_parameters("The function to apply to the action program.", "The censor cell.")]
		#[document_returns("A thread-safe censor cell with the action thunk mapped by `f`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		ArcBrand,
		/// 		SendWriterCensorBrand,
		/// 	},
		/// 	classes::{
		/// 		SendFunctor,
		/// 		ToDynSendFn,
		/// 	},
		/// 	types::effects::writer::SendWriterCensor,
		/// };
		///
		/// let cell: SendWriterCensor<'static, ArcBrand, String, i32> = SendWriterCensor::Censor {
		/// 	censor: <ArcBrand as ToDynSendFn>::new(|log: String| format!("{log}!")),
		/// 	action: <ArcBrand as ToDynSendFn>::new(|_: ()| 41),
		/// };
		/// let mapped =
		/// 	<SendWriterCensorBrand<ArcBrand, String> as SendFunctor>::send_map(|x| x + 1, cell);
		/// match mapped {
		/// 	SendWriterCensor::Censor {
		/// 		censor,
		/// 		action,
		/// 	} => {
		/// 		assert_eq!(censor("hello".to_owned()), "hello!");
		/// 		assert_eq!(action(()), 42);
		/// 	}
		/// }
		/// ```
		fn send_map<'a, A: Send + Sync + 'a, B: Send + Sync + 'a>(
			f: impl Fn(A) -> B + Send + Sync + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			match fa {
				SendWriterCensor::Censor {
					censor,
					action,
				} => SendWriterCensor::Censor {
					censor,
					action: <ArcBrand as ToDynSendFn>::new(move |()| f(action(()))),
				},
			}
		}
	}

	#[document_type_parameters(
		"The log type observed by listen.",
		"The selected action value type."
	)]
	impl<W, Action> SendFunctor for SendWriterListenBrand<ArcBrand, W, Action>
	where
		W: Send + Sync + 'static,
		Action: Send + Sync + 'static,
	{
		/// Thread-safe map over the selected action program while
		/// preserving the selected action value type recorded by the
		/// brand.
		#[document_signature]
		#[document_type_parameters(
			"The lifetime.",
			"The original action program type.",
			"The mapped action program type."
		)]
		#[document_parameters("The function to apply to the action program.", "The listen cell.")]
		#[document_returns("A thread-safe listen cell with the action thunk mapped by `f`.")]
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	core::marker::PhantomData,
		/// 	fp_library::{
		/// 		brands::{
		/// 			ArcBrand,
		/// 			SendWriterListenBrand,
		/// 		},
		/// 		classes::{
		/// 			SendFunctor,
		/// 			ToDynSendFn,
		/// 		},
		/// 		types::effects::writer::SendWriterListen,
		/// 	},
		/// };
		///
		/// let cell: SendWriterListen<'static, ArcBrand, String, i32, i32> = SendWriterListen::Listen {
		/// 	action: <ArcBrand as ToDynSendFn>::new(|_: ()| 41),
		/// 	result: PhantomData,
		/// };
		/// let mapped =
		/// 	<SendWriterListenBrand<ArcBrand, String, i32> as SendFunctor>::send_map(|x| x + 1, cell);
		/// match mapped {
		/// 	SendWriterListen::Listen {
		/// 		action,
		/// 		result: _,
		/// 	} => assert_eq!(action(()), 42),
		/// }
		/// ```
		fn send_map<'a, A: Send + Sync + 'a, B: Send + Sync + 'a>(
			f: impl Fn(A) -> B + Send + Sync + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			match fa {
				SendWriterListen::Listen {
					action,
					result: _,
				} => SendWriterListen::Listen {
					action: <ArcBrand as ToDynSendFn>::new(move |()| f(action(()))),
					result: PhantomData,
				},
			}
		}
	}

	#[document_type_parameters("The log type transformed by censor.")]
	impl<W> SendFunctor for BoxWriterCensorBrand<BoxBrand, W>
	where
		W: Send + Sync + 'static,
	{
		/// SendFunctor stub for the BoxBrand-flavoured Writer censor cell.
		#[document_signature]
		#[document_type_parameters(
			"The lifetime.",
			"The original action program type.",
			"The mapped action program type."
		)]
		#[document_parameters("The function to apply to the action program.", "The censor cell.")]
		#[document_returns("A censor cell with the action thunk mapped by `f`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		BoxBrand,
		/// 		BoxWriterCensorBrand,
		/// 	},
		/// 	classes::{
		/// 		SendFunctor,
		/// 		ToDynFn,
		/// 		ToDynFnOnce,
		/// 	},
		/// 	types::effects::writer::BoxWriterCensor,
		/// };
		///
		/// let cell: BoxWriterCensor<'static, BoxBrand, String, i32> = BoxWriterCensor::Censor {
		/// 	censor: <BoxBrand as ToDynFn>::new(|log: String| format!("{log}!")),
		/// 	action: <BoxBrand as ToDynFnOnce>::new(|_: ()| 41),
		/// };
		/// let mapped = <BoxWriterCensorBrand<BoxBrand, String> as SendFunctor>::send_map(|x| x + 1, cell);
		/// match mapped {
		/// 	BoxWriterCensor::Censor {
		/// 		action, ..
		/// 	} => assert_eq!(action(()), 42),
		/// }
		/// ```
		fn send_map<'a, A: Send + Sync + 'a, B: Send + Sync + 'a>(
			f: impl Fn(A) -> B + Send + Sync + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			<Self as Functor>::map(f, fa)
		}
	}

	#[document_type_parameters("The log type transformed by censor.")]
	impl<W> SendFunctor for WriterCensorBrand<RcBrand, W>
	where
		W: Send + Sync + 'static,
	{
		/// SendFunctor stub for the RcBrand-flavoured Writer censor cell.
		#[document_signature]
		#[document_type_parameters(
			"The lifetime.",
			"The original action program type.",
			"The mapped action program type."
		)]
		#[document_parameters("The function to apply to the action program.", "The censor cell.")]
		#[document_returns("A censor cell with the action thunk mapped by `f`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		RcBrand,
		/// 		WriterCensorBrand,
		/// 	},
		/// 	classes::{
		/// 		SendFunctor,
		/// 		ToDynCloneFn,
		/// 	},
		/// 	types::effects::writer::WriterCensor,
		/// };
		///
		/// let cell: WriterCensor<'static, RcBrand, String, i32> = WriterCensor::Censor {
		/// 	censor: <RcBrand as ToDynCloneFn>::new(|log: String| format!("{log}!")),
		/// 	action: <RcBrand as ToDynCloneFn>::new(|_: ()| 41),
		/// };
		/// let mapped = <WriterCensorBrand<RcBrand, String> as SendFunctor>::send_map(|x| x + 1, cell);
		/// match mapped {
		/// 	WriterCensor::Censor {
		/// 		action, ..
		/// 	} => assert_eq!(action(()), 42),
		/// }
		/// ```
		fn send_map<'a, A: Send + Sync + 'a, B: Send + Sync + 'a>(
			f: impl Fn(A) -> B + Send + Sync + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			<Self as Functor>::map(f, fa)
		}
	}

	#[document_type_parameters(
		"The log type observed by listen.",
		"The selected action value type."
	)]
	impl<W, Action> SendFunctor for BoxWriterListenBrand<BoxBrand, W, Action>
	where
		W: Send + Sync + 'static,
		Action: Send + Sync + 'static,
	{
		/// SendFunctor stub for the BoxBrand-flavoured Writer listen cell.
		#[document_signature]
		#[document_type_parameters(
			"The lifetime.",
			"The original action program type.",
			"The mapped action program type."
		)]
		#[document_parameters("The function to apply to the action program.", "The listen cell.")]
		#[document_returns("A listen cell with the action thunk mapped by `f`.")]
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	core::marker::PhantomData,
		/// 	fp_library::{
		/// 		brands::{
		/// 			BoxBrand,
		/// 			BoxWriterListenBrand,
		/// 		},
		/// 		classes::{
		/// 			SendFunctor,
		/// 			ToDynFnOnce,
		/// 		},
		/// 		types::effects::writer::BoxWriterListen,
		/// 	},
		/// };
		///
		/// let cell: BoxWriterListen<'static, BoxBrand, String, i32, i32> = BoxWriterListen::Listen {
		/// 	action: <BoxBrand as ToDynFnOnce>::new(|_: ()| 41),
		/// 	result: PhantomData,
		/// };
		/// let mapped =
		/// 	<BoxWriterListenBrand<BoxBrand, String, i32> as SendFunctor>::send_map(|x| x + 1, cell);
		/// match mapped {
		/// 	BoxWriterListen::Listen {
		/// 		action,
		/// 		result: _,
		/// 	} => assert_eq!(action(()), 42),
		/// }
		/// ```
		fn send_map<'a, A: Send + Sync + 'a, B: Send + Sync + 'a>(
			f: impl Fn(A) -> B + Send + Sync + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			<Self as Functor>::map(f, fa)
		}
	}

	#[document_type_parameters(
		"The log type observed by listen.",
		"The selected action value type."
	)]
	impl<W, Action> SendFunctor for WriterListenBrand<RcBrand, W, Action>
	where
		W: Send + Sync + 'static,
		Action: Send + Sync + 'static,
	{
		/// SendFunctor stub for the RcBrand-flavoured Writer listen cell.
		#[document_signature]
		#[document_type_parameters(
			"The lifetime.",
			"The original action program type.",
			"The mapped action program type."
		)]
		#[document_parameters("The function to apply to the action program.", "The listen cell.")]
		#[document_returns("A listen cell with the action thunk mapped by `f`.")]
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	core::marker::PhantomData,
		/// 	fp_library::{
		/// 		brands::{
		/// 			RcBrand,
		/// 			WriterListenBrand,
		/// 		},
		/// 		classes::{
		/// 			SendFunctor,
		/// 			ToDynCloneFn,
		/// 		},
		/// 		types::effects::writer::WriterListen,
		/// 	},
		/// };
		///
		/// let cell: WriterListen<'static, RcBrand, String, i32, i32> = WriterListen::Listen {
		/// 	action: <RcBrand as ToDynCloneFn>::new(|_: ()| 41),
		/// 	result: PhantomData,
		/// };
		/// let mapped =
		/// 	<WriterListenBrand<RcBrand, String, i32> as SendFunctor>::send_map(|x| x + 1, cell);
		/// match mapped {
		/// 	WriterListen::Listen {
		/// 		action,
		/// 		result: _,
		/// 	} => assert_eq!(action(()), 42),
		/// }
		/// ```
		fn send_map<'a, A: Send + Sync + 'a, B: Send + Sync + 'a>(
			f: impl Fn(A) -> B + Send + Sync + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			<Self as Functor>::map(f, fa)
		}
	}

	// ===== Scoped Writer WrapDrop impls =====

	#[document_type_parameters("The log type transformed by censor.")]
	impl<W> WrapDrop for BoxWriterCensorBrand<BoxBrand, W>
	where
		W: 'static,
	{
		/// Decomposes a censor cell by materialising and returning the selected action.
		#[document_signature]
		#[document_type_parameters("The lifetime.", "The selected action program type.")]
		#[document_parameters("The censor cell to decompose.")]
		#[document_returns(
			"`Some` of the selected action program; the censor function is dropped."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		BoxBrand,
		/// 		BoxWriterCensorBrand,
		/// 	},
		/// 	classes::{
		/// 		ToDynFn,
		/// 		ToDynFnOnce,
		/// 		WrapDrop,
		/// 	},
		/// 	types::effects::writer::BoxWriterCensor,
		/// };
		///
		/// let cell: BoxWriterCensor<'static, BoxBrand, String, i32> = BoxWriterCensor::Censor {
		/// 	censor: <BoxBrand as ToDynFn>::new(|log: String| log),
		/// 	action: <BoxBrand as ToDynFnOnce>::new(|_: ()| 42),
		/// };
		/// assert_eq!(<BoxWriterCensorBrand<BoxBrand, String> as WrapDrop>::drop(cell), Some(42));
		/// ```
		fn drop<'a, X: 'a>(
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, X>)
		) -> Option<X> {
			match fa {
				BoxWriterCensor::Censor {
					censor: _,
					action,
				} => Some(action(())),
			}
		}
	}

	#[document_type_parameters("The log type transformed by censor.")]
	impl<W> WrapDrop for WriterCensorBrand<RcBrand, W>
	where
		W: 'static,
	{
		/// Decomposes a censor cell by materialising and returning the selected action.
		#[document_signature]
		#[document_type_parameters("The lifetime.", "The selected action program type.")]
		#[document_parameters("The censor cell to decompose.")]
		#[document_returns(
			"`Some` of the selected action program; the censor function is dropped."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		RcBrand,
		/// 		WriterCensorBrand,
		/// 	},
		/// 	classes::{
		/// 		ToDynCloneFn,
		/// 		WrapDrop,
		/// 	},
		/// 	types::effects::writer::WriterCensor,
		/// };
		///
		/// let cell: WriterCensor<'static, RcBrand, String, i32> = WriterCensor::Censor {
		/// 	censor: <RcBrand as ToDynCloneFn>::new(|log: String| log),
		/// 	action: <RcBrand as ToDynCloneFn>::new(|_: ()| 42),
		/// };
		/// assert_eq!(<WriterCensorBrand<RcBrand, String> as WrapDrop>::drop(cell), Some(42));
		/// ```
		fn drop<'a, X: 'a>(
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, X>)
		) -> Option<X> {
			match fa {
				WriterCensor::Censor {
					censor: _,
					action,
				} => Some(action(())),
			}
		}
	}

	#[document_type_parameters("The log type transformed by censor.")]
	impl<W> WrapDrop for SendWriterCensorBrand<ArcBrand, W>
	where
		W: Send + Sync + 'static,
	{
		/// Decomposes a censor cell by materialising and returning the selected action.
		#[document_signature]
		#[document_type_parameters("The lifetime.", "The selected action program type.")]
		#[document_parameters("The censor cell to decompose.")]
		#[document_returns(
			"`Some` of the selected action program; the censor function is dropped."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		ArcBrand,
		/// 		SendWriterCensorBrand,
		/// 	},
		/// 	classes::{
		/// 		ToDynSendFn,
		/// 		WrapDrop,
		/// 	},
		/// 	types::effects::writer::SendWriterCensor,
		/// };
		///
		/// let cell: SendWriterCensor<'static, ArcBrand, String, i32> = SendWriterCensor::Censor {
		/// 	censor: <ArcBrand as ToDynSendFn>::new(|log: String| log),
		/// 	action: <ArcBrand as ToDynSendFn>::new(|_: ()| 42),
		/// };
		/// assert_eq!(<SendWriterCensorBrand<ArcBrand, String> as WrapDrop>::drop(cell), Some(42));
		/// ```
		fn drop<'a, X: 'a>(
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, X>)
		) -> Option<X> {
			match fa {
				SendWriterCensor::Censor {
					censor: _,
					action,
				} => Some(action(())),
			}
		}
	}

	#[document_type_parameters(
		"The log type observed by listen.",
		"The selected action value type."
	)]
	impl<W, Action> WrapDrop for BoxWriterListenBrand<BoxBrand, W, Action>
	where
		W: 'static,
		Action: 'static,
	{
		/// Decomposes a listen cell by materialising and returning the selected action.
		#[document_signature]
		#[document_type_parameters("The lifetime.", "The selected action program type.")]
		#[document_parameters("The listen cell to decompose.")]
		#[document_returns("`Some` of the selected action program.")]
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	core::marker::PhantomData,
		/// 	fp_library::{
		/// 		brands::{
		/// 			BoxBrand,
		/// 			BoxWriterListenBrand,
		/// 		},
		/// 		classes::{
		/// 			ToDynFnOnce,
		/// 			WrapDrop,
		/// 		},
		/// 		types::effects::writer::BoxWriterListen,
		/// 	},
		/// };
		///
		/// let cell: BoxWriterListen<'static, BoxBrand, String, i32, i32> = BoxWriterListen::Listen {
		/// 	action: <BoxBrand as ToDynFnOnce>::new(|_: ()| 42),
		/// 	result: PhantomData,
		/// };
		/// assert_eq!(<BoxWriterListenBrand<BoxBrand, String, i32> as WrapDrop>::drop(cell), Some(42));
		/// ```
		fn drop<'a, X: 'a>(
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, X>)
		) -> Option<X> {
			match fa {
				BoxWriterListen::Listen {
					action,
					result: _,
				} => Some(action(())),
			}
		}
	}

	#[document_type_parameters(
		"The log type observed by listen.",
		"The selected action value type."
	)]
	impl<W, Action> WrapDrop for WriterListenBrand<RcBrand, W, Action>
	where
		W: 'static,
		Action: 'static,
	{
		/// Decomposes a listen cell by materialising and returning the selected action.
		#[document_signature]
		#[document_type_parameters("The lifetime.", "The selected action program type.")]
		#[document_parameters("The listen cell to decompose.")]
		#[document_returns("`Some` of the selected action program.")]
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	core::marker::PhantomData,
		/// 	fp_library::{
		/// 		brands::{
		/// 			RcBrand,
		/// 			WriterListenBrand,
		/// 		},
		/// 		classes::{
		/// 			ToDynCloneFn,
		/// 			WrapDrop,
		/// 		},
		/// 		types::effects::writer::WriterListen,
		/// 	},
		/// };
		///
		/// let cell: WriterListen<'static, RcBrand, String, i32, i32> = WriterListen::Listen {
		/// 	action: <RcBrand as ToDynCloneFn>::new(|_: ()| 42),
		/// 	result: PhantomData,
		/// };
		/// assert_eq!(<WriterListenBrand<RcBrand, String, i32> as WrapDrop>::drop(cell), Some(42));
		/// ```
		fn drop<'a, X: 'a>(
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, X>)
		) -> Option<X> {
			match fa {
				WriterListen::Listen {
					action,
					result: _,
				} => Some(action(())),
			}
		}
	}

	#[document_type_parameters(
		"The log type observed by listen.",
		"The selected action value type."
	)]
	impl<W, Action> WrapDrop for SendWriterListenBrand<ArcBrand, W, Action>
	where
		W: Send + Sync + 'static,
		Action: Send + Sync + 'static,
	{
		/// Decomposes a listen cell by materialising and returning the selected action.
		#[document_signature]
		#[document_type_parameters("The lifetime.", "The selected action program type.")]
		#[document_parameters("The listen cell to decompose.")]
		#[document_returns("`Some` of the selected action program.")]
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	core::marker::PhantomData,
		/// 	fp_library::{
		/// 		brands::{
		/// 			ArcBrand,
		/// 			SendWriterListenBrand,
		/// 		},
		/// 		classes::{
		/// 			ToDynSendFn,
		/// 			WrapDrop,
		/// 		},
		/// 		types::effects::writer::SendWriterListen,
		/// 	},
		/// };
		///
		/// let cell: SendWriterListen<'static, ArcBrand, String, i32, i32> = SendWriterListen::Listen {
		/// 	action: <ArcBrand as ToDynSendFn>::new(|_: ()| 42),
		/// 	result: PhantomData,
		/// };
		/// assert_eq!(<SendWriterListenBrand<ArcBrand, String, i32> as WrapDrop>::drop(cell), Some(42));
		/// ```
		fn drop<'a, X: 'a>(
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, X>)
		) -> Option<X> {
			match fa {
				SendWriterListen::Listen {
					action,
					result: _,
				} => Some(action(())),
			}
		}
	}

	// ===== Scoped Writer Extract impls =====

	#[document_type_parameters("The log type transformed by censor.")]
	impl<W> Extract for BoxWriterCensorBrand<BoxBrand, W>
	where
		W: 'static,
	{
		/// Extracts the selected action by invoking its thunk.
		#[document_signature]
		#[document_type_parameters("The lifetime.", "The selected action program type.")]
		#[document_parameters("The censor cell.")]
		#[document_returns("The materialised selected action program.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		BoxBrand,
		/// 		BoxWriterCensorBrand,
		/// 	},
		/// 	classes::{
		/// 		Extract,
		/// 		ToDynFn,
		/// 		ToDynFnOnce,
		/// 	},
		/// 	types::effects::writer::BoxWriterCensor,
		/// };
		///
		/// let cell: BoxWriterCensor<'static, BoxBrand, String, i32> = BoxWriterCensor::Censor {
		/// 	censor: <BoxBrand as ToDynFn>::new(|log: String| log),
		/// 	action: <BoxBrand as ToDynFnOnce>::new(|_: ()| 42),
		/// };
		/// assert_eq!(<BoxWriterCensorBrand<BoxBrand, String> as Extract>::extract(cell), 42);
		/// ```
		fn extract<'a, A: 'a>(
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)
		) -> A {
			match fa {
				BoxWriterCensor::Censor {
					censor: _,
					action,
				} => action(()),
			}
		}
	}

	#[document_type_parameters("The log type transformed by censor.")]
	impl<W> Extract for WriterCensorBrand<RcBrand, W>
	where
		W: 'static,
	{
		/// Extracts the selected action by invoking its thunk.
		#[document_signature]
		#[document_type_parameters("The lifetime.", "The selected action program type.")]
		#[document_parameters("The censor cell.")]
		#[document_returns("The materialised selected action program.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		RcBrand,
		/// 		WriterCensorBrand,
		/// 	},
		/// 	classes::{
		/// 		Extract,
		/// 		ToDynCloneFn,
		/// 	},
		/// 	types::effects::writer::WriterCensor,
		/// };
		///
		/// let cell: WriterCensor<'static, RcBrand, String, i32> = WriterCensor::Censor {
		/// 	censor: <RcBrand as ToDynCloneFn>::new(|log: String| log),
		/// 	action: <RcBrand as ToDynCloneFn>::new(|_: ()| 42),
		/// };
		/// assert_eq!(<WriterCensorBrand<RcBrand, String> as Extract>::extract(cell), 42);
		/// ```
		fn extract<'a, A: 'a>(
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)
		) -> A {
			match fa {
				WriterCensor::Censor {
					censor: _,
					action,
				} => action(()),
			}
		}
	}

	#[document_type_parameters("The log type transformed by censor.")]
	impl<W> Extract for SendWriterCensorBrand<ArcBrand, W>
	where
		W: Send + Sync + 'static,
	{
		/// Extracts the selected action by invoking its thunk.
		#[document_signature]
		#[document_type_parameters("The lifetime.", "The selected action program type.")]
		#[document_parameters("The censor cell.")]
		#[document_returns("The materialised selected action program.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		ArcBrand,
		/// 		SendWriterCensorBrand,
		/// 	},
		/// 	classes::{
		/// 		Extract,
		/// 		ToDynSendFn,
		/// 	},
		/// 	types::effects::writer::SendWriterCensor,
		/// };
		///
		/// let cell: SendWriterCensor<'static, ArcBrand, String, i32> = SendWriterCensor::Censor {
		/// 	censor: <ArcBrand as ToDynSendFn>::new(|log: String| log),
		/// 	action: <ArcBrand as ToDynSendFn>::new(|_: ()| 42),
		/// };
		/// assert_eq!(<SendWriterCensorBrand<ArcBrand, String> as Extract>::extract(cell), 42);
		/// ```
		fn extract<'a, A: 'a>(
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)
		) -> A {
			match fa {
				SendWriterCensor::Censor {
					censor: _,
					action,
				} => action(()),
			}
		}
	}

	#[document_type_parameters(
		"The log type observed by listen.",
		"The selected action value type."
	)]
	impl<W, Action> Extract for BoxWriterListenBrand<BoxBrand, W, Action>
	where
		W: 'static,
		Action: 'static,
	{
		/// Extracts the selected action by invoking its thunk.
		#[document_signature]
		#[document_type_parameters("The lifetime.", "The selected action program type.")]
		#[document_parameters("The listen cell.")]
		#[document_returns("The materialised selected action program.")]
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	core::marker::PhantomData,
		/// 	fp_library::{
		/// 		brands::{
		/// 			BoxBrand,
		/// 			BoxWriterListenBrand,
		/// 		},
		/// 		classes::{
		/// 			Extract,
		/// 			ToDynFnOnce,
		/// 		},
		/// 		types::effects::writer::BoxWriterListen,
		/// 	},
		/// };
		///
		/// let cell: BoxWriterListen<'static, BoxBrand, String, i32, i32> = BoxWriterListen::Listen {
		/// 	action: <BoxBrand as ToDynFnOnce>::new(|_: ()| 42),
		/// 	result: PhantomData,
		/// };
		/// assert_eq!(<BoxWriterListenBrand<BoxBrand, String, i32> as Extract>::extract(cell), 42);
		/// ```
		fn extract<'a, A: 'a>(
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)
		) -> A {
			match fa {
				BoxWriterListen::Listen {
					action,
					result: _,
				} => action(()),
			}
		}
	}

	#[document_type_parameters(
		"The log type observed by listen.",
		"The selected action value type."
	)]
	impl<W, Action> Extract for WriterListenBrand<RcBrand, W, Action>
	where
		W: 'static,
		Action: 'static,
	{
		/// Extracts the selected action by invoking its thunk.
		#[document_signature]
		#[document_type_parameters("The lifetime.", "The selected action program type.")]
		#[document_parameters("The listen cell.")]
		#[document_returns("The materialised selected action program.")]
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	core::marker::PhantomData,
		/// 	fp_library::{
		/// 		brands::{
		/// 			RcBrand,
		/// 			WriterListenBrand,
		/// 		},
		/// 		classes::{
		/// 			Extract,
		/// 			ToDynCloneFn,
		/// 		},
		/// 		types::effects::writer::WriterListen,
		/// 	},
		/// };
		///
		/// let cell: WriterListen<'static, RcBrand, String, i32, i32> = WriterListen::Listen {
		/// 	action: <RcBrand as ToDynCloneFn>::new(|_: ()| 42),
		/// 	result: PhantomData,
		/// };
		/// assert_eq!(<WriterListenBrand<RcBrand, String, i32> as Extract>::extract(cell), 42);
		/// ```
		fn extract<'a, A: 'a>(
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)
		) -> A {
			match fa {
				WriterListen::Listen {
					action,
					result: _,
				} => action(()),
			}
		}
	}

	#[document_type_parameters(
		"The log type observed by listen.",
		"The selected action value type."
	)]
	impl<W, Action> Extract for SendWriterListenBrand<ArcBrand, W, Action>
	where
		W: Send + Sync + 'static,
		Action: Send + Sync + 'static,
	{
		/// Extracts the selected action by invoking its thunk.
		#[document_signature]
		#[document_type_parameters("The lifetime.", "The selected action program type.")]
		#[document_parameters("The listen cell.")]
		#[document_returns("The materialised selected action program.")]
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	core::marker::PhantomData,
		/// 	fp_library::{
		/// 		brands::{
		/// 			ArcBrand,
		/// 			SendWriterListenBrand,
		/// 		},
		/// 		classes::{
		/// 			Extract,
		/// 			ToDynSendFn,
		/// 		},
		/// 		types::effects::writer::SendWriterListen,
		/// 	},
		/// };
		///
		/// let cell: SendWriterListen<'static, ArcBrand, String, i32, i32> = SendWriterListen::Listen {
		/// 	action: <ArcBrand as ToDynSendFn>::new(|_: ()| 42),
		/// 	result: PhantomData,
		/// };
		/// assert_eq!(<SendWriterListenBrand<ArcBrand, String, i32> as Extract>::extract(cell), 42);
		/// ```
		fn extract<'a, A: 'a>(
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)
		) -> A {
			match fa {
				SendWriterListen::Listen {
					action,
					result: _,
				} => action(()),
			}
		}
	}

	// ===== Scoped Writer RefFunctor impls =====

	#[document_type_parameters("The log type transformed by censor.")]
	impl<W> RefFunctor for BoxWriterCensorBrand<BoxBrand, W>
	where
		W: 'static,
	{
		/// Ref-map stub for Box-backed Writer censor cells.
		///
		/// A borrowed `Box<dyn FnOnce>` action cannot be replicated
		/// through a reference, so the returned cell contains
		/// panicking stubs. This mirrors the Box-backed Catch and Span
		/// RefFunctor precedent.
		#[document_signature]
		#[document_type_parameters(
			"The lifetime.",
			"The original action program type.",
			"The mapped action program type."
		)]
		#[document_parameters("The function to apply by reference.", "The censor cell.")]
		#[document_returns("A censor cell with panicking action and censor stubs.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		BoxBrand,
		/// 		BoxWriterCensorBrand,
		/// 	},
		/// 	classes::{
		/// 		RefFunctor,
		/// 		ToDynFn,
		/// 		ToDynFnOnce,
		/// 	},
		/// 	types::effects::writer::BoxWriterCensor,
		/// };
		///
		/// let cell: BoxWriterCensor<'static, BoxBrand, String, i32> = BoxWriterCensor::Censor {
		/// 	censor: <BoxBrand as ToDynFn>::new(|log: String| log),
		/// 	action: <BoxBrand as ToDynFnOnce>::new(|_: ()| 41),
		/// };
		/// let mapped =
		/// 	<BoxWriterCensorBrand<BoxBrand, String> as RefFunctor>::ref_map(|x: &i32| *x + 1, &cell);
		/// match mapped {
		/// 	BoxWriterCensor::Censor {
		/// 		censor,
		/// 		action,
		/// 	} => {
		/// 		assert!(
		/// 			std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| censor("x".to_owned())))
		/// 				.is_err()
		/// 		);
		/// 		assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| action(()))).is_err());
		/// 	}
		/// }
		/// ```
		#[expect(
			clippy::unreachable,
			reason = "BoxWriterCensorBrand::ref_map cannot replicate the FnOnce action thunk through a reference."
		)]
		fn ref_map<'a, A: 'a, B: 'a>(
			_func: impl Fn(&A) -> B + 'a,
			_fa: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			BoxWriterCensor::Censor {
				censor: <BoxBrand as ToDynFn>::new(|_log: W| -> W {
					unreachable!(
						"BoxWriterCensorBrand::ref_map's stub censor invoked; the borrowed Box-backed censor cell cannot be reconstructed without the action thunk"
					)
				}),
				action: <BoxBrand as ToDynFnOnce>::new(|_: ()| -> B {
					unreachable!(
						"BoxWriterCensorBrand::ref_map's stub action invoked; the FnOnce action thunk cannot be replicated through a reference"
					)
				}),
			}
		}
	}

	#[document_type_parameters("The log type transformed by censor.")]
	impl<W> RefFunctor for WriterCensorBrand<RcBrand, W>
	where
		W: 'static,
	{
		/// Ref-map over the selected action program while preserving
		/// the neutral censor transformation by refcount.
		#[document_signature]
		#[document_type_parameters(
			"The lifetime.",
			"The original action program type.",
			"The mapped action program type."
		)]
		#[document_parameters("The function to apply by reference.", "The censor cell.")]
		#[document_returns("A censor cell with `func` composed over the action thunk.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		RcBrand,
		/// 		WriterCensorBrand,
		/// 	},
		/// 	classes::{
		/// 		RefFunctor,
		/// 		ToDynCloneFn,
		/// 	},
		/// 	types::effects::writer::WriterCensor,
		/// };
		///
		/// let cell: WriterCensor<'static, RcBrand, String, i32> = WriterCensor::Censor {
		/// 	censor: <RcBrand as ToDynCloneFn>::new(|log: String| format!("{log}!")),
		/// 	action: <RcBrand as ToDynCloneFn>::new(|_: ()| 41),
		/// };
		/// let mapped =
		/// 	<WriterCensorBrand<RcBrand, String> as RefFunctor>::ref_map(|x: &i32| *x + 1, &cell);
		/// match mapped {
		/// 	WriterCensor::Censor {
		/// 		censor,
		/// 		action,
		/// 	} => {
		/// 		assert_eq!(censor("hello".to_owned()), "hello!");
		/// 		assert_eq!(action(()), 42);
		/// 	}
		/// }
		/// ```
		fn ref_map<'a, A: 'a, B: 'a>(
			func: impl Fn(&A) -> B + 'a,
			fa: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			match fa {
				WriterCensor::Censor {
					censor,
					action,
				} => {
					let censor = <RcBrand as RefCountedPointer>::Of::clone(censor);
					let action = <RcBrand as RefCountedPointer>::Of::clone(action);
					let func = <RcBrand as ToDynCloneFn>::ref_new::<A, B>(func);
					WriterCensor::Censor {
						censor,
						action: <RcBrand as ToDynCloneFn>::new(move |()| {
							let value = action(());
							func(&value)
						}),
					}
				}
			}
		}
	}

	#[document_type_parameters(
		"The log type observed by listen.",
		"The selected action value type."
	)]
	impl<W, Action> RefFunctor for BoxWriterListenBrand<BoxBrand, W, Action>
	where
		W: 'static,
		Action: 'static,
	{
		/// Ref-map stub for Box-backed Writer listen cells.
		#[document_signature]
		#[document_type_parameters(
			"The lifetime.",
			"The original action program type.",
			"The mapped action program type."
		)]
		#[document_parameters("The function to apply by reference.", "The listen cell.")]
		#[document_returns("A listen cell with a panicking action stub.")]
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	core::marker::PhantomData,
		/// 	fp_library::{
		/// 		brands::{
		/// 			BoxBrand,
		/// 			BoxWriterListenBrand,
		/// 		},
		/// 		classes::{
		/// 			RefFunctor,
		/// 			ToDynFnOnce,
		/// 		},
		/// 		types::effects::writer::BoxWriterListen,
		/// 	},
		/// };
		///
		/// let cell: BoxWriterListen<'static, BoxBrand, String, i32, i32> = BoxWriterListen::Listen {
		/// 	action: <BoxBrand as ToDynFnOnce>::new(|_: ()| 41),
		/// 	result: PhantomData,
		/// };
		/// let mapped = <BoxWriterListenBrand<BoxBrand, String, i32> as RefFunctor>::ref_map(
		/// 	|x: &i32| *x + 1,
		/// 	&cell,
		/// );
		/// match mapped {
		/// 	BoxWriterListen::Listen {
		/// 		action,
		/// 		result: _,
		/// 	} => assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| action(()))).is_err()),
		/// }
		/// ```
		#[expect(
			clippy::unreachable,
			reason = "BoxWriterListenBrand::ref_map cannot replicate a FnOnce action thunk through a reference."
		)]
		fn ref_map<'a, A: 'a, B: 'a>(
			_func: impl Fn(&A) -> B + 'a,
			_fa: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			BoxWriterListen::Listen {
				action: <BoxBrand as ToDynFnOnce>::new(|_: ()| -> B {
					unreachable!(
						"BoxWriterListenBrand::ref_map's stub action invoked; the FnOnce action thunk cannot be replicated through a reference"
					)
				}),
				result: PhantomData,
			}
		}
	}

	#[document_type_parameters(
		"The log type observed by listen.",
		"The selected action value type."
	)]
	impl<W, Action> RefFunctor for WriterListenBrand<RcBrand, W, Action>
	where
		W: 'static,
		Action: 'static,
	{
		/// Ref-map over the selected action program while preserving
		/// the selected action value type recorded by the brand.
		#[document_signature]
		#[document_type_parameters(
			"The lifetime.",
			"The original action program type.",
			"The mapped action program type."
		)]
		#[document_parameters("The function to apply by reference.", "The listen cell.")]
		#[document_returns("A listen cell with `func` composed over the action thunk.")]
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	core::marker::PhantomData,
		/// 	fp_library::{
		/// 		brands::{
		/// 			RcBrand,
		/// 			WriterListenBrand,
		/// 		},
		/// 		classes::{
		/// 			RefFunctor,
		/// 			ToDynCloneFn,
		/// 		},
		/// 		types::effects::writer::WriterListen,
		/// 	},
		/// };
		///
		/// let cell: WriterListen<'static, RcBrand, String, i32, i32> = WriterListen::Listen {
		/// 	action: <RcBrand as ToDynCloneFn>::new(|_: ()| 41),
		/// 	result: PhantomData,
		/// };
		/// let mapped =
		/// 	<WriterListenBrand<RcBrand, String, i32> as RefFunctor>::ref_map(|x: &i32| *x + 1, &cell);
		/// match mapped {
		/// 	WriterListen::Listen {
		/// 		action,
		/// 		result: _,
		/// 	} => assert_eq!(action(()), 42),
		/// }
		/// ```
		fn ref_map<'a, A: 'a, B: 'a>(
			func: impl Fn(&A) -> B + 'a,
			fa: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			match fa {
				WriterListen::Listen {
					action,
					result: _,
				} => {
					let action = <RcBrand as RefCountedPointer>::Of::clone(action);
					let func = <RcBrand as ToDynCloneFn>::ref_new::<A, B>(func);
					WriterListen::Listen {
						action: <RcBrand as ToDynCloneFn>::new(move |()| {
							let value = action(());
							func(&value)
						}),
						result: PhantomData,
					}
				}
			}
		}
	}

	// SendWriterCensorBrand and SendWriterListenBrand do not implement
	// RefFunctor: the RefFunctor method lacks the Send + Sync bounds
	// required to store the mapping closure behind Arc-backed trait
	// objects. This matches the existing Arc-backed scoped-effect
	// pattern.
}

pub use inner::*;
