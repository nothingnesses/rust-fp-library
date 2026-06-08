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

pub(super) fn fresh_wrapper_impl_items_from_descriptor(
	wrapper: WrapperName,
	method: RunWrapperMethod,
) -> Option<syn::Result<Vec<ImplItem>>> {
	generator_descriptors::method_spec(EffectName::Fresh, method)?;
	generator_descriptors::wrapper_spec(wrapper)?;

	let tokens = match (wrapper, method) {
		(WrapperName::Run, RunWrapperMethod::Fresh) => run_fresh_tokens(),
		(WrapperName::Run, RunWrapperMethod::RunFreshWith) => run_run_fresh_with_tokens(),
		(WrapperName::Run, RunWrapperMethod::RunFresh) => run_run_fresh_tokens(),
		(WrapperName::RcRun, RunWrapperMethod::Fresh) => rcrun_fresh_tokens(),
		(WrapperName::RcRun, RunWrapperMethod::RunFreshWith) => rcrun_run_fresh_with_tokens(),
		(WrapperName::RcRun, RunWrapperMethod::RunFresh) => rcrun_run_fresh_tokens(),
		(WrapperName::ArcRun, RunWrapperMethod::Fresh) => arcrun_fresh_tokens(),
		(WrapperName::ArcRun, RunWrapperMethod::RunFreshWith) => arcrun_run_fresh_with_tokens(),
		(WrapperName::ArcRun, RunWrapperMethod::RunFresh) => arcrun_run_fresh_tokens(),
		(WrapperName::RunExplicit, RunWrapperMethod::Fresh) => run_explicit_fresh_tokens(),
		(WrapperName::RunExplicit, RunWrapperMethod::RunFreshWith) =>
			run_explicit_run_fresh_with_tokens(),
		(WrapperName::RunExplicit, RunWrapperMethod::RunFresh) => run_explicit_run_fresh_tokens(),
		(WrapperName::RcRunExplicit, RunWrapperMethod::Fresh) => rcrun_explicit_fresh_tokens(),
		(WrapperName::RcRunExplicit, RunWrapperMethod::RunFreshWith) =>
			rcrun_explicit_run_fresh_with_tokens(),
		(WrapperName::RcRunExplicit, RunWrapperMethod::RunFresh) =>
			rcrun_explicit_run_fresh_tokens(),
		(WrapperName::ArcRunExplicit, RunWrapperMethod::Fresh) => arcrun_explicit_fresh_tokens(),
		(WrapperName::ArcRunExplicit, RunWrapperMethod::RunFreshWith) =>
			arcrun_explicit_run_fresh_with_tokens(),
		(WrapperName::ArcRunExplicit, RunWrapperMethod::RunFresh) =>
			arcrun_explicit_run_fresh_tokens(),
		_ => return None,
	};

	Some(impl_items_from_tokens(tokens))
}

fn run_fresh_tokens() -> TokenStream {
	quote! {
		/// Lifts a Fresh effect into the Run program.
		///
		/// The program asks the handler for the current generated value
		/// and returns that value as its result.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters("The type-level Member-position witness (typically inferred).")]
		#[document_returns("A `Run` program suspended at the lifted Fresh effect.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::run::Run,
		/// };
		///
		/// type Row = CoproductBrand<CoyonedaBrand<BoxFreshBrand<BoxBrand, usize>>, CNilBrand>;
		///
		/// let program: Run<Row, CNilBrand, usize> = Run::fresh();
		/// let handled: Run<CNilBrand, CNilBrand, (usize, usize)> =
		/// 	program.run_fresh_with::<usize, _, CNilBrand>(0, |counter| counter + 1);
		/// assert_eq!(handled.extract(), (0, 1));
		/// ```
		#[inline]
		pub fn fresh<Idx>() -> Self
		where
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, A>):
				crate::types::effects::member::Member<
					crate::types::Coyoneda<
						'static,
						crate::brands::BoxFreshBrand<crate::brands::BoxBrand, A>,
						A,
					>,
					Idx,
				>, {
			let effect: crate::types::effects::fresh::BoxFresh<'static, crate::brands::BoxBrand, A, A> =
				crate::types::effects::fresh::BoxFresh::Fresh(
					<crate::brands::BoxBrand as crate::classes::ToDynFnOnce>::new(|id: A| id),
				);
			Self::lift::<crate::brands::BoxFreshBrand<crate::brands::BoxBrand, A>, Idx>(effect)
		}
	}
}

fn rcrun_fresh_tokens() -> TokenStream {
	quote! {
		/// Lifts a Fresh effect into the `RcRun` program.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters("The type-level Member-position witness (typically inferred).")]
		#[document_returns("An `RcRun` program suspended at the lifted Fresh effect.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::rc_run::RcRun,
		/// };
		///
		/// type Row = CoproductBrand<RcCoyonedaBrand<FreshBrand<RcBrand, usize>>, CNilBrand>;
		///
		/// let program: RcRun<Row, CNilBrand, usize> = RcRun::fresh();
		/// let handled: RcRun<CNilBrand, CNilBrand, (usize, usize)> =
		/// 	program.run_fresh_with::<usize, _, CNilBrand>(0, |counter| counter + 1);
		/// assert_eq!(handled.extract(), (0, 1));
		/// ```
		#[inline]
		pub fn fresh<Idx>() -> Self
		where
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, A>):
				Member<RcCoyoneda<'static, crate::brands::FreshBrand<crate::brands::RcBrand, A>, A>, Idx>,
			Apply!(<NodeBrand<R, ScopedRow> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<NodeBrand<R, ScopedRow>, crate::types::rc_free::RcTypeErasedValue>,
			>): Clone, {
			let effect: crate::types::effects::fresh::Fresh<'static, crate::brands::RcBrand, A, A> =
				crate::types::effects::fresh::Fresh::Fresh(
					<crate::brands::RcBrand as crate::classes::ToDynCloneFn>::new(|id: A| id),
				);
			Self::lift::<crate::brands::FreshBrand<crate::brands::RcBrand, A>, Idx>(effect)
		}
	}
}

