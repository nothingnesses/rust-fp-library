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
