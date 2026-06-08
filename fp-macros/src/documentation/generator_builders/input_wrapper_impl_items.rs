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

pub(super) fn input_wrapper_impl_items_from_descriptor(
	wrapper: WrapperName,
	method: RunWrapperMethod,
) -> Option<syn::Result<Vec<ImplItem>>> {
	generator_descriptors::method_spec(EffectName::Input, method)?;
	generator_descriptors::wrapper_spec(wrapper)?;

	let tokens = match (wrapper, method) {
		(WrapperName::Run, RunWrapperMethod::Input) => run_input_tokens(),
		(WrapperName::Run, RunWrapperMethod::RunInputSeq) => run_run_input_seq_tokens(),
		(WrapperName::RcRun, RunWrapperMethod::Input) => rcrun_input_tokens(),
		(WrapperName::RcRun, RunWrapperMethod::RunInputSeq) => rcrun_run_input_seq_tokens(),
		(WrapperName::ArcRun, RunWrapperMethod::Input) => arcrun_input_tokens(),
		(WrapperName::ArcRun, RunWrapperMethod::RunInputSeq) => arcrun_run_input_seq_tokens(),
		(WrapperName::RunExplicit, RunWrapperMethod::Input) => run_explicit_input_tokens(),
		(WrapperName::RunExplicit, RunWrapperMethod::RunInputSeq) =>
			run_explicit_run_input_seq_tokens(),
		(WrapperName::RcRunExplicit, RunWrapperMethod::Input) => rcrun_explicit_input_tokens(),
		(WrapperName::RcRunExplicit, RunWrapperMethod::RunInputSeq) =>
			rcrun_explicit_run_input_seq_tokens(),
		(WrapperName::ArcRunExplicit, RunWrapperMethod::Input) => arcrun_explicit_input_tokens(),
		(WrapperName::ArcRunExplicit, RunWrapperMethod::RunInputSeq) =>
			arcrun_explicit_run_input_seq_tokens(),
		_ => return None,
	};

	Some(impl_items_from_tokens(tokens))
}

fn run_input_tokens() -> TokenStream {
	quote! {
		/// Lifts an Input effect into the Run program.
		///
		/// The program asks the handler for one input value and returns
		/// that value as its result.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters("The type-level Member-position witness (typically inferred).")]
		#[document_returns("A `Run` program suspended at the lifted Input effect.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::run::Run,
		/// };
		///
		/// type Row = CoproductBrand<CoyonedaBrand<BoxInputBrand<BoxBrand, Option<i32>>>, CNilBrand>;
		///
		/// let program: Run<Row, CNilBrand, Option<i32>> = Run::input();
		/// let handled: Run<CNilBrand, CNilBrand, Option<i32>> =
		/// 	program.run_input_seq::<i32, _, CNilBrand>([7]);
		/// assert_eq!(handled.extract(), Some(7));
		/// ```
		#[inline]
		pub fn input<Idx>() -> Self
		where
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, A>):
				crate::types::effects::member::Member<
					crate::types::Coyoneda<
						'static,
						crate::brands::BoxInputBrand<crate::brands::BoxBrand, A>,
						A,
					>,
					Idx,
				>, {
			let effect: crate::types::effects::input::BoxInput<'static, crate::brands::BoxBrand, A, A> =
				crate::types::effects::input::BoxInput::Input(
					<crate::brands::BoxBrand as crate::classes::ToDynFnOnce>::new(|item: A| item),
				);
			Self::lift::<crate::brands::BoxInputBrand<crate::brands::BoxBrand, A>, Idx>(effect)
		}
	}
}

