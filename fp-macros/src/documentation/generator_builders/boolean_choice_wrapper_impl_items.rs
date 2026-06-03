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

pub(super) fn boolean_choice_wrapper_impl_items_from_descriptor(
	wrapper: WrapperName,
	method: RunWrapperMethod,
) -> Option<syn::Result<Vec<ImplItem>>> {
	generator_descriptors::method_spec(EffectName::Choose, method)?;
	generator_descriptors::wrapper_spec(wrapper)?;

	let tokens = match (wrapper, method) {
		(WrapperName::RcRun, RunWrapperMethod::Choose) => rcrun_choose_constructor_tokens(),
		(WrapperName::RcRun, RunWrapperMethod::RunChoose) => rcrun_run_choose_tokens(),
		(WrapperName::RcRun, RunWrapperMethod::RunNondet) => rcrun_run_nondet_tokens(),
		(WrapperName::RcRun, RunWrapperMethod::RunFirstSuccess) => rcrun_run_first_success_tokens(),
		(WrapperName::ArcRun, RunWrapperMethod::Choose) => arcrun_choose_constructor_tokens(),
		(WrapperName::ArcRun, RunWrapperMethod::RunChoose) => arcrun_run_choose_tokens(),
		(WrapperName::ArcRun, RunWrapperMethod::RunNondet) => arcrun_run_nondet_tokens(),
		(WrapperName::ArcRun, RunWrapperMethod::RunFirstSuccess) =>
			arcrun_run_first_success_tokens(),
		(WrapperName::RcRunExplicit, RunWrapperMethod::Choose) =>
			rcrun_explicit_choose_constructor_tokens(),
		(WrapperName::RcRunExplicit, RunWrapperMethod::RunChoose) =>
			rcrun_explicit_run_choose_tokens(),
		(WrapperName::RcRunExplicit, RunWrapperMethod::RunNondet) =>
			rcrun_explicit_run_nondet_tokens(),
		(WrapperName::RcRunExplicit, RunWrapperMethod::RunFirstSuccess) =>
			rcrun_explicit_run_first_success_tokens(),
		(WrapperName::ArcRunExplicit, RunWrapperMethod::Choose) =>
			arcrun_explicit_choose_constructor_tokens(),
		(WrapperName::ArcRunExplicit, RunWrapperMethod::RunChoose) =>
			arcrun_explicit_run_choose_tokens(),
		(WrapperName::ArcRunExplicit, RunWrapperMethod::RunNondet) =>
			arcrun_explicit_run_nondet_tokens(),
		(WrapperName::ArcRunExplicit, RunWrapperMethod::RunFirstSuccess) =>
			arcrun_explicit_run_first_success_tokens(),
		_ => return None,
	};

	Some(impl_items_from_tokens(tokens))
}

fn rcrun_choose_constructor_tokens() -> TokenStream {
	quote! {
		/// Lifts an `Alt` choose effect into the `RcRun` program.
		/// Direct analog of PureScript Run's `choose` /
		/// `runChoose`. The program nondeterministically returns
		/// `true` or `false`; the handler runs the continuation
		/// twice (once per branch) to capture both outcomes.
		///
		/// `Choose` ships only on the four multi-shot wrappers
		/// because the handler must clone the continuation to invoke
		/// it twice. Threads
		/// [`RcBrand`](crate::brands::RcBrand) as the pointer kind.
		#[__document_module_generated]
		#[document_signature]
		#[doc = ""]
		#[document_type_parameters("The type-level Member-position witness (typically inferred).")]
		#[doc = ""]
		#[document_returns("An `RcRun` program suspended at the lifted `Alt` effect.")]
		#[doc = ""]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::rc_run::RcRun,
		/// };
		///
		/// type FirstRow = CoproductBrand<RcCoyonedaBrand<ChooseBrand<RcBrand>>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let prog: RcRun<FirstRow, Scoped, i32> = RcRun::<FirstRow, Scoped, bool>::choose()
		/// 	.bind(|branch| RcRun::<FirstRow, Scoped, i32>::pure(if branch { 1 } else { 0 }));
		/// let handled: RcRun<CNilBrand, CNilBrand, Vec<i32>> = prog.run_choose::<_, CNilBrand>();
		/// assert_eq!(handled.extract(), vec![1, 0]);
		/// ```
		#[inline]
		pub fn choose<Idx>() -> Self
		where
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, bool>):
				Member<RcCoyoneda<'static, crate::brands::ChooseBrand<RcBrand>, bool>, Idx>,
			Apply!(<NodeBrand<R, ScopedRow> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<NodeBrand<R, ScopedRow>, crate::types::rc_free::RcTypeErasedValue>,
			>): Clone, {
			let effect: crate::types::effects::choose::Choose<'static, RcBrand, bool> =
				crate::types::effects::choose::Choose::Alt(
					<RcBrand as crate::classes::ToDynCloneFn>::new(|b: bool| b),
				);
			Self::lift::<crate::brands::ChooseBrand<RcBrand>, Idx>(effect)
		}
	}
}

fn arcrun_choose_constructor_tokens() -> TokenStream {
	quote! {
		/// Lifts an `Alt` choose effect into the `ArcRun` program.
		/// Mirrors
		/// [`RcRun::choose`](crate::types::effects::rc_run::RcRun::choose);
		/// see that method for cross-wrapper semantics. Differences for
		/// `ArcRun`: threads
		/// [`ArcBrand`](crate::brands::ArcBrand) as the pointer kind
		/// and uses
		/// [`SendChooseBrand`](crate::brands::SendChooseBrand) (rather
		/// than `ChooseBrand`) so the continuation projection is
		/// structurally `Send + Sync`.
		#[__document_module_generated]
		#[document_signature]
		#[doc = ""]
		#[document_type_parameters("The type-level Member-position witness (typically inferred).")]
		#[doc = ""]
		#[document_returns("An `ArcRun` program suspended at the lifted `Alt` effect.")]
		#[doc = ""]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::arc_run::ArcRun,
		/// };
		///
		/// type FirstRow = CoproductBrand<ArcCoyonedaBrand<SendChooseBrand<ArcBrand>>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let prog: ArcRun<FirstRow, Scoped, i32> = ArcRun::<FirstRow, Scoped, bool>::choose()
		/// 	.bind(|branch| ArcRun::<FirstRow, Scoped, i32>::pure(if branch { 1 } else { 0 }));
		/// let handled: ArcRun<CNilBrand, CNilBrand, Vec<i32>> = prog.run_choose::<_, CNilBrand>();
		/// assert_eq!(handled.extract(), vec![1, 0]);
		/// ```
		#[inline]
		pub fn choose<Idx>() -> Self
		where
			NodeBrand<R, ScopedRow>: SendFunctor,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, bool>): Member<
					ArcCoyoneda<
						'static,
						crate::brands::SendChooseBrand<crate::brands::ArcBrand>,
						bool,
					>,
					Idx,
				>,
			Apply!(<NodeBrand<R, ScopedRow> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<NodeBrand<R, ScopedRow>, ArcTypeErasedValue>,
			>): Clone, {
			let effect: crate::types::effects::choose::SendChoose<
				'static,
				crate::brands::ArcBrand,
				bool,
			> = crate::types::effects::choose::SendChoose::Alt(
				<crate::brands::ArcBrand as crate::classes::ToDynSendFn>::new(|b: bool| b),
			);
			Self::lift::<crate::brands::SendChooseBrand<crate::brands::ArcBrand>, Idx>(effect)
		}
	}
}

