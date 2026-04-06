//! Thread-safe by-reference variant of [`Foldable`](crate::classes::Foldable).
//!
//! **User story:** "I want to fold over a thread-safe memoized value by reference."
//!
//! ### Examples
//!
//! ```
//! use fp_library::{
//! 	brands::*,
//! 	classes::send_ref_foldable::*,
//! 	types::*,
//! };
//!
//! let lazy = ArcLazy::new(|| 10);
//! let result = send_ref_fold_map::<ArcFnBrand, LazyBrand<ArcLazyConfig>, _, _, _>(
//! 	|a: &i32| a.to_string(),
//! 	lazy,
//! );
//! assert_eq!(result, "10");
//! ```

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			classes::*,
			kinds::*,
		},
		fp_macros::*,
	};

	/// Thread-safe by-reference folding over a structure.
	///
	/// Similar to [`RefFoldable`], but closures and elements must be `Send + Sync`.
	#[kind(type Of<'a, A: 'a>: 'a;)]
	pub trait SendRefFoldable: RefFoldable {
		/// Maps values to a monoid by reference and combines them (thread-safe).
		#[document_signature]
		#[document_type_parameters(
			"The lifetime of the elements.",
			"The type of the elements.",
			"The monoid type."
		)]
		#[document_parameters(
			"The function to map each element reference to a monoid. Must be `Send + Sync`.",
			"The structure to fold."
		)]
		#[document_returns("The combined monoid value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::send_ref_foldable::*,
		/// 	types::*,
		/// };
		///
		/// let lazy = ArcLazy::new(|| 5);
		/// let result = send_ref_fold_map::<ArcFnBrand, LazyBrand<ArcLazyConfig>, _, _, _>(
		/// 	|a: &i32| a.to_string(),
		/// 	lazy,
		/// );
		/// assert_eq!(result, "5");
		/// ```
		fn send_ref_fold_map<'a, A: Send + Sync + 'a, M>(
			func: impl Fn(&A) -> M + Send + Sync + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> M
		where
			M: Monoid + 'a;
	}

	/// Maps values to a monoid by reference and combines them (thread-safe).
	///
	/// Free function version that dispatches to [the type class' associated function][`SendRefFoldable::send_ref_fold_map`].
	#[document_signature]
	#[document_type_parameters(
		"The lifetime of the elements.",
		"The brand of the structure.",
		"The type of the elements.",
		"The monoid type."
	)]
	#[document_parameters(
		"The function to map each element reference to a monoid.",
		"The structure to fold."
	)]
	#[document_returns("The combined monoid value.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	classes::send_ref_foldable::*,
	/// 	types::*,
	/// };
	///
	/// let lazy = ArcLazy::new(|| 5);
	/// let result = send_ref_fold_map::<ArcFnBrand, LazyBrand<ArcLazyConfig>, _, _, _>(
	/// 	|a: &i32| a.to_string(),
	/// 	lazy,
	/// );
	/// assert_eq!(result, "5");
	/// ```
	pub fn send_ref_fold_map<'a, Brand: SendRefFoldable, A: Send + Sync + 'a, M>(
		func: impl Fn(&A) -> M + Send + Sync + 'a,
		fa: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> M
	where
		M: Monoid + 'a, {
		Brand::send_ref_fold_map(func, fa)
	}
}

pub use inner::*;
