//! Dispatch for [`join`](crate::classes::semimonad::join) and
//! [`ref_join`](crate::classes::ref_semimonad::ref_join).
//!
//! Provides the [`JoinDispatch`] trait and a unified [`join`] free function
//! that routes to the appropriate trait method based on whether the container
//! is owned or borrowed.
//!
//! ### Examples
//!
//! ```
//! use fp_library::{
//! 	brands::*,
//! 	functions::*,
//! };
//!
//! // Owned: dispatches via Semimonad::bind(id)
//! let y = join_explicit::<OptionBrand, _, _, _>(Some(Some(5)));
//! assert_eq!(y, Some(5));
//! ```

pub(crate) mod inner {
	use {
		crate::{
			classes::{
				RefSemimonad,
				Semimonad,
			},
			dispatch::{
				Ref,
				Val,
			},
			kinds::*,
		},
		fp_macros::*,
	};

	/// Trait that routes a join operation to the appropriate type class method.
	///
	/// The `Marker` type parameter is an implementation detail resolved by
	/// the compiler from the container type; callers never specify it directly.
	/// Owned containers resolve to [`Val`], borrowed containers resolve to [`Ref`].
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the monad.",
		"The type of the value(s) inside the inner layer.",
		"Dispatch marker type, inferred automatically. Either [`Val`](crate::dispatch::Val) or [`Ref`](crate::dispatch::Ref)."
	)]
	#[document_parameters("The container implementing this dispatch.")]
	pub trait JoinDispatch<'a, Brand: Kind_cdc7cd43dac7585f, A: 'a, Marker> {
		/// Perform the dispatched join operation.
		#[document_signature]
		///
		#[document_returns("A container with one layer of nesting removed.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// let result = join_explicit::<OptionBrand, _, _, _>(Some(Some(5)));
		/// assert_eq!(result, Some(5));
		/// ```
		fn dispatch_join(self) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>);
	}

	// -- Val: owned container -> Semimonad::bind(id) --

	/// Routes owned containers to [`Semimonad::bind`] with identity.
	impl<'a, Brand, A> JoinDispatch<'a, Brand, A, Val> for Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)>)
	where
		Brand: Semimonad,
		A: 'a,
	{
		fn dispatch_join(self) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>) {
			Brand::bind(self, |ma| ma)
		}
	}

	// -- Ref: borrowed container -> RefSemimonad::ref_bind(clone) --

	/// Routes borrowed containers to [`RefSemimonad::ref_bind`] with clone.
	impl<'a, Brand, A> JoinDispatch<'a, Brand, A, Ref> for &Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)>)
	where
		Brand: RefSemimonad,
		A: 'a,
		Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>): Clone,
	{
		fn dispatch_join(self) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>) {
			Brand::ref_bind(self, |ma| ma.clone())
		}
	}

	// -- Unified free function --

	/// Removes one layer of monadic nesting.
	///
	/// Dispatches to either [`Semimonad::bind`] with identity or
	/// [`RefSemimonad::ref_bind`] with clone, based on whether the
	/// container is owned or borrowed.
	///
	/// The `Marker` type parameter is inferred automatically by the
	/// compiler from the container argument. Callers write
	/// `join_explicit::<Brand, _>(...)` and never need to specify `Marker` explicitly.
	///
	/// The dispatch is resolved at compile time with no runtime cost.
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the monad.",
		"The type of the value(s) inside the inner layer.",
		"The container type (owned or borrowed), inferred from the argument.",
		"Dispatch marker type, inferred automatically."
	)]
	///
	#[document_parameters("The nested monadic value (owned or borrowed).")]
	///
	#[document_returns("A container with one layer of nesting removed.")]
	///
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// };
	///
	/// // Owned: dispatches via Semimonad::bind(id)
	/// let y = join_explicit::<OptionBrand, _, _, _>(Some(Some(5)));
	/// assert_eq!(y, Some(5));
	///
	/// // By-ref: dispatches via RefSemimonad::ref_bind(clone)
	/// let x = Some(Some(5));
	/// let y = join_explicit::<OptionBrand, _, _, _>(&x);
	/// assert_eq!(y, Some(5));
	/// ```
	pub fn join<'a, Brand: Kind_cdc7cd43dac7585f, A: 'a, FA, Marker>(
		mma: FA
	) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)
	where
		FA: JoinDispatch<'a, Brand, A, Marker>, {
		mma.dispatch_join()
	}
}

pub use inner::*;
