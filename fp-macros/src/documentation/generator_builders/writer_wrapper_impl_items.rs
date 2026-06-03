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
		(WrapperName::Run, RunWrapperMethod::FoldWriter) => run_fold_writer_tokens(),
		(WrapperName::Run, RunWrapperMethod::RunWriter) => run_run_writer_tokens(),
		(WrapperName::RcRun, RunWrapperMethod::Tell) => rcrun_tell_tokens(),
		(WrapperName::RcRun, RunWrapperMethod::FoldWriter) => rcrun_fold_writer_tokens(),
		(WrapperName::RcRun, RunWrapperMethod::RunWriter) => rcrun_run_writer_tokens(),
		(WrapperName::ArcRun, RunWrapperMethod::Tell) => arcrun_tell_tokens(),
		(WrapperName::ArcRun, RunWrapperMethod::FoldWriter) => arcrun_fold_writer_tokens(),
		(WrapperName::ArcRun, RunWrapperMethod::RunWriter) => arcrun_run_writer_tokens(),
		(WrapperName::RunExplicit, RunWrapperMethod::Tell) => run_explicit_tell_tokens(),
		(WrapperName::RunExplicit, RunWrapperMethod::FoldWriter) =>
			run_explicit_fold_writer_tokens(),
		(WrapperName::RunExplicit, RunWrapperMethod::RunWriter) => run_explicit_run_writer_tokens(),
		(WrapperName::RcRunExplicit, RunWrapperMethod::Tell) => rcrun_explicit_tell_tokens(),
		(WrapperName::RcRunExplicit, RunWrapperMethod::FoldWriter) =>
			rcrun_explicit_fold_writer_tokens(),
		(WrapperName::RcRunExplicit, RunWrapperMethod::RunWriter) =>
			rcrun_explicit_run_writer_tokens(),
		(WrapperName::ArcRunExplicit, RunWrapperMethod::Tell) => arcrun_explicit_tell_tokens(),
		(WrapperName::ArcRunExplicit, RunWrapperMethod::FoldWriter) =>
			arcrun_explicit_fold_writer_tokens(),
		(WrapperName::ArcRunExplicit, RunWrapperMethod::RunWriter) =>
			arcrun_explicit_run_writer_tokens(),
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

fn run_fold_writer_tokens() -> TokenStream {
	quote! {
		/// Interprets one Writer effect with an explicit accumulator function.
		///
		/// Each `tell(log)` contributes `folder(current, log)` in program
		/// order. Other first-order effects remain in the narrowed row and
		/// continue to be represented by the returned `Run` program.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters(
			"The log type carried by `WriterBrand`.",
			"The accumulated value type.",
			"The type-level Member-position witness for the Writer effect.",
			"The first-order row brand with the Writer effect removed."
		)]
		#[document_parameters(
			"The initial accumulator value.",
			"The function that incorporates one emitted log into the accumulator."
		)]
		#[document_returns(
			"A first-order-only `Run` program returning `(result, accumulated_log)`."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::run::Run,
		/// };
		///
		/// type Row = CoproductBrand<CoyonedaBrand<WriterBrand<String>>, CNilBrand>;
		///
		/// let program: Run<Row, CNilBrand, i32> =
		/// 	Run::<Row, CNilBrand, ()>::tell::<String, _>("first".to_string())
		/// 		.bind(|()| Run::<Row, CNilBrand, ()>::tell::<String, _>("second".to_string()))
		/// 		.bind(|()| Run::<Row, CNilBrand, i32>::pure(7));
		/// let handled: Run<CNilBrand, CNilBrand, (i32, Vec<String>)> = program
		/// 	.fold_writer::<String, Vec<String>, _, CNilBrand>(Vec::new(), |mut logs, log| {
		/// 		logs.push(log);
		/// 		logs
		/// 	});
		/// assert_eq!(handled.extract(), (7, vec!["first".to_string(), "second".to_string()]));
		/// ```
		#[inline]
		pub fn fold_writer<LogType, Acc, Idx, RMinusWriter>(
			self,
			initial: Acc,
			folder: impl Fn(Acc, LogType) -> Acc + 'static,
		) -> Run<RMinusWriter, CNilBrand, (A, Acc)>
		where
			LogType: Clone + 'static,
			Acc: Clone + 'static,
			RMinusWriter: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				Run<R, CNilBrand, A>,
			>): Member<
					Coyoneda<'static, WriterBrand<LogType>, Run<R, CNilBrand, A>>,
					Idx,
					Remainder = Apply!(
									<RMinusWriter as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
										'static,
										Run<R, CNilBrand, A>,
									>
								),
				>, {
			let folder = StdRc::new(folder);
			let accumulator: SharedWriterFoldCell<'static, Acc> =
				StdRc::new(RefCell::new(StdRc::new(|acc| acc)));
			let handler_accumulator = StdRc::clone(&accumulator);
			let handled = self.handle_with::<WriterBrand<LogType>, Idx, RMinusWriter>(
				move |op: Writer<'static, LogType, Run<RMinusWriter, CNilBrand, A>>| match op {
					Writer::Tell(log, next, _) => {
						let previous = handler_accumulator.borrow().clone();
						let folder = StdRc::clone(&folder);
						*handler_accumulator.borrow_mut() = StdRc::new(move |initial| {
							let after_current = folder(initial, log.clone());
							previous(after_current)
						});
						next
					}
				},
			);
			handled.map(move |result| {
				let finish = accumulator.borrow().clone();
				(result, finish(initial.clone()))
			})
		}
	}
}