fn rcrun_input_tokens() -> TokenStream {
	quote! {
		/// Lifts an Input effect into the `RcRun` program.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters("The type-level Member-position witness (typically inferred).")]
		#[document_returns("An `RcRun` program suspended at the lifted Input effect.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::rc_run::RcRun,
		/// };
		///
		/// type Row = CoproductBrand<RcCoyonedaBrand<InputBrand<RcBrand, Option<i32>>>, CNilBrand>;
		///
		/// let program: RcRun<Row, CNilBrand, Option<i32>> = RcRun::input();
		/// let handled: RcRun<CNilBrand, CNilBrand, Option<i32>> =
		/// 	program.run_input_seq::<i32, _, CNilBrand>([7]);
		/// assert_eq!(handled.extract(), Some(7));
		/// ```
		#[inline]
		pub fn input<Idx>() -> Self
		where
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, A>):
				Member<RcCoyoneda<'static, crate::brands::InputBrand<crate::brands::RcBrand, A>, A>, Idx>,
			Apply!(<NodeBrand<R, ScopedRow> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<NodeBrand<R, ScopedRow>, crate::types::rc_free::RcTypeErasedValue>,
			>): Clone, {
			let effect: crate::types::effects::input::Input<'static, crate::brands::RcBrand, A, A> =
				crate::types::effects::input::Input::Input(
					<crate::brands::RcBrand as crate::classes::ToDynCloneFn>::new(|item: A| item),
				);
			Self::lift::<crate::brands::InputBrand<crate::brands::RcBrand, A>, Idx>(effect)
		}
	}
}

fn arcrun_input_tokens() -> TokenStream {
	quote! {
		/// Lifts an Input effect into the `ArcRun` program.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters("The type-level Member-position witness (typically inferred).")]
		#[document_returns("An `ArcRun` program suspended at the lifted Input effect.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::arc_run::ArcRun,
		/// };
		///
		/// type Row = CoproductBrand<ArcCoyonedaBrand<SendInputBrand<ArcBrand, Option<i32>>>, CNilBrand>;
		///
		/// let program: ArcRun<Row, CNilBrand, Option<i32>> = ArcRun::input();
		/// let handled: ArcRun<CNilBrand, CNilBrand, Option<i32>> =
		/// 	program.run_input_seq::<i32, _, CNilBrand>([7]);
		/// assert_eq!(handled.extract(), Some(7));
		/// ```
		#[inline]
		pub fn input<Idx>() -> Self
		where
			A: Clone + Send + Sync,
			NodeBrand<R, ScopedRow>: SendFunctor,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, A>): Member<
				ArcCoyoneda<'static, crate::brands::SendInputBrand<crate::brands::ArcBrand, A>, A>,
				Idx,
			>,
			Apply!(<NodeBrand<R, ScopedRow> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<NodeBrand<R, ScopedRow>, ArcTypeErasedValue>,
			>): Clone, {
			let effect: crate::types::effects::input::SendInput<
				'static,
				crate::brands::ArcBrand,
				A,
				A,
			> = crate::types::effects::input::SendInput::Input(
				<crate::brands::ArcBrand as crate::classes::ToDynSendFn>::new(|item: A| item),
			);
			Self::lift::<crate::brands::SendInputBrand<crate::brands::ArcBrand, A>, Idx>(effect)
		}
	}
}

fn run_explicit_input_tokens() -> TokenStream {
	quote! {
		/// Lifts an Input effect into the `RunExplicit` program.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters("The type-level Member-position witness (typically inferred).")]
		#[document_returns("A `RunExplicit` program suspended at the lifted Input effect.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::run_explicit::RunExplicit,
		/// };
		///
		/// type Row = CoproductBrand<CoyonedaBrand<BoxInputBrand<BoxBrand, Option<i32>>>, CNilBrand>;
		///
		/// let program: RunExplicit<'static, Row, CNilBrand, Option<i32>> = RunExplicit::input();
		/// let handled: RunExplicit<'static, CNilBrand, CNilBrand, Option<i32>> =
		/// 	program.run_input_seq::<i32, _, CNilBrand>([7]);
		/// assert_eq!(handled.extract(), Some(7));
		/// ```
		#[inline]
		pub fn input<Idx>() -> Self
		where
			A: 'static,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>):
				Member<Coyoneda<'a, crate::brands::BoxInputBrand<crate::brands::BoxBrand, A>, A>, Idx>, {
			let effect: crate::types::effects::input::BoxInput<'a, crate::brands::BoxBrand, A, A> =
				crate::types::effects::input::BoxInput::Input(
					<crate::brands::BoxBrand as crate::classes::ToDynFnOnce>::new(|item: A| item),
				);
			Self::lift::<crate::brands::BoxInputBrand<crate::brands::BoxBrand, A>, Idx>(effect)
		}
	}
}

