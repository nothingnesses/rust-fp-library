/// Lifts a `Get` state effect into the `ArcRunExplicit`
/// program. Mirrors
/// [`Run::get`](crate::types::effects::run::Run::get); see
/// that method for cross-wrapper semantics. Differences for
/// `ArcRunExplicit`: threads
/// [`ArcBrand`](crate::brands::ArcBrand) as the pointer kind
/// and uses
/// [`SendStateBrand`](crate::brands::SendStateBrand) (rather
/// than `StateBrand`) so the continuation projection is
/// structurally `Send + Sync`, which the `SendFunctor`
/// algebra requires across thread boundaries.
#[__document_module_generated]
#[document_signature]
///
#[document_type_parameters("The type-level Member-position witness (typically inferred).")]
///
#[document_returns("An `ArcRunExplicit` program suspended at the lifted `Get` effect.")]
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
/// let prog: ArcRunExplicit<'static, FirstRow, Scoped, i32> = ArcRunExplicit::get();
/// let handled: ArcRunExplicit<'static, CNilBrand, CNilBrand, (i32, i32)> =
/// 	prog.run_state::<i32, _, CNilBrand>(41);
/// assert_eq!(handled.extract(), (41, 41));
/// ```
#[inline]
pub fn get<Idx>() -> Self
where
	A: Clone + Send + Sync + 'static,
	Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>):
		Member<ArcCoyoneda<'a, crate::brands::SendStateBrand<crate::brands::ArcBrand, A>, A>, Idx>,
	Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
		'a,
		ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, A>,
	>): Send + Sync,
	Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
		'a,
		ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, A>,
	>): Send + Sync,
	Apply!(<NodeBrand<R, ScopedRow> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
		'a,
		ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, A>,
	>): Clone + Send + Sync, {
	let effect: crate::types::effects::state::SendState<'a, crate::brands::ArcBrand, A, A> =
		crate::types::effects::state::SendState::Get(
			<crate::brands::ArcBrand as crate::classes::ToDynSendFn>::new(|s: A| s),
		);
	Self::lift::<crate::brands::SendStateBrand<crate::brands::ArcBrand, A>, Idx>(effect)
}
