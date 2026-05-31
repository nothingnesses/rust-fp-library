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
