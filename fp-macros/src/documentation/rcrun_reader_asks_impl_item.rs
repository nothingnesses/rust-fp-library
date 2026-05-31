/// Reads the Reader environment and maps it immediately.
///
/// `asks(f)` is the Reader-specific convenience form of
/// `ask().map(f)` for `RcRun`.
#[__document_module_generated]
#[document_signature]
#[document_type_parameters(
	"The Reader environment type.",
	"The type-level Member-position witness for the Reader effect."
)]
#[document_parameters("The projection to apply to the environment.")]
#[document_returns("An `RcRun` program that asks for the environment and returns `f(env)`.")]
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
/// let program: RcRun<Row, CNilBrand, String> = RcRun::asks::<i32, _>(|env| format!("env={env}"));
/// let handled: RcRun<CNilBrand, CNilBrand, String> = program.run_reader::<i32, _, CNilBrand>(7);
/// assert_eq!(handled.extract(), "env=7");
/// ```
#[inline]
pub fn asks<E, Idx>(f: impl Fn(E) -> A + 'static) -> Self
where
	E: Clone + 'static,
	Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, E>):
		Member<RcCoyoneda<'static, ReaderBrand<RcBrand, E>, E>, Idx>,
	Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
		'static,
		RcFree<NodeBrand<R, S>, RcTypeErasedValue>,
	>): Clone, {
	RcRun::<R, S, E>::ask::<Idx>().map(f)
}
