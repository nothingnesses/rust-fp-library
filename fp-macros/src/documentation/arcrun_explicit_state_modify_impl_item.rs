/// Updates the State value with a function.
#[__document_module_generated]
#[document_signature]
#[document_type_parameters(
	"The State value type.",
	"The type-level Member-position witness for the State effect."
)]
#[document_parameters("The state update function.")]
#[document_returns("An `ArcRunExplicit` program that writes the updated state and returns unit.")]
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
/// let program: ArcRunExplicit<'static, Row, CNilBrand, ()> =
/// 	ArcRunExplicit::modify::<i32, _>(|state| state + 1);
/// let handled: ArcRunExplicit<'static, CNilBrand, CNilBrand, ((), i32)> =
/// 	program.run_state::<i32, _, CNilBrand>(41);
/// assert_eq!(handled.extract(), ((), 42));
/// ```
#[inline]
pub fn modify<StateType, Idx>(f: impl Fn(StateType) -> StateType + Send + Sync + 'a) -> Self
where
	StateType: Clone + Send + Sync + 'static,
	NodeBrand<R, ScopedRow>: SendFunctor,
	Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, StateType>):
		Member<ArcCoyoneda<'a, SendStateBrand<ArcBrand, StateType>, StateType>, Idx>,
	Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, ()>):
		Member<ArcCoyoneda<'a, SendStateBrand<ArcBrand, StateType>, ()>, Idx>,
	Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
		'a,
		ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, StateType>,
	>): Send + Sync,
	Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
		'a,
		ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, StateType>,
	>): Send + Sync,
	Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
		'a,
		ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, ()>,
	>): Send + Sync,
	Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
		'a,
		ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, ()>,
	>): Send + Sync,
	Apply!(<NodeBrand<R, ScopedRow> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
		'a,
		ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, StateType>,
	>): Clone + Send + Sync,
	Apply!(<NodeBrand<R, ScopedRow> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
		'a,
		ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, ()>,
	>): Clone + Send + Sync, {
	ArcRunExplicit::<'a, R, ScopedRow, StateType>::get::<Idx>()
		.bind(move |state| ArcRunExplicit::<'a, R, ScopedRow, ()>::put::<StateType, Idx>(f(state)))
}
