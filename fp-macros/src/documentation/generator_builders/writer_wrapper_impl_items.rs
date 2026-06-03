use {
	super::{
		super::generator_descriptors::{
			self,
			EffectName,
			RunWrapperMethod,
			WrapperName,
		},
		impl_items_from_tokens,
	},
	proc_macro2::TokenStream,
	quote::quote,
	syn::ImplItem,
};

pub(super) fn writer_wrapper_impl_items_from_descriptor(
	wrapper: WrapperName,
	method: RunWrapperMethod,
) -> Option<syn::Result<Vec<ImplItem>>> {
	generator_descriptors::method_spec(EffectName::Writer, method)?;
	generator_descriptors::wrapper_spec(wrapper)?;

	let tokens = match (wrapper, method) {
		(WrapperName::Run, RunWrapperMethod::Tell) => run_tell_tokens(),
		(WrapperName::RcRun, RunWrapperMethod::Tell) => rcrun_tell_tokens(),
		(WrapperName::ArcRun, RunWrapperMethod::Tell) => arcrun_tell_tokens(),
		(WrapperName::RunExplicit, RunWrapperMethod::Tell) => run_explicit_tell_tokens(),
		(WrapperName::RcRunExplicit, RunWrapperMethod::Tell) => rcrun_explicit_tell_tokens(),
		(WrapperName::ArcRunExplicit, RunWrapperMethod::Tell) => arcrun_explicit_tell_tokens(),
		_ => return None,
	};

	Some(impl_items_from_tokens(tokens))
}

fn run_tell_tokens() -> TokenStream {
	quote! {
		/// Lifts a `Tell` writer effect into the Run program. Direct
		/// analog of PureScript Run's `tell`. The program emits the
		/// log value `log` and returns `()` as the result type.
		///
		/// `LogType` is the log type carried by `WriterBrand` in the
		/// row. Rust may need a turbofish on `LogType` because
		/// `tell`'s result type is `()` (which doesn't constrain the
		/// log type from the call site).
		#[__document_module_generated]
		#[document_signature]
		#[doc = ""]
		#[document_type_parameters(
			"The log type carried by `WriterBrand` in the row.",
			"The type-level Member-position witness (typically inferred)."
		)]
		#[doc = ""]
		#[document_parameters("The log value to emit.")]
		#[doc = ""]
		#[document_returns("A `Run` program suspended at the lifted `Tell` effect.")]
		#[doc = ""]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::run::Run,
		/// };
		///
		/// type FirstRow = CoproductBrand<CoyonedaBrand<WriterBrand<String>>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let prog: Run<FirstRow, Scoped, ()> = Run::tell::<String, _>("logged".to_string());
		/// let handled: Run<CNilBrand, CNilBrand, ((), String)> =
		/// 	prog.run_writer::<String, _, CNilBrand>();
		/// assert_eq!(handled.extract(), ((), "logged".to_string()));
		/// ```
		#[inline]
		pub fn tell<LogType: 'static, Idx>(log: LogType) -> Self
		where
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, ()>):
				crate::types::effects::member::Member<
						crate::types::Coyoneda<'static, crate::brands::WriterBrand<LogType>, ()>,
						Idx,
					>, {
			let effect: crate::types::effects::writer::Writer<'static, LogType, ()> =
				crate::types::effects::writer::Writer::Tell(log, (), core::marker::PhantomData);
			Self::lift::<crate::brands::WriterBrand<LogType>, Idx>(effect)
		}
	}
}

fn rcrun_tell_tokens() -> TokenStream {
	quote! {
		/// Lifts a `Tell` writer effect into the `RcRun` program.
		/// Mirrors [`Run::tell`](crate::types::effects::run::Run::tell);
		/// see that method for cross-wrapper semantics.
		#[__document_module_generated]
		#[document_signature]
		#[doc = ""]
		#[document_type_parameters(
			"The log type carried by `WriterBrand` in the row.",
			"The type-level Member-position witness (typically inferred)."
		)]
		#[doc = ""]
		#[document_parameters("The log value to emit.")]
		#[doc = ""]
		#[document_returns("An `RcRun` program suspended at the lifted `Tell` effect.")]
		#[doc = ""]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::rc_run::RcRun,
		/// };
		///
		/// type FirstRow = CoproductBrand<RcCoyonedaBrand<WriterBrand<String>>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let prog: RcRun<FirstRow, Scoped, ()> = RcRun::tell::<String, _>("logged".to_string());
		/// let handled: RcRun<CNilBrand, CNilBrand, ((), String)> =
		/// 	prog.run_writer::<String, _, CNilBrand>();
		/// assert_eq!(handled.extract(), ((), "logged".to_string()));
		/// ```
		#[inline]
		pub fn tell<LogType: Clone + 'static, Idx>(log: LogType) -> Self
		where
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, ()>):
				Member<RcCoyoneda<'static, crate::brands::WriterBrand<LogType>, ()>, Idx>,
			Apply!(<NodeBrand<R, ScopedRow> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<NodeBrand<R, ScopedRow>, crate::types::rc_free::RcTypeErasedValue>,
			>): Clone, {
			let effect: crate::types::effects::writer::Writer<'static, LogType, ()> =
				crate::types::effects::writer::Writer::Tell(log, (), core::marker::PhantomData);
			Self::lift::<crate::brands::WriterBrand<LogType>, Idx>(effect)
		}
	}
}

