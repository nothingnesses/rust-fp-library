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

pub(super) fn state_wrapper_impl_items_from_descriptor(
	wrapper: WrapperName,
	method: RunWrapperMethod,
) -> Option<syn::Result<Vec<ImplItem>>> {
	generator_descriptors::method_spec(EffectName::State, method)?;
	generator_descriptors::wrapper_spec(wrapper)?;

	let tokens = match (wrapper, method) {
		(WrapperName::Run, RunWrapperMethod::Get) => run_state_get_tokens(),
		(WrapperName::Run, RunWrapperMethod::Put) => run_state_put_tokens(),
		(WrapperName::Run, RunWrapperMethod::Modify) => run_state_modify_tokens(),
		(WrapperName::Run, RunWrapperMethod::RunState) => run_state_run_state_tokens(),
		(WrapperName::RcRun, RunWrapperMethod::Get) => rcrun_state_get_tokens(),
		(WrapperName::RcRun, RunWrapperMethod::Put) => rcrun_state_put_tokens(),
		(WrapperName::RcRun, RunWrapperMethod::Modify) => rcrun_state_modify_tokens(),
		(WrapperName::RcRun, RunWrapperMethod::RunState) => rcrun_state_run_state_tokens(),
		(WrapperName::ArcRun, RunWrapperMethod::Get) => arcrun_state_get_tokens(),
		(WrapperName::ArcRun, RunWrapperMethod::Put) => arcrun_state_put_tokens(),
		(WrapperName::ArcRun, RunWrapperMethod::Modify) => arcrun_state_modify_tokens(),
		(WrapperName::ArcRun, RunWrapperMethod::RunState) => arcrun_state_run_state_tokens(),
		(WrapperName::RunExplicit, RunWrapperMethod::Get) => run_explicit_state_get_tokens(),
		(WrapperName::RunExplicit, RunWrapperMethod::Put) => run_explicit_state_put_tokens(),
		(WrapperName::RunExplicit, RunWrapperMethod::Modify) => run_explicit_state_modify_tokens(),
		(WrapperName::RunExplicit, RunWrapperMethod::RunState) =>
			run_explicit_state_run_state_tokens(),
		(WrapperName::RcRunExplicit, RunWrapperMethod::Get) => rcrun_explicit_state_get_tokens(),
		(WrapperName::RcRunExplicit, RunWrapperMethod::Put) => rcrun_explicit_state_put_tokens(),
		(WrapperName::RcRunExplicit, RunWrapperMethod::Modify) =>
			rcrun_explicit_state_modify_tokens(),
		(WrapperName::RcRunExplicit, RunWrapperMethod::RunState) =>
			rcrun_explicit_state_run_state_tokens(),
		(WrapperName::ArcRunExplicit, RunWrapperMethod::Get) => arcrun_explicit_state_get_tokens(),
		(WrapperName::ArcRunExplicit, RunWrapperMethod::Put) => arcrun_explicit_state_put_tokens(),
		(WrapperName::ArcRunExplicit, RunWrapperMethod::Modify) =>
			arcrun_explicit_state_modify_tokens(),
		(WrapperName::ArcRunExplicit, RunWrapperMethod::RunState) =>
			arcrun_explicit_state_run_state_tokens(),
		_ => return None,
	};

	Some(impl_items_from_tokens(tokens))
}

fn run_state_get_tokens() -> TokenStream {
	quote! {
		/// Lifts a `Get` state effect into the Run program. Direct
		/// analog of PureScript Run's
		/// [`get`](https://github.com/natefaubion/purescript-run/blob/main/src/Run/State.purs).
		/// The program reads the current state and returns it as the
		/// result type `A` (the state type and the result type
		/// coincide for `get`).
		///
		/// `Idx` is the type-level position witness identifying where
		/// `BoxStateBrand<BoxBrand, A>` lives in the row `R`. Rust
		/// infers `Idx` whenever the effect appears unambiguously in
		/// the row.
		#[__document_module_generated]
		#[document_signature]
		///
		#[document_type_parameters("The type-level Member-position witness (typically inferred).")]
		///
		#[document_returns("A `Run` program suspended at the lifted `Get` effect.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::run::Run,
		/// };
		///
		/// type FirstRow = CoproductBrand<CoyonedaBrand<BoxStateBrand<BoxBrand, i32>>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let prog: Run<FirstRow, Scoped, i32> = Run::get();
		/// let handled: Run<CNilBrand, CNilBrand, (i32, i32)> = prog.run_state::<i32, _, CNilBrand>(41);
		/// assert_eq!(handled.extract(), (41, 41));
		/// ```
		#[inline]
		pub fn get<Idx>() -> Self
		where
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, A>):
				crate::types::effects::member::Member<
						crate::types::Coyoneda<
							'static,
							crate::brands::BoxStateBrand<crate::brands::BoxBrand, A>,
							A,
						>,
						Idx,
					>, {
			let effect: crate::types::effects::state::BoxState<'static, crate::brands::BoxBrand, A, A> =
				crate::types::effects::state::BoxState::Get(
					<crate::brands::BoxBrand as crate::classes::ToDynFnOnce>::new(|s: A| s),
				);
			Self::lift::<crate::brands::BoxStateBrand<crate::brands::BoxBrand, A>, Idx>(effect)
		}
	}
}

fn run_state_put_tokens() -> TokenStream {
	quote! {
		/// Lifts a `Put` state effect into the Run program. Direct
		/// analog of PureScript Run's
		/// [`put`](https://github.com/natefaubion/purescript-run/blob/main/src/Run/State.purs).
		/// The program writes the supplied state value `s` and
		/// returns `()` as the result type.
		///
		/// `StateType` is the state type carried by `BoxStateBrand`
		/// in the row. Rust may need a turbofish on `StateType`
		/// because `put`'s result type is `()` (which doesn't
		/// constrain the state type from the call site).
		#[__document_module_generated]
		#[document_signature]
		///
		#[document_type_parameters(
			"The state type carried by `BoxStateBrand` in the row.",
			"The type-level Member-position witness (typically inferred)."
		)]
		///
		#[document_parameters("The new state value to write.")]
		///
		#[document_returns("A `Run` program suspended at the lifted `Put` effect.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::run::Run,
		/// };
		///
		/// type FirstRow = CoproductBrand<CoyonedaBrand<BoxStateBrand<BoxBrand, i32>>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let prog: Run<FirstRow, Scoped, ()> = Run::put::<i32, _>(42);
		/// let handled: Run<CNilBrand, CNilBrand, ((), i32)> = prog.run_state::<i32, _, CNilBrand>(0);
		/// assert_eq!(handled.extract(), ((), 42));
		/// ```
		#[inline]
		pub fn put<StateType: 'static, Idx>(s: StateType) -> Self
		where
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, ()>):
				crate::types::effects::member::Member<
						crate::types::Coyoneda<
							'static,
							crate::brands::BoxStateBrand<crate::brands::BoxBrand, StateType>,
							(),
						>,
						Idx,
					>, {
			let effect: crate::types::effects::state::BoxState<
				'static,
				crate::brands::BoxBrand,
				StateType,
				(),
			> = crate::types::effects::state::BoxState::Put(
				s,
				<crate::brands::BoxBrand as crate::classes::ToDynFnOnce>::new(|_: ()| ()),
			);
			Self::lift::<crate::brands::BoxStateBrand<crate::brands::BoxBrand, StateType>, Idx>(effect)
		}
	}
}

