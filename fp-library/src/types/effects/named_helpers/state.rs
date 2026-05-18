//! Named State helpers layered over the Run wrapper primitives.
//!
//! The helpers remain inherent methods on the public wrapper types and use the
//! existing `get`, `put`, and `handle_with` machinery.

#[fp_macros::document_module]
pub(crate) mod inner {
	use {
		crate::{
			Apply,
			brands::{
				ArcBrand,
				BoxBrand,
				BoxStateBrand,
				CNilBrand,
				NodeBrand,
				RcBrand,
				SendStateBrand,
				StateBrand,
			},
			classes::{
				Functor,
				SendFunctor,
				WrapDrop,
			},
			kinds::*,
			types::{
				ArcCoyoneda,
				ArcFree,
				Coyoneda,
				RcCoyoneda,
				RcFree,
				RcFreeExplicit,
				arc_free::ArcTypeErasedValue,
				effects::{
					arc_run::ArcRun,
					member::Member,
					rc_run::RcRun,
					rc_run_explicit::RcRunExplicit,
					run::Run,
					run_explicit::RunExplicit,
					state::{
						BoxState,
						SendState,
						State,
					},
				},
				rc_free::RcTypeErasedValue,
			},
		},
		fp_macros::*,
		std::{
			cell::RefCell,
			rc::Rc as StdRc,
			sync::{
				Arc as StdArc,
				Mutex,
			},
		},
	};

