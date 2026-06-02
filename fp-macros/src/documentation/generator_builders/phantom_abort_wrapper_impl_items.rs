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

pub(super) fn phantom_abort_wrapper_impl_items_from_descriptor(
	wrapper: WrapperName,
	method: RunWrapperMethod,
) -> Option<syn::Result<Vec<ImplItem>>> {
	generator_descriptors::method_spec(EffectName::Empty, method)?;
	generator_descriptors::wrapper_spec(wrapper)?;

	let tokens = match (wrapper, method) {
		(WrapperName::Run, RunWrapperMethod::Empty) => run_empty_constructor_tokens(),
		(WrapperName::Run, RunWrapperMethod::RunEmpty) => run_run_empty_tokens(),
		(WrapperName::RcRun, RunWrapperMethod::Empty) => rcrun_empty_constructor_tokens(),
		(WrapperName::RcRun, RunWrapperMethod::RunEmpty) => rcrun_run_empty_tokens(),
		(WrapperName::ArcRun, RunWrapperMethod::Empty) => arcrun_empty_constructor_tokens(),
		(WrapperName::ArcRun, RunWrapperMethod::RunEmpty) => arcrun_run_empty_tokens(),
		(WrapperName::RunExplicit, RunWrapperMethod::Empty) =>
			run_explicit_empty_constructor_tokens(),
		(WrapperName::RunExplicit, RunWrapperMethod::RunEmpty) => run_explicit_run_empty_tokens(),
		(WrapperName::RcRunExplicit, RunWrapperMethod::Empty) =>
			rcrun_explicit_empty_constructor_tokens(),
		(WrapperName::RcRunExplicit, RunWrapperMethod::RunEmpty) =>
			rcrun_explicit_run_empty_tokens(),
		(WrapperName::ArcRunExplicit, RunWrapperMethod::Empty) =>
			arcrun_explicit_empty_constructor_tokens(),
		(WrapperName::ArcRunExplicit, RunWrapperMethod::RunEmpty) =>
			arcrun_explicit_run_empty_tokens(),
		_ => return None,
	};

	Some(impl_items_from_tokens(tokens))
}

fn run_empty_constructor_tokens() -> TokenStream {
	quote! {
		/// Lifts an `Empty` effect into the Run program.
		///
		/// `Empty` aborts the current branch without producing the
		/// result type `A`. A handler decides how that absence is
		/// represented, such as returning an empty collection in a
		/// nondeterministic interpreter.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters("The type-level Member-position witness (typically inferred).")]
		#[document_returns("A `Run` program suspended at the lifted `Empty` effect.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::run::Run,
		/// };
		///
		/// type FirstRow = CoproductBrand<CoyonedaBrand<EmptyBrand>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let prog: Run<FirstRow, Scoped, i32> = Run::empty();
		/// let handled: Run<CNilBrand, CNilBrand, Option<i32>> = prog.run_empty::<_, CNilBrand>();
		/// assert_eq!(handled.extract(), None);
		/// ```
		#[inline]
		pub fn empty<Idx>() -> Self
		where
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, A>):
				crate::types::effects::member::Member<
						crate::types::Coyoneda<'static, crate::brands::EmptyBrand, A>,
						Idx,
					>, {
			let effect: crate::types::effects::empty::Empty<'static, A> =
				crate::types::effects::empty::Empty::Empty(core::marker::PhantomData);
			Self::lift::<crate::brands::EmptyBrand, Idx>(effect)
		}
	}
}