fn rcrun_explicit_input_tokens() -> TokenStream {
	quote! {
		/// Lifts an Input effect into the `RcRunExplicit` program.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters("The type-level Member-position witness (typically inferred).")]
		#[document_returns("An `RcRunExplicit` program suspended at the lifted Input effect.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::rc_run_explicit::RcRunExplicit,
		/// };
		///
		/// type Row = CoproductBrand<RcCoyonedaBrand<InputBrand<RcBrand, Option<i32>>>, CNilBrand>;
		///
		/// let program: RcRunExplicit<'static, Row, CNilBrand, Option<i32>> = RcRunExplicit::input();
		/// let handled: RcRunExplicit<'static, CNilBrand, CNilBrand, Option<i32>> =
		/// 	program.run_input_seq::<i32, _, CNilBrand>([7]);
		/// assert_eq!(handled.extract(), Some(7));
		/// ```
		#[inline]
		pub fn input<Idx>() -> Self
		where
			A: Clone + 'static,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>):
				Member<RcCoyoneda<'a, crate::brands::InputBrand<crate::brands::RcBrand, A>, A>, Idx>, {
			let effect: crate::types::effects::input::Input<'a, crate::brands::RcBrand, A, A> =
				crate::types::effects::input::Input::Input(
					<crate::brands::RcBrand as crate::classes::ToDynCloneFn>::new(|item: A| item),
				);
			Self::lift::<crate::brands::InputBrand<crate::brands::RcBrand, A>, Idx>(effect)
		}
	}
}

fn arcrun_explicit_input_tokens() -> TokenStream {
	quote! {
		/// Lifts an Input effect into the `ArcRunExplicit` program.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters("The type-level Member-position witness (typically inferred).")]
		#[document_returns("An `ArcRunExplicit` program suspended at the lifted Input effect.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::arc_run_explicit::ArcRunExplicit,
		/// };
		///
		/// type Row = CoproductBrand<ArcCoyonedaBrand<SendInputBrand<ArcBrand, Option<i32>>>, CNilBrand>;
		///
		/// let program: ArcRunExplicit<'static, Row, CNilBrand, Option<i32>> = ArcRunExplicit::input();
		/// let handled: ArcRunExplicit<'static, CNilBrand, CNilBrand, Option<i32>> =
		/// 	program.run_input_seq::<i32, _, CNilBrand>([7]);
		/// assert_eq!(handled.extract(), Some(7));
		/// ```
		#[inline]
		pub fn input<Idx>() -> Self
		where
			A: Clone + Send + Sync + 'static,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>):
				Member<ArcCoyoneda<'a, crate::brands::SendInputBrand<crate::brands::ArcBrand, A>, A>, Idx>,
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
			let effect: crate::types::effects::input::SendInput<'a, crate::brands::ArcBrand, A, A> =
				crate::types::effects::input::SendInput::Input(
					<crate::brands::ArcBrand as crate::classes::ToDynSendFn>::new(|item: A| item),
				);
			Self::lift::<crate::brands::SendInputBrand<crate::brands::ArcBrand, A>, Idx>(effect)
		}
	}
}