fn arcrun_fresh_tokens() -> TokenStream {
	quote! {
		/// Lifts a Fresh effect into the `ArcRun` program.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters("The type-level Member-position witness (typically inferred).")]
		#[document_returns("An `ArcRun` program suspended at the lifted Fresh effect.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::arc_run::ArcRun,
		/// };
		///
		/// type Row = CoproductBrand<ArcCoyonedaBrand<SendFreshBrand<ArcBrand, usize>>, CNilBrand>;
		///
		/// let program: ArcRun<Row, CNilBrand, usize> = ArcRun::fresh();
		/// let handled: ArcRun<CNilBrand, CNilBrand, (usize, usize)> =
		/// 	program.run_fresh_with::<usize, _, CNilBrand>(0, |counter| counter + 1);
		/// assert_eq!(handled.extract(), (0, 1));
		/// ```
		#[inline]
		pub fn fresh<Idx>() -> Self
		where
			A: Clone + Send + Sync,
			NodeBrand<R, ScopedRow>: SendFunctor,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, A>): Member<
				ArcCoyoneda<'static, crate::brands::SendFreshBrand<crate::brands::ArcBrand, A>, A>,
				Idx,
			>,
			Apply!(<NodeBrand<R, ScopedRow> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<NodeBrand<R, ScopedRow>, ArcTypeErasedValue>,
			>): Clone, {
			let effect: crate::types::effects::fresh::SendFresh<
				'static,
				crate::brands::ArcBrand,
				A,
				A,
			> = crate::types::effects::fresh::SendFresh::Fresh(
				<crate::brands::ArcBrand as crate::classes::ToDynSendFn>::new(|id: A| id),
			);
			Self::lift::<crate::brands::SendFreshBrand<crate::brands::ArcBrand, A>, Idx>(effect)
		}
	}
}

fn run_explicit_fresh_tokens() -> TokenStream {
	quote! {
		/// Lifts a Fresh effect into the `RunExplicit` program.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters("The type-level Member-position witness (typically inferred).")]
		#[document_returns("A `RunExplicit` program suspended at the lifted Fresh effect.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::run_explicit::RunExplicit,
		/// };
		///
		/// type Row = CoproductBrand<CoyonedaBrand<BoxFreshBrand<BoxBrand, usize>>, CNilBrand>;
		///
		/// let program: RunExplicit<'static, Row, CNilBrand, usize> = RunExplicit::fresh();
		/// let handled: RunExplicit<'static, CNilBrand, CNilBrand, (usize, usize)> =
		/// 	program.run_fresh_with::<usize, _, CNilBrand>(0, |counter| counter + 1);
		/// assert_eq!(handled.extract(), (0, 1));
		/// ```
		#[inline]
		pub fn fresh<Idx>() -> Self
		where
			A: 'static,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>):
				Member<Coyoneda<'a, crate::brands::BoxFreshBrand<crate::brands::BoxBrand, A>, A>, Idx>, {
			let effect: crate::types::effects::fresh::BoxFresh<'a, crate::brands::BoxBrand, A, A> =
				crate::types::effects::fresh::BoxFresh::Fresh(
					<crate::brands::BoxBrand as crate::classes::ToDynFnOnce>::new(|id: A| id),
				);
			Self::lift::<crate::brands::BoxFreshBrand<crate::brands::BoxBrand, A>, Idx>(effect)
		}
	}
}

fn rcrun_explicit_fresh_tokens() -> TokenStream {
	quote! {
		/// Lifts a Fresh effect into the `RcRunExplicit` program.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters("The type-level Member-position witness (typically inferred).")]
		#[document_returns("An `RcRunExplicit` program suspended at the lifted Fresh effect.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::rc_run_explicit::RcRunExplicit,
		/// };
		///
		/// type Row = CoproductBrand<RcCoyonedaBrand<FreshBrand<RcBrand, usize>>, CNilBrand>;
		///
		/// let program: RcRunExplicit<'static, Row, CNilBrand, usize> = RcRunExplicit::fresh();
		/// let handled: RcRunExplicit<'static, CNilBrand, CNilBrand, (usize, usize)> =
		/// 	program.run_fresh_with::<usize, _, CNilBrand>(0, |counter| counter + 1);
		/// assert_eq!(handled.extract(), (0, 1));
		/// ```
		#[inline]
		pub fn fresh<Idx>() -> Self
		where
			A: Clone + 'static,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>):
				Member<RcCoyoneda<'a, crate::brands::FreshBrand<crate::brands::RcBrand, A>, A>, Idx>, {
			let effect: crate::types::effects::fresh::Fresh<'a, crate::brands::RcBrand, A, A> =
				crate::types::effects::fresh::Fresh::Fresh(
					<crate::brands::RcBrand as crate::classes::ToDynCloneFn>::new(|id: A| id),
				);
			Self::lift::<crate::brands::FreshBrand<crate::brands::RcBrand, A>, Idx>(effect)
		}
	}
}

