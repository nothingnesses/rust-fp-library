/// Lifts an `Ask` reader effect into the `RunExplicit`
/// program. Mirrors
/// [`Run::ask`](crate::types::effects::run::Run::ask); see
/// that method for cross-wrapper semantics. Differences for
/// `RunExplicit`: the bare [`Coyoneda`] variant pairs with the
/// Box-in-Wrap Explicit substrate. Threads
/// [`BoxBrand`](crate::brands::BoxBrand) as the pointer kind
/// post-Phase-3.5 retrofit.
#[__document_module_generated]
#[document_signature]
///
#[document_type_parameters("The type-level Member-position witness (typically inferred).")]
///
#[document_returns("A `RunExplicit` program suspended at the lifted `Ask` effect.")]
///
#[document_examples]
///
/// ```
/// use fp_library::{
/// 	brands::*,
/// 	types::effects::run_explicit::RunExplicit,
/// };
///
/// type FirstRow = CoproductBrand<CoyonedaBrand<BoxReaderBrand<BoxBrand, i32>>, CNilBrand>;
/// type Scoped = CNilBrand;
///
/// let prog: RunExplicit<'static, FirstRow, Scoped, i32> = RunExplicit::ask();
/// let handled: RunExplicit<'static, CNilBrand, CNilBrand, i32> =
/// 	prog.run_reader::<i32, _, CNilBrand>(41);
/// assert_eq!(handled.extract(), 41);
/// ```
#[inline]
pub fn ask<Idx>() -> Self
where
	A: 'static,
	Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>):
		Member<Coyoneda<'a, crate::brands::BoxReaderBrand<crate::brands::BoxBrand, A>, A>, Idx>, {
	let effect: crate::types::effects::reader::BoxReader<'a, crate::brands::BoxBrand, A, A> =
		crate::types::effects::reader::BoxReader::Ask(
			<crate::brands::BoxBrand as crate::classes::ToDynFnOnce>::new(|e: A| e),
		);
	Self::lift::<crate::brands::BoxReaderBrand<crate::brands::BoxBrand, A>, Idx>(effect)
}
