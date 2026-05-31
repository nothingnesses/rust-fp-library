/// Lifts an `Ask` reader effect into the Run program. Direct
/// analog of PureScript Run's `ask`. The program reads the
/// immutable environment and returns it as the result type
/// `A` (the environment type and the result type coincide
/// for `ask`).
///
/// `Idx` is the type-level position witness identifying where
/// `BoxReaderBrand<BoxBrand, A>` lives in the row `R`. Rust
/// infers `Idx` whenever the effect appears unambiguously in
/// the row.
#[__document_module_generated]
#[document_signature]
///
#[document_type_parameters("The type-level Member-position witness (typically inferred).")]
///
#[document_returns("A `Run` program suspended at the lifted `Ask` effect.")]
///
#[document_examples]
///
/// ```
/// use fp_library::{
/// 	brands::*,
/// 	types::effects::run::Run,
/// };
///
/// type FirstRow = CoproductBrand<CoyonedaBrand<BoxReaderBrand<BoxBrand, i32>>, CNilBrand>;
/// type Scoped = CNilBrand;
///
/// let prog: Run<FirstRow, Scoped, i32> = Run::ask();
/// let handled: Run<CNilBrand, CNilBrand, i32> = prog.run_reader::<i32, _, CNilBrand>(41);
/// assert_eq!(handled.extract(), 41);
/// ```
#[inline]
pub fn ask<Idx>() -> Self
where
	Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, A>):
		crate::types::effects::member::Member<
				crate::types::Coyoneda<
					'static,
					crate::brands::BoxReaderBrand<crate::brands::BoxBrand, A>,
					A,
				>,
				Idx,
			>, {
	let effect: crate::types::effects::reader::BoxReader<'static, crate::brands::BoxBrand, A, A> =
		crate::types::effects::reader::BoxReader::Ask(
			<crate::brands::BoxBrand as crate::classes::ToDynFnOnce>::new(|e: A| e),
		);
	Self::lift::<crate::brands::BoxReaderBrand<crate::brands::BoxBrand, A>, Idx>(effect)
}
