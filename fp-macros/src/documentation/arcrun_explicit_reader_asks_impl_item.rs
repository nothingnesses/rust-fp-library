/// Reads the Reader environment and maps it immediately.
#[__document_module_generated]
#[document_signature]
#[document_type_parameters(
	"The Reader environment type.",
	"The type-level Member-position witness for the Reader effect."
)]
#[document_parameters("The projection to apply to the environment.")]
#[document_returns(
	"An `ArcRunExplicit` program that asks for the environment and returns `f(env)`."
)]
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
/// let program: ArcRunExplicit<'static, Row, CNilBrand, String> =
/// 	ArcRunExplicit::asks::<i32, _>(|env| format!("env={env}"));
/// let handled: ArcRunExplicit<'static, CNilBrand, CNilBrand, String> =
/// 	program.run_reader::<i32, _, CNilBrand>(7);
/// assert_eq!(handled.extract(), "env=7");
/// ```
#[inline]
pub fn asks<E, Idx>(f: impl Fn(E) -> A + Send + Sync + 'a) -> Self
where
	A: Clone,
	E: Clone + Send + Sync + 'static,
	NodeBrand<R, S>: SendFunctor,
	Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, E>):
		Member<ArcCoyoneda<'a, SendReaderBrand<ArcBrand, E>, E>, Idx>,
	Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
		'a,
		ArcFreeExplicit<'a, NodeBrand<R, S>, E>,
	>): Send + Sync,
	Apply!(<S as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
		'a,
		ArcFreeExplicit<'a, NodeBrand<R, S>, E>,
	>): Send + Sync,
	Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
		'a,
		ArcFreeExplicit<'a, NodeBrand<R, S>, E>,
	>): Clone + Send + Sync,
	Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
		'a,
		ArcFreeExplicit<'a, NodeBrand<R, S>, A>,
	>): Clone + Send + Sync, {
	ArcRunExplicit::<'a, R, S, E>::ask::<Idx>().map(f)
}
