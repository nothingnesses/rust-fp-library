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
