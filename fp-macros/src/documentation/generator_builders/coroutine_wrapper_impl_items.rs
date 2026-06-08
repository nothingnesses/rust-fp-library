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

pub(super) fn coroutine_wrapper_impl_items_from_descriptor(
	wrapper: WrapperName,
	method: RunWrapperMethod,
) -> Option<syn::Result<Vec<ImplItem>>> {
	generator_descriptors::method_spec(EffectName::Coroutine, method)?;
	generator_descriptors::wrapper_spec(wrapper)?;

	let tokens = match (wrapper, method) {
		(WrapperName::Run, RunWrapperMethod::YieldValue) => run_yield_value_tokens(),
		(WrapperName::Run, RunWrapperMethod::RunCoroutine) => run_run_coroutine_tokens(),
		(WrapperName::RcRun, RunWrapperMethod::YieldValue) => rcrun_yield_value_tokens(),
		(WrapperName::RcRun, RunWrapperMethod::RunCoroutine) => rcrun_run_coroutine_tokens(),
		(WrapperName::ArcRun, RunWrapperMethod::YieldValue) => arcrun_yield_value_tokens(),
		(WrapperName::ArcRun, RunWrapperMethod::RunCoroutine) => arcrun_run_coroutine_tokens(),
		(WrapperName::RunExplicit, RunWrapperMethod::YieldValue) =>
			run_explicit_yield_value_tokens(),
		(WrapperName::RunExplicit, RunWrapperMethod::RunCoroutine) =>
			run_explicit_run_coroutine_tokens(),
		(WrapperName::RcRunExplicit, RunWrapperMethod::YieldValue) =>
			rcrun_explicit_yield_value_tokens(),
		(WrapperName::RcRunExplicit, RunWrapperMethod::RunCoroutine) =>
			rcrun_explicit_run_coroutine_tokens(),
		(WrapperName::ArcRunExplicit, RunWrapperMethod::YieldValue) =>
			arcrun_explicit_yield_value_tokens(),
		(WrapperName::ArcRunExplicit, RunWrapperMethod::RunCoroutine) =>
			arcrun_explicit_run_coroutine_tokens(),
		_ => return None,
	};

	Some(impl_items_from_tokens(tokens))
}

fn run_yield_value_tokens() -> TokenStream {
	quote! {
		/// Lifts a Coroutine yield into the `Run` program.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters(
			"The yielded output type.",
			"The type-level Member-position witness for the Coroutine effect."
		)]
		#[document_parameters("The output value yielded to the coroutine runner.")]
		#[document_returns("A `Run` program suspended at the lifted Coroutine effect.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::run::Run,
		/// };
		///
		/// type Row = CoproductBrand<CoyonedaBrand<BoxCoroutineBrand<BoxBrand, &'static str, i32>>, CNilBrand>;
		///
		/// let program: Run<Row, CNilBrand, i32> = Run::yield_value::<&'static str, _>("next");
		/// let status = program.run_coroutine::<&'static str, i32, _, CNilBrand>().extract();
		/// assert!(matches!(status, fp_library::types::effects::coroutine::RunCoroutineStatus::Continue("next", _)));
		/// ```
		#[inline]
		pub fn yield_value<Out, Idx>(out: Out) -> Self
		where
			Out: 'static,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, A>):
				crate::types::effects::member::Member<
					crate::types::Coyoneda<
						'static,
						crate::brands::BoxCoroutineBrand<crate::brands::BoxBrand, Out, A>,
						A,
					>,
					Idx,
				>, {
			let effect: crate::types::effects::coroutine::BoxCoroutine<
				'static,
				crate::brands::BoxBrand,
				Out,
				A,
				A,
			> = crate::types::effects::coroutine::BoxCoroutine::Yield(
				out,
				<crate::brands::BoxBrand as crate::classes::ToDynFnOnce>::new(|input: A| input),
			);
			Self::lift::<crate::brands::BoxCoroutineBrand<crate::brands::BoxBrand, Out, A>, Idx>(
				effect,
			)
		}
	}
}