fn run_run_writer_tokens() -> TokenStream {
	quote! {
		/// Interprets one Writer effect by appending emitted logs.
		///
		/// `run_writer()` is the monoidal specialization of
		/// [`Run::fold_writer`]. The accumulator starts at
		/// [`Monoid::empty`] and each `tell(log)` appends that log to
		/// the accumulated value.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters(
			"The log type carried by `WriterBrand`.",
			"The type-level Member-position witness for the Writer effect.",
			"The first-order row brand with the Writer effect removed."
		)]
		#[document_returns("A first-order-only `Run` program returning `(result, log)`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::run::Run,
		/// };
		///
		/// type Row = CoproductBrand<CoyonedaBrand<WriterBrand<String>>, CNilBrand>;
		///
		/// let program: Run<Row, CNilBrand, i32> =
		/// 	Run::<Row, CNilBrand, ()>::tell::<String, _>("first".to_string())
		/// 		.bind(|()| Run::<Row, CNilBrand, ()>::tell::<String, _>("second".to_string()))
		/// 		.bind(|()| Run::<Row, CNilBrand, i32>::pure(7));
		/// let handled: Run<CNilBrand, CNilBrand, (i32, String)> =
		/// 	program.run_writer::<String, _, CNilBrand>();
		/// assert_eq!(handled.extract(), (7, "firstsecond".to_string()));
		/// ```
		#[inline]
		pub fn run_writer<LogType, Idx, RMinusWriter>(
			self
		) -> Run<RMinusWriter, CNilBrand, (A, LogType)>
		where
			LogType: Monoid + Clone + 'static,
			RMinusWriter: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				Run<R, CNilBrand, A>,
			>): Member<
					Coyoneda<'static, WriterBrand<LogType>, Run<R, CNilBrand, A>>,
					Idx,
					Remainder = Apply!(
									<RMinusWriter as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
										'static,
										Run<R, CNilBrand, A>,
									>
								),
				>, {
			self.fold_writer::<LogType, LogType, Idx, RMinusWriter>(
				LogType::empty(),
				LogType::append,
			)
		}
	}
}

fn rcrun_fold_writer_tokens() -> TokenStream {
	quote! {
		/// Interprets one Writer effect with an explicit accumulator function.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters(
			"The log type carried by `WriterBrand`.",
			"The accumulated value type.",
			"The type-level Member-position witness for the Writer effect.",
			"The first-order row brand with the Writer effect removed."
		)]
		#[document_parameters(
			"The initial accumulator value.",
			"The function that incorporates one emitted log into the accumulator."
		)]
		#[document_returns(
			"A first-order-only `RcRun` program returning `(result, accumulated_log)`."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::rc_run::RcRun,
		/// };
		///
		/// type Row = CoproductBrand<RcCoyonedaBrand<WriterBrand<String>>, CNilBrand>;
		///
		/// let program: RcRun<Row, CNilBrand, i32> =
		/// 	RcRun::<Row, CNilBrand, ()>::tell::<String, _>("first".to_string())
		/// 		.bind(|()| RcRun::<Row, CNilBrand, ()>::tell::<String, _>("second".to_string()))
		/// 		.bind(|()| RcRun::<Row, CNilBrand, i32>::pure(7));
		/// let handled: RcRun<CNilBrand, CNilBrand, (i32, Vec<String>)> = program
		/// 	.fold_writer::<String, Vec<String>, _, CNilBrand>(Vec::new(), |mut logs, log| {
		/// 		logs.push(log);
		/// 		logs
		/// 	});
		/// assert_eq!(handled.extract(), (7, vec!["first".to_string(), "second".to_string()]));
		/// ```
		#[inline]
		pub fn fold_writer<LogType, Acc, Idx, RMinusWriter>(
			self,
			initial: Acc,
			folder: impl Fn(Acc, LogType) -> Acc + 'static,
		) -> RcRun<RMinusWriter, CNilBrand, (A, Acc)>
		where
			A: Clone,
			LogType: Clone + 'static,
			Acc: Clone + 'static,
			RMinusWriter: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
			Apply!(<NodeBrand<R, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<NodeBrand<R, CNilBrand>, RcTypeErasedValue>,
			>): Clone,
			Apply!(<NodeBrand<RMinusWriter, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<NodeBrand<RMinusWriter, CNilBrand>, RcTypeErasedValue>,
			>): Clone,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcRun<R, CNilBrand, A>,
			>): Member<
					RcCoyoneda<'static, WriterBrand<LogType>, RcRun<R, CNilBrand, A>>,
					Idx,
					Remainder = Apply!(
									<RMinusWriter as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
										'static,
										RcRun<R, CNilBrand, A>,
									>
								),
				>, {
			let folder = StdRc::new(folder);
			let accumulator: SharedWriterFoldCell<'static, Acc> =
				StdRc::new(RefCell::new(StdRc::new(|acc| acc)));
			let handler_accumulator = StdRc::clone(&accumulator);
			let handled = self.handle_with::<WriterBrand<LogType>, Idx, RMinusWriter>(
				move |op: Writer<'static, LogType, RcRun<RMinusWriter, CNilBrand, A>>| match op {
					Writer::Tell(log, next, _) => {
						let previous = handler_accumulator.borrow().clone();
						let folder = StdRc::clone(&folder);
						*handler_accumulator.borrow_mut() = StdRc::new(move |initial| {
							let after_current = folder(initial, log.clone());
							previous(after_current)
						});
						next
					}
				},
			);
			handled.map(move |result| {
				let finish = accumulator.borrow().clone();
				(result, finish(initial.clone()))
			})
		}
	}
}