fn arcrun_explicit_fresh_tokens() -> TokenStream {
	quote! {
		/// Lifts a Fresh effect into the `ArcRunExplicit` program.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters("The type-level Member-position witness (typically inferred).")]
		#[document_returns("An `ArcRunExplicit` program suspended at the lifted Fresh effect.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::arc_run_explicit::ArcRunExplicit,
		/// };
		///
		/// type Row = CoproductBrand<ArcCoyonedaBrand<SendFreshBrand<ArcBrand, usize>>, CNilBrand>;
		///
		/// let program: ArcRunExplicit<'static, Row, CNilBrand, usize> = ArcRunExplicit::fresh();
		/// let handled: ArcRunExplicit<'static, CNilBrand, CNilBrand, (usize, usize)> =
		/// 	program.run_fresh_with::<usize, _, CNilBrand>(0, |counter| counter + 1);
		/// assert_eq!(handled.extract(), (0, 1));
		/// ```
		#[inline]
		pub fn fresh<Idx>() -> Self
		where
			A: Clone + Send + Sync + 'static,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>):
				Member<ArcCoyoneda<'a, crate::brands::SendFreshBrand<crate::brands::ArcBrand, A>, A>, Idx>,
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
			let effect: crate::types::effects::fresh::SendFresh<'a, crate::brands::ArcBrand, A, A> =
				crate::types::effects::fresh::SendFresh::Fresh(
					<crate::brands::ArcBrand as crate::classes::ToDynSendFn>::new(|id: A| id),
				);
			Self::lift::<crate::brands::SendFreshBrand<crate::brands::ArcBrand, A>, Idx>(effect)
		}
	}
}

fn run_run_fresh_with_tokens() -> TokenStream {
	quote! {
		/// Interprets one Fresh effect by threading a generated-value counter.
		///
		/// Each Fresh operation receives the current value, then the
		/// stored counter advances with `next(current)`.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters(
			"The generated value type.",
			"The type-level Member-position witness for the Fresh effect.",
			"The first-order row brand with the Fresh effect removed."
		)]
		#[document_parameters("The initial generated value.", "The successor function.")]
		#[document_returns("A first-order-only `Run` program returning `(result, final_value)`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::run::Run,
		/// };
		///
		/// type Row = CoproductBrand<CoyonedaBrand<BoxFreshBrand<BoxBrand, usize>>, CNilBrand>;
		///
		/// let program: Run<Row, CNilBrand, usize> = Run::fresh();
		/// let handled: Run<CNilBrand, CNilBrand, (usize, usize)> =
		/// 	program.run_fresh_with::<usize, _, CNilBrand>(10, |counter| counter + 2);
		/// assert_eq!(handled.extract(), (10, 12));
		/// ```
		#[inline]
		pub fn run_fresh_with<Id, Idx, RMinusFresh>(
			self,
			initial: Id,
			next: impl Fn(Id) -> Id + 'static,
		) -> Run<RMinusFresh, CNilBrand, (A, Id)>
		where
			Id: Clone + 'static,
			RMinusFresh: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				Run<R, CNilBrand, A>,
			>): Member<
				Coyoneda<'static, BoxFreshBrand<BoxBrand, Id>, Run<R, CNilBrand, A>>,
				Idx,
				Remainder = Apply!(
					<RMinusFresh as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
						'static,
						Run<R, CNilBrand, A>,
					>
				),
			>, {
			let counter = std::rc::Rc::new(std::cell::RefCell::new(initial));
			let handler_counter = std::rc::Rc::clone(&counter);
			let handled = self.handle_with::<BoxFreshBrand<BoxBrand, Id>, Idx, RMinusFresh>(
				move |op: BoxFresh<'static, BoxBrand, Id, Run<RMinusFresh, CNilBrand, A>>| match op {
					BoxFresh::Fresh(k) => {
						let current = handler_counter.borrow().clone();
						let next_value = next(current.clone());
						{
							*handler_counter.borrow_mut() = next_value;
						}
						k(current)
					}
				},
			);
			handled.map(move |result| (result, counter.borrow().clone()))
		}
	}
}