fn run_state_modify_tokens() -> TokenStream {
	quote! {
		/// Updates the State value with a function.
		///
		/// `modify(f)` is equivalent to `get().bind(|s| put(f(s)))`.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters(
			"The State value type.",
			"The type-level Member-position witness for the State effect."
		)]
		#[document_parameters("The state update function.")]
		#[document_returns("A `Run` program that writes the updated state and returns unit.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::run::Run,
		/// };
		///
		/// type Row = CoproductBrand<CoyonedaBrand<BoxStateBrand<BoxBrand, i32>>, CNilBrand>;
		///
		/// let program: Run<Row, CNilBrand, ()> = Run::modify::<i32, _>(|state| state + 1);
		/// let handled: Run<CNilBrand, CNilBrand, ((), i32)> = program.run_state::<i32, _, CNilBrand>(41);
		/// assert_eq!(handled.extract(), ((), 42));
		/// ```
		#[inline]
		pub fn modify<StateType, Idx>(f: impl FnOnce(StateType) -> StateType + 'static) -> Self
		where
			StateType: 'static,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, StateType>):
				Member<Coyoneda<'static, BoxStateBrand<BoxBrand, StateType>, StateType>, Idx>,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, ()>):
				Member<Coyoneda<'static, BoxStateBrand<BoxBrand, StateType>, ()>, Idx>, {
			Run::<R, ScopedRow, StateType>::get::<Idx>()
				.bind(move |state| Run::<R, ScopedRow, ()>::put::<StateType, Idx>(f(state)))
		}
	}
}

fn run_state_run_state_tokens() -> TokenStream {
	quote! {
		/// Interprets one State effect by threading an owned state value.
		///
		/// The returned program produces both the original result and the
		/// final state.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters(
			"The State value type.",
			"The type-level Member-position witness for the State effect.",
			"The first-order row brand with the State effect removed."
		)]
		#[document_parameters("The initial state value.")]
		#[document_returns("A first-order-only `Run` program returning `(result, final_state)`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::run::Run,
		/// };
		///
		/// type Row = CoproductBrand<CoyonedaBrand<BoxStateBrand<BoxBrand, i32>>, CNilBrand>;
		///
		/// let program: Run<Row, CNilBrand, i32> = Run::<Row, CNilBrand, i32>::get()
		/// 	.bind(|state| Run::<Row, CNilBrand, ()>::put::<i32, _>(state + 1))
		/// 	.bind(|()| Run::<Row, CNilBrand, i32>::get());
		/// let handled: Run<CNilBrand, CNilBrand, (i32, i32)> = program.run_state::<i32, _, CNilBrand>(41);
		/// assert_eq!(handled.extract(), (42, 42));
		/// ```
		#[inline]
		pub fn run_state<StateType, Idx, RMinusState>(
			self,
			initial: StateType,
		) -> Run<RMinusState, CNilBrand, (A, StateType)>
		where
			StateType: Clone + 'static,
			RMinusState: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				Run<R, CNilBrand, A>,
			>): Member<
					Coyoneda<'static, BoxStateBrand<BoxBrand, StateType>, Run<R, CNilBrand, A>>,
					Idx,
					Remainder = Apply!(
									<RMinusState as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
										'static,
										Run<R, CNilBrand, A>,
									>
								),
				>, {
			let state = StdRc::new(RefCell::new(initial));
			let handler_state = StdRc::clone(&state);
			let handled = self.handle_with::<BoxStateBrand<BoxBrand, StateType>, Idx, RMinusState>(
				move |op: BoxState<'static, BoxBrand, StateType, Run<RMinusState, CNilBrand, A>>| match op {
					BoxState::Get(k) => {
						let current = handler_state.borrow().clone();
						k(current)
					}
					BoxState::Put(new_state, k) => {
						{
							*handler_state.borrow_mut() = new_state;
						}
						k(())
					}
				},
			);
			handled.map(move |result| (result, state.borrow().clone()))
		}
	}
}

fn rcrun_state_get_tokens() -> TokenStream {
	quote! {
		/// Lifts a `Get` state effect into the `RcRun` program.
		/// Mirrors [`Run::get`](crate::types::effects::run::Run::get);
		/// see that method for cross-wrapper semantics. Differences for
		/// `RcRun`: the [`RcCoyoneda`] variant pairs with the `Rc`-shared
		/// substrate (single `Get` continuation cloning is via the
		/// `Rc<dyn Fn>` refcount bump). Threads
		/// [`RcBrand`](crate::brands::RcBrand) as the pointer kind.
		///
		/// `Idx` is the type-level position witness identifying where
		/// `StateBrand<RcBrand, A>` lives in the row `R`. Rust infers
		/// `Idx` whenever the effect appears unambiguously in the row.
		#[__document_module_generated]
		#[document_signature]
		///
		#[document_type_parameters("The type-level Member-position witness (typically inferred).")]
		///
		#[document_returns("An `RcRun` program suspended at the lifted `Get` effect.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::rc_run::RcRun,
		/// };
		///
		/// type FirstRow = CoproductBrand<RcCoyonedaBrand<StateBrand<RcBrand, i32>>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let prog: RcRun<FirstRow, Scoped, i32> = RcRun::get();
		/// let handled: RcRun<CNilBrand, CNilBrand, (i32, i32)> = prog.run_state::<i32, _, CNilBrand>(41);
		/// assert_eq!(handled.extract(), (41, 41));
		/// ```
		#[inline]
		pub fn get<Idx>() -> Self
		where
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, A>):
				Member<RcCoyoneda<'static, crate::brands::StateBrand<RcBrand, A>, A>, Idx>,
			Apply!(<NodeBrand<R, ScopedRow> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<NodeBrand<R, ScopedRow>, crate::types::rc_free::RcTypeErasedValue>,
			>): Clone, {
			let effect: crate::types::effects::state::State<'static, RcBrand, A, A> =
				crate::types::effects::state::State::Get(<RcBrand as crate::classes::ToDynCloneFn>::new(
					|s: A| s,
				));
			Self::lift::<crate::brands::StateBrand<RcBrand, A>, Idx>(effect)
		}
	}
}