fn run_run_input_seq_tokens() -> TokenStream {
	quote! {
		/// Interprets Input from a finite sequence.
		///
		/// Each Input operation receives `Some(item)` while the
		/// sequence has values remaining and `None` after exhaustion.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters(
			"The input item type.",
			"The type-level Member-position witness for the Input effect.",
			"The first-order row brand with the Input effect removed."
		)]
		#[document_parameters("The finite input sequence.")]
		#[document_returns("A first-order-only `Run` program with the Input effect removed.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::run::Run,
		/// };
		///
		/// type Row = CoproductBrand<CoyonedaBrand<BoxInputBrand<BoxBrand, Option<i32>>>, CNilBrand>;
		///
		/// let program: Run<Row, CNilBrand, (Option<i32>, Option<i32>)> =
		/// 	Run::<Row, CNilBrand, Option<i32>>::input()
		/// 		.bind(|first| Run::<Row, CNilBrand, Option<i32>>::input().map(move |second| (first, second)));
		/// let handled: Run<CNilBrand, CNilBrand, (Option<i32>, Option<i32>)> =
		/// 	program.run_input_seq::<i32, _, CNilBrand>([7]);
		/// assert_eq!(handled.extract(), (Some(7), None));
		/// ```
		#[inline]
		pub fn run_input_seq<Item, Idx, RMinusInput>(
			self,
			input: impl IntoIterator<Item = Item>,
		) -> Run<RMinusInput, CNilBrand, A>
		where
			Item: 'static,
			RMinusInput: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				Run<R, CNilBrand, A>,
			>): Member<
				Coyoneda<'static, BoxInputBrand<BoxBrand, Option<Item>>, Run<R, CNilBrand, A>>,
				Idx,
				Remainder = Apply!(
					<RMinusInput as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
						'static,
						Run<R, CNilBrand, A>,
					>
				),
			>, {
			let input =
				std::rc::Rc::new(std::cell::RefCell::new(input.into_iter().collect::<std::collections::VecDeque<_>>()));
			let handler_input = std::rc::Rc::clone(&input);
			self.handle_with::<BoxInputBrand<BoxBrand, Option<Item>>, Idx, RMinusInput>(
				move |op: BoxInput<'static, BoxBrand, Option<Item>, Run<RMinusInput, CNilBrand, A>>| {
					match op {
						BoxInput::Input(k) => {
							let next_item = {
								handler_input.borrow_mut().pop_front()
							};
							k(next_item)
						}
					}
				},
			)
		}
	}
}

fn rcrun_run_input_seq_tokens() -> TokenStream {
	quote! {
		/// Interprets Input from a finite sequence.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters(
			"The input item type.",
			"The type-level Member-position witness for the Input effect.",
			"The first-order row brand with the Input effect removed."
		)]
		#[document_parameters("The finite input sequence.")]
		#[document_returns("A first-order-only `RcRun` program with the Input effect removed.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::rc_run::RcRun,
		/// };
		///
		/// type Row = CoproductBrand<RcCoyonedaBrand<InputBrand<RcBrand, Option<i32>>>, CNilBrand>;
		///
		/// let program: RcRun<Row, CNilBrand, (Option<i32>, Option<i32>)> =
		/// 	RcRun::<Row, CNilBrand, Option<i32>>::input()
		/// 		.bind(|first| RcRun::<Row, CNilBrand, Option<i32>>::input().map(move |second| (first, second)));
		/// let handled: RcRun<CNilBrand, CNilBrand, (Option<i32>, Option<i32>)> =
		/// 	program.run_input_seq::<i32, _, CNilBrand>([7]);
		/// assert_eq!(handled.extract(), (Some(7), None));
		/// ```
		#[inline]
		pub fn run_input_seq<Item, Idx, RMinusInput>(
			self,
			input: impl IntoIterator<Item = Item>,
		) -> RcRun<RMinusInput, CNilBrand, A>
		where
			A: Clone,
			Item: 'static,
			RMinusInput: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
			Apply!(<NodeBrand<R, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<NodeBrand<R, CNilBrand>, RcTypeErasedValue>,
			>): Clone,
			Apply!(<NodeBrand<RMinusInput, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<NodeBrand<RMinusInput, CNilBrand>, RcTypeErasedValue>,
			>): Clone,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcRun<R, CNilBrand, A>,
			>): Member<
				RcCoyoneda<'static, InputBrand<RcBrand, Option<Item>>, RcRun<R, CNilBrand, A>>,
				Idx,
				Remainder = Apply!(
					<RMinusInput as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
						'static,
						RcRun<R, CNilBrand, A>,
					>
				),
			>, {
			let input =
				std::rc::Rc::new(std::cell::RefCell::new(input.into_iter().collect::<std::collections::VecDeque<_>>()));
			let handler_input = std::rc::Rc::clone(&input);
			self.handle_with::<InputBrand<RcBrand, Option<Item>>, Idx, RMinusInput>(
				move |op: Input<'static, RcBrand, Option<Item>, RcRun<RMinusInput, CNilBrand, A>>| {
					match op {
						Input::Input(k) => {
							let next_item = {
								handler_input.borrow_mut().pop_front()
							};
							(*k)(next_item)
						}
					}
				},
			)
		}
	}
}

