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

pub(super) fn fixed_message_abort_wrapper_impl_items_from_descriptor(
	wrapper: WrapperName,
	method: RunWrapperMethod,
) -> Option<syn::Result<Vec<ImplItem>>> {
	generator_descriptors::method_spec(EffectName::Fail, method)?;
	generator_descriptors::wrapper_spec(wrapper)?;

	let tokens = match (wrapper, method) {
		(WrapperName::Run, RunWrapperMethod::Fail) => run_fail_constructor_tokens(),
		(WrapperName::Run, RunWrapperMethod::RunFail) => run_run_fail_tokens(),
		(WrapperName::RcRun, RunWrapperMethod::Fail) => rcrun_fail_constructor_tokens(),
		(WrapperName::RcRun, RunWrapperMethod::RunFail) => rcrun_run_fail_tokens(),
		(WrapperName::ArcRun, RunWrapperMethod::Fail) => arcrun_fail_constructor_tokens(),
		(WrapperName::ArcRun, RunWrapperMethod::RunFail) => arcrun_run_fail_tokens(),
		(WrapperName::RunExplicit, RunWrapperMethod::Fail) =>
			run_explicit_fail_constructor_tokens(),
		(WrapperName::RunExplicit, RunWrapperMethod::RunFail) => run_explicit_run_fail_tokens(),
		(WrapperName::RcRunExplicit, RunWrapperMethod::Fail) =>
			rcrun_explicit_fail_constructor_tokens(),
		(WrapperName::RcRunExplicit, RunWrapperMethod::RunFail) => rcrun_explicit_run_fail_tokens(),
		(WrapperName::ArcRunExplicit, RunWrapperMethod::Fail) =>
			arcrun_explicit_fail_constructor_tokens(),
		(WrapperName::ArcRunExplicit, RunWrapperMethod::RunFail) =>
			arcrun_explicit_run_fail_tokens(),
		_ => return None,
	};

	Some(impl_items_from_tokens(tokens))
}

fn run_fail_constructor_tokens() -> TokenStream {
	quote! {
		/// Lifts a fixed-message Fail effect into the `Run` program.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters("The type-level Member-position witness for the Fail effect.")]
		#[document_parameters("The failure message.")]
		#[document_returns("A `Run` program suspended at the lifted Fail effect.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::run::Run,
		/// };
		///
		/// type Row = CoproductBrand<CoyonedaBrand<FailBrand>, CNilBrand>;
		///
		/// let program: Run<Row, CNilBrand, i32> = Run::fail("missing");
		/// let handled: Run<CNilBrand, CNilBrand, Result<i32, String>> =
		/// 	program.run_fail::<_, CNilBrand>();
		/// assert_eq!(handled.extract(), Err(String::from("missing")));
		/// ```
		#[inline]
		pub fn fail<Idx>(message: impl Into<String>) -> Self
		where
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, A>):
				crate::types::effects::member::Member<
					crate::types::Coyoneda<'static, crate::brands::FailBrand, A>,
					Idx,
				>, {
			let effect: crate::types::effects::fail::Fail<'static, A> =
				crate::types::effects::fail::Fail::Fail(
					message.into(),
					core::marker::PhantomData,
				);
			Self::lift::<crate::brands::FailBrand, Idx>(effect)
		}
	}
}

fn rcrun_fail_constructor_tokens() -> TokenStream {
	quote! {
		/// Lifts a fixed-message Fail effect into the `RcRun` program.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters("The type-level Member-position witness for the Fail effect.")]
		#[document_parameters("The failure message.")]
		#[document_returns("An `RcRun` program suspended at the lifted Fail effect.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::rc_run::RcRun,
		/// };
		///
		/// type Row = CoproductBrand<RcCoyonedaBrand<FailBrand>, CNilBrand>;
		///
		/// let program: RcRun<Row, CNilBrand, i32> = RcRun::fail("missing");
		/// let handled: RcRun<CNilBrand, CNilBrand, Result<i32, String>> =
		/// 	program.run_fail::<_, CNilBrand>();
		/// assert_eq!(handled.extract(), Err(String::from("missing")));
		/// ```
		#[inline]
		pub fn fail<Idx>(message: impl Into<String>) -> Self
		where
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, A>):
				Member<RcCoyoneda<'static, crate::brands::FailBrand, A>, Idx>,
			Apply!(<NodeBrand<R, ScopedRow> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<NodeBrand<R, ScopedRow>, crate::types::rc_free::RcTypeErasedValue>,
			>): Clone, {
			let effect: crate::types::effects::fail::Fail<'static, A> =
				crate::types::effects::fail::Fail::Fail(
					message.into(),
					core::marker::PhantomData,
				);
			Self::lift::<crate::brands::FailBrand, Idx>(effect)
		}
	}
}

