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

pub(super) fn reader_wrapper_impl_items_from_descriptor(
	wrapper: WrapperName,
	method: RunWrapperMethod,
) -> Option<syn::Result<Vec<ImplItem>>> {
	generator_descriptors::method_spec(EffectName::Reader, method)?;
	generator_descriptors::wrapper_spec(wrapper)?;

	let tokens = match (wrapper, method) {
		(WrapperName::Run, RunWrapperMethod::Ask) => run_reader_ask_tokens(),
		(WrapperName::Run, RunWrapperMethod::Asks) => run_reader_asks_tokens(),
		(WrapperName::Run, RunWrapperMethod::RunReader) => run_reader_run_reader_tokens(),
		(WrapperName::RcRun, RunWrapperMethod::Ask) => rcrun_reader_ask_tokens(),
		(WrapperName::RcRun, RunWrapperMethod::Asks) => rcrun_reader_asks_tokens(),
		(WrapperName::RcRun, RunWrapperMethod::RunReader) => rcrun_reader_run_reader_tokens(),
		(WrapperName::ArcRun, RunWrapperMethod::Ask) => arcrun_reader_ask_tokens(),
		(WrapperName::ArcRun, RunWrapperMethod::Asks) => arcrun_reader_asks_tokens(),
		(WrapperName::ArcRun, RunWrapperMethod::RunReader) => arcrun_reader_run_reader_tokens(),
		(WrapperName::RunExplicit, RunWrapperMethod::Ask) => run_explicit_reader_ask_tokens(),
		(WrapperName::RunExplicit, RunWrapperMethod::Asks) => run_explicit_reader_asks_tokens(),
		(WrapperName::RunExplicit, RunWrapperMethod::RunReader) =>
			run_explicit_reader_run_reader_tokens(),
		(WrapperName::RcRunExplicit, RunWrapperMethod::Ask) => rcrun_explicit_reader_ask_tokens(),
		(WrapperName::RcRunExplicit, RunWrapperMethod::Asks) => rcrun_explicit_reader_asks_tokens(),
		(WrapperName::RcRunExplicit, RunWrapperMethod::RunReader) =>
			rcrun_explicit_reader_run_reader_tokens(),
		(WrapperName::ArcRunExplicit, RunWrapperMethod::Ask) => arcrun_explicit_reader_ask_tokens(),
		(WrapperName::ArcRunExplicit, RunWrapperMethod::Asks) =>
			arcrun_explicit_reader_asks_tokens(),
		(WrapperName::ArcRunExplicit, RunWrapperMethod::RunReader) =>
			arcrun_explicit_reader_run_reader_tokens(),
		_ => return None,
	};

	Some(impl_items_from_tokens(tokens))
}

fn run_reader_ask_tokens() -> TokenStream {
	quote! {
		/// Lifts an `Ask` reader effect into the Run program. Direct
		/// analog of PureScript Run's `ask`. The program reads the
		/// immutable environment and returns it as the result type
		/// `A` (the environment type and the result type coincide
		/// for `ask`).
		///
		/// `Idx` is the type-level position witness identifying where
		/// `BoxReaderBrand<BoxBrand, A>` lives in the row `R`. Rust
		/// infers `Idx` whenever the effect appears unambiguously in
		/// the row.
		#[__document_module_generated]
		#[document_signature]
		///
		#[document_type_parameters("The type-level Member-position witness (typically inferred).")]
		///
		#[document_returns("A `Run` program suspended at the lifted `Ask` effect.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::run::Run,
		/// };
		///
		/// type FirstRow = CoproductBrand<CoyonedaBrand<BoxReaderBrand<BoxBrand, i32>>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let prog: Run<FirstRow, Scoped, i32> = Run::ask();
		/// let handled: Run<CNilBrand, CNilBrand, i32> = prog.run_reader::<i32, _, CNilBrand>(41);
		/// assert_eq!(handled.extract(), 41);
		/// ```
		#[inline]
		pub fn ask<Idx>() -> Self
		where
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, A>):
				crate::types::effects::member::Member<
						crate::types::Coyoneda<
							'static,
							crate::brands::BoxReaderBrand<crate::brands::BoxBrand, A>,
							A,
						>,
						Idx,
					>, {
			let effect: crate::types::effects::reader::BoxReader<'static, crate::brands::BoxBrand, A, A> =
				crate::types::effects::reader::BoxReader::Ask(
					<crate::brands::BoxBrand as crate::classes::ToDynFnOnce>::new(|e: A| e),
				);
			Self::lift::<crate::brands::BoxReaderBrand<crate::brands::BoxBrand, A>, Idx>(effect)
		}
	}
}