fn rcrun_empty_constructor_tokens() -> TokenStream {
	quote! {
		/// Lifts an `Empty` effect into the `RcRun` program.
		///
		/// `Empty` aborts the current branch without producing the
		/// result type `A`. A handler decides how that absence is
		/// represented, such as returning an empty collection in a
		/// multi-shot nondeterministic interpreter.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters("The type-level Member-position witness (typically inferred).")]
		#[document_returns("An `RcRun` program suspended at the lifted `Empty` effect.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::rc_run::RcRun,
		/// };
		///
		/// type FirstRow = CoproductBrand<RcCoyonedaBrand<EmptyBrand>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let prog: RcRun<FirstRow, Scoped, i32> = RcRun::empty();
		/// let handled: RcRun<CNilBrand, CNilBrand, Option<i32>> = prog.run_empty::<_, CNilBrand>();
		/// assert_eq!(handled.extract(), None);
		/// ```
		#[inline]
		pub fn empty<Idx>() -> Self
		where
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, A>):
				Member<RcCoyoneda<'static, crate::brands::EmptyBrand, A>, Idx>,
			Apply!(<NodeBrand<R, ScopedRow> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<NodeBrand<R, ScopedRow>, crate::types::rc_free::RcTypeErasedValue>,
			>): Clone, {
			let effect: crate::types::effects::empty::Empty<'static, A> =
				crate::types::effects::empty::Empty::Empty(core::marker::PhantomData);
			Self::lift::<crate::brands::EmptyBrand, Idx>(effect)
		}
	}
}

fn arcrun_empty_constructor_tokens() -> TokenStream {
	quote! {
		/// Lifts an `Empty` effect into the `ArcRun` program.
		///
		/// `Empty` aborts the current branch without producing the
		/// result type `A`. A handler decides how that absence is
		/// represented, such as returning an empty collection in a
		/// thread-safe multi-shot nondeterministic interpreter.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters("The type-level Member-position witness (typically inferred).")]
		#[document_returns("An `ArcRun` program suspended at the lifted `Empty` effect.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::arc_run::ArcRun,
		/// };
		///
		/// type FirstRow = CoproductBrand<ArcCoyonedaBrand<EmptyBrand>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let prog: ArcRun<FirstRow, Scoped, i32> = ArcRun::empty();
		/// let handled: ArcRun<CNilBrand, CNilBrand, Option<i32>> = prog.run_empty::<_, CNilBrand>();
		/// assert_eq!(handled.extract(), None);
		/// ```
		#[inline]
		pub fn empty<Idx>() -> Self
		where
			A: Send + Sync,
			NodeBrand<R, ScopedRow>: SendFunctor,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, A>):
				Member<ArcCoyoneda<'static, crate::brands::EmptyBrand, A>, Idx>,
			Apply!(<NodeBrand<R, ScopedRow> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<NodeBrand<R, ScopedRow>, ArcTypeErasedValue>,
			>): Clone, {
			let effect: crate::types::effects::empty::Empty<'static, A> =
				crate::types::effects::empty::Empty::Empty(core::marker::PhantomData);
			Self::lift::<crate::brands::EmptyBrand, Idx>(effect)
		}
	}
}

fn run_explicit_empty_constructor_tokens() -> TokenStream {
	quote! {
		/// Lifts an `Empty` effect into the `RunExplicit` program.
		///
		/// `Empty` aborts the current branch without producing the
		/// result type `A`. A handler decides how that absence is
		/// represented, such as returning a fallback value in a
		/// single-shot interpreter.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters("The type-level Member-position witness (typically inferred).")]
		#[document_returns("A `RunExplicit` program suspended at the lifted `Empty` effect.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::run_explicit::RunExplicit,
		/// };
		///
		/// type FirstRow = CoproductBrand<CoyonedaBrand<EmptyBrand>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let prog: RunExplicit<'static, FirstRow, Scoped, i32> = RunExplicit::empty();
		/// let handled: RunExplicit<'static, CNilBrand, CNilBrand, Option<i32>> =
		/// 	prog.run_empty::<_, CNilBrand>();
		/// assert_eq!(handled.extract(), None);
		/// ```
		#[inline]
		pub fn empty<Idx>() -> Self
		where
			A: 'static,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>):
				Member<Coyoneda<'a, crate::brands::EmptyBrand, A>, Idx>, {
			let effect: crate::types::effects::empty::Empty<'a, A> =
				crate::types::effects::empty::Empty::Empty(core::marker::PhantomData);
			Self::lift::<crate::brands::EmptyBrand, Idx>(effect)
		}
	}
}

