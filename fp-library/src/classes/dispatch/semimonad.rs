//! Dispatch for [`Semimonad::bind`](crate::classes::Semimonad::bind) and
//! [`RefSemimonad::ref_bind`](crate::classes::RefSemimonad::ref_bind).
//!
//! Provides the [`BindDispatch`] trait and a unified [`bind`] free function
//! that routes to the appropriate trait method based on the closure's argument
//! type.
//!
//! ### Examples
//!
//! ```
//! use fp_library::{
//! 	brands::*,
//! 	functions::*,
//! 	types::*,
//! };
//!
//! // Owned: dispatches to Semimonad::bind
//! let result = bind::<OptionBrand, _, _, _>(Some(5), |x: i32| Some(x * 2));
//! assert_eq!(result, Some(10));
//!
//! // By-ref: dispatches to RefSemimonad::ref_bind
//! let lazy = RcLazy::pure(5);
//! let result = bind::<LazyBrand<RcLazyConfig>, _, _, _>(lazy, |x: &i32| {
//! 	Lazy::<_, RcLazyConfig>::new({
//! 		let v = *x;
//! 		move || v * 2
//! 	})
//! });
//! assert_eq!(*result.evaluate(), 10);
//! ```

#[fp_macros::document_module]
pub(crate) mod inner {
	use {
		crate::{
			classes::{
				RefSemimonad,
				Semimonad,
			},
			kinds::*,
		},
		fp_macros::*,
	};

	/// Trait that routes a bind operation to the appropriate type class method.
	///
	/// The `Marker` type parameter is inferred from the closure's argument type:
	/// `Fn(A) -> Of<B>` resolves to [`Val`](crate::classes::dispatch::Val),
	/// `Fn(&A) -> Of<B>` resolves to [`Ref`](crate::classes::dispatch::Ref).
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the monad.",
		"The type of the value inside the monad.",
		"The type of the result.",
		"Dispatch marker type, inferred automatically."
	)]
	#[document_parameters("The closure implementing this dispatch.")]
	pub trait BindDispatch<'a, Brand: Kind_cdc7cd43dac7585f, A: 'a, B: 'a, Marker> {
		/// Perform the dispatched bind operation.
		#[document_signature]
		#[document_parameters("The monadic value.")]
		#[document_returns("The result of binding.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		/// let result = bind::<OptionBrand, _, _, _>(Some(5), |x: i32| Some(x * 2));
		/// assert_eq!(result, Some(10));
		/// ```
		fn dispatch_bind(
			self,
			ma: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>);
	}

	/// Routes `Fn(A) -> Of<B>` closures to [`Semimonad::bind`].
	#[document_type_parameters(
		"The lifetime.",
		"The brand.",
		"The input type.",
		"The output type.",
		"The closure type."
	)]
	#[document_parameters("The closure that takes owned values.")]
	impl<'a, Brand, A, B, F> BindDispatch<'a, Brand, A, B, super::super::Val> for F
	where
		Brand: Semimonad,
		A: 'a,
		B: 'a,
		F: Fn(A) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) + 'a,
	{
		#[document_signature]
		#[document_parameters("The monadic value.")]
		#[document_returns("The result of binding.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		/// let result = bind::<OptionBrand, _, _, _>(Some(5), |x: i32| Some(x * 2));
		/// assert_eq!(result, Some(10));
		/// ```
		fn dispatch_bind(
			self,
			ma: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			Brand::bind(ma, self)
		}
	}

	/// Routes `Fn(&A) -> Of<B>` closures to [`RefSemimonad::ref_bind`].
	#[document_type_parameters(
		"The lifetime.",
		"The brand.",
		"The input type.",
		"The output type.",
		"The closure type."
	)]
	#[document_parameters("The closure that takes references.")]
	impl<'a, Brand, A, B, F> BindDispatch<'a, Brand, A, B, super::super::Ref> for F
	where
		Brand: RefSemimonad,
		A: 'a,
		B: 'a,
		F: Fn(&A) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) + 'a,
	{
		#[document_signature]
		#[document_parameters("The monadic value.")]
		#[document_returns("The result of binding.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		/// let lazy = RcLazy::pure(5);
		/// let result = bind::<LazyBrand<RcLazyConfig>, _, _, _>(lazy, |x: &i32| {
		/// 	Lazy::<_, RcLazyConfig>::new({
		/// 		let v = *x;
		/// 		move || v * 2
		/// 	})
		/// });
		/// assert_eq!(*result.evaluate(), 10);
		/// ```
		fn dispatch_bind(
			self,
			ma: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			Brand::ref_bind(ma, self)
		}
	}

	/// Sequences a monadic computation with a function that produces the next computation.
	///
	/// Dispatches to either [`Semimonad::bind`] or [`RefSemimonad::ref_bind`]
	/// based on the closure's argument type.
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the monad.",
		"The type of the value inside the monad.",
		"The type of the result.",
		"Dispatch marker type, inferred automatically."
	)]
	///
	#[document_parameters("The monadic value.", "The function to apply to the value.")]
	///
	#[document_returns("The result of sequencing the computation.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// };
	/// let result = bind::<OptionBrand, _, _, _>(Some(5), |x: i32| Some(x * 2));
	/// assert_eq!(result, Some(10));
	/// ```
	pub fn bind<'a, Brand: Kind_cdc7cd43dac7585f, A: 'a, B: 'a, Marker>(
		ma: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		f: impl BindDispatch<'a, Brand, A, B, Marker>,
	) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
		f.dispatch_bind(ma)
	}
}

pub use inner::*;
