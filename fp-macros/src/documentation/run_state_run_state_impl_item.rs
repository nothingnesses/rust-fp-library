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