fn run_reader_asks_tokens() -> TokenStream {
	quote! {
		/// Reads the Reader environment and maps it immediately.
		///
		/// `asks(f)` is the Reader-specific convenience form of
		/// `ask().map(f)`. It keeps the effect row unchanged while
		/// letting the program return a projection of the environment.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters(
			"The Reader environment type.",
			"The type-level Member-position witness for the Reader effect."
		)]
		#[document_parameters("The projection to apply to the environment.")]
		#[document_returns("A `Run` program that asks for the environment and returns `f(env)`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::run::Run,
		/// };
		///
		/// type Row = CoproductBrand<CoyonedaBrand<BoxReaderBrand<BoxBrand, i32>>, CNilBrand>;
		///
		/// let program: Run<Row, CNilBrand, String> = Run::asks::<i32, _>(|env| format!("env={env}"));
		/// let handled: Run<CNilBrand, CNilBrand, String> = program.run_reader::<i32, _, CNilBrand>(7);
		/// assert_eq!(handled.extract(), "env=7");
		/// ```
		#[inline]
		pub fn asks<E, Idx>(f: impl FnOnce(E) -> A + 'static) -> Self
		where
			E: 'static,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, E>):
				Member<Coyoneda<'static, BoxReaderBrand<BoxBrand, E>, E>, Idx>, {
			Run::<R, S, E>::ask::<Idx>().map(f)
		}
	}
}

fn run_reader_run_reader_tokens() -> TokenStream {
	quote! {
		/// Interprets one Reader effect by supplying a fixed environment.
		///
		/// Each `Ask` receives a clone of `env`, so this helper is valid
		/// for programs with multiple Reader asks. The method narrows the
		/// first-order row by removing the selected Reader brand.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters(
			"The Reader environment type.",
			"The type-level Member-position witness for the Reader effect.",
			"The first-order row brand with the Reader effect removed."
		)]
		#[document_parameters("The environment value supplied to every Reader ask.")]
		#[document_returns("A first-order-only `Run` program with the Reader effect removed.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::run::Run,
		/// };
		///
		/// type Row = CoproductBrand<CoyonedaBrand<BoxReaderBrand<BoxBrand, i32>>, CNilBrand>;
		///
		/// let program: Run<Row, CNilBrand, i32> = Run::<Row, CNilBrand, i32>::ask().map(|env| env + 1);
		/// let handled: Run<CNilBrand, CNilBrand, i32> = program.run_reader::<i32, _, CNilBrand>(41);
		/// assert_eq!(handled.extract(), 42);
		/// ```
		#[inline]
		pub fn run_reader<E, Idx, RMinusReader>(
			self,
			env: E,
		) -> Run<RMinusReader, CNilBrand, A>
		where
			E: Clone + 'static,
			RMinusReader: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, Run<R, CNilBrand, A>>): Member<
					Coyoneda<'static, BoxReaderBrand<BoxBrand, E>, Run<R, CNilBrand, A>>,
					Idx,
					Remainder = Apply!(
									<RMinusReader as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
										'static,
										Run<R, CNilBrand, A>,
									>
								),
				>, {
			self.handle_with::<BoxReaderBrand<BoxBrand, E>, Idx, RMinusReader>(
				move |op: BoxReader<'static, BoxBrand, E, Run<RMinusReader, CNilBrand, A>>| match op {
					BoxReader::Ask(k) => k(env.clone()),
				},
			)
		}
	}
}

fn rcrun_reader_ask_tokens() -> TokenStream {
	quote! {
		/// Lifts an `Ask` reader effect into the `RcRun` program.
		/// Mirrors [`Run::ask`](crate::types::effects::run::Run::ask);
		/// see that method for cross-wrapper semantics. Differences for
		/// `RcRun`: the [`RcCoyoneda`] variant pairs with the `Rc`-shared
		/// substrate. Threads
		/// [`RcBrand`](crate::brands::RcBrand) as the pointer kind.
		///
		/// `Idx` is the type-level position witness identifying where
		/// `ReaderBrand<RcBrand, A>` lives in the row `R`. Rust infers
		/// `Idx` whenever the effect appears unambiguously in the row.
		#[__document_module_generated]
		#[document_signature]
		///
		#[document_type_parameters("The type-level Member-position witness (typically inferred).")]
		///
		#[document_returns("An `RcRun` program suspended at the lifted `Ask` effect.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::rc_run::RcRun,
		/// };
		///
		/// type FirstRow = CoproductBrand<RcCoyonedaBrand<ReaderBrand<RcBrand, i32>>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let prog: RcRun<FirstRow, Scoped, i32> = RcRun::ask();
		/// let handled: RcRun<CNilBrand, CNilBrand, i32> = prog.run_reader::<i32, _, CNilBrand>(41);
		/// assert_eq!(handled.extract(), 41);
		/// ```
		#[inline]
		pub fn ask<Idx>() -> Self
		where
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, A>):
				Member<RcCoyoneda<'static, crate::brands::ReaderBrand<RcBrand, A>, A>, Idx>,
			Apply!(<NodeBrand<R, ScopedRow> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<NodeBrand<R, ScopedRow>, crate::types::rc_free::RcTypeErasedValue>,
			>): Clone, {
			let effect: crate::types::effects::reader::Reader<'static, RcBrand, A, A> =
				crate::types::effects::reader::Reader::Ask(<RcBrand as crate::classes::ToDynCloneFn>::new(
					|e: A| e,
				));
			Self::lift::<crate::brands::ReaderBrand<RcBrand, A>, Idx>(effect)
		}
	}
}

