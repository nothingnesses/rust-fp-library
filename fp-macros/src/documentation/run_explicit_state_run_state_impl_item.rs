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