fn rcrun_explicit_choose_constructor_tokens() -> TokenStream {
	quote! {
		/// Lifts an `Alt` choose effect into the `RcRunExplicit`
		/// program. Mirrors
		/// [`RcRun::choose`](crate::types::effects::rc_run::RcRun::choose);
		/// see that method for cross-wrapper semantics.
		#[__document_module_generated]
		#[document_signature]
		#[doc = ""]
		#[document_type_parameters("The type-level Member-position witness (typically inferred).")]
		#[doc = ""]
		#[document_returns("An `RcRunExplicit` program suspended at the lifted `Alt` effect.")]
		#[doc = ""]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::rc_run_explicit::RcRunExplicit,
		/// };
		///
		/// type FirstRow = CoproductBrand<RcCoyonedaBrand<ChooseBrand<RcBrand>>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let prog: RcRunExplicit<'static, FirstRow, Scoped, i32> =
		/// 	RcRunExplicit::<FirstRow, Scoped, bool>::choose().bind(|branch| {
		/// 		RcRunExplicit::<FirstRow, Scoped, i32>::pure(if branch { 1 } else { 0 })
		/// 	});
		/// let handled: RcRunExplicit<'static, CNilBrand, CNilBrand, Vec<i32>> =
		/// 	prog.run_choose::<_, CNilBrand>();
		/// assert_eq!(handled.extract(), vec![1, 0]);
		/// ```
		#[inline]
		pub fn choose<Idx>() -> Self
		where
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, bool>): Member<
					RcCoyoneda<'a, crate::brands::ChooseBrand<crate::brands::RcBrand>, bool>,
					Idx,
				>, {
			let effect: crate::types::effects::choose::Choose<'a, crate::brands::RcBrand, bool> =
				crate::types::effects::choose::Choose::Alt(
					<crate::brands::RcBrand as crate::classes::ToDynCloneFn>::new(|b: bool| b),
				);
			Self::lift::<crate::brands::ChooseBrand<crate::brands::RcBrand>, Idx>(effect)
		}
	}
}

fn arcrun_explicit_choose_constructor_tokens() -> TokenStream {
	quote! {
		/// Lifts an `Alt` choose effect into the `ArcRunExplicit`
		/// program. Mirrors
		/// [`RcRun::choose`](crate::types::effects::rc_run::RcRun::choose);
		/// see that method for cross-wrapper semantics. Differences
		/// for `ArcRunExplicit`: threads
		/// [`ArcBrand`](crate::brands::ArcBrand) as the pointer kind
		/// and uses
		/// [`SendChooseBrand`](crate::brands::SendChooseBrand) (rather
		/// than `ChooseBrand`) so the continuation projection is
		/// structurally `Send + Sync`.
		#[__document_module_generated]
		#[document_signature]
		#[doc = ""]
		#[document_type_parameters("The type-level Member-position witness (typically inferred).")]
		#[doc = ""]
		#[document_returns("An `ArcRunExplicit` program suspended at the lifted `Alt` effect.")]
		#[doc = ""]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::arc_run_explicit::ArcRunExplicit,
		/// };
		///
		/// type FirstRow = CoproductBrand<ArcCoyonedaBrand<SendChooseBrand<ArcBrand>>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let prog: ArcRunExplicit<'static, FirstRow, Scoped, i32> =
		/// 	ArcRunExplicit::<FirstRow, Scoped, bool>::choose().bind(|branch| {
		/// 		ArcRunExplicit::<FirstRow, Scoped, i32>::pure(if branch { 1 } else { 0 })
		/// 	});
		/// let handled: ArcRunExplicit<'static, CNilBrand, CNilBrand, Vec<i32>> =
		/// 	prog.run_choose::<_, CNilBrand>();
		/// assert_eq!(handled.extract(), vec![1, 0]);
		/// ```
		#[inline]
		pub fn choose<Idx>() -> Self
		where
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, bool>): Member<
					ArcCoyoneda<'a, crate::brands::SendChooseBrand<crate::brands::ArcBrand>, bool>,
					Idx,
				>,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, bool>,
			>): Send + Sync,
			Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, bool>,
			>): Send + Sync,
			Apply!(<NodeBrand<R, ScopedRow> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, bool>,
			>): Clone + Send + Sync, {
			let effect: crate::types::effects::choose::SendChoose<
				'a,
				crate::brands::ArcBrand,
				bool,
			> = crate::types::effects::choose::SendChoose::Alt(
				<crate::brands::ArcBrand as crate::classes::ToDynSendFn>::new(|b: bool| b),
			);
			Self::lift::<crate::brands::SendChooseBrand<crate::brands::ArcBrand>, Idx>(effect)
		}
	}
}

fn rcrun_run_choose_tokens() -> TokenStream {
	quote! {
		/// Interprets one Choose effect into a `Vec`.
		///
		/// Pure results become singleton vectors. Each `choose()` branches
		/// into the `true` path followed by the `false` path and concatenates
		/// the branch results in that order.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters(
			"The type-level Member-position witness for the Choose effect.",
			"The first-order row brand with the Choose effect removed."
		)]
		#[document_returns("A first-order-only `RcRun` program returning all branch results.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::rc_run::RcRun,
		/// };
		///
		/// type Row = CoproductBrand<RcCoyonedaBrand<ChooseBrand<RcBrand>>, CNilBrand>;
		///
		/// let program: RcRun<Row, CNilBrand, i32> = RcRun::<Row, CNilBrand, bool>::choose()
		/// 	.bind(|branch| RcRun::<Row, CNilBrand, i32>::pure(if branch { 1 } else { 0 }));
		/// let handled: RcRun<CNilBrand, CNilBrand, Vec<i32>> = program.run_choose::<_, CNilBrand>();
		/// assert_eq!(handled.extract(), vec![1, 0]);
		/// ```
		#[inline]
		pub fn run_choose<Idx, RMinusChoose>(self) -> RcRun<RMinusChoose, CNilBrand, Vec<A>>
		where
			A: Clone,
			RMinusChoose: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
			Apply!(<NodeBrand<R, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<NodeBrand<R, CNilBrand>, RcTypeErasedValue>,
			>): Clone,
			Apply!(<NodeBrand<RMinusChoose, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<NodeBrand<RMinusChoose, CNilBrand>, RcTypeErasedValue>,
			>): Clone,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcRun<R, CNilBrand, Vec<A>>,
			>): Member<
					RcCoyoneda<'static, ChooseBrand<RcBrand>, RcRun<R, CNilBrand, Vec<A>>>,
					Idx,
					Remainder = Apply!(
									<RMinusChoose as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
										'static,
										RcRun<R, CNilBrand, Vec<A>>,
									>
								),
				>, {
			self.map(|value| vec![value]).handle_with::<ChooseBrand<RcBrand>, Idx, RMinusChoose>(
				|op: Choose<'static, RcBrand, RcRun<RMinusChoose, CNilBrand, Vec<A>>>| match op {
					Choose::Alt(k) => {
						let true_branch = (*k)(true);
						let false_branch = (*k)(false);
						true_branch.bind(move |true_values| {
							let false_branch = false_branch.clone();
							false_branch.map(move |false_values| {
								let mut values = true_values.clone();
								values.extend(false_values);
								values
							})
						})
					}
				},
			)
		}
	}
}

fn arcrun_run_choose_tokens() -> TokenStream {
	quote! {
		/// Interprets one Choose effect into a `Vec`.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters(
			"The type-level Member-position witness for the Choose effect.",
			"The first-order row brand with the Choose effect removed."
		)]
		#[document_returns("A first-order-only `ArcRun` program returning all branch results.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::arc_run::ArcRun,
		/// };
		///
		/// type Row = CoproductBrand<ArcCoyonedaBrand<SendChooseBrand<ArcBrand>>, CNilBrand>;
		///
		/// let program: ArcRun<Row, CNilBrand, i32> = ArcRun::<Row, CNilBrand, bool>::choose()
		/// 	.bind(|branch| ArcRun::<Row, CNilBrand, i32>::pure(if branch { 1 } else { 0 }));
		/// let handled: ArcRun<CNilBrand, CNilBrand, Vec<i32>> = program.run_choose::<_, CNilBrand>();
		/// assert_eq!(handled.extract(), vec![1, 0]);
		/// ```
		#[inline]
		pub fn run_choose<Idx, RMinusChoose>(
			self
		) -> ArcRun<RMinusChoose, CNilBrand, Vec<A>>
		where
			A: Clone + Send + Sync,
			R: Kind_cdc7cd43dac7585f + 'static,
			RMinusChoose: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
			NodeBrand<R, CNilBrand>: SendFunctor,
			NodeBrand<RMinusChoose, CNilBrand>: WrapDrop
				+ Kind_cdc7cd43dac7585f<
					Of<'static, ArcFree<NodeBrand<RMinusChoose, CNilBrand>, ArcTypeErasedValue>>:
						Send + Sync,
				> + SendFunctor,
			Apply!(<NodeBrand<R, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<NodeBrand<R, CNilBrand>, ArcTypeErasedValue>,
			>): Clone,
			Apply!(<NodeBrand<RMinusChoose, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<NodeBrand<RMinusChoose, CNilBrand>, ArcTypeErasedValue>,
			>): Clone,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcRun<R, CNilBrand, Vec<A>>,
			>): Member<
					ArcCoyoneda<
						'static,
						SendChooseBrand<ArcBrand>,
						ArcRun<R, CNilBrand, Vec<A>>,
					>,
					Idx,
					Remainder = Apply!(
									<RMinusChoose as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
										'static,
										ArcRun<R, CNilBrand, Vec<A>>,
									>
								),
		>,{
			self.map(|value| vec![value])
				.handle_with::<SendChooseBrand<ArcBrand>, Idx, RMinusChoose>(
					|op: SendChoose<'static, ArcBrand, ArcRun<RMinusChoose, CNilBrand, Vec<A>>>| {
						match op {
							SendChoose::Alt(k) => {
								let true_branch = (*k)(true);
								let false_branch = (*k)(false);
								true_branch.bind(move |true_values| {
									let false_branch = false_branch.clone();
									false_branch.map(move |false_values| {
										let mut values = true_values.clone();
										values.extend(false_values);
										values
									})
								})
							}
						}
					},
				)
		}
	}
}