fn arcrun_fail_constructor_tokens() -> TokenStream {
	quote! {
		/// Lifts a fixed-message Fail effect into the `ArcRun` program.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters("The type-level Member-position witness for the Fail effect.")]
		#[document_parameters("The failure message.")]
		#[document_returns("An `ArcRun` program suspended at the lifted Fail effect.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::arc_run::ArcRun,
		/// };
		///
		/// type Row = CoproductBrand<ArcCoyonedaBrand<FailBrand>, CNilBrand>;
		///
		/// let program: ArcRun<Row, CNilBrand, i32> = ArcRun::fail("missing");
		/// let handled: ArcRun<CNilBrand, CNilBrand, Result<i32, String>> =
		/// 	program.run_fail::<_, CNilBrand>();
		/// assert_eq!(handled.extract(), Err(String::from("missing")));
		/// ```
		#[inline]
		pub fn fail<Idx>(message: impl Into<String>) -> Self
		where
			A: Send + Sync,
			NodeBrand<R, ScopedRow>: SendFunctor,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, A>):
				Member<ArcCoyoneda<'static, crate::brands::FailBrand, A>, Idx>,
			Apply!(<NodeBrand<R, ScopedRow> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<NodeBrand<R, ScopedRow>, ArcTypeErasedValue>,
			>): Clone, {
			let effect: crate::types::effects::fail::Fail<'static, A> =
				crate::types::effects::fail::Fail::Fail(
					message.into(),
					core::marker::PhantomData,
				);
			Self::lift::<crate::brands::FailBrand, Idx>(effect)
		}
	}
}

fn run_explicit_fail_constructor_tokens() -> TokenStream {
	quote! {
		/// Lifts a fixed-message Fail effect into the `RunExplicit` program.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters("The type-level Member-position witness for the Fail effect.")]
		#[document_parameters("The failure message.")]
		#[document_returns("A `RunExplicit` program suspended at the lifted Fail effect.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::run_explicit::RunExplicit,
		/// };
		///
		/// type Row = CoproductBrand<CoyonedaBrand<FailBrand>, CNilBrand>;
		///
		/// let program: RunExplicit<'static, Row, CNilBrand, i32> = RunExplicit::fail("missing");
		/// let handled: RunExplicit<'static, CNilBrand, CNilBrand, Result<i32, String>> =
		/// 	program.run_fail::<_, CNilBrand>();
		/// assert_eq!(handled.extract(), Err(String::from("missing")));
		/// ```
		#[inline]
		pub fn fail<Idx>(message: impl Into<String>) -> Self
		where
			A: 'static,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>):
				Member<Coyoneda<'a, crate::brands::FailBrand, A>, Idx>, {
			let effect: crate::types::effects::fail::Fail<'a, A> =
				crate::types::effects::fail::Fail::Fail(
					message.into(),
					core::marker::PhantomData,
				);
			Self::lift::<crate::brands::FailBrand, Idx>(effect)
		}
	}
}