fn rcrun_explicit_empty_constructor_tokens() -> TokenStream {
	quote! {
		/// Lifts an `Empty` effect into the `RcRunExplicit` program.
		///
		/// `Empty` aborts the current branch without producing the
		/// result type `A`. A handler decides how that absence is
		/// represented, such as returning an empty collection in a
		/// multi-shot nondeterministic interpreter.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters("The type-level Member-position witness (typically inferred).")]
		#[document_returns("An `RcRunExplicit` program suspended at the lifted `Empty` effect.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::rc_run_explicit::RcRunExplicit,
		/// };
		///
		/// type FirstRow = CoproductBrand<RcCoyonedaBrand<EmptyBrand>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let prog: RcRunExplicit<'static, FirstRow, Scoped, i32> = RcRunExplicit::empty();
		/// let handled: RcRunExplicit<'static, CNilBrand, CNilBrand, Option<i32>> =
		/// 	prog.run_empty::<_, CNilBrand>();
		/// assert_eq!(handled.extract(), None);
		/// ```
		#[inline]
		pub fn empty<Idx>() -> Self
		where
			A: Clone + 'static,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>):
				Member<RcCoyoneda<'a, crate::brands::EmptyBrand, A>, Idx>, {
			let effect: crate::types::effects::empty::Empty<'a, A> =
				crate::types::effects::empty::Empty::Empty(core::marker::PhantomData);
			Self::lift::<crate::brands::EmptyBrand, Idx>(effect)
		}
	}
}

fn arcrun_explicit_empty_constructor_tokens() -> TokenStream {
	quote! {
		/// Lifts an `Empty` effect into the `ArcRunExplicit` program.
		///
		/// `Empty` aborts the current branch without producing the
		/// result type `A`. A handler decides how that absence is
		/// represented, such as returning an empty collection in a
		/// thread-safe multi-shot nondeterministic interpreter.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters("The type-level Member-position witness (typically inferred).")]
		#[document_returns("An `ArcRunExplicit` program suspended at the lifted `Empty` effect.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::arc_run_explicit::ArcRunExplicit,
		/// };
		///
		/// type FirstRow = CoproductBrand<ArcCoyonedaBrand<EmptyBrand>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let prog: ArcRunExplicit<'static, FirstRow, Scoped, i32> = ArcRunExplicit::empty();
		/// let handled: ArcRunExplicit<'static, CNilBrand, CNilBrand, Option<i32>> =
		/// 	prog.run_empty::<_, CNilBrand>();
		/// assert_eq!(handled.extract(), None);
		/// ```
		#[inline]
		pub fn empty<Idx>() -> Self
		where
			A: Clone + Send + Sync + 'static,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>):
				Member<ArcCoyoneda<'a, crate::brands::EmptyBrand, A>, Idx>,
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
			let effect: crate::types::effects::empty::Empty<'a, A> =
				crate::types::effects::empty::Empty::Empty(core::marker::PhantomData);
			Self::lift::<crate::brands::EmptyBrand, Idx>(effect)
		}
	}
}

fn run_run_empty_tokens() -> TokenStream {
	quote! {
		/// Interprets one Empty effect into `Option`.
		///
		/// Normal completion becomes `Some(value)`. An `Empty` operation
		/// aborts the current branch and becomes `None`.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters(
			"The type-level Member-position witness for the Empty effect.",
			"The first-order row brand with the Empty effect removed."
		)]
		#[document_returns("A first-order-only `Run` program returning `Some(result)` or `None`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::run::Run,
		/// };
		///
		/// type Row = CoproductBrand<CoyonedaBrand<EmptyBrand>, CNilBrand>;
		///
		/// let program: Run<Row, CNilBrand, i32> = Run::empty();
		/// let handled: Run<CNilBrand, CNilBrand, Option<i32>> = program.run_empty::<_, CNilBrand>();
		/// assert_eq!(handled.extract(), None);
		/// ```
		#[inline]
		pub fn run_empty<Idx, RMinusEmpty>(self) -> Run<RMinusEmpty, CNilBrand, Option<A>>
		where
			RMinusEmpty: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				Run<R, CNilBrand, Option<A>>,
			>): Member<
					Coyoneda<'static, EmptyBrand, Run<R, CNilBrand, Option<A>>>,
					Idx,
					Remainder = Apply!(
									<RMinusEmpty as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
										'static,
										Run<R, CNilBrand, Option<A>>,
									>
								),
				>, {
			self.map(Some).handle_with::<EmptyBrand, Idx, RMinusEmpty>(
				|op: Empty<'static, Run<RMinusEmpty, CNilBrand, Option<A>>>| match op {
					Empty::Empty(_) => Run::pure(None),
				},
			)
		}
	}
}

