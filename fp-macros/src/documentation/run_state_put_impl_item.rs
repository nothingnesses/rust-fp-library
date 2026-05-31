/// Lifts a `Put` state effect into the Run program. Direct
/// analog of PureScript Run's
/// [`put`](https://github.com/natefaubion/purescript-run/blob/main/src/Run/State.purs).
/// The program writes the supplied state value `s` and
/// returns `()` as the result type.
///
/// `StateType` is the state type carried by `BoxStateBrand`
/// in the row. Rust may need a turbofish on `StateType`
/// because `put`'s result type is `()` (which doesn't
/// constrain the state type from the call site).
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
#[document_returns("A `Run` program suspended at the lifted `Put` effect.")]
///
#[document_examples]
///
/// ```
/// use fp_library::{
/// 	brands::*,
/// 	types::effects::run::Run,
/// };
///
/// type FirstRow = CoproductBrand<CoyonedaBrand<BoxStateBrand<BoxBrand, i32>>, CNilBrand>;
/// type Scoped = CNilBrand;
///
/// let prog: Run<FirstRow, Scoped, ()> = Run::put::<i32, _>(42);
/// let handled: Run<CNilBrand, CNilBrand, ((), i32)> = prog.run_state::<i32, _, CNilBrand>(0);
/// assert_eq!(handled.extract(), ((), 42));
/// ```
#[inline]
pub fn put<StateType: 'static, Idx>(s: StateType) -> Self
where
	Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, ()>):
		crate::types::effects::member::Member<
				crate::types::Coyoneda<
					'static,
					crate::brands::BoxStateBrand<crate::brands::BoxBrand, StateType>,
					(),
				>,
				Idx,
			>, {
	let effect: crate::types::effects::state::BoxState<
		'static,
		crate::brands::BoxBrand,
		StateType,
		(),
	> = crate::types::effects::state::BoxState::Put(
		s,
		<crate::brands::BoxBrand as crate::classes::ToDynFnOnce>::new(|_: ()| ()),
	);
	Self::lift::<crate::brands::BoxStateBrand<crate::brands::BoxBrand, StateType>, Idx>(effect)
}
