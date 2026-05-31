/// Lifts a `Get` state effect into the `RunExplicit` program.
/// Mirrors [`Run::get`](crate::types::effects::run::Run::get);
/// see that method for cross-wrapper semantics. Differences for
/// `RunExplicit`: the bare [`Coyoneda`] variant pairs with the
/// Box-in-Wrap Explicit substrate (the substrate's `peel` does
/// not require a `Clone` bound on the inner effect). Threads
/// [`BoxBrand`](crate::brands::BoxBrand) as the pointer kind
/// post-Phase-3.5 retrofit so the continuation projection is
/// `Box<dyn FnOnce>` rather than `Rc<dyn Fn>`.
#[__document_module_generated]
#[document_signature]
///
#[document_type_parameters("The type-level Member-position witness (typically inferred).")]
///
#[document_returns("A `RunExplicit` program suspended at the lifted `Get` effect.")]
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
/// let prog: RunExplicit<'static, FirstRow, Scoped, i32> = RunExplicit::get();
/// let handled: RunExplicit<'static, CNilBrand, CNilBrand, (i32, i32)> =
/// 	prog.run_state::<i32, _, CNilBrand>(41);
/// assert_eq!(handled.extract(), (41, 41));
/// ```
#[inline]
pub fn get<Idx>() -> Self
where
	A: 'static,
	Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>):
		Member<Coyoneda<'a, crate::brands::BoxStateBrand<crate::brands::BoxBrand, A>, A>, Idx>, {
	let effect: crate::types::effects::state::BoxState<'a, crate::brands::BoxBrand, A, A> =
		crate::types::effects::state::BoxState::Get(
			<crate::brands::BoxBrand as crate::classes::ToDynFnOnce>::new(|s: A| s),
		);
	Self::lift::<crate::brands::BoxStateBrand<crate::brands::BoxBrand, A>, Idx>(effect)
}