fn rcrun_explicit_run_choose_tokens() -> TokenStream {
	quote! {
		/// Interprets one Choose effect into a `Vec`.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters(
			"The type-level Member-position witness for the Choose effect.",
			"The first-order row brand with the Choose effect removed."
		)]
		#[document_returns(
			"A first-order-only `RcRunExplicit` program returning all branch results."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::rc_run_explicit::RcRunExplicit,
		/// };
		///
		/// type Row = CoproductBrand<RcCoyonedaBrand<ChooseBrand<RcBrand>>, CNilBrand>;
		///
		/// let program: RcRunExplicit<'static, Row, CNilBrand, i32> =
		/// 	RcRunExplicit::<'static, Row, CNilBrand, bool>::choose().bind(|branch| {
		/// 		RcRunExplicit::<'static, Row, CNilBrand, i32>::pure(if branch { 1 } else { 0 })
		/// 	});
		/// let handled: RcRunExplicit<'static, CNilBrand, CNilBrand, Vec<i32>> =
		/// 	program.run_choose::<_, CNilBrand>();
		/// assert_eq!(handled.extract(), vec![1, 0]);
		/// ```
		#[inline]
		pub fn run_choose<Idx, RMinusChoose>(
			self
		) -> RcRunExplicit<'a, RMinusChoose, CNilBrand, Vec<A>>
		where
			A: Clone,
			RMinusChoose: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
			Apply!(<NodeBrand<R, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, CNilBrand>, A>,
			>): Clone,
			Apply!(<NodeBrand<R, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, CNilBrand>, Vec<A>>,
			>): Clone,
			Apply!(<NodeBrand<RMinusChoose, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<RMinusChoose, CNilBrand>, Vec<A>>,
			>): Clone,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcRunExplicit<'a, R, CNilBrand, Vec<A>>,
			>): Member<
					RcCoyoneda<'a, ChooseBrand<RcBrand>, RcRunExplicit<'a, R, CNilBrand, Vec<A>>>,
					Idx,
					Remainder = Apply!(
									<RMinusChoose as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
										'a,
										RcRunExplicit<'a, R, CNilBrand, Vec<A>>,
									>
								),
				>, {
			self.map(|value| vec![value]).handle_with::<ChooseBrand<RcBrand>, Idx, RMinusChoose>(
				|op: Choose<'a, RcBrand, RcRunExplicit<'a, RMinusChoose, CNilBrand, Vec<A>>>| {
					match op {
						Choose::Alt(k) => {
							let true_branch = (*k)(true);
							let false_branch = (*k)(false);
							true_branch.bind(move |true_values| {
								let false_branch = false_branch.clone();
								false_branch.map(move |false_values| {
									let mut values = true_values.clone();
									values.extend(false_values);
									values
								})
							})
						}
					}
				},
			)
		}
	}
}

fn arcrun_explicit_run_choose_tokens() -> TokenStream {
	quote! {
		/// Interprets one Choose effect into a `Vec`.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters(
			"The type-level Member-position witness for the Choose effect.",
			"The first-order row brand with the Choose effect removed."
		)]
		#[document_returns(
			"A first-order-only `ArcRunExplicit` program returning all branch results."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::arc_run_explicit::ArcRunExplicit,
		/// };
		///
		/// type Row = CoproductBrand<ArcCoyonedaBrand<SendChooseBrand<ArcBrand>>, CNilBrand>;
		///
		/// let program: ArcRunExplicit<'static, Row, CNilBrand, i32> =
		/// 	ArcRunExplicit::<'static, Row, CNilBrand, bool>::choose().bind(|branch| {
		/// 		ArcRunExplicit::<'static, Row, CNilBrand, i32>::pure(if branch { 1 } else { 0 })
		/// 	});
		/// let handled: ArcRunExplicit<'static, CNilBrand, CNilBrand, Vec<i32>> =
		/// 	program.run_choose::<_, CNilBrand>();
		/// assert_eq!(handled.extract(), vec![1, 0]);
		/// ```
		#[inline]
		pub fn run_choose<Idx, RMinusChoose>(
			self
		) -> ArcRunExplicit<'a, RMinusChoose, CNilBrand, Vec<A>>
		where
			A: Clone + Send + Sync,
			RMinusChoose: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
			NodeBrand<R, CNilBrand>: SendFunctor,
			NodeBrand<RMinusChoose, CNilBrand>: SendFunctor,
			Apply!(<NodeBrand<R, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, CNilBrand>, A>,
			>): Clone + Send + Sync,
			Apply!(<NodeBrand<R, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, CNilBrand>, Vec<A>>,
			>): Clone + Send + Sync,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, CNilBrand>, A>,
			>): Send + Sync,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, CNilBrand>, Vec<A>>,
			>): Send + Sync,
			Apply!(<CNilBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, CNilBrand>, A>,
			>): Send + Sync,
			Apply!(<CNilBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, CNilBrand>, Vec<A>>,
			>): Send + Sync,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, R, CNilBrand, Vec<A>>,
			>): Send + Sync,
			Apply!(<CNilBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, R, CNilBrand, Vec<A>>,
			>): Send + Sync,
			Apply!(<NodeBrand<RMinusChoose, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<RMinusChoose, CNilBrand>, Vec<A>>,
			>): Clone + Send + Sync,
			Apply!(<RMinusChoose as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<RMinusChoose, CNilBrand>, Vec<A>>,
			>): Send + Sync,
			Apply!(<RMinusChoose as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, RMinusChoose, CNilBrand, Vec<A>>,
			>): Send + Sync,
			Apply!(<CNilBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<RMinusChoose, CNilBrand>, Vec<A>>,
			>): Send + Sync,
			Apply!(<CNilBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, RMinusChoose, CNilBrand, Vec<A>>,
			>): Send + Sync,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, R, CNilBrand, Vec<A>>,
			>): Member<
					ArcCoyoneda<
						'a,
						SendChooseBrand<ArcBrand>,
						ArcRunExplicit<'a, R, CNilBrand, Vec<A>>,
					>,
					Idx,
					Remainder = Apply!(
									<RMinusChoose as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
										'a,
										ArcRunExplicit<'a, R, CNilBrand, Vec<A>>,
									>
								),
				>, {
			self.map(|value| vec![value])
				.handle_with::<SendChooseBrand<ArcBrand>, Idx, RMinusChoose>(
					|op: SendChoose<
						'a,
						ArcBrand,
						ArcRunExplicit<'a, RMinusChoose, CNilBrand, Vec<A>>,
					>| {
						match op {
							SendChoose::Alt(k) => {
								let true_branch = (*k)(true);
								let false_branch = (*k)(false);
								true_branch.bind(move |true_values| {
									let false_branch = false_branch.clone();
									false_branch.map(move |false_values| {
										let mut values = true_values.clone();
										values.extend(false_values);
										values
									})
								})
							}
						}
					},
				)
		}
	}
}

