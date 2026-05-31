/// Lifts a `Get` state effect into the `RcRunExplicit` program.
/// Mirrors [`Run::get`](crate::types::effects::run::Run::get);
/// see that method for cross-wrapper semantics. Differences for
/// `RcRunExplicit`: the [`RcCoyoneda`] variant pairs with the
/// `Rc`-shared Explicit substrate (multi-shot continuations);
/// `A: Clone` is required because the underlying `RcCoyoneda`
/// substrate's `peel` walks shared continuation projections.
/// Threads [`RcBrand`](crate::brands::RcBrand) as the pointer
/// kind.
#[__document_module_generated]
#[document_signature]
///
#[document_type_parameters("The type-level Member-position witness (typically inferred).")]
///
#[document_returns("An `RcRunExplicit` program suspended at the lifted `Get` effect.")]
///
#[document_examples]
///
/// ```
/// use fp_library::{
/// 	brands::*,
/// 	types::effects::rc_run_explicit::RcRunExplicit,
/// };
///
/// type FirstRow = CoproductBrand<RcCoyonedaBrand<StateBrand<RcBrand, i32>>, CNilBrand>;
/// type Scoped = CNilBrand;
///
/// let prog: RcRunExplicit<'static, FirstRow, Scoped, i32> = RcRunExplicit::get();
/// let handled: RcRunExplicit<'static, CNilBrand, CNilBrand, (i32, i32)> =
/// 	prog.run_state::<i32, _, CNilBrand>(41);
/// assert_eq!(handled.extract(), (41, 41));
/// ```
#[inline]
pub fn get<Idx>() -> Self
where
	A: Clone + 'static,
	Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>):
		Member<RcCoyoneda<'a, crate::brands::StateBrand<crate::brands::RcBrand, A>, A>, Idx>, {
	let effect: crate::types::effects::state::State<'a, crate::brands::RcBrand, A, A> =
		crate::types::effects::state::State::Get(
			<crate::brands::RcBrand as crate::classes::ToDynCloneFn>::new(|s: A| s),
		);
	Self::lift::<crate::brands::StateBrand<crate::brands::RcBrand, A>, Idx>(effect)
}
