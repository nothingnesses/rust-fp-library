/// Lifts an `Ask` reader effect into the `ArcRunExplicit`
/// program. Mirrors
/// [`Run::ask`](crate::types::effects::run::Run::ask); see
/// that method for cross-wrapper semantics. Differences for
/// `ArcRunExplicit`: threads
/// [`ArcBrand`](crate::brands::ArcBrand) as the pointer kind
/// and uses
/// [`SendReaderBrand`](crate::brands::SendReaderBrand) (rather
/// than `ReaderBrand`) so the continuation projection is
/// structurally `Send + Sync`.
#[__document_module_generated]
#[document_signature]
///
#[document_type_parameters("The type-level Member-position witness (typically inferred).")]
///
#[document_returns("An `ArcRunExplicit` program suspended at the lifted `Ask` effect.")]
///
#[document_examples]
///
/// ```
/// use fp_library::{
/// 	brands::*,
/// 	types::effects::arc_run_explicit::ArcRunExplicit,
/// };
///
/// type FirstRow = CoproductBrand<ArcCoyonedaBrand<SendReaderBrand<ArcBrand, i32>>, CNilBrand>;
/// type Scoped = CNilBrand;
///
/// let prog: ArcRunExplicit<'static, FirstRow, Scoped, i32> = ArcRunExplicit::ask();
/// let handled: ArcRunExplicit<'static, CNilBrand, CNilBrand, i32> =
/// 	prog.run_reader::<i32, _, CNilBrand>(41);
/// assert_eq!(handled.extract(), 41);
/// ```
#[inline]
pub fn ask<Idx>() -> Self
where
	A: Clone + Send + Sync + 'static,
	Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>):
		Member<ArcCoyoneda<'a, crate::brands::SendReaderBrand<crate::brands::ArcBrand, A>, A>, Idx>,
	Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
		'a,
		ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, A>,
	>): Send + Sync,
	Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
		'a,
		ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, A>,
	>): Send + Sync,
	Apply!(<NodeBrand<R, ScopedRow> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
		'a,
		ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, A>,
	>): Clone + Send + Sync, {
	let effect: crate::types::effects::reader::SendReader<'a, crate::brands::ArcBrand, A, A> =
		crate::types::effects::reader::SendReader::Ask(
			<crate::brands::ArcBrand as crate::classes::ToDynSendFn>::new(|e: A| e),
		);
	Self::lift::<crate::brands::SendReaderBrand<crate::brands::ArcBrand, A>, Idx>(effect)
}