fn rcrun_run_nondet_tokens() -> TokenStream {
	quote! {
		/// Interprets `Choose` and `Empty` together into a `Vec`.
		///
		/// Pure results become singleton vectors, `Empty` contributes
		/// no results, and `Choose` explores the `true` branch before
		/// the `false` branch.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters(
			"The type-level Member-position witness for the Choose effect.",
			"The type-level Member-position witness for the Empty effect after Choose is removed.",
			"The row brand with Choose removed.",
			"The first-order row brand with both NonDet effects removed."
		)]
		#[document_returns("A first-order-only `RcRun` program returning all successful results.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::rc_run::RcRun,
		/// };
		///
		/// type EmptyRow = CoproductBrand<RcCoyonedaBrand<EmptyBrand>, CNilBrand>;
		/// type Row = CoproductBrand<RcCoyonedaBrand<ChooseBrand<RcBrand>>, EmptyRow>;
		///
		/// let program: RcRun<Row, CNilBrand, i32> = RcRun::<Row, CNilBrand, bool>::choose()
		/// 	.bind(|branch| if branch { RcRun::pure(1) } else { RcRun::empty() });
		/// let handled: RcRun<CNilBrand, CNilBrand, Vec<i32>> =
		/// 	program.run_nondet::<_, _, EmptyRow, CNilBrand>();
		/// assert_eq!(handled.extract(), vec![1]);
		/// ```
		#[inline]
		pub fn run_nondet<ChooseIdx, EmptyIdx, RMinusChoose, RMinusNonDet>(
			self
		) -> RcRun<RMinusNonDet, CNilBrand, Vec<A>>
		where
			A: Clone,
			RMinusChoose: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
			RMinusNonDet: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
			Apply!(<NodeBrand<R, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<NodeBrand<R, CNilBrand>, RcTypeErasedValue>,
			>): Clone,
			Apply!(<NodeBrand<RMinusNonDet, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<NodeBrand<RMinusNonDet, CNilBrand>, RcTypeErasedValue>,
			>): Clone,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcRun<R, CNilBrand, A>,
			>): Member<
					RcCoyoneda<'static, ChooseBrand<RcBrand>, RcRun<R, CNilBrand, A>>,
					ChooseIdx,
					Remainder = Apply!(
									<RMinusChoose as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
										'static,
										RcRun<R, CNilBrand, A>,
									>
								),
				>,
			Apply!(<RMinusChoose as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcRun<R, CNilBrand, A>,
			>): Member<
					RcCoyoneda<'static, EmptyBrand, RcRun<R, CNilBrand, A>>,
					EmptyIdx,
					Remainder = Apply!(
									<RMinusNonDet as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
										'static,
										RcRun<R, CNilBrand, A>,
									>
								),
				>, {
			match self.peel() {
				Ok(a) => RcRun::pure(vec![a]),
				Err(Node::First(layer)) => match <Apply!(
					<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, RcRun<R, CNilBrand, A>>
				) as Member<
					RcCoyoneda<'static, ChooseBrand<RcBrand>, RcRun<R, CNilBrand, A>>,
					ChooseIdx,
				>>::project(layer)
				{
					Ok(coyo) => {
						let Choose::Alt(k) = coyo.lower_ref();
						let true_branch = (*k)(true);
						let k_for_false = k.clone();
						true_branch
							.run_nondet::<ChooseIdx, EmptyIdx, RMinusChoose, RMinusNonDet>()
							.bind(move |true_values| {
								let false_branch = (*k_for_false)(false);
								false_branch
									.run_nondet::<ChooseIdx, EmptyIdx, RMinusChoose, RMinusNonDet>()
									.map(move |false_values| {
										let mut values = true_values.clone();
										values.extend(false_values);
										values
									})
							})
					}
					Err(rest) => match <Apply!(
						<RMinusChoose as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
							'static,
							RcRun<R, CNilBrand, A>
						>
					) as Member<
						RcCoyoneda<'static, EmptyBrand, RcRun<R, CNilBrand, A>>,
						EmptyIdx,
					>>::project(rest)
					{
						Ok(coyo) => {
							let Empty::Empty(_) = coyo.lower_ref();
							RcRun::pure(Vec::new())
						}
						Err(rest) => {
							let mapped_free = <RMinusNonDet as Functor>::map(
								move |inner: RcRun<R, CNilBrand, A>| {
									inner
										.run_nondet::<ChooseIdx, EmptyIdx, RMinusChoose, RMinusNonDet>(
										)
										.into_rc_free()
								},
								rest,
							);
							RcRun::from_rc_free(
								RcFree::<NodeBrand<RMinusNonDet, CNilBrand>, Vec<A>>::wrap(
									Node::First(mapped_free),
								),
							)
						}
					},
				},
				Err(Node::Scoped(layer)) => match layer {},
			}
		}
	}
}

fn arcrun_run_nondet_tokens() -> TokenStream {
	quote! {
		/// Interprets `Choose` and `Empty` together into a `Vec`.
		///
		/// Pure results become singleton vectors, `Empty` contributes
		/// no results, and `Choose` explores the `true` branch before
		/// the `false` branch.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters(
			"The type-level Member-position witness for the Choose effect.",
			"The type-level Member-position witness for the Empty effect after Choose is removed.",
			"The row brand with Choose removed.",
			"The first-order row brand with both NonDet effects removed."
		)]
		#[document_returns("A first-order-only `ArcRun` program returning all successful results.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::arc_run::ArcRun,
		/// };
		///
		/// type EmptyRow = CoproductBrand<ArcCoyonedaBrand<EmptyBrand>, CNilBrand>;
		/// type Row = CoproductBrand<ArcCoyonedaBrand<SendChooseBrand<ArcBrand>>, EmptyRow>;
		///
		/// let program: ArcRun<Row, CNilBrand, i32> = ArcRun::<Row, CNilBrand, bool>::choose()
		/// 	.bind(|branch| if branch { ArcRun::pure(1) } else { ArcRun::empty() });
		/// let handled: ArcRun<CNilBrand, CNilBrand, Vec<i32>> =
		/// 	program.run_nondet::<_, _, EmptyRow, CNilBrand>();
		/// assert_eq!(handled.extract(), vec![1]);
		/// ```
		#[inline]
		pub fn run_nondet<ChooseIdx, EmptyIdx, RMinusChoose, RMinusNonDet>(
			self
		) -> ArcRun<RMinusNonDet, CNilBrand, Vec<A>>
		where
			A: Clone + Send + Sync,
			R: Kind_cdc7cd43dac7585f + 'static,
			RMinusChoose: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
			RMinusNonDet: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
			NodeBrand<R, CNilBrand>: SendFunctor,
			NodeBrand<RMinusNonDet, CNilBrand>: WrapDrop
				+ Kind_cdc7cd43dac7585f<
					Of<'static, ArcFree<NodeBrand<RMinusNonDet, CNilBrand>, ArcTypeErasedValue>>:
						Send + Sync,
				> + SendFunctor,
			Apply!(<NodeBrand<R, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<NodeBrand<R, CNilBrand>, ArcTypeErasedValue>,
			>): Clone,
			Apply!(<NodeBrand<RMinusNonDet, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<NodeBrand<RMinusNonDet, CNilBrand>, ArcTypeErasedValue>,
			>): Clone,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcRun<R, CNilBrand, A>,
			>): Member<
					ArcCoyoneda<
						'static,
						SendChooseBrand<ArcBrand>,
						ArcRun<R, CNilBrand, A>,
					>,
					ChooseIdx,
					Remainder = Apply!(
									<RMinusChoose as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
										'static,
										ArcRun<R, CNilBrand, A>,
									>
								),
				>,
			Apply!(<RMinusChoose as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcRun<R, CNilBrand, A>,
			>): Member<
					ArcCoyoneda<'static, EmptyBrand, ArcRun<R, CNilBrand, A>>,
					EmptyIdx,
					Remainder = Apply!(
									<RMinusNonDet as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
										'static,
										ArcRun<R, CNilBrand, A>,
									>
								),
		>,{
			match self.peel() {
				Ok(a) => ArcRun::pure(vec![a]),
				Err(node) => match unwrap_node::<R, CNilBrand, ArcRun<R, CNilBrand, A>>(node) {
					Node::First(layer) => match <Apply!(
						<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, ArcRun<R, CNilBrand, A>>
					) as Member<
						ArcCoyoneda<'static, SendChooseBrand<ArcBrand>, ArcRun<R, CNilBrand, A>>,
						ChooseIdx,
					>>::project(layer)
					{
						Ok(coyo) => {
							let SendChoose::Alt(k) = coyo.lower_ref();
							let true_branch = (*k)(true);
							let k_for_false = k.clone();
							true_branch
								.run_nondet::<ChooseIdx, EmptyIdx, RMinusChoose, RMinusNonDet>()
								.bind(move |true_values| {
									let false_branch = (*k_for_false)(false);
									false_branch
										.run_nondet::<ChooseIdx, EmptyIdx, RMinusChoose, RMinusNonDet>(
										)
										.map(move |false_values| {
											let mut values = true_values.clone();
											values.extend(false_values);
											values
										})
								})
						}
						Err(rest) => match <Apply!(
							<RMinusChoose as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
								'static,
								ArcRun<R, CNilBrand, A>
							>
						) as Member<
							ArcCoyoneda<'static, EmptyBrand, ArcRun<R, CNilBrand, A>>,
							EmptyIdx,
						>>::project(rest)
						{
							Ok(coyo) => {
								let Empty::Empty(_) = coyo.lower_ref();
								ArcRun::pure(Vec::new())
							}
							Err(rest) => {
								let mapped_free = <RMinusNonDet as SendFunctor>::send_map(
									move |inner: ArcRun<R, CNilBrand, A>| {
										inner
											.run_nondet::<ChooseIdx, EmptyIdx, RMinusChoose, RMinusNonDet>(
											)
											.into_arc_free()
									},
									rest,
								);
								let node_first = make_node_first::<
									RMinusNonDet,
									CNilBrand,
									ArcFree<NodeBrand<RMinusNonDet, CNilBrand>, Vec<A>>,
								>(mapped_free);
								ArcRun::from_arc_free(wrap_first_arc::<
									RMinusNonDet,
									CNilBrand,
									Vec<A>,
								>(node_first))
							}
						},
					},
					Node::Scoped(layer) => match layer {},
				},
			}
		}
	}
}

