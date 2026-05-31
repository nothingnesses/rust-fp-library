/// Reads the Reader environment and maps it immediately.
#[__document_module_generated]
#[document_signature]
#[document_type_parameters(
	"The Reader environment type.",
	"The type-level Member-position witness for the Reader effect."
)]
#[document_parameters("The projection to apply to the environment.")]
#[document_returns(
	"An `RcRunExplicit` program that asks for the environment and returns `f(env)`."
)]
#[document_examples]
///
/// ```
/// use fp_library::{
/// 	brands::*,
/// 	types::effects::rc_run_explicit::RcRunExplicit,
/// };
///
/// type Row = CoproductBrand<RcCoyonedaBrand<ReaderBrand<RcBrand, i32>>, CNilBrand>;
///
/// let program: RcRunExplicit<'static, Row, CNilBrand, String> =
/// 	RcRunExplicit::asks::<i32, _>(|env| format!("env={env}"));
/// let handled: RcRunExplicit<'static, CNilBrand, CNilBrand, String> =
/// 	program.run_reader::<i32, _, CNilBrand>(7);
/// assert_eq!(handled.extract(), "env=7");
/// ```
#[inline]
pub fn asks<E, Idx>(f: impl Fn(E) -> A + 'a) -> Self
where
	E: Clone + 'static,
	Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, E>):
		Member<RcCoyoneda<'a, ReaderBrand<RcBrand, E>, E>, Idx>,
	Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
		'a,
		RcFreeExplicit<'a, NodeBrand<R, S>, E>,
	>): Clone,
	Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
		'a,
		RcFreeExplicit<'a, NodeBrand<R, S>, A>,
	>): Clone, {
	RcRunExplicit::<'a, R, S, E>::ask::<Idx>().map(f)
}