fn run_run_coroutine_tokens() -> TokenStream {
	quote! {
		/// Interprets Coroutine by returning the current coroutine status.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters(
			"The yielded output type.",
			"The resume input type.",
			"The type-level Member-position witness for the Coroutine effect.",
			"The first-order row brand with the Coroutine effect removed."
		)]
		#[document_returns("A first-order-only `Run` program returning the next Coroutine status.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::{
		/// 		coroutine::RunCoroutineStatus,
		/// 		run::Run,
		/// 	},
		/// };
		///
		/// type Row = CoproductBrand<CoyonedaBrand<BoxCoroutineBrand<BoxBrand, &'static str, i32>>, CNilBrand>;
		///
		/// let program: Run<Row, CNilBrand, i32> = Run::yield_value::<&'static str, _>("next");
		/// let status = program.run_coroutine::<&'static str, i32, _, CNilBrand>().extract();
		/// assert!(matches!(status, RunCoroutineStatus::Continue("next", _)));
		/// ```
		#[inline]
		pub fn run_coroutine<Out, In, Idx, RMinusCoroutine>(
			self
		) -> Run<
			RMinusCoroutine,
			CNilBrand,
			crate::types::effects::coroutine::RunCoroutineStatus<
				RMinusCoroutine,
				CNilBrand,
				Out,
				In,
				A,
			>,
		>
		where
			Out: 'static,
			In: 'static,
			RMinusCoroutine: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				Run<
					R,
					CNilBrand,
					crate::types::effects::coroutine::RunCoroutineStatus<
						RMinusCoroutine,
						CNilBrand,
						Out,
						In,
						A,
					>,
				>,
			>): crate::types::effects::member::Member<
				crate::types::Coyoneda<
					'static,
					crate::brands::BoxCoroutineBrand<crate::brands::BoxBrand, Out, In>,
					Run<
						R,
						CNilBrand,
						crate::types::effects::coroutine::RunCoroutineStatus<
							RMinusCoroutine,
							CNilBrand,
							Out,
							In,
							A,
						>,
					>,
				>,
				Idx,
				Remainder = Apply!(
					<RMinusCoroutine as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
						'static,
						Run<
							R,
							CNilBrand,
							crate::types::effects::coroutine::RunCoroutineStatus<
								RMinusCoroutine,
								CNilBrand,
								Out,
								In,
								A,
							>,
						>,
					>
				),
			>, {
			type Status<RMinusCoroutine, Out, In, A> =
				crate::types::effects::coroutine::RunCoroutineStatus<
					RMinusCoroutine,
					CNilBrand,
					Out,
					In,
					A,
				>;
			self.map(Status::<RMinusCoroutine, Out, In, A>::Done)
				.handle_with::<
					crate::brands::BoxCoroutineBrand<crate::brands::BoxBrand, Out, In>,
					Idx,
					RMinusCoroutine,
				>(
					|op: crate::types::effects::coroutine::BoxCoroutine<
						'static,
						crate::brands::BoxBrand,
						Out,
						In,
						Run<RMinusCoroutine, CNilBrand, Status<RMinusCoroutine, Out, In, A>>,
					>| match op {
						crate::types::effects::coroutine::BoxCoroutine::Yield(out, resume) =>
							Run::pure(Status::Continue(out, resume)),
					},
				)
		}
	}
}

fn rcrun_yield_value_tokens() -> TokenStream {
	quote! {
		/// Lifts a Coroutine yield into the `RcRun` program.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters(
			"The yielded output type.",
			"The type-level Member-position witness for the Coroutine effect."
		)]
		#[document_parameters("The output value yielded to the coroutine runner.")]
		#[document_returns("An `RcRun` program suspended at the lifted Coroutine effect.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::rc_run::RcRun,
		/// };
		///
		/// type Row = CoproductBrand<RcCoyonedaBrand<CoroutineBrand<RcBrand, &'static str, i32>>, CNilBrand>;
		///
		/// let program: RcRun<Row, CNilBrand, i32> = RcRun::yield_value::<&'static str, _>("next");
		/// let status = program.run_coroutine::<&'static str, i32, _, CNilBrand>().extract();
		/// assert!(matches!(status, fp_library::types::effects::coroutine::RcRunCoroutineStatus::Continue("next", _)));
		/// ```
		#[inline]
		pub fn yield_value<Out, Idx>(out: Out) -> Self
		where
			A: Clone + 'static,
			Out: Clone + 'static,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, A>): Member<
				RcCoyoneda<'static, crate::brands::CoroutineBrand<crate::brands::RcBrand, Out, A>, A>,
				Idx,
			>,
			Apply!(<NodeBrand<R, ScopedRow> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<NodeBrand<R, ScopedRow>, RcTypeErasedValue>,
			>): Clone, {
			let effect: crate::types::effects::coroutine::Coroutine<
				'static,
				crate::brands::RcBrand,
				Out,
				A,
				A,
			> = crate::types::effects::coroutine::Coroutine::Yield(
				out,
				<crate::brands::RcBrand as crate::classes::ToDynCloneFn>::new(|input: A| input),
			);
			Self::lift::<crate::brands::CoroutineBrand<crate::brands::RcBrand, Out, A>, Idx>(
				effect,
			)
		}
	}
}

