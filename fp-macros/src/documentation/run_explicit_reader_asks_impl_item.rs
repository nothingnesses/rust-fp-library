/// Reads the Reader environment and maps it immediately.
#[__document_module_generated]
#[document_signature]
#[document_type_parameters(
	"The Reader environment type.",
	"The type-level Member-position witness for the Reader effect."
)]
#[document_parameters("The projection to apply to the environment.")]
#[document_returns("A `RunExplicit` program that asks for the environment and returns `f(env)`.")]
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
/// let program: RunExplicit<'static, Row, CNilBrand, String> =
/// 	RunExplicit::asks::<i32, _>(|env| format!("env={env}"));
/// let handled: RunExplicit<'static, CNilBrand, CNilBrand, String> =
/// 	program.run_reader::<i32, _, CNilBrand>(7);
/// assert_eq!(handled.extract(), "env=7");
/// ```
#[inline]
pub fn asks<E, Idx>(f: impl Fn(E) -> A + 'a) -> Self
where
	E: 'static,
	Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, E>):
		Member<Coyoneda<'a, BoxReaderBrand<BoxBrand, E>, E>, Idx>, {
	RunExplicit::<'a, R, S, E>::ask::<Idx>().map(f)
}