fn rcrun_reader_asks_tokens() -> TokenStream {
	quote! {
		/// Reads the Reader environment and maps it immediately.
		///
		/// `asks(f)` is the Reader-specific convenience form of
		/// `ask().map(f)` for `RcRun`.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters(
			"The Reader environment type.",
			"The type-level Member-position witness for the Reader effect."
		)]
		#[document_parameters("The projection to apply to the environment.")]
		#[document_returns("An `RcRun` program that asks for the environment and returns `f(env)`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::rc_run::RcRun,
		/// };
		///
		/// type Row = CoproductBrand<RcCoyonedaBrand<ReaderBrand<RcBrand, i32>>, CNilBrand>;
		///
		/// let program: RcRun<Row, CNilBrand, String> = RcRun::asks::<i32, _>(|env| format!("env={env}"));
		/// let handled: RcRun<CNilBrand, CNilBrand, String> = program.run_reader::<i32, _, CNilBrand>(7);
		/// assert_eq!(handled.extract(), "env=7");
		/// ```
		#[inline]
		pub fn asks<E, Idx>(f: impl Fn(E) -> A + 'static) -> Self
		where
			E: Clone + 'static,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, E>):
				Member<RcCoyoneda<'static, ReaderBrand<RcBrand, E>, E>, Idx>,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<NodeBrand<R, S>, RcTypeErasedValue>,
			>): Clone, {
			RcRun::<R, S, E>::ask::<Idx>().map(f)
		}
	}
}

fn rcrun_reader_run_reader_tokens() -> TokenStream {
	quote! {
		/// Interprets one Reader effect by supplying a fixed environment.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters(
			"The Reader environment type.",
			"The type-level Member-position witness for the Reader effect.",
			"The first-order row brand with the Reader effect removed."
		)]
		#[document_parameters("The environment value supplied to every Reader ask.")]
		#[document_returns("A first-order-only `RcRun` program with the Reader effect removed.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::rc_run::RcRun,
		/// };
		///
		/// type Row = CoproductBrand<RcCoyonedaBrand<ReaderBrand<RcBrand, i32>>, CNilBrand>;
		///
		/// let program: RcRun<Row, CNilBrand, i32> =
		/// 	RcRun::<Row, CNilBrand, i32>::ask().map(|env| env + 1);
		/// let handled: RcRun<CNilBrand, CNilBrand, i32> = program.run_reader::<i32, _, CNilBrand>(41);
		/// assert_eq!(handled.extract(), 42);
		/// ```
		#[inline]
		pub fn run_reader<E, Idx, RMinusReader>(
			self,
			env: E,
		) -> RcRun<RMinusReader, CNilBrand, A>
		where
			A: Clone,
			E: Clone + 'static,
			RMinusReader: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
			Apply!(<NodeBrand<R, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<NodeBrand<R, CNilBrand>, RcTypeErasedValue>,
			>): Clone,
			Apply!(<NodeBrand<RMinusReader, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<NodeBrand<RMinusReader, CNilBrand>, RcTypeErasedValue>,
			>): Clone,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcRun<R, CNilBrand, A>,
			>): Member<
					RcCoyoneda<'static, ReaderBrand<RcBrand, E>, RcRun<R, CNilBrand, A>>,
					Idx,
					Remainder = Apply!(
									<RMinusReader as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
										'static,
										RcRun<R, CNilBrand, A>,
									>
								),
				>, {
			self.handle_with::<ReaderBrand<RcBrand, E>, Idx, RMinusReader>(
				move |op: Reader<'static, RcBrand, E, RcRun<RMinusReader, CNilBrand, A>>| match op {
					Reader::Ask(k) => (*k)(env.clone()),
				},
			)
		}
	}
}