fn rcrun_state_put_tokens() -> TokenStream {
	quote! {
		/// Lifts a `Put` state effect into the `RcRun` program.
		/// Mirrors [`Run::put`](crate::types::effects::run::Run::put);
		/// see that method for cross-wrapper semantics. Threads
		/// [`RcBrand`](crate::brands::RcBrand) as the pointer kind.
		///
		/// `StateType` is the state type carried by `StateBrand` in
		/// the row. Rust may need a turbofish on `StateType` because
		/// `put`'s result type is `()` (which doesn't constrain the
		/// state type from the call site).
		#[__document_module_generated]
		#[document_signature]
		///
		#[document_type_parameters(
			"The state type carried by `StateBrand` in the row.",
			"The type-level Member-position witness (typically inferred)."
		)]
		///
		#[document_parameters("The new state value to write.")]
		///
		#[document_returns("An `RcRun` program suspended at the lifted `Put` effect.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::rc_run::RcRun,
		/// };
		///
		/// type FirstRow = CoproductBrand<RcCoyonedaBrand<StateBrand<RcBrand, i32>>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let prog: RcRun<FirstRow, Scoped, ()> = RcRun::put::<i32, _>(42);
		/// let handled: RcRun<CNilBrand, CNilBrand, ((), i32)> = prog.run_state::<i32, _, CNilBrand>(0);
		/// assert_eq!(handled.extract(), ((), 42));
		/// ```
		#[inline]
		pub fn put<StateType: Clone + 'static, Idx>(s: StateType) -> Self
		where
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, ()>):
				Member<RcCoyoneda<'static, crate::brands::StateBrand<RcBrand, StateType>, ()>, Idx>,
			Apply!(<NodeBrand<R, ScopedRow> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<NodeBrand<R, ScopedRow>, crate::types::rc_free::RcTypeErasedValue>,
			>): Clone, {
			let effect: crate::types::effects::state::State<'static, RcBrand, StateType, ()> =
				crate::types::effects::state::State::Put(
					s,
					<RcBrand as crate::classes::ToDynCloneFn>::new(|_: ()| ()),
				);
			Self::lift::<crate::brands::StateBrand<RcBrand, StateType>, Idx>(effect)
		}
	}
}

fn rcrun_state_modify_tokens() -> TokenStream {
	quote! {
		/// Updates the State value with a function.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters(
			"The State value type.",
			"The type-level Member-position witness for the State effect."
		)]
		#[document_parameters("The state update function.")]
		#[document_returns("An `RcRun` program that writes the updated state and returns unit.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::rc_run::RcRun,
		/// };
		///
		/// type Row = CoproductBrand<RcCoyonedaBrand<StateBrand<RcBrand, i32>>, CNilBrand>;
		///
		/// let program: RcRun<Row, CNilBrand, ()> = RcRun::modify::<i32, _>(|state| state + 1);
		/// let handled: RcRun<CNilBrand, CNilBrand, ((), i32)> =
		/// 	program.run_state::<i32, _, CNilBrand>(41);
		/// assert_eq!(handled.extract(), ((), 42));
		/// ```
		#[inline]
		pub fn modify<StateType, Idx>(f: impl Fn(StateType) -> StateType + 'static) -> Self
		where
			StateType: Clone + 'static,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, StateType>):
				Member<RcCoyoneda<'static, StateBrand<RcBrand, StateType>, StateType>, Idx>,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, ()>):
				Member<RcCoyoneda<'static, StateBrand<RcBrand, StateType>, ()>, Idx>,
			Apply!(<NodeBrand<R, ScopedRow> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<NodeBrand<R, ScopedRow>, RcTypeErasedValue>,
			>): Clone, {
			RcRun::<R, ScopedRow, StateType>::get::<Idx>()
				.bind(move |state| RcRun::<R, ScopedRow, ()>::put::<StateType, Idx>(f(state)))
		}
	}
}

fn rcrun_state_run_state_tokens() -> TokenStream {
	quote! {
		/// Interprets one State effect by threading an owned state value.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters(
			"The State value type.",
			"The type-level Member-position witness for the State effect.",
			"The first-order row brand with the State effect removed."
		)]
		#[document_parameters("The initial state value.")]
		#[document_returns("A first-order-only `RcRun` program returning `(result, final_state)`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::rc_run::RcRun,
		/// };
		///
		/// type Row = CoproductBrand<RcCoyonedaBrand<StateBrand<RcBrand, i32>>, CNilBrand>;
		///
		/// let program: RcRun<Row, CNilBrand, i32> = RcRun::<Row, CNilBrand, i32>::get()
		/// 	.bind(|state| RcRun::<Row, CNilBrand, ()>::put::<i32, _>(state + 1))
		/// 	.bind(|()| RcRun::<Row, CNilBrand, i32>::get());
		/// let handled: RcRun<CNilBrand, CNilBrand, (i32, i32)> =
		/// 	program.run_state::<i32, _, CNilBrand>(41);
		/// assert_eq!(handled.extract(), (42, 42));
		/// ```
		#[inline]
		pub fn run_state<StateType, Idx, RMinusState>(
			self,
			initial: StateType,
		) -> RcRun<RMinusState, CNilBrand, (A, StateType)>
		where
			A: Clone,
			StateType: Clone + 'static,
			RMinusState: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
			Apply!(<NodeBrand<R, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<NodeBrand<R, CNilBrand>, RcTypeErasedValue>,
			>): Clone,
			Apply!(<NodeBrand<RMinusState, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<NodeBrand<RMinusState, CNilBrand>, RcTypeErasedValue>,
			>): Clone,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcRun<R, CNilBrand, A>,
			>): Member<
					RcCoyoneda<'static, StateBrand<RcBrand, StateType>, RcRun<R, CNilBrand, A>>,
					Idx,
					Remainder = Apply!(
									<RMinusState as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
										'static,
										RcRun<R, CNilBrand, A>,
									>
								),
				>, {
			let state = StdRc::new(RefCell::new(initial));
			let handler_state = StdRc::clone(&state);
			let handled = self.handle_with::<StateBrand<RcBrand, StateType>, Idx, RMinusState>(
				move |op: State<'static, RcBrand, StateType, RcRun<RMinusState, CNilBrand, A>>| match op {
					State::Get(k) => {
						let current = handler_state.borrow().clone();
						(*k)(current)
					}
					State::Put(new_state, k) => {
						{
							*handler_state.borrow_mut() = new_state;
						}
						(*k)(())
					}
				},
			);
			handled.map(move |result| (result, state.borrow().clone()))
		}
	}
}

fn arcrun_state_get_tokens() -> TokenStream {
	quote! {
		/// Lifts a `Get` state effect into the `ArcRun` program.
		/// Mirrors [`Run::get`](crate::types::effects::run::Run::get);
		/// see that method for cross-wrapper semantics. Differences for
		/// `ArcRun`: threads
		/// [`ArcBrand`](crate::brands::ArcBrand) as the pointer kind
		/// and uses
		/// [`SendStateBrand`](crate::brands::SendStateBrand) (rather
		/// than `StateBrand`) so the continuation projection
		/// `<ArcBrand as SendRefCountedPointer>::Of<'_, dyn Fn(...) + Send + Sync>`
		/// is structurally `Send + Sync`, which the `SendFunctor`
		/// algebra requires across thread boundaries.
		#[__document_module_generated]
		#[document_signature]
		///
		#[document_type_parameters("The type-level Member-position witness (typically inferred).")]
		///
		#[document_returns("An `ArcRun` program suspended at the lifted `Get` effect.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::arc_run::ArcRun,
		/// };
		///
		/// type FirstRow = CoproductBrand<ArcCoyonedaBrand<SendStateBrand<ArcBrand, i32>>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let prog: ArcRun<FirstRow, Scoped, i32> = ArcRun::get();
		/// let handled: ArcRun<CNilBrand, CNilBrand, (i32, i32)> = prog.run_state::<i32, _, CNilBrand>(41);
		/// assert_eq!(handled.extract(), (41, 41));
		/// ```
		#[inline]
		pub fn get<Idx>() -> Self
		where
			A: Clone + Send + Sync,
			NodeBrand<R, ScopedRow>: SendFunctor,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, A>): Member<
					ArcCoyoneda<'static, crate::brands::SendStateBrand<crate::brands::ArcBrand, A>, A>,
					Idx,
				>,
			Apply!(<NodeBrand<R, ScopedRow> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<NodeBrand<R, ScopedRow>, ArcTypeErasedValue>,
			>): Clone, {
			let effect: crate::types::effects::state::SendState<'static, crate::brands::ArcBrand, A, A> =
				crate::types::effects::state::SendState::Get(
					<crate::brands::ArcBrand as crate::classes::ToDynSendFn>::new(|s: A| s),
				);
			Self::lift::<crate::brands::SendStateBrand<crate::brands::ArcBrand, A>, Idx>(effect)
		}
	}
}

