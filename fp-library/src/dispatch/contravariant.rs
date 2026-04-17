//! Dispatch for [`Contravariant::contramap`](crate::classes::Contravariant::contramap).
//!
//! Provides the [`ContravariantDispatch`] trait and a unified
//! [`explicit::contramap`] free function. Unlike other dispatch modules, there
//! is no Ref variant because `contramap`'s closure produces elements
//! (`Fn(B) -> A`), not consumes them. The `&A` convention does not apply.
//!
//! ### Examples
//!
//! ```
//! use fp_library::{
//! 	brands::*,
//! 	functions::explicit::*,
//! };
//!
//! let f = std::rc::Rc::new(|x: i32| x + 1) as std::rc::Rc<dyn Fn(i32) -> i32>;
//! let g =
//! 	contramap::<ProfunctorSecondAppliedBrand<RcFnBrand, i32>, _, _, _, _>(|x: i32| x * 2, f);
//! assert_eq!(g(5), 11); // (5 * 2) + 1
//! ```

#[fp_macros::document_module]
pub(crate) mod inner {
	use {
		crate::{
			classes::Contravariant,
			dispatch::Val,
			kinds::*,
		},
		fp_macros::*,
	};

	/// Trait that routes a contramap operation to [`Contravariant::contramap`].
	///
	/// Only a Val dispatch exists. There is no Ref variant because
	/// `contramap`'s closure produces elements (`Fn(B) -> A`), not consumes
	/// them, so the `&A` convention does not apply.
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the contravariant functor.",
		"The type of the value(s) inside the contravariant functor.",
		"The type that the new contravariant functor accepts.",
		"The container type, inferred from the argument.",
		"Dispatch marker type, inferred automatically."
	)]
	#[document_parameters("The closure implementing this dispatch.")]
	pub trait ContravariantDispatch<'a, Brand: Kind_cdc7cd43dac7585f, A: 'a, B: 'a, FA, Marker> {
		/// Perform the dispatched contramap operation.
		#[document_signature]
		///
		#[document_parameters("The contravariant functor instance.")]
		///
		#[document_returns("A new contravariant functor that accepts values of type `B`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::explicit::*,
		/// };
		///
		/// let f = std::rc::Rc::new(|x: i32| x + 1) as std::rc::Rc<dyn Fn(i32) -> i32>;
		/// let g =
		/// 	contramap::<ProfunctorSecondAppliedBrand<RcFnBrand, i32>, _, _, _, _>(|x: i32| x * 2, f);
		/// assert_eq!(g(5), 11);
		/// ```
		fn dispatch(
			self,
			fa: FA,
		) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>);
	}

	// -- Val: Fn(B) -> A -> Contravariant::contramap --

	/// Routes `Fn(B) -> A` closures to [`Contravariant::contramap`].
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the contravariant functor.",
		"The type of the value(s) inside the contravariant functor.",
		"The type that the new contravariant functor accepts.",
		"The closure type."
	)]
	#[document_parameters("The closure that maps the new input type to the original.")]
	impl<'a, Brand, A, B, F>
		ContravariantDispatch<
			'a,
			Brand,
			A,
			B,
			Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			Val,
		> for F
	where
		Brand: Contravariant,
		A: 'a,
		B: 'a,
		F: Fn(B) -> A + 'a,
	{
		#[document_signature]
		///
		#[document_parameters("The contravariant functor instance.")]
		///
		#[document_returns("A new contravariant functor that accepts values of type `B`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::explicit::*,
		/// };
		///
		/// let f = std::rc::Rc::new(|x: i32| x + 1) as std::rc::Rc<dyn Fn(i32) -> i32>;
		/// let g =
		/// 	contramap::<ProfunctorSecondAppliedBrand<RcFnBrand, i32>, _, _, _, _>(|x: i32| x * 2, f);
		/// assert_eq!(g(5), 11);
		/// ```
		fn dispatch(
			self,
			fa: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			Brand::contramap(self, fa)
		}
	}

	// -- Inference wrapper --

	/// Contravariantly maps a function over a value, inferring the brand from the container type.
	///
	/// The `Brand` type parameter is inferred from the concrete type of `fa`
	/// via the `InferableBrand` trait. Only owned containers are supported; there is no
	/// `RefContravariant` trait because the Ref pattern is about closures
	/// receiving element references (`&A`), but `contramap`'s closure
	/// produces elements (`Fn(B) -> A`), not consumes them.
	///
	/// For types with multiple brands, use
	/// [`explicit::contramap`](crate::functions::explicit::contramap) with a turbofish.
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The container type. Brand is inferred from this.",
		"The type of the value(s) inside the contravariant functor.",
		"The type that the new contravariant functor accepts.",
		"The brand, inferred via InferableBrand from FA and the element type."
	)]
	///
	#[document_parameters(
		"The function mapping the new input type to the original input type.",
		"The contravariant functor instance."
	)]
	///
	#[document_returns("A new contravariant functor that accepts values of type `B`.")]
	#[document_examples]
	///
	/// This example uses [`explicit::contramap`](crate::functions::explicit::contramap)
	/// because the primary contravariant types are profunctor-based (e.g.,
	/// `Rc<dyn Fn(A) -> B>`), which have multiple possible brands depending
	/// on which type parameter varies. Brand inference cannot resolve this
	/// ambiguity, so a turbofish is required.
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::explicit::*,
	/// };
	///
	/// let f = std::rc::Rc::new(|x: i32| x + 1) as std::rc::Rc<dyn Fn(i32) -> i32>;
	/// let g =
	/// 	contramap::<ProfunctorSecondAppliedBrand<RcFnBrand, i32>, _, _, _, _>(|x: i32| x * 2, f);
	/// assert_eq!(g(5), 11);
	/// ```
	pub fn contramap<'a, FA, A: 'a, B: 'a, Brand>(
		f: impl ContravariantDispatch<
			'a,
			Brand,
			A,
			B,
			FA,
			<FA as InferableBrand_cdc7cd43dac7585f<'a, Brand, A>>::Marker,
		>,
		fa: FA,
	) -> <Brand as Kind_cdc7cd43dac7585f>::Of<'a, B>
	where
		Brand: Kind_cdc7cd43dac7585f,
		FA: InferableBrand_cdc7cd43dac7585f<'a, Brand, A>, {
		f.dispatch(fa)
	}

	// -- Explicit dispatch free function --

	/// Explicit dispatch functions requiring a Brand turbofish.
	///
	/// For most use cases, prefer the inference-enabled wrappers from
	/// [`functions`](crate::functions).
	pub mod explicit {
		use super::*;

		/// Contravariantly maps a function over a contravariant functor.
		///
		/// Dispatches to [`Contravariant::contramap`].
		///
		/// The `Marker` and `FA` type parameters are inferred automatically by the
		/// compiler. Callers write `contramap::<Brand, _, _, _, _>(...)` and never
		/// need to specify `Marker` or `FA` explicitly.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The brand of the contravariant functor.",
			"The type of the value(s) inside the contravariant functor.",
			"The type that the new contravariant functor accepts.",
			"The container type, inferred from the argument.",
			"Dispatch marker type, inferred automatically."
		)]
		///
		#[document_parameters(
			"The function mapping the new input type to the original input type.",
			"The contravariant functor instance."
		)]
		///
		#[document_returns("A new contravariant functor that accepts values of type `B`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::explicit::*,
		/// };
		///
		/// let f = std::rc::Rc::new(|x: i32| x + 1) as std::rc::Rc<dyn Fn(i32) -> i32>;
		/// let g =
		/// 	contramap::<ProfunctorSecondAppliedBrand<RcFnBrand, i32>, _, _, _, _>(|x: i32| x * 2, f);
		/// assert_eq!(g(5), 11);
		/// ```
		pub fn contramap<'a, Brand: Kind_cdc7cd43dac7585f, A: 'a, B: 'a, FA, Marker>(
			f: impl ContravariantDispatch<'a, Brand, A, B, FA, Marker>,
			fa: FA,
		) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			f.dispatch(fa)
		}
	}
}

pub use inner::*;
