//! The `Tagged` profunctor, used for reviews.
//!
//! `Tagged<A, B>` simply wraps a value of type `B`, ignoring the `A` parameter.

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			Apply,
			classes::profunctor::{
				Choice,
				Cochoice,
				Costrong,
				Profunctor,
			},
			impl_kind,
			kinds::*,
		},
		fp_macros::{
			document_parameters,
			document_type_parameters,
			document_return,
		},
		std::marker::PhantomData,
	};

	/// The `Tagged` profunctor.
	///
	/// `Tagged<A, B>` is a profunctor that ignores its first type argument `A`
	/// and instead stores a value of type `B`.
	#[document_type_parameters(
		"The lifetime of the values.",
		"The ignored type.",
		"The value type."
	)]
	#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
	pub struct Tagged<'a, A, B>(pub B, pub PhantomData<&'a A>);

	#[document_type_parameters(
		"The lifetime of the values.",
		"The ignored type.",
		"The value type."
	)]
	impl<'a, A, B> Tagged<'a, A, B> {
		/// Creates a new `Tagged` instance.
		#[document_signature]
		///
		#[document_parameters("The value to wrap.")]
		///
		#[document_return("A new instance of the type.")]
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::types::optics::Tagged;
		///
		/// let tagged = Tagged::<String, i32>::new(123);
		/// assert_eq!(tagged.0, 123);
		/// ```
		pub fn new(b: B) -> Self {
			Tagged(b, PhantomData)
		}
	}

	/// Brand for the `Tagged` profunctor.
	pub struct TaggedBrand;

	impl_kind! {
		impl for TaggedBrand {
			#[document_default]
			type Of<'a, A: 'a, B: 'a>: 'a = Tagged<'a, A, B>;
		}
	}

	impl Profunctor for TaggedBrand {
		/// Maps functions over the input and output of the `Tagged` profunctor.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the functions.",
			"The source type of the new structure.",
			"The target type of the new structure.",
			"The source type of the original structure.",
			"The target type of the original structure.",
			"The type of the function to apply to the input.",
			"The type of the function to apply to the output."
		)]
		///
		#[document_parameters(
			"The function to apply to the input.",
			"The function to apply to the output.",
			"The tagged instance to transform."
		)]
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::{
		/// 		optics::*,
		/// 		profunctor::*,
		/// 		*,
		/// 	},
		/// 	types::optics::*,
		/// };
		///
		/// let tagged: Tagged<String, usize> = Tagged::new(123);
		/// let transformed = <TaggedBrand as Profunctor>::dimap(
		/// 	|s: &str| s.to_string(),
		/// 	|n: usize| n.to_string(),
		/// 	tagged,
		/// );
		/// ```
		fn dimap<'a, A: 'a, B: 'a, C: 'a, D: 'a, FuncAB, FuncCD>(
			_ab: FuncAB,
			cd: FuncCD,
			pbc: Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, B, C>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, A, D>)
		where
			FuncAB: Fn(A) -> B + 'a,
			FuncCD: Fn(C) -> D + 'a, {
			Tagged::new(cd(pbc.0))
		}
	}

	impl Choice for TaggedBrand {
		/// Lifts the `Tagged` profunctor to operate on the left component of a `Result`.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the functions.",
			"The type of the left component.",
			"The type of the target left component.",
			"The type of the right component."
		)]
		///
		#[document_parameters("The tagged instance to transform.")]
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::{
		/// 		optics::*,
		/// 		profunctor::*,
		/// 		*,
		/// 	},
		/// 	types::optics::*,
		/// };
		///
		/// let tagged: Tagged<usize, usize> = Tagged::new(123);
		/// let transformed = <TaggedBrand as Choice>::left::<usize, usize, String>(tagged);
		/// assert_eq!(transformed.0, Err(123));
		/// ```
		fn left<'a, A: 'a, B: 'a, C: 'a>(
			pab: Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, A, B>)
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, Result<C, A>, Result<C, B>>)
		{
			Tagged::new(Err(pab.0))
		}

		/// Lifts the `Tagged` profunctor to operate on the right component of a `Result`.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the functions.",
			"The type of the left component.",
			"The type of the right component.",
			"The target type of the right component."
		)]
		///
		#[document_parameters("The tagged instance to transform.")]
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::{
		/// 		optics::*,
		/// 		profunctor::*,
		/// 		*,
		/// 	},
		/// 	types::optics::*,
		/// };
		///
		/// let tagged: Tagged<usize, usize> = Tagged::new(123);
		/// let transformed = <TaggedBrand as Choice>::right::<usize, usize, String>(tagged);
		/// assert_eq!(transformed.0, Ok(123));
		/// ```
		fn right<'a, A: 'a, B: 'a, C: 'a>(
			pab: Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, A, B>)
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, Result<A, C>, Result<B, C>>)
		{
			Tagged::new(Ok(pab.0))
		}
	}

	impl Cochoice for TaggedBrand {
		/// Extracts the `Tagged` value from one operating on the left (Err) component of a `Result`.
		///
		/// Assuming the inner `Result<C, B>` is `Err(b)` (as produced by [`Choice::left`]),
		/// this unwraps the `B` value.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The input type of the resulting tagged.",
			"The value type of the resulting tagged.",
			"The type of the alternative (Ok) variant."
		)]
		///
		#[document_parameters("The tagged instance to extract from.")]
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	classes::profunctor::*,
		/// 	types::optics::*,
		/// };
		///
		/// let tagged: Tagged<Result<String, i32>, Result<String, i32>> = Tagged::new(Err(42));
		/// let result = <TaggedBrand as Cochoice>::unleft::<i32, i32, String>(tagged);
		/// assert_eq!(result.0, 42);
		/// ```
		fn unleft<'a, A: 'a, B: 'a, C: 'a>(
			pab: Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, Result<C, A>, Result<C, B>>)
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, A, B>) {
			match pab.0 {
				Err(b) => Tagged::new(b),
				Ok(_) => unreachable!("Cochoice::unleft on Tagged: value was Ok, expected Err"),
			}
		}

		/// Extracts the `Tagged` value from one operating on the right (Ok) component of a `Result`.
		///
		/// Assuming the inner `Result<B, C>` is `Ok(b)` (as produced by [`Choice::right`]),
		/// this unwraps the `B` value.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The input type of the resulting tagged.",
			"The value type of the resulting tagged.",
			"The type of the alternative (Err) variant."
		)]
		///
		#[document_parameters("The tagged instance to extract from.")]
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	classes::profunctor::*,
		/// 	types::optics::*,
		/// };
		///
		/// let tagged: Tagged<Result<i32, String>, Result<i32, String>> = Tagged::new(Ok(42));
		/// let result = <TaggedBrand as Cochoice>::unright::<i32, i32, String>(tagged);
		/// assert_eq!(result.0, 42);
		/// ```
		fn unright<'a, A: 'a, B: 'a, C: 'a>(
			pab: Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, Result<A, C>, Result<B, C>>)
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, A, B>) {
			match pab.0 {
				Ok(b) => Tagged::new(b),
				Err(_) => unreachable!("Cochoice::unright on Tagged: value was Err, expected Ok"),
			}
		}
	}

	impl Costrong for TaggedBrand {
		/// Extracts the `Tagged` value from one operating on the first component of a pair.
		///
		/// Since `Tagged` ignores its first type argument, this simply extracts the
		/// first element `B` from the stored `(B, C)` pair.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The input type of the resulting tagged.",
			"The value type of the resulting tagged.",
			"The type of the second component (discarded)."
		)]
		///
		#[document_parameters("The tagged instance to extract from.")]
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	classes::profunctor::*,
		/// 	types::optics::*,
		/// };
		///
		/// let tagged: Tagged<(i32, String), (i32, String)> = Tagged::new((42, "hello".to_string()));
		/// let result = <TaggedBrand as Costrong>::unfirst::<i32, i32, String>(tagged);
		/// assert_eq!(result.0, 42);
		/// ```
		fn unfirst<'a, A: 'a, B: 'a, C: 'a>(
			pab: Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, (A, C), (B, C)>)
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, A, B>) {
			Tagged::new(pab.0.0)
		}

		/// Extracts the `Tagged` value from one operating on the second component of a pair.
		///
		/// Since `Tagged` ignores its first type argument, this simply extracts the
		/// second element `B` from the stored `(C, B)` pair.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The input type of the resulting tagged.",
			"The value type of the resulting tagged.",
			"The type of the first component (discarded)."
		)]
		///
		#[document_parameters("The tagged instance to extract from.")]
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	classes::profunctor::*,
		/// 	types::optics::*,
		/// };
		///
		/// let tagged: Tagged<(String, i32), (String, i32)> = Tagged::new(("hello".to_string(), 42));
		/// let result = <TaggedBrand as Costrong>::unsecond::<i32, i32, String>(tagged);
		/// assert_eq!(result.0, 42);
		/// ```
		fn unsecond<'a, A: 'a, B: 'a, C: 'a>(
			pab: Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, (C, A), (C, B)>)
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, A, B>) {
			Tagged::new(pab.0.1)
		}
	}
}
pub use inner::*;
