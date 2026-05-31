/// Lifts a `Put` state effect into the `RcRunExplicit` program.
/// Mirrors [`Run::put`](crate::types::effects::run::Run::put);
/// see that method for cross-wrapper semantics. Threads
/// [`RcBrand`](crate::brands::RcBrand) as the pointer kind.
#[__document_module_generated]
#[document_signature]
///
#[document_type_parameters(
	"The state type carried by `StateBrand` in the row.",
	"The type-level Member-position witness (typically inferred)."
)]
///
#[document_parameters("The new state value to write.")]
///
#[document_returns("An `RcRunExplicit` program suspended at the lifted `Put` effect.")]
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
/// let prog: RcRunExplicit<'static, FirstRow, Scoped, ()> = RcRunExplicit::put::<i32, _>(42);
/// let handled: RcRunExplicit<'static, CNilBrand, CNilBrand, ((), i32)> =
/// 	prog.run_state::<i32, _, CNilBrand>(0);
/// assert_eq!(handled.extract(), ((), 42));
/// ```
#[inline]
pub fn put<StateType: Clone + 'static, Idx>(s: StateType) -> Self
where
	Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, ()>): Member<
			RcCoyoneda<'a, crate::brands::StateBrand<crate::brands::RcBrand, StateType>, ()>,
			Idx,
		>, {
	let effect: crate::types::effects::state::State<'a, crate::brands::RcBrand, StateType, ()> =
		crate::types::effects::state::State::Put(
			s,
			<crate::brands::RcBrand as crate::classes::ToDynCloneFn>::new(|_: ()| ()),
		);
	Self::lift::<crate::brands::StateBrand<crate::brands::RcBrand, StateType>, Idx>(effect)
}
