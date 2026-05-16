//! Scoped instrumentation effect type with `Span` (attach a tag to
//! the execution of an action program) as its sole operation. The
//! corresponding brands are [`BoxSpanBrand`](crate::brands::BoxSpanBrand),
//! [`SpanBrand`](crate::brands::SpanBrand), and
//! [`SendSpanBrand`](crate::brands::SendSpanBrand).
//!
//! `Span` is Val-only at the user-facing dispatch level: no closure
//! receives borrowed user data, so there is no Ref flavour. The action
//! program is still stored as a unit-argument B-thunk to avoid the same
//! recursive layout cycle that Catch and Local avoid. The tag stays
//! stored by value; Rc-backed clone/ref-map paths require `Tag: Clone`
//! and Arc-backed paths require `Tag: Clone + Send + Sync` only where
//! the substrate needs those bounds.

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			Apply,
			brands::{
				ArcBrand,
				BoxBrand,
				BoxSpanBrand,
				RcBrand,
				SendSpanBrand,
				SpanBrand,
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

	// ===== BoxSpan (BoxBrand + FnOnce, single-shot) =====

	/// Scoped span effect for default `Run` / `RunExplicit`
	/// substrates. The tag is stored by value and the action is stored
	/// as a unit-argument `Box<dyn FnOnce>` thunk.
	#[document_type_parameters(
		"The lifetime of the action closure.",
		"The pointer brand storing the action thunk (BoxBrand only by structural bound).",
		"The span tag type.",
		"The result type of the action."
	)]
	pub enum BoxSpan<'a, P, Tag, A>
	where
		P: ToDynFnOnce,
		Tag: 'a,
		A: 'a, {
		/// Run the `action` thunk under instrumentation identified by
		/// `tag`.
		Span {
			/// The instrumentation tag stored by value.
			tag: Tag,
			/// The protected action program, stored as a unit-argument
			/// single-shot thunk.
			action: <P as Pointer>::Of<'a, dyn 'a + FnOnce(()) -> A>,
		},
	}

	impl_kind! {
		impl<P: ToDynFnOnce, Tag: 'static> for BoxSpanBrand<P, Tag> {
			type Of<'a, A: 'a>: 'a = BoxSpan<'a, P, Tag, A>;
		}
	}

	#[document_type_parameters("The span tag type.")]
	impl<Tag> Functor for BoxSpanBrand<BoxBrand, Tag>
	where
		Tag: 'static,
	{
		/// Maps `f` over the action result while preserving the tag.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the action closure.",
			"The original result type.",
			"The new result type after applying `f`."
		)]
		///
		#[document_parameters("The function to apply.", "The span effect.")]
		///
		#[document_returns("A new span effect with `f` composed over the action.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		BoxBrand,
		/// 		BoxSpanBrand,
		/// 	},
		/// 	classes::{
		/// 		Functor,
		/// 		ToDynFnOnce,
		/// 	},
		/// 	types::effects::span::BoxSpan,
		/// };
		///
		/// let span: BoxSpan<'static, BoxBrand, &'static str, i32> = BoxSpan::Span {
		/// 	tag: "request",
		/// 	action: <BoxBrand as ToDynFnOnce>::new(|_: ()| 41),
		/// };
		/// let mapped = <BoxSpanBrand<BoxBrand, &'static str> as Functor>::map(|x| x + 1, span);
		/// match mapped {
		/// 	BoxSpan::Span {
		/// 		tag,
		/// 		action,
		/// 	} => {
		/// 		assert_eq!(tag, "request");
		/// 		assert_eq!(action(()), 42);
		/// 	}
		/// }
		/// ```
		fn map<'a, A: 'a, B: 'a>(
			f: impl Fn(A) -> B + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			match fa {
				BoxSpan::Span {
					tag,
					action,
				} => BoxSpan::Span {
					tag,
					action: <BoxBrand as ToDynFnOnce>::new(move |()| f(action(()))),
				},
			}
		}
	}

	// ===== Span (RcBrand + Fn, multi-shot clone-capable) =====

	/// Scoped span effect for `RcRun` / `RcRunExplicit` substrates.
	/// The tag is stored by value and the action is stored as a
	/// unit-argument `Rc<dyn Fn>` thunk.
	#[document_type_parameters(
		"The lifetime of the action closure.",
		"The pointer brand storing the action thunk (RcBrand only).",
		"The span tag type.",
		"The result type of the action."
	)]
	pub enum Span<'a, P, Tag, A>
	where
		P: ToDynCloneFn,
		Tag: 'a,
		A: 'a, {
		/// Run the `action` thunk under instrumentation identified by
		/// `tag`.
		Span {
			/// The instrumentation tag stored by value.
			tag: Tag,
			/// The protected action program, stored as a unit-argument
			/// multi-shot thunk.
			action: <P as RefCountedPointer>::Of<'a, dyn 'a + Fn(()) -> A>,
		},
	}

	impl_kind! {
		impl<P: ToDynCloneFn, Tag: 'static> for SpanBrand<P, Tag> {
			type Of<'a, A: 'a>: 'a = Span<'a, P, Tag, A>;
		}
	}

	#[document_type_parameters(
		"The lifetime of the action closure.",
		"The pointer brand storing the action thunk.",
		"The span tag type.",
		"The result type of the action."
	)]
	#[document_parameters("The span effect to clone.")]
	impl<'a, P, Tag, A> Clone for Span<'a, P, Tag, A>
	where
		P: ToDynCloneFn,
		Tag: Clone + 'a,
		A: 'a,
	{
		/// Clones the span effect by cloning the by-value tag and
		/// refcount-bumping the stored action thunk.
		#[document_signature]
		///
		#[document_returns("A new span effect sharing the action thunk by refcount.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::RcBrand,
		/// 	classes::ToDynCloneFn,
		/// 	types::effects::span::Span,
		/// };
		///
		/// let span: Span<'static, RcBrand, String, i32> = Span::Span {
		/// 	tag: "request".to_owned(),
		/// 	action: <RcBrand as ToDynCloneFn>::new(|_: ()| 42),
		/// };
		/// let cloned = span.clone();
		/// match cloned {
		/// 	Span::Span {
		/// 		tag,
		/// 		action,
		/// 	} => {
		/// 		assert_eq!(tag, "request");
		/// 		assert_eq!(action(()), 42);
		/// 	}
		/// }
		/// ```
		fn clone(&self) -> Self {
			match self {
				Span::Span {
					tag,
					action,
				} => Span::Span {
					tag: tag.clone(),
					action: <P as RefCountedPointer>::Of::clone(action),
				},
			}
		}
	}

	#[document_type_parameters("The span tag type.")]
	impl<Tag> Functor for SpanBrand<RcBrand, Tag>
	where
		Tag: 'static,
	{
		/// Maps `f` over the action result while preserving the tag.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the action closure.",
			"The original result type.",
			"The new result type after applying `f`."
		)]
		///
		#[document_parameters("The function to apply.", "The span effect.")]
		///
		#[document_returns("A new span effect with `f` composed over the action.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		RcBrand,
		/// 		SpanBrand,
		/// 	},
		/// 	classes::{
		/// 		Functor,
		/// 		ToDynCloneFn,
		/// 	},
		/// 	types::effects::span::Span,
		/// };
		///
		/// let span: Span<'static, RcBrand, &'static str, i32> = Span::Span {
		/// 	tag: "request",
		/// 	action: <RcBrand as ToDynCloneFn>::new(|_: ()| 41),
		/// };
		/// let mapped = <SpanBrand<RcBrand, &'static str> as Functor>::map(|x| x + 1, span);
		/// match mapped {
		/// 	Span::Span {
		/// 		tag,
		/// 		action,
		/// 	} => {
		/// 		assert_eq!(tag, "request");
		/// 		assert_eq!(action(()), 42);
		/// 	}
		/// }
		/// ```
		fn map<'a, A: 'a, B: 'a>(
			f: impl Fn(A) -> B + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			match fa {
				Span::Span {
					tag,
					action,
				} => Span::Span {
					tag,
					action: <RcBrand as ToDynCloneFn>::new(move |()| f(action(()))),
				},
			}
		}
	}

	// ===== SendSpan (ArcBrand + Fn + Send + Sync, thread-safe) =====

	/// Scoped span effect for `ArcRun` / `ArcRunExplicit` substrates.
	/// The tag is stored by value and the action is stored as a
	/// unit-argument `Arc<dyn Fn + Send + Sync>` thunk.
	#[document_type_parameters(
		"The lifetime of the action closure.",
		"The pointer brand storing the action thunk (ArcBrand only).",
		"The span tag type.",
		"The result type of the action."
	)]
	pub enum SendSpan<'a, P, Tag, A>
	where
		P: ToDynSendFn,
		Tag: Send + Sync + 'a,
		A: 'a, {
		/// Run the `action` thunk under instrumentation identified by
		/// `tag`.
		Span {
			/// The instrumentation tag stored by value.
			tag: Tag,
			/// The protected action program, stored as a unit-argument
			/// thread-safe multi-shot thunk.
			action: <P as SendRefCountedPointer>::Of<'a, dyn 'a + Fn(()) -> A + Send + Sync>,
		},
	}

	impl_kind! {
		impl<P: ToDynSendFn, Tag: Send + Sync + 'static> for SendSpanBrand<P, Tag> {
			type Of<'a, A: 'a>: 'a = SendSpan<'a, P, Tag, A>;
		}
	}

	#[document_type_parameters(
		"The lifetime of the action closure.",
		"The pointer brand storing the action thunk.",
		"The span tag type.",
		"The result type of the action."
	)]
	#[document_parameters("The span effect to clone.")]
	impl<'a, P, Tag, A> Clone for SendSpan<'a, P, Tag, A>
	where
		P: ToDynSendFn,
		Tag: Clone + Send + Sync + 'a,
		A: 'a,
	{
		/// Clones the span effect by cloning the by-value tag and
		/// refcount-bumping the stored action thunk.
		#[document_signature]
		///
		#[document_returns("A new span effect sharing the action thunk by refcount.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::ArcBrand,
		/// 	classes::ToDynSendFn,
		/// 	types::effects::span::SendSpan,
		/// };
		///
		/// let span: SendSpan<'static, ArcBrand, String, i32> = SendSpan::Span {
		/// 	tag: "request".to_owned(),
		/// 	action: <ArcBrand as ToDynSendFn>::new(|_: ()| 42),
		/// };
		/// let cloned = span.clone();
		/// match cloned {
		/// 	SendSpan::Span {
		/// 		tag,
		/// 		action,
		/// 	} => {
		/// 		assert_eq!(tag, "request");
		/// 		assert_eq!(action(()), 42);
		/// 	}
		/// }
		/// ```
		fn clone(&self) -> Self {
			match self {
				SendSpan::Span {
					tag,
					action,
				} => SendSpan::Span {
					tag: tag.clone(),
					action: <P as SendRefCountedPointer>::Of::clone(action),
				},
			}
		}
	}

	// SendSpanBrand does not implement Functor: the trait method's
	// closure lacks the Send + Sync bounds required to rebuild an
	// Arc-backed action thunk. Arc-family substrates use SendFunctor.

	#[document_type_parameters("The span tag type.")]
	impl<Tag> SendFunctor for SendSpanBrand<ArcBrand, Tag>
	where
		Tag: Send + Sync + 'static,
	{
		/// Thread-safe map over the action result while preserving the
		/// tag.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the action closure.",
			"The original result type.",
			"The new result type after applying `f`."
		)]
		///
		#[document_parameters("The function to apply.", "The span effect.")]
		///
		#[document_returns("A new span effect with `f` composed over the action.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		ArcBrand,
		/// 		SendSpanBrand,
		/// 	},
		/// 	classes::{
		/// 		SendFunctor,
		/// 		ToDynSendFn,
		/// 	},
		/// 	types::effects::span::SendSpan,
		/// };
		///
		/// let span: SendSpan<'static, ArcBrand, &'static str, i32> = SendSpan::Span {
		/// 	tag: "request",
		/// 	action: <ArcBrand as ToDynSendFn>::new(|_: ()| 41),
		/// };
		/// let mapped = <SendSpanBrand<ArcBrand, &'static str> as SendFunctor>::send_map(|x| x + 1, span);
		/// match mapped {
		/// 	SendSpan::Span {
		/// 		tag,
		/// 		action,
		/// 	} => {
		/// 		assert_eq!(tag, "request");
		/// 		assert_eq!(action(()), 42);
		/// 	}
		/// }
		/// ```
		fn send_map<'a, A: Send + Sync + 'a, B: Send + Sync + 'a>(
			f: impl Fn(A) -> B + Send + Sync + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			match fa {
				SendSpan::Span {
					tag,
					action,
				} => SendSpan::Span {
					tag,
					action: <ArcBrand as ToDynSendFn>::new(move |()| f(action(()))),
				},
			}
		}
	}

	#[document_type_parameters("The span tag type.")]
	impl<Tag> SendFunctor for BoxSpanBrand<BoxBrand, Tag>
	where
		Tag: Send + Sync + 'static,
	{
		/// SendFunctor stub for the BoxBrand-flavoured Span. Delegates
		/// to [`Functor::map`]; this path is not exercised by Arc
		/// substrates.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the action closure.",
			"The original result type.",
			"The new result type after applying `f`."
		)]
		///
		#[document_parameters("The function to apply.", "The span effect.")]
		///
		#[document_returns("A new span effect with `f` composed over the action.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		BoxBrand,
		/// 		BoxSpanBrand,
		/// 	},
		/// 	classes::{
		/// 		SendFunctor,
		/// 		ToDynFnOnce,
		/// 	},
		/// 	types::effects::span::BoxSpan,
		/// };
		///
		/// let span: BoxSpan<'static, BoxBrand, &'static str, i32> = BoxSpan::Span {
		/// 	tag: "request",
		/// 	action: <BoxBrand as ToDynFnOnce>::new(|_: ()| 41),
		/// };
		/// let mapped = <BoxSpanBrand<BoxBrand, &'static str> as SendFunctor>::send_map(|x| x + 1, span);
		/// match mapped {
		/// 	BoxSpan::Span {
		/// 		tag,
		/// 		action,
		/// 	} => {
		/// 		assert_eq!(tag, "request");
		/// 		assert_eq!(action(()), 42);
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

	#[document_type_parameters("The span tag type.")]
	impl<Tag> SendFunctor for SpanBrand<RcBrand, Tag>
	where
		Tag: Send + Sync + 'static,
	{
		/// SendFunctor stub for the RcBrand-flavoured Span. Delegates
		/// to [`Functor::map`]; this path is not exercised by Arc
		/// substrates.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the action closure.",
			"The original result type.",
			"The new result type after applying `f`."
		)]
		///
		#[document_parameters("The function to apply.", "The span effect.")]
		///
		#[document_returns("A new span effect with `f` composed over the action.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		RcBrand,
		/// 		SpanBrand,
		/// 	},
		/// 	classes::{
		/// 		SendFunctor,
		/// 		ToDynCloneFn,
		/// 	},
		/// 	types::effects::span::Span,
		/// };
		///
		/// let span: Span<'static, RcBrand, &'static str, i32> = Span::Span {
		/// 	tag: "request",
		/// 	action: <RcBrand as ToDynCloneFn>::new(|_: ()| 41),
		/// };
		/// let mapped = <SpanBrand<RcBrand, &'static str> as SendFunctor>::send_map(|x| x + 1, span);
		/// match mapped {
		/// 	Span::Span {
		/// 		tag,
		/// 		action,
		/// 	} => {
		/// 		assert_eq!(tag, "request");
		/// 		assert_eq!(action(()), 42);
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

	#[document_type_parameters("The span tag type.")]
	impl<Tag> WrapDrop for BoxSpanBrand<BoxBrand, Tag>
	where
		Tag: 'static,
	{
		/// Decomposes a span by materialising and returning the action.
		#[document_signature]
		///
		#[document_type_parameters("The lifetime.", "The result type.")]
		///
		#[document_parameters("The span effect to decompose.")]
		///
		#[document_returns("`Some` of the materialised action; the tag is dropped.")]
		///
		#[document_examples(skip_call_check)]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		BoxBrand,
		/// 		BoxSpanBrand,
		/// 	},
		/// 	classes::{
		/// 		ToDynFnOnce,
		/// 		WrapDrop,
		/// 	},
		/// 	types::effects::span::BoxSpan,
		/// };
		///
		/// let span: BoxSpan<'static, BoxBrand, &'static str, i32> = BoxSpan::Span {
		/// 	tag: "request",
		/// 	action: <BoxBrand as ToDynFnOnce>::new(|_: ()| 42),
		/// };
		/// assert_eq!(<BoxSpanBrand<BoxBrand, &'static str> as WrapDrop>::drop(span), Some(42));
		/// ```
		fn drop<'a, X: 'a>(
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, X>)
		) -> Option<X> {
			match fa {
				BoxSpan::Span {
					tag: _,
					action,
				} => Some(action(())),
			}
		}
	}

	#[document_type_parameters("The span tag type.")]
	impl<Tag> WrapDrop for SpanBrand<RcBrand, Tag>
	where
		Tag: 'static,
	{
		/// Decomposes a span by materialising and returning the action.
		#[document_signature]
		///
		#[document_type_parameters("The lifetime.", "The result type.")]
		///
		#[document_parameters("The span effect to decompose.")]
		///
		#[document_returns("`Some` of the materialised action; the tag is dropped.")]
		///
		#[document_examples(skip_call_check)]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		RcBrand,
		/// 		SpanBrand,
		/// 	},
		/// 	classes::{
		/// 		ToDynCloneFn,
		/// 		WrapDrop,
		/// 	},
		/// 	types::effects::span::Span,
		/// };
		///
		/// let span: Span<'static, RcBrand, &'static str, i32> = Span::Span {
		/// 	tag: "request",
		/// 	action: <RcBrand as ToDynCloneFn>::new(|_: ()| 42),
		/// };
		/// assert_eq!(<SpanBrand<RcBrand, &'static str> as WrapDrop>::drop(span), Some(42));
		/// ```
		fn drop<'a, X: 'a>(
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, X>)
		) -> Option<X> {
			match fa {
				Span::Span {
					tag: _,
					action,
				} => Some(action(())),
			}
		}
	}

	#[document_type_parameters("The span tag type.")]
	impl<Tag> WrapDrop for SendSpanBrand<ArcBrand, Tag>
	where
		Tag: Send + Sync + 'static,
	{
		/// Decomposes a span by materialising and returning the action.
		#[document_signature]
		///
		#[document_type_parameters("The lifetime.", "The result type.")]
		///
		#[document_parameters("The span effect to decompose.")]
		///
		#[document_returns("`Some` of the materialised action; the tag is dropped.")]
		///
		#[document_examples(skip_call_check)]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		ArcBrand,
		/// 		SendSpanBrand,
		/// 	},
		/// 	classes::{
		/// 		ToDynSendFn,
		/// 		WrapDrop,
		/// 	},
		/// 	types::effects::span::SendSpan,
		/// };
		///
		/// let span: SendSpan<'static, ArcBrand, &'static str, i32> = SendSpan::Span {
		/// 	tag: "request",
		/// 	action: <ArcBrand as ToDynSendFn>::new(|_: ()| 42),
		/// };
		/// assert_eq!(<SendSpanBrand<ArcBrand, &'static str> as WrapDrop>::drop(span), Some(42));
		/// ```
		fn drop<'a, X: 'a>(
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, X>)
		) -> Option<X> {
			match fa {
				SendSpan::Span {
					tag: _,
					action,
				} => Some(action(())),
			}
		}
	}

	// ===== Extract impls =====

	#[document_type_parameters("The span tag type.")]
	impl<Tag> Extract for BoxSpanBrand<BoxBrand, Tag>
	where
		Tag: 'static,
	{
		/// Extracts the span action by invoking its thunk.
		#[document_signature]
		///
		#[document_type_parameters("The lifetime.", "The result type.")]
		///
		#[document_parameters("The span effect.")]
		///
		#[document_returns("The materialised action.")]
		///
		#[document_examples(skip_call_check)]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		BoxBrand,
		/// 		BoxSpanBrand,
		/// 	},
		/// 	classes::{
		/// 		Extract,
		/// 		ToDynFnOnce,
		/// 	},
		/// 	types::effects::span::BoxSpan,
		/// };
		///
		/// let span: BoxSpan<'static, BoxBrand, &'static str, i32> = BoxSpan::Span {
		/// 	tag: "request",
		/// 	action: <BoxBrand as ToDynFnOnce>::new(|_: ()| 42),
		/// };
		/// assert_eq!(<BoxSpanBrand<BoxBrand, &'static str> as Extract>::extract(span), 42);
		/// ```
		fn extract<'a, A: 'a>(
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)
		) -> A {
			match fa {
				BoxSpan::Span {
					tag: _,
					action,
				} => action(()),
			}
		}
	}

	#[document_type_parameters("The span tag type.")]
	impl<Tag> Extract for SpanBrand<RcBrand, Tag>
	where
		Tag: 'static,
	{
		/// Extracts the span action by invoking its thunk.
		#[document_signature]
		///
		#[document_type_parameters("The lifetime.", "The result type.")]
		///
		#[document_parameters("The span effect.")]
		///
		#[document_returns("The materialised action.")]
		///
		#[document_examples(skip_call_check)]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		RcBrand,
		/// 		SpanBrand,
		/// 	},
		/// 	classes::{
		/// 		Extract,
		/// 		ToDynCloneFn,
		/// 	},
		/// 	types::effects::span::Span,
		/// };
		///
		/// let span: Span<'static, RcBrand, &'static str, i32> = Span::Span {
		/// 	tag: "request",
		/// 	action: <RcBrand as ToDynCloneFn>::new(|_: ()| 42),
		/// };
		/// assert_eq!(<SpanBrand<RcBrand, &'static str> as Extract>::extract(span), 42);
		/// ```
		fn extract<'a, A: 'a>(
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)
		) -> A {
			match fa {
				Span::Span {
					tag: _,
					action,
				} => action(()),
			}
		}
	}

	#[document_type_parameters("The span tag type.")]
	impl<Tag> Extract for SendSpanBrand<ArcBrand, Tag>
	where
		Tag: Send + Sync + 'static,
	{
		/// Extracts the span action by invoking its thunk.
		#[document_signature]
		///
		#[document_type_parameters("The lifetime.", "The result type.")]
		///
		#[document_parameters("The span effect.")]
		///
		#[document_returns("The materialised action.")]
		///
		#[document_examples(skip_call_check)]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		ArcBrand,
		/// 		SendSpanBrand,
		/// 	},
		/// 	classes::{
		/// 		Extract,
		/// 		ToDynSendFn,
		/// 	},
		/// 	types::effects::span::SendSpan,
		/// };
		///
		/// let span: SendSpan<'static, ArcBrand, &'static str, i32> = SendSpan::Span {
		/// 	tag: "request",
		/// 	action: <ArcBrand as ToDynSendFn>::new(|_: ()| 42),
		/// };
		/// assert_eq!(<SendSpanBrand<ArcBrand, &'static str> as Extract>::extract(span), 42);
		/// ```
		fn extract<'a, A: 'a>(
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)
		) -> A {
			match fa {
				SendSpan::Span {
					tag: _,
					action,
				} => action(()),
			}
		}
	}

	// ===== Brand-projection helpers (RefFunctor support) =====

	/// Projects the tag reference out of a [`BoxSpan`] GAT projection.
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the span effect's contents.",
		"The borrow lifetime of the input projection.",
		"The pointer brand storing the action thunk.",
		"The span tag type.",
		"The result type of the action."
	)]
	///
	#[document_parameters("The span effect projection.")]
	///
	#[document_returns("A reference to the stored span tag.")]
	///
	#[document_examples(skip_call_check)]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::BoxBrand,
	/// 	classes::ToDynFnOnce,
	/// 	types::effects::span::{
	/// 		BoxSpan,
	/// 		box_span_tag_ref,
	/// 	},
	/// };
	///
	/// let span: BoxSpan<'static, BoxBrand, &'static str, i32> = BoxSpan::Span {
	/// 	tag: "request",
	/// 	action: <BoxBrand as ToDynFnOnce>::new(|_: ()| 42),
	/// };
	/// assert_eq!(*box_span_tag_ref::<BoxBrand, &'static str, i32>(&span), "request");
	/// ```
	#[doc(hidden)]
	pub fn box_span_tag_ref<'a, 'b, P, Tag, A>(
		fa: &'b Apply!(<BoxSpanBrand<P, Tag> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)
	) -> &'b Tag
	where
		P: ToDynFnOnce,
		Tag: 'static,
		A: 'a, {
		match fa {
			BoxSpan::Span {
				tag,
				action: _,
			} => tag,
		}
	}

	/// Projects the tag reference out of a [`Span`] GAT projection.
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the span effect's contents.",
		"The borrow lifetime of the input projection.",
		"The pointer brand storing the action thunk.",
		"The span tag type.",
		"The result type of the action."
	)]
	///
	#[document_parameters("The span effect projection.")]
	///
	#[document_returns("A reference to the stored span tag.")]
	///
	#[document_examples(skip_call_check)]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::RcBrand,
	/// 	classes::ToDynCloneFn,
	/// 	types::effects::span::{
	/// 		Span,
	/// 		span_tag_ref,
	/// 	},
	/// };
	///
	/// let span: Span<'static, RcBrand, &'static str, i32> = Span::Span {
	/// 	tag: "request",
	/// 	action: <RcBrand as ToDynCloneFn>::new(|_: ()| 42),
	/// };
	/// assert_eq!(*span_tag_ref::<RcBrand, &'static str, i32>(&span), "request");
	/// ```
	#[doc(hidden)]
	pub fn span_tag_ref<'a, 'b, P, Tag, A>(
		fa: &'b Apply!(<SpanBrand<P, Tag> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)
	) -> &'b Tag
	where
		P: ToDynCloneFn,
		Tag: 'static,
		A: 'a, {
		match fa {
			Span::Span {
				tag,
				action: _,
			} => tag,
		}
	}

	/// Projects the action-thunk reference out of a [`Span`] GAT
	/// projection.
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the span effect's contents.",
		"The borrow lifetime of the input projection.",
		"The pointer brand storing the action thunk.",
		"The span tag type.",
		"The result type of the action."
	)]
	///
	#[document_parameters("The span effect projection.")]
	///
	#[document_returns("A reference to the stored action-thunk pointer.")]
	///
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::RcBrand,
	/// 	classes::ToDynCloneFn,
	/// 	types::effects::span::{
	/// 		Span,
	/// 		span_action_thunk_ref,
	/// 	},
	/// };
	///
	/// let span: Span<'static, RcBrand, &'static str, i32> = Span::Span {
	/// 	tag: "request",
	/// 	action: <RcBrand as ToDynCloneFn>::new(|_: ()| 42),
	/// };
	/// let action = span_action_thunk_ref::<RcBrand, &'static str, i32>(&span);
	/// assert_eq!(action(()), 42);
	/// ```
	#[doc(hidden)]
	pub fn span_action_thunk_ref<'a, 'b, P, Tag, A>(
		fa: &'b Apply!(<SpanBrand<P, Tag> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)
	) -> &'b <P as RefCountedPointer>::Of<'a, dyn 'a + Fn(()) -> A>
	where
		P: ToDynCloneFn,
		Tag: 'static,
		A: 'a, {
		match fa {
			Span::Span {
				tag: _,
				action,
			} => action,
		}
	}

	// ===== RefFunctor impls =====

	#[document_type_parameters("The span tag type.")]
	impl<Tag> RefFunctor for BoxSpanBrand<BoxBrand, Tag>
	where
		Tag: Clone + 'static,
	{
		/// Ref-map stub for BoxSpan. The tag can be cloned from the
		/// borrowed cell, but the `Box<dyn FnOnce>` action cannot be
		/// replicated through a reference, so the new action is a
		/// panicking thunk.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the action closure.",
			"The original result type.",
			"The new result type after applying `func`."
		)]
		///
		#[document_parameters("The function to apply by reference.", "The span effect.")]
		///
		#[document_returns("A new span effect with a cloned tag and a panicking action stub.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		BoxBrand,
		/// 		BoxSpanBrand,
		/// 	},
		/// 	classes::{
		/// 		RefFunctor,
		/// 		ToDynFnOnce,
		/// 	},
		/// 	types::effects::span::BoxSpan,
		/// };
		///
		/// let span: BoxSpan<'static, BoxBrand, String, i32> = BoxSpan::Span {
		/// 	tag: "request".to_owned(),
		/// 	action: <BoxBrand as ToDynFnOnce>::new(|_: ()| 41),
		/// };
		/// let mapped = <BoxSpanBrand<BoxBrand, String> as RefFunctor>::ref_map(|x: &i32| *x + 1, &span);
		/// match mapped {
		/// 	BoxSpan::Span {
		/// 		tag,
		/// 		action,
		/// 	} => {
		/// 		assert_eq!(tag, "request");
		/// 		assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| action(()))).is_err());
		/// 	}
		/// }
		/// ```
		#[expect(
			clippy::unreachable,
			reason = "BoxSpanBrand::ref_map cannot replicate a FnOnce action thunk through a reference."
		)]
		fn ref_map<'a, A: 'a, B: 'a>(
			_func: impl Fn(&A) -> B + 'a,
			fa: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			let tag = box_span_tag_ref::<BoxBrand, Tag, A>(fa).clone();
			BoxSpan::Span {
				tag,
				action: <BoxBrand as ToDynFnOnce>::new(|_: ()| -> B {
					unreachable!(
						"BoxSpanBrand::ref_map's stub action invoked; the FnOnce action thunk cannot be replicated through a reference"
					)
				}),
			}
		}
	}

	#[document_type_parameters("The span tag type.")]
	impl<Tag> RefFunctor for SpanBrand<RcBrand, Tag>
	where
		Tag: Clone + 'static,
	{
		/// Ref-map over the action result while cloning the by-value
		/// tag and refcount-bumping the action thunk.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the action closure.",
			"The original result type.",
			"The new result type after applying `func`."
		)]
		///
		#[document_parameters("The function to apply by reference.", "The span effect.")]
		///
		#[document_returns("A new span effect with `func` composed over the action.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		RcBrand,
		/// 		SpanBrand,
		/// 	},
		/// 	classes::{
		/// 		RefFunctor,
		/// 		ToDynCloneFn,
		/// 	},
		/// 	types::effects::span::Span,
		/// };
		///
		/// let span: Span<'static, RcBrand, String, i32> = Span::Span {
		/// 	tag: "request".to_owned(),
		/// 	action: <RcBrand as ToDynCloneFn>::new(|_: ()| 41),
		/// };
		/// let mapped = <SpanBrand<RcBrand, String> as RefFunctor>::ref_map(|x: &i32| *x + 1, &span);
		/// match mapped {
		/// 	Span::Span {
		/// 		tag,
		/// 		action,
		/// 	} => {
		/// 		assert_eq!(tag, "request");
		/// 		assert_eq!(action(()), 42);
		/// 	}
		/// }
		/// ```
		fn ref_map<'a, A: 'a, B: 'a>(
			func: impl Fn(&A) -> B + 'a,
			fa: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			let tag = span_tag_ref::<RcBrand, Tag, A>(fa).clone();
			let action_thunk_ref = span_action_thunk_ref::<RcBrand, Tag, A>(fa);
			let action_thunk_clone = <RcBrand as RefCountedPointer>::Of::clone(action_thunk_ref);
			let func_rc = <RcBrand as ToDynCloneFn>::ref_new::<A, B>(func);
			let action = <RcBrand as ToDynCloneFn>::new::<(), B>(move |()| -> B {
				let a: A = action_thunk_clone(());
				func_rc(&a)
			});
			Span::Span {
				tag,
				action,
			}
		}
	}

	// SendSpanBrand does not implement RefFunctor for the same reason
	// SendCatchBrand and SendLocalBrand omit it: RefFunctor's closure
	// parameter lacks the Send + Sync bounds required by Arc storage,
	// and the ArcExplicit brand-level Ref cascade does not require it.
}

pub use inner::*;