fn rcrun_run_coroutine_tokens() -> TokenStream {
	quote! {
		/// Interprets Coroutine by returning the current coroutine status.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters(
			"The yielded output type.",
			"The resume input type.",
			"The type-level Member-position witness for the Coroutine effect.",
			"The first-order row brand with the Coroutine effect removed."
		)]
		#[document_returns("A first-order-only `RcRun` program returning the next Coroutine status.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::{
		/// 		coroutine::RcRunCoroutineStatus,
		/// 		rc_run::RcRun,
		/// 	},
		/// };
		///
		/// type Row = CoproductBrand<RcCoyonedaBrand<CoroutineBrand<RcBrand, &'static str, i32>>, CNilBrand>;
		///
		/// let program: RcRun<Row, CNilBrand, i32> = RcRun::yield_value::<&'static str, _>("next");
		/// let status = program.run_coroutine::<&'static str, i32, _, CNilBrand>().extract();
		/// assert!(matches!(status, RcRunCoroutineStatus::Continue("next", _)));
		/// ```
		#[inline]
		pub fn run_coroutine<Out, In, Idx, RMinusCoroutine>(
			self
		) -> RcRun<
			RMinusCoroutine,
			CNilBrand,
			crate::types::effects::coroutine::RcRunCoroutineStatus<
				RMinusCoroutine,
				CNilBrand,
				Out,
				In,
				A,
			>,
		>
		where
			A: Clone + 'static,
			Out: Clone + 'static,
			In: 'static,
			RMinusCoroutine: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
			Apply!(<NodeBrand<R, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<NodeBrand<R, CNilBrand>, RcTypeErasedValue>,
			>): Clone,
			Apply!(<NodeBrand<RMinusCoroutine, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<NodeBrand<RMinusCoroutine, CNilBrand>, RcTypeErasedValue>,
			>): Clone,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcRun<
					R,
					CNilBrand,
					crate::types::effects::coroutine::RcRunCoroutineStatus<
						RMinusCoroutine,
						CNilBrand,
						Out,
						In,
						A,
					>,
				>,
			>): Member<
				RcCoyoneda<
					'static,
					crate::brands::CoroutineBrand<crate::brands::RcBrand, Out, In>,
					RcRun<
						R,
						CNilBrand,
						crate::types::effects::coroutine::RcRunCoroutineStatus<
							RMinusCoroutine,
							CNilBrand,
							Out,
							In,
							A,
						>,
					>,
				>,
				Idx,
				Remainder = Apply!(
					<RMinusCoroutine as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
						'static,
						RcRun<
							R,
							CNilBrand,
							crate::types::effects::coroutine::RcRunCoroutineStatus<
								RMinusCoroutine,
								CNilBrand,
								Out,
								In,
								A,
							>,
						>,
					>
				),
			>, {
			type Status<RMinusCoroutine, Out, In, A> =
				crate::types::effects::coroutine::RcRunCoroutineStatus<
					RMinusCoroutine,
					CNilBrand,
					Out,
					In,
					A,
				>;
			self.map(Status::<RMinusCoroutine, Out, In, A>::Done)
				.handle_with::<
					crate::brands::CoroutineBrand<crate::brands::RcBrand, Out, In>,
					Idx,
					RMinusCoroutine,
				>(
					|op: crate::types::effects::coroutine::Coroutine<
						'static,
						crate::brands::RcBrand,
						Out,
						In,
						RcRun<RMinusCoroutine, CNilBrand, Status<RMinusCoroutine, Out, In, A>>,
					>| match op {
						crate::types::effects::coroutine::Coroutine::Yield(out, resume) =>
							RcRun::pure(Status::Continue(out, resume)),
					},
				)
		}
	}
}

fn arcrun_yield_value_tokens() -> TokenStream {
	quote! {
		/// Lifts a Coroutine yield into the `ArcRun` program.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters(
			"The yielded output type.",
			"The type-level Member-position witness for the Coroutine effect."
		)]
		#[document_parameters("The output value yielded to the coroutine runner.")]
		#[document_returns("An `ArcRun` program suspended at the lifted Coroutine effect.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::arc_run::ArcRun,
		/// };
		///
		/// type Row = CoproductBrand<ArcCoyonedaBrand<SendCoroutineBrand<ArcBrand, &'static str, i32>>, CNilBrand>;
		///
		/// let program: ArcRun<Row, CNilBrand, i32> = ArcRun::yield_value::<&'static str, _>("next");
		/// let status = program.run_coroutine::<&'static str, i32, _, CNilBrand>().extract();
		/// assert!(matches!(status, fp_library::types::effects::coroutine::ArcRunCoroutineStatus::Continue("next", _)));
		/// ```
		#[inline]
		pub fn yield_value<Out, Idx>(out: Out) -> Self
		where
			A: Clone + Send + Sync + 'static,
			Out: Clone + Send + Sync + 'static,
			NodeBrand<R, ScopedRow>: SendFunctor,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, A>): Member<
				ArcCoyoneda<
					'static,
					crate::brands::SendCoroutineBrand<crate::brands::ArcBrand, Out, A>,
					A,
				>,
				Idx,
			>,
			Apply!(<NodeBrand<R, ScopedRow> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<NodeBrand<R, ScopedRow>, ArcTypeErasedValue>,
			>): Clone, {
			let effect: crate::types::effects::coroutine::SendCoroutine<
				'static,
				crate::brands::ArcBrand,
				Out,
				A,
				A,
			> = crate::types::effects::coroutine::SendCoroutine::Yield(
				out,
				<crate::brands::ArcBrand as crate::classes::ToDynSendFn>::new(|input: A| input),
			);
			Self::lift::<crate::brands::SendCoroutineBrand<crate::brands::ArcBrand, Out, A>, Idx>(
				effect,
			)
		}
	}
}

