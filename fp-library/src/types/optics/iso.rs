//! Isomorphism optics for bidirectional conversions.

use {
	super::base::Optic,
	crate::{
		Apply,
		brands::FnBrand,
		classes::{
			CloneableFn,
			Profunctor,
			UnsizedCoercible,
		},
		kinds::*,
	},
	fp_macros::{
		document_parameters,
		document_signature,
		document_type_parameters,
	},
	std::marker::PhantomData,
};

/// A polymorphic isomorphism where types can change.
/// This matches PureScript's `Iso s t a b`.
///
/// An isomorphism represents a lossless bidirectional conversion between types.
/// Uses [`FnBrand`](crate::brands::FnBrand) to support capturing closures.
#[document_type_parameters(
	"The lifetime of the values.",
	"The reference-counted pointer type.",
	"The source type of the structure.",
	"The target type of the structure after an update.",
	"The source type of the focus.",
	"The target type of the focus after an update."
)]
pub struct Iso<'a, P, S, T, A, B>
where
	P: UnsizedCoercible,
	S: 'a,
	T: 'a,
	A: 'a,
	B: 'a, {
	/// Forward conversion: from structure to focus.
	pub from: Apply!(<FnBrand<P> as Kind!( type Of<'b, U: 'b, V: 'b>: 'b; )>::Of<'a, S, A>),
	/// Backward conversion: from focus to structure.
	pub to: Apply!(<FnBrand<P> as Kind!( type Of<'b, U: 'b, V: 'b>: 'b; )>::Of<'a, B, T>),
	pub(crate) _phantom: PhantomData<P>,
}

#[document_type_parameters(
	"The lifetime of the values.",
	"The reference-counted pointer type.",
	"The source type of the structure.",
	"The target type of the structure after an update.",
	"The source type of the focus.",
	"The target type of the focus after an update."
)]
#[document_parameters("The iso instance.")]
impl<'a, P, S: 'a, T: 'a, A: 'a, B: 'a> Iso<'a, P, S, T, A, B>
where
	P: UnsizedCoercible,
{
	/// Create a new polymorphic isomorphism.
	#[document_signature]
	///
	#[document_parameters("The forward conversion function.", "The backward conversion function.")]
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{
	/// 	brands::RcBrand,
	/// 	types::optics::Iso,
	/// };
	///
	/// // Iso between String and Vec<char>
	/// let string_chars: Iso<RcBrand, String, String, Vec<char>, Vec<char>> =
	/// 	Iso::new(|s: String| s.chars().collect(), |v: Vec<char>| v.into_iter().collect());
	/// ```
	pub fn new(
		from: impl 'a + Fn(S) -> A,
		to: impl 'a + Fn(B) -> T,
	) -> Self {
		Iso {
			from: <FnBrand<P> as CloneableFn>::new(from),
			to: <FnBrand<P> as CloneableFn>::new(to),
			_phantom: PhantomData,
		}
	}

	/// Apply the forward conversion.
	#[document_signature]
	///
	#[document_parameters("The structure to convert.")]
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{
	/// 	brands::RcBrand,
	/// 	types::optics::Iso,
	/// };
	///
	/// let string_chars: Iso<RcBrand, String, String, Vec<char>, Vec<char>> =
	/// 	Iso::new(|s: String| s.chars().collect(), |v: Vec<char>| v.into_iter().collect());
	/// let chars = string_chars.from("hello".to_string());
	/// assert_eq!(chars, vec!['h', 'e', 'l', 'l', 'o']);
	/// ```
	pub fn from(
		&self,
		s: S,
	) -> A {
		(self.from)(s)
	}

	/// Apply the backward conversion.
	#[document_signature]
	///
	#[document_parameters("The focus value to convert.")]
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{
	/// 	brands::RcBrand,
	/// 	types::optics::Iso,
	/// };
	///
	/// let string_chars: Iso<RcBrand, String, String, Vec<char>, Vec<char>> =
	/// 	Iso::new(|s: String| s.chars().collect(), |v: Vec<char>| v.into_iter().collect());
	/// let s = string_chars.to(vec!['h', 'i']);
	/// assert_eq!(s, "hi");
	/// ```
	pub fn to(
		&self,
		b: B,
	) -> T {
		(self.to)(b)
	}
}

#[document_type_parameters(
	"The lifetime of the values.",
	"The profunctor type.",
	"The reference-counted pointer type.",
	"The source type of the structure.",
	"The target type of the structure after an update.",
	"The source type of the focus.",
	"The target type of the focus after an update."
)]
#[document_parameters("The iso instance.")]
impl<'a, Q, P, S, T, A, B> Optic<'a, Q, S, T, A, B> for Iso<'a, P, S, T, A, B>
where
	Q: Profunctor,
	P: UnsizedCoercible,
	S: 'a,
	T: 'a,
	A: 'a,
	B: 'a,
{
	#[document_signature]
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
	/// let iso: Iso<RcBrand, (i32,), (i32,), i32, i32> = Iso::new(|(x,)| x, |x| (x,));
	/// let f = cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x + 1);
	/// let modifier = Optic::<RcFnBrand, _, _, _, _>::evaluate(&iso, f);
	/// assert_eq!(modifier((41,)), (42,));
	/// ```
	fn evaluate(
		&self,
		pab: Apply!(<Q as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, A, B>),
	) -> Apply!(<Q as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, S, T>) {
		let from = self.from.clone();
		let to = self.to.clone();

		// The Profunctor encoding of an Iso is:
		// iso from to = dimap from to
		Q::dimap(move |s| from(s), move |b| to(b), pab)
	}
}

/// A concrete isomorphism type where types do not change.
/// This matches PureScript's `Iso' s a`.
///
/// Uses [`FnBrand`](crate::brands::FnBrand) to support capturing closures.
#[document_type_parameters(
	"The lifetime of the values.",
	"The reference-counted pointer type.",
	"The type of the structure.",
	"The type of the focus."
)]
pub struct IsoPrime<'a, P, S, A>
where
	P: UnsizedCoercible,
	S: 'a,
	A: 'a, {
	pub(crate) from_fn: Apply!(<FnBrand<P> as Kind!( type Of<'b, U: 'b, V: 'b>: 'b; )>::Of<'a, S, A>),
	pub(crate) to_fn: Apply!(<FnBrand<P> as Kind!( type Of<'b, U: 'b, V: 'b>: 'b; )>::Of<'a, A, S>),
	pub(crate) _phantom: PhantomData<P>,
}

#[document_type_parameters(
	"The lifetime of the values.",
	"The reference-counted pointer type.",
	"The type of the structure.",
	"The type of the focus."
)]
#[document_parameters("The iso instance.")]
impl<'a, P, S, A> Clone for IsoPrime<'a, P, S, A>
where
	P: UnsizedCoercible,
	S: 'a,
	A: 'a,
{
	#[document_signature]
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{
	/// 	brands::RcBrand,
	/// 	types::optics::IsoPrime,
	/// };
	///
	/// let iso: IsoPrime<RcBrand, (i32,), i32> = IsoPrime::new(|(x,)| x, |x| (x,));
	/// let cloned = iso.clone();
	/// ```
	fn clone(&self) -> Self {
		IsoPrime {
			from_fn: self.from_fn.clone(),
			to_fn: self.to_fn.clone(),
			_phantom: PhantomData,
		}
	}
}

#[document_type_parameters(
	"The lifetime of the values.",
	"The reference-counted pointer type.",
	"The type of the structure.",
	"The type of the focus."
)]
#[document_parameters("The monomorphic iso instance.")]
impl<'a, P, S: 'a, A: 'a> IsoPrime<'a, P, S, A>
where
	P: UnsizedCoercible,
{
	/// Create a new monomorphic isomorphism from bidirectional conversion functions.
	#[document_signature]
	///
	#[document_parameters("The forward conversion function.", "The backward conversion function.")]
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{
	/// 	brands::RcBrand,
	/// 	types::optics::IsoPrime,
	/// };
	///
	/// // Iso between a newtype and its inner value
	/// let iso: IsoPrime<RcBrand, (i32,), i32> = IsoPrime::new(|(x,)| x, |x| (x,));
	/// ```
	pub fn new(
		from: impl 'a + Fn(S) -> A,
		to: impl 'a + Fn(A) -> S,
	) -> Self {
		IsoPrime {
			from_fn: <FnBrand<P> as CloneableFn>::new(from),
			to_fn: <FnBrand<P> as CloneableFn>::new(to),
			_phantom: PhantomData,
		}
	}

	/// Apply the forward conversion.
	#[document_signature]
	///
	#[document_parameters("The structure to convert.")]
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{
	/// 	brands::RcBrand,
	/// 	types::optics::IsoPrime,
	/// };
	///
	/// let iso: IsoPrime<RcBrand, (i32,), i32> = IsoPrime::new(|(x,)| x, |x| (x,));
	/// assert_eq!(iso.from((42,)), 42);
	/// ```
	pub fn from(
		&self,
		s: S,
	) -> A {
		(self.from_fn)(s)
	}

	/// Apply the backward conversion.
	#[document_signature]
	///
	#[document_parameters("The focus value to convert.")]
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{
	/// 	brands::RcBrand,
	/// 	types::optics::IsoPrime,
	/// };
	///
	/// let iso: IsoPrime<RcBrand, (i32,), i32> = IsoPrime::new(|(x,)| x, |x| (x,));
	/// assert_eq!(iso.to(42), (42,));
	/// ```
	pub fn to(
		&self,
		a: A,
	) -> S {
		(self.to_fn)(a)
	}

	/// Reverse the isomorphism.
	#[document_signature]
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{
	/// 	brands::RcBrand,
	/// 	types::optics::IsoPrime,
	/// };
	///
	/// let iso: IsoPrime<RcBrand, (i32,), i32> = IsoPrime::new(|(x,)| x, |x| (x,));
	/// let reversed = iso.reversed();
	/// assert_eq!(reversed.from(42), (42,));
	/// assert_eq!(reversed.to((42,)), 42);
	/// ```
	pub fn reversed(&self) -> IsoPrime<'a, P, A, S> {
		IsoPrime {
			from_fn: self.to_fn.clone(),
			to_fn: self.from_fn.clone(),
			_phantom: PhantomData,
		}
	}
}

// Optic implementation for IsoPrime<P, S, A>
#[document_type_parameters(
	"The lifetime of the values.",
	"The profunctor type.",
	"The reference-counted pointer type.",
	"The type of the structure.",
	"The type of the focus."
)]
#[document_parameters("The monomorphic iso instance.")]
impl<'a, Q, P, S, A> Optic<'a, Q, S, S, A, A> for IsoPrime<'a, P, S, A>
where
	Q: Profunctor,
	P: UnsizedCoercible,
	S: 'a,
	A: 'a,
{
	#[document_signature]
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
	/// let iso: IsoPrime<RcBrand, (i32,), i32> = IsoPrime::new(|(x,)| x, |x| (x,));
	/// let f = cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x + 1);
	/// let modifier = Optic::<RcFnBrand, _, _, _, _>::evaluate(&iso, f);
	/// assert_eq!(modifier((41,)), (42,));
	/// ```
	fn evaluate(
		&self,
		pab: Apply!(<Q as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, A, A>),
	) -> Apply!(<Q as Kind!( type Of<'b, T: 'b, U: 'b>: 'b; )>::Of<'a, S, S>) {
		let from_fn = self.from_fn.clone();
		let to_fn = self.to_fn.clone();

		// The Profunctor encoding of an Iso is:
		// iso from to = dimap from to
		Q::dimap(move |s| from_fn(s), move |a| to_fn(a), pab)
	}
}