fn arcrun_state_put_tokens() -> TokenStream {
	quote! {
		/// Lifts a `Put` state effect into the `ArcRun` program.
		/// Mirrors [`Run::put`](crate::types::effects::run::Run::put);
		/// see that method for cross-wrapper semantics. Threads
		/// [`ArcBrand`](crate::brands::ArcBrand) as the pointer kind
		/// and uses
		/// [`SendStateBrand`](crate::brands::SendStateBrand); see
		/// [`get`](ArcRun::get) for the design rationale.
		#[__document_module_generated]
		#[document_signature]
		///
		#[document_type_parameters(
			"The state type carried by `SendStateBrand` in the row.",
			"The type-level Member-position witness (typically inferred)."
		)]
		///
		#[document_parameters("The new state value to write.")]
		///
		#[document_returns("An `ArcRun` program suspended at the lifted `Put` effect.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::arc_run::ArcRun,
		/// };
		///
		/// type FirstRow = CoproductBrand<ArcCoyonedaBrand<SendStateBrand<ArcBrand, i32>>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let prog: ArcRun<FirstRow, Scoped, ()> = ArcRun::put::<i32, _>(42);
		/// let handled: ArcRun<CNilBrand, CNilBrand, ((), i32)> = prog.run_state::<i32, _, CNilBrand>(0);
		/// assert_eq!(handled.extract(), ((), 42));
		/// ```
		#[inline]
		pub fn put<StateType: Clone + Send + Sync + 'static, Idx>(s: StateType) -> Self
		where
			NodeBrand<R, ScopedRow>: SendFunctor,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, ()>): Member<
					ArcCoyoneda<
						'static,
						crate::brands::SendStateBrand<crate::brands::ArcBrand, StateType>,
						(),
					>,
					Idx,
				>,
			Apply!(<NodeBrand<R, ScopedRow> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<NodeBrand<R, ScopedRow>, ArcTypeErasedValue>,
			>): Clone, {
			let effect: crate::types::effects::state::SendState<
				'static,
				crate::brands::ArcBrand,
				StateType,
				(),
			> = crate::types::effects::state::SendState::Put(
				s,
				<crate::brands::ArcBrand as crate::classes::ToDynSendFn>::new(|_: ()| ()),
			);
			Self::lift::<crate::brands::SendStateBrand<crate::brands::ArcBrand, StateType>, Idx>(effect)
		}
	}
}

fn arcrun_state_modify_tokens() -> TokenStream {
	quote! {
		/// Updates the State value with a function.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters(
			"The State value type.",
			"The type-level Member-position witness for the State effect."
		)]
		#[document_parameters("The state update function.")]
		#[document_returns("An `ArcRun` program that writes the updated state and returns unit.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::arc_run::ArcRun,
		/// };
		///
		/// type Row = CoproductBrand<ArcCoyonedaBrand<SendStateBrand<ArcBrand, i32>>, CNilBrand>;
		///
		/// let program: ArcRun<Row, CNilBrand, ()> = ArcRun::modify::<i32, _>(|state| state + 1);
		/// let handled: ArcRun<CNilBrand, CNilBrand, ((), i32)> =
		/// 	program.run_state::<i32, _, CNilBrand>(41);
		/// assert_eq!(handled.extract(), ((), 42));
		/// ```
		#[inline]
		pub fn modify<StateType, Idx>(f: impl Fn(StateType) -> StateType + Send + Sync + 'static) -> Self
		where
			StateType: Clone + Send + Sync + 'static,
			NodeBrand<R, ScopedRow>: SendFunctor,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, StateType>):
				Member<ArcCoyoneda<'static, SendStateBrand<ArcBrand, StateType>, StateType>, Idx>,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, ()>):
				Member<ArcCoyoneda<'static, SendStateBrand<ArcBrand, StateType>, ()>, Idx>,
			Apply!(<NodeBrand<R, ScopedRow> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<NodeBrand<R, ScopedRow>, ArcTypeErasedValue>,
			>): Clone, {
			ArcRun::<R, ScopedRow, StateType>::get::<Idx>()
				.bind(move |state| ArcRun::<R, ScopedRow, ()>::put::<StateType, Idx>(f(state)))
		}
	}
}

fn arcrun_state_run_state_tokens() -> TokenStream {
	quote! {
		/// Interprets one State effect by threading an owned state value.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters(
			"The State value type.",
			"The type-level Member-position witness for the State effect.",
			"The first-order row brand with the State effect removed."
		)]
		#[document_parameters("The initial state value.")]
		#[document_returns("A first-order-only `ArcRun` program returning `(result, final_state)`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::arc_run::ArcRun,
		/// };
		///
		/// type Row = CoproductBrand<ArcCoyonedaBrand<SendStateBrand<ArcBrand, i32>>, CNilBrand>;
		///
		/// let program: ArcRun<Row, CNilBrand, i32> = ArcRun::<Row, CNilBrand, i32>::get()
		/// 	.bind(|state| ArcRun::<Row, CNilBrand, ()>::put::<i32, _>(state + 1))
		/// 	.bind(|()| ArcRun::<Row, CNilBrand, i32>::get());
		/// let handled: ArcRun<CNilBrand, CNilBrand, (i32, i32)> =
		/// 	program.run_state::<i32, _, CNilBrand>(41);
		/// assert_eq!(handled.extract(), (42, 42));
		/// ```
		#[inline]
		pub fn run_state<StateType, Idx, RMinusState>(
			self,
			initial: StateType,
		) -> ArcRun<RMinusState, CNilBrand, (A, StateType)>
		where
			A: Clone + Send + Sync,
			StateType: Clone + Send + Sync + 'static,
			R: Kind_cdc7cd43dac7585f + 'static,
			RMinusState: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
			NodeBrand<R, CNilBrand>: SendFunctor,
			NodeBrand<RMinusState, CNilBrand>: WrapDrop
				+ Kind_cdc7cd43dac7585f<
					Of<'static, ArcFree<NodeBrand<RMinusState, CNilBrand>, ArcTypeErasedValue>>: Send
																									 + Sync,
				> + SendFunctor,
			Apply!(<NodeBrand<R, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<NodeBrand<R, CNilBrand>, ArcTypeErasedValue>,
			>): Clone,
			Apply!(<NodeBrand<RMinusState, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<NodeBrand<RMinusState, CNilBrand>, ArcTypeErasedValue>,
			>): Clone,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcRun<R, CNilBrand, A>,
			>): Member<
					ArcCoyoneda<'static, SendStateBrand<ArcBrand, StateType>, ArcRun<R, CNilBrand, A>>,
					Idx,
					Remainder = Apply!(
									<RMinusState as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
										'static,
										ArcRun<R, CNilBrand, A>,
									>
								),
				>, {
			let state = StdArc::new(Mutex::new(initial));
			let handler_state = StdArc::clone(&state);
			let handled = self.handle_with::<SendStateBrand<ArcBrand, StateType>, Idx, RMinusState>(
				move |op: SendState<'static, ArcBrand, StateType, ArcRun<RMinusState, CNilBrand, A>>| {
					match op {
						SendState::Get(k) => {
							let current = {
								let guard = match handler_state.lock() {
									Ok(guard) => guard,
									Err(poisoned) => poisoned.into_inner(),
								};
								guard.clone()
							};
							(*k)(current)
						}
						SendState::Put(new_state, k) => {
							{
								let mut guard = match handler_state.lock() {
									Ok(guard) => guard,
									Err(poisoned) => poisoned.into_inner(),
								};
								*guard = new_state;
							}
							(*k)(())
						}
					}
				},
			);
			handled.map(move |result| {
				let guard = match state.lock() {
					Ok(guard) => guard,
					Err(poisoned) => poisoned.into_inner(),
				};
				(result, guard.clone())
			})
		}
	}
}