fn rcrun_run_empty_tokens() -> TokenStream {
	quote! {
		/// Interprets one Empty effect into `Option`.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters(
			"The type-level Member-position witness for the Empty effect.",
			"The first-order row brand with the Empty effect removed."
		)]
		#[document_returns(
			"A first-order-only `RcRun` program returning `Some(result)` or `None`."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::rc_run::RcRun,
		/// };
		///
		/// type Row = CoproductBrand<RcCoyonedaBrand<EmptyBrand>, CNilBrand>;
		///
		/// let program: RcRun<Row, CNilBrand, i32> = RcRun::empty();
		/// let handled: RcRun<CNilBrand, CNilBrand, Option<i32>> = program.run_empty::<_, CNilBrand>();
		/// assert_eq!(handled.extract(), None);
		/// ```
		#[inline]
		pub fn run_empty<Idx, RMinusEmpty>(self) -> RcRun<RMinusEmpty, CNilBrand, Option<A>>
		where
			A: Clone,
			RMinusEmpty: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
			Apply!(<NodeBrand<R, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<NodeBrand<R, CNilBrand>, RcTypeErasedValue>,
			>): Clone,
			Apply!(<NodeBrand<RMinusEmpty, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<NodeBrand<RMinusEmpty, CNilBrand>, RcTypeErasedValue>,
			>): Clone,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcRun<R, CNilBrand, Option<A>>,
			>): Member<
					RcCoyoneda<'static, EmptyBrand, RcRun<R, CNilBrand, Option<A>>>,
					Idx,
					Remainder = Apply!(
									<RMinusEmpty as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
										'static,
										RcRun<R, CNilBrand, Option<A>>,
									>
								),
				>, {
			self.map(Some).handle_with::<EmptyBrand, Idx, RMinusEmpty>(
				|op: Empty<'static, RcRun<RMinusEmpty, CNilBrand, Option<A>>>| match op {
					Empty::Empty(_) => RcRun::pure(None),
				},
			)
		}
	}
}

fn arcrun_run_empty_tokens() -> TokenStream {
	quote! {
		/// Interprets one Empty effect into `Option`.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters(
			"The type-level Member-position witness for the Empty effect.",
			"The first-order row brand with the Empty effect removed."
		)]
		#[document_returns(
			"A first-order-only `ArcRun` program returning `Some(result)` or `None`."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::arc_run::ArcRun,
		/// };
		///
		/// type Row = CoproductBrand<ArcCoyonedaBrand<EmptyBrand>, CNilBrand>;
		///
		/// let program: ArcRun<Row, CNilBrand, i32> = ArcRun::empty();
		/// let handled: ArcRun<CNilBrand, CNilBrand, Option<i32>> = program.run_empty::<_, CNilBrand>();
		/// assert_eq!(handled.extract(), None);
		/// ```
		#[inline]
		pub fn run_empty<Idx, RMinusEmpty>(
			self
		) -> ArcRun<RMinusEmpty, CNilBrand, Option<A>>
		where
			A: Clone + Send + Sync,
			R: Kind_cdc7cd43dac7585f + 'static,
			RMinusEmpty: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
			NodeBrand<R, CNilBrand>: SendFunctor,
			NodeBrand<RMinusEmpty, CNilBrand>: WrapDrop
				+ Kind_cdc7cd43dac7585f<
					Of<'static, ArcFree<NodeBrand<RMinusEmpty, CNilBrand>, ArcTypeErasedValue>>:
						Send + Sync,
				> + SendFunctor,
			Apply!(<NodeBrand<R, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<NodeBrand<R, CNilBrand>, ArcTypeErasedValue>,
			>): Clone,
			Apply!(<NodeBrand<RMinusEmpty, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<NodeBrand<RMinusEmpty, CNilBrand>, ArcTypeErasedValue>,
			>): Clone,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcRun<R, CNilBrand, Option<A>>,
			>): Member<
					ArcCoyoneda<'static, EmptyBrand, ArcRun<R, CNilBrand, Option<A>>>,
					Idx,
					Remainder = Apply!(
									<RMinusEmpty as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
										'static,
										ArcRun<R, CNilBrand, Option<A>>,
									>
								),
		>,{
			self.map(Some).handle_with::<EmptyBrand, Idx, RMinusEmpty>(
				|op: Empty<'static, ArcRun<RMinusEmpty, CNilBrand, Option<A>>>| match op {
					Empty::Empty(_) => ArcRun::pure(None),
				},
			)
		}
	}
}