fn rcrun_run_writer_tokens() -> TokenStream {
	quote! {
		/// Interprets one Writer effect by appending emitted logs.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters(
			"The log type carried by `WriterBrand`.",
			"The type-level Member-position witness for the Writer effect.",
			"The first-order row brand with the Writer effect removed."
		)]
		#[document_returns("A first-order-only `RcRun` program returning `(result, log)`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::rc_run::RcRun,
		/// };
		///
		/// type Row = CoproductBrand<RcCoyonedaBrand<WriterBrand<String>>, CNilBrand>;
		///
		/// let program: RcRun<Row, CNilBrand, i32> =
		/// 	RcRun::<Row, CNilBrand, ()>::tell::<String, _>("first".to_string())
		/// 		.bind(|()| RcRun::<Row, CNilBrand, ()>::tell::<String, _>("second".to_string()))
		/// 		.bind(|()| RcRun::<Row, CNilBrand, i32>::pure(7));
		/// let handled: RcRun<CNilBrand, CNilBrand, (i32, String)> =
		/// 	program.run_writer::<String, _, CNilBrand>();
		/// assert_eq!(handled.extract(), (7, "firstsecond".to_string()));
		/// ```
		#[inline]
		pub fn run_writer<LogType, Idx, RMinusWriter>(
			self
		) -> RcRun<RMinusWriter, CNilBrand, (A, LogType)>
		where
			A: Clone,
			LogType: Monoid + Clone + 'static,
			RMinusWriter: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
			Apply!(<NodeBrand<R, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<NodeBrand<R, CNilBrand>, RcTypeErasedValue>,
			>): Clone,
			Apply!(<NodeBrand<RMinusWriter, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<NodeBrand<RMinusWriter, CNilBrand>, RcTypeErasedValue>,
			>): Clone,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcRun<R, CNilBrand, A>,
			>): Member<
					RcCoyoneda<'static, WriterBrand<LogType>, RcRun<R, CNilBrand, A>>,
					Idx,
					Remainder = Apply!(
									<RMinusWriter as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
										'static,
										RcRun<R, CNilBrand, A>,
									>
								),
				>, {
			self.fold_writer::<LogType, LogType, Idx, RMinusWriter>(
				LogType::empty(),
				LogType::append,
			)
		}
	}
}

fn arcrun_fold_writer_tokens() -> TokenStream {
	quote! {
		/// Interprets one Writer effect with an explicit accumulator function.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters(
			"The log type carried by `WriterBrand`.",
			"The accumulated value type.",
			"The type-level Member-position witness for the Writer effect.",
			"The first-order row brand with the Writer effect removed."
		)]
		#[document_parameters(
			"The initial accumulator value.",
			"The function that incorporates one emitted log into the accumulator."
		)]
		#[document_returns(
			"A first-order-only `ArcRun` program returning `(result, accumulated_log)`."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::arc_run::ArcRun,
		/// };
		///
		/// type Row = CoproductBrand<ArcCoyonedaBrand<WriterBrand<String>>, CNilBrand>;
		///
		/// let program: ArcRun<Row, CNilBrand, i32> =
		/// 	ArcRun::<Row, CNilBrand, ()>::tell::<String, _>("first".to_string())
		/// 		.bind(|()| ArcRun::<Row, CNilBrand, ()>::tell::<String, _>("second".to_string()))
		/// 		.bind(|()| ArcRun::<Row, CNilBrand, i32>::pure(7));
		/// let handled: ArcRun<CNilBrand, CNilBrand, (i32, Vec<String>)> = program
		/// 	.fold_writer::<String, Vec<String>, _, CNilBrand>(Vec::new(), |mut logs, log| {
		/// 		logs.push(log);
		/// 		logs
		/// 	});
		/// assert_eq!(handled.extract(), (7, vec!["first".to_string(), "second".to_string()]));
		/// ```
		#[inline]
		pub fn fold_writer<LogType, Acc, Idx, RMinusWriter>(
			self,
			initial: Acc,
			folder: impl Fn(Acc, LogType) -> Acc + Send + Sync + 'static,
		) -> ArcRun<RMinusWriter, CNilBrand, (A, Acc)>
		where
			A: Clone + Send + Sync,
			LogType: Clone + Send + Sync + 'static,
			Acc: Clone + Send + Sync + 'static,
			R: Kind_cdc7cd43dac7585f + 'static,
			RMinusWriter: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
			NodeBrand<R, CNilBrand>: SendFunctor,
			NodeBrand<RMinusWriter, CNilBrand>: WrapDrop
				+ Kind_cdc7cd43dac7585f<
					Of<'static, ArcFree<NodeBrand<RMinusWriter, CNilBrand>, ArcTypeErasedValue>>:
						Send + Sync,
				> + SendFunctor,
			Apply!(<NodeBrand<R, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<NodeBrand<R, CNilBrand>, ArcTypeErasedValue>,
			>): Clone,
			Apply!(<NodeBrand<RMinusWriter, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<NodeBrand<RMinusWriter, CNilBrand>, ArcTypeErasedValue>,
			>): Clone,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcRun<R, CNilBrand, A>,
			>): Member<
					ArcCoyoneda<'static, WriterBrand<LogType>, ArcRun<R, CNilBrand, A>>,
					Idx,
					Remainder = Apply!(
									<RMinusWriter as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
										'static,
										ArcRun<R, CNilBrand, A>,
									>
								),
				>, {
			let folder = StdArc::new(folder);
			let accumulator: SendSharedWriterFoldCell<'static, Acc> =
				StdArc::new(Mutex::new(StdArc::new(|acc| acc)));
			let handler_accumulator = StdArc::clone(&accumulator);
			let handled = self.handle_with::<WriterBrand<LogType>, Idx, RMinusWriter>(
				move |op: Writer<'static, LogType, ArcRun<RMinusWriter, CNilBrand, A>>| match op {
					Writer::Tell(log, next, _) => {
						let previous = {
							let guard = match handler_accumulator.lock() {
								Ok(guard) => guard,
								Err(poisoned) => poisoned.into_inner(),
							};
							guard.clone()
						};
						let folder = StdArc::clone(&folder);
						let mut guard = match handler_accumulator.lock() {
							Ok(guard) => guard,
							Err(poisoned) => poisoned.into_inner(),
						};
						*guard = StdArc::new(move |initial| {
							let after_current = folder(initial, log.clone());
							previous(after_current)
						});
						next
					}
				},
			);
			handled.map(move |result| {
				let finish = {
					let guard = match accumulator.lock() {
						Ok(guard) => guard,
						Err(poisoned) => poisoned.into_inner(),
					};
					guard.clone()
				};
				(result, finish(initial.clone()))
			})
		}
	}
}