fn rcrun_explicit_run_nondet_tokens() -> TokenStream {
	quote! {
		/// Interprets `Choose` and `Empty` together into a `Vec`.
		///
		/// Pure results become singleton vectors, `Empty` contributes
		/// no results, and `Choose` explores the `true` branch before
		/// the `false` branch.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters(
			"The type-level Member-position witness for the Choose effect.",
			"The type-level Member-position witness for the Empty effect after Choose is removed.",
			"The row brand with Choose removed.",
			"The first-order row brand with both NonDet effects removed."
		)]
		#[document_returns(
			"A first-order-only `RcRunExplicit` program returning all successful results."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::rc_run_explicit::RcRunExplicit,
		/// };
		///
		/// type EmptyRow = CoproductBrand<RcCoyonedaBrand<EmptyBrand>, CNilBrand>;
		/// type Row = CoproductBrand<RcCoyonedaBrand<ChooseBrand<RcBrand>>, EmptyRow>;
		///
		/// let program: RcRunExplicit<'static, Row, CNilBrand, i32> =
		/// 	RcRunExplicit::<'static, Row, CNilBrand, bool>::choose()
		/// 		.bind(|branch| if branch { RcRunExplicit::pure(1) } else { RcRunExplicit::empty() });
		/// let handled: RcRunExplicit<'static, CNilBrand, CNilBrand, Vec<i32>> =
		/// 	program.run_nondet::<_, _, EmptyRow, CNilBrand>();
		/// assert_eq!(handled.extract(), vec![1]);
		/// ```
		#[inline]
		pub fn run_nondet<ChooseIdx, EmptyIdx, RMinusChoose, RMinusNonDet>(
			self
		) -> RcRunExplicit<'a, RMinusNonDet, CNilBrand, Vec<A>>
		where
			A: Clone,
			RMinusChoose: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
			RMinusNonDet: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
			Apply!(<NodeBrand<R, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, CNilBrand>, A>,
			>): Clone,
			Apply!(<NodeBrand<RMinusNonDet, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<RMinusNonDet, CNilBrand>, Vec<A>>,
			>): Clone,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcRunExplicit<'a, R, CNilBrand, A>,
			>): Member<
					RcCoyoneda<'a, ChooseBrand<RcBrand>, RcRunExplicit<'a, R, CNilBrand, A>>,
					ChooseIdx,
					Remainder = Apply!(
									<RMinusChoose as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
										'a,
										RcRunExplicit<'a, R, CNilBrand, A>,
									>
								),
				>,
			Apply!(<RMinusChoose as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcRunExplicit<'a, R, CNilBrand, A>,
			>): Member<
					RcCoyoneda<'a, EmptyBrand, RcRunExplicit<'a, R, CNilBrand, A>>,
					EmptyIdx,
					Remainder = Apply!(
									<RMinusNonDet as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
										'a,
										RcRunExplicit<'a, R, CNilBrand, A>,
									>
								),
				>, {
			match self.peel() {
				Ok(a) => RcRunExplicit::pure(vec![a]),
				Err(Node::First(layer)) => match <Apply!(
					<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
						'a,
						RcRunExplicit<'a, R, CNilBrand, A>
					>
				) as Member<
					RcCoyoneda<'a, ChooseBrand<RcBrand>, RcRunExplicit<'a, R, CNilBrand, A>>,
					ChooseIdx,
				>>::project(layer)
				{
					Ok(coyo) => {
						let Choose::Alt(k) = coyo.lower_ref();
						let true_branch = (*k)(true);
						let k_for_false = k.clone();
						true_branch
							.run_nondet::<ChooseIdx, EmptyIdx, RMinusChoose, RMinusNonDet>()
							.bind(move |true_values| {
								let false_branch = (*k_for_false)(false);
								false_branch
									.run_nondet::<ChooseIdx, EmptyIdx, RMinusChoose, RMinusNonDet>()
									.map(move |false_values| {
										let mut values = true_values.clone();
										values.extend(false_values);
										values
									})
							})
					}
					Err(rest) => match <Apply!(
						<RMinusChoose as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
							'a,
							RcRunExplicit<'a, R, CNilBrand, A>
						>
					) as Member<
						RcCoyoneda<'a, EmptyBrand, RcRunExplicit<'a, R, CNilBrand, A>>,
						EmptyIdx,
					>>::project(rest)
					{
						Ok(coyo) => {
							let Empty::Empty(_) = coyo.lower_ref();
							RcRunExplicit::pure(Vec::new())
						}
						Err(rest) => {
							let mapped_free = <RMinusNonDet as Functor>::map(
								move |inner: RcRunExplicit<'a, R, CNilBrand, A>| {
									inner
										.run_nondet::<ChooseIdx, EmptyIdx, RMinusChoose, RMinusNonDet>(
										)
										.into_rc_free_explicit()
								},
								rest,
							);
							RcRunExplicit::from_rc_free_explicit(RcFreeExplicit::<
								'a,
								NodeBrand<RMinusNonDet, CNilBrand>,
								Vec<A>,
							>::wrap(Node::First(
								mapped_free,
							)))
						}
					},
				},
				Err(Node::Scoped(layer)) => match layer {},
			}
		}
	}
}

