/// Lifts a `Put` state effect into the `ArcRunExplicit`
/// program. Mirrors
/// [`Run::put`](crate::types::effects::run::Run::put); see
/// that method for cross-wrapper semantics. Threads
/// [`ArcBrand`](crate::brands::ArcBrand) as the pointer kind
/// and uses
/// [`SendStateBrand`](crate::brands::SendStateBrand); see
/// [`get`](ArcRunExplicit::get) for the design rationale.
#[__document_module_generated]
#[document_signature]
///
#[document_type_parameters(
	"The state type carried by `SendStateBrand` in the row.",
	"The type-level Member-position witness (typically inferred)."
)]
///
#[document_parameters("The new state value to write.")]
///
#[document_returns("An `ArcRunExplicit` program suspended at the lifted `Put` effect.")]
///
#[document_examples]
///
/// ```
/// use fp_library::{
/// 	brands::*,
/// 	types::effects::arc_run_explicit::ArcRunExplicit,
/// };
///
/// type FirstRow = CoproductBrand<ArcCoyonedaBrand<SendStateBrand<ArcBrand, i32>>, CNilBrand>;
/// type Scoped = CNilBrand;
///
/// let prog: ArcRunExplicit<'static, FirstRow, Scoped, ()> = ArcRunExplicit::put::<i32, _>(42);
/// let handled: ArcRunExplicit<'static, CNilBrand, CNilBrand, ((), i32)> =
/// 	prog.run_state::<i32, _, CNilBrand>(0);
/// assert_eq!(handled.extract(), ((), 42));
/// ```
#[inline]
pub fn put<StateType: Clone + Send + Sync + 'static, Idx>(s: StateType) -> Self
where
	Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, ()>): Member<
			ArcCoyoneda<'a, crate::brands::SendStateBrand<crate::brands::ArcBrand, StateType>, ()>,
			Idx,
		>,
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
		ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, ()>,
	>): Clone + Send + Sync, {
	let effect: crate::types::effects::state::SendState<
		'a,
		crate::brands::ArcBrand,
		StateType,
		(),
	> = crate::types::effects::state::SendState::Put(
		s,
		<crate::brands::ArcBrand as crate::classes::ToDynSendFn>::new(|_: ()| ()),
	);
	Self::lift::<crate::brands::SendStateBrand<crate::brands::ArcBrand, StateType>, Idx>(effect)
}