fn arcrun_run_coroutine_tokens() -> TokenStream {
	quote! {
		/// Interprets Coroutine by returning the current coroutine status.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters(
			"The yielded output type.",
			"The resume input type.",
			"The type-level Member-position witness for the Coroutine effect.",
			"The first-order row brand with the Coroutine effect removed."
		)]
		#[document_returns("A first-order-only `ArcRun` program returning the next Coroutine status.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::{
		/// 		arc_run::ArcRun,
		/// 		coroutine::ArcRunCoroutineStatus,
		/// 	},
		/// };
		///
		/// type Row = CoproductBrand<ArcCoyonedaBrand<SendCoroutineBrand<ArcBrand, &'static str, i32>>, CNilBrand>;
		///
		/// let program: ArcRun<Row, CNilBrand, i32> = ArcRun::yield_value::<&'static str, _>("next");
		/// let status = program.run_coroutine::<&'static str, i32, _, CNilBrand>().extract();
		/// assert!(matches!(status, ArcRunCoroutineStatus::Continue("next", _)));
		/// ```
		#[inline]
		pub fn run_coroutine<Out, In, Idx, RMinusCoroutine>(
			self
		) -> ArcRun<
			RMinusCoroutine,
			CNilBrand,
			crate::types::effects::coroutine::ArcRunCoroutineStatus<
				RMinusCoroutine,
				CNilBrand,
				Out,
				In,
				A,
			>,
		>
		where
			A: Clone + Send + Sync + 'static,
			Out: Clone + Send + Sync + 'static,
			In: Send + Sync + 'static,
			R: Kind_cdc7cd43dac7585f + 'static,
			RMinusCoroutine: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
			NodeBrand<R, CNilBrand>: SendFunctor,
			NodeBrand<RMinusCoroutine, CNilBrand>: WrapDrop
				+ Kind_cdc7cd43dac7585f<
					Of<'static, ArcFree<NodeBrand<RMinusCoroutine, CNilBrand>, ArcTypeErasedValue>>:
						Send + Sync,
				> + SendFunctor,
			Apply!(<NodeBrand<R, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<NodeBrand<R, CNilBrand>, ArcTypeErasedValue>,
			>): Clone,
			Apply!(<NodeBrand<RMinusCoroutine, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<NodeBrand<RMinusCoroutine, CNilBrand>, ArcTypeErasedValue>,
			>): Clone,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcRun<
					R,
					CNilBrand,
					crate::types::effects::coroutine::ArcRunCoroutineStatus<
						RMinusCoroutine,
						CNilBrand,
						Out,
						In,
						A,
					>,
				>,
			>): Member<
				ArcCoyoneda<
					'static,
					crate::brands::SendCoroutineBrand<crate::brands::ArcBrand, Out, In>,
					ArcRun<
						R,
						CNilBrand,
						crate::types::effects::coroutine::ArcRunCoroutineStatus<
							RMinusCoroutine,
							CNilBrand,
							Out,
							In,
							A,
						>,
					>,
				>,
				Idx,
				Remainder = Apply!(
					<RMinusCoroutine as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
						'static,
						ArcRun<
							R,
							CNilBrand,
							crate::types::effects::coroutine::ArcRunCoroutineStatus<
								RMinusCoroutine,
								CNilBrand,
								Out,
								In,
								A,
							>,
						>,
					>
				),
			>, {
			type Status<RMinusCoroutine, Out, In, A> =
				crate::types::effects::coroutine::ArcRunCoroutineStatus<
					RMinusCoroutine,
					CNilBrand,
					Out,
					In,
					A,
				>;
			self.map(Status::<RMinusCoroutine, Out, In, A>::Done)
				.handle_with::<
					crate::brands::SendCoroutineBrand<crate::brands::ArcBrand, Out, In>,
					Idx,
					RMinusCoroutine,
				>(
					|op: crate::types::effects::coroutine::SendCoroutine<
						'static,
						crate::brands::ArcBrand,
						Out,
						In,
						ArcRun<RMinusCoroutine, CNilBrand, Status<RMinusCoroutine, Out, In, A>>,
					>| match op {
						crate::types::effects::coroutine::SendCoroutine::Yield(out, resume) =>
							ArcRun::pure(Status::Continue(out, resume)),
					},
				)
		}
	}
}

fn run_explicit_yield_value_tokens() -> TokenStream {
	quote! {
		/// Lifts a Coroutine yield into the `RunExplicit` program.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters(
			"The yielded output type.",
			"The type-level Member-position witness for the Coroutine effect."
		)]
		#[document_parameters("The output value yielded to the coroutine runner.")]
		#[document_returns("A `RunExplicit` program suspended at the lifted Coroutine effect.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::run_explicit::RunExplicit,
		/// };
		///
		/// type Row = CoproductBrand<CoyonedaBrand<BoxCoroutineBrand<BoxBrand, &'static str, i32>>, CNilBrand>;
		///
		/// let program: RunExplicit<'static, Row, CNilBrand, i32> =
		/// 	RunExplicit::yield_value::<&'static str, _>("next");
		/// let status = program.run_coroutine::<&'static str, i32, _, CNilBrand>().extract();
		/// assert!(matches!(status, fp_library::types::effects::coroutine::RunExplicitCoroutineStatus::Continue("next", _)));
		/// ```
		#[inline]
		pub fn yield_value<Out, Idx>(out: Out) -> Self
		where
			A: 'static,
			Out: 'static + 'a,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>):
				crate::types::effects::member::Member<
					crate::types::Coyoneda<
						'a,
						crate::brands::BoxCoroutineBrand<crate::brands::BoxBrand, Out, A>,
						A,
					>,
					Idx,
				>, {
			let effect: crate::types::effects::coroutine::BoxCoroutine<
				'a,
				crate::brands::BoxBrand,
				Out,
				A,
				A,
			> = crate::types::effects::coroutine::BoxCoroutine::Yield(
				out,
				<crate::brands::BoxBrand as crate::classes::ToDynFnOnce>::new(|input: A| input),
			);
			Self::lift::<crate::brands::BoxCoroutineBrand<crate::brands::BoxBrand, Out, A>, Idx>(
				effect,
			)
		}
	}
}