fn arcrun_run_writer_tokens() -> TokenStream {
	quote! {
		/// Interprets one Writer effect by appending emitted logs.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters(
			"The log type carried by `WriterBrand`.",
			"The type-level Member-position witness for the Writer effect.",
			"The first-order row brand with the Writer effect removed."
		)]
		#[document_returns("A first-order-only `ArcRun` program returning `(result, log)`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::arc_run::ArcRun,
		/// };
		///
		/// type Row = CoproductBrand<ArcCoyonedaBrand<WriterBrand<String>>, CNilBrand>;
		///
		/// let program: ArcRun<Row, CNilBrand, i32> =
		/// 	ArcRun::<Row, CNilBrand, ()>::tell::<String, _>("first".to_string())
		/// 		.bind(|()| ArcRun::<Row, CNilBrand, ()>::tell::<String, _>("second".to_string()))
		/// 		.bind(|()| ArcRun::<Row, CNilBrand, i32>::pure(7));
		/// let handled: ArcRun<CNilBrand, CNilBrand, (i32, String)> =
		/// 	program.run_writer::<String, _, CNilBrand>();
		/// assert_eq!(handled.extract(), (7, "firstsecond".to_string()));
		/// ```
		#[inline]
		pub fn run_writer<LogType, Idx, RMinusWriter>(
			self
		) -> ArcRun<RMinusWriter, CNilBrand, (A, LogType)>
		where
			A: Clone + Send + Sync,
			LogType: Monoid + Clone + Send + Sync + 'static,
			R: Kind_cdc7cd43dac7585f + 'static,
			RMinusWriter: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
			NodeBrand<R, CNilBrand>: SendFunctor,
			NodeBrand<RMinusWriter, CNilBrand>: WrapDrop
				+ Kind_cdc7cd43dac7585f<
					Of<'static, ArcFree<NodeBrand<RMinusWriter, CNilBrand>, ArcTypeErasedValue>>:
						Send + Sync,
				> + SendFunctor,
			Apply!(<NodeBrand<R, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<NodeBrand<R, CNilBrand>, ArcTypeErasedValue>,
			>): Clone,
			Apply!(<NodeBrand<RMinusWriter, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<NodeBrand<RMinusWriter, CNilBrand>, ArcTypeErasedValue>,
			>): Clone,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcRun<R, CNilBrand, A>,
			>): Member<
					ArcCoyoneda<'static, WriterBrand<LogType>, ArcRun<R, CNilBrand, A>>,
					Idx,
					Remainder = Apply!(
									<RMinusWriter as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
										'static,
										ArcRun<R, CNilBrand, A>,
									>
								),
				>, {
			self.fold_writer::<LogType, LogType, Idx, RMinusWriter>(
				LogType::empty(),
				LogType::append,
			)
		}
	}
}