fn arcrun_reader_ask_tokens() -> TokenStream {
	quote! {
		/// Lifts an `Ask` reader effect into the `ArcRun` program.
		/// Mirrors [`Run::ask`](crate::types::effects::run::Run::ask);
		/// see that method for cross-wrapper semantics. Differences for
		/// `ArcRun`: threads
		/// [`ArcBrand`](crate::brands::ArcBrand) as the pointer kind
		/// and uses
		/// [`SendReaderBrand`](crate::brands::SendReaderBrand) (rather
		/// than `ReaderBrand`) so the continuation projection
		/// `<ArcBrand as SendRefCountedPointer>::Of<'_, dyn Fn(...) + Send + Sync>`
		/// is structurally `Send + Sync`.
		#[__document_module_generated]
		#[document_signature]
		///
		#[document_type_parameters("The type-level Member-position witness (typically inferred).")]
		///
		#[document_returns("An `ArcRun` program suspended at the lifted `Ask` effect.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::arc_run::ArcRun,
		/// };
		///
		/// type FirstRow = CoproductBrand<ArcCoyonedaBrand<SendReaderBrand<ArcBrand, i32>>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let prog: ArcRun<FirstRow, Scoped, i32> = ArcRun::ask();
		/// let handled: ArcRun<CNilBrand, CNilBrand, i32> = prog.run_reader::<i32, _, CNilBrand>(41);
		/// assert_eq!(handled.extract(), 41);
		/// ```
		#[inline]
		pub fn ask<Idx>() -> Self
		where
			A: Clone + Send + Sync,
			NodeBrand<R, ScopedRow>: SendFunctor,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, A>): Member<
					ArcCoyoneda<'static, crate::brands::SendReaderBrand<crate::brands::ArcBrand, A>, A>,
					Idx,
				>,
			Apply!(<NodeBrand<R, ScopedRow> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<NodeBrand<R, ScopedRow>, ArcTypeErasedValue>,
			>): Clone, {
			let effect: crate::types::effects::reader::SendReader<'static, crate::brands::ArcBrand, A, A> =
				crate::types::effects::reader::SendReader::Ask(
					<crate::brands::ArcBrand as crate::classes::ToDynSendFn>::new(|e: A| e),
				);
			Self::lift::<crate::brands::SendReaderBrand<crate::brands::ArcBrand, A>, Idx>(effect)
		}
	}
}

fn arcrun_reader_asks_tokens() -> TokenStream {
	quote! {
		/// Reads the Reader environment and maps it immediately.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters(
			"The Reader environment type.",
			"The type-level Member-position witness for the Reader effect."
		)]
		#[document_parameters("The projection to apply to the environment.")]
		#[document_returns("An `ArcRun` program that asks for the environment and returns `f(env)`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::arc_run::ArcRun,
		/// };
		///
		/// type Row = CoproductBrand<ArcCoyonedaBrand<SendReaderBrand<ArcBrand, i32>>, CNilBrand>;
		///
		/// let program: ArcRun<Row, CNilBrand, String> =
		/// 	ArcRun::asks::<i32, _>(|env| format!("env={env}"));
		/// let handled: ArcRun<CNilBrand, CNilBrand, String> = program.run_reader::<i32, _, CNilBrand>(7);
		/// assert_eq!(handled.extract(), "env=7");
		/// ```
		#[inline]
		pub fn asks<E, Idx>(f: impl Fn(E) -> A + Send + Sync + 'static) -> Self
		where
			E: Clone + Send + Sync + 'static,
			A: Clone,
			NodeBrand<R, S>: SendFunctor,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, E>):
				Member<ArcCoyoneda<'static, SendReaderBrand<ArcBrand, E>, E>, Idx>,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<NodeBrand<R, S>, ArcTypeErasedValue>,
			>): Clone, {
			ArcRun::<R, S, E>::ask::<Idx>().map(f)
		}
	}
}

fn arcrun_reader_run_reader_tokens() -> TokenStream {
	quote! {
		/// Interprets one Reader effect by supplying a fixed environment.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters(
			"The Reader environment type.",
			"The type-level Member-position witness for the Reader effect.",
			"The first-order row brand with the Reader effect removed."
		)]
		#[document_parameters("The environment value supplied to every Reader ask.")]
		#[document_returns("A first-order-only `ArcRun` program with the Reader effect removed.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::arc_run::ArcRun,
		/// };
		///
		/// type Row = CoproductBrand<ArcCoyonedaBrand<SendReaderBrand<ArcBrand, i32>>, CNilBrand>;
		///
		/// let program: ArcRun<Row, CNilBrand, i32> =
		/// 	ArcRun::<Row, CNilBrand, i32>::ask().map(|env| env + 1);
		/// let handled: ArcRun<CNilBrand, CNilBrand, i32> = program.run_reader::<i32, _, CNilBrand>(41);
		/// assert_eq!(handled.extract(), 42);
		/// ```
		#[inline]
		pub fn run_reader<E, Idx, RMinusReader>(
			self,
			env: E,
		) -> ArcRun<RMinusReader, CNilBrand, A>
		where
			A: Clone + Send + Sync,
			E: Clone + Send + Sync + 'static,
			R: Kind_cdc7cd43dac7585f + 'static,
			RMinusReader: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
			NodeBrand<R, CNilBrand>: SendFunctor,
			NodeBrand<RMinusReader, CNilBrand>: WrapDrop
				+ Kind_cdc7cd43dac7585f<
					Of<'static, ArcFree<NodeBrand<RMinusReader, CNilBrand>, ArcTypeErasedValue>>: Send
																									  + Sync,
				> + SendFunctor,
			Apply!(<NodeBrand<R, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<NodeBrand<R, CNilBrand>, ArcTypeErasedValue>,
			>): Clone,
			Apply!(<NodeBrand<RMinusReader, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<NodeBrand<RMinusReader, CNilBrand>, ArcTypeErasedValue>,
			>): Clone,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcRun<R, CNilBrand, A>,
			>): Member<
					ArcCoyoneda<'static, SendReaderBrand<ArcBrand, E>, ArcRun<R, CNilBrand, A>>,
					Idx,
					Remainder = Apply!(
									<RMinusReader as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
										'static,
										ArcRun<R, CNilBrand, A>,
									>
								),
				>, {
			self.handle_with::<SendReaderBrand<ArcBrand, E>, Idx, RMinusReader>(
				move |op: SendReader<'static, ArcBrand, E, ArcRun<RMinusReader, CNilBrand, A>>| match op {
					SendReader::Ask(k) => (*k)(env.clone()),
				},
			)
		}
	}
}

