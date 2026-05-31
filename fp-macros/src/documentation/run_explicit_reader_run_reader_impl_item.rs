/// Interprets one Reader effect by supplying a fixed environment.
#[__document_module_generated]
#[document_signature]
#[document_type_parameters(
	"The Reader environment type.",
	"The type-level Member-position witness for the Reader effect.",
	"The first-order row brand with the Reader effect removed."
)]
#[document_parameters("The environment value supplied to every Reader ask.")]
#[document_returns("A first-order-only `RunExplicit` program with the Reader effect removed.")]
#[document_examples]
///
/// ```
/// use fp_library::{
/// 	brands::*,
/// 	types::effects::run_explicit::RunExplicit,
/// };
///
/// type Row = CoproductBrand<CoyonedaBrand<BoxReaderBrand<BoxBrand, i32>>, CNilBrand>;
///
/// let program: RunExplicit<'static, Row, CNilBrand, i32> =
/// 	RunExplicit::<'static, Row, CNilBrand, i32>::ask().map(|env| env + 1);
/// let handled: RunExplicit<'static, CNilBrand, CNilBrand, i32> =
/// 	program.run_reader::<i32, _, CNilBrand>(41);
/// assert_eq!(handled.extract(), 42);
/// ```
#[inline]
pub fn run_reader<E, Idx, RMinusReader>(
	self,
	env: E,
) -> RunExplicit<'a, RMinusReader, CNilBrand, A>
where
	E: Clone + 'static + 'a,
	RMinusReader: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
	Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
		'a,
		RunExplicit<'a, R, CNilBrand, A>,
	>): Member<
			Coyoneda<'a, BoxReaderBrand<BoxBrand, E>, RunExplicit<'a, R, CNilBrand, A>>,
			Idx,
			Remainder = Apply!(
							<RMinusReader as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
								'a,
								RunExplicit<'a, R, CNilBrand, A>,
							>
						),
		>, {
	self.handle_with::<BoxReaderBrand<BoxBrand, E>, Idx, RMinusReader>(
		move |op: BoxReader<'a, BoxBrand, E, RunExplicit<'a, RMinusReader, CNilBrand, A>>| match op
		{
			BoxReader::Ask(k) => k(env.clone()),
		},
	)
}
