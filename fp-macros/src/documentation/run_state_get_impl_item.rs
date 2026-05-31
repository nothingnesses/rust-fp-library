/// Lifts a `Get` state effect into the Run program. Direct
/// analog of PureScript Run's
/// [`get`](https://github.com/natefaubion/purescript-run/blob/main/src/Run/State.purs).
/// The program reads the current state and returns it as the
/// result type `A` (the state type and the result type
/// coincide for `get`).
///
/// `Idx` is the type-level position witness identifying where
/// `BoxStateBrand<BoxBrand, A>` lives in the row `R`. Rust
/// infers `Idx` whenever the effect appears unambiguously in
/// the row.
#[__document_module_generated]
#[document_signature]
///
#[document_type_parameters("The type-level Member-position witness (typically inferred).")]
///
#[document_returns("A `Run` program suspended at the lifted `Get` effect.")]
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
/// let prog: Run<FirstRow, Scoped, i32> = Run::get();
/// let handled: Run<CNilBrand, CNilBrand, (i32, i32)> = prog.run_state::<i32, _, CNilBrand>(41);
/// assert_eq!(handled.extract(), (41, 41));
/// ```
#[inline]
pub fn get<Idx>() -> Self
where
	Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, A>):
		crate::types::effects::member::Member<
				crate::types::Coyoneda<
					'static,
					crate::brands::BoxStateBrand<crate::brands::BoxBrand, A>,
					A,
				>,
				Idx,
			>, {
	let effect: crate::types::effects::state::BoxState<'static, crate::brands::BoxBrand, A, A> =
		crate::types::effects::state::BoxState::Get(
			<crate::brands::BoxBrand as crate::classes::ToDynFnOnce>::new(|s: A| s),
		);
	Self::lift::<crate::brands::BoxStateBrand<crate::brands::BoxBrand, A>, Idx>(effect)
}