fn arcrun_tell_tokens() -> TokenStream {
	quote! {
		/// Lifts a `Tell` writer effect into the `ArcRun` program.
		/// Mirrors [`Run::tell`](crate::types::effects::run::Run::tell);
		/// see that method for cross-wrapper semantics. Differences for
		/// `ArcRun`: requires `LogType: Send + Sync` so the lifted
		/// layer participates in the Arc substrate's thread-safety
		/// cascade. The same
		/// [`WriterBrand`](crate::brands::WriterBrand) serves all six
		/// wrappers because [`Writer`](crate::types::effects::writer::Writer)
		/// has no `dyn Fn` continuation; no parallel `SendWriterBrand`
		/// is needed.
		#[__document_module_generated]
		#[document_signature]
		#[doc = ""]
		#[document_type_parameters(
			"The log type carried by `WriterBrand` in the row.",
			"The type-level Member-position witness (typically inferred)."
		)]
		#[doc = ""]
		#[document_parameters("The log value to emit.")]
		#[doc = ""]
		#[document_returns("An `ArcRun` program suspended at the lifted `Tell` effect.")]
		#[doc = ""]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::arc_run::ArcRun,
		/// };
		///
		/// type FirstRow = CoproductBrand<ArcCoyonedaBrand<WriterBrand<String>>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let prog: ArcRun<FirstRow, Scoped, ()> = ArcRun::tell::<String, _>("logged".to_string());
		/// let handled: ArcRun<CNilBrand, CNilBrand, ((), String)> =
		/// 	prog.run_writer::<String, _, CNilBrand>();
		/// assert_eq!(handled.extract(), ((), "logged".to_string()));
		/// ```
		#[inline]
		pub fn tell<LogType: Clone + Send + Sync + 'static, Idx>(log: LogType) -> Self
		where
			NodeBrand<R, ScopedRow>: SendFunctor,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, ()>):
				Member<ArcCoyoneda<'static, crate::brands::WriterBrand<LogType>, ()>, Idx>,
			Apply!(<NodeBrand<R, ScopedRow> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<NodeBrand<R, ScopedRow>, ArcTypeErasedValue>,
			>): Clone, {
			let effect: crate::types::effects::writer::Writer<'static, LogType, ()> =
				crate::types::effects::writer::Writer::Tell(log, (), core::marker::PhantomData);
			Self::lift::<crate::brands::WriterBrand<LogType>, Idx>(effect)
		}
	}
}

fn run_explicit_tell_tokens() -> TokenStream {
	quote! {
		/// Lifts a `Tell` writer effect into the `RunExplicit`
		/// program. Mirrors
		/// [`Run::tell`](crate::types::effects::run::Run::tell); see
		/// that method for cross-wrapper semantics.
		#[__document_module_generated]
		#[document_signature]
		#[doc = ""]
		#[document_type_parameters(
			"The log type carried by `WriterBrand` in the row.",
			"The type-level Member-position witness (typically inferred)."
		)]
		#[doc = ""]
		#[document_parameters("The log value to emit.")]
		#[doc = ""]
		#[document_returns("A `RunExplicit` program suspended at the lifted `Tell` effect.")]
		#[doc = ""]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::run_explicit::RunExplicit,
		/// };
		///
		/// type FirstRow = CoproductBrand<CoyonedaBrand<WriterBrand<String>>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let prog: RunExplicit<'static, FirstRow, Scoped, ()> =
		/// 	RunExplicit::tell::<String, _>("logged".to_string());
		/// let handled: RunExplicit<'static, CNilBrand, CNilBrand, ((), String)> =
		/// 	prog.run_writer::<String, _, CNilBrand>();
		/// assert_eq!(handled.extract(), ((), "logged".to_string()));
		/// ```
		#[inline]
		pub fn tell<LogType: 'static, Idx>(log: LogType) -> Self
		where
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, ()>):
				Member<Coyoneda<'a, crate::brands::WriterBrand<LogType>, ()>, Idx>, {
			let effect: crate::types::effects::writer::Writer<'a, LogType, ()> =
				crate::types::effects::writer::Writer::Tell(log, (), core::marker::PhantomData);
			Self::lift::<crate::brands::WriterBrand<LogType>, Idx>(effect)
		}
	}
}