fn rcrun_explicit_fail_constructor_tokens() -> TokenStream {
	quote! {
		/// Lifts a fixed-message Fail effect into the `RcRunExplicit` program.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters("The type-level Member-position witness for the Fail effect.")]
		#[document_parameters("The failure message.")]
		#[document_returns("An `RcRunExplicit` program suspended at the lifted Fail effect.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::rc_run_explicit::RcRunExplicit,
		/// };
		///
		/// type Row = CoproductBrand<RcCoyonedaBrand<FailBrand>, CNilBrand>;
		///
		/// let program: RcRunExplicit<'static, Row, CNilBrand, i32> =
		/// 	RcRunExplicit::fail("missing");
		/// let handled: RcRunExplicit<'static, CNilBrand, CNilBrand, Result<i32, String>> =
		/// 	program.run_fail::<_, CNilBrand>();
		/// assert_eq!(handled.extract(), Err(String::from("missing")));
		/// ```
		#[inline]
		pub fn fail<Idx>(message: impl Into<String>) -> Self
		where
			A: Clone + 'static,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>):
				Member<RcCoyoneda<'a, crate::brands::FailBrand, A>, Idx>, {
			let effect: crate::types::effects::fail::Fail<'a, A> =
				crate::types::effects::fail::Fail::Fail(
					message.into(),
					core::marker::PhantomData,
				);
			Self::lift::<crate::brands::FailBrand, Idx>(effect)
		}
	}
}

fn arcrun_explicit_fail_constructor_tokens() -> TokenStream {
	quote! {
		/// Lifts a fixed-message Fail effect into the `ArcRunExplicit` program.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters("The type-level Member-position witness for the Fail effect.")]
		#[document_parameters("The failure message.")]
		#[document_returns("An `ArcRunExplicit` program suspended at the lifted Fail effect.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::arc_run_explicit::ArcRunExplicit,
		/// };
		///
		/// type Row = CoproductBrand<ArcCoyonedaBrand<FailBrand>, CNilBrand>;
		///
		/// let program: ArcRunExplicit<'static, Row, CNilBrand, i32> =
		/// 	ArcRunExplicit::fail("missing");
		/// let handled: ArcRunExplicit<'static, CNilBrand, CNilBrand, Result<i32, String>> =
		/// 	program.run_fail::<_, CNilBrand>();
		/// assert_eq!(handled.extract(), Err(String::from("missing")));
		/// ```
		#[inline]
		pub fn fail<Idx>(message: impl Into<String>) -> Self
		where
			A: Clone + Send + Sync + 'static,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>):
				Member<ArcCoyoneda<'a, crate::brands::FailBrand, A>, Idx>,
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
			let effect: crate::types::effects::fail::Fail<'a, A> =
				crate::types::effects::fail::Fail::Fail(
					message.into(),
					core::marker::PhantomData,
				);
			Self::lift::<crate::brands::FailBrand, Idx>(effect)
		}
	}
}

fn run_run_fail_tokens() -> TokenStream {
	quote! {
		/// Interprets one Fail effect into a Rust `Result`.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters(
			"The type-level Member-position witness for the Fail effect.",
			"The first-order row brand with the Fail effect removed."
		)]
		#[document_returns("A first-order-only `Run` program returning `Ok(result)` or `Err(message)`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::run::Run,
		/// };
		///
		/// type Row = CoproductBrand<CoyonedaBrand<FailBrand>, CNilBrand>;
		///
		/// let program: Run<Row, CNilBrand, i32> = Run::fail("missing");
		/// let handled: Run<CNilBrand, CNilBrand, Result<i32, String>> =
		/// 	program.run_fail::<_, CNilBrand>();
		/// assert_eq!(handled.extract(), Err(String::from("missing")));
		/// ```
		#[inline]
		pub fn run_fail<Idx, RMinusFail>(self) -> Run<RMinusFail, CNilBrand, Result<A, String>>
		where
			RMinusFail: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				Run<R, CNilBrand, Result<A, String>>,
			>): Member<
					Coyoneda<'static, FailBrand, Run<R, CNilBrand, Result<A, String>>>,
					Idx,
					Remainder = Apply!(
									<RMinusFail as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
										'static,
										Run<R, CNilBrand, Result<A, String>>,
									>
								),
				>, {
			self.map(Ok).handle_with::<FailBrand, Idx, RMinusFail>(
				|op: Fail<'static, Run<RMinusFail, CNilBrand, Result<A, String>>>| match op {
					Fail::Fail(message, _) => Run::pure(Err(message)),
				},
			)
		}
	}
}

