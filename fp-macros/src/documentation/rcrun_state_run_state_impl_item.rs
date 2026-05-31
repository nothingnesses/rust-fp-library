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