fn run_explicit_run_coroutine_tokens() -> TokenStream {
	quote! {
		/// Interprets Coroutine by returning the current coroutine status.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters(
			"The yielded output type.",
			"The resume input type.",
			"The type-level Member-position witness for the Coroutine effect.",
			"The first-order row brand with the Coroutine effect removed."
		)]
		#[document_returns("A first-order-only `RunExplicit` program returning the next Coroutine status.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::{
		/// 		coroutine::RunExplicitCoroutineStatus,
		/// 		run_explicit::RunExplicit,
		/// 	},
		/// };
		///
		/// type Row = CoproductBrand<CoyonedaBrand<BoxCoroutineBrand<BoxBrand, &'static str, i32>>, CNilBrand>;
		///
		/// let program: RunExplicit<'static, Row, CNilBrand, i32> =
		/// 	RunExplicit::yield_value::<&'static str, _>("next");
		/// let status = program.run_coroutine::<&'static str, i32, _, CNilBrand>().extract();
		/// assert!(matches!(status, RunExplicitCoroutineStatus::Continue("next", _)));
		/// ```
		#[inline]
		pub fn run_coroutine<Out, In, Idx, RMinusCoroutine>(
			self
		) -> RunExplicit<
			'a,
			RMinusCoroutine,
			CNilBrand,
			crate::types::effects::coroutine::RunExplicitCoroutineStatus<
				'a,
				RMinusCoroutine,
				CNilBrand,
				Out,
				In,
				A,
			>,
		>
		where
			Out: 'static + 'a,
			In: 'static + 'a,
			RMinusCoroutine: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RunExplicit<
					'a,
					R,
					CNilBrand,
					crate::types::effects::coroutine::RunExplicitCoroutineStatus<
						'a,
						RMinusCoroutine,
						CNilBrand,
						Out,
						In,
						A,
					>,
				>,
			>): crate::types::effects::member::Member<
				crate::types::Coyoneda<
					'a,
					crate::brands::BoxCoroutineBrand<crate::brands::BoxBrand, Out, In>,
					RunExplicit<
						'a,
						R,
						CNilBrand,
						crate::types::effects::coroutine::RunExplicitCoroutineStatus<
							'a,
							RMinusCoroutine,
							CNilBrand,
							Out,
							In,
							A,
						>,
					>,
				>,
				Idx,
				Remainder = Apply!(
					<RMinusCoroutine as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
						'a,
						RunExplicit<
							'a,
							R,
							CNilBrand,
							crate::types::effects::coroutine::RunExplicitCoroutineStatus<
								'a,
								RMinusCoroutine,
								CNilBrand,
								Out,
								In,
								A,
							>,
						>,
					>
				),
			>, {
			type Status<'a, RMinusCoroutine, Out, In, A> =
				crate::types::effects::coroutine::RunExplicitCoroutineStatus<
					'a,
					RMinusCoroutine,
					CNilBrand,
					Out,
					In,
					A,
				>;
			self.map(Status::<RMinusCoroutine, Out, In, A>::Done)
				.handle_with::<
					crate::brands::BoxCoroutineBrand<crate::brands::BoxBrand, Out, In>,
					Idx,
					RMinusCoroutine,
				>(
					|op: crate::types::effects::coroutine::BoxCoroutine<
						'a,
						crate::brands::BoxBrand,
						Out,
						In,
						RunExplicit<'a, RMinusCoroutine, CNilBrand, Status<RMinusCoroutine, Out, In, A>>,
					>| match op {
						crate::types::effects::coroutine::BoxCoroutine::Yield(out, resume) =>
							RunExplicit::pure(Status::Continue(out, resume)),
					},
				)
		}
	}
}