fn run_explicit_reader_ask_tokens() -> TokenStream {
	quote! {
		/// Lifts an `Ask` reader effect into the `RunExplicit`
		/// program. Mirrors
		/// [`Run::ask`](crate::types::effects::run::Run::ask); see
		/// that method for cross-wrapper semantics. Differences for
		/// `RunExplicit`: the bare [`Coyoneda`] variant pairs with the
		/// Box-in-Wrap Explicit substrate. Threads
		/// [`BoxBrand`](crate::brands::BoxBrand) as the pointer kind
		/// post-Phase-3.5 retrofit.
		#[__document_module_generated]
		#[document_signature]
		///
		#[document_type_parameters("The type-level Member-position witness (typically inferred).")]
		///
		#[document_returns("A `RunExplicit` program suspended at the lifted `Ask` effect.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::run_explicit::RunExplicit,
		/// };
		///
		/// type FirstRow = CoproductBrand<CoyonedaBrand<BoxReaderBrand<BoxBrand, i32>>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let prog: RunExplicit<'static, FirstRow, Scoped, i32> = RunExplicit::ask();
		/// let handled: RunExplicit<'static, CNilBrand, CNilBrand, i32> =
		/// 	prog.run_reader::<i32, _, CNilBrand>(41);
		/// assert_eq!(handled.extract(), 41);
		/// ```
		#[inline]
		pub fn ask<Idx>() -> Self
		where
			A: 'static,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>):
				Member<Coyoneda<'a, crate::brands::BoxReaderBrand<crate::brands::BoxBrand, A>, A>, Idx>, {
			let effect: crate::types::effects::reader::BoxReader<'a, crate::brands::BoxBrand, A, A> =
				crate::types::effects::reader::BoxReader::Ask(
					<crate::brands::BoxBrand as crate::classes::ToDynFnOnce>::new(|e: A| e),
				);
			Self::lift::<crate::brands::BoxReaderBrand<crate::brands::BoxBrand, A>, Idx>(effect)
		}
	}
}

fn run_explicit_reader_asks_tokens() -> TokenStream {
	quote! {
		/// Reads the Reader environment and maps it immediately.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters(
			"The Reader environment type.",
			"The type-level Member-position witness for the Reader effect."
		)]
		#[document_parameters("The projection to apply to the environment.")]
		#[document_returns("A `RunExplicit` program that asks for the environment and returns `f(env)`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::run_explicit::RunExplicit,
		/// };
		///
		/// type Row = CoproductBrand<CoyonedaBrand<BoxReaderBrand<BoxBrand, i32>>, CNilBrand>;
		///
		/// let program: RunExplicit<'static, Row, CNilBrand, String> =
		/// 	RunExplicit::asks::<i32, _>(|env| format!("env={env}"));
		/// let handled: RunExplicit<'static, CNilBrand, CNilBrand, String> =
		/// 	program.run_reader::<i32, _, CNilBrand>(7);
		/// assert_eq!(handled.extract(), "env=7");
		/// ```
		#[inline]
		pub fn asks<E, Idx>(f: impl Fn(E) -> A + 'a) -> Self
		where
			E: 'static,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, E>):
				Member<Coyoneda<'a, BoxReaderBrand<BoxBrand, E>, E>, Idx>, {
			RunExplicit::<'a, R, S, E>::ask::<Idx>().map(f)
		}
	}
}

