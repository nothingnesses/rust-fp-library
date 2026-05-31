/// Lifts a `Put` state effect into the `RcRun` program.
/// Mirrors [`Run::put`](crate::types::effects::run::Run::put);
/// see that method for cross-wrapper semantics. Threads
/// [`RcBrand`](crate::brands::RcBrand) as the pointer kind.
///
/// `StateType` is the state type carried by `StateBrand` in
/// the row. Rust may need a turbofish on `StateType` because
/// `put`'s result type is `()` (which doesn't constrain the
/// state type from the call site).
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
#[document_returns("An `RcRun` program suspended at the lifted `Put` effect.")]
///
#[document_examples]
///
/// ```
/// use fp_library::{
/// 	brands::*,
/// 	types::effects::rc_run::RcRun,
/// };
///
/// type FirstRow = CoproductBrand<RcCoyonedaBrand<StateBrand<RcBrand, i32>>, CNilBrand>;
/// type Scoped = CNilBrand;
///
/// let prog: RcRun<FirstRow, Scoped, ()> = RcRun::put::<i32, _>(42);
/// let handled: RcRun<CNilBrand, CNilBrand, ((), i32)> = prog.run_state::<i32, _, CNilBrand>(0);
/// assert_eq!(handled.extract(), ((), 42));
/// ```
#[inline]
pub fn put<StateType: Clone + 'static, Idx>(s: StateType) -> Self
where
	Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, ()>):
		Member<RcCoyoneda<'static, crate::brands::StateBrand<RcBrand, StateType>, ()>, Idx>,
	Apply!(<NodeBrand<R, ScopedRow> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
		'static,
		RcFree<NodeBrand<R, ScopedRow>, crate::types::rc_free::RcTypeErasedValue>,
	>): Clone, {
	let effect: crate::types::effects::state::State<'static, RcBrand, StateType, ()> =
		crate::types::effects::state::State::Put(
			s,
			<RcBrand as crate::classes::ToDynCloneFn>::new(|_: ()| ()),
		);
	Self::lift::<crate::brands::StateBrand<RcBrand, StateType>, Idx>(effect)
}