fn rcrun_explicit_yield_value_tokens() -> TokenStream {
	quote! {
		/// Lifts a Coroutine yield into the `RcRunExplicit` program.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters(
			"The yielded output type.",
			"The type-level Member-position witness for the Coroutine effect."
		)]
		#[document_parameters("The output value yielded to the coroutine runner.")]
		#[document_returns("An `RcRunExplicit` program suspended at the lifted Coroutine effect.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::rc_run_explicit::RcRunExplicit,
		/// };
		///
		/// type Row = CoproductBrand<RcCoyonedaBrand<CoroutineBrand<RcBrand, &'static str, i32>>, CNilBrand>;
		///
		/// let program: RcRunExplicit<'static, Row, CNilBrand, i32> =
		/// 	RcRunExplicit::yield_value::<&'static str, _>("next");
		/// let status = program.run_coroutine::<&'static str, i32, _, CNilBrand>().extract();
		/// assert!(matches!(status, fp_library::types::effects::coroutine::RcRunExplicitCoroutineStatus::Continue("next", _)));
		/// ```
		#[inline]
		pub fn yield_value<Out, Idx>(out: Out) -> Self
		where
			A: Clone + 'static,
			Out: Clone + 'static + 'a,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>): Member<
				RcCoyoneda<'a, crate::brands::CoroutineBrand<crate::brands::RcBrand, Out, A>, A>,
				Idx,
			>, {
			let effect: crate::types::effects::coroutine::Coroutine<
				'a,
				crate::brands::RcBrand,
				Out,
				A,
				A,
			> = crate::types::effects::coroutine::Coroutine::Yield(
				out,
				<crate::brands::RcBrand as crate::classes::ToDynCloneFn>::new(|input: A| input),
			);
			Self::lift::<crate::brands::CoroutineBrand<crate::brands::RcBrand, Out, A>, Idx>(
				effect,
			)
		}
	}
}

fn rcrun_explicit_run_coroutine_tokens() -> TokenStream {
	quote! {
		/// Interprets Coroutine by returning the current coroutine status.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters(
			"The yielded output type.",
			"The resume input type.",
			"The type-level Member-position witness for the Coroutine effect.",
			"The first-order row brand with the Coroutine effect removed."
		)]
		#[document_returns("A first-order-only `RcRunExplicit` program returning the next Coroutine status.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::{
		/// 		coroutine::RcRunExplicitCoroutineStatus,
		/// 		rc_run_explicit::RcRunExplicit,
		/// 	},
		/// };
		///
		/// type Row = CoproductBrand<RcCoyonedaBrand<CoroutineBrand<RcBrand, &'static str, i32>>, CNilBrand>;
		///
		/// let program: RcRunExplicit<'static, Row, CNilBrand, i32> =
		/// 	RcRunExplicit::yield_value::<&'static str, _>("next");
		/// let status = program.run_coroutine::<&'static str, i32, _, CNilBrand>().extract();
		/// assert!(matches!(status, RcRunExplicitCoroutineStatus::Continue("next", _)));
		/// ```
		#[inline]
		pub fn run_coroutine<Out, In, Idx, RMinusCoroutine>(
			self
		) -> RcRunExplicit<
			'a,
			RMinusCoroutine,
			CNilBrand,
			crate::types::effects::coroutine::RcRunExplicitCoroutineStatus<
				'a,
				RMinusCoroutine,
				CNilBrand,
				Out,
				In,
				A,
			>,
		>
		where
			A: Clone + 'a,
			Out: Clone + 'static + 'a,
			In: 'static + 'a,
			RMinusCoroutine: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
			Apply!(<NodeBrand<R, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, CNilBrand>, A>,
			>): Clone,
			Apply!(<NodeBrand<R, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<
					'a,
					NodeBrand<R, CNilBrand>,
					crate::types::effects::coroutine::RcRunExplicitCoroutineStatus<
						'a,
						RMinusCoroutine,
						CNilBrand,
						Out,
						In,
						A,
					>,
				>,
			>): Clone,
			Apply!(<NodeBrand<RMinusCoroutine, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<
					'a,
					NodeBrand<RMinusCoroutine, CNilBrand>,
					crate::types::effects::coroutine::RcRunExplicitCoroutineStatus<
						'a,
						RMinusCoroutine,
						CNilBrand,
						Out,
						In,
						A,
					>,
				>,
			>): Clone,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcRunExplicit<
					'a,
					R,
					CNilBrand,
					crate::types::effects::coroutine::RcRunExplicitCoroutineStatus<
						'a,
						RMinusCoroutine,
						CNilBrand,
						Out,
						In,
						A,
					>,
				>,
			>): Member<
				RcCoyoneda<
					'a,
					crate::brands::CoroutineBrand<crate::brands::RcBrand, Out, In>,
					RcRunExplicit<
						'a,
						R,
						CNilBrand,
						crate::types::effects::coroutine::RcRunExplicitCoroutineStatus<
							'a,
							RMinusCoroutine,
							CNilBrand,
							Out,
							In,
							A,
						>,
					>,
				>,
				Idx,
				Remainder = Apply!(
					<RMinusCoroutine as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
						'a,
						RcRunExplicit<
							'a,
							R,
							CNilBrand,
							crate::types::effects::coroutine::RcRunExplicitCoroutineStatus<
								'a,
								RMinusCoroutine,
								CNilBrand,
								Out,
								In,
								A,
							>,
						>,
					>
				),
			>, {
			type Status<'a, RMinusCoroutine, Out, In, A> =
				crate::types::effects::coroutine::RcRunExplicitCoroutineStatus<
					'a,
					RMinusCoroutine,
					CNilBrand,
					Out,
					In,
					A,
				>;
			self.map(Status::<RMinusCoroutine, Out, In, A>::Done)
				.handle_with::<
					crate::brands::CoroutineBrand<crate::brands::RcBrand, Out, In>,
					Idx,
					RMinusCoroutine,
				>(
					|op: crate::types::effects::coroutine::Coroutine<
						'a,
						crate::brands::RcBrand,
						Out,
						In,
						RcRunExplicit<'a, RMinusCoroutine, CNilBrand, Status<RMinusCoroutine, Out, In, A>>,
					>| match op {
						crate::types::effects::coroutine::Coroutine::Yield(out, resume) =>
							RcRunExplicit::pure(Status::Continue(out, resume)),
					},
				)
		}
	}
}