fn run_explicit_run_empty_tokens() -> TokenStream {
	quote! {
		/// Interprets one Empty effect into `Option`.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters(
			"The type-level Member-position witness for the Empty effect.",
			"The first-order row brand with the Empty effect removed."
		)]
		#[document_returns(
			"A first-order-only `RunExplicit` program returning `Some(result)` or `None`."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::run_explicit::RunExplicit,
		/// };
		///
		/// type Row = CoproductBrand<CoyonedaBrand<EmptyBrand>, CNilBrand>;
		///
		/// let program: RunExplicit<'static, Row, CNilBrand, i32> = RunExplicit::empty();
		/// let handled: RunExplicit<'static, CNilBrand, CNilBrand, Option<i32>> =
		/// 	program.run_empty::<_, CNilBrand>();
		/// assert_eq!(handled.extract(), None);
		/// ```
		#[inline]
		pub fn run_empty<Idx, RMinusEmpty>(
			self
		) -> RunExplicit<'a, RMinusEmpty, CNilBrand, Option<A>>
		where
			RMinusEmpty: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RunExplicit<'a, R, CNilBrand, Option<A>>,
			>): Member<
					Coyoneda<'a, EmptyBrand, RunExplicit<'a, R, CNilBrand, Option<A>>>,
					Idx,
					Remainder = Apply!(
									<RMinusEmpty as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
										'a,
										RunExplicit<'a, R, CNilBrand, Option<A>>,
									>
								),
				>, {
			self.map(Some).handle_with::<EmptyBrand, Idx, RMinusEmpty>(
				|op: Empty<'a, RunExplicit<'a, RMinusEmpty, CNilBrand, Option<A>>>| match op {
					Empty::Empty(_) => RunExplicit::pure(None),
				},
			)
		}
	}
}

fn rcrun_explicit_run_empty_tokens() -> TokenStream {
	quote! {
		/// Interprets one Empty effect into `Option`.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters(
			"The type-level Member-position witness for the Empty effect.",
			"The first-order row brand with the Empty effect removed."
		)]
		#[document_returns(
			"A first-order-only `RcRunExplicit` program returning `Some(result)` or `None`."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::rc_run_explicit::RcRunExplicit,
		/// };
		///
		/// type Row = CoproductBrand<RcCoyonedaBrand<EmptyBrand>, CNilBrand>;
		///
		/// let program: RcRunExplicit<'static, Row, CNilBrand, i32> = RcRunExplicit::empty();
		/// let handled: RcRunExplicit<'static, CNilBrand, CNilBrand, Option<i32>> =
		/// 	program.run_empty::<_, CNilBrand>();
		/// assert_eq!(handled.extract(), None);
		/// ```
		#[inline]
		pub fn run_empty<Idx, RMinusEmpty>(
			self
		) -> RcRunExplicit<'a, RMinusEmpty, CNilBrand, Option<A>>
		where
			A: Clone,
			RMinusEmpty: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
			Apply!(<NodeBrand<R, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, CNilBrand>, A>,
			>): Clone,
			Apply!(<NodeBrand<R, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, CNilBrand>, Option<A>>,
			>): Clone,
			Apply!(<NodeBrand<RMinusEmpty, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<RMinusEmpty, CNilBrand>, Option<A>>,
			>): Clone,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcRunExplicit<'a, R, CNilBrand, Option<A>>,
			>): Member<
					RcCoyoneda<'a, EmptyBrand, RcRunExplicit<'a, R, CNilBrand, Option<A>>>,
					Idx,
					Remainder = Apply!(
									<RMinusEmpty as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
										'a,
										RcRunExplicit<'a, R, CNilBrand, Option<A>>,
									>
								),
				>, {
			self.map(Some).handle_with::<EmptyBrand, Idx, RMinusEmpty>(
				|op: Empty<'a, RcRunExplicit<'a, RMinusEmpty, CNilBrand, Option<A>>>| match op {
					Empty::Empty(_) => RcRunExplicit::pure(None),
				},
			)
		}
	}
}

