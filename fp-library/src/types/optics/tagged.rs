//! The `Tagged` profunctor, used for reviews.
//!
//! `Tagged<A, B>` simply wraps a value of type `B`, ignoring the `A` parameter.

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			Apply,
			classes::{
				Choice,
				Profunctor,
			},
			impl_kind,
			kinds::*,
		},
		fp_macros::{
			document_parameters,
			document_type_parameters,
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
		/// 	classes::*,
		/// 	types::optics::*,
		/// };
		///
		/// let tagged: Tagged<String, usize> = Tagged::new(123);
		/// let transformed = Profunctor::dimap(|s: &str| s.to_string(), |n: usize| n.to_string(), tagged);
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
		/// 	classes::*,
		/// 	types::optics::*,
		/// };
		///
		/// let tagged: Tagged<String, usize> = Tagged::new(123);
		/// let transformed = Choice::left::<usize, usize, String>(tagged);
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
		/// 	classes::*,
		/// 	types::optics::*,
		/// };
		///
		/// let tagged: Tagged<String, usize> = Tagged::new(123);
		/// let transformed = Choice::right::<usize, usize, String>(tagged);
		/// ```
		fn right<'a, A: 'a, B: 'a, C: 'a>(
			pab: Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, A, B>)
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, Result<A, C>, Result<B, C>>)
		{
			Tagged::new(Ok(pab.0))
		}
	}
}
pub use inner::*;
