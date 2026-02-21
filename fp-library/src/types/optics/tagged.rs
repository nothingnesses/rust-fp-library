//! The `Tagged` profunctor, used for reviews.
//!
//! `Tagged<A, B>` simply wraps a value of type `B`, ignoring the `A` parameter.

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
	fp_macros::document_type_parameters,
	std::marker::PhantomData,
};

/// The `Tagged` profunctor.
///
/// `Tagged<A, B>` is a profunctor that ignores its first type argument `A`
/// and instead stores a value of type `B`.
#[document_type_parameters("The lifetime of the values.", "The ignored type.", "The value type.")]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Tagged<'a, A, B>(pub B, pub PhantomData<&'a A>);

impl<'a, A, B> Tagged<'a, A, B> {
	/// Creates a new `Tagged` instance.
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
		type Of<'a, A: 'a, B: 'a>: 'a = Tagged<'a, A, B>;
	}
}

impl Profunctor for TaggedBrand {
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
	fn left<'a, A: 'a, B: 'a, C: 'a>(
		pab: Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, A, B>)
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, Result<C, A>, Result<C, B>>)
	{
		Tagged::new(Err(pab.0))
	}

	fn right<'a, A: 'a, B: 'a, C: 'a>(
		pab: Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, A, B>)
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, Result<A, C>, Result<B, C>>)
	{
		Tagged::new(Ok(pab.0))
	}
}