fn arcrun_explicit_run_nondet_tokens() -> TokenStream {
	quote! {
		/// Interprets `Choose` and `Empty` together into a `Vec`.
		///
		/// Pure results become singleton vectors, `Empty` contributes
		/// no results, and `Choose` explores the `true` branch before
		/// the `false` branch.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters(
			"The type-level Member-position witness for the Choose effect.",
			"The type-level Member-position witness for the Empty effect after Choose is removed.",
			"The row brand with Choose removed.",
			"The first-order row brand with both NonDet effects removed."
		)]
		#[document_returns(
			"A first-order-only `ArcRunExplicit` program returning all successful results."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::arc_run_explicit::ArcRunExplicit,
		/// };
		///
		/// type EmptyRow = CoproductBrand<ArcCoyonedaBrand<EmptyBrand>, CNilBrand>;
		/// type Row = CoproductBrand<ArcCoyonedaBrand<SendChooseBrand<ArcBrand>>, EmptyRow>;
		///
		/// let program: ArcRunExplicit<'static, Row, CNilBrand, i32> =
		/// 	ArcRunExplicit::<'static, Row, CNilBrand, bool>::choose()
		/// 		.bind(|branch| if branch { ArcRunExplicit::pure(1) } else { ArcRunExplicit::empty() });
		/// let handled: ArcRunExplicit<'static, CNilBrand, CNilBrand, Vec<i32>> =
		/// 	program.run_nondet::<_, _, EmptyRow, CNilBrand>();
		/// assert_eq!(handled.extract(), vec![1]);
		/// ```
		#[inline]
		pub fn run_nondet<ChooseIdx, EmptyIdx, RMinusChoose, RMinusNonDet>(
			self
		) -> ArcRunExplicit<'a, RMinusNonDet, CNilBrand, Vec<A>>
		where
			A: Clone + Send + Sync,
			RMinusChoose: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
			RMinusNonDet: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
			NodeBrand<R, CNilBrand>: SendFunctor,
			NodeBrand<RMinusNonDet, CNilBrand>: SendFunctor,
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
			Apply!(<NodeBrand<RMinusNonDet, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<RMinusNonDet, CNilBrand>, Vec<A>>,
			>): Clone + Send + Sync,
			Apply!(<RMinusNonDet as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<RMinusNonDet, CNilBrand>, Vec<A>>,
			>): Send + Sync,
			Apply!(<RMinusNonDet as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, RMinusNonDet, CNilBrand, Vec<A>>,
			>): Send + Sync,
			Apply!(<CNilBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<RMinusNonDet, CNilBrand>, Vec<A>>,
			>): Send + Sync,
			Apply!(<CNilBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, RMinusNonDet, CNilBrand, Vec<A>>,
			>): Send + Sync,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, R, CNilBrand, A>,
			>): Member<
					ArcCoyoneda<'a, SendChooseBrand<ArcBrand>, ArcRunExplicit<'a, R, CNilBrand, A>>,
					ChooseIdx,
					Remainder = Apply!(
									<RMinusChoose as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
										'a,
										ArcRunExplicit<'a, R, CNilBrand, A>,
									>
								),
				>,
			Apply!(<RMinusChoose as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, R, CNilBrand, A>,
			>): Member<
					ArcCoyoneda<'a, EmptyBrand, ArcRunExplicit<'a, R, CNilBrand, A>>,
					EmptyIdx,
					Remainder = Apply!(
									<RMinusNonDet as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
										'a,
										ArcRunExplicit<'a, R, CNilBrand, A>,
									>
								),
				>, {
			match self.peel() {
				Ok(a) => ArcRunExplicit::pure(vec![a]),
				Err(node) => match unwrap_arc_run_explicit_nondet_node::<R, A>(node) {
					Node::First(layer) => match <Apply!(
						<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
							'a,
							ArcRunExplicit<'a, R, CNilBrand, A>
						>
					) as Member<
						ArcCoyoneda<
							'a,
							SendChooseBrand<ArcBrand>,
							ArcRunExplicit<'a, R, CNilBrand, A>,
						>,
						ChooseIdx,
					>>::project(layer)
					{
						Ok(coyo) => {
							let SendChoose::Alt(k) = coyo.lower_ref();
							let true_branch = (*k)(true);
							let k_for_false = k.clone();
							true_branch
								.run_nondet::<ChooseIdx, EmptyIdx, RMinusChoose, RMinusNonDet>()
								.bind(move |true_values| {
									let false_branch = (*k_for_false)(false);
									false_branch
										.run_nondet::<ChooseIdx, EmptyIdx, RMinusChoose, RMinusNonDet>(
										)
										.map(move |false_values| {
											let mut values = true_values.clone();
											values.extend(false_values);
											values
										})
								})
						}
						Err(rest) => match <Apply!(
							<RMinusChoose as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
								'a,
								ArcRunExplicit<'a, R, CNilBrand, A>
							>
						) as Member<
							ArcCoyoneda<'a, EmptyBrand, ArcRunExplicit<'a, R, CNilBrand, A>>,
							EmptyIdx,
						>>::project(rest)
						{
							Ok(coyo) => {
								let Empty::Empty(_) = coyo.lower_ref();
								ArcRunExplicit::pure(Vec::new())
							}
							Err(rest) => {
								let mapped_free = <RMinusNonDet as SendFunctor>::send_map(
									move |inner: ArcRunExplicit<'a, R, CNilBrand, A>| {
										inner
											.run_nondet::<ChooseIdx, EmptyIdx, RMinusChoose, RMinusNonDet>(
											)
											.into_arc_free_explicit()
									},
									rest,
								);
								let node_first = make_arc_run_explicit_nondet_node_first::<
									RMinusNonDet,
									ArcFreeExplicit<'a, NodeBrand<RMinusNonDet, CNilBrand>, Vec<A>>,
								>(mapped_free);
								ArcRunExplicit::from_arc_free_explicit(ArcFreeExplicit::<
									'a,
									NodeBrand<RMinusNonDet, CNilBrand>,
									Vec<A>,
								>::wrap(node_first))
							}
						},
					},
					Node::Scoped(layer) => match layer {},
				},
			}
		}
	}
}

fn rcrun_run_first_success_tokens() -> TokenStream {
	quote! {
		/// Interprets `Choose` and `Empty` into the first successful result.
		///
		/// Pure results become `Some(value)`, `Empty` becomes `None`,
		/// and `Choose` tries the `true` branch before evaluating the
		/// `false` branch.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters(
			"The type-level Member-position witness for the Choose effect.",
			"The type-level Member-position witness for the Empty effect after Choose is removed.",
			"The row brand with Choose removed.",
			"The first-order row brand with both NonDet effects removed."
		)]
		#[document_returns(
			"A first-order-only `RcRun` program returning the first successful result."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::rc_run::RcRun,
		/// };
		///
		/// type EmptyRow = CoproductBrand<RcCoyonedaBrand<EmptyBrand>, CNilBrand>;
		/// type Row = CoproductBrand<RcCoyonedaBrand<ChooseBrand<RcBrand>>, EmptyRow>;
		///
		/// let program: RcRun<Row, CNilBrand, i32> = RcRun::<Row, CNilBrand, bool>::choose()
		/// 	.bind(|branch| if branch { RcRun::empty() } else { RcRun::pure(2) });
		/// let handled: RcRun<CNilBrand, CNilBrand, Option<i32>> =
		/// 	program.run_first_success::<_, _, EmptyRow, CNilBrand>();
		/// assert_eq!(handled.extract(), Some(2));
		/// ```
		#[inline]
		pub fn run_first_success<ChooseIdx, EmptyIdx, RMinusChoose, RMinusNonDet>(
			self
		) -> RcRun<RMinusNonDet, CNilBrand, Option<A>>
		where
			A: Clone,
			RMinusChoose: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
			RMinusNonDet: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
			Apply!(<NodeBrand<R, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<NodeBrand<R, CNilBrand>, RcTypeErasedValue>,
			>): Clone,
			Apply!(<NodeBrand<RMinusNonDet, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<NodeBrand<RMinusNonDet, CNilBrand>, RcTypeErasedValue>,
			>): Clone,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcRun<R, CNilBrand, A>,
			>): Member<
					RcCoyoneda<'static, ChooseBrand<RcBrand>, RcRun<R, CNilBrand, A>>,
					ChooseIdx,
					Remainder = Apply!(
									<RMinusChoose as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
										'static,
										RcRun<R, CNilBrand, A>,
									>
								),
				>,
			Apply!(<RMinusChoose as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcRun<R, CNilBrand, A>,
			>): Member<
					RcCoyoneda<'static, EmptyBrand, RcRun<R, CNilBrand, A>>,
					EmptyIdx,
					Remainder = Apply!(
									<RMinusNonDet as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
										'static,
										RcRun<R, CNilBrand, A>,
									>
								),
				>, {
			match self.peel() {
				Ok(a) => RcRun::pure(Some(a)),
				Err(Node::First(layer)) => match <Apply!(
					<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, RcRun<R, CNilBrand, A>>
				) as Member<
					RcCoyoneda<'static, ChooseBrand<RcBrand>, RcRun<R, CNilBrand, A>>,
					ChooseIdx,
				>>::project(layer)
				{
					Ok(coyo) => {
						let Choose::Alt(k) = coyo.lower_ref();
						let true_branch = (*k)(true);
						let k_for_false = k.clone();
						true_branch
							.run_first_success::<ChooseIdx, EmptyIdx, RMinusChoose, RMinusNonDet>()
							.bind(move |first| match first {
								Some(value) => RcRun::pure(Some(value)),
								None => (*k_for_false)(false)
									.run_first_success::<ChooseIdx, EmptyIdx, RMinusChoose, RMinusNonDet>(
									),
							})
					}
					Err(rest) => match <Apply!(
						<RMinusChoose as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
							'static,
							RcRun<R, CNilBrand, A>
						>
					) as Member<
						RcCoyoneda<'static, EmptyBrand, RcRun<R, CNilBrand, A>>,
						EmptyIdx,
					>>::project(rest)
					{
						Ok(coyo) => {
							let Empty::Empty(_) = coyo.lower_ref();
							RcRun::pure(None)
						}
						Err(rest) => {
							let mapped_free = <RMinusNonDet as Functor>::map(
								move |inner: RcRun<R, CNilBrand, A>| {
									inner
										.run_first_success::<ChooseIdx, EmptyIdx, RMinusChoose, RMinusNonDet>(
										)
										.into_rc_free()
								},
								rest,
							);
							RcRun::from_rc_free(RcFree::<
								NodeBrand<RMinusNonDet, CNilBrand>,
								Option<A>,
							>::wrap(Node::First(mapped_free)))
						}
					},
				},
				Err(Node::Scoped(layer)) => match layer {},
			}
		}
	}
}