fn arcrun_explicit_yield_value_tokens() -> TokenStream {
	quote! {
		/// Lifts a Coroutine yield into the `ArcRunExplicit` program.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters(
			"The yielded output type.",
			"The type-level Member-position witness for the Coroutine effect."
		)]
		#[document_parameters("The output value yielded to the coroutine runner.")]
		#[document_returns("An `ArcRunExplicit` program suspended at the lifted Coroutine effect.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::arc_run_explicit::ArcRunExplicit,
		/// };
		///
		/// type Row = CoproductBrand<ArcCoyonedaBrand<SendCoroutineBrand<ArcBrand, &'static str, i32>>, CNilBrand>;
		///
		/// let program: ArcRunExplicit<'static, Row, CNilBrand, i32> =
		/// 	ArcRunExplicit::yield_value::<&'static str, _>("next");
		/// let status = program.run_coroutine::<&'static str, i32, _, CNilBrand>().extract();
		/// assert!(matches!(status, fp_library::types::effects::coroutine::ArcRunExplicitCoroutineStatus::Continue("next", _)));
		/// ```
		#[inline]
		pub fn yield_value<Out, Idx>(out: Out) -> Self
		where
			A: Clone + Send + Sync + 'static,
			Out: Clone + Send + Sync + 'static + 'a,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>): Member<
				ArcCoyoneda<
					'a,
					crate::brands::SendCoroutineBrand<crate::brands::ArcBrand, Out, A>,
					A,
				>,
				Idx,
			>,
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
			let effect: crate::types::effects::coroutine::SendCoroutine<
				'a,
				crate::brands::ArcBrand,
				Out,
				A,
				A,
			> = crate::types::effects::coroutine::SendCoroutine::Yield(
				out,
				<crate::brands::ArcBrand as crate::classes::ToDynSendFn>::new(|input: A| input),
			);
			Self::lift::<crate::brands::SendCoroutineBrand<crate::brands::ArcBrand, Out, A>, Idx>(
				effect,
			)
		}
	}
}