fn arcrun_run_input_seq_tokens() -> TokenStream {
	quote! {
		/// Interprets Input from a finite sequence.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters(
			"The input item type.",
			"The type-level Member-position witness for the Input effect.",
			"The first-order row brand with the Input effect removed."
		)]
		#[document_parameters("The finite input sequence.")]
		#[document_returns("A first-order-only `ArcRun` program with the Input effect removed.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::arc_run::ArcRun,
		/// };
		///
		/// type Row = CoproductBrand<ArcCoyonedaBrand<SendInputBrand<ArcBrand, Option<i32>>>, CNilBrand>;
		///
		/// let program: ArcRun<Row, CNilBrand, (Option<i32>, Option<i32>)> =
		/// 	ArcRun::<Row, CNilBrand, Option<i32>>::input()
		/// 		.bind(|first| ArcRun::<Row, CNilBrand, Option<i32>>::input().map(move |second| (first, second)));
		/// let handled: ArcRun<CNilBrand, CNilBrand, (Option<i32>, Option<i32>)> =
		/// 	program.run_input_seq::<i32, _, CNilBrand>([7]);
		/// assert_eq!(handled.extract(), (Some(7), None));
		/// ```
		#[inline]
		pub fn run_input_seq<Item, Idx, RMinusInput>(
			self,
			input: impl IntoIterator<Item = Item>,
		) -> ArcRun<RMinusInput, CNilBrand, A>
		where
			A: Clone + Send + Sync,
			Item: Send + Sync + 'static,
			R: Kind_cdc7cd43dac7585f + 'static,
			RMinusInput: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
			NodeBrand<R, CNilBrand>: SendFunctor,
			NodeBrand<RMinusInput, CNilBrand>: WrapDrop
				+ Kind_cdc7cd43dac7585f<
					Of<'static, ArcFree<NodeBrand<RMinusInput, CNilBrand>, ArcTypeErasedValue>>: Send
						+ Sync,
				> + SendFunctor,
			Apply!(<NodeBrand<R, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<NodeBrand<R, CNilBrand>, ArcTypeErasedValue>,
			>): Clone,
			Apply!(<NodeBrand<RMinusInput, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<NodeBrand<RMinusInput, CNilBrand>, ArcTypeErasedValue>,
			>): Clone,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcRun<R, CNilBrand, A>,
			>): Member<
				ArcCoyoneda<'static, SendInputBrand<ArcBrand, Option<Item>>, ArcRun<R, CNilBrand, A>>,
				Idx,
				Remainder = Apply!(
					<RMinusInput as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
						'static,
						ArcRun<R, CNilBrand, A>,
					>
				),
			>, {
			let input = std::sync::Arc::new(std::sync::Mutex::new(
				input.into_iter().collect::<std::collections::VecDeque<_>>(),
			));
			let handler_input = std::sync::Arc::clone(&input);
			self.handle_with::<SendInputBrand<ArcBrand, Option<Item>>, Idx, RMinusInput>(
				move |op: SendInput<'static, ArcBrand, Option<Item>, ArcRun<RMinusInput, CNilBrand, A>>| {
					match op {
						SendInput::Input(k) => {
							let next_item = {
								let mut guard = match handler_input.lock() {
									Ok(guard) => guard,
									Err(poisoned) => poisoned.into_inner(),
								};
								guard.pop_front()
							};
							(*k)(next_item)
						}
					}
				},
			)
		}
	}
}