fn run_explicit_state_get_tokens() -> TokenStream {
	quote! {
		/// Lifts a `Get` state effect into the `RunExplicit` program.
		/// Mirrors [`Run::get`](crate::types::effects::run::Run::get);
		/// see that method for cross-wrapper semantics. Differences for
		/// `RunExplicit`: the bare [`Coyoneda`] variant pairs with the
		/// Box-in-Wrap Explicit substrate (the substrate's `peel` does
		/// not require a `Clone` bound on the inner effect). Threads
		/// [`BoxBrand`](crate::brands::BoxBrand) as the pointer kind
		/// post-Phase-3.5 retrofit so the continuation projection is
		/// `Box<dyn FnOnce>` rather than `Rc<dyn Fn>`.
		#[__document_module_generated]
		#[document_signature]
		///
		#[document_type_parameters("The type-level Member-position witness (typically inferred).")]
		///
		#[document_returns("A `RunExplicit` program suspended at the lifted `Get` effect.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::run_explicit::RunExplicit,
		/// };
		///
		/// type FirstRow = CoproductBrand<CoyonedaBrand<BoxStateBrand<BoxBrand, i32>>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let prog: RunExplicit<'static, FirstRow, Scoped, i32> = RunExplicit::get();
		/// let handled: RunExplicit<'static, CNilBrand, CNilBrand, (i32, i32)> =
		/// 	prog.run_state::<i32, _, CNilBrand>(41);
		/// assert_eq!(handled.extract(), (41, 41));
		/// ```
		#[inline]
		pub fn get<Idx>() -> Self
		where
			A: 'static,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>):
				Member<Coyoneda<'a, crate::brands::BoxStateBrand<crate::brands::BoxBrand, A>, A>, Idx>, {
			let effect: crate::types::effects::state::BoxState<'a, crate::brands::BoxBrand, A, A> =
				crate::types::effects::state::BoxState::Get(
					<crate::brands::BoxBrand as crate::classes::ToDynFnOnce>::new(|s: A| s),
				);
			Self::lift::<crate::brands::BoxStateBrand<crate::brands::BoxBrand, A>, Idx>(effect)
		}
	}
}

fn run_explicit_state_put_tokens() -> TokenStream {
	quote! {
		/// Lifts a `Put` state effect into the `RunExplicit` program.
		/// Mirrors [`Run::put`](crate::types::effects::run::Run::put);
		/// see that method for cross-wrapper semantics. Threads
		/// [`BoxBrand`](crate::brands::BoxBrand) as the pointer kind
		/// post-Phase-3.5 retrofit so the continuation projection is
		/// `Box<dyn FnOnce>`.
		#[__document_module_generated]
		#[document_signature]
		///
		#[document_type_parameters(
			"The state type carried by `BoxStateBrand` in the row.",
			"The type-level Member-position witness (typically inferred)."
		)]
		///
		#[document_parameters("The new state value to write.")]
		///
		#[document_returns("A `RunExplicit` program suspended at the lifted `Put` effect.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::run_explicit::RunExplicit,
		/// };
		///
		/// type FirstRow = CoproductBrand<CoyonedaBrand<BoxStateBrand<BoxBrand, i32>>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let prog: RunExplicit<'static, FirstRow, Scoped, ()> = RunExplicit::put::<i32, _>(42);
		/// let handled: RunExplicit<'static, CNilBrand, CNilBrand, ((), i32)> =
		/// 	prog.run_state::<i32, _, CNilBrand>(0);
		/// assert_eq!(handled.extract(), ((), 42));
		/// ```
		#[inline]
		pub fn put<StateType: 'static, Idx>(s: StateType) -> Self
		where
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, ()>): Member<
					Coyoneda<'a, crate::brands::BoxStateBrand<crate::brands::BoxBrand, StateType>, ()>,
					Idx,
				>, {
			let effect: crate::types::effects::state::BoxState<'a, crate::brands::BoxBrand, StateType, ()> =
				crate::types::effects::state::BoxState::Put(
					s,
					<crate::brands::BoxBrand as crate::classes::ToDynFnOnce>::new(|_: ()| ()),
				);
			Self::lift::<crate::brands::BoxStateBrand<crate::brands::BoxBrand, StateType>, Idx>(effect)
		}
	}
}

fn run_explicit_state_modify_tokens() -> TokenStream {
	quote! {
		/// Updates the State value with a function.
		///
		/// `modify(f)` is equivalent to `get().bind(|s| put(f(s)))`.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters(
			"The State value type.",
			"The type-level Member-position witness for the State effect."
		)]
		#[document_parameters("The state update function.")]
		#[document_returns("A `RunExplicit` program that writes the updated state and returns unit.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::run_explicit::RunExplicit,
		/// };
		///
		/// type Row = CoproductBrand<CoyonedaBrand<BoxStateBrand<BoxBrand, i32>>, CNilBrand>;
		///
		/// let program: RunExplicit<'static, Row, CNilBrand, ()> =
		/// 	RunExplicit::modify::<i32, _>(|state| state + 1);
		/// let handled: RunExplicit<'static, CNilBrand, CNilBrand, ((), i32)> =
		/// 	program.run_state::<i32, _, CNilBrand>(41);
		/// assert_eq!(handled.extract(), ((), 42));
		/// ```
		#[inline]
		pub fn modify<StateType, Idx>(f: impl Fn(StateType) -> StateType + 'a) -> Self
		where
			StateType: 'static,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, StateType>):
				Member<Coyoneda<'a, BoxStateBrand<BoxBrand, StateType>, StateType>, Idx>,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, ()>):
				Member<Coyoneda<'a, BoxStateBrand<BoxBrand, StateType>, ()>, Idx>, {
			RunExplicit::<'a, R, ScopedRow, StateType>::get::<Idx>()
				.bind(move |state| RunExplicit::<'a, R, ScopedRow, ()>::put::<StateType, Idx>(f(state)))
		}
	}
}

