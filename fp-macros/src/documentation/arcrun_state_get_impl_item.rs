/// Lifts a `Get` state effect into the `ArcRun` program.
/// Mirrors [`Run::get`](crate::types::effects::run::Run::get);
/// see that method for cross-wrapper semantics. Differences for
/// `ArcRun`: threads
/// [`ArcBrand`](crate::brands::ArcBrand) as the pointer kind
/// and uses
/// [`SendStateBrand`](crate::brands::SendStateBrand) (rather
/// than `StateBrand`) so the continuation projection
/// `<ArcBrand as SendRefCountedPointer>::Of<'_, dyn Fn(...) + Send + Sync>`
/// is structurally `Send + Sync`, which the `SendFunctor`
/// algebra requires across thread boundaries.
#[__document_module_generated]
#[document_signature]
///
#[document_type_parameters("The type-level Member-position witness (typically inferred).")]
///
#[document_returns("An `ArcRun` program suspended at the lifted `Get` effect.")]
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
/// let prog: ArcRun<FirstRow, Scoped, i32> = ArcRun::get();
/// let handled: ArcRun<CNilBrand, CNilBrand, (i32, i32)> = prog.run_state::<i32, _, CNilBrand>(41);
/// assert_eq!(handled.extract(), (41, 41));
/// ```
#[inline]
pub fn get<Idx>() -> Self
where
	A: Clone + Send + Sync,
	NodeBrand<R, ScopedRow>: SendFunctor,
	Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, A>): Member<
			ArcCoyoneda<'static, crate::brands::SendStateBrand<crate::brands::ArcBrand, A>, A>,
			Idx,
		>,
	Apply!(<NodeBrand<R, ScopedRow> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
		'static,
		ArcFree<NodeBrand<R, ScopedRow>, ArcTypeErasedValue>,
	>): Clone, {
	let effect: crate::types::effects::state::SendState<'static, crate::brands::ArcBrand, A, A> =
		crate::types::effects::state::SendState::Get(
			<crate::brands::ArcBrand as crate::classes::ToDynSendFn>::new(|s: A| s),
		);
	Self::lift::<crate::brands::SendStateBrand<crate::brands::ArcBrand, A>, Idx>(effect)
}