fn rcrun_run_fresh_with_tokens() -> TokenStream {
	quote! {
		/// Interprets one Fresh effect by threading a generated-value counter.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters(
			"The generated value type.",
			"The type-level Member-position witness for the Fresh effect.",
			"The first-order row brand with the Fresh effect removed."
		)]
		#[document_parameters("The initial generated value.", "The successor function.")]
		#[document_returns("A first-order-only `RcRun` program returning `(result, final_value)`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::rc_run::RcRun,
		/// };
		///
		/// type Row = CoproductBrand<RcCoyonedaBrand<FreshBrand<RcBrand, usize>>, CNilBrand>;
		///
		/// let program: RcRun<Row, CNilBrand, usize> = RcRun::fresh();
		/// let handled: RcRun<CNilBrand, CNilBrand, (usize, usize)> =
		/// 	program.run_fresh_with::<usize, _, CNilBrand>(10, |counter| counter + 2);
		/// assert_eq!(handled.extract(), (10, 12));
		/// ```
		#[inline]
		pub fn run_fresh_with<Id, Idx, RMinusFresh>(
			self,
			initial: Id,
			next: impl Fn(Id) -> Id + 'static,
		) -> RcRun<RMinusFresh, CNilBrand, (A, Id)>
		where
			A: Clone,
			Id: Clone + 'static,
			RMinusFresh: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
			Apply!(<NodeBrand<R, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<NodeBrand<R, CNilBrand>, RcTypeErasedValue>,
			>): Clone,
			Apply!(<NodeBrand<RMinusFresh, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<NodeBrand<RMinusFresh, CNilBrand>, RcTypeErasedValue>,
			>): Clone,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcRun<R, CNilBrand, A>,
			>): Member<
				RcCoyoneda<'static, FreshBrand<RcBrand, Id>, RcRun<R, CNilBrand, A>>,
				Idx,
				Remainder = Apply!(
					<RMinusFresh as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
						'static,
						RcRun<R, CNilBrand, A>,
					>
				),
			>, {
			let counter = std::rc::Rc::new(std::cell::RefCell::new(initial));
			let handler_counter = std::rc::Rc::clone(&counter);
			let handled = self.handle_with::<FreshBrand<RcBrand, Id>, Idx, RMinusFresh>(
				move |op: Fresh<'static, RcBrand, Id, RcRun<RMinusFresh, CNilBrand, A>>| match op {
					Fresh::Fresh(k) => {
						let current = handler_counter.borrow().clone();
						let next_value = next(current.clone());
						{
							*handler_counter.borrow_mut() = next_value;
						}
						(*k)(current)
					}
				},
			);
			handled.map(move |result| (result, counter.borrow().clone()))
		}
	}
}

fn arcrun_run_fresh_with_tokens() -> TokenStream {
	quote! {
		/// Interprets one Fresh effect by threading a generated-value counter.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters(
			"The generated value type.",
			"The type-level Member-position witness for the Fresh effect.",
			"The first-order row brand with the Fresh effect removed."
		)]
		#[document_parameters("The initial generated value.", "The successor function.")]
		#[document_returns("A first-order-only `ArcRun` program returning `(result, final_value)`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::arc_run::ArcRun,
		/// };
		///
		/// type Row = CoproductBrand<ArcCoyonedaBrand<SendFreshBrand<ArcBrand, usize>>, CNilBrand>;
		///
		/// let program: ArcRun<Row, CNilBrand, usize> = ArcRun::fresh();
		/// let handled: ArcRun<CNilBrand, CNilBrand, (usize, usize)> =
		/// 	program.run_fresh_with::<usize, _, CNilBrand>(10, |counter| counter + 2);
		/// assert_eq!(handled.extract(), (10, 12));
		/// ```
		#[inline]
		pub fn run_fresh_with<Id, Idx, RMinusFresh>(
			self,
			initial: Id,
			next: impl Fn(Id) -> Id + Send + Sync + 'static,
		) -> ArcRun<RMinusFresh, CNilBrand, (A, Id)>
		where
			A: Clone + Send + Sync,
			Id: Clone + Send + Sync + 'static,
			R: Kind_cdc7cd43dac7585f + 'static,
			RMinusFresh: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
			NodeBrand<R, CNilBrand>: SendFunctor,
			NodeBrand<RMinusFresh, CNilBrand>: WrapDrop
				+ Kind_cdc7cd43dac7585f<
					Of<'static, ArcFree<NodeBrand<RMinusFresh, CNilBrand>, ArcTypeErasedValue>>: Send
						+ Sync,
				> + SendFunctor,
			Apply!(<NodeBrand<R, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<NodeBrand<R, CNilBrand>, ArcTypeErasedValue>,
			>): Clone,
			Apply!(<NodeBrand<RMinusFresh, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<NodeBrand<RMinusFresh, CNilBrand>, ArcTypeErasedValue>,
			>): Clone,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcRun<R, CNilBrand, A>,
			>): Member<
				ArcCoyoneda<'static, SendFreshBrand<ArcBrand, Id>, ArcRun<R, CNilBrand, A>>,
				Idx,
				Remainder = Apply!(
					<RMinusFresh as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
						'static,
						ArcRun<R, CNilBrand, A>,
					>
				),
			>, {
			let counter = std::sync::Arc::new(std::sync::Mutex::new(initial));
			let handler_counter = std::sync::Arc::clone(&counter);
			let handled = self.handle_with::<SendFreshBrand<ArcBrand, Id>, Idx, RMinusFresh>(
				move |op: SendFresh<'static, ArcBrand, Id, ArcRun<RMinusFresh, CNilBrand, A>>| {
					match op {
						SendFresh::Fresh(k) => {
							let current = {
								let guard = match handler_counter.lock() {
									Ok(guard) => guard,
									Err(poisoned) => poisoned.into_inner(),
								};
								guard.clone()
							};
							let next_value = next(current.clone());
							{
								let mut guard = match handler_counter.lock() {
									Ok(guard) => guard,
									Err(poisoned) => poisoned.into_inner(),
								};
								*guard = next_value;
							}
							(*k)(current)
						}
					}
				},
			);
			handled.map(move |result| {
				let guard = match counter.lock() {
					Ok(guard) => guard,
					Err(poisoned) => poisoned.into_inner(),
				};
				(result, guard.clone())
			})
		}
	}
}