fn run_explicit_state_run_state_tokens() -> TokenStream {
	quote! {
		/// Interprets one State effect by threading an owned state value.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters(
			"The State value type.",
			"The type-level Member-position witness for the State effect.",
			"The first-order row brand with the State effect removed."
		)]
		#[document_parameters("The initial state value.")]
		#[document_returns("A first-order-only `RunExplicit` program returning `(result, final_state)`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::run_explicit::RunExplicit,
		/// };
		///
		/// type Row = CoproductBrand<CoyonedaBrand<BoxStateBrand<BoxBrand, i32>>, CNilBrand>;
		///
		/// let program: RunExplicit<'static, Row, CNilBrand, i32> =
		/// 	RunExplicit::<'static, Row, CNilBrand, i32>::get()
		/// 		.bind(|state| RunExplicit::<'static, Row, CNilBrand, ()>::put::<i32, _>(state + 1))
		/// 		.bind(|()| RunExplicit::<'static, Row, CNilBrand, i32>::get());
		/// let handled: RunExplicit<'static, CNilBrand, CNilBrand, (i32, i32)> =
		/// 	program.run_state::<i32, _, CNilBrand>(41);
		/// assert_eq!(handled.extract(), (42, 42));
		/// ```
		#[inline]
		pub fn run_state<StateType, Idx, RMinusState>(
			self,
			initial: StateType,
		) -> RunExplicit<'a, RMinusState, CNilBrand, (A, StateType)>
		where
			StateType: Clone + 'static + 'a,
			RMinusState: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RunExplicit<'a, R, CNilBrand, A>,
			>): Member<
					Coyoneda<'a, BoxStateBrand<BoxBrand, StateType>, RunExplicit<'a, R, CNilBrand, A>>,
					Idx,
					Remainder = Apply!(
									<RMinusState as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
										'a,
										RunExplicit<'a, R, CNilBrand, A>,
									>
								),
				>, {
			let state = StdRc::new(RefCell::new(initial));
			let handler_state = StdRc::clone(&state);
			let handled = self.handle_with::<BoxStateBrand<BoxBrand, StateType>, Idx, RMinusState>(
				move |op: BoxState<'a, BoxBrand, StateType, RunExplicit<'a, RMinusState, CNilBrand, A>>| {
					match op {
						BoxState::Get(k) => {
							let current = handler_state.borrow().clone();
							k(current)
						}
						BoxState::Put(new_state, k) => {
							{
								*handler_state.borrow_mut() = new_state;
							}
							k(())
						}
					}
				},
			);
			handled.map(move |result| (result, state.borrow().clone()))
		}
	}
}

fn rcrun_explicit_state_get_tokens() -> TokenStream {
	quote! {
		/// Lifts a `Get` state effect into the `RcRunExplicit` program.
		/// Mirrors [`Run::get`](crate::types::effects::run::Run::get);
		/// see that method for cross-wrapper semantics. Differences for
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
		#[document_returns("An `RcRunExplicit` program suspended at the lifted `Get` effect.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::rc_run_explicit::RcRunExplicit,
		/// };
		///
		/// type FirstRow = CoproductBrand<RcCoyonedaBrand<StateBrand<RcBrand, i32>>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let prog: RcRunExplicit<'static, FirstRow, Scoped, i32> = RcRunExplicit::get();
		/// let handled: RcRunExplicit<'static, CNilBrand, CNilBrand, (i32, i32)> =
		/// 	prog.run_state::<i32, _, CNilBrand>(41);
		/// assert_eq!(handled.extract(), (41, 41));
		/// ```
		#[inline]
		pub fn get<Idx>() -> Self
		where
			A: Clone + 'static,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>):
				Member<RcCoyoneda<'a, crate::brands::StateBrand<crate::brands::RcBrand, A>, A>, Idx>, {
			let effect: crate::types::effects::state::State<'a, crate::brands::RcBrand, A, A> =
				crate::types::effects::state::State::Get(
					<crate::brands::RcBrand as crate::classes::ToDynCloneFn>::new(|s: A| s),
				);
			Self::lift::<crate::brands::StateBrand<crate::brands::RcBrand, A>, Idx>(effect)
		}
	}
}

fn rcrun_explicit_state_put_tokens() -> TokenStream {
	quote! {
		/// Lifts a `Put` state effect into the `RcRunExplicit` program.
		/// Mirrors [`Run::put`](crate::types::effects::run::Run::put);
		/// see that method for cross-wrapper semantics. Threads
		/// [`RcBrand`](crate::brands::RcBrand) as the pointer kind.
		#[__document_module_generated]
		#[document_signature]
		///
		#[document_type_parameters(
			"The state type carried by `StateBrand` in the row.",
			"The type-level Member-position witness (typically inferred)."
		)]
		///
		#[document_parameters("The new state value to write.")]
		///
		#[document_returns("An `RcRunExplicit` program suspended at the lifted `Put` effect.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::rc_run_explicit::RcRunExplicit,
		/// };
		///
		/// type FirstRow = CoproductBrand<RcCoyonedaBrand<StateBrand<RcBrand, i32>>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let prog: RcRunExplicit<'static, FirstRow, Scoped, ()> = RcRunExplicit::put::<i32, _>(42);
		/// let handled: RcRunExplicit<'static, CNilBrand, CNilBrand, ((), i32)> =
		/// 	prog.run_state::<i32, _, CNilBrand>(0);
		/// assert_eq!(handled.extract(), ((), 42));
		/// ```
		#[inline]
		pub fn put<StateType: Clone + 'static, Idx>(s: StateType) -> Self
		where
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, ()>): Member<
					RcCoyoneda<'a, crate::brands::StateBrand<crate::brands::RcBrand, StateType>, ()>,
					Idx,
				>, {
			let effect: crate::types::effects::state::State<'a, crate::brands::RcBrand, StateType, ()> =
				crate::types::effects::state::State::Put(
					s,
					<crate::brands::RcBrand as crate::classes::ToDynCloneFn>::new(|_: ()| ()),
				);
			Self::lift::<crate::brands::StateBrand<crate::brands::RcBrand, StateType>, Idx>(effect)
		}
	}
}

fn rcrun_explicit_state_modify_tokens() -> TokenStream {
	quote! {
		/// Updates the State value with a function.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters(
			"The State value type.",
			"The type-level Member-position witness for the State effect."
		)]
		#[document_parameters("The state update function.")]
		#[document_returns("An `RcRunExplicit` program that writes the updated state and returns unit.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::rc_run_explicit::RcRunExplicit,
		/// };
		///
		/// type Row = CoproductBrand<RcCoyonedaBrand<StateBrand<RcBrand, i32>>, CNilBrand>;
		///
		/// let program: RcRunExplicit<'static, Row, CNilBrand, ()> =
		/// 	RcRunExplicit::modify::<i32, _>(|state| state + 1);
		/// let handled: RcRunExplicit<'static, CNilBrand, CNilBrand, ((), i32)> =
		/// 	program.run_state::<i32, _, CNilBrand>(41);
		/// assert_eq!(handled.extract(), ((), 42));
		/// ```
		#[inline]
		pub fn modify<StateType, Idx>(f: impl Fn(StateType) -> StateType + 'a) -> Self
		where
			StateType: Clone + 'static,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, StateType>):
				Member<RcCoyoneda<'a, StateBrand<RcBrand, StateType>, StateType>, Idx>,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, ()>):
				Member<RcCoyoneda<'a, StateBrand<RcBrand, StateType>, ()>, Idx>,
			Apply!(<NodeBrand<R, ScopedRow> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, ScopedRow>, StateType>,
			>): Clone,
			Apply!(<NodeBrand<R, ScopedRow> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, ScopedRow>, ()>,
			>): Clone, {
			RcRunExplicit::<'a, R, ScopedRow, StateType>::get::<Idx>()
				.bind(move |state| RcRunExplicit::<'a, R, ScopedRow, ()>::put::<StateType, Idx>(f(state)))
		}
	}
}

