/// Lifts a `Get` state effect into the `RcRun` program.
/// Mirrors [`Run::get`](crate::types::effects::run::Run::get);
/// see that method for cross-wrapper semantics. Differences for
/// `RcRun`: the [`RcCoyoneda`] variant pairs with the `Rc`-shared
/// substrate (single `Get` continuation cloning is via the
/// `Rc<dyn Fn>` refcount bump). Threads
/// [`RcBrand`](crate::brands::RcBrand) as the pointer kind.
///
/// `Idx` is the type-level position witness identifying where
/// `StateBrand<RcBrand, A>` lives in the row `R`. Rust infers
/// `Idx` whenever the effect appears unambiguously in the row.
#[__document_module_generated]
#[document_signature]
///
#[document_type_parameters("The type-level Member-position witness (typically inferred).")]
///
#[document_returns("An `RcRun` program suspended at the lifted `Get` effect.")]
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
/// let prog: RcRun<FirstRow, Scoped, i32> = RcRun::get();
/// let handled: RcRun<CNilBrand, CNilBrand, (i32, i32)> = prog.run_state::<i32, _, CNilBrand>(41);
/// assert_eq!(handled.extract(), (41, 41));
/// ```
#[inline]
pub fn get<Idx>() -> Self
where
	Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, A>):
		Member<RcCoyoneda<'static, crate::brands::StateBrand<RcBrand, A>, A>, Idx>,
	Apply!(<NodeBrand<R, ScopedRow> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
		'static,
		RcFree<NodeBrand<R, ScopedRow>, crate::types::rc_free::RcTypeErasedValue>,
	>): Clone, {
	let effect: crate::types::effects::state::State<'static, RcBrand, A, A> =
		crate::types::effects::state::State::Get(<RcBrand as crate::classes::ToDynCloneFn>::new(
			|s: A| s,
		));
	Self::lift::<crate::brands::StateBrand<RcBrand, A>, Idx>(effect)
}