fn rcrun_run_fail_tokens() -> TokenStream {
	quote! {
		/// Interprets one Fail effect into a Rust `Result`.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters(
			"The type-level Member-position witness for the Fail effect.",
			"The first-order row brand with the Fail effect removed."
		)]
		#[document_returns(
			"A first-order-only `RcRun` program returning `Ok(result)` or `Err(message)`."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::rc_run::RcRun,
		/// };
		///
		/// type Row = CoproductBrand<RcCoyonedaBrand<FailBrand>, CNilBrand>;
		///
		/// let program: RcRun<Row, CNilBrand, i32> = RcRun::fail("missing");
		/// let handled: RcRun<CNilBrand, CNilBrand, Result<i32, String>> =
		/// 	program.run_fail::<_, CNilBrand>();
		/// assert_eq!(handled.extract(), Err(String::from("missing")));
		/// ```
		#[inline]
		pub fn run_fail<Idx, RMinusFail>(self) -> RcRun<RMinusFail, CNilBrand, Result<A, String>>
		where
			A: Clone,
			RMinusFail: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
			Apply!(<NodeBrand<R, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<NodeBrand<R, CNilBrand>, RcTypeErasedValue>,
			>): Clone,
			Apply!(<NodeBrand<RMinusFail, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<NodeBrand<RMinusFail, CNilBrand>, RcTypeErasedValue>,
			>): Clone,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcRun<R, CNilBrand, Result<A, String>>,
			>): Member<
					RcCoyoneda<'static, FailBrand, RcRun<R, CNilBrand, Result<A, String>>>,
					Idx,
					Remainder = Apply!(
									<RMinusFail as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
										'static,
										RcRun<R, CNilBrand, Result<A, String>>,
									>
								),
				>, {
			self.map(Ok).handle_with::<FailBrand, Idx, RMinusFail>(
				|op: Fail<'static, RcRun<RMinusFail, CNilBrand, Result<A, String>>>| match op {
					Fail::Fail(message, _) => RcRun::pure(Err(message)),
				},
			)
		}
	}
}

fn arcrun_run_fail_tokens() -> TokenStream {
	quote! {
		/// Interprets one Fail effect into a Rust `Result`.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters(
			"The type-level Member-position witness for the Fail effect.",
			"The first-order row brand with the Fail effect removed."
		)]
		#[document_returns(
			"A first-order-only `ArcRun` program returning `Ok(result)` or `Err(message)`."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::arc_run::ArcRun,
		/// };
		///
		/// type Row = CoproductBrand<ArcCoyonedaBrand<FailBrand>, CNilBrand>;
		///
		/// let program: ArcRun<Row, CNilBrand, i32> = ArcRun::fail("missing");
		/// let handled: ArcRun<CNilBrand, CNilBrand, Result<i32, String>> =
		/// 	program.run_fail::<_, CNilBrand>();
		/// assert_eq!(handled.extract(), Err(String::from("missing")));
		/// ```
		#[inline]
		pub fn run_fail<Idx, RMinusFail>(self) -> ArcRun<RMinusFail, CNilBrand, Result<A, String>>
		where
			A: Clone + Send + Sync,
			R: Kind_cdc7cd43dac7585f + 'static,
			RMinusFail: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
			NodeBrand<R, CNilBrand>: SendFunctor,
			NodeBrand<RMinusFail, CNilBrand>: WrapDrop
				+ Kind_cdc7cd43dac7585f<
					Of<'static, ArcFree<NodeBrand<RMinusFail, CNilBrand>, ArcTypeErasedValue>>:
						Send + Sync,
				> + SendFunctor,
			Apply!(<NodeBrand<R, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<NodeBrand<R, CNilBrand>, ArcTypeErasedValue>,
			>): Clone,
			Apply!(<NodeBrand<RMinusFail, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<NodeBrand<RMinusFail, CNilBrand>, ArcTypeErasedValue>,
			>): Clone,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcRun<R, CNilBrand, Result<A, String>>,
			>): Member<
					ArcCoyoneda<'static, FailBrand, ArcRun<R, CNilBrand, Result<A, String>>>,
					Idx,
					Remainder = Apply!(
									<RMinusFail as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
										'static,
										ArcRun<R, CNilBrand, Result<A, String>>,
									>
								),
				>, {
			self.map(Ok).handle_with::<FailBrand, Idx, RMinusFail>(
				|op: Fail<'static, ArcRun<RMinusFail, CNilBrand, Result<A, String>>>| match op {
					Fail::Fail(message, _) => ArcRun::pure(Err(message)),
				},
			)
		}
	}
}