fn run_explicit_run_fresh_with_tokens() -> TokenStream {
	quote! {
		/// Interprets one Fresh effect by threading a generated-value counter.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters(
			"The generated value type.",
			"The type-level Member-position witness for the Fresh effect.",
			"The first-order row brand with the Fresh effect removed."
		)]
		#[document_parameters("The initial generated value.", "The successor function.")]
		#[document_returns(
			"A first-order-only `RunExplicit` program returning `(result, final_value)`."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::run_explicit::RunExplicit,
		/// };
		///
		/// type Row = CoproductBrand<CoyonedaBrand<BoxFreshBrand<BoxBrand, usize>>, CNilBrand>;
		///
		/// let program: RunExplicit<'static, Row, CNilBrand, usize> = RunExplicit::fresh();
		/// let handled: RunExplicit<'static, CNilBrand, CNilBrand, (usize, usize)> =
		/// 	program.run_fresh_with::<usize, _, CNilBrand>(10, |counter| counter + 2);
		/// assert_eq!(handled.extract(), (10, 12));
		/// ```
		#[inline]
		pub fn run_fresh_with<Id, Idx, RMinusFresh>(
			self,
			initial: Id,
			next: impl Fn(Id) -> Id + 'a,
		) -> RunExplicit<'a, RMinusFresh, CNilBrand, (A, Id)>
		where
			Id: Clone + 'static + 'a,
			RMinusFresh: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RunExplicit<'a, R, CNilBrand, A>,
			>): Member<
				Coyoneda<'a, BoxFreshBrand<BoxBrand, Id>, RunExplicit<'a, R, CNilBrand, A>>,
				Idx,
				Remainder = Apply!(
					<RMinusFresh as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
						'a,
						RunExplicit<'a, R, CNilBrand, A>,
					>
				),
			>, {
			let counter = std::rc::Rc::new(std::cell::RefCell::new(initial));
			let handler_counter = std::rc::Rc::clone(&counter);
			let handled = self.handle_with::<BoxFreshBrand<BoxBrand, Id>, Idx, RMinusFresh>(
				move |op: BoxFresh<'a, BoxBrand, Id, RunExplicit<'a, RMinusFresh, CNilBrand, A>>| {
					match op {
						BoxFresh::Fresh(k) => {
							let current = handler_counter.borrow().clone();
							let next_value = next(current.clone());
							{
								*handler_counter.borrow_mut() = next_value;
							}
							k(current)
						}
					}
				},
			);
			handled.map(move |result| (result, counter.borrow().clone()))
		}
	}
}

fn rcrun_explicit_run_fresh_with_tokens() -> TokenStream {
	quote! {
		/// Interprets one Fresh effect by threading a generated-value counter.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters(
			"The generated value type.",
			"The type-level Member-position witness for the Fresh effect.",
			"The first-order row brand with the Fresh effect removed."
		)]
		#[document_parameters("The initial generated value.", "The successor function.")]
		#[document_returns(
			"A first-order-only `RcRunExplicit` program returning `(result, final_value)`."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::rc_run_explicit::RcRunExplicit,
		/// };
		///
		/// type Row = CoproductBrand<RcCoyonedaBrand<FreshBrand<RcBrand, usize>>, CNilBrand>;
		///
		/// let program: RcRunExplicit<'static, Row, CNilBrand, usize> = RcRunExplicit::fresh();
		/// let handled: RcRunExplicit<'static, CNilBrand, CNilBrand, (usize, usize)> =
		/// 	program.run_fresh_with::<usize, _, CNilBrand>(10, |counter| counter + 2);
		/// assert_eq!(handled.extract(), (10, 12));
		/// ```
		#[inline]
		pub fn run_fresh_with<Id, Idx, RMinusFresh>(
			self,
			initial: Id,
			next: impl Fn(Id) -> Id + 'a,
		) -> RcRunExplicit<'a, RMinusFresh, CNilBrand, (A, Id)>
		where
			A: Clone,
			Id: Clone + 'static + 'a,
			RMinusFresh: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
			Apply!(<NodeBrand<R, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, CNilBrand>, A>,
			>): Clone,
			Apply!(<NodeBrand<RMinusFresh, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<RMinusFresh, CNilBrand>, A>,
			>): Clone,
			Apply!(<NodeBrand<RMinusFresh, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<RMinusFresh, CNilBrand>, (A, Id)>,
			>): Clone,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcRunExplicit<'a, R, CNilBrand, A>,
			>): Member<
				RcCoyoneda<'a, FreshBrand<RcBrand, Id>, RcRunExplicit<'a, R, CNilBrand, A>>,
				Idx,
				Remainder = Apply!(
					<RMinusFresh as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
						'a,
						RcRunExplicit<'a, R, CNilBrand, A>,
					>
				),
			>, {
			let counter = std::rc::Rc::new(std::cell::RefCell::new(initial));
			let handler_counter = std::rc::Rc::clone(&counter);
			let handled = self.handle_with::<FreshBrand<RcBrand, Id>, Idx, RMinusFresh>(
				move |op: Fresh<'a, RcBrand, Id, RcRunExplicit<'a, RMinusFresh, CNilBrand, A>>| {
					match op {
						Fresh::Fresh(k) => {
							let current = handler_counter.borrow().clone();
							let next_value = next(current.clone());
							{
								*handler_counter.borrow_mut() = next_value;
							}
							(*k)(current)
						}
					}
				},
			);
			handled.map(move |result| (result, counter.borrow().clone()))
		}
	}
}