fn arcrun_run_first_success_tokens() -> TokenStream {
	quote! {
		/// Interprets `Choose` and `Empty` into the first successful result.
		///
		/// Pure results become `Some(value)`, `Empty` becomes `None`,
		/// and `Choose` tries the `true` branch before evaluating the
		/// `false` branch.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters(
			"The type-level Member-position witness for the Choose effect.",
			"The type-level Member-position witness for the Empty effect after Choose is removed.",
			"The row brand with Choose removed.",
			"The first-order row brand with both NonDet effects removed."
		)]
		#[document_returns(
			"A first-order-only `ArcRun` program returning the first successful result."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::arc_run::ArcRun,
		/// };
		///
		/// type EmptyRow = CoproductBrand<ArcCoyonedaBrand<EmptyBrand>, CNilBrand>;
		/// type Row = CoproductBrand<ArcCoyonedaBrand<SendChooseBrand<ArcBrand>>, EmptyRow>;
		///
		/// let program: ArcRun<Row, CNilBrand, i32> = ArcRun::<Row, CNilBrand, bool>::choose()
		/// 	.bind(|branch| if branch { ArcRun::empty() } else { ArcRun::pure(2) });
		/// let handled: ArcRun<CNilBrand, CNilBrand, Option<i32>> =
		/// 	program.run_first_success::<_, _, EmptyRow, CNilBrand>();
		/// assert_eq!(handled.extract(), Some(2));
		/// ```
		#[inline]
		pub fn run_first_success<ChooseIdx, EmptyIdx, RMinusChoose, RMinusNonDet>(
			self
		) -> ArcRun<RMinusNonDet, CNilBrand, Option<A>>
		where
			A: Clone + Send + Sync,
			R: Kind_cdc7cd43dac7585f + 'static,
			RMinusChoose: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
			RMinusNonDet: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
			NodeBrand<R, CNilBrand>: SendFunctor,
			NodeBrand<RMinusNonDet, CNilBrand>: WrapDrop
				+ Kind_cdc7cd43dac7585f<
					Of<'static, ArcFree<NodeBrand<RMinusNonDet, CNilBrand>, ArcTypeErasedValue>>:
						Send + Sync,
				> + SendFunctor,
			Apply!(<NodeBrand<R, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<NodeBrand<R, CNilBrand>, ArcTypeErasedValue>,
			>): Clone,
			Apply!(<NodeBrand<RMinusNonDet, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<NodeBrand<RMinusNonDet, CNilBrand>, ArcTypeErasedValue>,
			>): Clone,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcRun<R, CNilBrand, A>,
			>): Member<
					ArcCoyoneda<
						'static,
						SendChooseBrand<ArcBrand>,
						ArcRun<R, CNilBrand, A>,
					>,
					ChooseIdx,
					Remainder = Apply!(
									<RMinusChoose as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
										'static,
										ArcRun<R, CNilBrand, A>,
									>
								),
				>,
			Apply!(<RMinusChoose as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcRun<R, CNilBrand, A>,
			>): Member<
					ArcCoyoneda<'static, EmptyBrand, ArcRun<R, CNilBrand, A>>,
					EmptyIdx,
					Remainder = Apply!(
									<RMinusNonDet as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
										'static,
										ArcRun<R, CNilBrand, A>,
									>
								),
		>,{
			match self.peel() {
				Ok(a) => ArcRun::pure(Some(a)),
				Err(node) => match unwrap_node::<R, CNilBrand, ArcRun<R, CNilBrand, A>>(node) {
					Node::First(layer) => match <Apply!(
						<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, ArcRun<R, CNilBrand, A>>
					) as Member<
						ArcCoyoneda<'static, SendChooseBrand<ArcBrand>, ArcRun<R, CNilBrand, A>>,
						ChooseIdx,
					>>::project(layer)
					{
						Ok(coyo) => {
							let SendChoose::Alt(k) = coyo.lower_ref();
							let true_branch = (*k)(true);
							let k_for_false = k.clone();
							true_branch
								.run_first_success::<ChooseIdx, EmptyIdx, RMinusChoose, RMinusNonDet>(
								)
								.bind(move |first| match first {
									Some(value) => ArcRun::pure(Some(value)),
									None => (*k_for_false)(false)
										.run_first_success::<ChooseIdx, EmptyIdx, RMinusChoose, RMinusNonDet>(
										),
								})
						}
						Err(rest) => match <Apply!(
							<RMinusChoose as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
								'static,
								ArcRun<R, CNilBrand, A>
							>
						) as Member<
							ArcCoyoneda<'static, EmptyBrand, ArcRun<R, CNilBrand, A>>,
							EmptyIdx,
						>>::project(rest)
						{
							Ok(coyo) => {
								let Empty::Empty(_) = coyo.lower_ref();
								ArcRun::pure(None)
							}
							Err(rest) => {
								let mapped_free = <RMinusNonDet as SendFunctor>::send_map(
									move |inner: ArcRun<R, CNilBrand, A>| {
										inner
											.run_first_success::<ChooseIdx, EmptyIdx, RMinusChoose, RMinusNonDet>(
											)
											.into_arc_free()
									},
									rest,
								);
								let node_first = make_node_first::<
									RMinusNonDet,
									CNilBrand,
									ArcFree<NodeBrand<RMinusNonDet, CNilBrand>, Option<A>>,
								>(mapped_free);
								ArcRun::from_arc_free(wrap_first_arc::<
									RMinusNonDet,
									CNilBrand,
									Option<A>,
								>(node_first))
							}
						},
					},
					Node::Scoped(layer) => match layer {},
				},
			}
		}
	}
}

fn rcrun_explicit_run_first_success_tokens() -> TokenStream {
	quote! {
		/// Interprets `Choose` and `Empty` into the first successful result.
		///
		/// Pure results become `Some(value)`, `Empty` becomes `None`,
		/// and `Choose` tries the `true` branch before evaluating the
		/// `false` branch.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters(
			"The type-level Member-position witness for the Choose effect.",
			"The type-level Member-position witness for the Empty effect after Choose is removed.",
			"The row brand with Choose removed.",
			"The first-order row brand with both NonDet effects removed."
		)]
		#[document_returns(
			"A first-order-only `RcRunExplicit` program returning the first successful result."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::rc_run_explicit::RcRunExplicit,
		/// };
		///
		/// type EmptyRow = CoproductBrand<RcCoyonedaBrand<EmptyBrand>, CNilBrand>;
		/// type Row = CoproductBrand<RcCoyonedaBrand<ChooseBrand<RcBrand>>, EmptyRow>;
		///
		/// let program: RcRunExplicit<'static, Row, CNilBrand, i32> =
		/// 	RcRunExplicit::<'static, Row, CNilBrand, bool>::choose()
		/// 		.bind(|branch| if branch { RcRunExplicit::empty() } else { RcRunExplicit::pure(2) });
		/// let handled: RcRunExplicit<'static, CNilBrand, CNilBrand, Option<i32>> =
		/// 	program.run_first_success::<_, _, EmptyRow, CNilBrand>();
		/// assert_eq!(handled.extract(), Some(2));
		/// ```
		#[inline]
		pub fn run_first_success<ChooseIdx, EmptyIdx, RMinusChoose, RMinusNonDet>(
			self
		) -> RcRunExplicit<'a, RMinusNonDet, CNilBrand, Option<A>>
		where
			A: Clone,
			RMinusChoose: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
			RMinusNonDet: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
			Apply!(<NodeBrand<R, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, CNilBrand>, A>,
			>): Clone,
			Apply!(<NodeBrand<RMinusNonDet, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<RMinusNonDet, CNilBrand>, Option<A>>,
			>): Clone,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcRunExplicit<'a, R, CNilBrand, A>,
			>): Member<
					RcCoyoneda<'a, ChooseBrand<RcBrand>, RcRunExplicit<'a, R, CNilBrand, A>>,
					ChooseIdx,
					Remainder = Apply!(
									<RMinusChoose as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
										'a,
										RcRunExplicit<'a, R, CNilBrand, A>,
									>
								),
				>,
			Apply!(<RMinusChoose as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcRunExplicit<'a, R, CNilBrand, A>,
			>): Member<
					RcCoyoneda<'a, EmptyBrand, RcRunExplicit<'a, R, CNilBrand, A>>,
					EmptyIdx,
					Remainder = Apply!(
									<RMinusNonDet as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
										'a,
										RcRunExplicit<'a, R, CNilBrand, A>,
									>
								),
				>, {
			match self.peel() {
				Ok(a) => RcRunExplicit::pure(Some(a)),
				Err(Node::First(layer)) => match <Apply!(
					<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
						'a,
						RcRunExplicit<'a, R, CNilBrand, A>
					>
				) as Member<
					RcCoyoneda<'a, ChooseBrand<RcBrand>, RcRunExplicit<'a, R, CNilBrand, A>>,
					ChooseIdx,
				>>::project(layer)
				{
					Ok(coyo) => {
						let Choose::Alt(k) = coyo.lower_ref();
						let true_branch = (*k)(true);
						let k_for_false = k.clone();
						true_branch
							.run_first_success::<ChooseIdx, EmptyIdx, RMinusChoose, RMinusNonDet>()
							.bind(move |first| match first {
								Some(value) => RcRunExplicit::pure(Some(value)),
								None => (*k_for_false)(false)
									.run_first_success::<ChooseIdx, EmptyIdx, RMinusChoose, RMinusNonDet>(
									),
							})
					}
					Err(rest) => match <Apply!(
						<RMinusChoose as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
							'a,
							RcRunExplicit<'a, R, CNilBrand, A>
						>
					) as Member<
						RcCoyoneda<'a, EmptyBrand, RcRunExplicit<'a, R, CNilBrand, A>>,
						EmptyIdx,
					>>::project(rest)
					{
						Ok(coyo) => {
							let Empty::Empty(_) = coyo.lower_ref();
							RcRunExplicit::pure(None)
						}
						Err(rest) => {
							let mapped_free = <RMinusNonDet as Functor>::map(
								move |inner: RcRunExplicit<'a, R, CNilBrand, A>| {
									inner
										.run_first_success::<ChooseIdx, EmptyIdx, RMinusChoose, RMinusNonDet>(
										)
										.into_rc_free_explicit()
								},
								rest,
							);
							RcRunExplicit::from_rc_free_explicit(RcFreeExplicit::<
								'a,
								NodeBrand<RMinusNonDet, CNilBrand>,
								Option<A>,
							>::wrap(Node::First(
								mapped_free,
							)))
						}
					},
				},
				Err(Node::Scoped(layer)) => match layer {},
			}
		}
	}
}

