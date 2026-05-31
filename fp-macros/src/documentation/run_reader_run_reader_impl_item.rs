/// Interprets one Reader effect by supplying a fixed environment.
///
/// Each `Ask` receives a clone of `env`, so this helper is valid
/// for programs with multiple Reader asks. The method narrows the
/// first-order row by removing the selected Reader brand.
#[__document_module_generated]
#[document_signature]
#[document_type_parameters(
	"The Reader environment type.",
	"The type-level Member-position witness for the Reader effect.",
	"The first-order row brand with the Reader effect removed."
)]
#[document_parameters("The environment value supplied to every Reader ask.")]
#[document_returns("A first-order-only `Run` program with the Reader effect removed.")]
#[document_examples]
///
/// ```
/// use fp_library::{
/// 	brands::*,
/// 	types::effects::run::Run,
/// };
///
/// type Row = CoproductBrand<CoyonedaBrand<BoxReaderBrand<BoxBrand, i32>>, CNilBrand>;
///
/// let program: Run<Row, CNilBrand, i32> = Run::<Row, CNilBrand, i32>::ask().map(|env| env + 1);
/// let handled: Run<CNilBrand, CNilBrand, i32> = program.run_reader::<i32, _, CNilBrand>(41);
/// assert_eq!(handled.extract(), 42);
/// ```
#[inline]
pub fn run_reader<E, Idx, RMinusReader>(
	self,
	env: E,
) -> Run<RMinusReader, CNilBrand, A>
where
	E: Clone + 'static,
	RMinusReader: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
	Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, Run<R, CNilBrand, A>>): Member<
			Coyoneda<'static, BoxReaderBrand<BoxBrand, E>, Run<R, CNilBrand, A>>,
			Idx,
			Remainder = Apply!(
							<RMinusReader as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
								'static,
								Run<R, CNilBrand, A>,
							>
						),
		>, {
	self.handle_with::<BoxReaderBrand<BoxBrand, E>, Idx, RMinusReader>(
		move |op: BoxReader<'static, BoxBrand, E, Run<RMinusReader, CNilBrand, A>>| match op {
			BoxReader::Ask(k) => k(env.clone()),
		},
	)
}