fn arcrun_explicit_run_empty_tokens() -> TokenStream {
	quote! {
		/// Interprets one Empty effect into `Option`.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters(
			"The type-level Member-position witness for the Empty effect.",
			"The first-order row brand with the Empty effect removed."
		)]
		#[document_returns(
			"A first-order-only `ArcRunExplicit` program returning `Some(result)` or `None`."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::arc_run_explicit::ArcRunExplicit,
		/// };
		///
		/// type Row = CoproductBrand<ArcCoyonedaBrand<EmptyBrand>, CNilBrand>;
		///
		/// let program: ArcRunExplicit<'static, Row, CNilBrand, i32> = ArcRunExplicit::empty();
		/// let handled: ArcRunExplicit<'static, CNilBrand, CNilBrand, Option<i32>> =
		/// 	program.run_empty::<_, CNilBrand>();
		/// assert_eq!(handled.extract(), None);
		/// ```
		#[inline]
		pub fn run_empty<Idx, RMinusEmpty>(
			self
		) -> ArcRunExplicit<'a, RMinusEmpty, CNilBrand, Option<A>>
		where
			A: Clone + Send + Sync,
			RMinusEmpty: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
			NodeBrand<R, CNilBrand>: SendFunctor,
			NodeBrand<RMinusEmpty, CNilBrand>: SendFunctor,
			Apply!(<NodeBrand<R, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, CNilBrand>, A>,
			>): Clone + Send + Sync,
			Apply!(<NodeBrand<R, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, CNilBrand>, Option<A>>,
			>): Clone + Send + Sync,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, CNilBrand>, A>,
			>): Send + Sync,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, CNilBrand>, Option<A>>,
			>): Send + Sync,
			Apply!(<CNilBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, CNilBrand>, A>,
			>): Send + Sync,
			Apply!(<CNilBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, CNilBrand>, Option<A>>,
			>): Send + Sync,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, R, CNilBrand, Option<A>>,
			>): Send + Sync,
			Apply!(<CNilBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, R, CNilBrand, Option<A>>,
			>): Send + Sync,
			Apply!(<NodeBrand<RMinusEmpty, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<RMinusEmpty, CNilBrand>, Option<A>>,
			>): Clone + Send + Sync,
			Apply!(<RMinusEmpty as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<RMinusEmpty, CNilBrand>, Option<A>>,
			>): Send + Sync,
			Apply!(<RMinusEmpty as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, RMinusEmpty, CNilBrand, Option<A>>,
			>): Send + Sync,
			Apply!(<CNilBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<RMinusEmpty, CNilBrand>, Option<A>>,
			>): Send + Sync,
			Apply!(<CNilBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, RMinusEmpty, CNilBrand, Option<A>>,
			>): Send + Sync,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, R, CNilBrand, Option<A>>,
			>): Member<
					ArcCoyoneda<'a, EmptyBrand, ArcRunExplicit<'a, R, CNilBrand, Option<A>>>,
					Idx,
					Remainder = Apply!(
									<RMinusEmpty as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
										'a,
										ArcRunExplicit<'a, R, CNilBrand, Option<A>>,
									>
								),
				>, {
			self.map(Some).handle_with::<EmptyBrand, Idx, RMinusEmpty>(
				|op: Empty<'a, ArcRunExplicit<'a, RMinusEmpty, CNilBrand, Option<A>>>| match op {
					Empty::Empty(_) => ArcRunExplicit::pure(None),
				},
			)
		}
	}
}