fn run_explicit_fold_writer_tokens() -> TokenStream {
	quote! {
		/// Interprets one Writer effect with an explicit accumulator function.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters(
			"The log type carried by `WriterBrand`.",
			"The accumulated value type.",
			"The type-level Member-position witness for the Writer effect.",
			"The first-order row brand with the Writer effect removed."
		)]
		#[document_parameters(
			"The initial accumulator value.",
			"The function that incorporates one emitted log into the accumulator."
		)]
		#[document_returns(
			"A first-order-only `RunExplicit` program returning `(result, accumulated_log)`."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::run_explicit::RunExplicit,
		/// };
		///
		/// type Row = CoproductBrand<CoyonedaBrand<WriterBrand<String>>, CNilBrand>;
		///
		/// let program: RunExplicit<'static, Row, CNilBrand, i32> =
		/// 	RunExplicit::<'static, Row, CNilBrand, ()>::tell::<String, _>("first".to_string())
		/// 		.bind(|()| {
		/// 			RunExplicit::<'static, Row, CNilBrand, ()>::tell::<String, _>("second".to_string())
		/// 		})
		/// 		.bind(|()| RunExplicit::<'static, Row, CNilBrand, i32>::pure(7));
		/// let handled: RunExplicit<'static, CNilBrand, CNilBrand, (i32, Vec<String>)> = program
		/// 	.fold_writer::<String, Vec<String>, _, CNilBrand>(Vec::new(), |mut logs, log| {
		/// 		logs.push(log);
		/// 		logs
		/// 	});
		/// assert_eq!(handled.extract(), (7, vec!["first".to_string(), "second".to_string()]));
		/// ```
		#[inline]
		pub fn fold_writer<LogType, Acc, Idx, RMinusWriter>(
			self,
			initial: Acc,
			folder: impl Fn(Acc, LogType) -> Acc + 'a,
		) -> RunExplicit<'a, RMinusWriter, CNilBrand, (A, Acc)>
		where
			LogType: Clone + 'static,
			Acc: Clone + 'a,
			RMinusWriter: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RunExplicit<'a, R, CNilBrand, A>,
			>): Member<
					Coyoneda<'a, WriterBrand<LogType>, RunExplicit<'a, R, CNilBrand, A>>,
					Idx,
					Remainder = Apply!(
									<RMinusWriter as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
										'a,
										RunExplicit<'a, R, CNilBrand, A>,
									>
								),
				>, {
			let folder = StdRc::new(folder);
			let accumulator: SharedWriterFoldCell<'a, Acc> =
				StdRc::new(RefCell::new(StdRc::new(|acc| acc)));
			let handler_accumulator = StdRc::clone(&accumulator);
			let handled = self.handle_with::<WriterBrand<LogType>, Idx, RMinusWriter>(
				move |op: Writer<'a, LogType, RunExplicit<'a, RMinusWriter, CNilBrand, A>>| match op
				{
					Writer::Tell(log, next, _) => {
						let previous = handler_accumulator.borrow().clone();
						let folder = StdRc::clone(&folder);
						*handler_accumulator.borrow_mut() = StdRc::new(move |initial| {
							let after_current = folder(initial, log.clone());
							previous(after_current)
						});
						next
					}
				},
			);
			handled.map(move |result| {
				let finish = accumulator.borrow().clone();
				(result, finish(initial.clone()))
			})
		}
	}
}

fn run_explicit_run_writer_tokens() -> TokenStream {
	quote! {
		/// Interprets one Writer effect by appending emitted logs.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters(
			"The log type carried by `WriterBrand`.",
			"The type-level Member-position witness for the Writer effect.",
			"The first-order row brand with the Writer effect removed."
		)]
		#[document_returns("A first-order-only `RunExplicit` program returning `(result, log)`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::run_explicit::RunExplicit,
		/// };
		///
		/// type Row = CoproductBrand<CoyonedaBrand<WriterBrand<String>>, CNilBrand>;
		///
		/// let program: RunExplicit<'static, Row, CNilBrand, i32> =
		/// 	RunExplicit::<'static, Row, CNilBrand, ()>::tell::<String, _>("first".to_string())
		/// 		.bind(|()| {
		/// 			RunExplicit::<'static, Row, CNilBrand, ()>::tell::<String, _>("second".to_string())
		/// 		})
		/// 		.bind(|()| RunExplicit::<'static, Row, CNilBrand, i32>::pure(7));
		/// let handled: RunExplicit<'static, CNilBrand, CNilBrand, (i32, String)> =
		/// 	program.run_writer::<String, _, CNilBrand>();
		/// assert_eq!(handled.extract(), (7, "firstsecond".to_string()));
		/// ```
		#[inline]
		pub fn run_writer<LogType, Idx, RMinusWriter>(
			self
		) -> RunExplicit<'a, RMinusWriter, CNilBrand, (A, LogType)>
		where
			LogType: Monoid + Clone + 'static,
			RMinusWriter: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RunExplicit<'a, R, CNilBrand, A>,
			>): Member<
					Coyoneda<'a, WriterBrand<LogType>, RunExplicit<'a, R, CNilBrand, A>>,
					Idx,
					Remainder = Apply!(
									<RMinusWriter as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
										'a,
										RunExplicit<'a, R, CNilBrand, A>,
									>
								),
				>, {
			self.fold_writer::<LogType, LogType, Idx, RMinusWriter>(
				LogType::empty(),
				LogType::append,
			)
		}
	}
}

