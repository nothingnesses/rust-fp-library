//! Core optic trait and composition.

use {
	crate::{
		Apply,
		classes::Profunctor,
		kinds::*,
	},
	fp_macros::{
		document_parameters,
		document_signature,
		document_type_parameters,
	},
	std::marker::PhantomData,
};

/// A trait for optics that can be evaluated with any profunctor constraint.
///
/// This trait allows optics to be first-class values that can be composed
/// and stored while preserving their polymorphism over profunctor types.
#[document_type_parameters(
	"The lifetime of the values.",
	"The profunctor type.",
	"The source type of the structure.",
	"The target type of the structure after an update.",
	"The source type of the focus.",
	"The target type of the focus after an update."
)]
pub trait Optic<'a, P: Profunctor, S: 'a, T: 'a, A: 'a, B: 'a> {
	/// Evaluate the optic with a profunctor.
	///
	/// This method applies the optic transformation to a profunctor value.
	#[document_signature]
	///
	#[document_parameters("The profunctor value to transform.")]
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// 	types::optics::*,
	/// };
	///
	/// let l: LensPrime<RcBrand, (i32, String), i32> =
	/// 	LensPrime::new(|(x, _)| x, |((_, s), x)| (x, s));
	///
	/// let f = cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2);
	/// let modifier = Optic::<RcFnBrand, _, _, _, _>::evaluate(&l, f);
	/// assert_eq!(modifier((21, "hello".to_string())), (42, "hello".to_string()));
	/// ```
	fn evaluate(
		&self,
		pab: Apply!(<P as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, A, B>),
	) -> Apply!(<P as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, S, T>);
}

/// Composition of two optics.
///
/// This struct represents the composition of two optics, allowing them to be
/// combined into a single optic that applies both transformations.
#[document_type_parameters(
	"The lifetime of the values.",
	"The source type of the outer structure.",
	"The target type of the outer structure.",
	"The source type of the intermediate structure.",
	"The target type of the intermediate structure.",
	"The source type of the focus.",
	"The target type of the focus.",
	"The first optic.",
	"The second optic."
)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Composed<'a, S, T, M, N, A, B, O1, O2> {
	/// The outer optic (applied second).
	pub first: O1,
	/// The inner optic (applied first).
	pub second: O2,
	pub(crate) _phantom: PhantomData<&'a (S, T, M, N, A, B)>,
}

#[document_type_parameters(
	"The lifetime of the values.",
	"The source type of the outer structure.",
	"The target type of the outer structure.",
	"The source type of the intermediate structure.",
	"The target type of the intermediate structure.",
	"The source type of the focus.",
	"The target type of the focus.",
	"The first optic.",
	"The second optic."
)]
impl<'a, S, T, M, N, A, B, O1, O2> Composed<'a, S, T, M, N, A, B, O1, O2> {
	/// Create a new composed optic.
	#[document_signature]
	///
	#[document_parameters("The outer optic (applied second).", "The inner optic (applied first).")]
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// 	types::optics::*,
	/// };
	///
	/// let l1: LensPrime<RcBrand, (i32, String), i32> =
	/// 	LensPrime::new(|(x, _): (i32, String)| x, |((_, s), x)| (x, s));
	/// let l2: LensPrime<RcBrand, i32, i32> = LensPrime::new(|x: i32| x, |(_, x)| x);
	/// let composed = Composed::new(l1, l2);
	///
	/// let f = cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2);
	/// let modifier = Optic::<RcFnBrand, _, _, _, _>::evaluate(&composed, f);
	/// assert_eq!(modifier((21, "hi".to_string())), (42, "hi".to_string()));
	/// ```
	pub fn new(
		first: O1,
		second: O2,
	) -> Self {
		Composed {
			first,
			second,
			_phantom: PhantomData,
		}
	}
}

#[document_type_parameters(
	"The lifetime of the values.",
	"The profunctor type.",
	"The source type of the outer structure.",
	"The target type of the outer structure.",
	"The source type of the intermediate structure.",
	"The target type of the intermediate structure.",
	"The source type of the focus.",
	"The target type of the focus.",
	"The first optic.",
	"The second optic."
)]
#[document_parameters("The composed optic instance.")]
impl<'a, P, S: 'a, T: 'a, M: 'a, N: 'a, A: 'a, B: 'a, O1, O2> Optic<'a, P, S, T, A, B>
	for Composed<'a, S, T, M, N, A, B, O1, O2>
where
	P: Profunctor,
	O1: Optic<'a, P, S, T, M, N>,
	O2: Optic<'a, P, M, N, A, B>,
{
	#[document_signature]
	#[document_parameters("The profunctor value to transform.")]
	fn evaluate(
		&self,
		pab: Apply!(<P as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, A, B>),
	) -> Apply!(<P as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, S, T>) {
		let pmn = self.second.evaluate(pab);
		self.first.evaluate(pmn)
	}
}

/// Compose two optics into a single optic.
///
/// While PureScript uses the `Semigroupoid` operator (`<<<`) for composition because
/// its optics are functions, this library uses a specialized `Composed` struct.
/// This is necessary because Rust represents the polymorphic profunctor constraint
/// as a parameterized trait (`Optic<'a, P, ...>`), and the `Composed` struct enables
/// static dispatch and zero-cost composition through monomorphization.
#[document_signature]
///
#[document_type_parameters(
	"The lifetime of the values.",
	"The source type of the outer structure.",
	"The target type of the outer structure.",
	"The source type of the intermediate structure.",
	"The target type of the intermediate structure.",
	"The source type of the focus.",
	"The target type of the focus.",
	"The first optic.",
	"The second optic."
)]
///
#[document_parameters("The outer optic (applied second).", "The inner optic (applied first).")]
///
/// ### Examples
///
/// ```
/// use fp_library::{
/// 	brands::*,
/// 	functions::*,
/// 	types::optics::*,
/// };
///
/// #[derive(Clone, Debug, PartialEq)]
/// struct Address {
/// 	street: String,
/// }
/// #[derive(Clone, Debug, PartialEq)]
/// struct User {
/// 	address: Address,
/// }
///
/// let address_lens: LensPrime<RcBrand, User, Address> = LensPrime::new(
/// 	|u: User| u.address.clone(),
/// 	|(_, a)| User {
/// 		address: a,
/// 	},
/// );
/// let street_lens: LensPrime<RcBrand, Address, String> = LensPrime::new(
/// 	|a: Address| a.street.clone(),
/// 	|(_, s)| Address {
/// 		street: s,
/// 	},
/// );
///
/// let user_street = optics_compose(address_lens, street_lens);
/// let user = User {
/// 	address: Address {
/// 		street: "High St".to_string(),
/// 	},
/// };
///
/// let f = cloneable_fn_new::<RcFnBrand, _, _>(|s: String| s.to_uppercase());
/// let modifier = Optic::<RcFnBrand, _, _, _, _>::evaluate(&user_street, f);
/// let updated = modifier(user);
///
/// assert_eq!(updated.address.street, "HIGH ST");
/// ```
pub fn optics_compose<'a, S: 'a, T: 'a, M: 'a, N: 'a, A: 'a, B: 'a, O1, O2>(
	first: O1,
	second: O2,
) -> Composed<'a, S, T, M, N, A, B, O1, O2> {
	Composed::new(first, second)
}
