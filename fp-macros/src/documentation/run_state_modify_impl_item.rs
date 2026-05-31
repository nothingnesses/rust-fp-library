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