fn rcrun_explicit_fold_writer_tokens() -> TokenStream {
	quote! {
		/// Interprets one Writer effect with an explicit accumulator function.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters(
			"The log type carried by `WriterBrand`.",
			"The accumulated value type.",
			"The type-level Member-position witness for the Writer effect.",
			"The first-order row brand with the Writer effect removed."
		)]
		#[document_parameters(
			"The initial accumulator value.",
			"The function that incorporates one emitted log into the accumulator."
		)]
		#[document_returns(
			"A first-order-only `RcRunExplicit` program returning `(result, accumulated_log)`."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::rc_run_explicit::RcRunExplicit,
		/// };
		///
		/// type Row = CoproductBrand<RcCoyonedaBrand<WriterBrand<String>>, CNilBrand>;
		///
		/// let program: RcRunExplicit<'static, Row, CNilBrand, i32> =
		/// 	RcRunExplicit::<'static, Row, CNilBrand, ()>::tell::<String, _>("first".to_string())
		/// 		.bind(|()| {
		/// 			RcRunExplicit::<'static, Row, CNilBrand, ()>::tell::<String, _>(
		/// 				"second".to_string(),
		/// 			)
		/// 		})
		/// 		.bind(|()| RcRunExplicit::<'static, Row, CNilBrand, i32>::pure(7));
		/// let handled: RcRunExplicit<'static, CNilBrand, CNilBrand, (i32, Vec<String>)> =
		/// 	program.fold_writer::<String, Vec<String>, _, CNilBrand>(Vec::new(), |mut logs, log| {
		/// 		logs.push(log);
		/// 		logs
		/// 	});
		/// assert_eq!(handled.extract(), (7, vec!["first".to_string(), "second".to_string()]));
		/// ```
		#[inline]
		pub fn fold_writer<LogType, Acc, Idx, RMinusWriter>(
			self,
			initial: Acc,
			folder: impl Fn(Acc, LogType) -> Acc + 'a,
		) -> RcRunExplicit<'a, RMinusWriter, CNilBrand, (A, Acc)>
		where
			A: Clone,
			LogType: Clone + 'static,
			Acc: Clone + 'a,
			RMinusWriter: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
			Apply!(<NodeBrand<R, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, CNilBrand>, A>,
			>): Clone,
			Apply!(<NodeBrand<RMinusWriter, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<RMinusWriter, CNilBrand>, A>,
			>): Clone,
			Apply!(<NodeBrand<RMinusWriter, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<RMinusWriter, CNilBrand>, (A, Acc)>,
			>): Clone,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcRunExplicit<'a, R, CNilBrand, A>,
			>): Member<
					RcCoyoneda<'a, WriterBrand<LogType>, RcRunExplicit<'a, R, CNilBrand, A>>,
					Idx,
					Remainder = Apply!(
									<RMinusWriter as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
										'a,
										RcRunExplicit<'a, R, CNilBrand, A>,
									>
								),
				>, {
			let folder = StdRc::new(folder);
			let accumulator: SharedWriterFoldCell<'a, Acc> =
				StdRc::new(RefCell::new(StdRc::new(|acc| acc)));
			let handler_accumulator = StdRc::clone(&accumulator);
			let handled = self.handle_with::<WriterBrand<LogType>, Idx, RMinusWriter>(
				move |op: Writer<'a, LogType, RcRunExplicit<'a, RMinusWriter, CNilBrand, A>>| {
					match op {
						Writer::Tell(log, next, _) => {
							let previous = handler_accumulator.borrow().clone();
							let folder = StdRc::clone(&folder);
							*handler_accumulator.borrow_mut() = StdRc::new(move |initial| {
								let after_current = folder(initial, log.clone());
								previous(after_current)
							});
							next
						}
					}
				},
			);
			handled.map(move |result| {
				let finish = accumulator.borrow().clone();
				(result, finish(initial.clone()))
			})
		}
	}
}

fn rcrun_explicit_run_writer_tokens() -> TokenStream {
	quote! {
		/// Interprets one Writer effect by appending emitted logs.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters(
			"The log type carried by `WriterBrand`.",
			"The type-level Member-position witness for the Writer effect.",
			"The first-order row brand with the Writer effect removed."
		)]
		#[document_returns("A first-order-only `RcRunExplicit` program returning `(result, log)`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::rc_run_explicit::RcRunExplicit,
		/// };
		///
		/// type Row = CoproductBrand<RcCoyonedaBrand<WriterBrand<String>>, CNilBrand>;
		///
		/// let program: RcRunExplicit<'static, Row, CNilBrand, i32> =
		/// 	RcRunExplicit::<'static, Row, CNilBrand, ()>::tell::<String, _>("first".to_string())
		/// 		.bind(|()| {
		/// 			RcRunExplicit::<'static, Row, CNilBrand, ()>::tell::<String, _>(
		/// 				"second".to_string(),
		/// 			)
		/// 		})
		/// 		.bind(|()| RcRunExplicit::<'static, Row, CNilBrand, i32>::pure(7));
		/// let handled: RcRunExplicit<'static, CNilBrand, CNilBrand, (i32, String)> =
		/// 	program.run_writer::<String, _, CNilBrand>();
		/// assert_eq!(handled.extract(), (7, "firstsecond".to_string()));
		/// ```
		#[inline]
		pub fn run_writer<LogType, Idx, RMinusWriter>(
			self
		) -> RcRunExplicit<'a, RMinusWriter, CNilBrand, (A, LogType)>
		where
			A: Clone,
			LogType: Monoid + Clone + 'static,
			RMinusWriter: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
			Apply!(<NodeBrand<R, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, CNilBrand>, A>,
			>): Clone,
			Apply!(<NodeBrand<RMinusWriter, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<RMinusWriter, CNilBrand>, A>,
			>): Clone,
			Apply!(<NodeBrand<RMinusWriter, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<RMinusWriter, CNilBrand>, (A, LogType)>,
			>): Clone,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcRunExplicit<'a, R, CNilBrand, A>,
			>): Member<
					RcCoyoneda<'a, WriterBrand<LogType>, RcRunExplicit<'a, R, CNilBrand, A>>,
					Idx,
					Remainder = Apply!(
									<RMinusWriter as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
										'a,
										RcRunExplicit<'a, R, CNilBrand, A>,
									>
								),
				>, {
			self.fold_writer::<LogType, LogType, Idx, RMinusWriter>(
				LogType::empty(),
				LogType::append,
			)
		}
	}
}

