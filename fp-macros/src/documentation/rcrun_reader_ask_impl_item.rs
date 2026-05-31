/// Lifts an `Ask` reader effect into the `RcRun` program.
/// Mirrors [`Run::ask`](crate::types::effects::run::Run::ask);
/// see that method for cross-wrapper semantics. Differences for
/// `RcRun`: the [`RcCoyoneda`] variant pairs with the `Rc`-shared
/// substrate. Threads
/// [`RcBrand`](crate::brands::RcBrand) as the pointer kind.
///
/// `Idx` is the type-level position witness identifying where
/// `ReaderBrand<RcBrand, A>` lives in the row `R`. Rust infers
/// `Idx` whenever the effect appears unambiguously in the row.
#[__document_module_generated]
#[document_signature]
///
#[document_type_parameters("The type-level Member-position witness (typically inferred).")]
///
#[document_returns("An `RcRun` program suspended at the lifted `Ask` effect.")]
///
#[document_examples]
///
/// ```
/// use fp_library::{
/// 	brands::*,
/// 	types::effects::rc_run::RcRun,
/// };
///
/// type FirstRow = CoproductBrand<RcCoyonedaBrand<ReaderBrand<RcBrand, i32>>, CNilBrand>;
/// type Scoped = CNilBrand;
///
/// let prog: RcRun<FirstRow, Scoped, i32> = RcRun::ask();
/// let handled: RcRun<CNilBrand, CNilBrand, i32> = prog.run_reader::<i32, _, CNilBrand>(41);
/// assert_eq!(handled.extract(), 41);
/// ```
#[inline]
pub fn ask<Idx>() -> Self
where
	Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, A>):
		Member<RcCoyoneda<'static, crate::brands::ReaderBrand<RcBrand, A>, A>, Idx>,
	Apply!(<NodeBrand<R, ScopedRow> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
		'static,
		RcFree<NodeBrand<R, ScopedRow>, crate::types::rc_free::RcTypeErasedValue>,
	>): Clone, {
	let effect: crate::types::effects::reader::Reader<'static, RcBrand, A, A> =
		crate::types::effects::reader::Reader::Ask(<RcBrand as crate::classes::ToDynCloneFn>::new(
			|e: A| e,
		));
	Self::lift::<crate::brands::ReaderBrand<RcBrand, A>, Idx>(effect)
}
