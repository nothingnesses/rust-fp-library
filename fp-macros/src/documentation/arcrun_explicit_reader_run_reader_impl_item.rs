/// Interprets one Reader effect by supplying a fixed environment.
#[__document_module_generated]
#[document_signature]
#[document_type_parameters(
	"The Reader environment type.",
	"The type-level Member-position witness for the Reader effect.",
	"The first-order row brand with the Reader effect removed."
)]
#[document_parameters("The environment value supplied to every Reader ask.")]
#[document_returns("A first-order-only `ArcRunExplicit` program with the Reader effect removed.")]
#[document_examples]
///
/// ```
/// use fp_library::{
/// 	brands::*,
/// 	types::effects::arc_run_explicit::ArcRunExplicit,
/// };
///
/// type Row = CoproductBrand<ArcCoyonedaBrand<SendReaderBrand<ArcBrand, i32>>, CNilBrand>;
///
/// let program: ArcRunExplicit<'static, Row, CNilBrand, i32> =
/// 	ArcRunExplicit::<'static, Row, CNilBrand, i32>::ask().map(|env| env + 1);
/// let handled: ArcRunExplicit<'static, CNilBrand, CNilBrand, i32> =
/// 	program.run_reader::<i32, _, CNilBrand>(41);
/// assert_eq!(handled.extract(), 42);
/// ```
#[inline]
pub fn run_reader<E, Idx, RMinusReader>(
	self,
	env: E,
) -> ArcRunExplicit<'a, RMinusReader, CNilBrand, A>
where
	A: Clone + Send + Sync,
	E: Clone + Send + Sync + 'static + 'a,
	RMinusReader: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
	Apply!(<NodeBrand<R, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
		'a,
		ArcFreeExplicit<'a, NodeBrand<R, CNilBrand>, A>,
	>): Clone + Send + Sync,
	Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
		'a,
		ArcFreeExplicit<'a, NodeBrand<R, CNilBrand>, A>,
	>): Send + Sync,
	Apply!(<CNilBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
		'a,
		ArcFreeExplicit<'a, NodeBrand<R, CNilBrand>, A>,
	>): Send + Sync,
	Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
		'a,
		ArcRunExplicit<'a, R, CNilBrand, A>,
	>): Send + Sync,
	Apply!(<CNilBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
		'a,
		ArcRunExplicit<'a, R, CNilBrand, A>,
	>): Send + Sync,
	Apply!(<NodeBrand<RMinusReader, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
		'a,
		ArcFreeExplicit<'a, NodeBrand<RMinusReader, CNilBrand>, A>,
	>): Clone + Send + Sync,
	Apply!(<RMinusReader as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
		'a,
		ArcFreeExplicit<'a, NodeBrand<RMinusReader, CNilBrand>, A>,
	>): Send + Sync,
	Apply!(<RMinusReader as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
		'a,
		ArcRunExplicit<'a, RMinusReader, CNilBrand, A>,
	>): Send + Sync,
	Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
		'a,
		ArcRunExplicit<'a, R, CNilBrand, A>,
	>): Member<
			ArcCoyoneda<'a, SendReaderBrand<ArcBrand, E>, ArcRunExplicit<'a, R, CNilBrand, A>>,
			Idx,
			Remainder = Apply!(
							<RMinusReader as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
								'a,
								ArcRunExplicit<'a, R, CNilBrand, A>,
							>
						),
		>, {
	self.handle_with::<SendReaderBrand<ArcBrand, E>, Idx, RMinusReader>(
		move |op: SendReader<'a, ArcBrand, E, ArcRunExplicit<'a, RMinusReader, CNilBrand, A>>| {
			match op {
				SendReader::Ask(k) => (*k)(env.clone()),
			}
		},
	)
}