fn arcrun_explicit_fold_writer_tokens() -> TokenStream {
	quote! {
		/// Interprets one Writer effect with an explicit accumulator function.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters(
			"The log type carried by `WriterBrand`.",
			"The accumulated value type.",
			"The type-level Member-position witness for the Writer effect.",
			"The first-order row brand with the Writer effect removed."
		)]
		#[document_parameters(
			"The initial accumulator value.",
			"The function that incorporates one emitted log into the accumulator."
		)]
		#[document_returns(
			"A first-order-only `ArcRunExplicit` program returning `(result, accumulated_log)`."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::arc_run_explicit::ArcRunExplicit,
		/// };
		///
		/// type Row = CoproductBrand<ArcCoyonedaBrand<WriterBrand<String>>, CNilBrand>;
		///
		/// let program: ArcRunExplicit<'static, Row, CNilBrand, i32> =
		/// 	ArcRunExplicit::<'static, Row, CNilBrand, ()>::tell::<String, _>("first".to_string())
		/// 		.bind(|()| {
		/// 			ArcRunExplicit::<'static, Row, CNilBrand, ()>::tell::<String, _>(
		/// 				"second".to_string(),
		/// 			)
		/// 		})
		/// 		.bind(|()| ArcRunExplicit::<'static, Row, CNilBrand, i32>::pure(7));
		/// let handled: ArcRunExplicit<'static, CNilBrand, CNilBrand, (i32, Vec<String>)> =
		/// 	program.fold_writer::<String, Vec<String>, _, CNilBrand>(Vec::new(), |mut logs, log| {
		/// 		logs.push(log);
		/// 		logs
		/// 	});
		/// assert_eq!(handled.extract(), (7, vec!["first".to_string(), "second".to_string()]));
		/// ```
		#[inline]
		pub fn fold_writer<LogType, Acc, Idx, RMinusWriter>(
			self,
			initial: Acc,
			folder: impl Fn(Acc, LogType) -> Acc + Send + Sync + 'a,
		) -> ArcRunExplicit<'a, RMinusWriter, CNilBrand, (A, Acc)>
		where
			A: Clone + Send + Sync,
			LogType: Clone + Send + Sync + 'static,
			Acc: Clone + Send + Sync + 'a,
			RMinusWriter: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
			NodeBrand<R, CNilBrand>: SendFunctor,
			NodeBrand<RMinusWriter, CNilBrand>: SendFunctor,
			Apply!(<NodeBrand<R, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, CNilBrand>, A>,
			>): Clone + Send + Sync,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, CNilBrand>, A>,
			>): Send + Sync,
			Apply!(<CNilBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, CNilBrand>, A>,
			>): Send + Sync,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, R, CNilBrand, A>,
			>): Send + Sync,
			Apply!(<CNilBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, R, CNilBrand, A>,
			>): Send + Sync,
			Apply!(<NodeBrand<RMinusWriter, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<RMinusWriter, CNilBrand>, A>,
			>): Clone + Send + Sync,
			Apply!(<NodeBrand<RMinusWriter, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<RMinusWriter, CNilBrand>, (A, Acc)>,
			>): Clone + Send + Sync,
			Apply!(<RMinusWriter as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<RMinusWriter, CNilBrand>, A>,
			>): Send + Sync,
			Apply!(<RMinusWriter as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<RMinusWriter, CNilBrand>, (A, Acc)>,
			>): Send + Sync,
			Apply!(<RMinusWriter as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, RMinusWriter, CNilBrand, A>,
			>): Send + Sync,
			Apply!(<CNilBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<RMinusWriter, CNilBrand>, A>,
			>): Send + Sync,
			Apply!(<CNilBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<RMinusWriter, CNilBrand>, (A, Acc)>,
			>): Send + Sync,
			Apply!(<CNilBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, RMinusWriter, CNilBrand, A>,
			>): Send + Sync,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, R, CNilBrand, A>,
			>): Member<
					ArcCoyoneda<'a, WriterBrand<LogType>, ArcRunExplicit<'a, R, CNilBrand, A>>,
					Idx,
					Remainder = Apply!(
									<RMinusWriter as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
										'a,
										ArcRunExplicit<'a, R, CNilBrand, A>,
									>
								),
				>, {
			let folder = StdArc::new(folder);
			let accumulator: SendSharedWriterFoldCell<'a, Acc> =
				StdArc::new(Mutex::new(StdArc::new(|acc| acc)));
			let handler_accumulator = StdArc::clone(&accumulator);
			let handled = self.handle_with::<WriterBrand<LogType>, Idx, RMinusWriter>(
				move |op: Writer<'a, LogType, ArcRunExplicit<'a, RMinusWriter, CNilBrand, A>>| {
					match op {
						Writer::Tell(log, next, _) => {
							let previous = {
								let guard = match handler_accumulator.lock() {
									Ok(guard) => guard,
									Err(poisoned) => poisoned.into_inner(),
								};
								guard.clone()
							};
							let folder = StdArc::clone(&folder);
							let mut guard = match handler_accumulator.lock() {
								Ok(guard) => guard,
								Err(poisoned) => poisoned.into_inner(),
							};
							*guard = StdArc::new(move |initial| {
								let after_current = folder(initial, log.clone());
								previous(after_current)
							});
							next
						}
					}
				},
			);
			handled.map(move |result| {
				let finish = {
					let guard = match accumulator.lock() {
						Ok(guard) => guard,
						Err(poisoned) => poisoned.into_inner(),
					};
					guard.clone()
				};
				(result, finish(initial.clone()))
			})
		}
	}
}

