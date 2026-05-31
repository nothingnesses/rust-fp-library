/// Lifts a `Put` state effect into the `ArcRun` program.
/// Mirrors [`Run::put`](crate::types::effects::run::Run::put);
/// see that method for cross-wrapper semantics. Threads
/// [`ArcBrand`](crate::brands::ArcBrand) as the pointer kind
/// and uses
/// [`SendStateBrand`](crate::brands::SendStateBrand); see
/// [`get`](ArcRun::get) for the design rationale.
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
#[document_returns("An `ArcRun` program suspended at the lifted `Put` effect.")]
///
#[document_examples]
///
/// ```
/// use fp_library::{
/// 	brands::*,
/// 	types::effects::arc_run::ArcRun,
/// };
///
/// type FirstRow = CoproductBrand<ArcCoyonedaBrand<SendStateBrand<ArcBrand, i32>>, CNilBrand>;
/// type Scoped = CNilBrand;
///
/// let prog: ArcRun<FirstRow, Scoped, ()> = ArcRun::put::<i32, _>(42);
/// let handled: ArcRun<CNilBrand, CNilBrand, ((), i32)> = prog.run_state::<i32, _, CNilBrand>(0);
/// assert_eq!(handled.extract(), ((), 42));
/// ```
#[inline]
pub fn put<StateType: Clone + Send + Sync + 'static, Idx>(s: StateType) -> Self
where
	NodeBrand<R, ScopedRow>: SendFunctor,
	Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, ()>): Member<
			ArcCoyoneda<
				'static,
				crate::brands::SendStateBrand<crate::brands::ArcBrand, StateType>,
				(),
			>,
			Idx,
		>,
	Apply!(<NodeBrand<R, ScopedRow> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
		'static,
		ArcFree<NodeBrand<R, ScopedRow>, ArcTypeErasedValue>,
	>): Clone, {
	let effect: crate::types::effects::state::SendState<
		'static,
		crate::brands::ArcBrand,
		StateType,
		(),
	> = crate::types::effects::state::SendState::Put(
		s,
		<crate::brands::ArcBrand as crate::classes::ToDynSendFn>::new(|_: ()| ()),
	);
	Self::lift::<crate::brands::SendStateBrand<crate::brands::ArcBrand, StateType>, Idx>(effect)
}