fn rcrun_explicit_state_run_state_tokens() -> TokenStream {
	quote! {
		/// Interprets one State effect by threading an owned state value.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters(
			"The State value type.",
			"The type-level Member-position witness for the State effect.",
			"The first-order row brand with the State effect removed."
		)]
		#[document_parameters("The initial state value.")]
		#[document_returns("A first-order-only `RcRunExplicit` program returning `(result, final_state)`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::rc_run_explicit::RcRunExplicit,
		/// };
		///
		/// type Row = CoproductBrand<RcCoyonedaBrand<StateBrand<RcBrand, i32>>, CNilBrand>;
		///
		/// let program: RcRunExplicit<'static, Row, CNilBrand, i32> =
		/// 	RcRunExplicit::<'static, Row, CNilBrand, i32>::get()
		/// 		.bind(|state| RcRunExplicit::<'static, Row, CNilBrand, ()>::put::<i32, _>(state + 1))
		/// 		.bind(|()| RcRunExplicit::<'static, Row, CNilBrand, i32>::get());
		/// let handled: RcRunExplicit<'static, CNilBrand, CNilBrand, (i32, i32)> =
		/// 	program.run_state::<i32, _, CNilBrand>(41);
		/// assert_eq!(handled.extract(), (42, 42));
		/// ```
		#[inline]
		pub fn run_state<StateType, Idx, RMinusState>(
			self,
			initial: StateType,
		) -> RcRunExplicit<'a, RMinusState, CNilBrand, (A, StateType)>
		where
			A: Clone,
			StateType: Clone + 'static + 'a,
			RMinusState: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
			Apply!(<NodeBrand<R, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, CNilBrand>, A>,
			>): Clone,
			Apply!(<NodeBrand<RMinusState, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<RMinusState, CNilBrand>, A>,
			>): Clone,
			Apply!(<NodeBrand<RMinusState, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<RMinusState, CNilBrand>, (A, StateType)>,
			>): Clone,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcRunExplicit<'a, R, CNilBrand, A>,
			>): Member<
					RcCoyoneda<'a, StateBrand<RcBrand, StateType>, RcRunExplicit<'a, R, CNilBrand, A>>,
					Idx,
					Remainder = Apply!(
									<RMinusState as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
										'a,
										RcRunExplicit<'a, R, CNilBrand, A>,
									>
								),
				>, {
			let state = StdRc::new(RefCell::new(initial));
			let handler_state = StdRc::clone(&state);
			let handled = self.handle_with::<StateBrand<RcBrand, StateType>, Idx, RMinusState>(
				move |op: State<'a, RcBrand, StateType, RcRunExplicit<'a, RMinusState, CNilBrand, A>>| {
					match op {
						State::Get(k) => {
							let current = handler_state.borrow().clone();
							(*k)(current)
						}
						State::Put(new_state, k) => {
							{
								*handler_state.borrow_mut() = new_state;
							}
							(*k)(())
						}
					}
				},
			);
			handled.map(move |result| (result, state.borrow().clone()))
		}
	}
}

fn arcrun_explicit_state_get_tokens() -> TokenStream {
	quote! {
		/// Lifts a `Get` state effect into the `ArcRunExplicit`
		/// program. Mirrors
		/// [`Run::get`](crate::types::effects::run::Run::get); see
		/// that method for cross-wrapper semantics. Differences for
		/// `ArcRunExplicit`: threads
		/// [`ArcBrand`](crate::brands::ArcBrand) as the pointer kind
		/// and uses
		/// [`SendStateBrand`](crate::brands::SendStateBrand) (rather
		/// than `StateBrand`) so the continuation projection is
		/// structurally `Send + Sync`, which the `SendFunctor`
		/// algebra requires across thread boundaries.
		#[__document_module_generated]
		#[document_signature]
		///
		#[document_type_parameters("The type-level Member-position witness (typically inferred).")]
		///
		#[document_returns("An `ArcRunExplicit` program suspended at the lifted `Get` effect.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::arc_run_explicit::ArcRunExplicit,
		/// };
		///
		/// type FirstRow = CoproductBrand<ArcCoyonedaBrand<SendStateBrand<ArcBrand, i32>>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let prog: ArcRunExplicit<'static, FirstRow, Scoped, i32> = ArcRunExplicit::get();
		/// let handled: ArcRunExplicit<'static, CNilBrand, CNilBrand, (i32, i32)> =
		/// 	prog.run_state::<i32, _, CNilBrand>(41);
		/// assert_eq!(handled.extract(), (41, 41));
		/// ```
		#[inline]
		pub fn get<Idx>() -> Self
		where
			A: Clone + Send + Sync + 'static,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>):
				Member<ArcCoyoneda<'a, crate::brands::SendStateBrand<crate::brands::ArcBrand, A>, A>, Idx>,
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
			let effect: crate::types::effects::state::SendState<'a, crate::brands::ArcBrand, A, A> =
				crate::types::effects::state::SendState::Get(
					<crate::brands::ArcBrand as crate::classes::ToDynSendFn>::new(|s: A| s),
				);
			Self::lift::<crate::brands::SendStateBrand<crate::brands::ArcBrand, A>, Idx>(effect)
		}
	}
}

fn arcrun_explicit_state_put_tokens() -> TokenStream {
	quote! {
		/// Lifts a `Put` state effect into the `ArcRunExplicit`
		/// program. Mirrors
		/// [`Run::put`](crate::types::effects::run::Run::put); see
		/// that method for cross-wrapper semantics. Threads
		/// [`ArcBrand`](crate::brands::ArcBrand) as the pointer kind
		/// and uses
		/// [`SendStateBrand`](crate::brands::SendStateBrand); see
		/// [`get`](ArcRunExplicit::get) for the design rationale.
		#[__document_module_generated]
		#[document_signature]
		///
		#[document_type_parameters(
			"The state type carried by `SendStateBrand` in the row.",
			"The type-level Member-position witness (typically inferred)."
		)]
		///
		#[document_parameters("The new state value to write.")]
		///
		#[document_returns("An `ArcRunExplicit` program suspended at the lifted `Put` effect.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::arc_run_explicit::ArcRunExplicit,
		/// };
		///
		/// type FirstRow = CoproductBrand<ArcCoyonedaBrand<SendStateBrand<ArcBrand, i32>>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let prog: ArcRunExplicit<'static, FirstRow, Scoped, ()> = ArcRunExplicit::put::<i32, _>(42);
		/// let handled: ArcRunExplicit<'static, CNilBrand, CNilBrand, ((), i32)> =
		/// 	prog.run_state::<i32, _, CNilBrand>(0);
		/// assert_eq!(handled.extract(), ((), 42));
		/// ```
		#[inline]
		pub fn put<StateType: Clone + Send + Sync + 'static, Idx>(s: StateType) -> Self
		where
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, ()>): Member<
					ArcCoyoneda<'a, crate::brands::SendStateBrand<crate::brands::ArcBrand, StateType>, ()>,
					Idx,
				>,
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
			let effect: crate::types::effects::state::SendState<
				'a,
				crate::brands::ArcBrand,
				StateType,
				(),
			> = crate::types::effects::state::SendState::Put(
				s,
				<crate::brands::ArcBrand as crate::classes::ToDynSendFn>::new(|_: ()| ()),
			);
			Self::lift::<crate::brands::SendStateBrand<crate::brands::ArcBrand, StateType>, Idx>(effect)
		}
	}
}