fn run_explicit_reader_run_reader_tokens() -> TokenStream {
	quote! {
		/// Interprets one Reader effect by supplying a fixed environment.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters(
			"The Reader environment type.",
			"The type-level Member-position witness for the Reader effect.",
			"The first-order row brand with the Reader effect removed."
		)]
		#[document_parameters("The environment value supplied to every Reader ask.")]
		#[document_returns("A first-order-only `RunExplicit` program with the Reader effect removed.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::run_explicit::RunExplicit,
		/// };
		///
		/// type Row = CoproductBrand<CoyonedaBrand<BoxReaderBrand<BoxBrand, i32>>, CNilBrand>;
		///
		/// let program: RunExplicit<'static, Row, CNilBrand, i32> =
		/// 	RunExplicit::<'static, Row, CNilBrand, i32>::ask().map(|env| env + 1);
		/// let handled: RunExplicit<'static, CNilBrand, CNilBrand, i32> =
		/// 	program.run_reader::<i32, _, CNilBrand>(41);
		/// assert_eq!(handled.extract(), 42);
		/// ```
		#[inline]
		pub fn run_reader<E, Idx, RMinusReader>(
			self,
			env: E,
		) -> RunExplicit<'a, RMinusReader, CNilBrand, A>
		where
			E: Clone + 'static + 'a,
			RMinusReader: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RunExplicit<'a, R, CNilBrand, A>,
			>): Member<
					Coyoneda<'a, BoxReaderBrand<BoxBrand, E>, RunExplicit<'a, R, CNilBrand, A>>,
					Idx,
					Remainder = Apply!(
									<RMinusReader as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
										'a,
										RunExplicit<'a, R, CNilBrand, A>,
									>
								),
				>, {
			self.handle_with::<BoxReaderBrand<BoxBrand, E>, Idx, RMinusReader>(
				move |op: BoxReader<'a, BoxBrand, E, RunExplicit<'a, RMinusReader, CNilBrand, A>>| match op
				{
					BoxReader::Ask(k) => k(env.clone()),
				},
			)
		}
	}
}

fn rcrun_explicit_reader_ask_tokens() -> TokenStream {
	quote! {
		/// Lifts an `Ask` reader effect into the `RcRunExplicit`
		/// program. Mirrors
		/// [`Run::ask`](crate::types::effects::run::Run::ask); see
		/// that method for cross-wrapper semantics. Differences for
		/// `RcRunExplicit`: the [`RcCoyoneda`] variant pairs with the
		/// `Rc`-shared Explicit substrate (multi-shot continuations);
		/// `A: Clone` is required because the underlying `RcCoyoneda`
		/// substrate's `peel` walks shared continuation projections.
		/// Threads [`RcBrand`](crate::brands::RcBrand) as the pointer
		/// kind.
		#[__document_module_generated]
		#[document_signature]
		///
		#[document_type_parameters("The type-level Member-position witness (typically inferred).")]
		///
		#[document_returns("An `RcRunExplicit` program suspended at the lifted `Ask` effect.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::rc_run_explicit::RcRunExplicit,
		/// };
		///
		/// type FirstRow = CoproductBrand<RcCoyonedaBrand<ReaderBrand<RcBrand, i32>>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let prog: RcRunExplicit<'static, FirstRow, Scoped, i32> = RcRunExplicit::ask();
		/// let handled: RcRunExplicit<'static, CNilBrand, CNilBrand, i32> =
		/// 	prog.run_reader::<i32, _, CNilBrand>(41);
		/// assert_eq!(handled.extract(), 41);
		/// ```
		#[inline]
		pub fn ask<Idx>() -> Self
		where
			A: Clone + 'static,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>):
				Member<RcCoyoneda<'a, crate::brands::ReaderBrand<crate::brands::RcBrand, A>, A>, Idx>, {
			let effect: crate::types::effects::reader::Reader<'a, crate::brands::RcBrand, A, A> =
				crate::types::effects::reader::Reader::Ask(
					<crate::brands::RcBrand as crate::classes::ToDynCloneFn>::new(|e: A| e),
				);
			Self::lift::<crate::brands::ReaderBrand<crate::brands::RcBrand, A>, Idx>(effect)
		}
	}
}

fn rcrun_explicit_reader_asks_tokens() -> TokenStream {
	quote! {
		/// Reads the Reader environment and maps it immediately.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters(
			"The Reader environment type.",
			"The type-level Member-position witness for the Reader effect."
		)]
		#[document_parameters("The projection to apply to the environment.")]
		#[document_returns(
			"An `RcRunExplicit` program that asks for the environment and returns `f(env)`."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::rc_run_explicit::RcRunExplicit,
		/// };
		///
		/// type Row = CoproductBrand<RcCoyonedaBrand<ReaderBrand<RcBrand, i32>>, CNilBrand>;
		///
		/// let program: RcRunExplicit<'static, Row, CNilBrand, String> =
		/// 	RcRunExplicit::asks::<i32, _>(|env| format!("env={env}"));
		/// let handled: RcRunExplicit<'static, CNilBrand, CNilBrand, String> =
		/// 	program.run_reader::<i32, _, CNilBrand>(7);
		/// assert_eq!(handled.extract(), "env=7");
		/// ```
		#[inline]
		pub fn asks<E, Idx>(f: impl Fn(E) -> A + 'a) -> Self
		where
			E: Clone + 'static,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, E>):
				Member<RcCoyoneda<'a, ReaderBrand<RcBrand, E>, E>, Idx>,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, S>, E>,
			>): Clone,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, S>, A>,
			>): Clone, {
			RcRunExplicit::<'a, R, S, E>::ask::<Idx>().map(f)
		}
	}
}

