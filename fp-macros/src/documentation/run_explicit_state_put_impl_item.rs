/// Lifts a `Put` state effect into the `RunExplicit` program.
/// Mirrors [`Run::put`](crate::types::effects::run::Run::put);
/// see that method for cross-wrapper semantics. Threads
/// [`BoxBrand`](crate::brands::BoxBrand) as the pointer kind
/// post-Phase-3.5 retrofit so the continuation projection is
/// `Box<dyn FnOnce>`.
#[__document_module_generated]
#[document_signature]
///
#[document_type_parameters(
	"The state type carried by `BoxStateBrand` in the row.",
	"The type-level Member-position witness (typically inferred)."
)]
///
#[document_parameters("The new state value to write.")]
///
#[document_returns("A `RunExplicit` program suspended at the lifted `Put` effect.")]
///
#[document_examples]
///
/// ```
/// use fp_library::{
/// 	brands::*,
/// 	types::effects::run_explicit::RunExplicit,
/// };
///
/// type FirstRow = CoproductBrand<CoyonedaBrand<BoxStateBrand<BoxBrand, i32>>, CNilBrand>;
/// type Scoped = CNilBrand;
///
/// let prog: RunExplicit<'static, FirstRow, Scoped, ()> = RunExplicit::put::<i32, _>(42);
/// let handled: RunExplicit<'static, CNilBrand, CNilBrand, ((), i32)> =
/// 	prog.run_state::<i32, _, CNilBrand>(0);
/// assert_eq!(handled.extract(), ((), 42));
/// ```
#[inline]
pub fn put<StateType: 'static, Idx>(s: StateType) -> Self
where
	Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, ()>): Member<
			Coyoneda<'a, crate::brands::BoxStateBrand<crate::brands::BoxBrand, StateType>, ()>,
			Idx,
		>, {
	let effect: crate::types::effects::state::BoxState<'a, crate::brands::BoxBrand, StateType, ()> =
		crate::types::effects::state::BoxState::Put(
			s,
			<crate::brands::BoxBrand as crate::classes::ToDynFnOnce>::new(|_: ()| ()),
		);
	Self::lift::<crate::brands::BoxStateBrand<crate::brands::BoxBrand, StateType>, Idx>(effect)
}