fn arcrun_explicit_run_coroutine_tokens() -> TokenStream {
	quote! {
		/// Interprets Coroutine by returning the current coroutine status.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters(
			"The yielded output type.",
			"The resume input type.",
			"The type-level Member-position witness for the Coroutine effect.",
			"The first-order row brand with the Coroutine effect removed."
		)]
		#[document_returns("A first-order-only `ArcRunExplicit` program returning the next Coroutine status.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::{
		/// 		arc_run_explicit::ArcRunExplicit,
		/// 		coroutine::ArcRunExplicitCoroutineStatus,
		/// 	},
		/// };
		///
		/// type Row = CoproductBrand<ArcCoyonedaBrand<SendCoroutineBrand<ArcBrand, &'static str, i32>>, CNilBrand>;
		///
		/// let program: ArcRunExplicit<'static, Row, CNilBrand, i32> =
		/// 	ArcRunExplicit::yield_value::<&'static str, _>("next");
		/// let status = program.run_coroutine::<&'static str, i32, _, CNilBrand>().extract();
		/// assert!(matches!(status, ArcRunExplicitCoroutineStatus::Continue("next", _)));
		/// ```
		#[inline]
		pub fn run_coroutine<Out, In, Idx, RMinusCoroutine>(
			self
		) -> ArcRunExplicit<
			'a,
			RMinusCoroutine,
			CNilBrand,
			crate::types::effects::coroutine::ArcRunExplicitCoroutineStatus<
				'a,
				RMinusCoroutine,
				CNilBrand,
				Out,
				In,
				A,
			>,
		>
		where
			A: Clone + Send + Sync + 'a,
			Out: Clone + Send + Sync + 'static + 'a,
			In: Send + Sync + 'static + 'a,
			RMinusCoroutine: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
			NodeBrand<R, CNilBrand>: SendFunctor,
			NodeBrand<RMinusCoroutine, CNilBrand>: SendFunctor,
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
			Apply!(<NodeBrand<R, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<
					'a,
					NodeBrand<R, CNilBrand>,
					crate::types::effects::coroutine::ArcRunExplicitCoroutineStatus<
						'a,
						RMinusCoroutine,
						CNilBrand,
						Out,
						In,
						A,
					>,
				>,
			>): Clone + Send + Sync,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<
					'a,
					NodeBrand<R, CNilBrand>,
					crate::types::effects::coroutine::ArcRunExplicitCoroutineStatus<
						'a,
						RMinusCoroutine,
						CNilBrand,
						Out,
						In,
						A,
					>,
				>,
			>): Send + Sync,
			Apply!(<CNilBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<
					'a,
					NodeBrand<R, CNilBrand>,
					crate::types::effects::coroutine::ArcRunExplicitCoroutineStatus<
						'a,
						RMinusCoroutine,
						CNilBrand,
						Out,
						In,
						A,
					>,
				>,
			>): Send + Sync,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<
					'a,
					R,
					CNilBrand,
					crate::types::effects::coroutine::ArcRunExplicitCoroutineStatus<
						'a,
						RMinusCoroutine,
						CNilBrand,
						Out,
						In,
						A,
					>,
				>,
			>): Send + Sync,
			Apply!(<CNilBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<
					'a,
					R,
					CNilBrand,
					crate::types::effects::coroutine::ArcRunExplicitCoroutineStatus<
						'a,
						RMinusCoroutine,
						CNilBrand,
						Out,
						In,
						A,
					>,
				>,
			>): Send + Sync,
			Apply!(<NodeBrand<RMinusCoroutine, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<
					'a,
					NodeBrand<RMinusCoroutine, CNilBrand>,
					crate::types::effects::coroutine::ArcRunExplicitCoroutineStatus<
						'a,
						RMinusCoroutine,
						CNilBrand,
						Out,
						In,
						A,
					>,
				>,
			>): Clone + Send + Sync,
			Apply!(<RMinusCoroutine as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<
					'a,
					NodeBrand<RMinusCoroutine, CNilBrand>,
					crate::types::effects::coroutine::ArcRunExplicitCoroutineStatus<
						'a,
						RMinusCoroutine,
						CNilBrand,
						Out,
						In,
						A,
					>,
				>,
			>): Send + Sync,
			Apply!(<CNilBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<
					'a,
					NodeBrand<RMinusCoroutine, CNilBrand>,
					crate::types::effects::coroutine::ArcRunExplicitCoroutineStatus<
						'a,
						RMinusCoroutine,
						CNilBrand,
						Out,
						In,
						A,
					>,
				>,
			>): Send + Sync,
			Apply!(<RMinusCoroutine as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<
					'a,
					RMinusCoroutine,
					CNilBrand,
					crate::types::effects::coroutine::ArcRunExplicitCoroutineStatus<
						'a,
						RMinusCoroutine,
						CNilBrand,
						Out,
						In,
						A,
					>,
				>,
			>): Send + Sync,
			Apply!(<CNilBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<
					'a,
					RMinusCoroutine,
					CNilBrand,
					crate::types::effects::coroutine::ArcRunExplicitCoroutineStatus<
						'a,
						RMinusCoroutine,
						CNilBrand,
						Out,
						In,
						A,
					>,
				>,
			>): Send + Sync,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<
					'a,
					R,
					CNilBrand,
					crate::types::effects::coroutine::ArcRunExplicitCoroutineStatus<
						'a,
						RMinusCoroutine,
						CNilBrand,
						Out,
						In,
						A,
					>,
				>,
			>): Member<
				ArcCoyoneda<
					'a,
					crate::brands::SendCoroutineBrand<crate::brands::ArcBrand, Out, In>,
					ArcRunExplicit<
						'a,
						R,
						CNilBrand,
						crate::types::effects::coroutine::ArcRunExplicitCoroutineStatus<
							'a,
							RMinusCoroutine,
							CNilBrand,
							Out,
							In,
							A,
						>,
					>,
				>,
				Idx,
				Remainder = Apply!(
					<RMinusCoroutine as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
						'a,
						ArcRunExplicit<
							'a,
							R,
							CNilBrand,
							crate::types::effects::coroutine::ArcRunExplicitCoroutineStatus<
								'a,
								RMinusCoroutine,
								CNilBrand,
								Out,
								In,
								A,
							>,
						>,
					>
				),
			>, {
			type Status<'a, RMinusCoroutine, Out, In, A> =
				crate::types::effects::coroutine::ArcRunExplicitCoroutineStatus<
					'a,
					RMinusCoroutine,
					CNilBrand,
					Out,
					In,
					A,
				>;
			self.map(Status::<RMinusCoroutine, Out, In, A>::Done)
				.handle_with::<
					crate::brands::SendCoroutineBrand<crate::brands::ArcBrand, Out, In>,
					Idx,
					RMinusCoroutine,
				>(
					|op: crate::types::effects::coroutine::SendCoroutine<
						'a,
						crate::brands::ArcBrand,
						Out,
						In,
						ArcRunExplicit<'a, RMinusCoroutine, CNilBrand, Status<RMinusCoroutine, Out, In, A>>,
					>| match op {
						crate::types::effects::coroutine::SendCoroutine::Yield(out, resume) =>
							ArcRunExplicit::pure(Status::Continue(out, resume)),
					},
				)
		}
	}
}