fn rcrun_explicit_tell_tokens() -> TokenStream {
	quote! {
		/// Lifts a `Tell` writer effect into the `RcRunExplicit`
		/// program. Mirrors
		/// [`Run::tell`](crate::types::effects::run::Run::tell); see
		/// that method for cross-wrapper semantics.
		#[__document_module_generated]
		#[document_signature]
		#[doc = ""]
		#[document_type_parameters(
			"The log type carried by `WriterBrand` in the row.",
			"The type-level Member-position witness (typically inferred)."
		)]
		#[doc = ""]
		#[document_parameters("The log value to emit.")]
		#[doc = ""]
		#[document_returns("An `RcRunExplicit` program suspended at the lifted `Tell` effect.")]
		#[doc = ""]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::rc_run_explicit::RcRunExplicit,
		/// };
		///
		/// type FirstRow = CoproductBrand<RcCoyonedaBrand<WriterBrand<String>>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let prog: RcRunExplicit<'static, FirstRow, Scoped, ()> =
		/// 	RcRunExplicit::tell::<String, _>("logged".to_string());
		/// let handled: RcRunExplicit<'static, CNilBrand, CNilBrand, ((), String)> =
		/// 	prog.run_writer::<String, _, CNilBrand>();
		/// assert_eq!(handled.extract(), ((), "logged".to_string()));
		/// ```
		#[inline]
		pub fn tell<LogType: Clone + 'static, Idx>(log: LogType) -> Self
		where
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, ()>):
				Member<RcCoyoneda<'a, crate::brands::WriterBrand<LogType>, ()>, Idx>, {
			let effect: crate::types::effects::writer::Writer<'a, LogType, ()> =
				crate::types::effects::writer::Writer::Tell(log, (), core::marker::PhantomData);
			Self::lift::<crate::brands::WriterBrand<LogType>, Idx>(effect)
		}
	}
}

fn arcrun_explicit_tell_tokens() -> TokenStream {
	quote! {
		/// Lifts a `Tell` writer effect into the `ArcRunExplicit`
		/// program. Mirrors
		/// [`Run::tell`](crate::types::effects::run::Run::tell); see
		/// that method for cross-wrapper semantics. The same
		/// [`WriterBrand`](crate::brands::WriterBrand) serves all six
		/// wrappers because [`Writer`](crate::types::effects::writer::Writer)
		/// has no `dyn Fn` continuation; no parallel `SendWriterBrand`
		/// is needed. Requires `LogType: Send + Sync` so the lifted
		/// layer participates in the Arc substrate's thread-safety
		/// cascade.
		#[__document_module_generated]
		#[document_signature]
		#[doc = ""]
		#[document_type_parameters(
			"The log type carried by `WriterBrand` in the row.",
			"The type-level Member-position witness (typically inferred)."
		)]
		#[doc = ""]
		#[document_parameters("The log value to emit.")]
		#[doc = ""]
		#[document_returns("An `ArcRunExplicit` program suspended at the lifted `Tell` effect.")]
		#[doc = ""]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::arc_run_explicit::ArcRunExplicit,
		/// };
		///
		/// type FirstRow = CoproductBrand<ArcCoyonedaBrand<WriterBrand<String>>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let prog: ArcRunExplicit<'static, FirstRow, Scoped, ()> =
		/// 	ArcRunExplicit::tell::<String, _>("logged".to_string());
		/// let handled: ArcRunExplicit<'static, CNilBrand, CNilBrand, ((), String)> =
		/// 	prog.run_writer::<String, _, CNilBrand>();
		/// assert_eq!(handled.extract(), ((), "logged".to_string()));
		/// ```
		#[inline]
		pub fn tell<LogType: Clone + Send + Sync + 'static, Idx>(log: LogType) -> Self
		where
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, ()>):
				Member<ArcCoyoneda<'a, crate::brands::WriterBrand<LogType>, ()>, Idx>,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, ()>,
			>): Send + Sync,
			Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, ()>,
			>): Send + Sync,
			Apply!(<NodeBrand<R, ScopedRow> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, ()>,
			>): Clone + Send + Sync, {
			let effect: crate::types::effects::writer::Writer<'a, LogType, ()> =
				crate::types::effects::writer::Writer::Tell(log, (), core::marker::PhantomData);
			Self::lift::<crate::brands::WriterBrand<LogType>, Idx>(effect)
		}
	}
}
