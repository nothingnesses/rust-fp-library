/// Interprets one Reader effect by supplying a fixed environment.
#[__document_module_generated]
#[document_signature]
#[document_type_parameters(
	"The Reader environment type.",
	"The type-level Member-position witness for the Reader effect.",
	"The first-order row brand with the Reader effect removed."
)]
#[document_parameters("The environment value supplied to every Reader ask.")]
#[document_returns("A first-order-only `ArcRun` program with the Reader effect removed.")]
#[document_examples]
///
/// ```
/// use fp_library::{
/// 	brands::*,
/// 	types::effects::arc_run::ArcRun,
/// };
///
/// type Row = CoproductBrand<ArcCoyonedaBrand<SendReaderBrand<ArcBrand, i32>>, CNilBrand>;
///
/// let program: ArcRun<Row, CNilBrand, i32> =
/// 	ArcRun::<Row, CNilBrand, i32>::ask().map(|env| env + 1);
/// let handled: ArcRun<CNilBrand, CNilBrand, i32> = program.run_reader::<i32, _, CNilBrand>(41);
/// assert_eq!(handled.extract(), 42);
/// ```
#[inline]
pub fn run_reader<E, Idx, RMinusReader>(
	self,
	env: E,
) -> ArcRun<RMinusReader, CNilBrand, A>
where
	A: Clone + Send + Sync,
	E: Clone + Send + Sync + 'static,
	R: Kind_cdc7cd43dac7585f + 'static,
	RMinusReader: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
	NodeBrand<R, CNilBrand>: SendFunctor,
	NodeBrand<RMinusReader, CNilBrand>: WrapDrop
		+ Kind_cdc7cd43dac7585f<
			Of<'static, ArcFree<NodeBrand<RMinusReader, CNilBrand>, ArcTypeErasedValue>>: Send
			                                                                                  + Sync,
		> + SendFunctor,
	Apply!(<NodeBrand<R, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
		'static,
		ArcFree<NodeBrand<R, CNilBrand>, ArcTypeErasedValue>,
	>): Clone,
	Apply!(<NodeBrand<RMinusReader, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
		'static,
		ArcFree<NodeBrand<RMinusReader, CNilBrand>, ArcTypeErasedValue>,
	>): Clone,
	Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
		'static,
		ArcRun<R, CNilBrand, A>,
	>): Member<
			ArcCoyoneda<'static, SendReaderBrand<ArcBrand, E>, ArcRun<R, CNilBrand, A>>,
			Idx,
			Remainder = Apply!(
							<RMinusReader as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
								'static,
								ArcRun<R, CNilBrand, A>,
							>
						),
		>, {
	self.handle_with::<SendReaderBrand<ArcBrand, E>, Idx, RMinusReader>(
		move |op: SendReader<'static, ArcBrand, E, ArcRun<RMinusReader, CNilBrand, A>>| match op {
			SendReader::Ask(k) => (*k)(env.clone()),
		},
	)
}