fn arcrun_explicit_run_writer_tokens() -> TokenStream {
	quote! {
		/// Interprets one Writer effect by appending emitted logs.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters(
			"The log type carried by `WriterBrand`.",
			"The type-level Member-position witness for the Writer effect.",
			"The first-order row brand with the Writer effect removed."
		)]
		#[document_returns(
			"A first-order-only `ArcRunExplicit` program returning `(result, log)`."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::arc_run_explicit::ArcRunExplicit,
		/// };
		///
		/// type Row = CoproductBrand<ArcCoyonedaBrand<WriterBrand<String>>, CNilBrand>;
		///
		/// let program: ArcRunExplicit<'static, Row, CNilBrand, i32> =
		/// 	ArcRunExplicit::<'static, Row, CNilBrand, ()>::tell::<String, _>("first".to_string())
		/// 		.bind(|()| {
		/// 			ArcRunExplicit::<'static, Row, CNilBrand, ()>::tell::<String, _>(
		/// 				"second".to_string(),
		/// 			)
		/// 		})
		/// 		.bind(|()| ArcRunExplicit::<'static, Row, CNilBrand, i32>::pure(7));
		/// let handled: ArcRunExplicit<'static, CNilBrand, CNilBrand, (i32, String)> =
		/// 	program.run_writer::<String, _, CNilBrand>();
		/// assert_eq!(handled.extract(), (7, "firstsecond".to_string()));
		/// ```
		#[inline]
		pub fn run_writer<LogType, Idx, RMinusWriter>(
			self
		) -> ArcRunExplicit<'a, RMinusWriter, CNilBrand, (A, LogType)>
		where
			A: Clone + Send + Sync,
			LogType: Monoid + Clone + Send + Sync + 'static,
			RMinusWriter: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
			NodeBrand<R, CNilBrand>: SendFunctor,
			NodeBrand<RMinusWriter, CNilBrand>: SendFunctor,
			Apply!(<NodeBrand<R, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, CNilBrand>, A>,
			>): Clone + Send + Sync,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, CNilBrand>, A>,
			>): Send + Sync,
			Apply!(<CNilBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, CNilBrand>, A>,
			>): Send + Sync,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, R, CNilBrand, A>,
			>): Send + Sync,
			Apply!(<CNilBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, R, CNilBrand, A>,
			>): Send + Sync,
			Apply!(<NodeBrand<RMinusWriter, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<RMinusWriter, CNilBrand>, A>,
			>): Clone + Send + Sync,
			Apply!(<NodeBrand<RMinusWriter, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<RMinusWriter, CNilBrand>, (A, LogType)>,
			>): Clone + Send + Sync,
			Apply!(<RMinusWriter as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<RMinusWriter, CNilBrand>, A>,
			>): Send + Sync,
			Apply!(<RMinusWriter as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<RMinusWriter, CNilBrand>, (A, LogType)>,
			>): Send + Sync,
			Apply!(<RMinusWriter as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, RMinusWriter, CNilBrand, A>,
			>): Send + Sync,
			Apply!(<CNilBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<RMinusWriter, CNilBrand>, A>,
			>): Send + Sync,
			Apply!(<CNilBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<RMinusWriter, CNilBrand>, (A, LogType)>,
			>): Send + Sync,
			Apply!(<CNilBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, RMinusWriter, CNilBrand, A>,
			>): Send + Sync,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, R, CNilBrand, A>,
			>): Member<
					ArcCoyoneda<'a, WriterBrand<LogType>, ArcRunExplicit<'a, R, CNilBrand, A>>,
					Idx,
					Remainder = Apply!(
									<RMinusWriter as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
										'a,
										ArcRunExplicit<'a, R, CNilBrand, A>,
									>
								),
				>, {
			self.fold_writer::<LogType, LogType, Idx, RMinusWriter>(
				LogType::empty(),
				LogType::append,
			)
		}
	}
}