	#[document_type_parameters(
		"The first-order effect row brand.",
		"The scoped-effect row brand.",
		"The result type."
	)]
	impl<R, ScopedRow, A> Run<R, ScopedRow, A>
	where
		R: WrapDrop + Functor + 'static,
		ScopedRow: WrapDrop + Functor + 'static,
		A: 'static,
	{
		/// Reads the State value and maps it immediately.
		///
		/// `gets(f)` is the State-specific convenience form of
		/// `get().map(f)`.
		#[document_signature]
		#[document_type_parameters(
			"The State value type.",
			"The type-level Member-position witness for the State effect."
		)]
		#[document_parameters("The projection to apply to the current state.")]
		#[document_returns("A `Run` program that reads the state and returns `f(state)`.")]
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
		/// let program: Run<Row, CNilBrand, String> =
		/// 	Run::gets::<i32, _>(|state| format!("state={state}"));
		/// let handled: Run<CNilBrand, CNilBrand, (String, i32)> =
		/// 	program.run_state::<i32, _, CNilBrand>(7);
		/// assert_eq!(handled.extract(), ("state=7".to_string(), 7));
		/// ```
		#[inline]
		pub fn gets<StateType, Idx>(f: impl FnOnce(StateType) -> A + 'static) -> Self
		where
			StateType: 'static,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, StateType>):
				Member<Coyoneda<'static, BoxStateBrand<BoxBrand, StateType>, StateType>, Idx>, {
			Run::<R, ScopedRow, StateType>::get::<Idx>().map(f)
		}
	}

	#[document_type_parameters("The first-order effect row brand.", "The scoped-effect row brand.")]
	impl<R, ScopedRow> Run<R, ScopedRow, ()>
	where
		R: WrapDrop + Functor + 'static,
		ScopedRow: WrapDrop + Functor + 'static,
	{
		/// Updates the State value with a function.
		///
		/// `modify(f)` is equivalent to `get().bind(|s| put(f(s)))`.
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

	#[document_type_parameters("The first-order effect row brand.", "The result type.")]
	#[document_parameters("The `Run` program to interpret.")]
	impl<R, A> Run<R, CNilBrand, A>
	where
		R: WrapDrop + Functor + 'static,
		A: 'static,
	{
		/// Interprets one State effect by threading an owned state value.
		///
		/// The returned program produces both the original result and the
		/// final state.
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
				move |op: BoxState<
					'static,
					BoxBrand,
					StateType,
					Run<RMinusState, CNilBrand, A>,
				>| {
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

		/// Interprets one State effect and returns only the program result.
		#[document_signature]
		#[document_type_parameters(
			"The State value type.",
			"The type-level Member-position witness for the State effect.",
			"The first-order row brand with the State effect removed."
		)]
		#[document_parameters("The initial state value.")]
		#[document_returns("A first-order-only `Run` program returning the original result.")]
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
		/// let program: Run<Row, CNilBrand, i32> =
		/// 	Run::<Row, CNilBrand, i32>::gets::<i32, _>(|state| state + 1);
		/// let handled: Run<CNilBrand, CNilBrand, i32> = program.eval_state::<i32, _, CNilBrand>(41);
		/// assert_eq!(handled.extract(), 42);
		/// ```
		#[inline]
		pub fn eval_state<StateType, Idx, RMinusState>(
			self,
			initial: StateType,
		) -> Run<RMinusState, CNilBrand, A>
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
			self.run_state::<StateType, Idx, RMinusState>(initial).map(|(result, _state)| result)
		}

		/// Interprets one State effect and returns only the final state.
		#[document_signature]
		#[document_type_parameters(
			"The State value type.",
			"The type-level Member-position witness for the State effect.",
			"The first-order row brand with the State effect removed."
		)]
		#[document_parameters("The initial state value.")]
		#[document_returns("A first-order-only `Run` program returning the final state.")]
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
		/// let program: Run<Row, CNilBrand, ()> =
		/// 	Run::<Row, CNilBrand, ()>::modify::<i32, _>(|state| state + 1);
		/// let handled: Run<CNilBrand, CNilBrand, i32> = program.exec_state::<i32, _, CNilBrand>(41);
		/// assert_eq!(handled.extract(), 42);
		/// ```
		#[inline]
		pub fn exec_state<StateType, Idx, RMinusState>(
			self,
			initial: StateType,
		) -> Run<RMinusState, CNilBrand, StateType>
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
			self.run_state::<StateType, Idx, RMinusState>(initial).map(|(_result, state)| state)
		}
	}

	#[document_type_parameters(
		"The first-order effect row brand.",
		"The scoped-effect row brand.",
		"The result type."
	)]
	impl<R, ScopedRow, A> RcRun<R, ScopedRow, A>
	where
		R: WrapDrop + Functor + 'static,
		ScopedRow: WrapDrop + Functor + 'static,
		A: 'static,
	{
		/// Reads the State value and maps it immediately.
		#[document_signature]
		#[document_type_parameters(
			"The State value type.",
			"The type-level Member-position witness for the State effect."
		)]
		#[document_parameters("The projection to apply to the current state.")]
		#[document_returns("An `RcRun` program that reads the state and returns `f(state)`.")]
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
		/// let program: RcRun<Row, CNilBrand, String> =
		/// 	RcRun::gets::<i32, _>(|state| format!("state={state}"));
		/// let handled: RcRun<CNilBrand, CNilBrand, (String, i32)> =
		/// 	program.run_state::<i32, _, CNilBrand>(7);
		/// assert_eq!(handled.extract(), ("state=7".to_string(), 7));
		/// ```
		#[inline]
		pub fn gets<StateType, Idx>(f: impl Fn(StateType) -> A + 'static) -> Self
		where
			StateType: Clone + 'static,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, StateType>):
				Member<RcCoyoneda<'static, StateBrand<RcBrand, StateType>, StateType>, Idx>,
			Apply!(<NodeBrand<R, ScopedRow> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<NodeBrand<R, ScopedRow>, RcTypeErasedValue>,
			>): Clone, {
			RcRun::<R, ScopedRow, StateType>::get::<Idx>().map(f)
		}
	}

	#[document_type_parameters("The first-order effect row brand.", "The scoped-effect row brand.")]
	impl<R, ScopedRow> RcRun<R, ScopedRow, ()>
	where
		R: WrapDrop + Functor + 'static,
		ScopedRow: WrapDrop + Functor + 'static,
	{
		/// Updates the State value with a function.
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

	#[document_type_parameters("The first-order effect row brand.", "The result type.")]
	#[document_parameters("The `RcRun` program to interpret.")]
	impl<R, A> RcRun<R, CNilBrand, A>
	where
		R: WrapDrop + Functor + 'static,
		A: 'static,
	{
		/// Interprets one State effect by threading an owned state value.
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
				move |op: State<'static, RcBrand, StateType, RcRun<RMinusState, CNilBrand, A>>| {
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

		/// Interprets one State effect and returns only the program result.
		#[document_signature]
		#[document_type_parameters(
			"The State value type.",
			"The type-level Member-position witness for the State effect.",
			"The first-order row brand with the State effect removed."
		)]
		#[document_parameters("The initial state value.")]
		#[document_returns("A first-order-only `RcRun` program returning the original result.")]
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
		/// let program: RcRun<Row, CNilBrand, i32> =
		/// 	RcRun::<Row, CNilBrand, i32>::gets::<i32, _>(|state| state + 1);
		/// let handled: RcRun<CNilBrand, CNilBrand, i32> = program.eval_state::<i32, _, CNilBrand>(41);
		/// assert_eq!(handled.extract(), 42);
		/// ```
		#[inline]
		pub fn eval_state<StateType, Idx, RMinusState>(
			self,
			initial: StateType,
		) -> RcRun<RMinusState, CNilBrand, A>
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
			self.run_state::<StateType, Idx, RMinusState>(initial).map(|(result, _state)| result)
		}

		/// Interprets one State effect and returns only the final state.
		#[document_signature]
		#[document_type_parameters(
			"The State value type.",
			"The type-level Member-position witness for the State effect.",
			"The first-order row brand with the State effect removed."
		)]
		#[document_parameters("The initial state value.")]
		#[document_returns("A first-order-only `RcRun` program returning the final state.")]
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
		/// let program: RcRun<Row, CNilBrand, ()> =
		/// 	RcRun::<Row, CNilBrand, ()>::modify::<i32, _>(|state| state + 1);
		/// let handled: RcRun<CNilBrand, CNilBrand, i32> = program.exec_state::<i32, _, CNilBrand>(41);
		/// assert_eq!(handled.extract(), 42);
		/// ```
		#[inline]
		pub fn exec_state<StateType, Idx, RMinusState>(
			self,
			initial: StateType,
		) -> RcRun<RMinusState, CNilBrand, StateType>
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
			self.run_state::<StateType, Idx, RMinusState>(initial).map(|(_result, state)| state)
		}
	}

	#[document_type_parameters(
		"The first-order effect row brand.",
		"The scoped-effect row brand.",
		"The result type."
	)]
	impl<R, ScopedRow, A> ArcRun<R, ScopedRow, A>
	where
		R: WrapDrop + SendFunctor + 'static,
		ScopedRow: WrapDrop + SendFunctor + 'static,
		NodeBrand<R, ScopedRow>: WrapDrop
			+ Kind_cdc7cd43dac7585f<
				Of<'static, ArcFree<NodeBrand<R, ScopedRow>, ArcTypeErasedValue>>: Send + Sync,
			> + 'static,
		A: Send + Sync + 'static,
	{
		/// Reads the State value and maps it immediately.
		#[document_signature]
		#[document_type_parameters(
			"The State value type.",
			"The type-level Member-position witness for the State effect."
		)]
		#[document_parameters("The projection to apply to the current state.")]
		#[document_returns("An `ArcRun` program that reads the state and returns `f(state)`.")]
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
		/// let program: ArcRun<Row, CNilBrand, String> =
		/// 	ArcRun::gets::<i32, _>(|state| format!("state={state}"));
		/// let handled: ArcRun<CNilBrand, CNilBrand, (String, i32)> =
		/// 	program.run_state::<i32, _, CNilBrand>(7);
		/// assert_eq!(handled.extract(), ("state=7".to_string(), 7));
		/// ```
		#[inline]
		pub fn gets<StateType, Idx>(f: impl Fn(StateType) -> A + Send + Sync + 'static) -> Self
		where
			StateType: Clone + Send + Sync + 'static,
			NodeBrand<R, ScopedRow>: SendFunctor,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, StateType>):
				Member<ArcCoyoneda<'static, SendStateBrand<ArcBrand, StateType>, StateType>, Idx>,
			Apply!(<NodeBrand<R, ScopedRow> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<NodeBrand<R, ScopedRow>, ArcTypeErasedValue>,
			>): Clone, {
			ArcRun::<R, ScopedRow, StateType>::get::<Idx>().map(f)
		}
	}

	#[document_type_parameters("The first-order effect row brand.", "The scoped-effect row brand.")]
	impl<R, ScopedRow> ArcRun<R, ScopedRow, ()>
	where
		R: WrapDrop + SendFunctor + 'static,
		ScopedRow: WrapDrop + SendFunctor + 'static,
		NodeBrand<R, ScopedRow>: WrapDrop
			+ Kind_cdc7cd43dac7585f<
				Of<'static, ArcFree<NodeBrand<R, ScopedRow>, ArcTypeErasedValue>>: Send + Sync,
			> + 'static,
	{
		/// Updates the State value with a function.
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
		pub fn modify<StateType, Idx>(
			f: impl Fn(StateType) -> StateType + Send + Sync + 'static
		) -> Self
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

	#[document_type_parameters("The first-order effect row brand.", "The result type.")]
	#[document_parameters("The `ArcRun` program to interpret.")]
	impl<R, A> ArcRun<R, CNilBrand, A>
	where
		R: WrapDrop + SendFunctor + 'static,
		NodeBrand<R, CNilBrand>: WrapDrop
			+ Kind_cdc7cd43dac7585f<
				Of<'static, ArcFree<NodeBrand<R, CNilBrand>, ArcTypeErasedValue>>: Send + Sync,
			> + 'static,
		A: Send + Sync + 'static,
	{
		/// Interprets one State effect by threading an owned state value.
		#[document_signature]
		#[document_type_parameters(
			"The State value type.",
			"The type-level Member-position witness for the State effect.",
			"The first-order row brand with the State effect removed."
		)]
		#[document_parameters("The initial state value.")]
		#[document_returns(
			"A first-order-only `ArcRun` program returning `(result, final_state)`."
		)]
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
		>,{
			let state = StdArc::new(Mutex::new(initial));
			let handler_state = StdArc::clone(&state);
			let handled = self
				.handle_with::<SendStateBrand<ArcBrand, StateType>, Idx, RMinusState>(
					move |op: SendState<
						'static,
						ArcBrand,
						StateType,
						ArcRun<RMinusState, CNilBrand, A>,
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

		/// Interprets one State effect and returns only the program result.
		#[document_signature]
		#[document_type_parameters(
			"The State value type.",
			"The type-level Member-position witness for the State effect.",
			"The first-order row brand with the State effect removed."
		)]
		#[document_parameters("The initial state value.")]
		#[document_returns("A first-order-only `ArcRun` program returning the original result.")]
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
		/// let program: ArcRun<Row, CNilBrand, i32> =
		/// 	ArcRun::<Row, CNilBrand, i32>::gets::<i32, _>(|state| state + 1);
		/// let handled: ArcRun<CNilBrand, CNilBrand, i32> = program.eval_state::<i32, _, CNilBrand>(41);
		/// assert_eq!(handled.extract(), 42);
		/// ```
		#[inline]
		pub fn eval_state<StateType, Idx, RMinusState>(
			self,
			initial: StateType,
		) -> ArcRun<RMinusState, CNilBrand, A>
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
		>,{
			self.run_state::<StateType, Idx, RMinusState>(initial).map(|(result, _state)| result)
		}

		/// Interprets one State effect and returns only the final state.
		#[document_signature]
		#[document_type_parameters(
			"The State value type.",
			"The type-level Member-position witness for the State effect.",
			"The first-order row brand with the State effect removed."
		)]
		#[document_parameters("The initial state value.")]
		#[document_returns("A first-order-only `ArcRun` program returning the final state.")]
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
		/// let program: ArcRun<Row, CNilBrand, ()> =
		/// 	ArcRun::<Row, CNilBrand, ()>::modify::<i32, _>(|state| state + 1);
		/// let handled: ArcRun<CNilBrand, CNilBrand, i32> = program.exec_state::<i32, _, CNilBrand>(41);
		/// assert_eq!(handled.extract(), 42);
		/// ```
		#[inline]
		pub fn exec_state<StateType, Idx, RMinusState>(
			self,
			initial: StateType,
		) -> ArcRun<RMinusState, CNilBrand, StateType>
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
		>,{
			self.run_state::<StateType, Idx, RMinusState>(initial).map(|(_result, state)| state)
		}
	}

	#[document_type_parameters(
		"The lifetime carried by the explicit wrapper.",
		"The first-order effect row brand.",
		"The scoped-effect row brand.",
		"The result type."
	)]
	impl<'a, R, ScopedRow, A> RunExplicit<'a, R, ScopedRow, A>
	where
		R: WrapDrop + Functor + 'static,
		ScopedRow: WrapDrop + Functor + 'static,
		A: 'a,
	{
		/// Reads the State value and maps it immediately.
		///
		/// `gets(f)` is the State-specific convenience form of
		/// `get().map(f)`.
		#[document_signature]
		#[document_type_parameters(
			"The State value type.",
			"The type-level Member-position witness for the State effect."
		)]
		#[document_parameters("The projection to apply to the current state.")]
		#[document_returns("A `RunExplicit` program that reads the state and returns `f(state)`.")]
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
		/// let program: RunExplicit<'static, Row, CNilBrand, String> =
		/// 	RunExplicit::gets::<i32, _>(|state| format!("state={state}"));
		/// let handled: RunExplicit<'static, CNilBrand, CNilBrand, (String, i32)> =
		/// 	program.run_state::<i32, _, CNilBrand>(7);
		/// assert_eq!(handled.extract(), ("state=7".to_string(), 7));
		/// ```
		#[inline]
		pub fn gets<StateType, Idx>(f: impl Fn(StateType) -> A + 'a) -> Self
		where
			StateType: 'static,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, StateType>):
				Member<Coyoneda<'a, BoxStateBrand<BoxBrand, StateType>, StateType>, Idx>, {
			RunExplicit::<'a, R, ScopedRow, StateType>::get::<Idx>().map(f)
		}
	}

	#[document_type_parameters(
		"The lifetime carried by the explicit wrapper.",
		"The first-order effect row brand.",
		"The scoped-effect row brand."
	)]
	impl<'a, R, ScopedRow> RunExplicit<'a, R, ScopedRow, ()>
	where
		R: WrapDrop + Functor + 'static,
		ScopedRow: WrapDrop + Functor + 'static,
	{
		/// Updates the State value with a function.
		///
		/// `modify(f)` is equivalent to `get().bind(|s| put(f(s)))`.
		#[document_signature]
		#[document_type_parameters(
			"The State value type.",
			"The type-level Member-position witness for the State effect."
		)]
		#[document_parameters("The state update function.")]
		#[document_returns(
			"A `RunExplicit` program that writes the updated state and returns unit."
		)]
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
			RunExplicit::<'a, R, ScopedRow, StateType>::get::<Idx>().bind(move |state| {
				RunExplicit::<'a, R, ScopedRow, ()>::put::<StateType, Idx>(f(state))
			})
		}
	}

	#[document_type_parameters(
		"The lifetime carried by the explicit wrapper.",
		"The first-order effect row brand.",
		"The result type."
	)]
	#[document_parameters("The `RunExplicit` program to interpret.")]
	impl<'a, R, A> RunExplicit<'a, R, CNilBrand, A>
	where
		R: WrapDrop + Functor + 'static,
		A: 'a,
	{
		/// Interprets one State effect by threading an owned state value.
		#[document_signature]
		#[document_type_parameters(
			"The State value type.",
			"The type-level Member-position witness for the State effect.",
			"The first-order row brand with the State effect removed."
		)]
		#[document_parameters("The initial state value.")]
		#[document_returns(
			"A first-order-only `RunExplicit` program returning `(result, final_state)`."
		)]
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
					Coyoneda<
						'a,
						BoxStateBrand<BoxBrand, StateType>,
						RunExplicit<'a, R, CNilBrand, A>,
					>,
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
				move |op: BoxState<
					'a,
					BoxBrand,
					StateType,
					RunExplicit<'a, RMinusState, CNilBrand, A>,
				>| {
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

		/// Interprets one State effect and returns only the program result.
		#[document_signature]
		#[document_type_parameters(
			"The State value type.",
			"The type-level Member-position witness for the State effect.",
			"The first-order row brand with the State effect removed."
		)]
		#[document_parameters("The initial state value.")]
		#[document_returns(
			"A first-order-only `RunExplicit` program returning the original result."
		)]
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
		/// 	RunExplicit::<'static, Row, CNilBrand, i32>::gets::<i32, _>(|state| state + 1);
		/// let handled: RunExplicit<'static, CNilBrand, CNilBrand, i32> =
		/// 	program.eval_state::<i32, _, CNilBrand>(41);
		/// assert_eq!(handled.extract(), 42);
		/// ```
		#[inline]
		pub fn eval_state<StateType, Idx, RMinusState>(
			self,
			initial: StateType,
		) -> RunExplicit<'a, RMinusState, CNilBrand, A>
		where
			StateType: Clone + 'static + 'a,
			RMinusState: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RunExplicit<'a, R, CNilBrand, A>,
			>): Member<
					Coyoneda<
						'a,
						BoxStateBrand<BoxBrand, StateType>,
						RunExplicit<'a, R, CNilBrand, A>,
					>,
					Idx,
					Remainder = Apply!(
									<RMinusState as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
										'a,
										RunExplicit<'a, R, CNilBrand, A>,
									>
								),
				>, {
			self.run_state::<StateType, Idx, RMinusState>(initial).map(|(result, _state)| result)
		}

		/// Interprets one State effect and returns only the final state.
		#[document_signature]
		#[document_type_parameters(
			"The State value type.",
			"The type-level Member-position witness for the State effect.",
			"The first-order row brand with the State effect removed."
		)]
		#[document_parameters("The initial state value.")]
		#[document_returns("A first-order-only `RunExplicit` program returning the final state.")]
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
		/// 	RunExplicit::<'static, Row, CNilBrand, ()>::modify::<i32, _>(|state| state + 1);
		/// let handled: RunExplicit<'static, CNilBrand, CNilBrand, i32> =
		/// 	program.exec_state::<i32, _, CNilBrand>(41);
		/// assert_eq!(handled.extract(), 42);
		/// ```
		#[inline]
		pub fn exec_state<StateType, Idx, RMinusState>(
			self,
			initial: StateType,
		) -> RunExplicit<'a, RMinusState, CNilBrand, StateType>
		where
			StateType: Clone + 'static + 'a,
			RMinusState: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RunExplicit<'a, R, CNilBrand, A>,
			>): Member<
					Coyoneda<
						'a,
						BoxStateBrand<BoxBrand, StateType>,
						RunExplicit<'a, R, CNilBrand, A>,
					>,
					Idx,
					Remainder = Apply!(
									<RMinusState as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
										'a,
										RunExplicit<'a, R, CNilBrand, A>,
									>
								),
				>, {
			self.run_state::<StateType, Idx, RMinusState>(initial).map(|(_result, state)| state)
		}
	}

	#[document_type_parameters(
		"The lifetime carried by the explicit wrapper.",
		"The first-order effect row brand.",
		"The scoped-effect row brand.",
		"The result type."
	)]
	impl<'a, R, ScopedRow, A> RcRunExplicit<'a, R, ScopedRow, A>
	where
		R: WrapDrop + Functor + 'static,
		ScopedRow: WrapDrop + Functor + 'static,
		A: 'a,
	{
		/// Reads the State value and maps it immediately.
		#[document_signature]
		#[document_type_parameters(
			"The State value type.",
			"The type-level Member-position witness for the State effect."
		)]
		#[document_parameters("The projection to apply to the current state.")]
		#[document_returns(
			"An `RcRunExplicit` program that reads the state and returns `f(state)`."
		)]
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
		/// let program: RcRunExplicit<'static, Row, CNilBrand, String> =
		/// 	RcRunExplicit::gets::<i32, _>(|state| format!("state={state}"));
		/// let handled: RcRunExplicit<'static, CNilBrand, CNilBrand, (String, i32)> =
		/// 	program.run_state::<i32, _, CNilBrand>(7);
		/// assert_eq!(handled.extract(), ("state=7".to_string(), 7));
		/// ```
		#[inline]
		pub fn gets<StateType, Idx>(f: impl Fn(StateType) -> A + 'a) -> Self
		where
			A: Clone,
			StateType: Clone + 'static,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, StateType>):
				Member<RcCoyoneda<'a, StateBrand<RcBrand, StateType>, StateType>, Idx>,
			Apply!(<NodeBrand<R, ScopedRow> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, ScopedRow>, StateType>,
			>): Clone,
			Apply!(<NodeBrand<R, ScopedRow> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, ScopedRow>, A>,
			>): Clone, {
			RcRunExplicit::<'a, R, ScopedRow, StateType>::get::<Idx>().map(f)
		}
	}

	#[document_type_parameters(
		"The lifetime carried by the explicit wrapper.",
		"The first-order effect row brand.",
		"The scoped-effect row brand."
	)]
	impl<'a, R, ScopedRow> RcRunExplicit<'a, R, ScopedRow, ()>
	where
		R: WrapDrop + Functor + 'static,
		ScopedRow: WrapDrop + Functor + 'static,
	{
		/// Updates the State value with a function.
		#[document_signature]
		#[document_type_parameters(
			"The State value type.",
			"The type-level Member-position witness for the State effect."
		)]
		#[document_parameters("The state update function.")]
		#[document_returns(
			"An `RcRunExplicit` program that writes the updated state and returns unit."
		)]
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
			RcRunExplicit::<'a, R, ScopedRow, StateType>::get::<Idx>().bind(move |state| {
				RcRunExplicit::<'a, R, ScopedRow, ()>::put::<StateType, Idx>(f(state))
			})
		}
	}

	#[document_type_parameters(
		"The lifetime carried by the explicit wrapper.",
		"The first-order effect row brand.",
		"The result type."
	)]
	#[document_parameters("The `RcRunExplicit` program to interpret.")]
	impl<'a, R, A> RcRunExplicit<'a, R, CNilBrand, A>
	where
		R: WrapDrop + Functor + 'static,
		A: 'a,
	{
		/// Interprets one State effect by threading an owned state value.
		#[document_signature]
		#[document_type_parameters(
			"The State value type.",
			"The type-level Member-position witness for the State effect.",
			"The first-order row brand with the State effect removed."
		)]
		#[document_parameters("The initial state value.")]
		#[document_returns(
			"A first-order-only `RcRunExplicit` program returning `(result, final_state)`."
		)]
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
					RcCoyoneda<
						'a,
						StateBrand<RcBrand, StateType>,
						RcRunExplicit<'a, R, CNilBrand, A>,
					>,
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
				move |op: State<
					'a,
					RcBrand,
					StateType,
					RcRunExplicit<'a, RMinusState, CNilBrand, A>,
				>| {
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

		/// Interprets one State effect and returns only the program result.
		#[document_signature]
		#[document_type_parameters(
			"The State value type.",
			"The type-level Member-position witness for the State effect.",
			"The first-order row brand with the State effect removed."
		)]
		#[document_parameters("The initial state value.")]
		#[document_returns(
			"A first-order-only `RcRunExplicit` program returning the original result."
		)]
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
		/// 	RcRunExplicit::<'static, Row, CNilBrand, i32>::gets::<i32, _>(|state| state + 1);
		/// let handled: RcRunExplicit<'static, CNilBrand, CNilBrand, i32> =
		/// 	program.eval_state::<i32, _, CNilBrand>(41);
		/// assert_eq!(handled.extract(), 42);
		/// ```
		#[inline]
		pub fn eval_state<StateType, Idx, RMinusState>(
			self,
			initial: StateType,
		) -> RcRunExplicit<'a, RMinusState, CNilBrand, A>
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
					RcCoyoneda<
						'a,
						StateBrand<RcBrand, StateType>,
						RcRunExplicit<'a, R, CNilBrand, A>,
					>,
					Idx,
					Remainder = Apply!(
									<RMinusState as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
										'a,
										RcRunExplicit<'a, R, CNilBrand, A>,
									>
								),
				>, {
			self.run_state::<StateType, Idx, RMinusState>(initial).map(|(result, _state)| result)
		}

		/// Interprets one State effect and returns only the final state.
		#[document_signature]
		#[document_type_parameters(
			"The State value type.",
			"The type-level Member-position witness for the State effect.",
			"The first-order row brand with the State effect removed."
		)]
		#[document_parameters("The initial state value.")]
		#[document_returns("A first-order-only `RcRunExplicit` program returning the final state.")]
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
		/// 	RcRunExplicit::<'static, Row, CNilBrand, ()>::modify::<i32, _>(|state| state + 1);
		/// let handled: RcRunExplicit<'static, CNilBrand, CNilBrand, i32> =
		/// 	program.exec_state::<i32, _, CNilBrand>(41);
		/// assert_eq!(handled.extract(), 42);
		/// ```
		#[inline]
		pub fn exec_state<StateType, Idx, RMinusState>(
			self,
			initial: StateType,
		) -> RcRunExplicit<'a, RMinusState, CNilBrand, StateType>
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
			Apply!(<NodeBrand<RMinusState, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<RMinusState, CNilBrand>, StateType>,
			>): Clone,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcRunExplicit<'a, R, CNilBrand, A>,
			>): Member<
					RcCoyoneda<
						'a,
						StateBrand<RcBrand, StateType>,
						RcRunExplicit<'a, R, CNilBrand, A>,
					>,
					Idx,
					Remainder = Apply!(
									<RMinusState as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
										'a,
										RcRunExplicit<'a, R, CNilBrand, A>,
									>
								),
				>, {
			self.run_state::<StateType, Idx, RMinusState>(initial).map(|(_result, state)| state)
		}
	}
}
