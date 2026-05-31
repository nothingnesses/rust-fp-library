/// Lifts an `Ask` reader effect into the `ArcRun` program.
/// Mirrors [`Run::ask`](crate::types::effects::run::Run::ask);
/// see that method for cross-wrapper semantics. Differences for
/// `ArcRun`: threads
/// [`ArcBrand`](crate::brands::ArcBrand) as the pointer kind
/// and uses
/// [`SendReaderBrand`](crate::brands::SendReaderBrand) (rather
/// than `ReaderBrand`) so the continuation projection
/// `<ArcBrand as SendRefCountedPointer>::Of<'_, dyn Fn(...) + Send + Sync>`
/// is structurally `Send + Sync`.
#[__document_module_generated]
#[document_signature]
///
#[document_type_parameters("The type-level Member-position witness (typically inferred).")]
///
#[document_returns("An `ArcRun` program suspended at the lifted `Ask` effect.")]
///
#[document_examples]
///
/// ```
/// use fp_library::{
/// 	brands::*,
/// 	types::effects::arc_run::ArcRun,
/// };
///
/// type FirstRow = CoproductBrand<ArcCoyonedaBrand<SendReaderBrand<ArcBrand, i32>>, CNilBrand>;
/// type Scoped = CNilBrand;
///
/// let prog: ArcRun<FirstRow, Scoped, i32> = ArcRun::ask();
/// let handled: ArcRun<CNilBrand, CNilBrand, i32> = prog.run_reader::<i32, _, CNilBrand>(41);
/// assert_eq!(handled.extract(), 41);
/// ```
#[inline]
pub fn ask<Idx>() -> Self
where
	A: Clone + Send + Sync,
	NodeBrand<R, ScopedRow>: SendFunctor,
	Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, A>): Member<
			ArcCoyoneda<'static, crate::brands::SendReaderBrand<crate::brands::ArcBrand, A>, A>,
			Idx,
		>,
	Apply!(<NodeBrand<R, ScopedRow> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
		'static,
		ArcFree<NodeBrand<R, ScopedRow>, ArcTypeErasedValue>,
	>): Clone, {
	let effect: crate::types::effects::reader::SendReader<'static, crate::brands::ArcBrand, A, A> =
		crate::types::effects::reader::SendReader::Ask(
			<crate::brands::ArcBrand as crate::classes::ToDynSendFn>::new(|e: A| e),
		);
	Self::lift::<crate::brands::SendReaderBrand<crate::brands::ArcBrand, A>, Idx>(effect)
}