fn run_explicit_run_fail_tokens() -> TokenStream {
	quote! {
		/// Interprets one Fail effect into a Rust `Result`.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters(
			"The type-level Member-position witness for the Fail effect.",
			"The first-order row brand with the Fail effect removed."
		)]
		#[document_returns(
			"A first-order-only `RunExplicit` program returning `Ok(result)` or `Err(message)`."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::run_explicit::RunExplicit,
		/// };
		///
		/// type Row = CoproductBrand<CoyonedaBrand<FailBrand>, CNilBrand>;
		///
		/// let program: RunExplicit<'static, Row, CNilBrand, i32> = RunExplicit::fail("missing");
		/// let handled: RunExplicit<'static, CNilBrand, CNilBrand, Result<i32, String>> =
		/// 	program.run_fail::<_, CNilBrand>();
		/// assert_eq!(handled.extract(), Err(String::from("missing")));
		/// ```
		#[inline]
		pub fn run_fail<Idx, RMinusFail>(
			self
		) -> RunExplicit<'a, RMinusFail, CNilBrand, Result<A, String>>
		where
			RMinusFail: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RunExplicit<'a, R, CNilBrand, Result<A, String>>,
			>): Member<
					Coyoneda<'a, FailBrand, RunExplicit<'a, R, CNilBrand, Result<A, String>>>,
					Idx,
					Remainder = Apply!(
									<RMinusFail as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
										'a,
										RunExplicit<'a, R, CNilBrand, Result<A, String>>,
									>
								),
				>, {
			self.map(Ok).handle_with::<FailBrand, Idx, RMinusFail>(
				|op: Fail<'a, RunExplicit<'a, RMinusFail, CNilBrand, Result<A, String>>>| {
					match op {
						Fail::Fail(message, _) => RunExplicit::pure(Err(message)),
					}
				},
			)
		}
	}
}

fn rcrun_explicit_run_fail_tokens() -> TokenStream {
	quote! {
		/// Interprets one Fail effect into a Rust `Result`.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters(
			"The type-level Member-position witness for the Fail effect.",
			"The first-order row brand with the Fail effect removed."
		)]
		#[document_returns(
			"A first-order-only `RcRunExplicit` program returning `Ok(result)` or `Err(message)`."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::rc_run_explicit::RcRunExplicit,
		/// };
		///
		/// type Row = CoproductBrand<RcCoyonedaBrand<FailBrand>, CNilBrand>;
		///
		/// let program: RcRunExplicit<'static, Row, CNilBrand, i32> =
		/// 	RcRunExplicit::fail("missing");
		/// let handled: RcRunExplicit<'static, CNilBrand, CNilBrand, Result<i32, String>> =
		/// 	program.run_fail::<_, CNilBrand>();
		/// assert_eq!(handled.extract(), Err(String::from("missing")));
		/// ```
		#[inline]
		pub fn run_fail<Idx, RMinusFail>(
			self
		) -> RcRunExplicit<'a, RMinusFail, CNilBrand, Result<A, String>>
		where
			A: Clone,
			RMinusFail: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
			Apply!(<NodeBrand<R, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, CNilBrand>, A>,
			>): Clone,
			Apply!(<NodeBrand<R, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, CNilBrand>, Result<A, String>>,
			>): Clone,
			Apply!(<NodeBrand<RMinusFail, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<RMinusFail, CNilBrand>, Result<A, String>>,
			>): Clone,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcRunExplicit<'a, R, CNilBrand, Result<A, String>>,
			>): Member<
					RcCoyoneda<
						'a,
						FailBrand,
						RcRunExplicit<'a, R, CNilBrand, Result<A, String>>,
					>,
					Idx,
					Remainder = Apply!(
									<RMinusFail as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
										'a,
										RcRunExplicit<'a, R, CNilBrand, Result<A, String>>,
									>
								),
				>, {
			self.map(Ok).handle_with::<FailBrand, Idx, RMinusFail>(
				|op: Fail<'a, RcRunExplicit<'a, RMinusFail, CNilBrand, Result<A, String>>>| {
					match op {
						Fail::Fail(message, _) => RcRunExplicit::pure(Err(message)),
					}
				},
			)
		}
	}
}