fn arcrun_explicit_run_first_success_tokens() -> TokenStream {
	quote! {
		/// Interprets `Choose` and `Empty` into the first successful result.
		///
		/// Pure results become `Some(value)`, `Empty` becomes `None`,
		/// and `Choose` tries the `true` branch before evaluating the
		/// `false` branch.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters(
			"The type-level Member-position witness for the Choose effect.",
			"The type-level Member-position witness for the Empty effect after Choose is removed.",
			"The row brand with Choose removed.",
			"The first-order row brand with both NonDet effects removed."
		)]
		#[document_returns(
			"A first-order-only `ArcRunExplicit` program returning the first successful result."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::arc_run_explicit::ArcRunExplicit,
		/// };
		///
		/// type EmptyRow = CoproductBrand<ArcCoyonedaBrand<EmptyBrand>, CNilBrand>;
		/// type Row = CoproductBrand<ArcCoyonedaBrand<SendChooseBrand<ArcBrand>>, EmptyRow>;
		///
		/// let program: ArcRunExplicit<'static, Row, CNilBrand, i32> =
		/// 	ArcRunExplicit::<'static, Row, CNilBrand, bool>::choose()
		/// 		.bind(|branch| if branch { ArcRunExplicit::empty() } else { ArcRunExplicit::pure(2) });
		/// let handled: ArcRunExplicit<'static, CNilBrand, CNilBrand, Option<i32>> =
		/// 	program.run_first_success::<_, _, EmptyRow, CNilBrand>();
		/// assert_eq!(handled.extract(), Some(2));
		/// ```
		#[inline]
		pub fn run_first_success<ChooseIdx, EmptyIdx, RMinusChoose, RMinusNonDet>(
			self
		) -> ArcRunExplicit<'a, RMinusNonDet, CNilBrand, Option<A>>
		where
			A: Clone + Send + Sync,
			RMinusChoose: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
			RMinusNonDet: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
			NodeBrand<R, CNilBrand>: SendFunctor,
			NodeBrand<RMinusNonDet, CNilBrand>: SendFunctor,
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
			Apply!(<NodeBrand<RMinusNonDet, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<RMinusNonDet, CNilBrand>, Option<A>>,
			>): Clone + Send + Sync,
			Apply!(<RMinusNonDet as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<RMinusNonDet, CNilBrand>, Option<A>>,
			>): Send + Sync,
			Apply!(<RMinusNonDet as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, RMinusNonDet, CNilBrand, Option<A>>,
			>): Send + Sync,
			Apply!(<CNilBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<RMinusNonDet, CNilBrand>, Option<A>>,
			>): Send + Sync,
			Apply!(<CNilBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, RMinusNonDet, CNilBrand, Option<A>>,
			>): Send + Sync,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, R, CNilBrand, A>,
			>): Member<
					ArcCoyoneda<'a, SendChooseBrand<ArcBrand>, ArcRunExplicit<'a, R, CNilBrand, A>>,
					ChooseIdx,
					Remainder = Apply!(
									<RMinusChoose as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
										'a,
										ArcRunExplicit<'a, R, CNilBrand, A>,
									>
								),
				>,
			Apply!(<RMinusChoose as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, R, CNilBrand, A>,
			>): Member<
					ArcCoyoneda<'a, EmptyBrand, ArcRunExplicit<'a, R, CNilBrand, A>>,
					EmptyIdx,
					Remainder = Apply!(
									<RMinusNonDet as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
										'a,
										ArcRunExplicit<'a, R, CNilBrand, A>,
									>
								),
				>, {
			match self.peel() {
				Ok(a) => ArcRunExplicit::pure(Some(a)),
				Err(node) => match unwrap_arc_run_explicit_nondet_node::<R, A>(node) {
					Node::First(layer) => match <Apply!(
						<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
							'a,
							ArcRunExplicit<'a, R, CNilBrand, A>
						>
					) as Member<
						ArcCoyoneda<
							'a,
							SendChooseBrand<ArcBrand>,
							ArcRunExplicit<'a, R, CNilBrand, A>,
						>,
						ChooseIdx,
					>>::project(layer)
					{
						Ok(coyo) => {
							let SendChoose::Alt(k) = coyo.lower_ref();
							let true_branch = (*k)(true);
							let k_for_false = k.clone();
							true_branch
								.run_first_success::<ChooseIdx, EmptyIdx, RMinusChoose, RMinusNonDet>(
								)
								.bind(move |first| match first {
									Some(value) => ArcRunExplicit::pure(Some(value)),
									None => (*k_for_false)(false)
										.run_first_success::<ChooseIdx, EmptyIdx, RMinusChoose, RMinusNonDet>(
										),
								})
						}
						Err(rest) => match <Apply!(
							<RMinusChoose as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
								'a,
								ArcRunExplicit<'a, R, CNilBrand, A>
							>
						) as Member<
							ArcCoyoneda<'a, EmptyBrand, ArcRunExplicit<'a, R, CNilBrand, A>>,
							EmptyIdx,
						>>::project(rest)
						{
							Ok(coyo) => {
								let Empty::Empty(_) = coyo.lower_ref();
								ArcRunExplicit::pure(None)
							}
							Err(rest) => {
								let mapped_free = <RMinusNonDet as SendFunctor>::send_map(
									move |inner: ArcRunExplicit<'a, R, CNilBrand, A>| {
										inner
											.run_first_success::<ChooseIdx, EmptyIdx, RMinusChoose, RMinusNonDet>(
											)
											.into_arc_free_explicit()
									},
									rest,
								);
								let node_first = make_arc_run_explicit_nondet_node_first::<
									RMinusNonDet,
									ArcFreeExplicit<
										'a,
										NodeBrand<RMinusNonDet, CNilBrand>,
										Option<A>,
									>,
								>(mapped_free);
								ArcRunExplicit::from_arc_free_explicit(ArcFreeExplicit::<
									'a,
									NodeBrand<RMinusNonDet, CNilBrand>,
									Option<A>,
								>::wrap(node_first))
							}
						},
					},
					Node::Scoped(layer) => match layer {},
				},
			}
		}
	}
}
