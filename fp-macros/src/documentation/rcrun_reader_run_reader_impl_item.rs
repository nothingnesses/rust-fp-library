/// Interprets one Reader effect by supplying a fixed environment.
#[__document_module_generated]
#[document_signature]
#[document_type_parameters(
	"The Reader environment type.",
	"The type-level Member-position witness for the Reader effect.",
	"The first-order row brand with the Reader effect removed."
)]
#[document_parameters("The environment value supplied to every Reader ask.")]
#[document_returns("A first-order-only `RcRun` program with the Reader effect removed.")]
#[document_examples]
///
/// ```
/// use fp_library::{
/// 	brands::*,
/// 	types::effects::rc_run::RcRun,
/// };
///
/// type Row = CoproductBrand<RcCoyonedaBrand<ReaderBrand<RcBrand, i32>>, CNilBrand>;
///
/// let program: RcRun<Row, CNilBrand, i32> =
/// 	RcRun::<Row, CNilBrand, i32>::ask().map(|env| env + 1);
/// let handled: RcRun<CNilBrand, CNilBrand, i32> = program.run_reader::<i32, _, CNilBrand>(41);
/// assert_eq!(handled.extract(), 42);
/// ```
#[inline]
pub fn run_reader<E, Idx, RMinusReader>(
	self,
	env: E,
) -> RcRun<RMinusReader, CNilBrand, A>
where
	A: Clone,
	E: Clone + 'static,
	RMinusReader: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
	Apply!(<NodeBrand<R, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
		'static,
		RcFree<NodeBrand<R, CNilBrand>, RcTypeErasedValue>,
	>): Clone,
	Apply!(<NodeBrand<RMinusReader, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
		'static,
		RcFree<NodeBrand<RMinusReader, CNilBrand>, RcTypeErasedValue>,
	>): Clone,
	Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
		'static,
		RcRun<R, CNilBrand, A>,
	>): Member<
			RcCoyoneda<'static, ReaderBrand<RcBrand, E>, RcRun<R, CNilBrand, A>>,
			Idx,
			Remainder = Apply!(
							<RMinusReader as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
								'static,
								RcRun<R, CNilBrand, A>,
							>
						),
		>, {
	self.handle_with::<ReaderBrand<RcBrand, E>, Idx, RMinusReader>(
		move |op: Reader<'static, RcBrand, E, RcRun<RMinusReader, CNilBrand, A>>| match op {
			Reader::Ask(k) => (*k)(env.clone()),
		},
	)
}