fn arcrun_explicit_run_fail_tokens() -> TokenStream {
	quote! {
		/// Interprets one Fail effect into a Rust `Result`.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters(
			"The type-level Member-position witness for the Fail effect.",
			"The first-order row brand with the Fail effect removed."
		)]
		#[document_returns(
			"A first-order-only `ArcRunExplicit` program returning `Ok(result)` or `Err(message)`."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::arc_run_explicit::ArcRunExplicit,
		/// };
		///
		/// type Row = CoproductBrand<ArcCoyonedaBrand<FailBrand>, CNilBrand>;
		///
		/// let program: ArcRunExplicit<'static, Row, CNilBrand, i32> =
		/// 	ArcRunExplicit::fail("missing");
		/// let handled: ArcRunExplicit<'static, CNilBrand, CNilBrand, Result<i32, String>> =
		/// 	program.run_fail::<_, CNilBrand>();
		/// assert_eq!(handled.extract(), Err(String::from("missing")));
		/// ```
		#[inline]
		pub fn run_fail<Idx, RMinusFail>(
			self
		) -> ArcRunExplicit<'a, RMinusFail, CNilBrand, Result<A, String>>
		where
			A: Clone + Send + Sync,
			RMinusFail: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
			NodeBrand<R, CNilBrand>: SendFunctor,
			NodeBrand<RMinusFail, CNilBrand>: SendFunctor,
			Apply!(<NodeBrand<R, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, CNilBrand>, A>,
			>): Clone + Send + Sync,
			Apply!(<NodeBrand<R, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, CNilBrand>, Result<A, String>>,
			>): Clone + Send + Sync,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, CNilBrand>, Result<A, String>>,
			>): Send + Sync,
			Apply!(<CNilBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, CNilBrand>, Result<A, String>>,
			>): Send + Sync,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, R, CNilBrand, Result<A, String>>,
			>): Send + Sync,
			Apply!(<CNilBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, R, CNilBrand, Result<A, String>>,
			>): Send + Sync,
			Apply!(<NodeBrand<RMinusFail, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<RMinusFail, CNilBrand>, Result<A, String>>,
			>): Clone + Send + Sync,
			Apply!(<RMinusFail as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<RMinusFail, CNilBrand>, Result<A, String>>,
			>): Send + Sync,
			Apply!(<CNilBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<RMinusFail, CNilBrand>, Result<A, String>>,
			>): Send + Sync,
			Apply!(<RMinusFail as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, RMinusFail, CNilBrand, Result<A, String>>,
			>): Send + Sync,
			Apply!(<CNilBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, RMinusFail, CNilBrand, Result<A, String>>,
			>): Send + Sync,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, R, CNilBrand, Result<A, String>>,
			>): Member<
					ArcCoyoneda<
						'a,
						FailBrand,
						ArcRunExplicit<'a, R, CNilBrand, Result<A, String>>,
					>,
					Idx,
					Remainder = Apply!(
									<RMinusFail as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
										'a,
										ArcRunExplicit<'a, R, CNilBrand, Result<A, String>>,
									>
								),
				>, {
			self.map(Ok).handle_with::<FailBrand, Idx, RMinusFail>(
				|op: Fail<'a, ArcRunExplicit<'a, RMinusFail, CNilBrand, Result<A, String>>>| {
					match op {
						Fail::Fail(message, _) => ArcRunExplicit::pure(Err(message)),
					}
				},
			)
		}
	}
}
