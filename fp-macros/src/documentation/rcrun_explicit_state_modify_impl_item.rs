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