fn arcrun_explicit_run_fresh_with_tokens() -> TokenStream {
	quote! {
		/// Interprets one Fresh effect by threading a generated-value counter.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters(
			"The generated value type.",
			"The type-level Member-position witness for the Fresh effect.",
			"The first-order row brand with the Fresh effect removed."
		)]
		#[document_parameters("The initial generated value.", "The successor function.")]
		#[document_returns(
			"A first-order-only `ArcRunExplicit` program returning `(result, final_value)`."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::arc_run_explicit::ArcRunExplicit,
		/// };
		///
		/// type Row = CoproductBrand<ArcCoyonedaBrand<SendFreshBrand<ArcBrand, usize>>, CNilBrand>;
		///
		/// let program: ArcRunExplicit<'static, Row, CNilBrand, usize> = ArcRunExplicit::fresh();
		/// let handled: ArcRunExplicit<'static, CNilBrand, CNilBrand, (usize, usize)> =
		/// 	program.run_fresh_with::<usize, _, CNilBrand>(10, |counter| counter + 2);
		/// assert_eq!(handled.extract(), (10, 12));
		/// ```
		#[inline]
		pub fn run_fresh_with<Id, Idx, RMinusFresh>(
			self,
			initial: Id,
			next: impl Fn(Id) -> Id + Send + Sync + 'a,
		) -> ArcRunExplicit<'a, RMinusFresh, CNilBrand, (A, Id)>
		where
			A: Clone + Send + Sync,
			Id: Clone + Send + Sync + 'static + 'a,
			RMinusFresh: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
			NodeBrand<R, CNilBrand>: SendFunctor,
			NodeBrand<RMinusFresh, CNilBrand>: SendFunctor,
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
			Apply!(<NodeBrand<RMinusFresh, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<RMinusFresh, CNilBrand>, A>,
			>): Clone + Send + Sync,
			Apply!(<NodeBrand<RMinusFresh, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<RMinusFresh, CNilBrand>, (A, Id)>,
			>): Clone + Send + Sync,
			Apply!(<RMinusFresh as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<RMinusFresh, CNilBrand>, A>,
			>): Send + Sync,
			Apply!(<RMinusFresh as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<RMinusFresh, CNilBrand>, (A, Id)>,
			>): Send + Sync,
			Apply!(<RMinusFresh as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, RMinusFresh, CNilBrand, A>,
			>): Send + Sync,
			Apply!(<CNilBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<RMinusFresh, CNilBrand>, A>,
			>): Send + Sync,
			Apply!(<CNilBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<RMinusFresh, CNilBrand>, (A, Id)>,
			>): Send + Sync,
			Apply!(<CNilBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, RMinusFresh, CNilBrand, A>,
			>): Send + Sync,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, R, CNilBrand, A>,
			>): Member<
				ArcCoyoneda<
					'a,
					SendFreshBrand<ArcBrand, Id>,
					ArcRunExplicit<'a, R, CNilBrand, A>,
				>,
				Idx,
				Remainder = Apply!(
					<RMinusFresh as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
						'a,
						ArcRunExplicit<'a, R, CNilBrand, A>,
					>
				),
			>, {
			let counter = std::sync::Arc::new(std::sync::Mutex::new(initial));
			let handler_counter = std::sync::Arc::clone(&counter);
			let handled = self.handle_with::<SendFreshBrand<ArcBrand, Id>, Idx, RMinusFresh>(
				move |op: SendFresh<'a, ArcBrand, Id, ArcRunExplicit<'a, RMinusFresh, CNilBrand, A>>| {
					match op {
						SendFresh::Fresh(k) => {
							let current = {
								let guard = match handler_counter.lock() {
									Ok(guard) => guard,
									Err(poisoned) => poisoned.into_inner(),
								};
								guard.clone()
							};
							let next_value = next(current.clone());
							{
								let mut guard = match handler_counter.lock() {
									Ok(guard) => guard,
									Err(poisoned) => poisoned.into_inner(),
								};
								*guard = next_value;
							}
							(*k)(current)
						}
					}
				},
			);
			handled.map(move |result| {
				let guard = match counter.lock() {
					Ok(guard) => guard,
					Err(poisoned) => poisoned.into_inner(),
				};
				(result, guard.clone())
			})
		}
	}
}

fn run_run_fresh_tokens() -> TokenStream {
	quote! {
		/// Interprets Fresh with a zero-based `usize` counter.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters(
			"The type-level Member-position witness for the Fresh effect.",
			"The first-order row brand with the Fresh effect removed."
		)]
		#[document_returns("A first-order-only `Run` program returning `(result, final_counter)`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::run::Run,
		/// };
		///
		/// type Row = CoproductBrand<CoyonedaBrand<BoxFreshBrand<BoxBrand, usize>>, CNilBrand>;
		///
		/// let program: Run<Row, CNilBrand, usize> = Run::fresh();
		/// let handled: Run<CNilBrand, CNilBrand, (usize, usize)> =
		/// 	program.run_fresh::<_, CNilBrand>();
		/// assert_eq!(handled.extract(), (0, 1));
		/// ```
		#[inline]
		pub fn run_fresh<Idx, RMinusFresh>(self) -> Run<RMinusFresh, CNilBrand, (A, usize)>
		where
			RMinusFresh: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				Run<R, CNilBrand, A>,
			>): Member<
				Coyoneda<'static, BoxFreshBrand<BoxBrand, usize>, Run<R, CNilBrand, A>>,
				Idx,
				Remainder = Apply!(
					<RMinusFresh as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
						'static,
						Run<R, CNilBrand, A>,
					>
				),
			>, {
			self.run_fresh_with::<usize, Idx, RMinusFresh>(0usize, |counter| counter + 1)
		}
	}
}

