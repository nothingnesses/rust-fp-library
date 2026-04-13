//! Natural transformations between type constructors.
//!
//! A natural transformation is a polymorphic function from `F<A>` to `G<A>` that
//! works uniformly for any inner type `A`. This is the Rust encoding of PureScript's
//! `type NaturalTransformation f g = forall a. f a -> g a` (also written `f ~> g`).
//!
//! ### Examples
//!
//! ```
//! use fp_library::{
//! 	brands::*,
//! 	classes::NaturalTransformation,
//! 	types::*,
//! };
//!
//! #[derive(Clone)]
//! struct ThunkToOption;
//! impl NaturalTransformation<ThunkBrand, OptionBrand> for ThunkToOption {
//! 	fn transform<'a, A: 'a>(
//! 		&self,
//! 		fa: Thunk<'a, A>,
//! 	) -> Option<A> {
//! 		Some(fa.evaluate())
//! 	}
//! }
//!
//! let nt = ThunkToOption;
//! let result = nt.transform(Thunk::new(|| 42));
//! assert_eq!(result, Some(42));
//! ```

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			Apply,
			kinds::*,
		},
		fp_macros::*,
	};

	/// A natural transformation from type constructor `F` to type constructor `G`.
	///
	/// A natural transformation is a polymorphic function `F<A> -> G<A>` that works
	/// uniformly for any inner type `A`. Unlike Haskell/PureScript where this is a simple
	/// type alias (`type f ~> g = forall a. f a -> g a`), Rust requires a trait to express
	/// rank-2 polymorphism.
	///
	/// Only the [`Kind`](crate::kinds) bound is required on `F` and `G`; no `Functor`
	/// constraint is imposed. Consumers like [`Free::fold_free`](crate::types::Free::fold_free)
	/// add their own bounds (e.g., `Functor`, `Monad`) as needed.
	///
	/// # Laws
	///
	/// For any functors `F` and `G` where `Functor::map` is defined, a natural
	/// transformation `nt` must satisfy the naturality condition:
	///
	/// `nt(map(f, fa)) == map(f, nt(fa))`
	///
	/// That is, transforming the structure and then mapping a function over the result
	/// must be the same as mapping first and then transforming. This law ensures the
	/// transformation only changes the "container," not the "content."
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	classes::NaturalTransformation,
	/// 	functions::explicit::*,
	/// 	types::*,
	/// };
	///
	/// #[derive(Clone)]
	/// struct ThunkToOption;
	/// impl NaturalTransformation<ThunkBrand, OptionBrand> for ThunkToOption {
	/// 	fn transform<'a, A: 'a>(
	/// 		&self,
	/// 		fa: Thunk<'a, A>,
	/// 	) -> Option<A> {
	/// 		Some(fa.evaluate())
	/// 	}
	/// }
	///
	/// let nt = ThunkToOption;
	/// let f = |x: i32| x * 3;
	///
	/// // Naturality: nt(map(f, fa)) == map(f, nt(fa))
	/// let lhs: Option<i32> = nt.transform(map::<ThunkBrand, _, _, _, _>(f, Thunk::new(|| 7)));
	/// let rhs: Option<i32> = map::<OptionBrand, _, _, _, _>(f, nt.transform(Thunk::new(|| 7)));
	/// assert_eq!(lhs, rhs);
	/// assert_eq!(lhs, Some(21));
	/// ```
	#[document_type_parameters(
		"The source type constructor brand.",
		"The target type constructor brand."
	)]
	#[document_parameters("The natural transformation to apply.")]
	pub trait NaturalTransformation<F: Kind_cdc7cd43dac7585f, G: Kind_cdc7cd43dac7585f> {
		/// Applies the natural transformation to a value of type `F<A>`,
		/// producing a value of type `G<A>`.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the value.",
			"The inner type being transformed over."
		)]
		///
		#[document_parameters("The value in the source type constructor.")]
		///
		#[document_returns("The transformed value in the target type constructor.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::NaturalTransformation,
		/// 	types::*,
		/// };
		///
		/// #[derive(Clone)]
		/// struct ThunkToOption;
		/// impl NaturalTransformation<ThunkBrand, OptionBrand> for ThunkToOption {
		/// 	fn transform<'a, A: 'a>(
		/// 		&self,
		/// 		fa: Thunk<'a, A>,
		/// 	) -> Option<A> {
		/// 		Some(fa.evaluate())
		/// 	}
		/// }
		///
		/// let nt = ThunkToOption;
		/// let result = nt.transform(Thunk::new(|| 42));
		/// assert_eq!(result, Some(42));
		/// ```
		fn transform<'a, A: 'a>(
			&self,
			fa: Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<G as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>);
	}
}

pub use inner::*;