fn rcrun_explicit_reader_run_reader_tokens() -> TokenStream {
	quote! {
		/// Interprets one Reader effect by supplying a fixed environment.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters(
			"The Reader environment type.",
			"The type-level Member-position witness for the Reader effect.",
			"The first-order row brand with the Reader effect removed."
		)]
		#[document_parameters("The environment value supplied to every Reader ask.")]
		#[document_returns("A first-order-only `RcRunExplicit` program with the Reader effect removed.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::rc_run_explicit::RcRunExplicit,
		/// };
		///
		/// type Row = CoproductBrand<RcCoyonedaBrand<ReaderBrand<RcBrand, i32>>, CNilBrand>;
		///
		/// let program: RcRunExplicit<'static, Row, CNilBrand, i32> =
		/// 	RcRunExplicit::<'static, Row, CNilBrand, i32>::ask().map(|env| env + 1);
		/// let handled: RcRunExplicit<'static, CNilBrand, CNilBrand, i32> =
		/// 	program.run_reader::<i32, _, CNilBrand>(41);
		/// assert_eq!(handled.extract(), 42);
		/// ```
		#[inline]
		pub fn run_reader<E, Idx, RMinusReader>(
			self,
			env: E,
		) -> RcRunExplicit<'a, RMinusReader, CNilBrand, A>
		where
			A: Clone,
			E: Clone + 'static + 'a,
			RMinusReader: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
			Apply!(<NodeBrand<R, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, CNilBrand>, A>,
			>): Clone,
			Apply!(<NodeBrand<RMinusReader, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<RMinusReader, CNilBrand>, A>,
			>): Clone,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcRunExplicit<'a, R, CNilBrand, A>,
			>): Member<
					RcCoyoneda<'a, ReaderBrand<RcBrand, E>, RcRunExplicit<'a, R, CNilBrand, A>>,
					Idx,
					Remainder = Apply!(
									<RMinusReader as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
										'a,
										RcRunExplicit<'a, R, CNilBrand, A>,
									>
								),
				>, {
			self.handle_with::<ReaderBrand<RcBrand, E>, Idx, RMinusReader>(
				move |op: Reader<'a, RcBrand, E, RcRunExplicit<'a, RMinusReader, CNilBrand, A>>| match op {
					Reader::Ask(k) => (*k)(env.clone()),
				},
			)
		}
	}
}

fn arcrun_explicit_reader_ask_tokens() -> TokenStream {
	quote! {
		/// Lifts an `Ask` reader effect into the `ArcRunExplicit`
		/// program. Mirrors
		/// [`Run::ask`](crate::types::effects::run::Run::ask); see
		/// that method for cross-wrapper semantics. Differences for
		/// `ArcRunExplicit`: threads
		/// [`ArcBrand`](crate::brands::ArcBrand) as the pointer kind
		/// and uses
		/// [`SendReaderBrand`](crate::brands::SendReaderBrand) (rather
		/// than `ReaderBrand`) so the continuation projection is
		/// structurally `Send + Sync`.
		#[__document_module_generated]
		#[document_signature]
		///
		#[document_type_parameters("The type-level Member-position witness (typically inferred).")]
		///
		#[document_returns("An `ArcRunExplicit` program suspended at the lifted `Ask` effect.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::arc_run_explicit::ArcRunExplicit,
		/// };
		///
		/// type FirstRow = CoproductBrand<ArcCoyonedaBrand<SendReaderBrand<ArcBrand, i32>>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let prog: ArcRunExplicit<'static, FirstRow, Scoped, i32> = ArcRunExplicit::ask();
		/// let handled: ArcRunExplicit<'static, CNilBrand, CNilBrand, i32> =
		/// 	prog.run_reader::<i32, _, CNilBrand>(41);
		/// assert_eq!(handled.extract(), 41);
		/// ```
		#[inline]
		pub fn ask<Idx>() -> Self
		where
			A: Clone + Send + Sync + 'static,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>):
				Member<ArcCoyoneda<'a, crate::brands::SendReaderBrand<crate::brands::ArcBrand, A>, A>, Idx>,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, A>,
			>): Send + Sync,
			Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, A>,
			>): Send + Sync,
			Apply!(<NodeBrand<R, ScopedRow> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, A>,
			>): Clone + Send + Sync, {
			let effect: crate::types::effects::reader::SendReader<'a, crate::brands::ArcBrand, A, A> =
				crate::types::effects::reader::SendReader::Ask(
					<crate::brands::ArcBrand as crate::classes::ToDynSendFn>::new(|e: A| e),
				);
			Self::lift::<crate::brands::SendReaderBrand<crate::brands::ArcBrand, A>, Idx>(effect)
		}
	}
}