fn arcrun_explicit_state_modify_tokens() -> TokenStream {
	quote! {
		/// Updates the State value with a function.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters(
			"The State value type.",
			"The type-level Member-position witness for the State effect."
		)]
		#[document_parameters("The state update function.")]
		#[document_returns("An `ArcRunExplicit` program that writes the updated state and returns unit.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::arc_run_explicit::ArcRunExplicit,
		/// };
		///
		/// type Row = CoproductBrand<ArcCoyonedaBrand<SendStateBrand<ArcBrand, i32>>, CNilBrand>;
		///
		/// let program: ArcRunExplicit<'static, Row, CNilBrand, ()> =
		/// 	ArcRunExplicit::modify::<i32, _>(|state| state + 1);
		/// let handled: ArcRunExplicit<'static, CNilBrand, CNilBrand, ((), i32)> =
		/// 	program.run_state::<i32, _, CNilBrand>(41);
		/// assert_eq!(handled.extract(), ((), 42));
		/// ```
		#[inline]
		pub fn modify<StateType, Idx>(f: impl Fn(StateType) -> StateType + Send + Sync + 'a) -> Self
		where
			StateType: Clone + Send + Sync + 'static,
			NodeBrand<R, ScopedRow>: SendFunctor,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, StateType>):
				Member<ArcCoyoneda<'a, SendStateBrand<ArcBrand, StateType>, StateType>, Idx>,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, ()>):
				Member<ArcCoyoneda<'a, SendStateBrand<ArcBrand, StateType>, ()>, Idx>,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, StateType>,
			>): Send + Sync,
			Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, StateType>,
			>): Send + Sync,
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
				ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, StateType>,
			>): Clone + Send + Sync,
			Apply!(<NodeBrand<R, ScopedRow> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, ()>,
			>): Clone + Send + Sync, {
			ArcRunExplicit::<'a, R, ScopedRow, StateType>::get::<Idx>()
				.bind(move |state| ArcRunExplicit::<'a, R, ScopedRow, ()>::put::<StateType, Idx>(f(state)))
		}
	}
}

fn arcrun_explicit_state_run_state_tokens() -> TokenStream {
	quote! {
		/// Interprets one State effect by threading an owned state value.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters(
			"The State value type.",
			"The type-level Member-position witness for the State effect.",
			"The first-order row brand with the State effect removed."
		)]
		#[document_parameters("The initial state value.")]
		#[document_returns(
			"A first-order-only `ArcRunExplicit` program returning `(result, final_state)`."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::arc_run_explicit::ArcRunExplicit,
		/// };
		///
		/// type Row = CoproductBrand<ArcCoyonedaBrand<SendStateBrand<ArcBrand, i32>>, CNilBrand>;
		///
		/// let program: ArcRunExplicit<'static, Row, CNilBrand, i32> =
		/// 	ArcRunExplicit::<'static, Row, CNilBrand, i32>::get()
		/// 		.bind(|state| ArcRunExplicit::<'static, Row, CNilBrand, ()>::put::<i32, _>(state + 1))
		/// 		.bind(|()| ArcRunExplicit::<'static, Row, CNilBrand, i32>::get());
		/// let handled: ArcRunExplicit<'static, CNilBrand, CNilBrand, (i32, i32)> =
		/// 	program.run_state::<i32, _, CNilBrand>(41);
		/// assert_eq!(handled.extract(), (42, 42));
		/// ```
		#[inline]
		pub fn run_state<StateType, Idx, RMinusState>(
			self,
			initial: StateType,
		) -> ArcRunExplicit<'a, RMinusState, CNilBrand, (A, StateType)>
		where
			A: Clone + Send + Sync,
			StateType: Clone + Send + Sync + 'static + 'a,
			RMinusState: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
			NodeBrand<R, CNilBrand>: SendFunctor,
			NodeBrand<RMinusState, CNilBrand>: SendFunctor,
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
			Apply!(<NodeBrand<RMinusState, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<RMinusState, CNilBrand>, A>,
			>): Clone + Send + Sync,
			Apply!(<NodeBrand<RMinusState, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<RMinusState, CNilBrand>, (A, StateType)>,
			>): Clone + Send + Sync,
			Apply!(<RMinusState as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<RMinusState, CNilBrand>, A>,
			>): Send + Sync,
			Apply!(<RMinusState as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<RMinusState, CNilBrand>, (A, StateType)>,
			>): Send + Sync,
			Apply!(<RMinusState as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, RMinusState, CNilBrand, A>,
			>): Send + Sync,
			Apply!(<CNilBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<RMinusState, CNilBrand>, A>,
			>): Send + Sync,
			Apply!(<CNilBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<RMinusState, CNilBrand>, (A, StateType)>,
			>): Send + Sync,
			Apply!(<CNilBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, RMinusState, CNilBrand, A>,
			>): Send + Sync,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, R, CNilBrand, A>,
			>): Member<
					ArcCoyoneda<
						'a,
						SendStateBrand<ArcBrand, StateType>,
						ArcRunExplicit<'a, R, CNilBrand, A>,
					>,
					Idx,
					Remainder = Apply!(
									<RMinusState as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
										'a,
										ArcRunExplicit<'a, R, CNilBrand, A>,
									>
								),
				>, {
			let state = StdArc::new(Mutex::new(initial));
			let handler_state = StdArc::clone(&state);
			let handled = self.handle_with::<SendStateBrand<ArcBrand, StateType>, Idx, RMinusState>(
				move |op: SendState<
					'a,
					ArcBrand,
					StateType,
					ArcRunExplicit<'a, RMinusState, CNilBrand, A>,
				>| {
					match op {
						SendState::Get(k) => {
							let current = {
								let guard = match handler_state.lock() {
									Ok(guard) => guard,
									Err(poisoned) => poisoned.into_inner(),
								};
								guard.clone()
							};
							(*k)(current)
						}
						SendState::Put(new_state, k) => {
							{
								let mut guard = match handler_state.lock() {
									Ok(guard) => guard,
									Err(poisoned) => poisoned.into_inner(),
								};
								*guard = new_state;
							}
							(*k)(())
						}
					}
				},
			);
			handled.map(move |result| {
				let guard = match state.lock() {
					Ok(guard) => guard,
					Err(poisoned) => poisoned.into_inner(),
				};
				(result, guard.clone())
			})
		}
	}
}