fn run_explicit_run_input_seq_tokens() -> TokenStream {
	quote! {
		/// Interprets Input from a finite sequence.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters(
			"The input item type.",
			"The type-level Member-position witness for the Input effect.",
			"The first-order row brand with the Input effect removed."
		)]
		#[document_parameters("The finite input sequence.")]
		#[document_returns("A first-order-only `RunExplicit` program with the Input effect removed.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::run_explicit::RunExplicit,
		/// };
		///
		/// type Row = CoproductBrand<CoyonedaBrand<BoxInputBrand<BoxBrand, Option<i32>>>, CNilBrand>;
		///
		/// let program: RunExplicit<'static, Row, CNilBrand, (Option<i32>, Option<i32>)> =
		/// 	RunExplicit::<'static, Row, CNilBrand, Option<i32>>::input()
		/// 		.bind(|first| {
		/// 			RunExplicit::<'static, Row, CNilBrand, Option<i32>>::input()
		/// 				.map(move |second| (first, second))
		/// 		});
		/// let handled: RunExplicit<'static, CNilBrand, CNilBrand, (Option<i32>, Option<i32>)> =
		/// 	program.run_input_seq::<i32, _, CNilBrand>([7]);
		/// assert_eq!(handled.extract(), (Some(7), None));
		/// ```
		#[inline]
		pub fn run_input_seq<Item, Idx, RMinusInput>(
			self,
			input: impl IntoIterator<Item = Item>,
		) -> RunExplicit<'a, RMinusInput, CNilBrand, A>
		where
			Item: 'static + 'a,
			RMinusInput: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RunExplicit<'a, R, CNilBrand, A>,
			>): Member<
				Coyoneda<'a, BoxInputBrand<BoxBrand, Option<Item>>, RunExplicit<'a, R, CNilBrand, A>>,
				Idx,
				Remainder = Apply!(
					<RMinusInput as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
						'a,
						RunExplicit<'a, R, CNilBrand, A>,
					>
				),
			>, {
			let input =
				std::rc::Rc::new(std::cell::RefCell::new(input.into_iter().collect::<std::collections::VecDeque<_>>()));
			let handler_input = std::rc::Rc::clone(&input);
			self.handle_with::<BoxInputBrand<BoxBrand, Option<Item>>, Idx, RMinusInput>(
				move |op: BoxInput<'a, BoxBrand, Option<Item>, RunExplicit<'a, RMinusInput, CNilBrand, A>>| {
					match op {
						BoxInput::Input(k) => {
							let next_item = {
								handler_input.borrow_mut().pop_front()
							};
							k(next_item)
						}
					}
				},
			)
		}
	}
}

fn rcrun_explicit_run_input_seq_tokens() -> TokenStream {
	quote! {
		/// Interprets Input from a finite sequence.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters(
			"The input item type.",
			"The type-level Member-position witness for the Input effect.",
			"The first-order row brand with the Input effect removed."
		)]
		#[document_parameters("The finite input sequence.")]
		#[document_returns(
			"A first-order-only `RcRunExplicit` program with the Input effect removed."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::rc_run_explicit::RcRunExplicit,
		/// };
		///
		/// type Row = CoproductBrand<RcCoyonedaBrand<InputBrand<RcBrand, Option<i32>>>, CNilBrand>;
		///
		/// let program: RcRunExplicit<'static, Row, CNilBrand, (Option<i32>, Option<i32>)> =
		/// 	RcRunExplicit::<'static, Row, CNilBrand, Option<i32>>::input()
		/// 		.bind(|first| {
		/// 			RcRunExplicit::<'static, Row, CNilBrand, Option<i32>>::input()
		/// 				.map(move |second| (first, second))
		/// 		});
		/// let handled: RcRunExplicit<'static, CNilBrand, CNilBrand, (Option<i32>, Option<i32>)> =
		/// 	program.run_input_seq::<i32, _, CNilBrand>([7]);
		/// assert_eq!(handled.extract(), (Some(7), None));
		/// ```
		#[inline]
		pub fn run_input_seq<Item, Idx, RMinusInput>(
			self,
			input: impl IntoIterator<Item = Item>,
		) -> RcRunExplicit<'a, RMinusInput, CNilBrand, A>
		where
			A: Clone,
			Item: 'static + 'a,
			RMinusInput: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
			Apply!(<NodeBrand<R, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, CNilBrand>, A>,
			>): Clone,
			Apply!(<NodeBrand<RMinusInput, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<RMinusInput, CNilBrand>, A>,
			>): Clone,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcRunExplicit<'a, R, CNilBrand, A>,
			>): Member<
				RcCoyoneda<'a, InputBrand<RcBrand, Option<Item>>, RcRunExplicit<'a, R, CNilBrand, A>>,
				Idx,
				Remainder = Apply!(
					<RMinusInput as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
						'a,
						RcRunExplicit<'a, R, CNilBrand, A>,
					>
				),
			>, {
			let input =
				std::rc::Rc::new(std::cell::RefCell::new(input.into_iter().collect::<std::collections::VecDeque<_>>()));
			let handler_input = std::rc::Rc::clone(&input);
			self.handle_with::<InputBrand<RcBrand, Option<Item>>, Idx, RMinusInput>(
				move |op: Input<'a, RcBrand, Option<Item>, RcRunExplicit<'a, RMinusInput, CNilBrand, A>>| {
					match op {
						Input::Input(k) => {
							let next_item = {
								handler_input.borrow_mut().pop_front()
							};
							(*k)(next_item)
						}
					}
				},
			)
		}
	}
}