fn rcrun_run_fresh_tokens() -> TokenStream {
	quote! {
		/// Interprets Fresh with a zero-based `usize` counter.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters(
			"The type-level Member-position witness for the Fresh effect.",
			"The first-order row brand with the Fresh effect removed."
		)]
		#[document_returns("A first-order-only `RcRun` program returning `(result, final_counter)`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::rc_run::RcRun,
		/// };
		///
		/// type Row = CoproductBrand<RcCoyonedaBrand<FreshBrand<RcBrand, usize>>, CNilBrand>;
		///
		/// let program: RcRun<Row, CNilBrand, usize> = RcRun::fresh();
		/// let handled: RcRun<CNilBrand, CNilBrand, (usize, usize)> =
		/// 	program.run_fresh::<_, CNilBrand>();
		/// assert_eq!(handled.extract(), (0, 1));
		/// ```
		#[inline]
		pub fn run_fresh<Idx, RMinusFresh>(self) -> RcRun<RMinusFresh, CNilBrand, (A, usize)>
		where
			A: Clone,
			RMinusFresh: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
			Apply!(<NodeBrand<R, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<NodeBrand<R, CNilBrand>, RcTypeErasedValue>,
			>): Clone,
			Apply!(<NodeBrand<RMinusFresh, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<NodeBrand<RMinusFresh, CNilBrand>, RcTypeErasedValue>,
			>): Clone,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcRun<R, CNilBrand, A>,
			>): Member<
				RcCoyoneda<'static, FreshBrand<RcBrand, usize>, RcRun<R, CNilBrand, A>>,
				Idx,
				Remainder = Apply!(
					<RMinusFresh as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
						'static,
						RcRun<R, CNilBrand, A>,
					>
				),
			>, {
			self.run_fresh_with::<usize, Idx, RMinusFresh>(0usize, |counter| counter + 1)
		}
	}
}

fn arcrun_run_fresh_tokens() -> TokenStream {
	quote! {
		/// Interprets Fresh with a zero-based `usize` counter.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters(
			"The type-level Member-position witness for the Fresh effect.",
			"The first-order row brand with the Fresh effect removed."
		)]
		#[document_returns("A first-order-only `ArcRun` program returning `(result, final_counter)`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::arc_run::ArcRun,
		/// };
		///
		/// type Row = CoproductBrand<ArcCoyonedaBrand<SendFreshBrand<ArcBrand, usize>>, CNilBrand>;
		///
		/// let program: ArcRun<Row, CNilBrand, usize> = ArcRun::fresh();
		/// let handled: ArcRun<CNilBrand, CNilBrand, (usize, usize)> =
		/// 	program.run_fresh::<_, CNilBrand>();
		/// assert_eq!(handled.extract(), (0, 1));
		/// ```
		#[inline]
		pub fn run_fresh<Idx, RMinusFresh>(self) -> ArcRun<RMinusFresh, CNilBrand, (A, usize)>
		where
			A: Clone + Send + Sync,
			R: Kind_cdc7cd43dac7585f + 'static,
			RMinusFresh: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
			NodeBrand<R, CNilBrand>: SendFunctor,
			NodeBrand<RMinusFresh, CNilBrand>: WrapDrop
				+ Kind_cdc7cd43dac7585f<
					Of<'static, ArcFree<NodeBrand<RMinusFresh, CNilBrand>, ArcTypeErasedValue>>: Send
						+ Sync,
				> + SendFunctor,
			Apply!(<NodeBrand<R, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<NodeBrand<R, CNilBrand>, ArcTypeErasedValue>,
			>): Clone,
			Apply!(<NodeBrand<RMinusFresh, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<NodeBrand<RMinusFresh, CNilBrand>, ArcTypeErasedValue>,
			>): Clone,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcRun<R, CNilBrand, A>,
			>): Member<
				ArcCoyoneda<'static, SendFreshBrand<ArcBrand, usize>, ArcRun<R, CNilBrand, A>>,
				Idx,
				Remainder = Apply!(
					<RMinusFresh as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
						'static,
						ArcRun<R, CNilBrand, A>,
					>
				),
			>, {
			self.run_fresh_with::<usize, Idx, RMinusFresh>(0usize, |counter| counter + 1)
		}
	}
}

fn run_explicit_run_fresh_tokens() -> TokenStream {
	quote! {
		/// Interprets Fresh with a zero-based `usize` counter.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters(
			"The type-level Member-position witness for the Fresh effect.",
			"The first-order row brand with the Fresh effect removed."
		)]
		#[document_returns(
			"A first-order-only `RunExplicit` program returning `(result, final_counter)`."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::run_explicit::RunExplicit,
		/// };
		///
		/// type Row = CoproductBrand<CoyonedaBrand<BoxFreshBrand<BoxBrand, usize>>, CNilBrand>;
		///
		/// let program: RunExplicit<'static, Row, CNilBrand, usize> = RunExplicit::fresh();
		/// let handled: RunExplicit<'static, CNilBrand, CNilBrand, (usize, usize)> =
		/// 	program.run_fresh::<_, CNilBrand>();
		/// assert_eq!(handled.extract(), (0, 1));
		/// ```
		#[inline]
		pub fn run_fresh<Idx, RMinusFresh>(
			self
		) -> RunExplicit<'a, RMinusFresh, CNilBrand, (A, usize)>
		where
			RMinusFresh: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RunExplicit<'a, R, CNilBrand, A>,
			>): Member<
				Coyoneda<'a, BoxFreshBrand<BoxBrand, usize>, RunExplicit<'a, R, CNilBrand, A>>,
				Idx,
				Remainder = Apply!(
					<RMinusFresh as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
						'a,
						RunExplicit<'a, R, CNilBrand, A>,
					>
				),
			>, {
			self.run_fresh_with::<usize, Idx, RMinusFresh>(0usize, |counter| counter + 1)
		}
	}
}

