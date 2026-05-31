/// Reads the Reader environment and maps it immediately.
#[__document_module_generated]
#[document_signature]
#[document_type_parameters(
	"The Reader environment type.",
	"The type-level Member-position witness for the Reader effect."
)]
#[document_parameters("The projection to apply to the environment.")]
#[document_returns("An `ArcRun` program that asks for the environment and returns `f(env)`.")]
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
/// let program: ArcRun<Row, CNilBrand, String> =
/// 	ArcRun::asks::<i32, _>(|env| format!("env={env}"));
/// let handled: ArcRun<CNilBrand, CNilBrand, String> = program.run_reader::<i32, _, CNilBrand>(7);
/// assert_eq!(handled.extract(), "env=7");
/// ```
#[inline]
pub fn asks<E, Idx>(f: impl Fn(E) -> A + Send + Sync + 'static) -> Self
where
	E: Clone + Send + Sync + 'static,
	A: Clone,
	NodeBrand<R, S>: SendFunctor,
	Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, E>):
		Member<ArcCoyoneda<'static, SendReaderBrand<ArcBrand, E>, E>, Idx>,
	Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
		'static,
		ArcFree<NodeBrand<R, S>, ArcTypeErasedValue>,
	>): Clone, {
	ArcRun::<R, S, E>::ask::<Idx>().map(f)
}
