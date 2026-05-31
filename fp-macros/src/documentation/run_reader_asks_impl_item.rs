/// Reads the Reader environment and maps it immediately.
///
/// `asks(f)` is the Reader-specific convenience form of
/// `ask().map(f)`. It keeps the effect row unchanged while
/// letting the program return a projection of the environment.
#[__document_module_generated]
#[document_signature]
#[document_type_parameters(
	"The Reader environment type.",
	"The type-level Member-position witness for the Reader effect."
)]
#[document_parameters("The projection to apply to the environment.")]
#[document_returns("A `Run` program that asks for the environment and returns `f(env)`.")]
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
/// let program: Run<Row, CNilBrand, String> = Run::asks::<i32, _>(|env| format!("env={env}"));
/// let handled: Run<CNilBrand, CNilBrand, String> = program.run_reader::<i32, _, CNilBrand>(7);
/// assert_eq!(handled.extract(), "env=7");
/// ```
#[inline]
pub fn asks<E, Idx>(f: impl FnOnce(E) -> A + 'static) -> Self
where
	E: 'static,
	Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, E>):
		Member<Coyoneda<'static, BoxReaderBrand<BoxBrand, E>, E>, Idx>, {
	Run::<R, S, E>::ask::<Idx>().map(f)
}