fn arcrun_explicit_reader_asks_tokens() -> TokenStream {
	quote! {
		/// Reads the Reader environment and maps it immediately.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters(
			"The Reader environment type.",
			"The type-level Member-position witness for the Reader effect."
		)]
		#[document_parameters("The projection to apply to the environment.")]
		#[document_returns(
			"An `ArcRunExplicit` program that asks for the environment and returns `f(env)`."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::arc_run_explicit::ArcRunExplicit,
		/// };
		///
		/// type Row = CoproductBrand<ArcCoyonedaBrand<SendReaderBrand<ArcBrand, i32>>, CNilBrand>;
		///
		/// let program: ArcRunExplicit<'static, Row, CNilBrand, String> =
		/// 	ArcRunExplicit::asks::<i32, _>(|env| format!("env={env}"));
		/// let handled: ArcRunExplicit<'static, CNilBrand, CNilBrand, String> =
		/// 	program.run_reader::<i32, _, CNilBrand>(7);
		/// assert_eq!(handled.extract(), "env=7");
		/// ```
		#[inline]
		pub fn asks<E, Idx>(f: impl Fn(E) -> A + Send + Sync + 'a) -> Self
		where
			A: Clone,
			E: Clone + Send + Sync + 'static,
			NodeBrand<R, S>: SendFunctor,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, E>):
				Member<ArcCoyoneda<'a, SendReaderBrand<ArcBrand, E>, E>, Idx>,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, S>, E>,
			>): Send + Sync,
			Apply!(<S as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, S>, E>,
			>): Send + Sync,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, S>, E>,
			>): Clone + Send + Sync,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, S>, A>,
			>): Clone + Send + Sync, {
			ArcRunExplicit::<'a, R, S, E>::ask::<Idx>().map(f)
		}
	}
}

fn arcrun_explicit_reader_run_reader_tokens() -> TokenStream {
	quote! {
		/// Interprets one Reader effect by supplying a fixed environment.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters(
			"The Reader environment type.",
			"The type-level Member-position witness for the Reader effect.",
			"The first-order row brand with the Reader effect removed."
		)]
		#[document_parameters("The environment value supplied to every Reader ask.")]
		#[document_returns("A first-order-only `ArcRunExplicit` program with the Reader effect removed.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::arc_run_explicit::ArcRunExplicit,
		/// };
		///
		/// type Row = CoproductBrand<ArcCoyonedaBrand<SendReaderBrand<ArcBrand, i32>>, CNilBrand>;
		///
		/// let program: ArcRunExplicit<'static, Row, CNilBrand, i32> =
		/// 	ArcRunExplicit::<'static, Row, CNilBrand, i32>::ask().map(|env| env + 1);
		/// let handled: ArcRunExplicit<'static, CNilBrand, CNilBrand, i32> =
		/// 	program.run_reader::<i32, _, CNilBrand>(41);
		/// assert_eq!(handled.extract(), 42);
		/// ```
		#[inline]
		pub fn run_reader<E, Idx, RMinusReader>(
			self,
			env: E,
		) -> ArcRunExplicit<'a, RMinusReader, CNilBrand, A>
		where
			A: Clone + Send + Sync,
			E: Clone + Send + Sync + 'static + 'a,
			RMinusReader: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
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
			Apply!(<NodeBrand<RMinusReader, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<RMinusReader, CNilBrand>, A>,
			>): Clone + Send + Sync,
			Apply!(<RMinusReader as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<RMinusReader, CNilBrand>, A>,
			>): Send + Sync,
			Apply!(<RMinusReader as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, RMinusReader, CNilBrand, A>,
			>): Send + Sync,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, R, CNilBrand, A>,
			>): Member<
					ArcCoyoneda<'a, SendReaderBrand<ArcBrand, E>, ArcRunExplicit<'a, R, CNilBrand, A>>,
					Idx,
					Remainder = Apply!(
									<RMinusReader as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
										'a,
										ArcRunExplicit<'a, R, CNilBrand, A>,
									>
								),
				>, {
			self.handle_with::<SendReaderBrand<ArcBrand, E>, Idx, RMinusReader>(
				move |op: SendReader<'a, ArcBrand, E, ArcRunExplicit<'a, RMinusReader, CNilBrand, A>>| {
					match op {
						SendReader::Ask(k) => (*k)(env.clone()),
					}
				},
			)
		}
	}
}