fn arcrun_explicit_run_input_seq_tokens() -> TokenStream {
	quote! {
		/// Interprets Input from a finite sequence.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters(
			"The input item type.",
			"The type-level Member-position witness for the Input effect.",
			"The first-order row brand with the Input effect removed."
		)]
		#[document_parameters("The finite input sequence.")]
		#[document_returns(
			"A first-order-only `ArcRunExplicit` program with the Input effect removed."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::arc_run_explicit::ArcRunExplicit,
		/// };
		///
		/// type Row = CoproductBrand<ArcCoyonedaBrand<SendInputBrand<ArcBrand, Option<i32>>>, CNilBrand>;
		///
		/// let program: ArcRunExplicit<'static, Row, CNilBrand, (Option<i32>, Option<i32>)> =
		/// 	ArcRunExplicit::<'static, Row, CNilBrand, Option<i32>>::input()
		/// 		.bind(|first| {
		/// 			ArcRunExplicit::<'static, Row, CNilBrand, Option<i32>>::input()
		/// 				.map(move |second| (first, second))
		/// 		});
		/// let handled: ArcRunExplicit<'static, CNilBrand, CNilBrand, (Option<i32>, Option<i32>)> =
		/// 	program.run_input_seq::<i32, _, CNilBrand>([7]);
		/// assert_eq!(handled.extract(), (Some(7), None));
		/// ```
		#[inline]
		pub fn run_input_seq<Item, Idx, RMinusInput>(
			self,
			input: impl IntoIterator<Item = Item>,
		) -> ArcRunExplicit<'a, RMinusInput, CNilBrand, A>
		where
			A: Clone + Send + Sync,
			Item: Send + Sync + 'static + 'a,
			RMinusInput: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
			NodeBrand<R, CNilBrand>: SendFunctor,
			NodeBrand<RMinusInput, CNilBrand>: SendFunctor,
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
			Apply!(<NodeBrand<RMinusInput, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<RMinusInput, CNilBrand>, A>,
			>): Clone + Send + Sync,
			Apply!(<RMinusInput as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<RMinusInput, CNilBrand>, A>,
			>): Send + Sync,
			Apply!(<RMinusInput as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, RMinusInput, CNilBrand, A>,
			>): Send + Sync,
			Apply!(<CNilBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<RMinusInput, CNilBrand>, A>,
			>): Send + Sync,
			Apply!(<CNilBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, RMinusInput, CNilBrand, A>,
			>): Send + Sync,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, R, CNilBrand, A>,
			>): Member<
				ArcCoyoneda<
					'a,
					SendInputBrand<ArcBrand, Option<Item>>,
					ArcRunExplicit<'a, R, CNilBrand, A>,
				>,
				Idx,
				Remainder = Apply!(
					<RMinusInput as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
						'a,
						ArcRunExplicit<'a, R, CNilBrand, A>,
					>
				),
			>, {
			let input = std::sync::Arc::new(std::sync::Mutex::new(
				input.into_iter().collect::<std::collections::VecDeque<_>>(),
			));
			let handler_input = std::sync::Arc::clone(&input);
			self.handle_with::<SendInputBrand<ArcBrand, Option<Item>>, Idx, RMinusInput>(
				move |op: SendInput<
					'a,
					ArcBrand,
					Option<Item>,
					ArcRunExplicit<'a, RMinusInput, CNilBrand, A>,
				>| {
					match op {
						SendInput::Input(k) => {
							let next_item = {
								let mut guard = match handler_input.lock() {
									Ok(guard) => guard,
									Err(poisoned) => poisoned.into_inner(),
								};
								guard.pop_front()
							};
							(*k)(next_item)
						}
					}
				},
			)
		}
	}
}