fn rcrun_explicit_run_fresh_tokens() -> TokenStream {
	quote! {
		/// Interprets Fresh with a zero-based `usize` counter.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters(
			"The type-level Member-position witness for the Fresh effect.",
			"The first-order row brand with the Fresh effect removed."
		)]
		#[document_returns(
			"A first-order-only `RcRunExplicit` program returning `(result, final_counter)`."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::rc_run_explicit::RcRunExplicit,
		/// };
		///
		/// type Row = CoproductBrand<RcCoyonedaBrand<FreshBrand<RcBrand, usize>>, CNilBrand>;
		///
		/// let program: RcRunExplicit<'static, Row, CNilBrand, usize> = RcRunExplicit::fresh();
		/// let handled: RcRunExplicit<'static, CNilBrand, CNilBrand, (usize, usize)> =
		/// 	program.run_fresh::<_, CNilBrand>();
		/// assert_eq!(handled.extract(), (0, 1));
		/// ```
		#[inline]
		pub fn run_fresh<Idx, RMinusFresh>(
			self
		) -> RcRunExplicit<'a, RMinusFresh, CNilBrand, (A, usize)>
		where
			A: Clone,
			RMinusFresh: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
			Apply!(<NodeBrand<R, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, CNilBrand>, A>,
			>): Clone,
			Apply!(<NodeBrand<RMinusFresh, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<RMinusFresh, CNilBrand>, A>,
			>): Clone,
			Apply!(<NodeBrand<RMinusFresh, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<RMinusFresh, CNilBrand>, (A, usize)>,
			>): Clone,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcRunExplicit<'a, R, CNilBrand, A>,
			>): Member<
				RcCoyoneda<'a, FreshBrand<RcBrand, usize>, RcRunExplicit<'a, R, CNilBrand, A>>,
				Idx,
				Remainder = Apply!(
					<RMinusFresh as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
						'a,
						RcRunExplicit<'a, R, CNilBrand, A>,
					>
				),
			>, {
			self.run_fresh_with::<usize, Idx, RMinusFresh>(0usize, |counter| counter + 1)
		}
	}
}

fn arcrun_explicit_run_fresh_tokens() -> TokenStream {
	quote! {
		/// Interprets Fresh with a zero-based `usize` counter.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters(
			"The type-level Member-position witness for the Fresh effect.",
			"The first-order row brand with the Fresh effect removed."
		)]
		#[document_returns(
			"A first-order-only `ArcRunExplicit` program returning `(result, final_counter)`."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::arc_run_explicit::ArcRunExplicit,
		/// };
		///
		/// type Row = CoproductBrand<ArcCoyonedaBrand<SendFreshBrand<ArcBrand, usize>>, CNilBrand>;
		///
		/// let program: ArcRunExplicit<'static, Row, CNilBrand, usize> = ArcRunExplicit::fresh();
		/// let handled: ArcRunExplicit<'static, CNilBrand, CNilBrand, (usize, usize)> =
		/// 	program.run_fresh::<_, CNilBrand>();
		/// assert_eq!(handled.extract(), (0, 1));
		/// ```
		#[inline]
		pub fn run_fresh<Idx, RMinusFresh>(
			self
		) -> ArcRunExplicit<'a, RMinusFresh, CNilBrand, (A, usize)>
		where
			A: Clone + Send + Sync,
			RMinusFresh: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
			NodeBrand<R, CNilBrand>: SendFunctor,
			NodeBrand<RMinusFresh, CNilBrand>: SendFunctor,
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
			Apply!(<NodeBrand<RMinusFresh, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<RMinusFresh, CNilBrand>, A>,
			>): Clone + Send + Sync,
			Apply!(<NodeBrand<RMinusFresh, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<RMinusFresh, CNilBrand>, (A, usize)>,
			>): Clone + Send + Sync,
			Apply!(<RMinusFresh as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<RMinusFresh, CNilBrand>, A>,
			>): Send + Sync,
			Apply!(<RMinusFresh as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<RMinusFresh, CNilBrand>, (A, usize)>,
			>): Send + Sync,
			Apply!(<RMinusFresh as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, RMinusFresh, CNilBrand, A>,
			>): Send + Sync,
			Apply!(<CNilBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<RMinusFresh, CNilBrand>, A>,
			>): Send + Sync,
			Apply!(<CNilBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<RMinusFresh, CNilBrand>, (A, usize)>,
			>): Send + Sync,
			Apply!(<CNilBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, RMinusFresh, CNilBrand, A>,
			>): Send + Sync,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, R, CNilBrand, A>,
			>): Member<
				ArcCoyoneda<
					'a,
					SendFreshBrand<ArcBrand, usize>,
					ArcRunExplicit<'a, R, CNilBrand, A>,
				>,
				Idx,
				Remainder = Apply!(
					<RMinusFresh as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
						'a,
						ArcRunExplicit<'a, R, CNilBrand, A>,
					>
				),
			>, {
			self.run_fresh_with::<usize, Idx, RMinusFresh>(0usize, |counter| counter + 1)
		}
	}
}
