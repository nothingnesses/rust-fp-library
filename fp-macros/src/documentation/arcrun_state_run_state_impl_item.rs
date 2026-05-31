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
